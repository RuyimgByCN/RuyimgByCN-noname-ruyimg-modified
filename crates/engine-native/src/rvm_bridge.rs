//! T8 增量 1（ADR 0782 定案后的引擎接线）：engine-native 触发调度接
//! rule-vm——扩展包数据声明的 VM 技能（`VmSkillDecl`）经目录 SkillDef
//! 进入行为总线，触发时以内核状态为只读查询面执行字节码，效果按原型
//! 同款**两层提交协议**落账：
//!
//! - VM 排队效果仅在 `Halted` 时提交（recover/draw/damage 映射）；
//! - `Paused`（选择挂起）本增量未接——排队效果不落账（增量 2）；
//! - `Err`（fuel 耗尽/trap）排队效果整批丢弃——半次效果不落账（回滚）。
//!   两种情况在事件面表现为「无效果落账」（可断言）。
//!
//! 查询面增量 1 只映射 `Hp`/`MaxHp`/`Distance`；未映射查询/效果在
//! emit/query 时即返回 `InvalidRequest`（模块 trap → 回滚协议）。
//!
//! 玩家 id 映射：rule-api `PlayerId` 为 1 基（`NONE=0` 哨兵），内核座位
//! 为 0 基——桥内统一 `内核座位 s ↔ rule_id = s+1`。

use std::sync::Arc;

use rule_api::{CapabilitySet, EffectRequest, HostTrap, QueryRequest, QueryResponse};
use rule_vm::{BytecodeModule, Continuation, ExecResult, HostContext, RuleVm, VmHost, DEFAULT_FUEL};

use crate::catalog::{SkillBehavior, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::catalog::QueryCtx;
use crate::machine::{Ctx, Flow};
use crate::snap::MachineSnapshot;

/// 数据驱动的 VM 技能程序（扩展包声明携带；目录 def 的 behavior 持有）。
#[derive(Debug, Clone)]
pub struct VmSkillProgram {
    /// 已验证字节码模块（装载/编译产物；`logic_hash` 为内容锁锚）。
    /// PA06：类型即「装载期已校验」——只能经 [`VmSkillProgram::new`]
    /// 构造，畸形模块装载期拒绝；执行期不再重复静态校验。
    pub module: rule_vm::VerifiedModule,
    /// 目录 def 名（T8 增量 8：HostContext.skill_id 反查键）。
    pub skill_name: String,
}

impl VmSkillProgram {
    /// PA06：装载期校验一次——畸形模块构造即拒。
    pub fn new(
        module: BytecodeModule,
        skill_name: String,
    ) -> Result<Self, rule_vm::VmError> {
        Ok(Self {
            module: rule_vm::VerifiedModule::verify(module)?,
            skill_name,
        })
    }

    /// 内容锁锚（模块逻辑哈希）。
    pub fn logic_hash(&self) -> [u8; 32] {
        self.module.logic_hash
    }
}

/// VM 技能触发时捕获的宿主上下文（从触发事件提取）。
/// targets 为堆面（增量 17），弃 Copy 保 Clone。
#[derive(Debug, Clone, Default)]
struct VmTriggerCtx {
    event_player: Option<PlayerId>,
    event_source: Option<PlayerId>,
    event_card: Option<CardId>,
    event_amount: u32,
    /// 阶段事件携带（阶段种，begin 由 EventKind 区分；增量 14）。
    phase: Option<crate::event::Phase>,
    /// 伤害载荷携带（增量 16；非伤害帧为 None——串帧防护面）。
    nature: Option<crate::event::DamageNature>,
    /// 用牌目标集（增量 17；CardUsed 载荷——card_targeted 语义以
    /// CardUsed+targets 表达，原生无独立 targeted 帧）。
    targets: Vec<u8>,
    /// view-as 声明牌种。
    use_as: Option<crate::catalog::CardKind>,
    /// 铁索传导。
    from_chain: bool,
    /// 本批张数（弃牌/移牌）。
    batch_len: u32,
    /// 移牌来源座位。
    moved_from: Option<PlayerId>,
    /// 阶段开始/结束。
    begin: Option<bool>,
    /// 1 伤害 2 用牌 3 阶段 4 弃牌 5 移牌 6 濒死 7 死亡 8 打出。
    kind: u8,
}

/// 普通锦囊判定（增量 16）：锦囊三类（非基本/非装备/非异类）且非延时
///（乐/兵/闪电/洪水——G1 运筹洪水同为延时）。异类（地/机关）不计。
fn kind_is_ordinary_trick(kind: crate::catalog::CardKind) -> bool {
    use crate::catalog::CardKind as K;
    if kind.is_basic_kind() || kind.equip_slot().is_some() {
        return false;
    }
    !matches!(kind, K::Le | K::Bing | K::ShanDian | K::HongShui)
        && kind.is_basic_trick_or_equip()
}

fn trigger_ctx_of(event: &EventData) -> VmTriggerCtx {
    match event {
        EventData::Damage(d) => VmTriggerCtx {
            event_player: Some(d.target),
            event_source: d.source,
            event_card: d.card,
            event_amount: d.amount,
            nature: Some(d.nature),
            from_chain: d.from_chain,
            kind: 1,
            ..VmTriggerCtx::default()
        },
        EventData::Phase {
            player,
            phase,
            begin,
        } => VmTriggerCtx {
            event_player: Some(*player),
            phase: Some(*phase),
            begin: Some(*begin),
            kind: 3,
            ..VmTriggerCtx::default()
        },
        // 杀被闪避（PA12 批②）：来源=杀的使用者、事件主玩家=闪避的目标。
        EventData::ShaDodged { card, source, target } => VmTriggerCtx {
            event_player: Some(*target),
            event_source: Some(*source),
            event_card: Some(*card),
            kind: 8,
            ..VmTriggerCtx::default()
        },
        EventData::CardUsed(u) => VmTriggerCtx {
            event_player: Some(u.player),
            // 旧线 CardUsed 的 TriggerContext.source = 用牌者；原生此前未填
            // 导致 `owner_is_source` 对用牌事件恒假（PA12 批①）。
            event_source: Some(u.player),
            event_card: Some(u.card),
            targets: u.targets.clone(),
            use_as: u.use_as,
            nature: Some(u.nature),
            kind: 2,
            ..VmTriggerCtx::default()
        },
        EventData::Death { player, killer } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *killer,
            kind: 7,
            ..VmTriggerCtx::default()
        },
        EventData::CardsMoved { cards, from, to } => VmTriggerCtx {
            event_player: Some(*to),
            event_card: cards.first().copied(),
            moved_from: Some(*from),
            batch_len: cards.len() as u32,
            kind: 5,
            ..VmTriggerCtx::default()
        },
        EventData::CardsDiscarded { player, cards } => VmTriggerCtx {
            event_player: Some(*player),
            event_card: cards.first().copied(),
            batch_len: cards.len() as u32,
            kind: 4,
            ..VmTriggerCtx::default()
        },
        EventData::Dying { player, source } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *source,
            kind: 6,
            ..VmTriggerCtx::default()
        },
        EventData::CardResponded { card, player } => VmTriggerCtx {
            event_player: Some(*player),
            event_card: Some(*card),
            kind: 8,
            ..VmTriggerCtx::default()
        },
        _ => VmTriggerCtx::default(),
    }
}

