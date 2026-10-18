//! 引擎外壳：命令通道 + 单一工作副本回滚 + 确定性摘要。
//!
//! 与旧实现的差别：
//! - 目录经 `Arc` 共享，`submit` 只克隆紧凑状态（不克隆内容目录/字节码）；
//! - 宿主日志（展示/录像）在引擎侧，不进状态、不进哈希；
//! - 拒绝或预算耗尽时丢弃工作副本，状态逐字节不变。

use blake3::Hasher;

use crate::catalog::{CardKind, SharedCatalog};
use crate::choice::{ChoiceOption, OpenChoice};
use crate::content;
use crate::event::{EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::LogEvent;
use crate::resolution::{run, Resolution, StepOutcome};
use crate::rng::RuleRng;
use crate::snap::{
    bounds_from, validate_core, validate_resolution, JiJiStageSnap, SnapshotError, StateSnapshot,
};
use crate::state::{GameMode, GameState, GameStateCore, Outcome, PlayerState, TurnState};

/// 命令动作：宿主可请求的对局操作。
#[derive(Debug, Clone)]
pub enum GameAction {
    /// 出牌阶段主动使用一张牌。
    PlayCard {
        card: CardId,
        targets: Vec<PlayerId>,
    },
    /// 推进当前阶段（终局阶段则推进回合）。
    NextPhase,
    /// N11 主动技：发动一个已授予的主动技能（每回合限一次类限制由技能定义）。
    ActivateSkill { skill: crate::ids::SkillDefId },
    /// N11 国战明置门控（ADR 0757）：明置一张武将牌（0=主将 1=副将）。
    /// 仅国战模式、自己回合出牌阶段、槽已配置且未明置时合法。
    RevealGeneral { slot: u8 },
    /// 应答选择。
    Answer {
        choice_id: u64,
        selection: Vec<ChoiceOption>,
    },
}

/// 玩家命令（外部信任边界信封）。
///
/// - `actor`：命令发起者。授权校验：Answer 必须是选择观察者，
///   PlayCard/NextPhase 必须是当前回合玩家；越界 actor 拒绝。
/// - `expected_revision`：客户端已知的最新状态修订号，不符即拒绝
///   （过期命令不得作用于已前进的状态）。
/// - `dedup_id`：命令去重键。必须为正且在**已被接受**的命令中严格递增
///   （高水位语义）；被拒绝的命令不消耗键，允许同键换修订号重试。
#[derive(Debug, Clone)]
pub struct GameCommand {
    pub actor: PlayerId,
    pub expected_revision: u64,
    pub dedup_id: u64,
    pub action: GameAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    ChoicePending,
    NoSuchChoice,
    InvalidChoice,
    NotYourTurn,
    WrongPhase,
    CardNotInHand,
    IllegalTargets,
    ShaLimit,
    HpFull,
    MatchOver,
    BudgetExhausted,
    /// actor 不是该命令的合法发起者（如应答他人的选择）。
    WrongPlayer,
    /// actor 超出座位范围。
    UnknownActor,
    /// expected_revision 与当前状态修订号不符。
    StaleRevision,
    /// dedup_id 缺失或不超过已接受命令的高水位。
    DuplicateCommand,
    /// 每回合限一次的牌本回合已使用（如酒）。
    CardLimit,
    /// 段因时间片暂停中：须先 `resume` 完成本段再提交命令。
    SegmentSuspended,
    /// 段未处于暂停状态，无续行可做。
    NotSuspended,
    /// 出牌封锁（swd 批 134 冰封·封）：该玩家不能使用牌。
    CardBlocked,
    /// N10 有界计算池：该座位本轮命令配额已用尽（宿主策略，默认不限制）。
    QuotaExhausted,
    /// N11 认证：令牌不存在或已失效（会话数据不入快照）。
    NoSuchSession,
    /// N11 认证：令牌绑定的座位与命令 actor 不符（冒充）。
    Impersonation,
    /// N11 主动技：技能未授予。
    NoSuchSkill,
    /// N11 主动技：技能已禁用。
    SkillDisabled,
    /// N11 主动技：每回合限一次的技能本回合已发动。
    SkillAlreadyUsed,
    /// N11 国战明置门控：命令仅限特定对局模式（如明置仅国战可发）。
    WrongMode,
    /// N11 国战明置门控：目标武将槽未配置武将。
    NoSuchGeneral,
    /// N11 国战明置门控：目标武将牌已处于明置状态。
    AlreadyRevealed,
    /// NC04：VM 执行故障（fuel/栈/内存/能力/校验等）——该段已整段回滚。
    VmFault {
        /// 触发故障的 VM 技能名（可空）。
        skill: String,
        /// 错误类别与详情（含触发事件）。
        detail: String,
    },
}

#[derive(Debug, Clone)]
pub enum CommandOutcome {
    /// 命令完整结算。
    Accepted { state_hash: [u8; 32] },
    /// 结算暂停于一次选择。
    Pending { choice: OpenChoice },
    /// 命令被拒绝；状态不变。
    Rejected(RejectReason),
    /// 宿主时间片耗尽：本段暂停在一致的步边界（状态已提交、无开放选择），
    /// 以 `resume()` 续行；续行至 Accepted/Pending/Rejected 结束。
    Suspended,
}

/// 无界面对局引擎。
/// N11 认证：会话令牌——由权威服务签发、与座位绑定的 opaque 凭证。
/// 宿主只能持有与比较令牌，不能凭空构造有效令牌（测试可构造无效令牌，
/// 内核以签发表为准拒绝）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthToken {
    pub seat: PlayerId,
    pub id: u64,
}

