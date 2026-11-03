//! 显式事件栈调度器。
//!
//! 帧的推进是一条均匀状态机：
//! `BeforeTriggers -> Commit -> AfterTriggers (-> Intrinsic) -> Done -> 弹栈`；
//! 各阶段进入时收集触发队列（`planned` 门闩），队列按全序键逐个运行。
//! 机器可随时 emit 子帧并挂起（`parks` 按恢复深度唤醒），可请求选择
//! （`open_choice` + 应答校验后恢复）。预算耗尽时停止并让上层回滚。

use crate::catalog::{QueryCtx, SkillMachine};
use crate::choice::{ChoiceOption, OpenChoice, Selection};
use crate::event::{EventData, EventKind, Frame, QueuedTrigger, Stage, Timing};
use crate::ids::{EventId, PlayerId, SkillDefId};
use crate::kernel;
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;
mod trigger_order;
pub(crate) use trigger_order::event_kind as trigger_window_kind;
pub use trigger_order::{
    refresh_order_choice, register_added, sync_trigger_owners, validate_trigger_bindings,
    TriggerOrder,
};

/// 挂起的机器与其归属槽位。
/// 无帧哨兵（N11 主动技等会话级挂起）：槽不关联任何栈帧时使用，
/// 恢复校验对其豁免帧存在性检查（真实帧 id 从事件号单调递增，不会撞上）。
pub const NO_FRAME: u32 = u32::MAX;

pub struct Park {
    /// 栈深回到该值时恢复。
    pub resume_depth: usize,
    pub slot: Slot,
    pub machine: Box<dyn SkillMachine>,
    /// 本机是否在等待「已开窗询问」的应答（运行时标记，不入快照/哈希）。
    ///
    /// 同一栈深可以同时停着多台机器（嵌套触发/AOE 逐目标派发/恢复后的
    /// 追加挂起）：应答只属于**提出询问的那一台**，按栈深顺序取第一台会
    /// 把应答交给没有提问的机器，被真实提问者随后读 `ctx.answer` 时
    /// panic（恢复后继续 `autoStep` 的 `expect_failed` 根因）。`run` 在机器
    /// 返回 `Flow::WaitChoice` 时置真、恢复该机时清除；读档时由
    /// `snap::active_ask` 与开放选择逐字段匹配唯一重建（见
    /// `Engine::restore_with_catalog`）。
    pub awaiting_answer: bool,
}

impl Clone for Park {
    fn clone(&self) -> Self {
        Self {
            resume_depth: self.resume_depth,
            slot: self.slot,
            machine: self.machine.fork(),
            awaiting_answer: self.awaiting_answer,
        }
    }
}

/// The formal ready consumer: effect/admission parks precede their suspended order park.
/// This read-only selection conveys neither answer nor execution authority.
pub fn ready_park_index(res: &Resolution) -> Option<usize> {
    res.parks
        .iter()
        .position(|park| {
            park.resume_depth == res.stack.len() && park.machine.trigger_order().is_none()
        })
        .or_else(|| {
            res.parks
                .iter()
                .position(|park| park.resume_depth == res.stack.len())
        })
}

/// 机器归属：帧固有流程，或某帧上的一次触发。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Intrinsic { frame: EventId },
    Trigger { frame: EventId },
}

/// 执行位置：事件栈 + 挂起机器 + 开放选择。
#[derive(Clone)]
pub struct Resolution {
    pub next_event_id: EventId,
    pub next_choice_id: u64,
    pub stack: Vec<Frame>,
    pub parks: Vec<Park>,
    pub open_choice: Option<OpenChoice>,
    /// 已校验、待机器读取的应答。
    pub answer: Option<Selection>,
    pub budget: u32,
    /// VM 执行故障（NC04，瞬态：命令结束时清空，不入快照/哈希）。
    pub fault: Option<crate::machine::VmFault>,
}

impl Resolution {
    pub fn new(budget: u32) -> Self {
        Self {
            next_event_id: 1,
            next_choice_id: 1,
            stack: Vec::new(),
            parks: Vec::new(),
            open_choice: None,
            answer: None,
            budget,
            fault: None,
        }
    }