fn filter_view(t: &VmTriggerCtx) -> crate::rvm_named::FilterView {
    crate::rvm_named::FilterView {
        event_player: t.event_player,
        event_source: t.event_source,
        event_card: t.event_card,
        event_amount: t.event_amount,
        phase: t.phase,
        nature: t.nature,
        targets: t.targets.clone(),
        use_as: t.use_as,
        from_chain: t.from_chain,
        batch_len: t.batch_len,
        moved_from: t.moved_from,
        begin: t.begin,
        kind: t.kind,
    }
}

/// DSL filter 映射（增量 6 三基 + 增量 9 基元扩展；增量 13 语义收敛——
/// `owner_is_target_*` 家族全部含「事件主玩家=持有者」合取段）：具名
/// 组合 filter 由装载器跳过登记（deferred）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VmFilterPolicy {
    /// DSL `owner_is_target`：事件主玩家=持有者。
    #[default]
    OwnerIsTarget,
    /// DSL `owner_alive`：持有者存活即过闸（**不含**事件主玩家合取——
    /// 他人结束阶段等场合也触发；PA12 批①，样本=清励）。旧线语义同。
    OwnerAlive,
    /// DSL `owner_source`：事件来源=持有者。
    OwnerIsSource,
    /// DSL `always`：无条件。
    Always,
    /// DSL `owner_is_target_wounded`：事件主玩家=持有者，且持有者已受伤
    /// （0<hp<maxHp）。
    OwnerWounded,
    /// DSL `owner_is_target_linked`：事件主玩家=持有者，且持有者横置。
    OwnerLinked,
    /// DSL `owner_is_target_hand_nonempty`：事件主玩家=持有者，且持有者
    /// 手牌非空。
    OwnerHandNonempty,
    /// DSL `owner_is_target_hand_empty`：事件主玩家=持有者，且持有者手牌
    /// 为空。
    OwnerHandEmpty,
    /// DSL `owner_is_target_no_turn_damage`：事件主玩家=持有者，且持有者
    /// 本回合未造成伤害。
    OwnerNoTurnDamage,
    /// DSL `owner_is_target_turn_damage`：事件主玩家=持有者，且持有者本
    /// 回合已造成伤害。
    OwnerTurnDamage,
    /// DSL `owner_is_target_hand_below_max_hp`：事件主玩家=持有者，且手
    /// 牌数 < 体力上限。
    OwnerHandBelowMaxHp,
    /// DSL `owner_is_target_hand_above_hp`：事件主玩家=持有者，且手牌数
    /// > 体力。
    OwnerHandAboveHp,
    /// DSL `owner_is_target_hand_at_least_hp`：事件主玩家=持有者，且手牌
    /// 数 >= 体力。
    OwnerHandAtLeastHp,
    /// DSL `owner_is_target_other_hand_nonempty`：事件主玩家=持有者，且
    /// 任一其他存活角色手牌非空。
    OwnerOtherHandNonempty,
    /// DSL `owner_is_target_turn_damage_at_least_three`：事件主玩家=持有
    /// 者，且本回合造成伤害 ≥3。
    OwnerTurnDamageAtLeastThree,
    /// DSL `owner_is_target_hand_at_least_hp_other_alive`：事件主玩家=持
    /// 有者，手牌数 ≥ 体力，且存在其他存活角色。
    OwnerHandAtLeastHpOtherAlive,
    /// DSL `owner_is_target_play_phase_card_category_count_positive`：事
    /// 件主玩家=持有者，事件为出牌阶段结束，且本出牌阶段已使用 ≥1 张基
    /// 本/锦囊/装备牌（增量 14，ABI 42）。
    OwnerPlayPhaseBteUsed,
    /// DSL `owner_alive_play_phase_card_category_count_positive`：事件为
    /// 出牌阶段结束，且持有者在该出牌阶段已使用 ≥1 张基本/锦囊/装备牌
    ///（仅当前回合玩家可用牌，故事件主玩家=持有者；增量 14，ABI 293）。
    OwnerAlivePlayPhaseBteUsed,
    /// DSL `owner_is_target_hp_below_{N}`：事件主玩家=持有者，且当前体力
    /// < N（增量 15，参数化）。
    OwnerHpBelow(u32),
    /// DSL `owner_source_play_card_count_eq_{N}`：事件主玩家=持有者（使
    /// 用者），且本次为其本出牌阶段第 N 张主动用牌（响应不计、出牌阶段
    /// 开始清零；增量 15，参数化）。
    OwnerPlayCountEq(u32),
    /// DSL `owner_source_play_card_count_eq_hp`：同上，N=当前体力（增量
    /// 15）。
    OwnerPlayCountEqHp,
    /// DSL `owner_is_target_card_used_at_least_hp`：事件主玩家=持有者，
    /// 且本回合（口径化归：出牌阶段计数，Play 开始清零=换回合归零）已
    /// 主动用牌 ≥ 当前体力（增量 15；响应不计）。
    OwnerCardUsedAtLeastHp,
    /// DSL `owner_target_hand_empty_elemental_or_trick_damage_before`：伤
    /// 害前置窗，事件主玩家=持有者、其手牌为空，且即将承受的伤害为元素
    /// 性（火/雷）或事件卡为普通锦囊（非延时；增量 16，ABI 358）。
    OwnerHandEmptyElementalOrTrickDamageBefore,
    /// DSL `owner_is_target_source_movable_field_card`：伤害事件主玩家=
    /// 持有者、来源存活，且任意其他存活角色的场牌（装备/判定区延时）至
    /// 少一张可移至来源（装备按栏空、判定区按同名唯一；增量 16，ABI
    /// 202）。移动可行性查询为近似：不校验移动技自身规则面。
    OwnerSourceMovableFieldCard,
    /// DSL `owner_is_target_tied_max_hand`：事件主玩家=持有者，且手牌数
    /// 与存活玩家最大值并列（增量 17，ABI 55）。
    OwnerTiedMaxHand,
    /// DSL `owner_is_target_tied_max_hp`：事件主玩家=持有者，且当前体力
    /// 与存活玩家最大值并列（增量 17，ABI 55）。
    OwnerTiedMaxHp,
    /// DSL `owner_is_target_tied_max_equip_nonempty`：事件主玩家=持有者，
    /// 有装备且装备数与存活玩家最大值并列（增量 17，ABI 55）。
    OwnerTiedMaxEquipNonempty,
    /// DSL `owner_is_target_not_unique_max_current_hp`：事件主玩家=持有
    /// 者，且体力并非严格高于所有其他存活角色（存在并列或更高者；增量
    /// 17）。
    OwnerNotUniqueMaxHp,
    /// DSL `owner_targeted_by_other_sha`：持有者被**另一玩家**的杀指定
    /// （增量 17；原生无独立 targeted 帧型，以 CardUsed+targets 表达，
    /// 登记差异）。
    OwnerTargetedByOtherSha,
    /// 具名组合 filter（T8 余项）。精确策略优先于本变体。
    Named(Arc<crate::rvm_named::NamedProgram>),
    /// DSL `owner_source_sha`：来源=持有者且事件牌为杀族（PA12 批③）。
    OwnerSourceSha,
    /// DSL `owner_source_fire_sha`：CardUsed 且来源=持有者、杀族、火属性。
    OwnerSourceFireSha,
    /// DSL `owner_source_ordinary_trick`：来源=持有者且事件牌为普通锦囊。
    OwnerSourceOrdinaryTrick,
    /// DSL `owner_alive_not_target`：持有者存活且事件主玩家≠持有者。
    OwnerAliveNotEventPlayer,
    /// DSL `owner_is_target_source_alive`：事件主玩家=持有者且事件来源存活。
    OwnerTargetSourceAlive,
    /// DSL `owner_is_source_or_target`：事件来源=持有者或事件主玩家=持有者。
    OwnerSourceOrTarget,
    /// 阶段门（PA12④）：事件携带的阶段须等于 `phase`，再评估 `inner`。
    /// 装载器用它把 DSL 的 `phase_end`/`phase_draw`/… 落到原生
    /// `PhaseBegin`/`PhaseEnd`（原生阶段帧按阶段逐一发出，旧线
    /// `phase_end` 只指「进入结束阶段」——无门会每阶段触发一次）。
    AtPhase {
        phase: crate::event::Phase,
        inner: Box<VmFilterPolicy>,
    },
}

