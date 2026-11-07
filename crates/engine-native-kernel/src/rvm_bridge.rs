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
pub use rule_vm::BytecodeModule;
use rule_vm::{Continuation, ExecResult, HostContext, RuleVm, VmHost, DEFAULT_FUEL};

use crate::catalog::QueryCtx;
use crate::catalog::{SkillBehavior, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, EventKind};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::MachineSnapshot;
mod context;
mod handlers;
mod identity;
pub use identity::hash_trigger_meta;
mod machine;
pub mod modifiers;
use context::{filter_view, kind_is_ordinary_trick, trigger_ctx_of, VmTriggerCtx};
pub use handlers::{
    VmActiveLimits, VmActiveSelection, VmActiveTargets, VmChoiceActor, VmHandlerSet,
    VmSelectionCardFilter, VmSelectionZone, VmTargetFilter, VmTriggerProgram, VmTurnUseSource,
    VmViewAsFilter, VmViewAsSpec, VmViewAsWindow, VmViewAsZone,
};
pub use modifiers::{
    VmModifierMode, VmModifierRule, VmModifierSelector, VmModifierTarget, VmModifierValueSource,
};

/// 数据驱动的 VM 技能程序（扩展包声明携带；目录 def 的 behavior 持有）。
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn new(module: BytecodeModule, skill_name: String) -> Result<Self, rule_vm::VmError> {
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


/// DSL filter 映射（增量 6 三基 + 增量 9 基元扩展；增量 13 语义收敛——
/// `owner_is_target_*` 家族全部含「事件主玩家=持有者」合取段）：具名
/// 组合 filter 由装载器跳过登记（deferred）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VmFilterPolicy {
    /// Active and trigger modules of one package skill share a catalog identity.
    Handlers(Arc<VmHandlerSet>),
    Declared {
        skill_def_id: u32,
        hand_limit_base_max_hp: bool,
        modifiers: Vec<VmModifierRule>,
        choice_actor: VmChoiceActor,
        inner: Box<VmFilterPolicy>,
    },
    /// DSL `owner_is_target`：事件主玩家=持有者。
    #[default]
    OwnerIsTarget,
    /// DSL `owner_alive`：持有者存活即过闸（**不含**事件主玩家合取——
    /// 他人结束阶段等场合也触发；PA12 批①，样本=清励）。旧线语义同。
    OwnerAlive,
    /// DSL `owner_source`：事件来源=持有者。
    OwnerIsSource,
    /// DSL `owner_alive_gave_cards_to_other_alive`（cards_given 触发窗）：
    /// 持有者=给牌者（来源）、双方存活、给牌者≠接受者、张数>0——旧线
    /// `TriggerFilter::OwnerAliveGaveCardsToOtherAlive` 同谓词。
    OwnerAliveGaveCardsToOtherAlive,
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
    OwnerSkippedPhasePositive,
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
    /// DSL `owner_is_target_other_source_alive`（PA12 批⑤-4，R01 涤仇族）：
    /// 事件主玩家=持有者 ∧ 事件来源存活 ∧ 来源≠持有者。
    OwnerTargetOtherSourceAlive,
    /// DSL `owner_source_target_mark_positive_{mark}`（批⑤-4 R01 参数化族）：
    /// 来源=持有者 ∧ 事件主玩家的该标记层数 > 0。
    OwnerSourceTargetMarkPositive(u32),
    /// DSL `owner_skill_added_self_{def}`：获得技能者=持有者 ∧ 刚获得的技能匹配 `def`。
    OwnerSkillAddedSelf(u32),
    /// DSL `owner_turned_over`：翻面者=持有者（正/背双向）。
    OwnerTurnedOver,
    /// DSL `owner_alive_event_player_became_face_up_or_linked`：持有者与事件角色存活，
    /// 且事件为翻至正面（1）或进入连环状态（3）。
    OwnerBecameFaceUpOrLinked,
    /// DSL `owner_alive_event_player_turn_changed_or_became_linked`：持有者与事件角色存活，
    /// 且事件为翻面（1/2）或进入连环状态（3）。
    OwnerTurnChangedOrBecameLinked,
    /// 阶段门（PA12④）：事件携带的阶段须等于 `phase`，再评估 `inner`。
    /// 装载器用它把 DSL 的 `phase_end`/`phase_draw`/… 落到原生
    /// `PhaseBegin`/`PhaseEnd`（原生阶段帧按阶段逐一发出，旧线
    /// `phase_end` 只指「进入结束阶段」——无门会每阶段触发一次）。
    AtPhase {
        phase: crate::event::Phase,
        inner: Box<VmFilterPolicy>,
    },
    /// 跳过摸牌阶段门（PA12⑫）：仅当 DrawPhase 终窗 outcome=Skipped 时才评估 `inner`。
    DrawPhaseSkipped {
        inner: Box<VmFilterPolicy>,
    },
    /// 游戏开始门（PA12⑬，UP37 R01/R06）：落于首轮（round==1）TurnBegin 窗，
    /// 并以 `once` 键 `("game_started:<skill>", 3)` 保证整局每名持有者仅触发一次。
    GameStarted {
        inner: Box<VmFilterPolicy>,
    },
    /// 转化牌来源技能门（PA12⑭）：`event_view_as_skill_<u32>`。
    EventViewAsSkill(u32),
    /// 任意阶段跳过门（PA12⑭）：`phase_any_skipped`。
    PhaseAnySkipped {
        inner: Box<VmFilterPolicy>,
    },
}