    pub fn frame(&self, id: EventId) -> Option<&Frame> {
        self.stack.iter().find(|f| f.id == id)
    }

    pub fn frame_mut(&mut self, id: EventId) -> Option<&mut Frame> {
        self.stack.iter_mut().find(|f| f.id == id)
    }
}

impl std::fmt::Debug for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolution")
            .field("next_event_id", &self.next_event_id)
            .field("next_choice_id", &self.next_choice_id)
            .field("stack", &self.stack)
            .field("parks", &self.parks.len())
            .field("open_choice", &self.open_choice)
            .field("budget", &self.budget)
            .finish()
    }
}

/// 压入事件帧（命令处理器与机器共用的入栈路径）。
pub fn push_frame(res: &mut Resolution, data: crate::event::EventData) -> EventId {
    let id = res.next_event_id;
    res.next_event_id += 1;
    let parent = res.stack.last().map(|f| f.id);
    let kind = data.kind();
    let stage = match &data {
        EventData::Damage(_) => Stage::Intrinsic,
        // R05：D/R/G 均为纯机器帧，无 commit/after 触发段。
        EventData::DrawPhase { .. } | EventData::Draw { .. } | EventData::DrawGain { .. } => {
            Stage::Intrinsic
        }
        _ => match kind {
            // E03 翻面：前置取消窗口（漩涡面具等锁定技在 Commit 前派发）
            EventKind::TurnOverBefore => Stage::BeforeTriggers,
            // 装备前置窗口（swd 批 143 改造，规则 v318）：BeforeTriggers 派发
            // useCardToBegin 类技（改造改写装备栏后再入槽）
            EventKind::Equipped => Stage::BeforeTriggers,
            // 死亡前置窗口（规则 v170）：BeforeTriggers 收集 dieBegin 类遗
            // 言/拦截技（changan 氏），Intrinsic 仍为 DeathMachine。
            EventKind::Death => Stage::BeforeTriggers,
            // 阶段开始前置窗口（PA12 批⑤-2，计划 §2 K）：PhaseBegin 触发的
            // 技能（VM/宿主）须在阶段入场行为（commit_phase→begin_*：摸牌等）
            // **之前**执行——「阶段开始时可跳过/改写本阶段」类技能（奋励
            // skip_current_phase）依赖此前置窗；此前起步 Commit 段导致 VM 机
            // 器在摸牌后才跑（批⑤首步实测：确认门打开时已摸 2）。
            EventKind::PhaseBegin => Stage::BeforeTriggers,
            // R05（规则 333）：回合开始窗——触发在 Commit（压准备阶段）之前。
            EventKind::TurnBegin => Stage::BeforeTriggers,
            // R05（K21）：整回合结束窗两段——BeforeTriggers 派发，after 段静默。
            EventKind::TurnPhaseEnd | EventKind::TurnPhaseAfter => Stage::BeforeTriggers,
            _ => Stage::Commit,
        },
    };
    res.stack.push(Frame {
        id,
        parent,
        kind,
        data,
        stage,
        queue: Vec::new(),
        planned: false,
        intrinsic_done: false,
    });
    id
}

/// 单次 `run` 的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// 栈空且无挂起：本段结算完成。
    Done,
    /// 等待选择应答。
    Pending,
    /// 预算耗尽（上层应回滚）。
    BudgetExhausted,
    /// 宿主时间片耗尽：状态在当前步边界一致，可在续行时无损接续
    /// （与硬性预算上限不同——不回滚，见 ADR 0707）。
    Yielded,
    /// VM 执行故障（NC04）：上层必须整段回滚并以 `RejectReason::VmFault`
    /// 拒绝命令，不得按正常规则继续。
    Fault,
}