/// 把 DSL filter 名映射到策略。未知名称返回 None（装载器跳过，不恒真）。
pub fn map_vm_filter(name: &str) -> Option<VmFilterPolicy> {
    let exact = match name {
        "owner_is_target" => Some(VmFilterPolicy::OwnerIsTarget),
        // PA12 批①：旧线 DSL 实际拼写为 owner_is_source（inventory 206 处），
        // 与既有 owner_source 同义（事件来源=持有者）。
        "owner_source" | "owner_is_source" => Some(VmFilterPolicy::OwnerIsSource),
        "owner_alive" => Some(VmFilterPolicy::OwnerAlive),
        "owner_source_sha" => Some(VmFilterPolicy::OwnerSourceSha),
        "owner_source_fire_sha" => Some(VmFilterPolicy::OwnerSourceFireSha),
        "owner_source_ordinary_trick" => Some(VmFilterPolicy::OwnerSourceOrdinaryTrick),
        "owner_alive_not_target" => Some(VmFilterPolicy::OwnerAliveNotEventPlayer),
        "owner_is_target_source_alive" => Some(VmFilterPolicy::OwnerTargetSourceAlive),
        "owner_is_source_or_target" => Some(VmFilterPolicy::OwnerSourceOrTarget),
        "always" => Some(VmFilterPolicy::Always),
        "owner_is_target_wounded" => Some(VmFilterPolicy::OwnerWounded),
        "owner_is_target_linked" => Some(VmFilterPolicy::OwnerLinked),
        "owner_is_target_hand_nonempty" => Some(VmFilterPolicy::OwnerHandNonempty),
        "owner_is_target_hand_empty" => Some(VmFilterPolicy::OwnerHandEmpty),
        "owner_is_target_no_turn_damage" => Some(VmFilterPolicy::OwnerNoTurnDamage),
        "owner_is_target_turn_damage" => Some(VmFilterPolicy::OwnerTurnDamage),
        "owner_is_target_turn_damage_at_least_three" => {
            Some(VmFilterPolicy::OwnerTurnDamageAtLeastThree)
        }
        "owner_is_target_hand_at_least_hp_other_alive" => {
            Some(VmFilterPolicy::OwnerHandAtLeastHpOtherAlive)
        }
        "owner_is_target_hand_below_max_hp" => Some(VmFilterPolicy::OwnerHandBelowMaxHp),
        "owner_is_target_hand_above_hp" => Some(VmFilterPolicy::OwnerHandAboveHp),
        "owner_is_target_other_hand_nonempty" => Some(VmFilterPolicy::OwnerOtherHandNonempty),
        "owner_is_target_play_phase_card_category_count_positive" => {
            Some(VmFilterPolicy::OwnerPlayPhaseBteUsed)
        }
        "owner_alive_play_phase_card_category_count_positive" => {
            Some(VmFilterPolicy::OwnerAlivePlayPhaseBteUsed)
        }
        "owner_source_play_card_count_eq_hp" => Some(VmFilterPolicy::OwnerPlayCountEqHp),
        "owner_is_target_card_used_at_least_hp" => Some(VmFilterPolicy::OwnerCardUsedAtLeastHp),
        "owner_target_hand_empty_elemental_or_trick_damage_before" => {
            Some(VmFilterPolicy::OwnerHandEmptyElementalOrTrickDamageBefore)
        }
        "owner_is_target_source_movable_field_card" => {
            Some(VmFilterPolicy::OwnerSourceMovableFieldCard)
        }
        "owner_is_target_tied_max_hand" => Some(VmFilterPolicy::OwnerTiedMaxHand),
        "owner_is_target_tied_max_hp" => Some(VmFilterPolicy::OwnerTiedMaxHp),
        "owner_is_target_tied_max_equip_nonempty" => Some(VmFilterPolicy::OwnerTiedMaxEquipNonempty),
        "owner_is_target_not_unique_max_current_hp" => Some(VmFilterPolicy::OwnerNotUniqueMaxHp),
        "owner_targeted_by_other_sha" => Some(VmFilterPolicy::OwnerTargetedByOtherSha),
        _ => None,
    };
    if exact.is_some() {
        return exact;
    }
    if let Some(n) = name
        .strip_prefix("owner_is_target_hp_below_")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
    {
        return Some(VmFilterPolicy::OwnerHpBelow(n));
    }
    if let Some(n) = name
        .strip_prefix("owner_source_play_card_count_eq_")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
    {
        return Some(VmFilterPolicy::OwnerPlayCountEq(n));
    }
    crate::rvm_named::compile_named(name).map(|p| VmFilterPolicy::Named(Arc::new(p)))
}