impl VmFilterPolicy {
    pub fn has_view_as(&self) -> bool {
        match self {
            Self::Handlers(set) => !set.view_as.is_empty(),
            Self::Declared { inner, .. }
            | Self::AtPhase { inner, .. }
            | Self::DrawPhaseSkipped { inner }
            | Self::PhaseAnySkipped { inner }
            | Self::GameStarted { inner } => inner.has_view_as(),
            _ => false,
        }
    }

    pub fn vm_view_as(&self) -> &[VmViewAsSpec] {
        match self {
            Self::Handlers(set) => &set.view_as,
            Self::Declared { inner, .. }
            | Self::AtPhase { inner, .. }
            | Self::DrawPhaseSkipped { inner }
            | Self::PhaseAnySkipped { inner }
            | Self::GameStarted { inner } => inner.vm_view_as(),
            _ => &[],
        }
    }
}

/// 把 DSL filter 名映射到策略。未知名称返回 None（装载器跳过，不恒真）。
pub fn map_vm_filter(name: &str) -> Option<VmFilterPolicy> {
    let exact = match name {
        "owner_is_target" => Some(VmFilterPolicy::OwnerIsTarget),
        // PA12 批①：旧线 DSL 实际拼写为 owner_is_source（inventory 206 处），
        // 与既有 owner_source 同义（事件来源=持有者）。
        "owner_source" | "owner_is_source" => Some(VmFilterPolicy::OwnerIsSource),
        "owner_alive_gave_cards_to_other_alive" => {
            Some(VmFilterPolicy::OwnerAliveGaveCardsToOtherAlive)
        }
        "owner_alive" => Some(VmFilterPolicy::OwnerAlive),
        "owner_source_sha" => Some(VmFilterPolicy::OwnerSourceSha),
        "owner_source_fire_sha" => Some(VmFilterPolicy::OwnerSourceFireSha),
        "owner_source_ordinary_trick" => Some(VmFilterPolicy::OwnerSourceOrdinaryTrick),
        "owner_alive_not_target" => Some(VmFilterPolicy::OwnerAliveNotEventPlayer),
        "owner_is_target_source_alive" => Some(VmFilterPolicy::OwnerTargetSourceAlive),
        "owner_is_source_or_target" => Some(VmFilterPolicy::OwnerSourceOrTarget),
        "owner_is_target_other_source_alive" => Some(VmFilterPolicy::OwnerTargetOtherSourceAlive),
        "always" => Some(VmFilterPolicy::Always),
        "owner_is_target_skipped_phase_count_positive" => {
            Some(VmFilterPolicy::OwnerSkippedPhasePositive)
        }
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
        "owner_is_target_tied_max_equip_nonempty" => {
            Some(VmFilterPolicy::OwnerTiedMaxEquipNonempty)
        }
        "owner_is_target_not_unique_max_current_hp" => Some(VmFilterPolicy::OwnerNotUniqueMaxHp),
        "owner_targeted_by_other_sha" => Some(VmFilterPolicy::OwnerTargetedByOtherSha),
        "owner_turned_over" => Some(VmFilterPolicy::OwnerTurnedOver),
        "owner_alive_event_player_became_face_up_or_linked" => {
            Some(VmFilterPolicy::OwnerBecameFaceUpOrLinked)
        }
        "owner_alive_event_player_turn_changed_or_became_linked" => {
            Some(VmFilterPolicy::OwnerTurnChangedOrBecameLinked)
        }
        _ => None,
    };
    if exact.is_some() {
        return exact;
    }
    if let Some(raw) = name
        .strip_prefix("event_view_as_skill_")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
    {
        return Some(VmFilterPolicy::EventViewAsSkill(raw));
    }
    if let Some(raw) = name
        .strip_prefix("owner_skill_added_self_")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
    {
        return Some(VmFilterPolicy::OwnerSkillAddedSelf(raw));
    }
    if let Some(n) = name
        .strip_prefix("owner_is_target_hp_below_")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
    {
        return Some(VmFilterPolicy::OwnerHpBelow(n));
    }
    // PA12 批⑤-4（R01 参数化族，与旧线 map_filter strip_prefix 同口径）
    if let Some(mark) = name
        .strip_prefix("owner_source_target_mark_positive_")
        .and_then(|v| v.parse::<u32>().ok())
    {
        return Some(VmFilterPolicy::OwnerSourceTargetMarkPositive(mark));
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
    fn vm_module_is_authorized(&self, bytes: &[u8]) -> bool {
        if self.program.module.to_bytes() == bytes {
            return true;
        }
        match &self.filter {
            VmFilterPolicy::Handlers(set) => {
                set.active
                    .as_ref()
                    .is_some_and(|active| active.module.to_bytes() == bytes)
                    || set
                        .triggers
                        .iter()
                        .any(|t| t.program.module.to_bytes() == bytes)
            }
            _ => false,
        }
    }

    fn vm_module_requires_trigger_frame(&self, bytes: &[u8]) -> bool {
        match &self.filter {
            VmFilterPolicy::Handlers(set) => {
                !set.active
                    .as_ref()
                    .is_some_and(|active| active.module.to_bytes() == bytes)
                    && set
                        .triggers
                        .iter()
                        .any(|entry| entry.program.module.to_bytes() == bytes)
            }
            _ => self.vm_module_is_authorized(bytes),
        }
    }

    fn vm_trigger_module_is_authorized(&self, bytes: &[u8], event: &EventData) -> bool {
        let VmFilterPolicy::Handlers(set) = &self.filter else {
            return false;
        };
        set.triggers.iter().any(|entry| {
            (entry.meta.event == event.kind()
                || entry.meta.extra_events.contains(&event.kind())
                // The saved payload is canonical damage; the owning frame validates the exact window.
                || (matches!(event, EventData::Damage(_)) && matches!(entry.meta.event,
                    EventKind::DamageBefore|EventKind::Damage|EventKind::DamageBegin|
                    EventKind::DamageBegin1|EventKind::DamageBegin2|EventKind::DamageBegin3|
                    EventKind::DamageBegin4|EventKind::DamageSource|EventKind::DamageEnd|
                    EventKind::DamageAfter|EventKind::DamageZero|EventKind::DamageCancelled))
                || (matches!(event, EventData::DrawPhase { .. }) && matches!(entry.meta.event,
                    EventKind::DrawPhaseBefore|EventKind::DrawPhaseBegin|
                    EventKind::DrawPhaseBegin1|EventKind::DrawPhaseBegin2|
                    EventKind::DrawPhaseEnd|EventKind::DrawPhaseAfter)))
                && entry.program.module.to_bytes() == bytes
        })
    }

    fn vm_trigger_still_matches(
        &self,
        bytes: &[u8],
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> bool {
        let VmFilterPolicy::Handlers(set) = &self.filter else {
            return false;
        };
        set.matching(ctx, owner, event, kind)
            .iter()
            .any(|entry| entry.program.module.to_bytes() == bytes)
    }

    fn vm_trigger_window_is_authorized(&self, bytes: &[u8], kind: EventKind) -> bool {
        match &self.filter {
            VmFilterPolicy::Handlers(set) => set.triggers.iter().any(|entry| {
                (entry.meta.event == kind || entry.meta.extra_events.contains(&kind))
                    && entry.program.module.to_bytes() == bytes
            }),
            _ => self.vm_module_is_authorized(bytes),
        }
    }
    fn filter_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> bool {
        match &self.filter {
            VmFilterPolicy::Handlers(set) => !set.matching(ctx, owner, event, kind).is_empty(),
            _ => self.filter(ctx, owner, event),
        }
    }

    fn hand_limit_base(
        &self,
        core: &crate::state::GameStateCore,
        owner: PlayerId,
    ) -> Option<usize> {
        match &self.filter {
            VmFilterPolicy::Declared {
                hand_limit_base_max_hp: true,
                ..
            } => Some(core.player(owner).max_hp as usize),
            VmFilterPolicy::Handlers(set)
                if set.triggers.iter().any(|entry| {
                    matches!(
                        entry.filter,
                        VmFilterPolicy::Declared {
                            hand_limit_base_max_hp: true,
                            ..
                        }
                    )
                }) =>
            {
                Some(core.player(owner).max_hp as usize)
            }
            _ => None,
        }
    }

    fn vm_modifiers(&self) -> &[VmModifierRule] {
        match &self.filter {
            VmFilterPolicy::Declared { modifiers, .. } => modifiers,
            VmFilterPolicy::Handlers(set) => &set.modifiers,
            _ => &[],
        }
    }

    fn vm_active_limits(&self) -> Option<&VmActiveLimits> {
        match &self.filter {
            VmFilterPolicy::Handlers(set) => Some(&set.limits),
            _ => None,
        }
    }

    fn has_view_as(&self) -> bool {
        self.filter.has_view_as()
    }

    fn vm_view_as(&self) -> &[VmViewAsSpec] {
        self.filter.vm_view_as()
    }

    fn confirmation_actor(
        &self,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> PlayerId {
        match &self.filter {
            VmFilterPolicy::Declared { choice_actor, .. } => choice_actor.resolve(owner, event),
            VmFilterPolicy::Handlers(set) => set
                .triggers
                .iter()
                .find(|entry| entry.meta.event == kind || entry.meta.extra_events.contains(&kind))
                .or_else(|| set.triggers.first())
                .map_or(owner, |entry| entry.choice_actor.resolve(owner, event)),
            _ => owner,
        }
    }

    fn package_skill_id(&self) -> Option<u32> {
        match &self.filter {
            VmFilterPolicy::Handlers(set) => Some(set.skill_def_id),
            VmFilterPolicy::Declared { skill_def_id, .. } => Some(*skill_def_id),
            _ => None,
        }
    }

    fn is_vm_skill(&self) -> bool {
        true
    }

    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        // 增量 13 语义收敛：`owner_is_target_*` 家族的权威语义（编译器
        // lsp 权威描述 "the skill owner is the event player and …"）是
        // 「事件主玩家=持有者」与状态谓词的合取——filter 是触发调度里
        // 唯一的持有者域闸（plan_triggers 只按 EventKind 找候选），漏掉
        // 合取段会让他人事件过闸（炙焚 holder 门同类缺陷）。turn_damage
        // 为本回合「造成」伤害计数（kernel 记在来源侧），与 lsp 一致。
        if let VmFilterPolicy::Declared { inner, .. } = &self.filter {
            return VmSkillBehavior {
                program: self.program.clone(),
                filter: (**inner).clone(),
            }
            .filter(ctx, owner, event);
        }
        if matches!(event, EventData::SkillToggled { enabled: false, .. }) {
            return false;
        }
        if ctx
            .state
            .vm_aux
            .flags
            .get(&(owner, format!("skill_banned_turn:{}", self.program.skill_name)))
            .copied()
            .unwrap_or(0)
            != 0
        {
            return false;
        }
        let mut trig = trigger_ctx_of(event);
        if matches!(event, EventData::OrdinaryJudge { .. }) && trig.event_card.is_none() {
            if let Some(c) = ctx.state.discard.last().copied() {
                trig.event_card = Some(c);
                trig.event_cards = vec![c];
            }
        }
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
        if let VmFilterPolicy::DrawPhaseSkipped { inner } = &self.filter {
            if !trig.draw_phase_skipped {
                return false;
            }
            let gated = VmSkillBehavior {
                program: self.program.clone(),
                filter: (**inner).clone(),
            };
            return gated.filter(ctx, owner, event);
        }
        if let VmFilterPolicy::PhaseAnySkipped { inner } = &self.filter {
            let skipped = trig.draw_phase_skipped
                || trig.event_player.is_some_and(|p| {
                    ctx.state
                        .vm_aux
                        .flags
                        .get(&(p, "vm_skipped_phase_count".to_string()))
                        .copied()
                        .unwrap_or(0)
                        > 0
                });
            if !skipped {
                return false;
            }
            let gated = VmSkillBehavior {
                program: self.program.clone(),
                filter: (**inner).clone(),
            };
            return gated.filter(ctx, owner, event);
        }
        if let VmFilterPolicy::GameStarted { inner } = &self.filter {
            if ctx.state.turn.round > 1
                || ctx.state.vm_aux.once.contains(&(
                    owner,
                    format!("game_started:{}", self.program.skill_name),
                    3,
                ))
            {
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
            VmFilterPolicy::Handlers(set) => {
                !set.matching(ctx, owner, event, event.kind()).is_empty()
            }
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
            VmFilterPolicy::OwnerTargetOtherSourceAlive => {
                // card_targeted（→CardUsed+targets，登记差异）下「事件主玩家」=
                // 目标之一；其余事件沿用通用语义（owner=事件主玩家）。
                let owner_is_subject =
                    is_event_player || (trig.kind == 2 && trig.targets.contains(&owner));
                owner_is_subject
                    && trig
                        .event_source
                        .is_some_and(|src| src != owner && ctx.state.player(src).alive)
            }
            VmFilterPolicy::OwnerSourceTargetMarkPositive(mark) => {
                // 同上：card_targeted 的「目标」在 targets 集（旧线逐目标发
                // 上下文，原生单事件——任一目标带标记即过闸，R01 单目标杀
                // 等价；多目标卡的逐目标窗属 §2 J 选择链登记面）。
                let has_mark = |seat: u8| {
                    ctx.state
                        .vm_aux
                        .marks
                        .get(&(seat, mark))
                        .copied()
                        .unwrap_or(0)
                        > 0
                };
                trig.event_source == Some(owner)
                    && (trig.event_player.is_some_and(has_mark)
                        || trig.targets.iter().any(|t| has_mark(*t)))
            }
            VmFilterPolicy::OwnerSkillAddedSelf(raw) => {
                is_event_player
                    && ctx.state.player(owner).alive
                    && trig.kind == 13
                    && trig.added_skill_def.is_some_and(|def| {
                        def as u32 == raw
                            || ctx.catalog.skills[def as usize].behavior.package_skill_id()
                                == Some(raw)
                    })
            }
            VmFilterPolicy::OwnerTurnedOver => {
                is_event_player && trig.kind == 12 && matches!(trig.event_amount, 1 | 2)
            }
            VmFilterPolicy::OwnerBecameFaceUpOrLinked => {
                ctx.state.player(owner).alive
                    && trig.event_player.is_some_and(|p| ctx.state.player(p).alive)
                    && trig.kind == 12
                    && matches!(trig.event_amount, 1 | 3)
            }
            VmFilterPolicy::OwnerTurnChangedOrBecameLinked => {
                ctx.state.player(owner).alive
                    && trig.event_player.is_some_and(|p| ctx.state.player(p).alive)
                    && trig.kind == 12
                    && matches!(trig.event_amount, 1 | 2 | 3)
            }
            VmFilterPolicy::OwnerIsSource => trig.event_source == Some(owner),
            VmFilterPolicy::OwnerAliveGaveCardsToOtherAlive => {
                ctx.state.player(owner).alive
                    && trig.kind == 9
                    && trig.event_source == Some(owner)
                    && trig
                        .event_player
                        .is_some_and(|p| ctx.state.player(p).alive && p != owner)
                    && trig.event_amount > 0
            }
            VmFilterPolicy::Always => true,
            VmFilterPolicy::OwnerWounded => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hp > 0 && p.hp < p.max_hp as i32
                }
            }
            VmFilterPolicy::OwnerLinked => is_event_player && holder_state(owner).linked,
            VmFilterPolicy::OwnerHandNonempty => {
                is_event_player && !holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerHandEmpty => {
                is_event_player && holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerSkippedPhasePositive => {
                is_event_player
                    && ctx
                        .state
                        .vm_aux
                        .flags
                        .get(&(owner, "vm_skipped_phase_count".to_string()))
                        .copied()
                        .unwrap_or(0)
                        > 0
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
                is_event_player
                    && {
                        let p = holder_state(owner);
                        p.hand.len() >= p.hp as usize
                    }
                    && (0..ctx.state.players.len())
                        .any(|i| i as u8 != owner && ctx.state.players[i].alive)
            }
            // 出牌阶段结束 && 本阶段三类牌已用 ≥1（计数随 Play 开始清
            // 零，事件时点值即本阶段值；ABI 293 的持有者归因由「仅当前
            // 回合玩家可用牌」化归为事件主玩家合取）。
            VmFilterPolicy::OwnerPlayPhaseBteUsed | VmFilterPolicy::OwnerAlivePlayPhaseBteUsed => {
                is_event_player
                    && (trig.phase == Some(crate::event::Phase::Play)
                        || (trig.phase == Some(crate::event::Phase::End)
                            && trig.begin == Some(true)))
                    && (ctx.state.turn.bte_used_play_phase > 0
                        || ctx
                            .state
                            .vm_aux
                            .flags
                            .get(&(owner, "play_cat_mask".to_string()))
                            .copied()
                            .unwrap_or(0)
                            > 0)
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
                is_event_player && ctx.state.turn.cards_used_play_phase == p.hp as u32
            }
            VmFilterPolicy::OwnerCardUsedAtLeastHp => {
                let p = holder_state(owner);
                is_event_player && ctx.state.turn.cards_used_play_phase >= p.hp as u32
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
                is_event_player && elemental | ordinary_trick && holder_state(owner).hand.is_empty()
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
                let Some(src) = trig.event_source else {
                    return false;
                };
                if !ctx.state.players[src as usize].alive || !is_event_player {
                    return false;
                };
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
            VmFilterPolicy::EventViewAsSkill(raw) => {
                let actor = trig.event_player.unwrap_or(owner);
                let flag_raw = ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(actor, "event_view_as_skill".to_string()))
                    .or_else(|| {
                        ctx.state
                            .vm_aux
                            .flags
                            .get(&(owner, "event_view_as_skill".to_string()))
                    })
                    .copied()
                    .unwrap_or(0);
                flag_raw != 0
                    && (flag_raw == raw
                        || ctx
                            .catalog
                            .skills
                            .get(flag_raw as usize)
                            .and_then(|d| d.behavior.package_skill_id())
                            == Some(raw))
            }
            // 阶段门已在函数头处理（递归评估 inner）；此处不可达，保守返回 false。
            VmFilterPolicy::AtPhase { .. }
            | VmFilterPolicy::DrawPhaseSkipped { .. }
            | VmFilterPolicy::PhaseAnySkipped { .. }
            | VmFilterPolicy::GameStarted { .. }
            | VmFilterPolicy::Declared { .. } => false,
            VmFilterPolicy::Named(prog) => crate::rvm_named::eval_program(
                &prog,
                ctx,
                owner,
                &self.program.skill_name,
                &filter_view(&trig),
            ),
        }
    }

    fn start_in_context(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> Box<dyn SkillMachine> {
        self.start_in_window(ctx, owner, event, event.kind())
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> Box<dyn SkillMachine> {
        let VmFilterPolicy::Handlers(set) = &self.filter else {
            return self.start(owner, event);
        };
        let mut matching = set.matching(ctx, owner, event, kind).into_iter();
        let Some(first) = matching.next() else {
            return self.start(owner, event);
        };
        let mut machine = handlers::machine(first.program.clone(), &first.filter, owner, event);
        machine.following = matching
            .map(|entry| {
                handlers::machine(entry.program.clone(), &entry.filter, owner, event).snapshot()
            })
            .collect();
        Box::new(machine)
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let VmFilterPolicy::Handlers(set) = &self.filter else {
            return None;
        };
        let active = set.active.as_ref()?;
        if !set.available(ctx, owner)
            || !(ctx.catalog.hooks.vm_active)(&active.module, &active.skill_name, owner, ctx.state)
        {
            return None;
        }
        let event = EventData::Phase {
            player: owner,
            phase: crate::event::Phase::Play,
            begin: true,
        };
        let mut machine = handlers::machine(active.clone(), &VmFilterPolicy::Always, owner, &event);
        if set.limits.once_per_game {
            machine.consume.push((String::new(), 3));
        }
        if set.limits.once_per_turn
            || set.limits.uses_per_turn.is_some()
            || set.limits.uses_per_turn_source.is_some()
        {
            machine.consume.push((String::new(), 1));
        }
        if set.limits.once_per_play_phase || set.limits.uses_per_play_phase.is_some() {
            machine.consume.push((String::new(), 4));
        }
        if set.limits.target_other {
            machine.consume.push((
                "__target_other".to_string(),
                set.limits.target_max_hp_gt.unwrap_or(0),
            ));
        }
        Some(Box::new(machine))
    }

    fn needs_vm_card_history(&self) -> bool {
        true
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(handlers::machine(
            self.program.clone(),
            &self.filter,
            owner,
            event,
        ))
    }
}

/// VM 挂起态（HOST_CHOICE）：continuation + 选项 tokens。
#[derive(Debug, Clone)]
pub struct VmPaused {
    pub continuation: Continuation,
    pub options: Vec<u32>,
}

/// J1（ADR 0786 §A，规则 v328）：**提交期挂起**——Halted 逐效果落账遇
/// `discard_hand_equip_count(N>0)` 时不再内联「确定性首 N」近似，而是暂停
/// 落账、向开窗座位发真实选牌窗（恰好 N 张手牌/装备；不足以可用数封顶，
/// 无牌空过）；应答后按所选弃置并继续提交剩余队列（队列保留在
/// `pending_effects`，头元素即本开窗效果）。与 `paused`（VM 执行期挂起）
/// 互斥——同一时刻至多一种挂起。
#[derive(Debug, Clone)]
struct VmCommitPause {
    /// 开窗座位（效果的目标玩家；问询 observer）。
    player: PlayerId,
    /// 弃置张数（min=max=封顶后的恰好值）。
    count: u32,
}

/// VM 技能机器：可挂起（HOST_CHOICE）——挂起期间排队效果保留（原型
/// 同款「Paused 层保留至续跑后落账」），存档快照携带模块字节+挂起态+
/// 排队效果（增量 2）；B7（规则 v324）起另携带声明技能名、触发事件
/// 原文、限次清单与命中旗标，恢复不再从模块名重建或重置这些字段。
#[derive(Debug, Clone)]
pub struct VmMachine {
    owner: PlayerId,
    program: Arc<VmSkillProgram>,
    trig: VmTriggerCtx,
    /// 触发事件原文（快照载体：恢复时经 `trigger_ctx_of` 重派生 trig）。
    event: EventData,
    /// 排队未落账效果（Halted 提交 / Paused 保留 / Err 丢弃）。
    pending_effects: Vec<EffectRequest>,
    /// 挂起态（含已捕获 continuation 与选项）。
    paused: Option<VmPaused>,
    /// 挂起询问已答（下一步续跑）。
    asked: bool,
    settled: bool,
    /// Halted 时写入 once（快照携带：挂起恢复后续跑不丢限次消费）。
    consume: Vec<(String, u8)>,
    count_hits: bool,
    /// J1：提交期挂起（开窗参数；剩余队列在 `pending_effects`）。
    commit_pause: Option<VmCommitPause>,
    following: Vec<MachineSnapshot>,
    commit_state: Vec<u32>,
    committing: bool,
}

#[cfg(test)]
mod damage_context_tests;