impl AuthToken {
    /// 令牌绑定的座位（仅诊断用途；内核以内部绑定表为准）。
    pub fn seat(&self) -> PlayerId {
        self.seat
    }
}

#[derive(Clone)]
pub struct Engine {
    catalog: SharedCatalog,
    state: GameState,
    log: Vec<LogEvent>,
    /// 命令去重高水位：已接受命令的最大 dedup_id（会话数据，不入状态/哈希）。
    pub(crate) last_dedup: u64,
    /// 宿主时间片（None = 不限）：每个命令段最多推进的调度步数，
    /// 用尽即 `Suspended` 暂停（状态已提交），`resume()` 无损续行。
    time_slice: Option<u32>,
    /// 段暂停中：段已部分结算并提交，须 `resume()` 完成后才能再提交命令。
    segment_open: bool,
    /// 调试预算覆盖（Some(b) = 下一段以 b 为硬上限；不入状态/哈希/快照）。
    debug_budget: Option<u32>,
    /// 段检查点：段启动时的（快照, 日志水位）。段开放期间保留，
    /// `abort_segment()` 据此回滚到段前；段关闭即清除。
    segment_checkpoint: Option<(StateSnapshot, usize)>,
    /// N10 有界计算池：每座位每轮命令配额（None = 不限制；
    /// 会话策略数据，不入状态/哈希/快照）。
    seat_quota: Option<u32>,
    /// 座位命令计数：(座位 → (轮次, 已接受命令数))；轮次变更自动归零。
    seat_cmd_counts: std::collections::BTreeMap<PlayerId, (u32, u32)>,
    /// N11 认证：座位 → 会话令牌（宿主会话数据，不入状态/哈希/快照）。
    auth_tokens: std::collections::BTreeMap<PlayerId, u64>,
    auth_counter: u64,
    /// N11-a：实例熵（OS 随机，不入快照）——跨实例令牌空间隔离。
    auth_secret: u64,
    /// N11-a：令牌代次（随快照延续；恢复时 +1）——恢复前旧令牌失效。
    auth_generation: u64,
}

const DEFAULT_BUDGET: u32 = 4096;

/// N11-a：实例熵源——`RandomState` 每实例随机种子派生（进程内唯一，
/// 无需密码学强度；令牌仅为宿主会话绑定）。
fn fresh_auth_secret() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut h = RandomState::new().build_hasher();
    h.write_u64(0x4E31_32A1_1CE5_C0_u64);
    h.finish()
}

/// 命令验证与入栈（在提交阶段之前，任何失败都不产生副作用）。
///
/// 本函数是外部输入的最后一道边界：actor 身份、目标/牌 ID 越界、
/// 规则合法性全部在此校验，任何路径不得 panic。
/// 装配授予（P2/N08）：按武将表配置体力上限（主将槽）并授予技能集。
/// 国战明置（`RevealGeneral`，ADR 0769）与原型装配原语
/// （`debug_assemble_general`）共用此逻辑，授予语义不得分叉。
/// 开局计划（建局面，引擎私有）：发牌数与回合起点。斗地主 binglin 与
/// Boss 关卡（gameDraw/loopFirst，规则 v207）两种变体；变体均为建局输
/// 入不入模式编码（开局手牌数与回合起点已进状态与哈希）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpeningPlan {
    /// 标准：各 4 张、回合起点座位 0。
    Standard,
    /// 斗地主 binglin：地主 5/农民 4。
    DoudizhuBinglin,
    /// Boss 关卡：boss/挑战者分列发牌数 + 回合起点座位（上游
    /// bossinfo.gameDraw/loopFirst 以显式参数承载）。
    BossStage {
        boss_cards: u32,
        player_cards: u32,
        first: PlayerId,
    },
}