/// VM 技能行为：触发即执行模块（事件门由 def 的 TriggerMeta、行为总线
/// 与 filter 策略共同承载）。
#[derive(Debug, Clone)]
pub struct VmSkillBehavior {
    pub program: Arc<VmSkillProgram>,
    pub filter: VmFilterPolicy,
}

impl SkillBehavior for VmSkillBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        // 增量 13 语义收敛：`owner_is_target_*` 家族的权威语义（编译器
        // lsp 权威描述 "the skill owner is the event player and …"）是
        // 「事件主玩家=持有者」与状态谓词的合取——filter 是触发调度里
        // 唯一的持有者域闸（plan_triggers 只按 EventKind 找候选），漏掉
        // 合取段会让他人事件过闸（炙焚 holder 门同类缺陷）。turn_damage
        // 为本回合「造成」伤害计数（kernel 记在来源侧），与 lsp 一致。
        let trig = trigger_ctx_of(event);
        if let VmFilterPolicy::AtPhase { phase, inner } = &self.filter {
            if trig.phase != Some(*phase) {
                return false;
            }
            let gated = VmSkillBehavior {
                program: self.program.clone(),
                filter: (**inner).clone(),
            };
            return gated.filter(ctx, owner, event);
        }
        let is_event_player = trig.event_player == Some(owner);
        let holder_state = |s: u8| ctx.state.player(s);
        match self.filter.clone() {
            VmFilterPolicy::OwnerIsTarget => is_event_player,
            VmFilterPolicy::OwnerAlive => ctx.state.player(owner).alive,
            VmFilterPolicy::OwnerSourceSha => {
                trig.event_source == Some(owner)
                    && trig
                        .event_card
                        .is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
            }
            VmFilterPolicy::OwnerSourceFireSha => {
                trig.kind == 2
                    && trig.event_source == Some(owner)
                    && trig
                        .event_card
                        .is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
                    && trig.nature == Some(crate::event::DamageNature::Fire)
            }
            VmFilterPolicy::OwnerSourceOrdinaryTrick => {
                trig.event_source == Some(owner)
                    && trig
                        .event_card
                        .is_some_and(|c| kind_is_ordinary_trick(ctx.catalog.card_kind(c)))
            }
            VmFilterPolicy::OwnerAliveNotEventPlayer => {
                ctx.state.player(owner).alive && trig.event_player != Some(owner)
            }
            VmFilterPolicy::OwnerTargetSourceAlive => {
                is_event_player
                    && trig
                        .event_source
                        .is_some_and(|src| ctx.state.player(src).alive)
            }
            VmFilterPolicy::OwnerSourceOrTarget => {
                trig.event_source == Some(owner) || is_event_player
            }
            VmFilterPolicy::OwnerIsSource => trig.event_source == Some(owner),
            VmFilterPolicy::Always => true,
            VmFilterPolicy::OwnerWounded => {
                is_event_player
                    && {
                        let p = holder_state(owner);
                        p.hp > 0 && p.hp < p.max_hp as i32
                    }
            }
            VmFilterPolicy::OwnerLinked => {
                is_event_player && holder_state(owner).linked
            }
            VmFilterPolicy::OwnerHandNonempty => {
                is_event_player && !holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerHandEmpty => {
                is_event_player && holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerNoTurnDamage => {
                is_event_player && holder_state(owner).turn_damage == 0
            }
            VmFilterPolicy::OwnerTurnDamage => {
                is_event_player && holder_state(owner).turn_damage > 0
            }
            VmFilterPolicy::OwnerHandBelowMaxHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() < p.max_hp as usize
                }
            }
            VmFilterPolicy::OwnerHandAboveHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() > p.hp as usize
                }
            }
            VmFilterPolicy::OwnerHandAtLeastHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() >= p.hp as usize
                }
            }
            VmFilterPolicy::OwnerOtherHandNonempty => {
                is_event_player
                    && (0..ctx.state.players.len()).any(|i| {
                        i as u8 != owner
                            && ctx.state.players[i].alive
                            && !ctx.state.players[i].hand.is_empty()
                    })
            }
            VmFilterPolicy::OwnerTurnDamageAtLeastThree => {
                is_event_player && holder_state(owner).turn_damage >= 3
            }
            VmFilterPolicy::OwnerHandAtLeastHpOtherAlive => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() >= p.hp as usize
                } && (0..ctx.state.players.len()).any(|i| {
                    i as u8 != owner && ctx.state.players[i].alive
                })
            }
            // 出牌阶段结束 && 本阶段三类牌已用 ≥1（计数随 Play 开始清
            // 零，事件时点值即本阶段值；ABI 293 的持有者归因由「仅当前
            // 回合玩家可用牌」化归为事件主玩家合取）。
            VmFilterPolicy::OwnerPlayPhaseBteUsed
            | VmFilterPolicy::OwnerAlivePlayPhaseBteUsed => {
                is_event_player
                    && trig.phase == Some(crate::event::Phase::Play)
                    && ctx.state.turn.bte_used_play_phase > 0
            }
            VmFilterPolicy::OwnerHpBelow(n) => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hp > 0 && p.hp < n as i32
                }
            }
            // 用牌序数：commit_card_used 尾计数=本次用牌的序数（响应走
            // CardResponded 不经此点=「Responses do not count」同口径）。
            VmFilterPolicy::OwnerPlayCountEq(n) => {
                is_event_player && ctx.state.turn.cards_used_play_phase == n
            }
            VmFilterPolicy::OwnerPlayCountEqHp => {
                let p = holder_state(owner);
                is_event_player
                    && ctx.state.turn.cards_used_play_phase == p.hp as u32
            }
            VmFilterPolicy::OwnerCardUsedAtLeastHp => {
                let p = holder_state(owner);
                is_event_player
                    && ctx.state.turn.cards_used_play_phase >= p.hp as u32
            }
            // 伤害前置窗：手牌空 ∧（元素性 ∨ 事件卡=普通锦囊）。非伤害
            // 载荷帧（before 窗对全帧型开放）nature/card 双 None → 恒否。
            VmFilterPolicy::OwnerHandEmptyElementalOrTrickDamageBefore => {
                let elemental = matches!(
                    trig.nature,
                    Some(crate::event::DamageNature::Fire)
                        | Some(crate::event::DamageNature::Thunder)
                );
                let ordinary_trick = trig
                    .event_card
                    .map(|c| kind_is_ordinary_trick(ctx.catalog.card_kind(c)))
                    .unwrap_or(false);
                is_event_player
                    && elemental | ordinary_trick
                    && holder_state(owner).hand.is_empty()
            }
            // 并列最大查询族（ABI 55）：存活玩家集合上的最大值并列。
            VmFilterPolicy::OwnerTiedMaxHand => {
                is_event_player && {
                    let me = holder_state(owner).hand.len();
                    me > 0
                        && (0..ctx.state.players.len()).all(|i| {
                            let p = &ctx.state.players[i];
                            !p.alive || p.hand.len() <= me
                        })
                }
            }
            VmFilterPolicy::OwnerTiedMaxHp => {
                is_event_player && {
                    let me = holder_state(owner).hp;
                    me > 0
                        && (0..ctx.state.players.len()).all(|i| {
                            let p = &ctx.state.players[i];
                            !p.alive || p.hp <= me
                        })
                }
            }
            VmFilterPolicy::OwnerTiedMaxEquipNonempty => {
                is_event_player && {
                    let me = holder_state(owner).equips.len();
                    me > 0
                        && (0..ctx.state.players.len()).all(|i| {
                            let p = &ctx.state.players[i];
                            !p.alive || p.equips.len() <= me
                        })
                }
            }
            VmFilterPolicy::OwnerNotUniqueMaxHp => {
                is_event_player && {
                    let me = holder_state(owner).hp;
                    (0..ctx.state.players.len()).any(|i| {
                        let p = &ctx.state.players[i];
                        p.alive && i as u8 != owner && p.hp >= me
                    })
                }
            }
            // 被另一玩家的杀指定（原生以 CardUsed+targets 表达）：
            // 持有者=目标集成员（**非**事件主玩家——lsp "the living owner
            // is targeted"）∧ 使用者非持有者 ∧ 事件卡为杀族。
            VmFilterPolicy::OwnerTargetedByOtherSha => {
                trig.event_player.is_some_and(|u| u != owner)
                    && trig
                        .event_card
                        .map(|c| ctx.catalog.card_kind(c).is_sha_like())
                        .unwrap_or(false)
                    && trig.targets.contains(&owner)
            }
            // 伤害事件：来源存活 ∧ 任意其他存活角色场上有可移至来源的
            // 装备（来源同栏空）或判定区延时（来源判定区无同名种）。
            VmFilterPolicy::OwnerSourceMovableFieldCard => {
                let Some(src) = trig.event_source else { return false };
                if !ctx.state.players[src as usize].alive || !is_event_player {
                    return false;
                }
                let src_slot: Vec<_> = ctx.state.players[src as usize]
                    .equips
                    .iter()
                    .map(|c| ctx.catalog.card_slot_of(ctx.state, *c))
                    .collect();
                let src_judge: Vec<_> = ctx.state.players[src as usize]
                    .judge
                    .iter()
                    .map(|c| ctx.catalog.card_kind(*c))
                    .collect();
                (0..ctx.state.players.len()).any(|i| {
                    let p = &ctx.state.players[i];
                    i as u8 != src
                        && p.alive
                        && p.equips.iter().any(|c| {
                            let slot = ctx.catalog.card_slot_of(ctx.state, *c);
                            slot.is_some() && !src_slot.contains(&slot)
                        })
                        || p.judge.iter().any(|c| {
                            let kind = ctx.catalog.card_kind(*c);
                            !src_judge.contains(&kind)
                        })
                })
            }
            // 阶段门已在函数头处理（递归评估 inner）；此处不可达，保守返回 false。
            VmFilterPolicy::AtPhase { .. } => false,
            VmFilterPolicy::Named(prog) => crate::rvm_named::eval_program(
                &prog,
                ctx,
                owner,
                &self.program.skill_name,
                &filter_view(&trig),
            ),
        }
    }

    fn needs_vm_card_history(&self) -> bool {
        true
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        // 阶段门（PA12 批②）包裹的内层 Named 元数据须透传：批③b 实证
        // `phase_begin owner_alive_once_per_round`（AtPhase(Named)）的 once
        // consume 在此被 `_ =>` 吞掉——限次滤名每回合重复开闸。
        fn named_meta(f: &VmFilterPolicy) -> (Vec<(String, u8)>, bool) {
            match f {
                VmFilterPolicy::Named(p) => (p.consume.clone(), p.count_hits),
                VmFilterPolicy::AtPhase { inner, .. } => named_meta(inner),
                _ => (Vec::new(), false),
            }
        }
        let (consume, count_hits) = named_meta(&self.filter);
        Box::new(VmMachine {
            owner,
            program: self.program.clone(),
            trig: trigger_ctx_of(event),
            pending_effects: Vec::new(),
            paused: None,
            asked: false,
            settled: false,
            consume,
            count_hits,
        })
    }
}