/// 阶段推进表。
fn next_stage(kind: EventKind, cur: Stage) -> Stage {
    match (kind, cur) {
        (EventKind::Damage, Stage::BeforeTriggers) => Stage::Commit,
        (EventKind::Damage, Stage::Commit) => Stage::AfterTriggers,
        (EventKind::Damage, Stage::AfterTriggers) => Stage::Done,
        (EventKind::Damage, Stage::Intrinsic) => Stage::Done,
        // E03 翻面：BeforeTriggers 派发前置取消（漩涡面具写回 cancelled）
        // → AfterTriggers 静默（无 after 触发）→ Commit 执行翻转。
        (EventKind::TurnOverBefore, Stage::BeforeTriggers) => Stage::AfterTriggers,
        (EventKind::TurnOverBefore, Stage::AfterTriggers) => Stage::Commit,
        (EventKind::TurnOverBefore, Stage::Commit) => Stage::Done,
        (EventKind::Equipped, Stage::BeforeTriggers) => Stage::Commit,
        (EventKind::CardUsed, Stage::Commit) => Stage::AfterTriggers,
        (EventKind::CardUsed, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::CardUsed, Stage::Intrinsic) => Stage::Done,
        (EventKind::Dying, Stage::Commit) => Stage::AfterTriggers,
        (EventKind::Dying, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::Dying, Stage::Intrinsic) => Stage::Done,
        (EventKind::Death, Stage::BeforeTriggers) => Stage::AfterTriggers,
        (EventKind::Death, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::Death, Stage::Intrinsic) => Stage::Done,
        // PA12 批⑤-2：PhaseBegin 前置窗（技能跑位）→ Commit（入场行为
        // begin_*）→ AfterTriggers → Intrinsic → Done。
        (EventKind::PhaseBegin, Stage::BeforeTriggers) => Stage::Commit,
        // R05：回合开始窗 BeforeTriggers → Commit（压准备阶段帧）→ 静默 after → Done。
        (EventKind::TurnBegin, Stage::BeforeTriggers) => Stage::Commit,
        (EventKind::TurnPhaseEnd | EventKind::TurnPhaseAfter, Stage::BeforeTriggers) => {
            Stage::Commit
        }
        (EventKind::PhaseBegin, Stage::Commit) => Stage::AfterTriggers,
        (EventKind::PhaseBegin, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::PhaseBegin, Stage::Intrinsic) => Stage::Done,
        (EventKind::PhaseEnd, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::PhaseEnd, Stage::Intrinsic) => Stage::Done,
        (
            EventKind::SkillBefore
            | EventKind::SkillBegin
            | EventKind::SkillEnd
            | EventKind::SkillAfter
            | EventKind::SkillOmitted,
            Stage::AfterTriggers,
        ) => Stage::Intrinsic,
        (EventKind::OrdinaryJudge, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::OrdinaryJudge, Stage::Intrinsic) => Stage::Done,
        // R05：D/R/G 由各自机器在 Intrinsic 段推进直至 Done。
        (
            EventKind::DrawPhase | EventKind::Draw | EventKind::DrawGain,
            Stage::Intrinsic,
        ) => Stage::Done,
        (_, Stage::Commit) => Stage::AfterTriggers,
        (_, Stage::AfterTriggers) => Stage::Done,
        (_, cur) => cur,
    }
}

/// 可选触发的确认闸门：确认后才启动真正的技能机器。
pub struct OptionalGate {
    owner: PlayerId,
    def: SkillDefId,
    /// 计划时的事件载荷（触发语义以进入时为准）。
    event: crate::event::EventData,
    asked: bool,
    inner: Option<Box<dyn SkillMachine>>,
}

impl OptionalGate {
    /// 运行时构造（触发入队时）与快照恢复共用。
    pub(crate) fn new(
        owner: PlayerId,
        def: SkillDefId,
        event: crate::event::EventData,
        asked: bool,
        inner: Option<Box<dyn SkillMachine>>,
    ) -> Self {
        Self {
            owner,
            def,
            event,
            asked,
            inner,
        }
    }
}