pub(crate) fn apply_general_assembly(
    state: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    slot: usize,
) {
    let g = state.players[player as usize].general_slots[slot].expect("general slot configured");
    let meta = catalog.general_meta(g);
    let p = state.players.get_mut(player as usize).expect("player");
    if slot == 0 {
        if let Some((_, _, _, hp, _)) = meta {
            p.max_hp = hp;
            if p.hp > hp as i32 {
                p.hp = hp as i32;
            }
        }
    }
    let skills: Vec<&'static str> = meta.map(|(_, _, skills, _, _)| skills).unwrap_or_default();
    // 禳星（T11 Boss 诸葛亮族批 115，规则 v303）：装配即初始化星系统
    //（上游 `hlss_xiangxing` init：3 枚星、计数 0、无受伤记录）。
    if skills.contains(&crate::content::host::HLSS_XIANGXING_KEY) {
        let p = state.players.get_mut(player as usize).expect("player");
        p.hlss_stars = 3;
        p.hlss_count = 0;
        p.hlss_damage = 0;
    }
    for skill in skills {
        let def = catalog
            .skill_by_name(skill)
            .unwrap_or_else(|| panic!("catalog skill {skill}"));
        // 隐贼/延妒（Boss 混沌/穷奇，规则 v210/v213）：装配即未激活
        //（上游 gameStart switch_off disableSkill 同口径；体力首次≤
        // 一半时启用）
        let enabled = skill != crate::content::standard::BOSS_YINZEI
            && skill != crate::content::standard::BOSS_YANDU
            && skill != crate::content::standard::BOSS_JICAI
            && skill != crate::content::standard::BOSS_LUANCHANG
            && skill != crate::content::standard::BOSS_ECHOU
            && skill != crate::content::standard::BOSS_XUSHI
            && skill != crate::content::standard::BOSS_YANYU
            && skill != crate::content::standard::BOSS_SIPU;
        state.skills.insert(
            (player, def),
            crate::state::SkillInstance {
                owner: player,
                enabled,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::DamageNature;
    use crate::kernel::ShaResolveMachine;

    /// 挂起机器的局部状态必须参与摘要：同一可见状态、同一机器类型、
    /// 仅内部字段（asked）不同 → 哈希不同。
    #[test]
    fn park_machine_state_changes_digest() {
        let mk_state = |asked: bool, armor_judged: bool| {
            let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
            let mut core = GameStateCore {
                revision: 0,
                ruleset_version: catalog.ruleset_version,
                mode: GameMode::Prototype,
                players: (0..2).map(|_| PlayerState::new(4)).collect(),
                seat_ring: (0..2).collect(),
                deck: Vec::new(),
                discard: Vec::new(),
                table: Vec::new(),
                rng: RuleRng::from_seed(1),
                turn: TurnState {
                    current: 0,
                    phase: Phase::Play,
                    round: 1,
                    sha_used_play_phase: false,
                    mufeng_used_basic: false,
                    wangchen_discard_basic: false,
                    shenmiguo_used: false,
                    shuchui_armed: false,
                    shuchui_hit: false,
                    lianda_used: false,
                    mengtai_draw_pending: false,
                    mengtai_use_pending: false,
                    rende_given: 0,
                    jiu_used: false,
                    mujiaren_used: false,
                    jiu_active: false,
                    turn_opened: true,
                    skip_draw_phase: false,
                    skip_play_phase: false,
                    guxing_armed: false,
                    guxing_count: 0,
                    zhufangshenshi_target: None,
                    yuedong_num: 0,
                    yuedong_eff: 0,
                    yuedong_recover: false,
                    liangou_hook: None,
                    liangou_bonus_used: false,
                    feitiao_target: None,
                    fanling_used: false,
                    xjyufeng_used: 0,
                    diewu_draw_used: false,
                    lueying_used: false,
                    txianqu_used: false,
                    jtjubao_used: false,
                    zhuyue_targets: Vec::new(),
                    sheling_used: 0,
                    sheying_used: false,
                    zhangmu_used: false,
                    yingfeng_chain: false,
                    luanjian_card: None,
                    minwan_targets: Vec::new(),
                    yanyu_count: 0,
                    qizuo_copy: None,
                    jidian_chain: false,
                    land_used: false,
                    land_sha_used: false,
                    discard_phase_discarded: 0,
                    ciqiu_dying: Vec::new(),
                    kangyin2: None,
                    huanxia_track: Vec::new(),
                    cards_used_play_phase: 0,
                    cards_used_turn: 0,
                    liaoyuan_count: 0,
                    xuanyuanjian_marked: false,
                    kunlunjing_snapshot: Vec::new(),
                    kunlunjing_hp: 0,
                    extra_phases: Vec::new(),
                    extra_phase_lingwu: false,
                    bte_used_play_phase: 0,
                    phase_number: 0,
                },
                skills: Default::default(),
                declared_kinds: Default::default(),
                declared_slots: Default::default(),
                card_natures: Default::default(),
                current_land: None,
                outcome: None,
                vm_aux: crate::state::VmAux::default(),
            };
            core.deck = (0..catalog.cards.len() as CardId).collect();
            let resolution = Resolution {
                parks: vec![crate::resolution::Park {
                    resume_depth: 0,
                    slot: crate::resolution::Slot::Intrinsic { frame: 1 },
                    machine: Box::new(ShaResolveMachine::new(
                        0,
                        0,
                        1,
                        DamageNature::Normal,
                        asked,
                        armor_judged,
                        0,
                        None,
                        false,
                        0,
                        None,
                        None,
                        None,
                        false,
                        false,
                        None,
                        false,
                        false,
                        false,
                        Vec::new(),
                        false,
                        false,
                        false,
                        Vec::new(),
                        0,
                        false,
                        false,
                        false,
                        false,
                        None,
                        false,
                        false,
                        None,
                        false,
                        false,
                        false,
                    )),
                }],
                ..Resolution::new(64)
            };
            GameState { core, resolution }
        };
        let fresh = canonical_hash(&mk_state(false, false));
        let resumed = canonical_hash(&mk_state(true, false));
        assert_ne!(fresh, resumed, "机器局部状态不同必须得到不同摘要");
        // armor_judged（八卦阵判定标志）同样参与摘要
        let armor = canonical_hash(&mk_state(false, true));
        assert_ne!(fresh, armor, "armor_judged 必须参与摘要");
    }

    /// 规则版本参与摘要：其余全同、版本不同 → 哈希不同。
    #[test]
    fn ruleset_version_changes_digest() {
        let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
        let mk = |version: u64| GameState {
            core: GameStateCore {
                revision: 0,
                ruleset_version: version,
                mode: GameMode::Prototype,
                players: (0..2).map(|_| PlayerState::new(4)).collect(),
                seat_ring: (0..2).collect(),
                deck: Vec::new(),
                discard: Vec::new(),
                table: Vec::new(),
                rng: RuleRng::from_seed(1),
                turn: TurnState {
                    current: 0,
                    phase: Phase::Play,
                    round: 1,
                    sha_used_play_phase: false,
                    mufeng_used_basic: false,
                    wangchen_discard_basic: false,
                    shenmiguo_used: false,
                    shuchui_armed: false,
                    shuchui_hit: false,
                    lianda_used: false,
                    mengtai_draw_pending: false,
                    mengtai_use_pending: false,
                    rende_given: 0,
                    jiu_used: false,
                    mujiaren_used: false,
                    jiu_active: false,
                    turn_opened: true,
                    skip_draw_phase: false,
                    skip_play_phase: false,
                    guxing_armed: false,
                    guxing_count: 0,
                    zhufangshenshi_target: None,
                    yuedong_num: 0,
                    yuedong_eff: 0,
                    yuedong_recover: false,
                    liangou_hook: None,
                    liangou_bonus_used: false,
                    feitiao_target: None,
                    fanling_used: false,
                    xjyufeng_used: 0,
                    diewu_draw_used: false,
                    lueying_used: false,
                    txianqu_used: false,
                    jtjubao_used: false,
                    zhuyue_targets: Vec::new(),
                    sheling_used: 0,
                    sheying_used: false,
                    zhangmu_used: false,
                    yingfeng_chain: false,
                    luanjian_card: None,
                    minwan_targets: Vec::new(),
                    yanyu_count: 0,
                    qizuo_copy: None,
                    jidian_chain: false,
                    land_used: false,
                    land_sha_used: false,
                    discard_phase_discarded: 0,
                    ciqiu_dying: Vec::new(),
                    kangyin2: None,
                    huanxia_track: Vec::new(),
                    cards_used_play_phase: 0,
                    cards_used_turn: 0,
                    liaoyuan_count: 0,
                    xuanyuanjian_marked: false,
                    kunlunjing_snapshot: Vec::new(),
                    kunlunjing_hp: 0,
                    extra_phases: Vec::new(),
                    extra_phase_lingwu: false,
                    bte_used_play_phase: 0,
                    phase_number: 0,
                },
                skills: Default::default(),
                declared_kinds: Default::default(),
                declared_slots: Default::default(),
                card_natures: Default::default(),
                current_land: None,
                outcome: None,
                vm_aux: crate::state::VmAux::default(),
            },
            resolution: Resolution::new(64),
        };
        assert_ne!(canonical_hash(&mk(0)), canonical_hash(&mk(1)));
        let _ = catalog;
    }
}

mod accessors;
mod apply_command;
mod commands;
mod construction;
mod debug;
mod sessions;
mod snapshot;
mod views;
use apply_command::*;
mod boss_campaign;
mod general_select;
mod hash;
pub mod legal_actions;
use hash::*;
mod play_card;
use play_card::*;
mod play_sha;
use play_sha::*;