/// VM 挂起态（HOST_CHOICE）：continuation + 选项 tokens。
#[derive(Debug, Clone)]
pub struct VmPaused {
    pub continuation: Continuation,
    pub options: Vec<u32>,
}

/// VM 技能机器：可挂起（HOST_CHOICE）——挂起期间排队效果保留（原型
/// 同款「Paused 层保留至续跑后落账」），存档快照携带模块字节+挂起态+
/// 排队效果（增量 2）。
#[derive(Debug, Clone)]
pub struct VmMachine {
    owner: PlayerId,
    program: Arc<VmSkillProgram>,
    trig: VmTriggerCtx,
    /// 排队未落账效果（Halted 提交 / Paused 保留 / Err 丢弃）。
    pending_effects: Vec<EffectRequest>,
    /// 挂起态（含已捕获 continuation 与选项）。
    paused: Option<VmPaused>,
    /// 挂起询问已答（下一步续跑）。
    asked: bool,
    settled: bool,
    /// Halted 时写入 once。挂起恢复后续跑不再次消费（登记差异：清单不入快照）。
    consume: Vec<(String, u8)>,
    count_hits: bool,
}

/// 内核座位 ↔ rule-api id（1 基，NONE=0）。
fn to_rule_pid(p: PlayerId) -> rule_api::PlayerId {
    rule_api::PlayerId::new(p as u32 + 1)
}