impl SkillMachine for OptionalGate {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            owner: self.owner,
            def: self.def,
            event: self.event.clone(),
            asked: self.asked,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::OptionalGate {
            owner: self.owner,
            def: self.def,
            event: self.event.clone(),
            asked: self.asked,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 内层机器存在即全权委托（其后续选择由内层自行发起）
        if let Some(inner) = self.inner.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                return Flow::Done;
            }
            return flow;
        }
        let Some(frame) = ctx.res.stack.last() else {
            return Flow::Done;
        };
        let qt = QueuedTrigger {
            owner: self.owner,
            def: self.def,
            order: (0, 0, 0, 0, self.def),
        };
        if !trigger_order::eligible(ctx.core, ctx.catalog, &qt, &ctx.res.stack, frame) {
            return Flow::Done;
        }
        let kind = trigger_order::event_kind(frame);
        self.event = frame.data.clone();
        if !self.asked {
            self.asked = true;
            return Flow::WaitChoice {
                observer: self.owner,
                spec: crate::choice::ChoiceSpec::confirm("trigger_confirm"),
            };
        }
        if !ctx.answer().contains(&ChoiceOption::Confirm) {
            return Flow::Done;
        }
        {
            let def = &ctx.catalog.skills[self.def as usize];
            if def.limits.once_per_turn {
                if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, self.def)) {
                    if inst.once_used {
                        return Flow::Done;
                    }
                    inst.once_used = true;
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: def.name,
                owner: self.owner,
            });
            let event =
                crate::event::damage_event(&ctx.res.stack, ctx.res.stack.last().expect("frame"))
                    .clone();
            self.inner = Some(
                if let Some(effect) = def.behavior.native_damage_effect(kind) {
                    native_machine(self.owner, self.def, effect)
                } else {
                    def.behavior
                        .start_in_window(&ctx.query(), self.owner, &event, kind)
                },
            );
        }
        let flow = self.inner.as_mut().expect("inner started").step(ctx);
        if matches!(flow, Flow::Done) {
            self.inner = None;
            Flow::Done
        } else {
            flow
        }
    }
}

