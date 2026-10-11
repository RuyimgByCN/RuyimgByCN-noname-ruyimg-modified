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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Engine {
    /// 开一局 `seats` 人原型对局（存活数 ≤ 1 终局），每人 4 点体力、摸 4 张牌；
    /// 种子决定牌库顺序。
    pub fn new(seed: u64, seats: usize) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Prototype, OpeningPlan::Standard)
    }

    /// 开一局 `seats` 人身份局（4–8 人）。座位 0 为主公（体力上限 +1），
    /// 其余身份按人数表分配并以规则随机流洗序；主忠反内胜负与奖惩见
    /// `GameMode::Identity` 与覆盖清单。人数不合法返回 `Err`
    /// （外部建局接口不依赖 panic；`new_identity` 为断言便捷封装）。
    pub fn try_new_identity(seed: u64, seats: usize) -> Result<Self, String> {
        if !(4..=8).contains(&seats) {
            return Err(format!(
                "identity mode supports 4-8 seats, got {seats}"
            ));
        }
        Ok(Self::new_identity_unchecked(seed, seats))
    }

    /// `try_new_identity` 的断言封装（内部与测试便捷用）。
    pub fn new_identity(seed: u64, seats: usize) -> Self {
        assert!(
            (4..=8).contains(&seats),
            "identity mode supports 4-8 seats, got {seats}"
        );
        Self::new_identity_unchecked(seed, seats)
    }

    fn new_identity_unchecked(seed: u64, seats: usize) -> Self {
        let loyalists = match seats {
            4 | 5 | 6 => 1,
            _ => 2,
        };
        let mut roles = vec![crate::state::Role::Lord];
        for _ in 0..loyalists {
            roles.push(crate::state::Role::Loyalist);
        }
        while roles.len() < seats - 1 {
            roles.push(crate::state::Role::Rebel);
        }
        roles.push(crate::state::Role::Traitor);
        debug_assert_eq!(roles.len(), seats);
        Self::new_with_mode(seed, seats, GameMode::Identity { roles }, OpeningPlan::Standard)
    }

    /// KOF 引擎层（P2/N08 模式入口批四）：2 人局，携带会话将池/比分/局数。
    pub fn new_kof(seed: u64, pools: [Vec<u16>; 2], wins: [u32; 2], games: u32) -> Self {
        Self::new_with_mode(seed, 2, GameMode::Kof { pools, wins, games }, OpeningPlan::Standard)
    }

    /// Boss（P2/N08 模式入口批三-a）：boss 1 名对挑战者，chongzheng=重整阈值（0=无复活）。
    pub fn new_boss(seed: u64, seats: usize, boss: PlayerId, chongzheng: u8) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Boss { boss, chongzheng, fellows: Vec::new() }, OpeningPlan::Standard)
    }

    /// Boss 助战中途入队（批三-b，规则 v84）：座位表队尾追加新玩家
    /// （登记差异：上游 addFellow 可按位置插入，原生固定队尾），
    /// 直授 4 张手牌（上游 directgain(4) 同），登记入 fellows。
    /// 仅 Boss 模式；返回新座位。
    pub fn debug_add_fellow(&mut self, hp: u32) -> Option<PlayerId> {
        if !matches!(&self.state.core.mode, GameMode::Boss { .. }) {
            return None;
        }
        let pid = self.state.core.players.len() as u8;
        let mut p = crate::state::PlayerState::new(hp);
        let take = self.state.core.deck.len().min(4);
        p.hand.extend(self.state.core.deck.drain(..take));
        self.state.core.players.push(p);
        if let GameMode::Boss { boss, fellows, .. } = &mut self.state.core.mode {
            fellows.push(pid);
            let _ = boss;
        }
        Some(pid)
    }

    /// 斗地主（P2/N08 模式入口批二）：地主 1 名，其余为农民。
    pub fn new_doudizhu(seed: u64, seats: usize, lord: PlayerId) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Doudizhu { lord }, OpeningPlan::Standard)
    }

    /// 斗地主 binglin 变体（T11 模式变体，规则 v205）：开局地主 5 张/
    /// 农民 4 张（上游 gameDraw num 覆盖同口径；replacePile 换牌堆登记
    /// 不建模，用标准牌堆）。变体仅影响初始发牌，不入模式编码。
    pub fn new_doudizhu_binglin(seed: u64, seats: usize, lord: PlayerId) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Doudizhu { lord },
            OpeningPlan::DoudizhuBinglin,
        )
    }

    /// Boss 关卡骨架（T11 模式变体，规则 v207）：按关卡配置开局——
    /// boss/挑战者分列发牌数（上游 bossinfo.gameDraw，如穷奇 boss 8/
    /// 玩家 4）与回合起点座位（上游 bossinfo.loopFirst 以显式座位承载，
    /// 越界回退 0）。变体为建局输入不入模式编码（同斗地主 v205 先例）。
    #[allow(clippy::too_many_arguments)]
    pub fn new_boss_stage(
        seed: u64,
        seats: usize,
        boss: PlayerId,
        chongzheng: u8,
        boss_cards: u32,
        player_cards: u32,
        first: PlayerId,
    ) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Boss {
                boss,
                chongzheng,
                fellows: Vec::new(),
            },
            OpeningPlan::BossStage {
                boss_cards,
                player_cards,
                first,
            },
        )
    }

    /// 对决 2v2 基线（P2/N08 模式入口批一）：`teams` 两队互斥并集=全部座位。
    pub fn new_versus(seed: u64, teams: [Vec<PlayerId>; 2]) -> Self {
        let subs = vec![Vec::new(), Vec::new()];
        Self::new_versus_with_substitutes(seed, teams, subs)
    }

    /// 对决替补变体（T11 模式变体，规则 v206）：每队携带替补武将池
    /// （与队伍同队序）——死亡座位从本队池弹队首换将复活（上游
    /// revive+uninit/init+摸 4+濒死链取消同口径；奖惩不结算登记差异）。
    pub fn new_versus_with_substitutes(
        seed: u64,
        teams: [Vec<PlayerId>; 2],
        substitutes: Vec<Vec<u16>>,
    ) -> Self {
        let seats = teams[0].len() + teams[1].len();
        assert_eq!(
            substitutes.len(),
            2,
            "替补池须与队伍同队序（每队一列）"
        );
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Versus {
                teams: teams.to_vec(),
                substitutes,
            },
            OpeningPlan::Standard,
        )
    }

    fn new_with_mode(
        seed: u64,
        seats: usize,
        mode: GameMode,
        opening: OpeningPlan,
    ) -> Self {
        let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
        Self::new_with_catalog(seed, seats, mode, catalog, opening)
    }

    /// 带扩展目录的建局（E03 内容锁接线）：目录由调用方按启用扩展
    /// 合并构建（`standard_catalog_with_extensions`）。
    fn new_with_catalog(
        seed: u64,
        seats: usize,
        mut mode: GameMode,
        catalog: SharedCatalog,
        opening: OpeningPlan,
    ) -> Self {
        let mut core = GameStateCore {
            revision: 0,
            ruleset_version: catalog.ruleset_version,
            mode: GameMode::Prototype,
            players: (0..seats).map(|_| PlayerState::new(4)).collect(),
            seat_ring: (0..seats).map(|i| i as PlayerId).collect(),
            deck: Vec::new(),
            discard: Vec::new(),
            table: Vec::new(),
            rng: RuleRng::from_seed(seed),
            turn: TurnState {
                current: 0,
                phase: Phase::Judge,
                round: 1,
                sha_used_play_phase: false,
                    rende_given: 0,
                    jiu_used: false,
                mujiaren_used: false,
                    jiu_active: false,
                turn_opened: true,
                skip_draw_phase: false,
                skip_play_phase: false,
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
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                cards_used_play_phase: 0,
                phase_number: 0,
            },
            skills: Default::default(),
            declared_kinds: Default::default(),
                card_natures: Default::default(),
                current_land: None,
            outcome: None,
        };
        // 身份局：其余身份洗序（种子决定），主公（座位 0）体力上限 +1。
        if let GameMode::Identity { roles } = &mut mode {
            core.rng.shuffle(&mut roles[1..]);
            let lord = roles
                .iter()
                .position(|r| *r == crate::state::Role::Lord)
                .expect("lord exists");
            let p = &mut core.players[lord];
            p.max_hp += 1;
            p.hp = p.max_hp as i32;
        }
        core.mode = mode;
        // 开局摸牌数表（v205 斗地主变体）：binglin 地主 5/农民 4（上游
        // gameDraw num 覆盖）；其余模式各 4 张。先于 core move 计算。
        let opening_cards: Vec<u32> = (0..seats as PlayerId)
            .map(|p| match (&core.mode, &opening) {
                (GameMode::Doudizhu { lord }, OpeningPlan::DoudizhuBinglin) => {
                    if p == *lord {
                        5
                    } else {
                        4
                    }
                }
                (
                    GameMode::Boss { boss, .. },
                    OpeningPlan::BossStage {
                        boss_cards,
                        player_cards,
                        ..
                    },
                ) if p == *boss => *boss_cards,
                (
                    GameMode::Boss { .. },
                    OpeningPlan::BossStage {
                        player_cards, ..
                    },
                ) => *player_cards,
                _ => 4,
            })
            .collect();
        // 牌库 = 目录牌序，开局用规则随机流洗牌。
        core.deck = (0..catalog.cards.len() as CardId).collect();
        core.rng.shuffle(&mut core.deck);
        let mut state = GameState {
            core,
            resolution: Resolution::new(DEFAULT_BUDGET),
        };
        // Boss 关卡回合起点（v207：loopFirst 以显式座位承载；越界回退 0）
        if let (GameMode::Boss { .. }, OpeningPlan::BossStage { first, .. }) =
            (&state.core.mode, opening)
        {
            state.core.turn.current = if (first as usize) < seats {
                first
            } else {
                0
            };
        }
        for (p, cards) in (0..seats as PlayerId).zip(opening_cards) {
            for _ in 0..cards {
                let card = state.core.deck.pop().expect("deck has cards");
                state.core.player_mut(p).hand.push(card);
            }
        }
        Self {
            catalog,
            state,
            log: Vec::new(),
            last_dedup: 0,
            time_slice: None,
            seat_quota: None,
            seat_cmd_counts: Default::default(),
            auth_tokens: Default::default(),
            auth_counter: 0,
            auth_secret: fresh_auth_secret(),
            auth_generation: 0,
            segment_open: false,
            debug_budget: None,
            segment_checkpoint: None,
        }
    }

    pub fn catalog(&self) -> &SharedCatalog {
        &self.catalog
    }

    pub fn log(&self) -> &[LogEvent] {
        &self.log
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.state.core.outcome.as_ref()
    }

    pub fn turn(&self) -> &TurnState {
        &self.state.core.turn
    }

    /// 身份局的身份表（座位序；非身份局为 None）。
    pub fn roles(&self) -> Option<&[crate::state::Role]> {
        self.state.core.identity_roles()
    }

    /// 当前状态修订号：客户端提交下一条命令时回填 `expected_revision`。
    pub fn revision(&self) -> u64 {
        self.state.core.revision
    }

    /// 本目录的规则版本（与状态内 `ruleset_version` 一致）。
    pub fn ruleset_version(&self) -> u64 {
        self.catalog.ruleset_version
    }

    pub fn open_choice(&self) -> Option<&OpenChoice> {
        self.state.resolution.open_choice.as_ref()
    }

    pub fn state_hash(&self) -> [u8; 32] {
        canonical_hash(&self.state)
    }

    /// 目录指针稳定性探针：命令前后应一致（目录不随状态克隆）。
    pub fn catalog_ptr(&self) -> *const crate::catalog::Catalog {
        std::ptr::from_ref(self.catalog.as_ref())
    }

    /// 保存对局：含挂起机器局部状态的完整快照（规则版本绑定）。
    ///
    /// 快照可在任意静止点（Accepted/Pending 之后）获取；宿主日志不进快照，
    /// 恢复后的引擎从空日志开始。字节级落盘属平台/持久化阶段（见进度文档）。
    /// 测试/工具：可变核心状态访问（供 ai 模块信息集重采样用）。
    pub(crate) fn core_mut(&mut self) -> &mut crate::state::GameStateCore {
        &mut self.state.core
    }

    pub fn save(&self) -> StateSnapshot {
        StateSnapshot {
            ruleset_version: self.state.core.ruleset_version,
            auth_generation: self.auth_generation,
            core: self.state.core.clone(),
            resolution: crate::snap::ResolutionSnapshot {
                next_event_id: self.state.resolution.next_event_id,
                next_choice_id: self.state.resolution.next_choice_id,
                stack: self.state.resolution.stack.clone(),
                parks: self
                    .state
                    .resolution
                    .parks
                    .iter()
                    .map(|p| crate::snap::ParkSnapshot {
                        resume_depth: p.resume_depth,
                        slot: p.slot,
                        machine: p.machine.snapshot(),
                    })
                    .collect(),
                open_choice: self.state.resolution.open_choice.clone(),
                answer: self.state.resolution.answer.clone(),
                budget: self.state.resolution.budget,
            },
            last_dedup: self.last_dedup,
            segment_open: self.segment_open,
            segment_checkpoint: self
                .segment_checkpoint
                .as_ref()
                .map(|(snap, _)| Box::new(snap.clone())),
        }
    }

    /// 恢复对局：版本不符或快照含越界 ID 时拒绝，不产出引擎。
    pub fn restore(snapshot: &StateSnapshot) -> Result<Self, SnapshotError> {
        let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
        Self::restore_with_catalog(snapshot, catalog)
    }

    /// 带扩展目录的恢复（E03 内容锁接线）：按启用扩展集合重建目录，
    /// 版本不一致经既有 RuleVersionMismatch 机制拒绝——存档只能用
    /// 同一扩展集合（名称+版本指纹）恢复。
    pub fn restore_with_extensions(
        snapshot: &StateSnapshot,
        packs: &[&crate::content::extension::ExtensionPack],
    ) -> Result<Self, SnapshotError> {
        let catalog = content::standard::standard_catalog_with_extensions(packs)
            .map_err(|_| SnapshotError::UnknownMachine)?;
        let catalog: SharedCatalog = std::sync::Arc::new(catalog);
        Self::restore_with_catalog(snapshot, catalog)
    }


    /// 带扩展的建局（E03）：启用扩展卡牌经冲突校验合并入目录——
    /// 牌库/发牌/牌类查询全部走合并后目录；目录版本绑定扩展集合。
    pub fn new_with_extensions(
        seed: u64,
        seats: usize,
        packs: &[&crate::content::extension::ExtensionPack],
    ) -> Result<Self, crate::content::extension::ExtensionError> {
        let catalog = content::standard::standard_catalog_with_extensions(packs)?;
        let catalog: SharedCatalog = std::sync::Arc::new(catalog);
        let mut e = Self::new_with_catalog(
            seed,
            seats,
            GameMode::Prototype,
            catalog,
            OpeningPlan::Standard,
        );
        // G1 wuxing 初始化钩子：每名角色随机五行属性 + 牌堆按 1/3 比例
        // 随机获得属性（规则随机性使用引擎 RNG，保持可重放）。
        if packs.iter().any(|p| p.enabled && p.init_hook == crate::content::extension::InitHook::WuxingNatures) {
            let core = &mut e.state.core;
            for pl in core.players.iter_mut() {
                if pl.alive {
                    pl.wunature = Some(core.rng.gen_range(5) as u8);
                }
            }
            let deck_len = core.deck.len();
            for i in 0..deck_len {
                if core.rng.gen_range(3) == 0 {
                    let card = core.deck[i];
                    let nature = core.rng.gen_range(5) as u8;
                    core.card_natures.insert(card, nature);
                }
            }
            // 生克技能授予全部存活角色（上游 `_` 前缀全局技能语义）
            let shengke: Vec<crate::ids::SkillDefId> = e
                .catalog
                .skills
                .iter()
                .enumerate()
                .filter(|(_, def)| {
                    def.behavior.as_host_behavior()
                        == Some(crate::content::host::HostBehavior::ShengkeCheck)
                })
                .map(|(i, _)| i as crate::ids::SkillDefId)
                .collect();
            for def in shengke {
                for p in 0..seats as crate::ids::PlayerId {
                    e.state.core.skills.insert(
                        (p, def),
                        crate::state::SkillInstance {
                            owner: p,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
        }
        // G1 古剑·金莲烛：伤害前置事件取消监听技授予全体角色——
        // 持有金莲烛时才经过滤器生效（上游 global damageBefore 触发
        // + chooseToUse 语义的确定性化）。
        if packs.iter().any(|p| {
            p.enabled
                && p.host_skills
                    .iter()
                    .any(|d| d.behavior == crate::content::host::HostBehavior::JinlianzhuCancel)
        }) {
            let listeners: Vec<crate::ids::SkillDefId> = e
                .catalog
                .skills
                .iter()
                .enumerate()
                .filter(|(_, def)| {
                    def.behavior.as_host_behavior()
                        == Some(crate::content::host::HostBehavior::JinlianzhuCancel)
                })
                .map(|(i, _)| i as crate::ids::SkillDefId)
                .collect();
            for def in listeners {
                for p in 0..seats as crate::ids::PlayerId {
                    e.state.core.skills.insert(
                        (p, def),
                        crate::state::SkillInstance {
                            owner: p,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
        }
        Ok(e)
    }

    fn restore_with_catalog(snapshot: &StateSnapshot, catalog: SharedCatalog) -> Result<Self, SnapshotError> {
        if snapshot.ruleset_version != catalog.ruleset_version {
            return Err(SnapshotError::RuleVersionMismatch {
                snapshot: snapshot.ruleset_version,
                catalog: catalog.ruleset_version,
            });
        }
        let bounds = bounds_from(&catalog, snapshot.core.players.len());
        validate_core(&snapshot.core, &bounds)?;
        // N04-f：牌归属多重集——每张牌恰在一个牌区，机器持牌登记在界
        crate::snap::validate_card_ownership(&snapshot.core, &snapshot.resolution, &bounds)?;
        // 栈帧父子/帧号水位、挂起深度与槽位帧引用、选择/应答 ID 与机器树全部校验。
        validate_resolution(&snapshot.resolution, &bounds)?;
        // 段开放必须携带检查点（N04）：无检查点的开放段快照恢复后
        // 无法中止，属非法状态而非「恢复后再报错」。
        if snapshot.segment_open && snapshot.segment_checkpoint.is_none() {
            return Err(SnapshotError::IdOutOfBounds {
                what: "checkpoint_missing_for_open_segment",
                got: 0,
            });
        }
        // 嵌套段检查点同样校验（N07-b：恢复件必须保留可用的中止能力）。
        if let Some(ckpt) = &snapshot.segment_checkpoint {
            if ckpt.ruleset_version != catalog.ruleset_version {
                return Err(SnapshotError::RuleVersionMismatch {
                    snapshot: ckpt.ruleset_version,
                    catalog: catalog.ruleset_version,
                });
            }
            // 层级一致性（N04）：嵌套检查点自身不得再嵌套（禁止无限层级）。
            if ckpt.segment_checkpoint.is_some() {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_nested",
                    got: 0,
                });
            }
            // 检查点与当前段关联（N04）：检查点是段启动时的静止快照——
            // 座位数/模式必须与当前一致；revision 与去重高水位单调
            //（检查点不得超前当前）；自身不得处于段开放状态。
            if ckpt.core.players.len() != snapshot.core.players.len() {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_players_mismatch",
                    got: ckpt.core.players.len() as u64,
                });
            }
            if ckpt.core.mode != snapshot.core.mode {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_mode_mismatch",
                    got: 0,
                });
            }
            if ckpt.core.revision > snapshot.core.revision {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_revision_ahead",
                    got: ckpt.core.revision,
                });
            }
            if ckpt.last_dedup > snapshot.last_dedup {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_dedup_ahead",
                    got: ckpt.last_dedup,
                });
            }
            if ckpt.segment_open {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_segment_open",
                    got: 1,
                });
            }
            validate_core(&ckpt.core, &bounds)?;
            validate_resolution(&ckpt.resolution, &bounds)?;
        }
        // 选择承接关系（N04-e）：开放选择必须恰好与某个续行机器的
        // 待答询问一致（观察者/提示/种类/选项含顺序/数量/放弃许可）；
        // 无开放选择时任何挂起机器都不得处于待询问状态。
        let asks: Vec<crate::snap::ExpectedAsk> = snapshot
            .resolution
            .parks
            .iter()
            .filter_map(|p| crate::snap::active_ask(&p.machine, &snapshot.core, &catalog))
            .collect();
        let ask_refs: Vec<&crate::snap::ExpectedAsk> = asks.iter().collect();
        match (&snapshot.resolution.open_choice, ask_refs.as_slice()) {
            (Some(open), [exp]) => {
                if open.observer != exp.observer
                    || open.spec.prompt != exp.prompt
                    || open.spec.kind != exp.kind
                    || open.spec.options != exp.options
                    || open.spec.min != exp.min
                    || open.spec.max != exp.max
                    || open.spec.allow_cancel != exp.allow_cancel
                {
                    return Err(SnapshotError::OpenChoiceMismatch {
                        observer: open.observer,
                    });
                }
            }
            (Some(open), _) => {
                return Err(SnapshotError::OpenChoiceMismatch {
                    observer: open.observer,
                });
            }
            (None, []) => {}
            (None, asks) => {
                // 无开放选择却存在待答询问：承接关系不一致（N04-e）。
                let _ = asks;
                return Err(SnapshotError::OpenChoiceMismatch { observer: 0 });
            }
        }
        // G1 规则宿主绑定：扩展机器的定义键必须能在当前目录解析为
        // 宿主行为技能且行为标签一致——扩展集合不含该包（或版本不符）
        // 时恢复拒绝（UnknownMachine）。
        for park in &snapshot.resolution.parks {
            if let crate::snap::MachineSnapshot::Host { def_key, behavior, .. } = &park.machine {
                // 技能型宿主机器：定义键须解析为目录技能且行为一致；
                // 卡牌型（手里剑）：目录含该牌种 ⇔ 声明它的包在场。
                let skill_ok = catalog
                    .skill_by_name(def_key)
                    .and_then(|def| catalog.skills[def as usize].behavior.as_host_behavior())
                    .map(|hb| hb.tag() == *behavior)
                    .unwrap_or(false);
                let card_ok = match def_key.as_str() {
                    crate::content::host::HOST_SHOULIJIAN_KEY => {
                        *behavior
                            == crate::content::host::HostBehavior::TargetDiscardEquipOrDamage
                                .tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::ShouLiJian)
                    }
                    crate::content::host::WD_WANGMEIZHIKE_KEY => {
                        *behavior == crate::content::host::HostBehavior::WangmeiZhikeCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::WangmeiZhike)
                    }
                    crate::content::host::WD_SHUIGONG_KEY => {
                        *behavior == crate::content::host::HostBehavior::ShuigongCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shuigong)
                    }
                    crate::content::host::WD_TOULIANG_KEY => {
                        *behavior == crate::content::host::HostBehavior::TouliangSwap.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Toulianghuanzhu)
                    }
                    crate::content::host::HT_LINGHUN_KEY => {
                        *behavior == crate::content::host::HostBehavior::LinghunzhihuoCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Linghunzhihuo)
                    }
                    crate::content::host::HT_ZHAOMING_KEY => {
                        *behavior == crate::content::host::HostBehavior::ZhaomingdanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zhaomingdan)
                    }
                    crate::content::host::WD_CAOCHUAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::CaochuanCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Caochuanjiejian)
                    }
                    crate::content::host::WD_SHEZHAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::ShezhanqunruCompare.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Shezhanqunru)
                    }
                    crate::content::host::GJ_SHUJINSAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianShujinsanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shujinsan)
                    }
                    crate::content::host::GJ_MIANLI_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianMianliCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Mianlijinzhen)
                    }
                    crate::content::host::GJ_FEIBIAO_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFeibiaoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Feibiao)
                    }
                    crate::content::host::GJ_TANHUADONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Tanhuadong)
                    }
                    crate::content::host::GJ_YOUGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yougeng)
                    }
                    crate::content::host::GJ_QINGTUAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Qingtuan)
                    }
                    crate::content::host::GJ_YUANBAOROU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yuanbaorou)
                    }
                    crate::content::host::GJ_GUDONGGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Gudonggeng)
                    }
                    crate::content::host::GJ_MAPODOUFU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Mapodoufu)
                    }
                    crate::content::host::GJ_CHUNBING_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Chunbing)
                    }
                    crate::content::host::GJ_LIYUTANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liyutang)
                    }
                    crate::content::host::GJ_MOLICHA_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Molicha)
                    }
                    crate::content::host::GJ_XIAJIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Xiajiao)
                    }
                    crate::content::host::GJ_MIZHLIANOU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Mizhilianou)
                    }
                    crate::content::host::GJ_LUYUGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Luyugeng)
                    }
                    crate::content::host::GJ_BINGPOTONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanBingpoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Bingpotong)
                    }
                    crate::content::host::GJ_QIANKUNBIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanQiankunCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Qiankunbiao)
                    }
                    crate::content::host::GJ_LONGXUGOU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanLongxuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Longxugou)
                    }
                    crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanShenhuoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shenhuofeiya)
                    }
                    crate::content::host::GJ_ZIYANGDAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Ziyangdan)
                    }
                    crate::content::host::GJ_YUNVYUANSHEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yunvyuanshen)
                    }
                    crate::content::host::GJ_SHIHUIFEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianShihuifenCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shihuifen)
                    }
                    crate::content::host::GJ_LIUFENGSAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianLiufengsanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liufengsan)
                    }
                    crate::content::host::GJ_SHATANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanShatangCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shatang)
                    }
                    crate::content::host::GJ_HEILONGLINPIAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanHeilongCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Heilonglinpian)
                    }
                    crate::content::host::GJ_LIUTOUGE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanLiutougeCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liutouge)
                    }
                    crate::content::host::GJ_DINVXUANSHUANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanDinvCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Dinvxuanshuang)
                    }
                    crate::content::host::HH_DUNPAI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthDunpaiCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Dunpaigedang)
                    }
                    crate::content::host::HH_ZHILIAOBO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthZhiliaoboCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zhiliaobo)
                    }
                    crate::content::host::HH_YUANSU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthYuansuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yuansuhuimie)
                    }
                    crate::content::host::HH_XINGJIE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthXingjieCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Xingjiegoutong)
                    }
                    crate::content::host::HH_CHUANSONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthChuansongCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Chuansongmen)
                    }
                    crate::content::host::HH_TANSHE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthTansheCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Tanshezhiren)
                    }
                    crate::content::host::HH_SHIJIE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthShijieshuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shijieshu)
                    }
                    crate::content::host::GW_YANZHI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentYanzhiCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yanziyaoshui)
                    }
                    crate::content::host::GW_AOZU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentAozuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Aozuzhilei)
                    }
                    crate::content::host::GW_WENYI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWenyiCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Wenyi)
                    }
                    crate::content::host::GW_ZIHUAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentZhihuanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zhihuanjun)
                    }
                    crate::content::host::GW_BAOXUE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentBaoxueCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Baoxueyaoshui)
                    }
                    crate::content::host::GW_QINPEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Qinpendayu)
                    }
                    crate::content::host::GW_BIRIN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Birinongwu)
                    }
                    crate::content::host::GW_CIGU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Ciguhanshuang)
                    }
                    crate::content::host::GW_KUNEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentKunenCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Kunenfayin)
                    }
                    crate::content::host::GW_SHANBENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentShanbengCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shanbengshu)
                    }
                    crate::content::host::GW_POXIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentPoxiaoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Poxiao)
                    }
                    crate::content::host::GW_ZUMO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentZumoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zumoshoukao)
                    }
                    crate::content::host::YC_GEAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::GeanguanhuoCompare.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Geanguanhuo)
                    }
                    crate::content::host::HT_JIHUO_KEY => {
                        *behavior == crate::content::host::HostBehavior::JihuocardDraw.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Jihuocard)
                    }
                    crate::content::host::WD_FUDI_KEY => {
                        *behavior == crate::content::host::HostBehavior::FudichouxinCompare.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Fudichouxin)
                    }
                    _ => false,
                };
                if !skill_ok && !card_ok {
                    return Err(SnapshotError::UnknownMachine);
                }
            }
        }
        // G1 装备技能生命周期一致性：已装备牌声明的技能必须存在实例
        //（授予/撤销由装备/离场路径维护，快照不得出现「有装备无技能」）。
        for (pi, p) in snapshot.core.players.iter().enumerate() {
            for c in &p.equips {
                for name in &catalog.cards[*c as usize].equip_skills {
                    let ok = catalog
                        .skill_by_name(name)
                        .map(|def| snapshot.core.skills.contains_key(&(pi as u8, def)))
                        .unwrap_or(false);
                    if !ok {
                        return Err(SnapshotError::EquipSkillMismatch { player: pi as u8 });
                    }
                }
            }
        }
        let mut parks = Vec::with_capacity(snapshot.resolution.parks.len());
        for park in &snapshot.resolution.parks {
            parks.push(crate::resolution::Park {
                resume_depth: park.resume_depth,
                slot: park.slot,
                machine: park.machine.restore()?,
            });
        }
        Ok(Self {
            catalog,
            state: GameState {
                core: snapshot.core.clone(),
                resolution: Resolution {
                    next_event_id: snapshot.resolution.next_event_id,
                    next_choice_id: snapshot.resolution.next_choice_id,
                    stack: snapshot.resolution.stack.clone(),
                    parks,
                    open_choice: snapshot.resolution.open_choice.clone(),
                    answer: snapshot.resolution.answer.clone(),
                    budget: snapshot.resolution.budget,
                },
            },
            log: Vec::new(),
            last_dedup: snapshot.last_dedup,
            time_slice: None,
            seat_quota: None,
            seat_cmd_counts: Default::default(),
            auth_tokens: Default::default(),
            auth_counter: 0,
            auth_secret: fresh_auth_secret(),
            // N11-a：恢复轮换代次——恢复前签发的旧令牌全部失效
            auth_generation: snapshot.auth_generation + 1,
            segment_open: snapshot.segment_open,
            debug_budget: None,
            // 检查点随快照延续（N07-b）；宿主日志水位重置为 0——
            // 恢复件的日志从空开始，中止时截断到 0。
            segment_checkpoint: snapshot
                .segment_checkpoint
                .clone()
                .map(|b| (*b, 0)),
        })
    }

    /// 测试布阵：把一张具名牌放到某玩家手牌（从牌库或弃牌堆取）。
    pub fn debug_place(&mut self, player: PlayerId, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let from_deck = self.state.core.deck.iter().position(|c| {
            self.catalog.cards[*c as usize].kind == kind
        });
        let card = if let Some(pos) = from_deck {
            self.state.core.deck.remove(pos)
        } else if let Some(pos) = self
            .state
            .core
            .discard
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            self.state.core.discard.remove(pos)
        } else {
            // 上帝视角布阵：最后从任意玩家手牌回收（仅测试用）
            let mut found = None;
            for (i, p) in self.state.core.players.iter_mut().enumerate() {
                if let Some(pos) = p
                    .hand
                    .iter()
                    .position(|c| self.catalog.cards[*c as usize].kind == kind)
                {
                    found = Some(p.hand.remove(pos));
                    break;
                }
                let _ = i;
            }
            found.unwrap_or_else(|| panic!("no {name} anywhere in deck/discard/hands"))
        };
        self.state.core.player_mut(player).hand.push(card);
        card
    }

    /// 上帝视角：按目录名把特定实体牌移入手牌（从牌库或弃牌堆取；
    /// 测试/负载布阵用——同 Kind 多牌种时按名字精确定位）。
    pub fn debug_move_named_card_to_hand(&mut self, player: PlayerId, name: &str) -> CardId {
        let id = self
            .catalog
            .cards
            .iter()
            .position(|d| d.name == name)
            .expect("目录中应存在该名") as CardId;
        let in_deck = self.state.core.deck.iter().position(|c| *c == id);
        match in_deck {
            Some(pos) => {
                self.state.core.deck.remove(pos);
            }
            None => {
                // 弃牌堆，最后从任意玩家手牌回收
                if let Some(pos) = self.state.core.discard.iter().position(|c| *c == id) {
                    self.state.core.discard.remove(pos);
                } else {
                    let mut found = false;
                    for p in self.state.core.players.iter_mut() {
                        if let Some(pos) = p.hand.iter().position(|c| *c == id) {
                            let c = p.hand.remove(pos);
                            self.state.core.player_mut(player).hand.push(c);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        panic!("{name} 不在牌库/弃牌堆/手牌");
                    }
                }
            }
        }
        self.state.core.player_mut(player).hand.push(id);
        id
    }

    /// 上帝视角：牌库全部移入弃牌堆（负载/测试布阵用，
    /// 下一次摸牌/翻判定牌即触发真实 DeckReshuffled 洗回）。
    pub fn debug_drain_deck_to_discard(&mut self) {
        let deck = std::mem::take(&mut self.state.core.deck);
        self.state.core.discard.extend(deck);
    }

    /// 上帝视角清场：全部玩家判定区移入弃牌堆（仅负载/测试布阵用，
    /// 防止闪电转移后同名锦囊滞留他区导致布阵无牌可回收）。
    pub fn debug_sweep_judge_zones(&mut self) {
        for i in 0..self.state.core.players.len() {
            let z = std::mem::take(&mut self.state.core.player_mut(i as u8).judge);
            self.state.core.discard.extend(z);
        }
    }

    /// 测试布阵：清空手牌（移入弃牌堆），保证确定性。
    pub fn debug_clear_hand(&mut self, player: PlayerId) {
        let hand = std::mem::take(&mut self.state.core.player_mut(player).hand);
        self.state.core.discard.extend(hand);
    }

    /// 测试布阵：直接设置体力。
    /// 测试/工具：设置护甲值。
    pub fn debug_set_hujia(&mut self, player: PlayerId, hujia: u8) {
        self.state.core.player_mut(player).hujia = hujia;
    }

    /// 测试/工具：设置机甲血（D.Va jijia，规则 v141）。
    pub fn debug_set_jijia(&mut self, player: PlayerId, hp: u8) {
        self.state.core.player_mut(player).jijia_hp = hp;
    }

    pub fn debug_set_hp(&mut self, player: PlayerId, hp: i32) {
        self.state.core.player_mut(player).hp = hp;
    }

    /// 测试布阵：授予一名玩家一个永久技能实例。
    /// 测试布阵：按名找一张牌置入离场区（从手牌/牌堆/弃堆回收）。
    pub fn debug_off_court_place(&mut self, player: PlayerId, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let mut found = None;
        let pos = self.state.core.players[player as usize]
            .hand
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind);
        if let Some(pos) = pos {
            found = Some(self.state.core.players[player as usize].hand.remove(pos));
        } else if let Some(pos) = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            found = Some(self.state.core.deck.remove(pos));
        } else if let Some(pos) = self
            .state
            .core
            .discard
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            found = Some(self.state.core.discard.remove(pos));
        }
        let card = found.expect("card available somewhere");
        self.state.core.players[player as usize]
            .off_court
            .push(card);
        card
    }

    pub fn debug_grant(&mut self, player: PlayerId, name: &str) {
        let def = self
            .catalog
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("unknown skill {name}"));
        self.state.core.skills.insert(
            (player, def),
            crate::state::SkillInstance {
                owner: player,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }

    /// 测试/工具：按武将名授予完整技能集（N08 完整武将授予）。
    pub fn debug_grant_general(&mut self, player: PlayerId, name: &str) {
        let skills = content::standard::STANDARD_GENERALS
            .iter()
            .find(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"))
            .2;
        for s in skills {
            self.debug_grant(player, s);
        }
    }

    /// 测试/工具：配置体力上限（N11 将领装配）。
    pub fn debug_set_max_hp(&mut self, player: PlayerId, hp: u32) {
        let p = self.state.core.players.get_mut(player as usize).expect("player");
        p.max_hp = hp;
        if p.hp > hp as i32 {
            p.hp = hp as i32;
        }
    }

    /// 测试/工具：配置主将槽（势力/性别资格类测试用——明置状态不变）。
    /// 按目录 def 索引装配将（测试面；Boss 模式 boss 将经 BOSS_GENERALS
    /// 段索引——general_meta 统一解析，标准表之后为 boss 段）。
    pub fn debug_set_general_def(&mut self, player: PlayerId, def: u16) {
        self.state.core.players
            .get_mut(player as usize)
            .expect("player")
            .general_slots[0] = Some(def);
        apply_general_assembly(&mut self.state.core, &self.catalog, player, 0);
    }

    pub fn debug_set_general(&mut self, player: PlayerId, name: &str) {
        let idx = content::standard::STANDARD_GENERALS
            .iter()
            .position(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"));
        let p = self.state.core.players.get_mut(player as usize).expect("player");
        p.general_slots[0] = Some(idx as u16);
    }

    /// 测试/工具：装配武将（P2/N08 装配授予原语）——配置主将槽并按
    /// 武将表授予技能集与体力上限，与国战 `RevealGeneral` 共用
    /// `apply_general_assembly`（授予语义不分叉）。原型模式无命令层
    /// 选将入口，装配类测试经此原语驱动。摘除见 `debug_remove_general`。
    pub fn debug_assemble_general(&mut self, player: PlayerId, name: &str) {
        let idx = content::standard::STANDARD_GENERALS
            .iter()
            .position(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"));
        self.state.core.players
            .get_mut(player as usize)
            .expect("player")
            .general_slots[0] = Some(idx as u16);
        apply_general_assembly(&mut self.state.core, &self.catalog, player, 0);
    }

    /// 测试/工具：摘除主将（装配撤销）——撤销该武将授予的技能实例
    /// （装备离场撤销语义）并清空主将槽。体力上限不回退（装配授予的
    /// 差异登记：原型测试工具不做体力还原）。
    pub fn debug_remove_general(&mut self, player: PlayerId) {
        let p = self.state.core.players.get_mut(player as usize).expect("player");
        let Some(g) = p.general_slots[0].take() else {
            return;
        };
        for skill in content::standard::general_skills(g) {
            let def = self
                .catalog
                .skill_by_name(skill)
                .unwrap_or_else(|| panic!("catalog skill {skill}"));
            self.state.core.skills.remove(&(player, def));
        }
    }

    /// 测试/工具：切换已授予技能的启停（N08 动态技能资格）。
    /// E03 翻面原语（测试驱动）：对 `player` 发起翻面——压入
    /// TurnOverBefore 帧并驱动调度到弹栈，前置取消窗口内锁定技
    /// （漩涡面具等）写回取消位则不翻转，返回翻面是否发生。
    pub fn debug_flip(&mut self, player: PlayerId) -> bool {
        let before_down = self.state.core.player(player).face_down;
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        let log_mark = self.log.len();
        let id = work.resolution.next_event_id;
        work.resolution.next_event_id += 1;
        work.resolution.stack.push(crate::event::Frame {
            id,
            parent: None,
            kind: crate::event::EventKind::TurnOverBefore,
            data: crate::event::EventData::TurnOverBefore {
                player,
                cancelled: false,
            },
            stage: crate::event::Stage::BeforeTriggers,
            queue: Vec::new(),
            planned: false,
            intrinsic_done: false,
        });
        match run(
            &mut work.core,
            &mut work.resolution,
            &self.catalog,
            &mut self.log,
            self.time_slice,
        ) {
            StepOutcome::Done => {
                work.core.revision += 1;
                self.state = work;
            }
            // 调试原语同步闭环契约：翻面窗口内不应开选择（当前目录中
            // TurnOverBefore 触发均为锁定技）；分片/预算/待决均视为未闭环。
            other => {
                self.log.truncate(log_mark);
                panic!("debug_flip: 翻面前置窗口未闭环: {other:?}");
            }
        }
        self.state.core.player(player).face_down != before_down
    }

    pub fn debug_set_skill_enabled(&mut self, player: PlayerId, name: &str, enabled: bool) {
        let def = self
            .catalog
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("unknown skill {name}"));
        let inst = self
            .state
            .core
            .skills
            .get_mut(&(player, def))
            .unwrap_or_else(|| panic!("skill {name} not granted to player {player}"));
        inst.enabled = enabled;
    }

    /// N10：座位 `seat` 的可见视图——自己的手牌可见，对手手牌与牌库内容隐藏。
    pub fn view_for(&self, seat: PlayerId) -> crate::view::PlayerView {
        self.build_view(Some(seat))
    }

    /// N10：观战者视图——所有手牌隐藏，仅公开信息。
    pub fn view_spectator(&self) -> crate::view::PlayerView {
        self.build_view(None)
    }

    fn build_view(&self, seat: Option<PlayerId>) -> crate::view::PlayerView {
        let core = &self.state.core;
        let players = core
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let i = i as PlayerId;
                let own = seat == Some(i);
                // 国战门控（ADR 0757）：本座位双将可见；他人只见明置槽。
                let mut generals: [Option<crate::view::GeneralView>; 2] = [None, None];
                for (s, g_slot) in p.general_slots.iter().enumerate() {
                    let Some(g) = g_slot else { continue };
                    if !own && !p.revealed_generals[s] {
                        continue; // 暗置槽：他人不可见
                    }
                    if let Some(f) = crate::content::standard::faction_of_general(*g) {
                        generals[s] = Some(crate::view::GeneralView {
                            name: crate::content::standard::STANDARD_GENERALS[*g as usize].0,
                            faction: f,
                        });
                    }
                }
                crate::view::PlayerCellView {
                    seat: i,
                    alive: p.alive,
                    hp: p.hp,
                    max_hp: p.max_hp,
                    hand: if own {
                        crate::view::HandView::Own(p.hand.clone())
                    } else {
                        crate::view::HandView::Hidden
                    },
                    hand_count: p.hand.len(),
                    equips: p.equips.clone(),
                    judge: p.judge.clone(),
                    generals,
                    revealed_generals: p.revealed_generals,
                }
            })
            .collect();
        crate::view::PlayerView {
            seat: seat.unwrap_or(PlayerId::MAX),
            players,
            deck_count: core.deck.len(),
            discard: core.discard.clone(),
            table: core.table.clone(),
            turn: core.turn.current,
            phase: core.turn.phase,
            round: core.turn.round,
            outcome: core.outcome.as_ref().map(|o| o.winner),
            ruleset_version: core.ruleset_version,
        }
    }

    /// N10 第二批：开放选择的座位可见性——观察者见完整选项，
    /// 他人仅见询问存在（选项会暴露应答者手牌）。无开放选择返回 None。
    pub fn choice_view_for(
        &self,
        seat: Option<PlayerId>,
    ) -> Option<crate::view::ChoiceView> {
        let open = self.state.resolution.open_choice.as_ref()?;
        Some(if seat == Some(open.observer) {
            crate::view::ChoiceView::Own {
                id: open.id,
                prompt: open.spec.prompt,
                options: open.spec.options.clone(),
                min: open.spec.min,
                max: open.spec.max,
                allow_cancel: open.spec.allow_cancel,
            }
        } else {
            crate::view::ChoiceView::Other {
                id: open.id,
                observer: open.observer,
                prompt: open.spec.prompt,
            }
        })
    }

    /// N10 第二批：日志事件的座位可见性——他人摸牌仅数量（牌 id 隐藏），
    /// 其余事件公开；自己的摸牌完整可见。
    pub fn event_view_for(&self, seat: PlayerId) -> Vec<crate::view::EventView> {
        // N10-a（ADR 0768）：私有选择应答仅观察者可见——沿日志维护
        // 询问 id → 观察者映射，非观察者收到内容不透明的脱敏事件。
        let mut choice_observers: std::collections::HashMap<u64, PlayerId> =
            std::collections::HashMap::new();
        self.log
            .iter()
            .filter_map(|l| match l {
                crate::machine::LogEvent::CardsDrawn { player, cards } if *player != seat => {
                    Some(crate::view::EventView::CardsDrawnHidden {
                        player: *player,
                        count: cards.len(),
                    })
                }
                crate::machine::LogEvent::ChoiceOpened {
                    id, observer, ..
                } => {
                    choice_observers.insert(*id, *observer);
                    Some(crate::view::EventView::Public(l.clone()))
                }
                crate::machine::LogEvent::ChoiceAnswered { id, .. } => {
                    match choice_observers.get(id) {
                        Some(&observer) if observer == seat => {
                            Some(crate::view::EventView::Public(l.clone()))
                        }
                        _ => Some(crate::view::EventView::ChoiceAnsweredHidden { id: *id }),
                    }
                }
                other => Some(crate::view::EventView::Public(other.clone())),
            })
            .collect()
    }

    /// N10 有界计算池：设置每座位每轮命令配额（None = 不限制；
    /// 会话策略，不入快照——恢复后需重新设置）。
    pub fn set_seat_quota(&mut self, quota: Option<u32>) {
        self.seat_quota = quota;
    }

    /// N11 认证：为座位签发会话令牌（重复签发覆盖旧令牌——旧令牌立即失效；
    /// 令牌为宿主会话数据，不入快照，恢复后需重新签发）。
    pub fn issue_token(&mut self, seat: PlayerId) -> AuthToken {
        self.auth_counter += 1;
        // N11-a：实例熵（OS 随机，跨实例不重叠）× 代次（恢复轮换，
        // 恢复前旧令牌失效）× 座位 × 计数器 混合派生。
        let id = self
            .auth_secret
            ^ self
                .auth_generation
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ ((seat as u64).rotate_left(17))
            ^ (self.auth_counter << 24);
        self.auth_tokens.insert(seat, id);
        AuthToken { seat, id }
    }

    /// N11 认证：经认证的命令提交——令牌必须存在且与命令 actor 绑定一致。
    /// 令牌失效 → `NoSuchSession`；令牌座位 ≠ 命令 actor → `Impersonation`；
    /// 校验通过后走与 [`Engine::submit`] 完全相同的主通道。
    pub fn submit_authed(&mut self, token: AuthToken, cmd: GameCommand) -> CommandOutcome {
        match self.auth_tokens.get(&token.seat) {
            None => return CommandOutcome::Rejected(RejectReason::NoSuchSession),
            Some(&id) if id != token.id => {
                return CommandOutcome::Rejected(RejectReason::NoSuchSession)
            }
            Some(_) => {}
        }
        if token.seat != cmd.actor {
            return CommandOutcome::Rejected(RejectReason::Impersonation);
        }
        self.submit(cmd)
    }

    /// N10：座位本轮剩余配额（未设置配额返回 None）。
    pub fn seat_quota_remaining(&self, seat: PlayerId) -> Option<u32> {
        let q = self.seat_quota?;
        let round = self.state.core.turn.round;
        let used = self
            .seat_cmd_counts
            .get(&seat)
            .and_then(|(r, c)| (*r == round).then_some(*c))
            .unwrap_or(0);
        Some(q.saturating_sub(used))
    }

    /// N10：AI 只读视角会话——座位固定，仅暴露可见性投影。
    pub fn ai_session(&self, seat: PlayerId) -> crate::view::AiSession<'_> {
        crate::view::AiSession { engine: self, seat }
    }

    /// 测试/工具：只读访问核心状态。
    pub fn state_core(&self) -> &GameStateCore {
        &self.state.core
    }

    /// 测试/工具：把一张牌移入某玩家判定区（按 ID；跨包无名牌用）。
    pub fn debug_place_judge_id(&mut self, player: PlayerId, card: CardId) {
        // 先从持有区域移除（手/牌库/弃牌堆）
        {
            let p = self.state.core.player_mut(player);
            p.hand.retain(|c| *c != card);
            p.equips.retain(|c| *c != card);
        }
        self.state.core.deck.retain(|c| *c != card);
        self.state.core.discard.retain(|c| *c != card);
        self.state.core.player_mut(player).judge.push(card);
    }

    /// 测试布阵：把一张具名延时锦囊放到某玩家判定区（从牌库或弃牌堆取）。
    pub fn debug_place_judge(&mut self, player: PlayerId, name: &str) -> CardId {
        let card = self.debug_place(player, name);
        let hand = &mut self.state.core.player_mut(player).hand;
        let pos = hand.iter().position(|c| *c == card).expect("just placed");
        hand.remove(pos);
        self.state.core.player_mut(player).judge.push(card);
        card
    }

    /// 测试布阵：把一张具名牌放到牌库顶（下一张摸/判定即该牌）。
    pub fn debug_set_deck_top(&mut self, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let pos = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
            .or_else(|| {
                // 牌库没有则从弃牌堆回收
                let dpos = self
                    .state
                    .core
                    .discard
                    .iter()
                    .position(|c| self.catalog.cards[*c as usize].kind == kind)?;
                let card = self.state.core.discard.remove(dpos);
                self.state.core.deck.push(card);
                Some(self.state.core.deck.len() - 1)
            })
            .expect("card available somewhere");
        let card = self.state.core.deck.remove(pos);
        self.state.core.deck.push(card);
        card
    }

    /// 测试布阵：按谓词挑一张牌压到牌库顶。
    pub fn debug_set_deck_top_by(
        &mut self,
        pred: impl Fn(&crate::catalog::CardDef) -> bool,
    ) -> CardId {
        let find = |src: &GameState| -> Option<usize> {
            src.core
                .deck
                .iter()
                .position(|c| pred(&self.catalog.cards[*c as usize]))
        };
        let pos = match find(&self.state) {
            Some(pos) => pos,
            None => {
                // 牌库没有 → 弃牌堆 → 其他玩家手牌（上帝视角布阵）
                if let Some(dpos) = self
                    .state
                    .core
                    .discard
                    .iter()
                    .position(|c| pred(&self.catalog.cards[*c as usize]))
                {
                    let card = self.state.core.discard.remove(dpos);
                    self.state.core.deck.push(card);
                    return self.state.core.deck.pop().expect("just pushed");
                }
                let mut found = None;
                for p in self.state.core.players.iter_mut() {
                    if let Some(hpos) = p
                        .hand
                        .iter()
                        .position(|c| pred(&self.catalog.cards[*c as usize]))
                    {
                        found = Some(p.hand.remove(hpos));
                        break;
                    }
                }
                let card =
                    found.expect("card matching predicate available in deck/discard/hands");
                self.state.core.deck.push(card);
                self.state.core.deck.len() - 1
            }
        };
        let card = self.state.core.deck.remove(pos);
        self.state.core.deck.push(card);
        card
    }

    /// 调试：读取判定区。
    pub fn debug_judge(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).judge.clone()
    }

    /// E03 翻面原语：读取武将牌朝向。
    pub fn debug_face_down(&self, player: PlayerId) -> bool {
        self.state.core.player(player).face_down
    }

    /// E03 测试原语：强制设置武将牌朝向（不经事件流，用于状态重置）。
    pub fn debug_set_face_down(&mut self, player: PlayerId, face_down: bool) {
        self.state.core.player_mut(player).face_down = face_down;
    }

    /// E03 测试辅助：移除玩家装备区的一张牌（离场撤销装备技）。
    pub fn debug_remove_equip(&mut self, player: PlayerId, card: CardId) {
        let equips = &mut self.state.core.players[player as usize].equips;
        equips.retain(|c| *c != card);
        crate::kernel::revoke_equip_skills(
            &mut self.state.core,
            &self.catalog,
            player,
            card,
        );
    }

    pub fn debug_discard_len(&self) -> usize {
        self.state.core.discard.len()
    }

    /// 测试钩子：设置下一命令段的调度预算上限（默认 4096 步/段）。
    ///
    /// 预算是**每段**硬上限（ADR 0708）：段开始时重置，不跨段累计——
    /// 旧实现预算随状态全局递减，长对局必然耗尽。覆盖值不入状态/哈希。
    pub fn debug_set_budget(&mut self, budget: u32) {
        self.debug_budget = Some(budget);
    }

    pub fn debug_hand(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).hand.clone()
    }

    pub fn debug_equips(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).equips.clone()
    }

    pub fn debug_hp(&self, player: PlayerId) -> i32 {
        self.state.core.player(player).hp
    }

    pub fn debug_alive(&self, player: PlayerId) -> bool {
        self.state.core.player(player).alive
    }

    pub fn debug_has_skill(&self, player: PlayerId, name: &str) -> bool {
        match self.catalog.skill_by_name(name) {
            Some(def) => self.state.core.has_skill(player, def),
            None => false,
        }
    }

    /// 提交命令：单一工作副本上结算，拒绝则整段丢弃。
    ///
    /// 信任边界（校验顺序即拒绝优先级）：
    /// 1. 去重键（`DuplicateCommand`：键必须为正且严格超过已接受高水位）；
    /// 2. 修订号（`StaleRevision`）；
    /// 3. actor 座位界（`UnknownActor`）；
    /// 4. 终局/选择挂起门禁；
    /// 5. 动作授权与参数校验（`apply_command`：actor 身份、ID 界、规则合法性）。
    /// 只有 Accepted/Pending 消耗去重键；拒绝路径不消耗，可同键重试。
    /// 设置宿主时间片（None = 不限）。默认不限片：命令要么完整结算、
    /// 要么暂停于选择、要么整体拒绝/回滚。
    pub fn set_time_slice(&mut self, slice: Option<u32>) {
        self.time_slice = slice;
    }

    /// 段是否处于时间片暂停中（状态已提交部分结算，须 resume 续行）。
    pub fn is_suspended(&self) -> bool {
        self.segment_open
    }

    /// 中止开放中的段（N07 失败契约）：回滚到段启动前的快照——状态、
    /// 修订号、去重水位、宿主日志全部恢复，段关闭。适用于
    /// 「暂停段遭遇 `BudgetExhausted`」等无法续行的失败：宿主可放弃该段、
    /// 以新命令继续对局（例如提高预算或绕开导致失败的玩法）。
    /// 未处于暂停段时返回 `NotSuspended`。
    pub fn abort_segment(&mut self) -> Result<(), RejectReason> {
        if !self.segment_open {
            return Err(RejectReason::NotSuspended);
        }
        // 无检查点的开放段不可静默关闭（N07-b：仅改标志并报成功是被
        // 禁止的契约违反）；检查点随快照延续，开放段必然携带。
        let Some((snapshot, log_mark)) = self.segment_checkpoint.take() else {
            return Err(RejectReason::NotSuspended);
        };
        let restored = Engine::restore(&snapshot).map_err(|_| RejectReason::NotSuspended)?;
        self.state = restored.state;
        self.last_dedup = restored.last_dedup;
        self.log.truncate(log_mark);
        self.segment_open = false;
        Ok(())
    }

    /// 续行时间片暂停的段：以新片继续推进，直至 Accepted/Pending/Rejected。
    pub fn resume(&mut self) -> CommandOutcome {
        if !self.segment_open {
            return CommandOutcome::Rejected(RejectReason::NotSuspended);
        }
        let log_mark = self.log.len();
        let work = self.state.clone();
        self.finish_segment(work, log_mark)
    }

    pub fn submit(&mut self, cmd: GameCommand) -> CommandOutcome {
        if self.segment_open {
            return CommandOutcome::Rejected(RejectReason::SegmentSuspended);
        }
        if cmd.dedup_id == 0 || cmd.dedup_id <= self.last_dedup {
            return CommandOutcome::Rejected(RejectReason::DuplicateCommand);
        }
        if cmd.expected_revision != self.state.core.revision {
            return CommandOutcome::Rejected(RejectReason::StaleRevision);
        }
        if cmd.actor as usize >= self.state.core.players.len() {
            return CommandOutcome::Rejected(RejectReason::UnknownActor);
        }
        if self.state.core.outcome.is_some() && !matches!(cmd.action, GameAction::Answer { .. }) {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        // 补偿明置（ADR 0762）：成为牌的目标时的明置窗口与挂起询问
        // 并行——RevealGeneral 不受 ChoicePending 拦截（其余命令照旧）。
        if self.state.resolution.open_choice.is_some()
            && !matches!(cmd.action, GameAction::Answer { .. })
            && !matches!(cmd.action, GameAction::RevealGeneral { .. })
        {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        // N10 有界计算池：每座位每轮配额（拒绝不消耗配额；轮次变更自动归零）
        if let Some(q) = self.seat_quota {
            let round = self.state.core.turn.round;
            let used = self
                .seat_cmd_counts
                .get(&cmd.actor)
                .and_then(|(r, c)| (*r == round).then_some(*c))
                .unwrap_or(0);
            if used >= q {
                return CommandOutcome::Rejected(RejectReason::QuotaExhausted);
            }
        }
        // 补偿明置并行提交（ADR 0762）：挂起询问在场时，只应用明置的
        // 状态变更、不推进结算——原询问保持挂起，结算轨迹不变。
        if matches!(cmd.action, GameAction::RevealGeneral { .. })
            && self.state.resolution.open_choice.is_some()
        {
            let log_mark = self.log.len();
            let mut work = self.state.clone();
            if let Err(reason) =
                apply_command(&mut work, cmd.clone(), &self.catalog, &mut self.log)
            {
                self.log.truncate(log_mark);
                return CommandOutcome::Rejected(reason);
            }
            work.core.revision += 1;
            let choice = work
                .resolution
                .open_choice
                .clone()
                .expect("parallel reveal keeps open choice");
            self.state = work;
            self.segment_open = false;
            self.segment_checkpoint = None;
            self.last_dedup = cmd.dedup_id;
            if self.seat_quota.is_some() {
                let entry = self
                    .seat_cmd_counts
                    .entry(cmd.actor)
                    .or_insert((self.state.core.turn.round, 0));
                entry.1 += 1;
            }
            return CommandOutcome::Pending { choice };
        }
        let log_mark = self.log.len();
        let submit_round = self.state.core.turn.round;
        let mut work = self.state.clone();
        // 每段重置预算：硬上限只约束本段，不跨段累计（ADR 0708 缺陷修复）。
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        // 段检查点：供开放段失败时 `abort_segment` 回滚到段前（N07）。
        let checkpoint = self.save();
        if let Err(reason) = apply_command(&mut work, cmd.clone(), &self.catalog, &mut self.log) {
            self.log.truncate(log_mark);
            return CommandOutcome::Rejected(reason);
        }
        match self.finish_segment(work, log_mark) {
            CommandOutcome::Rejected(reason) => {
                self.segment_checkpoint = None;
                CommandOutcome::Rejected(reason)
            }
            outcome => {
                // 命令已被接受（含暂停于选择的 Pending）：消耗去重键，
                // 并计入该座位本轮配额（按提交时的轮次记账）。
                self.last_dedup = cmd.dedup_id;
                if self.seat_quota.is_some() {
                    let entry = self.seat_cmd_counts.entry(cmd.actor).or_insert((submit_round, 0));
                    if entry.0 == submit_round {
                        entry.1 += 1;
                    } else {
                        *entry = (submit_round, 1);
                    }
                }
                match outcome {
                    CommandOutcome::Suspended => {
                        // 段开放：保留检查点供失败中止（N07）。
                        self.segment_checkpoint = Some((checkpoint, log_mark));
                    }
                    _ => self.segment_checkpoint = None,
                }
                outcome
            }
        }
    }

    /// 结算工作状态并提交/回滚（submit/resume/调试入口共用）。
    ///
    /// 时间片语义（ADR 0707）：`Yielded` 时状态在步边界提交、宿主日志保留，
    /// 段保持开放等待续行；已分片提交过的段无法再整段回滚，故开放段上的
    /// 硬预算耗尽同样提交当前进度并返回 `Rejected(BudgetExhausted)`——
    /// 段会确定性地重复耗尽，宿主应放弃该段（原型契约，见进度文档）。
    fn finish_segment(&mut self, mut work: GameState, log_mark: usize) -> CommandOutcome {
        match run(
            &mut work.core,
            &mut work.resolution,
            &self.catalog,
            &mut self.log,
            self.time_slice,
        ) {
            StepOutcome::BudgetExhausted => {
                if self.segment_open {
                    // 开放段（已分片提交过）：无法整段回滚，提交当前进度，
                    // 段保持开放——宿主应 `abort_segment()` 回滚到段前。
                    self.state = work;
                } else {
                    self.log.truncate(log_mark);
                    self.segment_checkpoint = None;
                }
                CommandOutcome::Rejected(RejectReason::BudgetExhausted)
            }
            StepOutcome::Pending => {
                work.core.revision += 1;
                self.state = work;
                self.segment_open = false;
                self.segment_checkpoint = None;
                let choice = self
                    .state
                    .resolution
                    .open_choice
                    .clone()
                    .expect("pending implies choice");
                CommandOutcome::Pending { choice }
            }
            StepOutcome::Yielded => {
                // 暂停不递增 revision：它是同一状态版本的执行延续，
                // 保证「分片续行」与「连续运行」的摘要轨迹完全一致（N07）。
                // （暂停态的内容仍随预算/执行位置变化而哈希不同。）
                self.state = work;
                self.segment_open = true;
                CommandOutcome::Suspended
            }
            StepOutcome::Done => {
                work.core.revision += 1;
                self.state = work;
                self.segment_open = false;
                self.segment_checkpoint = None;
                CommandOutcome::Accepted {
                    state_hash: canonical_hash(&self.state),
                }
            }
        }
    }

    /// 调试/后续内容迁移入口：直接注入一段属性伤害结算
    /// （走与命令相同的提交/回滚管线）。
    pub fn debug_deal_damage(
        &mut self,
        source: Option<PlayerId>,
        target: PlayerId,
        amount: u32,
        nature: crate::event::DamageNature,
    ) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::Damage(crate::event::DamageData {
                source,
                target,
                amount,
                nature,
                prevented: false,
                redirect_chain: Vec::new(),
                card: None,
                from_chain: false,
            }),
        );
        self.finish_segment(work, log_mark)
    }

    /// 调试：读取连环状态。
    pub fn debug_linked(&self, player: PlayerId) -> bool {
        self.state.core.player(player).linked
    }
}