/// 内核只读查询 + 排队效果的 VmHost 适配器。
struct EngineVmHost<'a> {
    ctx: HostContext,
    /// 事件目标集（CardUsed 载荷；PA12 批③：HOST_EVENT_TARGET_COUNT 此前恒读
    /// 默认空切片=0——旧线经 run_module_with_event_cards 传入同集）。
    targets: Vec<u32>,
    seats: u8,
    core: &'a crate::state::GameStateCore,
    catalog: &'a crate::catalog::Catalog,
    pending: &'a mut Vec<EffectRequest>,
}

impl VmHost for EngineVmHost<'_> {
    fn capabilities(&self) -> &CapabilitySet {
        // 标准技能能力面（emit/查询/选择申请）；选择挂起本增量未接。
        static CAPS: std::sync::OnceLock<CapabilitySet> = std::sync::OnceLock::new();
        CAPS.get_or_init(CapabilitySet::standard_skill)
    }

    fn context(&self) -> HostContext {
        self.ctx
    }

    fn event_targets(&self) -> &[u32] {
        &self.targets
    }

    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        crate::rvm_ops::accept_effect(&req, self.seats)?;
        self.pending.push(req);
        Ok(())
    }

    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        crate::rvm_ops::run_query(self.core, self.catalog, self.seats, req)
    }
}