/// 收集当前阶段的触发队列。
fn plan_triggers(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    frame_id: EventId,
    before: bool,
) {
    let kind = {
        let f = res.frame(frame_id).expect("frame");
        // before 阶段：伤害帧查 DamageBefore（伤害前置窗口）；翻面帧查
        // TurnOverBefore（E03 翻面前置取消窗口，漩涡面具等锁定技）；
        // 其余帧无前置窗口。
        // 翻面帧的 after 窗口静默：目录无「翻面后」触发种类，且 after
        // 再查 TurnOverBefore 会让取消技重复入队重复写回。
        if !before
            && (matches!(f.data, EventData::TurnOverBefore { .. })
                || matches!(f.data, EventData::Death { .. })
                || matches!(f.data, EventData::TurnBegin { .. })
                || matches!(
                    f.data,
                    EventData::TurnEndWindow { .. }
                ))
        {
            // E03 翻面 / 死亡前置窗口（规则 v170）：目录无「翻面后/死亡
            // 后」触发种类，且 after 再查会令取消/遗言技重复入队。
            let f = res.frame_mut(frame_id).expect("frame");
            f.queue = Vec::new();
            f.planned = true;
            return;
        }
        let before_kind = match &f.data {
            EventData::TurnOverBefore { .. } => EventKind::TurnOverBefore,
            EventData::Death { .. } => EventKind::Death,
            // PA12 批⑤-2（§2 K）：PhaseBegin 前置窗按帧自身 kind 查——
            // 阶段开始类技能（奋励 skip 系）在入场行为之前派发；after 段
            // 静默（上方列表）防同技重复入队。宿主 PhaseBegin 技能的执行
            // 点随此前移到 begin_* 之前（上游 phaseBegin 语义），全量回归
            // 把关、规则基准登记。
            EventData::Phase { begin: true, .. } => EventKind::PhaseBegin,
            // R05：回合开始窗只有前置段（after 静默，见上）。
            EventData::TurnBegin { .. } => EventKind::TurnBegin,
            EventData::TurnEndWindow { after, .. } => {
                if *after {
                    EventKind::TurnPhaseAfter
                } else {
                    EventKind::TurnPhaseEnd
                }
            }
            _ => EventKind::DamageBefore,
        };
        if before {
            before_kind
        } else {
            f.kind
        }
    };
    if let EventData::Damage(d) = &res.frame(frame_id).expect("frame").data {
        if !before
            && (d.settlement.committed_damage.is_none()
                || d.settlement.outcome != crate::event::DamageOutcome::Running
                || d.settlement.notrigger
                || d.amount <= 0)
        {
            let f = res.frame_mut(frame_id).unwrap();
            f.queue.clear();
            f.planned = true;
            return;
        }
    }
    let mut entries: Vec<QueuedTrigger> = Vec::new();
    let turn_current = core.turn.current;
    // PA01 活跃触发索引：候选遍历的代价与本局是否持有无关（目录 6215 技能
    // 档曾令 8 座提交 p50 ×3.3），改为从实例侧单遍出发，直接比对定义的
    // 触发元数据——dispatch 列表即由 event/extra_events 构建，成员关系
    // 严格等价。入队集合不变；最终顺序由下方含 def_id 的全序键决定，
    // 与遍历序无关。
    let mut providers: Vec<_> = core.skills.keys().copied().collect();
    for &def in catalog.candidates(kind) {
        if catalog.skills[def as usize]
            .behavior
            .native_damage_effect(kind)
            .is_some_and(|e| e.helper_name() == Some(catalog.skills[def as usize].name))
        {
            providers.extend((0..core.players.len() as PlayerId).map(|owner| (owner, def)));
        }
    }
    providers.sort_unstable();
    providers.dedup();
    for (owner, def_id) in &providers {
        let def = &catalog.skills[*def_id as usize];
        let native_meta = def.behavior.native_damage_effect(kind).map(|e| e.meta());
        let Some(meta) = native_meta.as_ref().or(def.trigger.as_ref()) else {
            continue;
        };
        if meta.event != kind && !meta.extra_events.contains(&kind) {
            continue;
        }
        // PA12 批⑤-2：PhaseBegin 前置窗只前移 VM 技能——宿主技保持在
        // 入场行为之后的原执行点（61 个宿主 PhaseBegin 技能与 golden 的
        // 行为不变；全量回归实测：不过滤时 16 个宿主技能/golden 失败）。
        // 前置窗（before=true）：PhaseBegin 只前移 VM 技能（宿主技保持
        // 原执行点——61 个宿主 PhaseBegin 技能与 golden 行为不变；全量实测
        // 不过滤时 16 项失败）；after 段（before=false）：VM 技能已在前置
        // 窗跑过，跳过防重复入队，宿主技照常派发。
        if kind == EventKind::PhaseBegin && def.behavior.is_vm_skill() != before {
            continue;
        }
        // NC01：触发结算顺序按**顺座次序**（不是最短距离）
        let seat_rel = core.seat_steps_from(turn_current, *owner);
        entries.push(QueuedTrigger {
            def: *def_id,
            owner: *owner,
            order: (
                match meta.timing {
                    Timing::First => 0,
                    Timing::Normal => 1,
                    Timing::Last => 2,
                },
                meta.priority,
                0,
                seat_rel,
                *def_id,
            ),
        });
    }
    entries.sort_by(trigger_order::compare);
    let f = res.frame_mut(frame_id).expect("frame");
    f.queue = entries;
    f.planned = true;
}