const DEFAULT_BUDGET: u32 = 4096;

impl Engine {
    fn card_kind_by_name(&self, name: &str) -> CardKind {
        self.catalog
            .cards
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.kind)
            .unwrap_or_else(|| panic!("unknown card name {name}"))
    }
}


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
    for skill in skills {
        let def = catalog
            .skill_by_name(skill)
            .unwrap_or_else(|| panic!("catalog skill {skill}"));
        // 隐贼/延妒（Boss 混沌/穷奇，规则 v210/v213）：装配即未激活
        //（上游 gameStart switch_off disableSkill 同口径；体力首次≤
        // 一半时启用）
        let enabled =
            skill != crate::content::standard::BOSS_YINZEI
                && skill != crate::content::standard::BOSS_YANDU;
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

fn apply_command(
    state: &mut GameState,
    cmd: GameCommand,
    catalog: &SharedCatalog,
    log: &mut Vec<LogEvent>,
) -> Result<(), RejectReason> {
    let actor = cmd.actor;
    match cmd.action {
        GameAction::RevealGeneral { slot } => {
            // 国战限定（ADR 0757 明置门控）：其余模式无武将牌语义。
            if !matches!(state.core.mode, crate::state::GameMode::Kingdoms) {
                return Err(RejectReason::WrongMode);
            }
            if slot > 1 {
                return Err(RejectReason::IllegalTargets);
            }
            // 时机门控：自己回合的出牌阶段空闲点，或成为牌目标的
            // 补偿窗口（结算栈中存在以 actor 为目标的牌）。
            let compensatory = state.resolution.stack.iter().any(|f| match &f.data {
                EventData::CardUsed(u) => u.targets.contains(&actor),
                _ => false,
            });
            if !compensatory {
                if actor != state.core.turn.current {
                    return Err(RejectReason::NotYourTurn);
                }
                if state.core.turn.phase != crate::event::Phase::Play {
                    return Err(RejectReason::WrongPhase);
                }
            }
            let p = state
                .core
                .players
                .get_mut(actor as usize)
                .ok_or(RejectReason::UnknownActor)?;
            if p.general_slots[slot as usize].is_none() {
                return Err(RejectReason::NoSuchGeneral);
            }
            if p.revealed_generals[slot as usize] {
                return Err(RejectReason::AlreadyRevealed);
            }
            p.revealed_generals[slot as usize] = true;
            // N11 将领装配（ADR 0769）：明置实际授予该武将技能集；
            // 首次明置主将时配置体力上限（副将明置不改体力）。
            apply_general_assembly(&mut state.core, catalog, actor, slot as usize);
            // 明置是公开行为：状态与视图即时可见（无隐藏事件载荷）。
            Ok(())
        }
        GameAction::ActivateSkill { skill } => {
            // 授权：主动技仅当前回合玩家可发动
            if actor != state.core.turn.current {
                return Err(RejectReason::NotYourTurn);
            }
            let Some(def) = catalog.skills.get(skill as usize) else {
                return Err(RejectReason::NoSuchSkill);
            };
            // 阶段门控（N08 上游对齐）：主动技默认限出牌阶段（上游
            // `enable: "phaseUse"`——仁德曾在判定阶段被放行）；响应入口
            // 类技能（激将 `chooseToUse`/`chooseToRespond`）在目录声明
            // 例外后不绑定阶段。
            if state.core.turn.phase != crate::event::Phase::Play && !def.limits.active_outside_play
            {
                return Err(RejectReason::WrongPhase);
            }
            let Some(inst) = state.core.skills.get(&(actor, skill)) else {
                return Err(RejectReason::NoSuchSkill);
            };
            if !inst.enabled {
                return Err(RejectReason::SkillDisabled);
            }
            if inst.once_used {
                return Err(RejectReason::SkillAlreadyUsed);
            }
            let qctx = crate::catalog::QueryCtx { state: &state.core, catalog };
            let Some(machine) = def.behavior.activate(&qctx, actor) else {
                return Err(RejectReason::InvalidChoice);
            };
            if def.limits.once_per_turn {
                if let Some(inst) = state.core.skills.get_mut(&(actor, skill)) {
                    inst.once_used = true;
                }
            }
            crate::resolution::push_frame(
                &mut state.resolution,
                EventData::SkillToggled {
                    player: actor,
                    skill,
                    enabled: true,
                    temporary: false,
                },
            );
            let frame_id = state.resolution.next_event_id;
            let _ = frame_id;
            // 主动技机器入栈顶挂起槽（复用 Intrinsic 槽语义）
            state.resolution.parks.push(crate::resolution::Park {
                resume_depth: state.resolution.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                machine,
            });
            Ok(())
        }
        GameAction::Answer {
            choice_id,
            selection,
        } => {
            let Some(open) = state.resolution.open_choice.as_ref() else {
                return Err(RejectReason::NoSuchChoice);
            };
            if open.id != choice_id {
                return Err(RejectReason::NoSuchChoice);
            }
            // 授权：只有选择观察者本人可以应答（不能替别人决策）。
            if actor != open.observer {
                return Err(RejectReason::WrongPlayer);
            }
            if !open.spec.accepts(&selection) {
                return Err(RejectReason::InvalidChoice);
            }
            let open = state.resolution.open_choice.take().expect("checked");
            log.push(LogEvent::ChoiceAnswered {
                id: open.id,
                options: selection.iter().map(|o| o.describe()).collect(),
            });
            state.resolution.answer = Some(selection);
            Ok(())
        }
        GameAction::PlayCard { card, targets } => {
            let core = &state.core;
            // 授权：只有当前回合玩家可以出牌。
            if actor != core.turn.current {
                return Err(RejectReason::NotYourTurn);
            }
            if core.turn.phase != Phase::Play {
                return Err(RejectReason::WrongPhase);
            }
            // 蛊舞·缚（U05 xianjian 阿奴，规则 v154）：蛊舞生效中每回合
            // 最多使用 3 张牌（上游 anwugu2 mod cardEnabled countUsed>=
            // storage 同口径；countUsed=本出牌阶段已用数——登记差异：
            // 上游跨阶段计 countUsed，原生以出牌阶段计数近似）
            if core.player(actor).anwugu_count > 0
                && core.turn.cards_used_play_phase >= 3
            {
                return Err(RejectReason::CardLimit);
            }
            if !core.player(core.turn.current).hand.contains(&card) {
                return Err(RejectReason::CardNotInHand);
            }
            // 目标 ID 越界必须显式拒绝（不得进入 player() 索引）。
            if targets.iter().any(|t| *t as usize >= core.players.len()) {
                return Err(RejectReason::IllegalTargets);
            }
            let mut kind = catalog.card_kind(card);
            let mut mark_mutoumianju = false;
            // view-as 转化（N08 官方迁移）：
            // 国色（方块手牌当乐不思蜀，优先于武圣）/ 武圣（红色手牌当杀）/
            // 龙胆（闪当杀）——以此后的路由与校验按声明牌种执行。
            if kind != CardKind::Le && kind != CardKind::Sha {
                let actor = core.turn.current;
                let guose = catalog
                    .skill_by_name(crate::content::standard::GUO_SE)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false);
                let diamond = matches!(
                    catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Diamond
                );
                if guose && diamond {
                    kind = CardKind::Le;
                }
            }
            if kind != CardKind::Sha && kind != CardKind::Le {
                let actor = core.turn.current;
                let wusheng = catalog
                    .skill_by_name(crate::content::standard::WU_SHENG)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false);
                let longdan = catalog
                    .skill_by_name(crate::content::standard::LONG_DAN)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false);
                let qixi = catalog
                    .skill_by_name(crate::content::standard::QI_XI)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false);
                let black = matches!(
                    catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                let red = matches!(
                    catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                if (wusheng && red) || (longdan && kind == CardKind::Shan) {
                    kind = CardKind::Sha;
                }
                // 奇袭（甘宁）：黑色牌当过河拆桥（优先级低于杀类转化）
                else if qixi && black {
                    kind = CardKind::Dismantle;
                }
                // 探囊（U05 xianjian 韩菱纱，规则 v148）：梅花手牌当顺手
                // 牵羊（优先级与奇袭同级并列）
                else if catalog
                    .skill_by_name(crate::content::host::XJ_TANNANG_KEY)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false)
                    && matches!(
                        catalog.cards[card as usize].suit,
                        crate::catalog::Suit::Club
                    )
                {
                    kind = CardKind::Snatch;
                }
                else {
                    // 木头面具（G1 古剑装备）：出牌阶段限一次，任意
                    // 手牌当【杀】（once_used 随回合重置；标记延迟至
                    // 借用结束后）
                    let muto_active = catalog
                        .skill_by_name(crate::content::host::GJ_MUTOU_SKILL_KEY)
                        .map(|d| {
                            core.skills
                                .get(&(actor, d))
                                .map(|i| i.enabled && !i.once_used)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    if muto_active {
                        kind = CardKind::Sha;
                        mark_mutoumianju = true;
                    } else {
                        // 蜜链藕（G1 古剑食物）：红桃手牌当桃
                        //（持有状态即生效）
                        let mizhilianou = catalog
                            .skill_by_name(crate::content::host::GJ_FOOD_MIZHLIANOU_KEY)
                            .map(|d| core.has_skill(actor, d))
                            .unwrap_or(false);
                        let is_heart = matches!(
                            catalog.cards[card as usize].suit,
                            crate::catalog::Suit::Heart
                        );
                        if mizhilianou && is_heart {
                            kind = CardKind::Tao;
                        }
                    }
                }
            }
            // 麻茶（G1 古剑食物）：持有者不能成为其他角色黑色牌的
            // 目标（登记差异：仅出牌使用路径过滤，不含响应路径）
            {
                let black = matches!(
                    catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if black && !targets.is_empty() {
                    let molicha = catalog.skill_by_name(crate::content::host::GJ_FOOD_MOLICHA_KEY);
                    if let Some(def) = molicha {
                        for t in &targets {
                            if *t != core.turn.current && core.skills.contains_key(&(*t, def)) {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                }
            }
            // 石会粉禁用状态（G1 古剑）：本回合无法对其他角色使用
            // 卡牌（expires_end_of_turn 于其回合结束清除）
            {
                let ban = catalog
                    .skill_by_name(crate::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
                    .map(|def| {
                        core.skills
                            .get(&(core.turn.current, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if ban && targets.iter().any(|t| *t != core.turn.current) {
                    return Err(RejectReason::IllegalTargets);
                }
            }
            // 急冻冰封（U05 ow 美，规则 v127）：冰封角色不能被其他角色
            // 的牌指定为目标（上游 jidong2 mod targetEnabled 同口径；
            // AOE 类自动指向全体的牌不受影响——上游同口径；登记差异：
            // 仅出牌使用路径，响应路径同麻茶）
            {
                for t in &targets {
                    if *t != core.turn.current
                        && (core.player(*t).jidong_active
                            || core.player(*t).yinshen_active
                            || core.player(*t).jileng_active
                            || core.player(*t).yinmi_active)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
            }
            if kind.is_equip() {
                if !targets.is_empty() {
                    return Err(RejectReason::IllegalTargets);
                }
                let player = core.turn.current;
                crate::resolution::push_frame(
                    &mut state.resolution,
                    EventData::Equipped {
                        player,
                        card,
                        replaced: None,
                    },
                );
                return Ok(());
            }
            // 紧急冷冻（U05 swd 灵剑，规则 v198）：冻结者不能使用卡牌
            //（上游 hslingjian_jinjilengdong mod cardEnabled 同口径）
            if core.player(core.turn.current).jileng_active {
                return Err(RejectReason::CardLimit);
            }
            match kind {
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                    // 咆哮（张飞）：出牌阶段使用杀无次数限制
                    let mut mark_yuanbaorou = false;
                    let player_id = core.turn.current;
                    let paoxiao = catalog
                        .skill_by_name(crate::content::standard::PAO_XIAO)
                        .map(|d| core.has_skill(core.turn.current, d))
                        .unwrap_or(false);
                    // 碧日浓雾（gwent 天气）：持有者不能使用杀
                    if catalog
                        .skill_by_name(crate::content::host::GW_BIRIN_STATE_KEY)
                        .map(|def| {
                            core.skills
                                .get(&(core.turn.current, def))
                                .map(|i| i.enabled)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 堡垒哨卫（U05 ow，规则 v125）：哨卫模式杀无次数
                    // 限制（上游 bshaowei_sha cardUsable Infinity 同口径）
                    let baolei_sentinel = {
                        let bdef = catalog
                            .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY);
                        bdef.map(|def| {
                            core.skills
                                .get(&(core.turn.current, def))
                                .map(|i| i.enabled)
                                .unwrap_or(false)
                                && core.player(core.turn.current).baolei_mode == 1
                        })
                        .unwrap_or(false)
                    };
                    // 守护（U07 ow，规则 v121）：锁定禁杀——持有者不能
                    // 使用杀（上游 shouhu mod cardEnabled 同口径）
                    if catalog
                        .skill_by_name(crate::content::host::WUDIAN_SHOUHU_KEY)
                        .map(|def| {
                            core.skills
                                .get(&(core.turn.current, def))
                                .map(|i| i.enabled)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 诸葛连弩（批③-b 第四把）：装备时出杀不限次数（与
                    // 咆哮同口径的锁定豁免，按装备牌种判断）。
                    let has_zhuge = core.player(player_id).equips.iter().any(|c| {
                        catalog.card_kind(*c) == CardKind::Zhuge
                    });
                    // 龙颅山峰（mtg 首切片，规则 v100）：地图效果全局
                    // 出杀上限 +1（上游 addGlobalSkill cardUsable 同口径）
                    let land_sha_bonus = core.current_land
                        == Some(CardKind::Longlushanfeng)
                        && !core.turn.land_sha_used;
                    if !paoxiao
                        && !has_zhuge
                        && !baolei_sentinel
                        && core.turn.sha_used_play_phase
                        && !land_sha_bonus
                    {
                        // 元宝肉（G1 古剑食物）：出牌阶段额外一张杀
                        //（每回合一次；once_used 随回合开始重置）
                        let extra = catalog
                            .skill_by_name(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
                            .map(|def| {
                                core.skills
                                    .get(&(core.turn.current, def))
                                    .map(|i| i.enabled && !i.once_used)
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false);
                        if !extra {
                            return Err(RejectReason::ShaLimit);
                        }
                        mark_yuanbaorou = true;
                    }
                    // 方天画戟（批③-b 第九把）：该杀为最后手牌且装备方天
                    // 时可指定至多 3 个目标（逐个独立合法性检查，去重）。
                    let fangtian = core.player(player_id).hand.len() == 1
                        && core
                            .player(player_id)
                            .equips
                            .iter()
                            .any(|c| catalog.card_kind(*c) == CardKind::Fangtian);
                    // 抗yin2（U07 批四，规则 v114）：杀额外目标上限 +X
                    let kangyin_extra = match core.turn.kangyin2 {
                        Some((o, x)) if o == player_id => x as usize,
                        _ => 0,
                    };
                    // 飞刃（U05 ow 源氏，规则 v129）：梅花杀额外目标上限
                    // +1（上游 selectTarget range[1]++ 同口径；花色按实
                    // 体牌——转化声明牌种不影响）
                    let feiren_extra = {
                        let club = matches!(
                            catalog.cards[card as usize].suit,
                            crate::catalog::Suit::Club
                        );
                        let sha_kind = kind.is_sha_like();
                        club
                            && sha_kind
                            && catalog
                                .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
                                .map(|def| core.has_skill(player_id, def))
                                .unwrap_or(false)
                    };
                    // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——杀
                    // 目标数上限无限（上游 mod selectTarget range[1]=∞
                    // 同口径；sha 有限上限必满足 !==-1 条件）
                    let tianxian_inf = kind.is_sha_like()
                        && catalog
                            .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
                            .map(|def| core.has_skill(player_id, def))
                            .unwrap_or(false);
                    let max_targets = if tianxian_inf {
                        usize::MAX
                    } else {
                        (if fangtian { 3 } else { 1 })
                            + kangyin_extra
                            + u32::from(feiren_extra) as usize
                    };
                    if targets.is_empty() || targets.len() > max_targets {
                        return Err(RejectReason::IllegalTargets);
                    }
                    {
                        let mut seen = std::collections::BTreeSet::new();
                        for t in &targets {
                            if !seen.insert(*t) {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 空城（诸葛亮）：无手牌者不能成为杀/决斗目标（锁定门控）
                    let kongcheng = catalog
                        .skill_by_name(crate::content::standard::KONG_CHENG)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if kongcheng && core.player(target).hand.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 圣休号角嘲讽（U05 swd 灵剑，规则 v198）：相邻有嘲讽
                    // 者时只能以其为杀目标（上游 hslingjian_chaofeng mod
                    // targetEnabled 同口径）
                    if kind.is_sha_like() {
                        let seats = core.players.len() as u8;
                        let cur = core.turn.current;
                        let holder = (0..seats).find(|p| {
                            *p != cur
                                && core.player(*p).alive
                                && core.player(*p).chaofeng_active
                                && {
                                    let d = (u16::from(*p) + u16::from(seats)
                                        - u16::from(cur))
                                        % u16::from(seats);
                                    d == 1 || d == u16::from(seats) - 1
                                }
                        });
                        if let Some(h) = holder {
                            if targets.iter().any(|t| *t != h) {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                    // 逐月（U05 xianjian 云天河，规则 v161）：本出牌阶段
                    // 使用杀须以逐月目标为目标（上游 mod playerEnabled 同
                    // 口径；白名单空=不限制）
                    if !core.turn.zhuyue_targets.is_empty()
                        && !targets
                            .iter()
                            .all(|t| core.turn.zhuyue_targets.contains(t))
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let from = core.turn.current;
                    // 贴房绅视（U05 swd，规则 v94）：本回合无视距离指定的
                    // 目标豁免距离校验（上游 targetInRange mod 同口径）
                    // 堡垒哨卫（U05 ow，规则 v125）：杀无视距离
                    let baolei_range_free = core
                        .skills
                        .keys()
                        .any(|(o, def)| {
                            *o == core.turn.current
                                && core.player(*o).baolei_mode == 1
                                && catalog.skills.get(*def as usize)
                                    .map(|sd| sd.name == "bshaowei")
                                    .unwrap_or(false)
                        });
                    // 飞刃（U05 ow 源氏，规则 v129）/飞弹（U05 ow 法老
                    // 之鹰，规则 v130）：锁定——你的杀无视距离（上游
                    // mod targetInRange 同口径）
                    let sha_range_free = catalog
                        .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
                        .map(|def| core.has_skill(core.turn.current, def))
                        .unwrap_or(false)
                        || catalog
                            .skill_by_name(crate::content::host::WUDIAN_FEIDAN_KEY)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                        // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——
                        // 杀无视距离（上游 mod targetInRange 同口径）
                        || catalog
                            .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false);
                    let feiren_range_free = sha_range_free;
                    let zfs_free = core.turn.zhufangshenshi_target == Some(target);
                    // 逐月（U05 xianjian 云天河，规则 v161）：杀向逐月目标
                    // 无距离限制（上游 mod targetInRange 同口径）
                    let zhuyue_free = core.turn.zhuyue_targets.contains(&target);
                    let dist = core.computed_distance(catalog, from, target);
                    if !zfs_free
                        && !baolei_range_free
                        && !feiren_range_free
                        && !zhuyue_free
                        && dist > core.attack_range(catalog, from)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 方天多目标：其余目标逐个查距离/存活/空城。
                    for t in targets.iter().skip(1) {
                        if !core.player(*t).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let kc = catalog
                            .skill_by_name(crate::content::standard::KONG_CHENG)
                            .map(|d| core.has_skill(*t, d))
                            .unwrap_or(false);
                        if kc && core.player(*t).hand.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let d2 = core.computed_distance(catalog, from, *t);
                        if !feiren_range_free && d2 > core.attack_range(catalog, from) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    let nature = if kind == CardKind::FireSha {
                        crate::event::DamageNature::Fire
                    } else if kind == CardKind::LeiSha {
                        crate::event::DamageNature::Thunder
                    } else {
                        crate::event::DamageNature::Normal
                    };
                    let player = core.turn.current;
                    if mark_yuanbaorou {
                        if let Some(def) = catalog
                            .skill_by_name(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
                        {
                            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                                inst.once_used = true;
                            }
                        }
                    }
                    if mark_mutoumianju {
                        if let Some(def) = catalog
                            .skill_by_name(crate::content::host::GJ_MUTOU_SKILL_KEY)
                        {
                            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                                inst.once_used = true;
                            }
                        }
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_with_nature(
                            card,
                            player,
                            targets,
                            nature,
                            Some(kind),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Chain => {
                    if targets.is_empty() || targets.len() > 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if targets.iter().any(|t| !core.player(*t).alive) {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 铁索为结算型锦囊：走 CardUsed → 无懈窗口 → ChainMachine
                    // 翻转（旧实现只推 Linked 帧，牌从未离手，ADR 0705 修复）。
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Duel => {
                    // 决斗：恰好 1 名其他存活目标，无距离/射程限制。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 空城（诸葛亮）：同杀
                    let kongcheng = catalog
                        .skill_by_name(crate::content::standard::KONG_CHENG)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if kongcheng && core.player(target).hand.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Dismantle => {
                    // 过河拆桥：弃置一名其他存活玩家的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(CardKind::Dismantle),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Snatch => {
                    // 顺手牵羊：获得距离 1 的一名其他存活角色的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 谦逊（陆逊）：不能成为顺手牵羊目标（锁定门控）
                    let qianxun = catalog
                        .skill_by_name(crate::content::standard::QIAN_XUN)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if qianxun {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let from = core.turn.current;
                    // 奇才（黄月英，锁定）：使用锦囊无距离限制
                    let qicai = catalog
                        .skill_by_name(crate::content::standard::QI_CAI)
                        .map(|d| core.has_skill(from, d))
                        .unwrap_or(false);
                    // 探囊（U05 xianjian 韩菱纱，规则 v148）：视为的顺手
                    // 牵羊无距离限制（上游 mod targetInRange 同口径；仅
                    // 声明转化命中——实体牌非顺牵即视为路径）
                    let tannang_free = catalog.card_kind(card) != CardKind::Snatch
                        && catalog
                            .skill_by_name(crate::content::host::XJ_TANNANG_KEY)
                            .map(|def| core.has_skill(from, def))
                            .unwrap_or(false);
                    if !qicai && !tannang_free && core.computed_distance(catalog, from, target) > 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = core.player(target);
                    if t.hand.is_empty() && t.equips.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(CardKind::Snatch),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Le | CardKind::Bing => {
                    // 延时锦囊：置于 1 名其他存活玩家的判定区（不立即生效）；
                    // 判定区不能有同名牌（本批接入的同名限制）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 谦逊（陆逊）：不能成为乐不思蜀目标（锁定门控；
                    // 上游无顺手牵羊，见覆盖清单）。
                    // G1 墨守宿主原语：延时锦囊目标免疫（乐/兵均免疫）
                    for t in &targets {
                        if catalog.blocks_delayed_trick(core, *t) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if kind == CardKind::Le {
                        let qianxun = catalog
                            .skill_by_name(crate::content::standard::QIAN_XUN)
                            .map(|d| core.has_skill(target, d))
                            .unwrap_or(false);
                        if qianxun {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if core
                        .player(target)
                        .judge
                        .iter()
                        .any(|c| catalog.card_kind(*c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(kind),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Longlushanfeng
                | CardKind::Haidao
                | CardKind::Shuimomuxue
                | CardKind::Feixu
                | CardKind::Yixialan
                | CardKind::Shamolvzhou
                | CardKind::Linzhongjianta
                | CardKind::Bingheyaosai
                | CardKind::Youlin
                | CardKind::Lindixiliu => {
                    // 地（mtg 首切片 v100 龙颅山峰/第二批 v101 海岛/第三批
                    // v102 水没墓穴）：无目标（notarget），每回合限一张
                    //（上游 land_used 临时技同口径）；结算在 CardUsed 帧
                    // 内核机器
                    if !targets.is_empty() || core.turn.land_used {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, targets,
                        )),
                    );
                    return Ok(());
                }
                // 无中生有/桃园结义/五谷丰登（P2/N08 缺牌种批①）：无
                // 目标锦囊，结算在 CardUsed 帧内核机器；羊皮卷（swd 第
                // 十三批 v97）对自己使用，同路；地载阵（zhenfa 首切片
                // v110）自动展开非大势力目标，同路。
                | CardKind::Wuzhong
                | CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::Yangpijuan
                | CardKind::Dizaizhen
                | CardKind::Changshezhen
                | CardKind::Pozhenjue
                | CardKind::ShanDian
                | CardKind::HongShui => {
                    // 长蛇阵：须处于队列或存在友方（上游 enable 同口径）；
                    // 破阵诀：恒可用（enable true）
                    if kind == CardKind::Changshezhen
                        && !core.inline(catalog, core.turn.current)
                        && !core
                            .alive_players()
                            .any(|p| {
                                p != core.turn.current
                                    && core.is_friend_of(catalog, core.turn.current, p)
                            })
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 地载阵：须存在非大势力存活角色（上游 enable 同口径）
                    if kind == CardKind::Dizaizhen
                        && !core
                            .alive_players()
                            .any(|p| !core.is_major_faction(catalog, p))
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 闪电/洪水：只能置入自己判定区；判定区不能有同名牌
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    if core
                        .player(player)
                        .judge
                        .iter()
                        .any(|c| catalog.card_kind(*c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen => {
                    // 阵法三张（U05 zhenfa，规则 v200/T19a；上游
                    // guozhanZhenfaEnabled=存活≥4 同口径；派生查询见
                    // state.rs）。天覆阵=AOE 全部大势力且有牌者；七星阵
                    // =自目标（须围攻/被围攻）；鸟翔阵=AOE 全部合法目
                    // 标（异势力、已知、与邻座同势力）。
                    if core.players.len() < 4 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    match kind {
                        CardKind::Tianfuzhen => {
                            if !core.alive_players().any(|p| core.is_major_faction(catalog, p)) {
                                return Err(RejectReason::IllegalTargets);
                            }
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|p| {
                                    core.is_major_faction(catalog, *p)
                                        && (!core.player(*p).hand.is_empty()
                                            || !core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        CardKind::Qixingzhen => {
                            let cur = core.turn.current;
                            let under_siege = core.alive_players().any(|q| {
                                core.sieges(catalog, q, cur)
                            });
                            let besieging = core.alive_neighbors(cur).0.is_some_and(|n| {
                                core.sieged(catalog, n)
                            }) || core.alive_neighbors(cur).1.is_some_and(|pv| {
                                core.sieged(catalog, pv)
                            });
                            if !(under_siege || besieging) || targets != [cur] {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        _ => {
                            // 鸟翔阵：targets=全部合法目标（异势力、已知、
                            // 与自身邻座同势力）
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|t| {
                                    *t != core.turn.current
                                        && !core.is_friend_of(catalog, core.turn.current, *t)
                                        && {
                                            let (n, pv) = core.alive_neighbors(*t);
                                            n.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            }) || pv.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            })
                                        }
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                }
                CardKind::Jiedao => {
                    // 持有者装备区须有武器，另一角色在其攻击范围内。
                    if targets.len() != 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let (holder, victim) = (targets[0], targets[1]);
                    if holder == core.turn.current
                        || victim == core.turn.current
                        || holder == victim
                        || !core.player(holder).alive
                        || !core.player(victim).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let has_weapon = core
                        .player(holder)
                        .equips
                        .iter()
                        .any(|c| catalog.card_kind(*c) == CardKind::Weapon);
                    let in_range = core.computed_distance(catalog, holder, victim)
                        <= core.attack_range(catalog, holder);
                    if !has_weapon || !in_range {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::SavageAttack | CardKind::ArcheryAttack => {
                    // AOE：目标 = 出牌时刻的全部其他存活玩家（自下家起按座位
                    // 顺序冻结），命令显式 targets 必须为空。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    let aoe_targets: Vec<PlayerId> = core
                        .seat_order_from(core.next_alive(player))
                        .into_iter()
                        .filter(|p| *p != player)
                        .collect();
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, aoe_targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::HorsePlus
                | CardKind::HorseMinus
                | CardKind::Weapon
                | CardKind::Armor
                | CardKind::Suolianjia
                | CardKind::WuxingPan
                | CardKind::Sanjian
                // 猴子/写轮眼/酒尾（E03）：equip5 宝物，equip 路径按 is_equip
                // 提前处理。
                | CardKind::Monkey
                | CardKind::Xuelunyang
                | CardKind::Jiuwei
                | CardKind::Mianju
                | CardKind::Renwang
                | CardKind::Qinggang
                | CardKind::Qilin
                | CardKind::Hanbing
                | CardKind::Zhuge
                | CardKind::Guanshi
                | CardKind::Qinglong
                | CardKind::Zhangba
                | CardKind::Cixiong
                | CardKind::Fangtian
                | CardKind::Chilongya
                | CardKind::Xixueguizhihuan
                | CardKind::Shentoumianju
                | CardKind::Xingjunyan
                | CardKind::Qinglonglingzhu => {
                    // 占栏装备：equip 路径在上方按 is_equip 提前处理；
                    // 此臂仅为穷尽性——正常不会到达
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Guiyoujie => {
                    // 归有界（U05 swd，规则 v91）：他人延时锦囊——恰 1 名
                    // 其他存活目标，且其判定区无同名延时
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                        || core.player(targets[0])
                            .judge
                            .iter()
                            .any(|c| catalog.card_kind(*c) == CardKind::Guiyoujie)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhufangshenshi => {
                    // 贴房绅视（U05 swd 第十批，规则 v94）：恰 1 名其他
                    // 存活目标（上游 filterTarget target !== player）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Fengxueren
                | CardKind::Yufulu
                | CardKind::Daihuofenglun
                | CardKind::Xiayuncailing
                | CardKind::Langeguaiyi => {
                    // 装备牌（风雪刃 equip1/御夫录·蓝格怪衣 equip5/带火
                    // 风轮 equip4/霞云彩绫 equip3）：走装配路径（is_equip
                    // 提前处理），出牌区使用为穷尽性占位拒绝
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Caoyao => {
                    // 草药（U05 swd 第三批，规则 v87）：为距离 1 以内的
                    // 受伤其他目标回复 1 点体力（上游 range.global 1 +
                    // filterTarget hp<maxHp + recover）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = targets[0];
                    let tp = core.player(t);
                    if !tp.alive || tp.hp >= tp.max_hp as i32 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    if core.computed_distance(catalog, user, t) > 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Tao | CardKind::Hufu => {
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    let p = core.player(user);
                    // 护符（U05 swd 首切片，规则 v85）：仅濒死（自己进入
                    // 濒死流程）时可用；非濒死拒绝（上游 savable 守卫）
                    if kind == CardKind::Hufu && !p.dying {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if p.hp >= p.max_hp as i32 {
                        return Err(RejectReason::HpFull);
                    }
                    // E03 猴子偷桃：他人装备猴子时开「使用前取消窗口」
                    // （命令层挂起机器，ActivateSkill 槽模式）——确认者弃
                    // 猴子把桃收为手牌（使用不发生，无 CardUsed 帧）；
                    // 全部放弃则按普通桃结算。
                    let monkey_def = catalog
                        .skill_by_name(crate::content::host::HUANLE_MONKEY_KEY);
                    let holders: Vec<u8> = monkey_def
                        .map(|d| {
                            core.skills
                                .range(..)
                                .filter(|((o, idef), inst)| {
                                    *idef == d && inst.enabled && core.player(*o).alive && *o != user
                                })
                                .map(|((o, _), _)| *o)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if !holders.is_empty() {
                        state.resolution.parks.push(crate::resolution::Park {
                            resume_depth: state.resolution.stack.len(),
                            slot: crate::resolution::Slot::Intrinsic {
                                frame: crate::resolution::NO_FRAME,
                            },
                            machine: Box::new(crate::kernel::MonkeyStealMachine::new(
                                card, user, holders,
                            )),
                        });
                        return Ok(());
                    }
                }
                CardKind::Jiu | CardKind::Tianxianjiu => {
                    // 酒/天仙酒（swd 变体，规则 v89；登记差异：与酒共享
                    // 每回合一次计数）：出牌阶段对自身使用，每回合限一次
                    // ——本回合使用的下一张杀伤害 +1；濒死使用走濒死救援
                    // 通道（不经此命令）。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if core.turn.jiu_used {
                        return Err(RejectReason::CardLimit);
                    }
                }
                CardKind::ShouLiJian => {
                    // 手里剑（G1 欢乐卡牌）：恰 1 名其他存活目标（无距离限制，
                    // 上游 outrange global）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                // 闪/无懈可击：仅在响应窗口打出，不可主动使用
                CardKind::Shan | CardKind::Nullify => return Err(RejectReason::WrongPhase),
                CardKind::Shuigong => {
                    // 水攻：所有有装备的存活角色（多目标）
                    for t in &targets {
                        if !core.player(*t).alive || core.player(*t).equips.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Fudichouxin => {
                    // 釜底抽薪：恰 1 名其他存活目标，双方均有手牌（可拼点）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                        || core.player(core.turn.current).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Chenhuodajie => {
                    // 趁火打劫（G1 运筹帷幄）：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Linghunzhihuo => {
                    // 灵魂之火：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhaomingdan => {
                    // 照明弹：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jihuocard => {
                    // 急耗卡：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Youdishenru => {
                    // 诱敌深入：仅作为杀的响应打出（响应窗口内），不可主动使用
                    return Err(RejectReason::WrongPhase);
                }
                CardKind::Caochuanjiejian => {
                    // 草船借箭：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shezhanqunru => {
                    // 舌战群儒：自-target；目标=所有有手牌的其他角色
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Geanguanhuo => {
                    // 隔岸观火：恰 2 名其他存活且有手牌的目标
                    if targets.len() != 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    for t in &targets {
                        if *t == core.turn.current
                            || !core.player(*t).alive
                            || core.player(*t).hand.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Toulianghuanzhu => {
                    // 偷梁换柱（G1 运筹帷幄）：恰 1 名其他存活且有手牌的目标；
                    // 己方也需有手牌（各选一张交换）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                        || core.player(core.turn.current).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::WangmeiZhike => {
                    // 望梅止渴（G1 玩点论杀）：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shujinsan => {
                    // 舒筋散（G1 古剑）：恰 1 名存活且有手牌的目标（可含自己；
                    // 登记差异：上游含装备区，原生仅手牌）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Mianlijinzhen => {
                    // 面里金针（G1 古剑）：恰 1 名体力值不小于使用者的存活目标
                    //（可含自己）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hp < core.player(core.turn.current).hp
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Feibiao => {
                    // 飞镖（G1 古剑）：恰 1 名其他存活目标（无距离限制）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jinlianzhu => {
                    // 金莲烛（G1 古剑）：响应牌，不能主动使用——仅经
                    // 伤害前置监听技打出
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Ziyangdan => {
                    // 紫阳丹：恰 1 名存活目标（可含自己；可叠加）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shihuifen | CardKind::Liufengsan => {
                    // 石会粉/六风散：恰 1 名其他存活目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shatang | CardKind::Liutouge => {
                    // 砂糖/硫头阁：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Heilonglinpian => {
                    // 黑龙鳞片：自-target（对自己使用）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Mutoumianju => {
                    // 木头面具：装备牌，无目标（走装备路径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Dinvxuanshuang => {
                    // 帝女玄霜：无目标、对自己使用、需已受伤（桃同款校验）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let p = core.player(core.turn.current);
                    if p.hp >= p.max_hp as i32 {
                        return Err(RejectReason::HpFull);
                    }
                }
                CardKind::Gjyuheng => {
                    // 玉衡：装备牌，无目标（走装备路径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Dunpaigedang
                | CardKind::Xingjiegoutong
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren => {
                    // 炉石自-target 锦囊：无目标
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhiliaobo => {
                    // 治疗波：恰 1 名已受伤的存活目标（可含自己）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hp >= core.player(targets[0]).max_hp as i32
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Yuansuhuimie => {
                    // 元素毁灭：自-target（AOE：所有其他存活角色）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shijieshu => {
                    // 世界树：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Kunenfayin | CardKind::Poxiao => {
                    // 昆恩/破晓：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zumoshoukao => {
                    // 祖墨手铠：恰 1 名其他存活目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shanbengshu => {
                    // 山崩术：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Yanziyaoshui
                | CardKind::Aozuzhilei
                | CardKind::Wenyi
                | CardKind::Zhihuanjun
                | CardKind::Baoxueyaoshui
                | CardKind::Liuxinghuoyu => {
                    // gwent 法术：恰 1 名存活目标
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jiguan => {
                    // 机关牌（swd 第十六~十八批，规则 v189/v190/v191）：
                    // 指路小虎/木甲人=自目标（上游 selectTarget -1+
                    // filterTarget target===player 同口径）；木甲人
                    // usable:1 每回合一次；勾魂锣=恰 1 名无挂附目标
                    // （上游 filterTarget !hasSkill("gouhunluo") 同口径）
                    if catalog.cards[card as usize].name.contains("jiguantong") {
                        // 机关桶：AOE=全部其他存活角色（上游 selectTarget
                        // -1 target!==player 同口径；结算序=座位序登记差异）
                        let expected: Vec<u8> = core
                            .seat_order_from(core.turn.current)
                            .into_iter()
                            .filter(|p| *p != core.turn.current && core.player(*p).alive)
                            .collect();
                        if targets != expected {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("gouhunluo") {
                        if targets.len() != 1 || !core.player(targets[0]).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if core.player(targets[0]).gouhunluo_rounds > 0 {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanfeng") {
                        // 机关凤（swd 第二十批，规则 v195）：恰 1 名其他
                        // 角色（上游 filterTarget target!==player 同口径）
                        if targets.len() != 1
                            || targets[0] == core.turn.current
                            || !core.player(targets[0]).alive
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanyaoshu") {
                        // 机关妖术（swd 第二十二批，规则 v197）：恰 1 名其
                        // 他存活角色（range.global 1 无距离约束）；目标有空
                        // 装备栏（上游 filterTarget 空 subtype 槽同口径）、
                        // 无防重标记（上游 !hasSkill 同口径）、非 isMin
                        //（手牌数>体力，上游 !isMin 同口径）
                        if targets.len() != 1 || !core.player(targets[0]).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let t = targets[0];
                        let marker = catalog
                            .skill_by_name(crate::content::host::SWD_JIGUANYAOSHU_STATE_KEY);
                        if marker.is_some_and(|d| core.skills.contains_key(&(t, d))) {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let hand = core.player(t).hand.len() as i32;
                        if hand <= core.player(t).hp {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let occupied: Vec<Option<crate::catalog::EquipSlot>> = core
                            .player(t)
                            .equips
                            .iter()
                            .map(|c| catalog.card_kind(*c).equip_slot())
                            .collect();
                        if occupied.len() >= 5 {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanshu") {
                        // 机关术（swd 机关术批，规则 v199/ADR 0780）：自目
                        // 标（上游 filterTarget target===player 同口径）；
                        // 至少一张有效装备
                        if targets.len() != 1 || targets[0] != core.turn.current {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if core.player(core.turn.current).equips.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanyuan") {
                        // 机关怨（swd 第二十一批，规则 v196）：恰 1 名其他
                        // 角色（上游 filterTarget target!==player+
                        // !hasSkill("jiguanyuan") 同口径）；enable 要求除本
                        // 体至少还有一张手牌（上游 enable 同口径）
                        if targets.len() != 1
                            || targets[0] == core.turn.current
                            || !core.player(targets[0]).alive
                            || !core.player(targets[0]).jiguanyuan_cards.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let hand = core.player(core.turn.current).hand.clone();
                        if hand.is_empty()
                            || (hand.len() == 1 && hand[0] == card)
                        {
                            return Err(RejectReason::CardLimit);
                        }
                    } else {
                        if targets != [core.turn.current] {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if catalog.cards[card as usize].name.contains("mujiaren")
                            && core.turn.mujiaren_used
                        {
                            return Err(RejectReason::CardLimit);
                        }
                    }
                }
                CardKind::Hslingjian => {
                    // 灵剑（swd 机关术批，规则 v198）：逐卡目标规则（上游
                    // 各卡 filterTarget 同口径）
                    let cname = &catalog.cards[card as usize].name;
                    let t_ok = if cname.contains("xingtigaizao") {
                        targets.len() == 1 && targets[0] == core.turn.current
                    } else if cname.contains("shijianhuisu") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).equips.is_empty()
                    } else if cname.contains("jinjilengdong") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).face_down
                    } else if cname.contains("yinmilichang") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).yinmi_active
                            && !core.player(targets[0]).yinshen_active
                    } else if cname.contains("shengxiuhaojiao") {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).chaofeng_active
                    } else if cname.contains("zhongxinghujia") {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && core.player(targets[0]).hand.len() as i32
                                > core.player(targets[0]).hp
                    } else {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && (!core.player(targets[0]).hand.is_empty()
                                || !core.player(targets[0]).equips.is_empty())
                    };
                    if !t_ok {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Qinpendayu | CardKind::Birinongwu | CardKind::Ciguhanshuang => {
                    // gwent 天气：恰 1 名其他存活目标（邻座结算侧展开；
                    // 已持有状态的目标由授予机跳过）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Bingpotong => {
                    // 冰魄通：1–3 名其他存活且有手牌的目标
                    if targets.is_empty() || targets.len() > 3 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    for t in &targets {
                        if *t == core.turn.current
                            || !core.player(*t).alive
                            || core.player(*t).hand.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Qiankunbiao | CardKind::Shenhuofeiya => {
                    // 乾坤镖/神火飞鸦：恰 1 名其他存活目标（邻座由结算
                    // 侧按距离展开）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }

                CardKind::Longxugou => {
                    // 龙须钩：恰 1 名其他存活且有装备的目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).equips.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Tanhuadong
                | CardKind::Yougeng
                | CardKind::Qingtuan
                | CardKind::Yuanbaorou
                | CardKind::Gudonggeng
                | CardKind::Mapodoufu
                | CardKind::Chunbing
                | CardKind::Liyutang
                | CardKind::Molicha
                | CardKind::Xiajiao
                | CardKind::Mizhilianou
                | CardKind::Luyugeng
                | CardKind::Yunvyuanshen => {
                    // 古剑食物：恰 1 名存活目标（可含自己），且目标尚
                    // 未持有同名状态（登记差异：上游按技能名过滤）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let listener = match kind {
                        CardKind::Tanhuadong => crate::content::host::GJ_FOOD_TANHUADONG_KEY,
                        CardKind::Qingtuan => crate::content::host::GJ_FOOD_QINGTUAN_KEY,
                        CardKind::Yuanbaorou => crate::content::host::GJ_FOOD_YUANBAOROU_KEY,
                        CardKind::Gudonggeng => crate::content::host::GJ_FOOD_GUDONGGENG_KEY,
                        CardKind::Mapodoufu => crate::content::host::GJ_FOOD_MAPODOUFU_KEY,
                        CardKind::Chunbing => crate::content::host::GJ_FOOD_CHUNBING_KEY,
                        CardKind::Liyutang => crate::content::host::GJ_FOOD_LIYUTANG_KEY,
                        CardKind::Molicha => crate::content::host::GJ_FOOD_MOLICHA_KEY,
                        CardKind::Xiajiao => crate::content::host::GJ_FOOD_XIAJIAO_KEY,
                        CardKind::Mizhilianou => crate::content::host::GJ_FOOD_MIZHLIANOU_KEY,
                        CardKind::Luyugeng => crate::content::host::GJ_FOOD_LUYUGENG_KEY,
                        CardKind::Yunvyuanshen => crate::content::host::GJ_YUNVYUANSHEN_LISTENER_KEY,
                        _ => crate::content::host::GJ_FOOD_YOUGENG_KEY,
                    };
                    if let Some(def) = catalog.skill_by_name(listener) {
                        if core.skills.contains_key(&(targets[0], def)) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
            }
            let player = core.turn.current;
            // view-as 声明（蜜链藕红桃当桃等）：转化后按声明牌种结算
            let use_as = if kind != catalog.card_kind(card) {
                Some(kind)
            } else {
                None
            };
            crate::resolution::push_frame(
                &mut state.resolution,
                EventData::CardUsed(crate::machine::card_use_as(
                    card, player, targets, use_as,
                )),
            );
            Ok(())
        }
        GameAction::NextPhase => {
            let core = &state.core;
            // 授权：只有当前回合玩家可以推进自己的阶段。
            if actor != core.turn.current {
                return Err(RejectReason::NotYourTurn);
            }
            let player = core.turn.current;
            let phase = core.turn.phase;
            crate::resolution::push_frame(
                &mut state.resolution,
                EventData::Phase {
                    player,
                    phase,
                    begin: false,
                },
            );
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// 确定性摘要：blake3 编码全部状态字段。执行位置含开放选择、挂起元数据与
// 每台挂起机器的**规范快照**（MachineSnapshot 全字段——不同机器的局部状态
// 可区分）；摘要以 ruleset_version 开头（规则版本绑定：跨规则版本的哈希
// 不互认）。重放一致性以静止点哈希核对。
// ---------------------------------------------------------------------------

fn canonical_hash(state: &GameState) -> [u8; 32] {
    let mut h = Hasher::new();
    let c = &state.core;
    push_u64(&mut h, c.revision);
    push_u64(&mut h, c.ruleset_version);
    match &c.mode {
        GameMode::Prototype => push_u8(&mut h, 1),
        GameMode::Kingdoms => push_u8(&mut h, 3),
        GameMode::Versus { teams, substitutes } => {
            push_u8(&mut h, 4);
            push_u32(&mut h, teams.len() as u32);
            for t in teams {
                push_players(&mut h, t);
            }
            // 替补武将池（v206）入确定性摘要
            push_u32(&mut h, substitutes.len() as u32);
            for pool in substitutes {
                push_u32(&mut h, pool.len() as u32);
                for g in pool {
                    push_u32(&mut h, *g as u32);
                }
            }
        }
        GameMode::Doudizhu { lord } => {
            push_u8(&mut h, 5);
            push_u8(&mut h, *lord);
        }
        GameMode::Boss { boss, chongzheng, fellows } => {
            push_u8(&mut h, 6);
            push_u8(&mut h, *boss);
            push_u8(&mut h, *chongzheng);
            push_players(&mut h, fellows);
        }
        GameMode::Kof { pools, wins, games } => {
            push_u8(&mut h, 7);
            for p in pools {
                push_u32(&mut h, p.len() as u32);
                for g in p {
                    push_u32(&mut h, *g as u32);
                }
            }
            push_u32(&mut h, wins[0]);
            push_u32(&mut h, wins[1]);
            push_u32(&mut h, *games);
        }
        GameMode::Identity { roles } => {
            push_u8(&mut h, 2);
            push_u32(&mut h, roles.len() as u32);
            for r in roles {
                push_u8(
                    &mut h,
                    match r {
                        crate::state::Role::Lord => 1,
                        crate::state::Role::Loyalist => 2,
                        crate::state::Role::Rebel => 3,
                        crate::state::Role::Traitor => 4,
                    },
                );
            }
        }
    }
    for p in &c.players {
        push_u8(&mut h, p.alive as u8);
        push_i32(&mut h, p.hp);
        push_u32(&mut h, p.max_hp);
        push_u8(&mut h, p.dying as u8);
        push_u8(&mut h, p.linked as u8);
        // 国战双将槽/明置状态入哈希（None 编码为 0）
        for slot in 0..2 {
            match p.general_slots[slot] {
                Some(g) => {
                    push_u8(&mut h, 1);
                    push_u16(&mut h, g);
                }
                None => push_u8(&mut h, 0),
            }
        }
        push_u8(&mut h, p.revealed_generals[0] as u8);
        push_u8(&mut h, p.revealed_generals[1] as u8);
        push_u8(&mut h, p.wunature.unwrap_or(255));
        // 天符挂附（规则 v188）
        push_u8(&mut h, p.ctianfu_source.unwrap_or(255));
        // 勾魂锣挂附（规则 v191）
        push_u32(&mut h, p.gouhunluo_rounds);
        push_u8(&mut h, p.gouhunluo_source.unwrap_or(255));
        push_u8(&mut h, p.ctianfu_card.map(|_| 1).unwrap_or(0));
        if let Some(v) = p.ctianfu_card {
            push_u32(&mut h, v);
        }
        push_u8(&mut h, p.hujia);
        // E03 翻面原语：武将牌朝向入摘要
        push_u8(&mut h, p.face_down as u8);
        push_u8(&mut h, p.boss_chongzheng);
        push_u32(&mut h, p.zhucheng.len() as u32);
        for c in &p.zhucheng {
            push_u32(&mut h, *c);
        }
        push_u8(&mut h, p.paotai_lv);
        push_u8(&mut h, p.baolei_mode);
        // 暴雪/急冻（U05 ow 美，规则 v127）
        push_u8(&mut h, p.baoxue_used as u8);
        push_u8(&mut h, p.jidong_active as u8);
        // 强固（U05 ow 奥丽莎，规则 v136）
        push_u8(&mut h, p.qianggu_active as u8);
        // 屏障（U05 ow 查莉娅，规则 v138）
        push_u8(&mut h, p.pingzhang_self_used as u8);
        push_u8(&mut h, p.pingzhang_other_used as u8);
        push_u8(&mut h, p.pingzhang_count);
        // 潜行（U05 ow 黑影，规则 v139）
        push_u8(&mut h, p.yinshen_active as u8);
        // 机甲血（U05 ow D.Va，规则 v141）
        push_u8(&mut h, p.jijia_hp);
        // 风雅颂计数器（U05 xianjian 雷元戈，规则 v150）
        push_u8(&mut h, p.feng_count);
        push_u8(&mut h, p.ya_count);
        push_u8(&mut h, p.song_count);
        // 蛊舞剩余回合数（U05 xianjian 阿奴，规则 v154）
        push_u8(&mut h, p.anwugu_count);
        // 龙火已觉醒（U05 xianjian 姜城，规则 v157）
        push_u8(&mut h, p.jiangcheng_longhuo as u8);
        // 月行对偶（U05 xianjian 龙幽，规则 v159）
        push_u8(&mut h, p.yuexing_pair.map(|x| x + 1).unwrap_or(0));
        // 明绣（U05 xianjian 明绣，规则 v160）
        push_u8(&mut h, p.turn_damage);
        push_u32(&mut h, p.linghuo_ready_round);
        push_u32(&mut h, p.guijin_ready_round);
        push_u8(&mut h, p.skip_next_draw as u8);
        // 贯日（U05 xianjian 云天河，规则 v161）
        push_u8(&mut h, p.guanri_used as u8);
        // 离场区+水云计数+忘忧戳（U05 xianjian 紫萱，规则 v162）
        push_u32(&mut h, p.off_court.len() as u32);
        for c in &p.off_court {
            push_u32(&mut h, *c);
        }
        push_u8(&mut h, p.shuiyun_count);
        push_u32(&mut h, p.wangyou3_round);
        // 小蛮（U05 xianjian 小蛮，规则 v163）
        push_u32(&mut h, p.total_used);
        push_u8(&mut h, p.lingquan_awakened as u8);
        push_u8(&mut h, p.shenwu_awakened as u8);
        // 诚信（U05 xianjian 明绣，规则 v164）
        push_u8(&mut h, p.chengxin_active as u8);
        push_u32(&mut h, p.chengxin_ready_round);
        // 虎煞（U05 xianjian 王小虎，规则 v169）
        push_u8(&mut h, p.husha_mark);
        // 适盈（U05 xianjian 月琦，规则 v171）
        push_u8(&mut h, p.death_cancelled as u8);
        push_u8(&mut h, p.shiying_used as u8);
        // 花好（U05 xianjian 苏媚，规则 v175）
        push_u8(&mut h, p.huahu_used as u8);
        // 玄砚计数（U05 xianjian 慕容紫英，规则 v179）
        push_u32(&mut h, p.xuanning_mark);
        push_cards(&mut h, &p.hand);
        push_cards(&mut h, &p.equips);
        push_cards(&mut h, &p.judge);
    }
    push_cards(&mut h, &c.deck);
    push_cards(&mut h, &c.discard);
    push_cards(&mut h, &c.table);
    let (rs, rc) = c.rng.state();
    push_u64(&mut h, rs);
    push_u64(&mut h, rc);
    push_u8(&mut h, c.turn.current);
    push_u8(&mut h, c.turn.phase as u8);
    push_u32(&mut h, c.turn.round);
    push_u8(&mut h, c.turn.sha_used_play_phase as u8);
    push_u8(&mut h, c.turn.rende_given);
    push_u8(&mut h, c.turn.jiu_used as u8);
    // 木甲人限次（规则 v190）
    push_u8(&mut h, c.turn.mujiaren_used as u8);
    push_u8(&mut h, c.turn.jiu_active as u8);
    push_u8(&mut h, c.turn.land_used as u8);
    push_u8(&mut h, c.turn.land_sha_used as u8);
    push_u32(&mut h, c.turn.discard_phase_discarded);
    push_u32(&mut h, c.turn.ciqiu_dying.len() as u32);
    for p in &c.turn.ciqiu_dying {
        push_u8(&mut h, *p);
    }
    match c.turn.kangyin2 {
        None => push_u8(&mut h, 0),
        Some((o, x)) => {
            push_u8(&mut h, 1);
            push_u8(&mut h, o);
            push_u32(&mut h, x);
        }
    }
    push_u32(&mut h, c.turn.cards_used_play_phase);
    push_u32(&mut h, c.turn.phase_number);
    // 贴房绅视：本回合无视距离目标（规则 v94）
    match c.turn.zhufangshenshi_target {
        None => push_u8(&mut h, 0),
        Some(t) => {
            push_u8(&mut h, 1);
            push_u8(&mut h, t);
        }
    }
    push_u8(&mut h, c.turn.turn_opened as u8);
    push_u8(&mut h, c.turn.skip_draw_phase as u8);
    push_u8(&mut h, c.turn.skip_play_phase as u8);
    // 乐动三角（U07 ow 卢西奥，规则 v126）
    push_u8(&mut h, c.turn.yuedong_num);
    push_u8(&mut h, c.turn.yuedong_eff);
    push_u8(&mut h, c.turn.yuedong_recover as u8);
    // 链钩（U05 ow 路霸，规则 v128）
    match c.turn.liangou_hook {
        None => push_u8(&mut h, 255),
        Some(t) => push_u8(&mut h, t),
    }
    push_u8(&mut h, c.turn.liangou_bonus_used as u8);
    // 跳跃（U05 ow 温斯顿，规则 v137）
    match c.turn.feitiao_target {
        None => push_u8(&mut h, 255),
        Some(t) => push_u8(&mut h, t),
    }
    // 返灵（U05 xianjian 瑕，规则 v145）
    push_u8(&mut h, c.turn.fanling_used as u8);
    // 御风（U05 xianjian 李逍遥，规则 v151）
    push_u8(&mut h, c.turn.xjyufeng_used);
    // 蝶舞（U05 xianjian 龙葵，规则 v153）
    push_u8(&mut h, c.turn.diewu_draw_used as u8);
    // 掠影（U05 xianjian 慕长兰，规则 v154）
    push_u8(&mut h, c.turn.lueying_used as u8);
    // 甜香去（U05 xianjian 唐雨柔，规则 v156）
    push_u8(&mut h, c.turn.txianqu_used as u8);
    // 聚宝（U05 xianjian 景天，规则 v158）
    push_u8(&mut h, c.turn.jtjubao_used as u8);
    // 逐月（U05 xianjian 云天河，规则 v161）
    push_u32(&mut h, c.turn.zhuyue_targets.len() as u32);
    for p in &c.turn.zhuyue_targets {
        push_u8(&mut h, *p);
    }
    // 摄灵（U05 xianjian 南宫煌，规则 v168）
    push_u8(&mut h, c.turn.sheling_used);
    // 射影（U05 xianjian 苏媚，规则 v175）
    push_u8(&mut h, c.turn.sheying_used as u8);
    // 张目（U05 xianjian 南宫煌，规则 v181）
    push_u8(&mut h, c.turn.zhangmu_used as u8);
    // 影锋链挡旗标（规则 v185）
    push_u8(&mut h, c.turn.yingfeng_chain as u8);
    // 乱剑载体锚定（规则 v187）
    match c.turn.luanjian_card {
        None => push_u8(&mut h, 0),
        Some(v) => {
            push_u8(&mut h, 1);
            push_u32(&mut h, v);
        }
    }
    for ((owner, def), inst) in &c.skills {
        push_u8(&mut h, *owner);
        push_u16(&mut h, *def);
        push_u8(&mut h, inst.enabled as u8);
        push_u8(&mut h, inst.once_used as u8);
        push_u8(&mut h, inst.expires_end_of_turn as u8);
        push_u8(&mut h, inst.rounds);
    }
    // view-as 声明牌种表入哈希（防篡改完整性，BTreeMap 键序确定性）
    for (card, kind) in &c.declared_kinds {
        push_u32(&mut h, *card);
        push_u8(&mut h, *kind as u8);
    }
    // 五行属性表入哈希（G1 wuxing，BTreeMap 键序确定性）
    for (card, nature) in &c.card_natures {
        push_u32(&mut h, *card);
        push_u8(&mut h, *nature);
    }
    // 当前地（mtg land，规则 v100）：仅编码牌种枚举序（与 archive 的
    // card_kind_to_num 同表，哈希口径只需引擎内自洽）
    match c.current_land {
        None => push_u8(&mut h, 0),
        Some(kind) => {
            push_u8(&mut h, 1);
            push_u32(&mut h, (kind as usize) as u32);
        }
    }
    match &c.outcome {
        None => push_u8(&mut h, 0),
        Some(o) => {
            push_u8(&mut h, 1);
            match o.winner {
                crate::state::Winner::Player(w) => {
                    push_u8(&mut h, 1);
                    push_u8(&mut h, w);
                }
                crate::state::Winner::LordSide => push_u8(&mut h, 2),
                crate::state::Winner::Rebels => push_u8(&mut h, 3),
                crate::state::Winner::Faction(f) => {
                    push_u8(&mut h, 4);
                    push_u8(&mut h, f.clone() as u8);
                }
                crate::state::Winner::Versus(t) => {
                    push_u8(&mut h, 6);
                    push_u8(&mut h, t as u8);
                }
                crate::state::Winner::Doudizhu { lord_won } => {
                    push_u8(&mut h, 7);
                    push_u8(&mut h, lord_won as u8);
                }
                crate::state::Winner::Boss { boss_won } => {
                    push_u8(&mut h, 8);
                    push_u8(&mut h, boss_won as u8);
                }
                crate::state::Winner::Traitor => push_u8(&mut h, 5),
            }
        }
    }
    let r = &state.resolution;
    push_u32(&mut h, r.next_event_id);
    push_u64(&mut h, r.next_choice_id);
    push_u32(&mut h, r.budget);
    for f in &r.stack {
        push_u32(&mut h, f.id);
        push_u32(&mut h, f.parent.unwrap_or(0));
        push_u8(&mut h, f.parent.is_none() as u8);
        push_u8(&mut h, f.kind as u8);
        push_u8(&mut h, f.stage as u8);
        push_u8(&mut h, f.planned as u8);
        push_u8(&mut h, f.intrinsic_done as u8);
        push_u32(&mut h, f.queue.len() as u32);
        for q in &f.queue {
            push_u16(&mut h, q.def);
            push_u8(&mut h, q.owner);
            push_u8(&mut h, q.order.0);
            push_i32(&mut h, q.order.1);
            push_u8(&mut h, q.order.2);
            push_u8(&mut h, q.order.3);
        }
        push_event_data(&mut h, &f.data);
    }
    match &r.open_choice {
        None => push_u8(&mut h, 0),
        Some(o) => {
            push_u8(&mut h, 1);
            push_u64(&mut h, o.id);
            push_u8(&mut h, o.observer);
            push_u8(&mut h, o.spec.kind as u8);
            push_static(&mut h, o.spec.prompt);
            push_u8(&mut h, o.spec.min as u8);
            push_u8(&mut h, o.spec.max as u8);
            push_u8(&mut h, o.spec.allow_cancel as u8);
            push_options(&mut h, &o.spec.options);
        }
    }
    match &r.answer {
        None => push_u8(&mut h, 0),
        Some(a) => {
            push_u8(&mut h, 1);
            push_options(&mut h, a);
        }
    }
    for p in &r.parks {
        push_usize(&mut h, p.resume_depth);
        match p.slot {
            crate::resolution::Slot::Intrinsic { frame } => {
                push_u8(&mut h, 1);
                push_u32(&mut h, frame);
            }
            crate::resolution::Slot::Trigger { frame } => {
                push_u8(&mut h, 2);
                push_u32(&mut h, frame);
            }
        }
        push_machine(&mut h, &p.machine.snapshot());
    }
    h.finalize().into()
}

/// 机器规范快照的摘要编码：判别符 + 全字段；嵌套机器递归。
fn push_machine(h: &mut Hasher, s: &crate::snap::MachineSnapshot) {
    use crate::snap::MachineSnapshot as M;
    match s {
        M::Sha { card, source, target, nature, asked, armor_judged, dodges_used, tieji_red, liuli_done, liuli_stage, liuli_card, hujia_queue, hujia_holder, jiu_bonus, asking_qilin, qilin_horse, asking_hanbing, asking_fengxueren, asking_guanshi, guanshi_cards, asking_qinglong, asking_cixiong, cixiong_done, extra_targets, hanbing_pending, asking_xiandan, xiandan_done, xiandan_bonus, asking_lueying, lueying_victim, lueying_declined } => {
            push_u8(h, 1);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *nature as u8);
            push_u8(h, *asked as u8);
            // 八卦阵判定标志入哈希（防篡改完整性）
            push_u8(h, *armor_judged as u8);
            // 无双已用闪数入哈希
            push_u8(h, *dodges_used as u8);
            // 铁骑判定结果入哈希
            push_u8(h, match tieji_red {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
            // 流离/护驾状态入哈希（ADR 0774/0776 消除恢复分歧）
            push_u8(h, *liuli_done as u8);
            push_u8(h, *liuli_stage);
            match liuli_card {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            // 护驾逐人询问队列入哈希
            match hujia_queue {
                None => push_u8(h, 0),
                Some(q) => {
                    push_u8(h, 1);
                    push_players(h, q);
                }
            }
            push_u8(h, hujia_holder.map(|p| p + 1).unwrap_or(0));
            // 酒效果入哈希（防篡改完整性）
            push_u8(h, (*jiu_bonus) as u8);
            // 麒麟弓弃马询问状态入哈希
            push_u8(h, (*asking_qilin) as u8);
            match qilin_horse {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            // 方天画戟剩余目标队列入哈希
            push_u32(h, extra_targets.len() as u32);
            for c in extra_targets {
                push_u32(h, *c);
            }
            // 寒冰剑防止伤害确认状态入哈希
            push_u8(h, (*asking_hanbing) as u8);
            // 风雪刃翻面+摸牌询问状态入哈希（规则 v95）
            push_u8(h, (*asking_fengxueren) as u8);
            // 贯石斧弃二强伤询问状态入哈希
            push_u8(h, (*asking_guanshi) as u8);
            push_u32(h, guanshi_cards.len() as u32);
            for c in guanshi_cards {
                push_u32(h, *c);
            }
            // 青龙再杀询问状态入哈希
            push_u8(h, (*asking_qinglong) as u8);
            // 雌雄双股剑状态入哈希
            push_u8(h, (*asking_cixiong) as u8);
            push_u8(h, (*cixiong_done) as u8);
            // 寒冰剑目标自选弃牌剩余次数入哈希（逐技对照批一 v75）
            push_u8(h, *hanbing_pending);
            push_u8(h, *asking_xiandan as u8);
            push_u8(h, *xiandan_done as u8);
            push_u8(h, *xiandan_bonus as u8);
            push_u8(h, *asking_lueying as u8);
            match *lueying_victim {
                None => push_u8(h, 255),
                Some(t) => push_u8(h, t as u8),
            }
            push_u8(h, *lueying_declined as u8);
        }
        M::Zhangba { owner, cards, stage } => {
            push_u8(h, 61);
            push_u8(h, *owner);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
            push_u8(h, *stage as u8);
        }
        M::BossXiangde { owner } => {
            push_u8(h, 62);
            push_u8(h, *owner);
        }
        M::BossFutai { owner } => {
            push_u8(h, 63);
            push_u8(h, *owner);
        }
        M::DyingSave { dying, source, responders, asked, round_used } => {
            push_u8(h, 2);
            push_u8(h, *dying);
            push_u8(h, source.map(|p| p + 1).unwrap_or(0));
            push_players(h, responders);
            push_u8(h, *asked as u8);
            // 轮次用桃标志入哈希（防篡改完整性）
            push_u8(h, *round_used as u8);
        }
        M::Duel { card, source, target, on_target, asked, second_pending } => {
            push_u8(h, 3);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *on_target as u8);
            push_u8(h, *asked as u8);
            push_u8(h, *second_pending as u8);
        }
        M::NullifyWindow { card, source, nullified, queue, asked, window_open, owns_disposal, effect } => {
            push_u8(h, 4);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *nullified as u8);
            push_players(h, queue);
            push_u8(h, *asked as u8);
            push_u8(h, *window_open as u8);
            push_u8(h, *owns_disposal as u8);
            match effect {
                None => push_u8(h, 0),
                Some(e) => {
                    push_u8(h, 1);
                    push_machine(h, e);
                }
            }
        }
        M::JudgePhase { owner, queue, current } => {
            push_u8(h, 5);
            push_u8(h, *owner);
            push_cards(h, queue);
            match current {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, c.trick);
                    push_u32(h, c.judge_card);
                    push_players(h, &c.rewriters);
                    push_u8(h, c.asked as u8);
                    push_players(h, &c.tian_du);
                    push_u8(h, c.gained as u8);
                    push_u8(h, c.in_rewrite as u8);
                    // 判定确定/机器持牌标志入哈希：防篡改完整性（N08-d）。
                    push_u8(h, c.finalized as u8);
                    push_u8(h, c.judge_held as u8);
                }
            }
        }
        M::Noop => push_u8(h, 18),
        M::Aoe { card, source, need, targets, idx, inner } => {
            push_u8(h, 6);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *need as u8);
            push_players(h, targets);
            push_usize(h, *idx);
            match inner {
                None => push_u8(h, 0),
                Some(m) => {
                    push_u8(h, 1);
                    push_machine(h, m);
                }
            }
        }
        M::AoeTarget { source, target, need, asked, card } => {
            push_u8(h, 15);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *need as u8);
            push_u8(h, *asked as u8);
            push_u32(h, *card);
        }
        M::Chain { card, source, targets, emitted } => {
            push_u8(h, 16);
            push_u32(h, *card);
            push_u8(h, *source);
            push_players(h, targets);
            push_usize(h, *emitted);
        }
        M::PlaceDelayed { card, target, placed } => {
            push_u8(h, 17);
            push_u32(h, *card);
            push_u8(h, *target);
            push_u8(h, *placed as u8);
        }
        M::DiscardToLimit { player, need, asked } => {
            push_u8(h, 7);
            push_u8(h, *player);
            push_usize(h, *need);
            push_u8(h, *asked as u8);
        }
        M::Death { player, killer } => {
            push_u8(h, 8);
            push_u8(h, *player);
            push_u8(h, killer.map(|p| p + 1).unwrap_or(0));
        }
        M::OptionalGate { owner, def, event, asked, inner } => {
            push_u8(h, 9);
            push_u8(h, *owner);
            push_u16(h, *def);
            push_event_data(h, event);
            push_u8(h, *asked as u8);
            match inner {
                None => push_u8(h, 0),
                Some(m) => {
                    push_u8(h, 1);
                    push_machine(h, m);
                }
            }
        }
        M::SanCai { owner, stage, target } => {
            push_u8(h, 10);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
        }
        M::YiShang { owner, stage } => {
            push_u8(h, 11);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::SuiDun => push_u8(h, 12),
        M::SuQing { target } => {
            push_u8(h, 13);
            push_u8(h, *target);
        }
        M::ShouYi { owner } => {
            push_u8(h, 14);
            push_u8(h, *owner);
        }
        M::FanKui {
            owner,
            source,
            asked,
        } => {
            push_u8(h, 15);
            push_u8(h, *owner);
            push_u8(h, source.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *asked as u8);
        }
        M::JianXiong { owner, card, asked } => {
            push_u8(h, 21);
            push_u8(h, *owner);
            push_u32(h, *card);
            push_u8(h, *asked as u8);
        }
        M::BiYue { owner } => {
            push_u8(h, 23);
            push_u8(h, *owner);
        }
        M::JiZhi { owner } => {
            push_u8(h, 25);
            push_u8(h, *owner);
        }
        M::KuRou { owner, stage } => {
            push_u8(h, 26);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::ZhiHeng { owner, stage } => {
            push_u8(h, 27);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::LuoShen { owner } => {
            push_u8(h, 24);
            push_u8(h, *owner);
        }
        M::YingZi { owner } => {
            push_u8(h, 28);
            push_u8(h, *owner);
        }
        M::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => {
            push_u8(h, 44);
            push_u32(h, *card);
            push_u8(h, *user);
            push_players(h, holders);
            push_u8(h, *asked as u8);
        }
        M::Dismantle {
            card,
            source,
            target,
            asked,
        } => {
            push_u8(h, 29);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *asked as u8);
        }
        M::XiaoJi { owner } => {
            push_u8(h, 29);
            push_u8(h, *owner);
        }
        M::QingNang {
            owner,
            stage,
            healed,
        } => {
            push_u8(h, 30);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, healed.map(|p| p + 1).unwrap_or(0));
        }
        M::JiJi {
            owner,
            stage,
            target,
            given,
        } => {
            push_u8(h, 22);
            push_u8(h, *owner);
            push_u8(h, match stage {
                JiJiStageSnap::Draw => 0,
                JiJiStageSnap::Target => 1,
                JiJiStageSnap::Retarget => 2,
                JiJiStageSnap::GiveCard => 3,
                JiJiStageSnap::GiveChooseCard => 4,
            });
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *given);
        }
        M::XiaoGuo {
            owner,
            target,
            stage,
        } => {
            push_u8(h, 45);
            push_u8(h, *owner);
            push_u8(h, *target);
            push_u8(h, *stage as u8);
        }
        M::KuangFu { owner, target } => {
            push_u8(h, 46);
            push_u8(h, *owner);
            push_u8(h, *target);
        }
        M::YaoWu {
            owner,
            source,
            stage,
        } => {
            push_u8(h, 47);
            push_u8(h, *owner);
            push_u8(h, *source);
            push_u8(h, *stage as u8);
        }
        M::ShuShen {
            owner,
            target,
            stage,
        } => {
            push_u8(h, 48);
            push_u8(h, *owner);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *stage as u8);
        }
        M::ShenZhi { owner } => {
            push_u8(h, 49);
            push_u8(h, *owner);
        }
        M::XunXun { owner, stage, top } => {
            push_u8(h, 50);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u32(h, top.len() as u32);
            for c in top {
                push_u32(h, *c);
            }
        }
        M::WangXi { owner, other } => {
            push_u8(h, 51);
            push_u8(h, *owner);
            push_u8(h, *other);
        }
        M::JiJie { owner, card, stage } => {
            push_u8(h, 52);
            push_u8(h, *owner);
            match card {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            push_u8(h, *stage as u8);
        }
        M::JiYuan { owner } => {
            push_u8(h, 53);
            push_u8(h, *owner);
        }
        M::WangZun { owner, target } => {
            push_u8(h, 54);
            push_u8(h, *owner);
            push_u8(h, *target);
        }
        M::TongJi { owner, stage } => {
            push_u8(h, 55);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::Wuzhong { owner } => {
            push_u8(h, 56);
            push_u8(h, *owner);
        }
        M::Taoyuan { owner } => {
            push_u8(h, 57);
            push_u8(h, *owner);
        }
        M::Wugu { owner, cards, pickers } => {
            push_u8(h, 58);
            push_u8(h, *owner);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
            push_u32(h, pickers.len() as u32);
            for p in pickers {
                push_u8(h, *p);
            }
        }
        M::Jiedao { owner, holder, victim, stage } => {
            push_u8(h, 59);
            push_u8(h, *owner);
            push_u8(h, *holder);
            push_u8(h, *victim);
            push_u8(h, *stage as u8);
        }
        M::GangLie {
            owner,
            source,
            asked,
        } => {
            push_u8(h, 31);
            push_u8(h, *owner);
            push_u8(h, *source);
            push_u8(h, *asked as u8);
        }
        M::TuXi { owner, stage } => {
            push_u8(h, 32);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::LuoYi { owner } => {
            push_u8(h, 33);
            push_u8(h, *owner);
        }
        M::GuanXing { owner, stage, top } => {
            push_u8(h, 34);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in top {
                push_u32(h, *c);
            }
        }
        M::LianYing { owner } => {
            push_u8(h, 35);
            push_u8(h, *owner);
        }
        M::Rende { owner, stage, cards } => {
            push_u8(h, 36);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in cards {
                push_u32(h, *c);
            }
        }
        M::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => {
            push_u8(h, 37);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            // 逐人询问队列入哈希
            match queue {
                None => push_u8(h, 0),
                Some(q) => {
                    push_u8(h, 1);
                    push_players(h, q);
                }
            }
            push_u8(h, helper.map(|p| p + 1).unwrap_or(0));
            push_u8(h, sha.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            push_u8(h, inner.is_some() as u8);
        }
        M::Host { def_key, behavior, stage, owner, target, card, cards, targets } => {
            push_u8(h, 42);
            // 定义键入哈希（有界）：逐字节 FNV-1a
            let mut hv: u64 = 0xcbf2_9ce4_8422_2325;
            for b in def_key.as_bytes() {
                hv ^= *b as u64;
                hv = hv.wrapping_mul(0x1000_0000_01b3);
            }
            push_u64(h, hv);
            push_u8(h, *behavior);
            push_u8(h, *stage);
            push_u8(h, *owner);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, card.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            for c in cards {
                push_u32(h, *c);
            }
            for p in targets {
                push_u8(h, *p);
            }
        }
        M::Snatch {
            card,
            source,
            target,
            asked,
        } => {
            push_u8(h, 38);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *asked as u8);
        }
        M::JieYin { owner, stage, cards } => {
            push_u8(h, 39);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in cards {
                push_u32(h, *c);
            }
        }
        M::FanJian { owner, stage, target, chosen_suit } => {
            push_u8(h, 40);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, chosen_suit.map(|t| t + 1).unwrap_or(0));
        }
        M::LiJian { owner, stage, card, inner } => {
            push_u8(h, 41);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, card.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            push_u8(h, inner.is_some() as u8);
        }
    }
}

fn push_event_data(h: &mut Hasher, data: &EventData) {
    match data {
        EventData::Damage(d) => {
            push_u8(h, 1);
            push_u8(h, d.source.map(|s| s + 1).unwrap_or(0));
            push_u8(h, d.target);
            push_u32(h, d.amount);
            push_players(h, &d.redirect_chain);
            push_u8(h, d.prevented as u8);
            push_u8(h, d.nature as u8);
        }
        EventData::Recover {
            player,
            amount,
            by_card,
        } => {
            push_u8(h, 2);
            push_u8(h, *player);
            push_u32(h, *amount);
            push_u32(h, by_card.unwrap_or(u32::MAX));
        }
        EventData::CardsDrawn { player, cards } => {
            push_u8(h, 3);
            push_u8(h, *player);
            push_cards(h, cards);
        }
        EventData::CardsMoved { cards, from, to } => {
            push_u8(h, 4);
            push_cards(h, cards);
            push_u8(h, *from);
            push_u8(h, *to);
        }
        EventData::CardUsed(u) => {
            push_u8(h, 5);
            push_u32(h, u.card);
            push_u8(h, u.player);
            push_players(h, &u.targets);
            push_u8(h, u.nature as u8);
        }
        EventData::CardResponded { card, player } => {
            push_u8(h, 6);
            push_u32(h, *card);
            push_u8(h, *player);
        }
        EventData::Dying { player, source } => {
            push_u8(h, 7);
            push_u8(h, *player);
            push_u8(h, source.map(|s| s + 1).unwrap_or(0));
        }
        EventData::Death { player, killer } => {
            push_u8(h, 8);
            push_u8(h, *player);
            push_u8(h, killer.map(|k| k + 1).unwrap_or(0));
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            push_u8(h, 9);
            push_u8(h, *player);
            push_u8(h, *phase as u8);
            push_u8(h, *begin as u8);
        }
        EventData::TurnEnded { player } => {
            push_u8(h, 10);
            push_u8(h, *player);
        }
        EventData::TurnOverBefore { player, cancelled } => {
            push_u8(h, 20);
            push_u8(h, *player);
            push_u8(h, *cancelled as u8);
        }
        EventData::TurnOvered { player, face_down } => {
            push_u8(h, 21);
            push_u8(h, *player);
            push_u8(h, *face_down as u8);
        }
        EventData::LoseHp { player, amount } => {
            push_u8(h, 22);
            push_u8(h, *player);
            push_u32(h, *amount);
        }
        EventData::DyingSaved { player, source } => {
            push_u8(h, 23);
            push_u8(h, *player);
            push_u8(h, source.map(|k| k + 1).unwrap_or(0));
        }
        EventData::SkillToggled {
            player,
            skill,
            enabled,
            temporary,
        } => {
            push_u8(h, 11);
            push_u8(h, *player);
            push_u16(h, *skill);
            push_u8(h, *enabled as u8);
            push_u8(h, *temporary as u8);
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            push_u8(h, 12);
            push_u8(h, *player);
            push_u32(h, *card);
            push_u32(h, replaced.unwrap_or(u32::MAX));
        }
        EventData::Linked { player, linked } => {
            push_u8(h, 13);
            push_u8(h, *player);
            push_u8(h, *linked as u8);
        }
        EventData::CardsDiscarded { player, cards } => {
            push_u8(h, 14);
            push_u8(h, *player);
            push_cards(h, cards);
        }
    }
}

fn push_u8(h: &mut Hasher, v: u8) {
    h.update(&[v]);
}
fn push_u16(h: &mut Hasher, v: u16) {
    h.update(&v.to_le_bytes());
}
fn push_u32(h: &mut Hasher, v: u32) {
    h.update(&v.to_le_bytes());
}
fn push_u64(h: &mut Hasher, v: u64) {
    h.update(&v.to_le_bytes());
}
fn push_i32(h: &mut Hasher, v: i32) {
    h.update(&v.to_le_bytes());
}
fn push_usize(h: &mut Hasher, v: usize) {
    h.update(&(v as u64).to_le_bytes());
}
fn push_cards(h: &mut Hasher, cards: &[CardId]) {
    push_u32(h, cards.len() as u32);
    for c in cards {
        push_u32(h, *c);
    }
}
fn push_players(h: &mut Hasher, players: &[PlayerId]) {
    push_u32(h, players.len() as u32);
    for p in players {
        push_u8(h, *p);
    }
}
fn push_options(h: &mut Hasher, opts: &[ChoiceOption]) {
    push_u32(h, opts.len() as u32);
    for o in opts {
        match o {
            ChoiceOption::Confirm => push_u8(h, 1),
            ChoiceOption::Pass => push_u8(h, 2),
            ChoiceOption::Card(c) => {
                push_u8(h, 3);
                push_u32(h, *c);
            }
            ChoiceOption::Player(p) => {
                push_u8(h, 4);
                push_u8(h, *p);
            }
            ChoiceOption::Tag(t) => {
                push_u8(h, 5);
                push_u8(h, *t);
            }
        }
    }
}
fn push_static(h: &mut Hasher, s: &'static str) {
    push_u32(h, s.len() as u32);
    h.update(s.as_bytes());
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
                    rende_given: 0,
                    jiu_used: false,
                mujiaren_used: false,
                    jiu_active: false,
                    turn_opened: true,
                    skip_draw_phase: false,
                    skip_play_phase: false,
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
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                cards_used_play_phase: 0,
                phase_number: 0,
                },
                skills: Default::default(),
                declared_kinds: Default::default(),
                card_natures: Default::default(),
                current_land: None,
                outcome: None,
            };
            core.deck = (0..catalog.cards.len() as CardId).collect();
            let resolution = Resolution {
                parks: vec![crate::resolution::Park {
                    resume_depth: 0,
                    slot: crate::resolution::Slot::Intrinsic { frame: 1 },
                    machine: Box::new(ShaResolveMachine::new(
                        0, 0, 1, DamageNature::Normal, asked, armor_judged, 0, None,
                        false, 0, None, None, None, false, false, None, false, false,
                        false, Vec::new(), false, false, false, Vec::new(), 0, false, false, false,
                        false, None, false,
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
        let mk = |version: u64| {
            GameState {
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
                    rende_given: 0,
                    jiu_used: false,
                mujiaren_used: false,
                    jiu_active: false,
                        turn_opened: true,
                        skip_draw_phase: false,
                        skip_play_phase: false,
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
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                cards_used_play_phase: 0,
                phase_number: 0,
                    },
                    skills: Default::default(),
                    declared_kinds: Default::default(),
                card_natures: Default::default(),
                current_land: None,
                    outcome: None,
                },
                resolution: Resolution::new(64),
            }
        };
        assert_ne!(canonical_hash(&mk(0)), canonical_hash(&mk(1)));
        let _ = catalog;
    }
}