/// 效果提交（`accept_effect` 已验证映射面；此处只落账）。

impl VmMachine {
    /// 存档重建（非挂起态）：从模块载体字节还原（快照点不会停在
    /// start→step 中段——此态重建后即待触发执行）。PA06：装载期
    /// 校验——恢复载体畸形即拒（等价于原执行期校验的前移）。
    pub(crate) fn from_module(owner: PlayerId, module: BytecodeModule) -> Result<Self, ()> {
        let skill_name = module.name.clone();
        let program = VmSkillProgram::new(module, skill_name).map_err(|_| ())?;
        Ok(Self {
            owner,
            program: Arc::new(program),
            trig: VmTriggerCtx::default(),
            pending_effects: Vec::new(),
            paused: None,
            asked: false,
            settled: false,
            consume: Vec::new(),
            count_hits: false,
        })
    }

    /// 存档重建（挂起态）：continuation + 选项 + 排队效果还原；内容锁
    /// 核对（continuation 绑定模块 logic_hash，换模块续跑被拒）。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn restore_paused(
        owner: PlayerId,
        module: BytecodeModule,
        frames: Vec<u8>,
        pc: u32,
        fuel_remaining: u32,
        id: u64,
        module_hash: [u8; 32],
        options: Vec<u32>,
        pending_bytes: Vec<u8>,
    ) -> Result<Self, ()> {
        if module.logic_hash != module_hash {
            return Err(());
        }
        let pending = dec_effects(&pending_bytes).ok_or(())?;
        let skill_name = module.name.clone();
        let program = VmSkillProgram::new(module, skill_name).map_err(|_| ())?;
        Ok(Self {
            owner,
            program: Arc::new(program),
            trig: VmTriggerCtx::default(),
            pending_effects: pending,
            paused: Some(VmPaused {
                continuation: Continuation {
                    id,
                    frames,
                    pc,
                    fuel_remaining,
                    module_hash,
                },
                options,
            }),
            // 挂起态快照 = 问询已发出（active_ask 派生同一询问）——下一
            // 应答即续跑，不重问。
            asked: true,
            settled: false,
            consume: Vec::new(),
            count_hits: false,
        })
    }
}

fn enc_effects(w: &mut Vec<u8>, effects: &[EffectRequest]) {
    crate::rvm_ops::enc_effects(w, effects);
}

fn dec_effects(r: &[u8]) -> Option<Vec<EffectRequest>> {
    crate::rvm_ops::dec_effects(r)
}

impl SkillMachine for VmMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Rvm {
            owner: self.owner,
            module: self.program.module.to_bytes(),
            paused: self.paused.as_ref().map(|p| crate::snap::RvmPaused {
                frames: p.continuation.frames.clone(),
                pc: p.continuation.pc,
                fuel_remaining: p.continuation.fuel_remaining,
                id: p.continuation.id,
                module_hash: p.continuation.module_hash,
                options: p.options.clone(),
                pending: {
                    let mut b = Vec::new();
                    enc_effects(&mut b, &self.pending_effects);
                    b
                },
            }),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.settled {
            return Flow::Done;
        }
        // 挂起态：先问（WaitChoice），已答则续跑
        if let Some(p) = self.paused.clone() {
            if !self.asked {
                self.asked = true;
                let options: Vec<ChoiceOption> =
                    p.options.iter().map(|t| ChoiceOption::Tag(*t as u8)).collect();
                return Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "rvm_choice",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            self.asked = false;
            self.paused = None;
            let token = match ctx.answer().first() {
                Some(ChoiceOption::Tag(t)) => *t as u32,
                _ => return self.trap_done(),
            };
            let program = self.program.clone();
            let cont = p.continuation;
            let resumed = move |vm: &mut RuleVm, host: &mut EngineVmHost<'_>| {
                vm.resume(&program.module, host, cont, token)
            };
            return self.run(ctx, resumed);
        }
        let program = self.program.clone();
        let module = program.module.clone();
        let executed = move |vm: &mut RuleVm, host: &mut EngineVmHost<'_>| {
            vm.execute(&module, host, DEFAULT_FUEL)
        };
        self.run(ctx, executed)
    }
}