/// 调度主循环。`slice` 为宿主时间片（None = 不限）：每完成一个调度步
/// 计一次数，片内步数用尽即在步边界返回 `Yielded`——状态此时自洽，
/// 续行（重新调用 run）与不分片连续运行产出完全相同的轨迹（ADR 0707）。
/// `budget` 是段内硬上限：耗尽返回 `BudgetExhausted` 由上层回滚。
pub fn run(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
    slice: Option<u32>,
) -> StepOutcome {
    let mut steps: u32 = 0;
    loop {
        if let Some(fault) = res.fault.take() {
            res.fault = Some(fault);
            return StepOutcome::Fault;
        }
        if res.budget == 0 {
            return StepOutcome::BudgetExhausted;
        }
        if let Some(limit) = slice {
            if steps >= limit {
                return StepOutcome::Yielded;
            }
        }
        steps += 1;
        res.budget -= 1;

        // Canonical source liveness is refreshed before every collector/choice/confirmation/start.
        for frame in &mut res.stack {
            if let EventData::Damage(d) = &mut frame.data {
                if d.source.is_some_and(|source| !core.player(source).alive) {
                    d.settlement.cleared_source = d.source;
                    d.source = None;
                }
            }
        }

        // A) 恢复深度匹配的挂起机器
        if res.open_choice.is_none() {
            // 有已校验应答时只唤醒**提出询问的那台**机器：同一栈深可能停着
            // 多台（嵌套触发/AOE 逐目标派发/恢复件里的追加挂起），按顺序取
            // 第一台会把应答交给没有提问的机器，真实提问者随后读
            // `ctx.answer` 即 panic（恢复后继续 autoStep 的 expect_failed
            // 根因）。无应答时保持原语义：同深度按挂起顺序恢复。
            let wake = if res.answer.is_some() {
                res.parks
                    .iter()
                    .position(|p| p.resume_depth == res.stack.len() && p.awaiting_answer)
            } else {
                ready_park_index(res)
            };
            if let Some(idx) = wake {
                let mut park = res.parks.remove(idx);
                // 恢复即消费该标记：下一次提问会重新置真。
                park.awaiting_answer = false;
                let prior_skills: std::collections::BTreeSet<_> =
                    core.skills.keys().copied().collect();
                let flow = {
                    let mut ctx = Ctx {
                        core,
                        res,
                        catalog,
                        out,
                    };
                    park.machine.step(&mut ctx)
                };
                notify_new_skills(core, res, catalog, &prior_skills);
                res.answer = None; // 应答只消费一次
                match flow {
                    Flow::Done => finish_slot(res, park.slot),
                    Flow::Yield => {
                        if park.machine.trigger_order().is_some() {
                            res.parks.insert(0, park);
                        } else {
                            res.parks.push(park);
                        }
                    }
                    Flow::WaitChoice { observer, spec } => {
                        let id = res.next_choice_id;
                        res.next_choice_id += 1;
                        out.push(LogEvent::ChoiceOpened {
                            id,
                            observer,
                            prompt: spec.prompt,
                        });
                        res.open_choice = Some(OpenChoice { id, observer, spec });
                        // 应答恢复点＝**开窗时刻**的栈深。机器可能在挂起期间
                        // 发射过子帧（如 AOE 逐目标派发：前一目标的伤害帧
                        // 尚未结算就为下一目标开响应窗），此时沿用创建时的
                        // 旧深度会与其它挂起机器错配——应答被深度恰好相等
                        // 的**别的**机器消费（AOE→昏迷应答错配：目标机器
                        // 恢复时 answer 已空，panic 实证）。开窗后对局暂停
                        // （open_choice 存在时不推进栈），故开窗时刻的栈深
                        // 即恢复点。
                        park.resume_depth = res.stack.len();
                        // 本机提出询问 → 应答归本机（同深度多机时唯一定位）。
                        park.awaiting_answer = true;
                        res.parks.push(park);
                        return StepOutcome::Pending;
                    }
                }
                continue;
            }
            // 应答在场却无可交付的挂起机器（如 VM 提交钩子自行消费应答、
            // 或承接关系已损坏）：**不转交**给没有提问的机器，照常推进栈顶。
            // 合法读档不会走到这里——`validate_resolution` 拒绝「有应答、
            // 无开放选择」的档，恢复件因此总有唯一认领者。
        }

        // B) 栈顶帧推进
        let Some(top) = res.stack.last().map(|f| f.id) else {
            return StepOutcome::Done;
        };
        let (stage, queue_len, planned, intrinsic_done) = {
            let f = res.frame(top).expect("top frame");
            (f.stage, f.queue.len(), f.planned, f.intrinsic_done)
        };
        match stage {
            Stage::BeforeTriggers | Stage::AfterTriggers => {
                if !planned {
                    plan_triggers(core, res, catalog, top, stage == Stage::BeforeTriggers);
                    res.parks.push(Park {
                        resume_depth: res.stack.len(),
                        slot: Slot::Trigger { frame: top },
                        awaiting_answer: false,
                        machine: Box::new(TriggerOrder::new(top, core)),
                    });
                    continue;
                }
                assert_eq!(queue_len, 0, "window machine drains its frame queue");
                let kind = res.frame(top).expect("frame").kind;
                let next = next_stage(kind, stage);
                let f = res.frame_mut(top).expect("frame");
                f.stage = next;
                f.planned = false;
            }
            Stage::Commit => {
                let prior_skills: std::collections::BTreeSet<_> =
                    core.skills.keys().copied().collect();
                {
                    let mut ctx = Ctx {
                        core,
                        res,
                        catalog,
                        out,
                    };
                    kernel::commit_frame(&mut ctx, top);
                }
                notify_new_skills(core, res, catalog, &prior_skills);
                let kind = res.frame(top).expect("frame").kind;
                let next = next_stage(kind, Stage::Commit);
                let f = res.frame_mut(top).expect("frame");
                f.stage = next;
                f.planned = false;
            }
            Stage::Intrinsic => {
                if intrinsic_done {
                    if matches!(
                        res.frame(top).expect("frame").data,
                        EventData::SkillInvocation(_)
                    ) {
                        let deferred = match &mut res.frame_mut(top).expect("frame").data {
                            EventData::SkillInvocation(d)
                                if d.phase == crate::event::InvocationPhase::After
                                    && d.deferred_log =>
                            {
                                d.deferred_log = false;
                                Some((d.owner, d.def))
                            }
                            _ => None,
                        };
                        if let Some((owner, def)) = deferred {
                            let mut ctx = Ctx {
                                core,
                                res,
                                catalog,
                                out,
                            };
                            ctx.emit(EventData::SkillLog {
                                owner,
                                def,
                                begin: false,
                            });
                        } else {
                            kernel::advance_invocation(res.frame_mut(top).expect("frame"));
                        }
                        continue;
                    }
                    let kind = res.frame(top).expect("frame").kind;
                    let next = next_stage(kind, Stage::Intrinsic);
                    res.frame_mut(top).expect("frame").stage = next;
                    continue;
                }
                let machine = {
                    let mut ctx = Ctx {
                        core,
                        res,
                        catalog,
                        out,
                    };
                    kernel::intrinsic_machine(&mut ctx, top)
                };
                match machine {
                    Some(m) => res.parks.push(Park {
                        resume_depth: res.stack.len(),
                        slot: Slot::Intrinsic { frame: top },
                        machine: m,
                        awaiting_answer: false,
                    }),
                    None => res.frame_mut(top).expect("frame").intrinsic_done = true,
                }
            }
            Stage::Done => {
                res.stack.pop();
            }
        }
    }
}

fn native_machine(
    owner: PlayerId,
    def: SkillDefId,
    effect: crate::event::NativeDamageEffect,
) -> Box<dyn SkillMachine> {
    if crate::event::uses_native_invocation(effect) {
        Box::new(kernel::NativeInvocation {
            owner,
            def,
            window: effect.window(),
            progress: crate::event::NativeInvocationProgress::Ready,
        })
    } else {
        Box::new(kernel::NativeDamageSkill {
            owner,
            def,
            window: effect.window(),
        })
    }
}

fn start_queued_trigger(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
    frame_id: EventId,
    def_id: SkillDefId,
    owner: PlayerId,
) {
    let Some(index) = res.frame(frame_id).and_then(|f| {
        f.queue
            .iter()
            .position(|q| q.def == def_id && q.owner == owner)
    }) else {
        return;
    };
    let qt = res.frame_mut(frame_id).expect("frame").queue.remove(index);
    let event = res.frame(frame_id).expect("frame").data.clone();
    let actual_event =
        crate::event::damage_event(&res.stack, res.frame(frame_id).expect("frame")).clone();
    if !trigger_order::eligible(
        core,
        catalog,
        &qt,
        &res.stack,
        res.frame(frame_id).expect("frame"),
    ) {
        return;
    }
    let def = &catalog.skills[qt.def as usize];
    let kind = trigger_order::event_kind(res.frame(frame_id).expect("frame"));
    let native_meta = def.behavior.native_damage_effect(kind).map(|e| e.meta());
    let meta = native_meta
        .as_ref()
        .or(def.trigger.as_ref())
        .expect("eligible meta");
    let machine: Box<dyn SkillMachine> = if meta.optional {
        Box::new(OptionalGate {
            owner: qt.owner,
            def: qt.def,
            event,
            asked: false,
            inner: None,
        })
    } else {
        if def.limits.once_per_turn {
            let inst = core.skills.get_mut(&(qt.owner, qt.def)).expect("instance");
            if inst.once_used {
                return;
            }
            inst.once_used = true;
        }
        let native = def.behavior.native_damage_effect(kind);
        if !matches!(
            native,
            Some(
                crate::event::NativeDamageEffect::Qinglong
                    | crate::event::NativeDamageEffect::Zhangwu
                    | crate::event::NativeDamageEffect::Yuance
                    | crate::event::NativeDamageEffect::Jidian
                    | crate::event::NativeDamageEffect::JidingRecover
            )
        ) {
            out.push(LogEvent::SkillTriggered {
                skill: def.name,
                owner: qt.owner,
            });
        }
        if let Some(effect) = native {
            res.parks.push(Park {
                resume_depth: res.stack.len(),
                slot: Slot::Trigger { frame: frame_id },
                awaiting_answer: false,
                machine: native_machine(qt.owner, qt.def, effect),
            });
            return;
        }
        def.behavior.start_in_window(
            &QueryCtx {
                state: core,
                catalog,
            },
            qt.owner,
            &actual_event,
            trigger_order::event_kind(res.frame(frame_id).expect("frame")),
        )
    };
    res.parks.push(Park {
        resume_depth: res.stack.len(),
        slot: Slot::Trigger { frame: frame_id },
        machine,
        awaiting_answer: false,
    });
}