impl VmMachine {
    /// 执行/续跑公共尾：Halted 提交排队效果；Paused 转挂起询问（排队
    /// 效果保留至续跑后一并落账）；Err 回滚（排队效果整批丢弃）。
    fn run(
        &mut self,
        ctx: &mut Ctx<'_>,
        f: impl FnOnce(&mut RuleVm, &mut EngineVmHost<'_>) -> Result<ExecResult, rule_vm::VmError>,
    ) -> Flow {
        let seats = ctx.core.players.len() as u8;
        let outcome = {
            let targets = self
                .trig
                .targets
                .iter()
                .map(|t| to_rule_pid(*t).raw())
                .collect::<Vec<_>>();
            let mut host = EngineVmHost {
                targets,
                ctx: HostContext {
                    // T8 增量 8：skill_id 经目录反查回填（查不到保持 0=NONE）。
                    skill_id: ctx
                        .catalog
                        .skill_by_name(&self.program.skill_name)
                        .map(|d| d as u32)
                        .unwrap_or(0),
                    self_id: to_rule_pid(self.owner).raw(),
                    event_player: self
                        .trig
                        .event_player
                        .map(to_rule_pid)
                        .map(|p| p.raw())
                        .unwrap_or(0),
                    event_source: self
                        .trig
                        .event_source
                        .map(to_rule_pid)
                        .map(|p| p.raw())
                        .unwrap_or(0),
                    event_card: self.trig.event_card.map(|c| c as u32).unwrap_or(0),
                    event_amount: self.trig.event_amount,
                },
                seats,
                core: ctx.core,
                catalog: ctx.catalog,
                pending: &mut self.pending_effects,
            };
            let mut vm = RuleVm::new();
            f(&mut vm, &mut host)
        };
        match outcome {
            Ok(ExecResult::Halted { .. }) => {
                for (key, scope) in &self.consume {
                    let k = if key.is_empty() {
                        self.program.skill_name.clone()
                    } else {
                        key.clone()
                    };
                    ctx.core.vm_aux.once.insert((self.owner, k, *scope));
                }
                if self.count_hits && self.trig.kind == 1 {
                    *ctx.core
                        .vm_aux
                        .hits
                        .entry((self.owner, self.program.skill_name.clone()))
                        .or_default() += 1;
                }
                let pending = std::mem::take(&mut self.pending_effects);
                let skill = self.program.skill_name.clone();
                for e in pending {
                    crate::rvm_ops::commit_effect(ctx, &skill, e);
                }
                self.settled = true;
                Flow::Done
            }
            Ok(ExecResult::Paused {
                continuation,
                options,
            }) => {
                // 选项超 u8 域（ChoiceOption::Tag 容量）→ 回滚。
                if options.iter().any(|t| *t > 255) {
                    return self.trap_done();
                }
                self.paused = Some(VmPaused {
                    continuation,
                    options,
                });
                // 本次返回的 WaitChoice 就是问询（机器契约：WaitChoice 后下一
                // step 读应答）。PA12④ 原型实证：此处若置 false，下一 step 会
                // 再问一次、首答被丢（恢复路径 restore_paused 早已置 true）。
                self.asked = true;
                let options: Vec<ChoiceOption> = self
                    .paused
                    .as_ref()
                    .expect("just set")
                    .options
                    .iter()
                    .map(|t| ChoiceOption::Tag(*t as u8))
                    .collect();
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "rvm_choice",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            Err(e) => {
                // NC04：故障必须可定位并向上传播——记录技能/座位/事件/错误类别，
                // 由调度循环（StepOutcome::Fault）整段回滚并以 VmFault 拒绝命令。
                ctx.res.fault = Some(crate::machine::VmFault {
                    skill: self.program.skill_name.clone(),
                    owner: self.owner,
                    event: format!("{:?}", self.trig.kind),
                    kind: match &e {
                        rule_vm::VmError::OutOfFuel => "OutOfFuel",
                        rule_vm::VmError::StackOverflow => "StackOverflow",
                        rule_vm::VmError::MemoryLimit => "MemoryLimit",
                        rule_vm::VmError::InvalidBytecode(_) => "InvalidBytecode",
                        rule_vm::VmError::CapabilityDenied(_) => "CapabilityDenied",
                        rule_vm::VmError::Host(_) => "Host",
                        rule_vm::VmError::Arithmetic(_) => "Arithmetic",
                        rule_vm::VmError::BadControlFlow(_) => "BadControlFlow",
                        rule_vm::VmError::Stack(_) => "Stack",
                    }
                    .to_string(),
                    detail: e.to_string(),
                });
                self.trap_done()
            }
        }
    }

    /// 回滚收尾：排队效果整批丢弃（半次效果不落账，原型同款）。
    fn trap_done(&mut self) -> Flow {
        self.pending_effects.clear();
        self.paused = None;
        self.settled = true;
        Flow::Done
    }
}