fn finish_slot(res: &mut Resolution, slot: Slot) {
    if let Slot::Intrinsic { frame } = slot {
        if frame == NO_FRAME {
            return; // 会话级挂起槽无关联帧
        }
        if let Some(f) = res.frame_mut(frame) {
            f.intrinsic_done = true;
        }
    }
}

// Core equipment/general assembly has no Resolution dependency; observe new keys at its step boundary.
fn notify_new_skills(
    core: &GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    prior: &std::collections::BTreeSet<(PlayerId, SkillDefId)>,
) {
    sync_trigger_owners(core, res);
    for &(owner, def) in core.skills.keys().filter(|key| !prior.contains(key)) {
        register_added(core, res, catalog, owner, def);
    }
    let mut helpers = Vec::new();
    for frame in &res.stack {
        if !matches!(frame.data, EventData::DamageWindow { .. }) {
            continue;
        }
        let kind = trigger_order::event_kind(frame);
        for &def in catalog.candidates(kind) {
            let skill = &catalog.skills[def as usize];
            if !skill
                .behavior
                .native_damage_effect(kind)
                .is_some_and(|e| e.helper_name() == Some(skill.name))
            {
                continue;
            }
            for owner in 0..core.players.len() as PlayerId {
                let q = QueuedTrigger {
                    owner,
                    def,
                    order: (0, 0, 0, 0, def),
                };
                if trigger_order::eligible(core, catalog, &q, &res.stack, frame) {
                    helpers.push((owner, def));
                }
            }
        }
    }
    helpers.sort_unstable();
    helpers.dedup();
    for (owner, def) in helpers {
        register_added(core, res, catalog, owner, def);
    }
}
