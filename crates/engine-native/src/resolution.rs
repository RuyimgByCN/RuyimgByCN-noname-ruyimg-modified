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

/// 挂起的机器与其归属槽位。
/// 无帧哨兵（N11 主动技等会话级挂起）：槽不关联任何栈帧时使用，
/// 恢复校验对其豁免帧存在性检查（真实帧 id 从事件号单调递增，不会撞上）。
pub const NO_FRAME: u32 = u32::MAX;

pub struct Park {
    /// 栈深回到该值时恢复。
    pub resume_depth: usize,
    pub slot: Slot,
    pub machine: Box<dyn SkillMachine>,
}

impl Clone for Park {
    fn clone(&self) -> Self {
        Self {
            resume_depth: self.resume_depth,
            slot: self.slot,
            machine: self.machine.fork(),
        }
    }
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
    let stage = match kind {
        EventKind::Damage => Stage::BeforeTriggers,
        // E03 翻面：前置取消窗口（漩涡面具等锁定技在 Commit 前派发）
        EventKind::TurnOverBefore => Stage::BeforeTriggers,
        // 装备前置窗口（swd 批 143 改造，规则 v318）：BeforeTriggers 派发
        // useCardToBegin 类技（改造改写装备栏后再入槽）
        EventKind::Equipped => Stage::BeforeTriggers,
        // 死亡前置窗口（规则 v170）：BeforeTriggers 收集 dieBegin 类遗
        // 言/拦截技（changan 氏），Intrinsic 仍为 DeathMachine。
        EventKind::Death => Stage::BeforeTriggers,
        _ => Stage::Commit,
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
        (EventKind::PhaseBegin, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::PhaseBegin, Stage::Intrinsic) => Stage::Done,
        (EventKind::PhaseEnd, Stage::AfterTriggers) => Stage::Intrinsic,
        (EventKind::PhaseEnd, Stage::Intrinsic) => Stage::Done,
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
        Self { owner, def, event, asked, inner }
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
            self.inner = Some(def.behavior.start(self.owner, &self.event));
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
fn plan_triggers(core: &mut GameStateCore, res: &mut Resolution, catalog: &crate::catalog::Catalog, frame_id: EventId, before: bool) {
    let (event, kind) = {
        let f = res.frame(frame_id).expect("frame");
        // before 阶段：伤害帧查 DamageBefore（伤害前置窗口）；翻面帧查
        // TurnOverBefore（E03 翻面前置取消窗口，漩涡面具等锁定技）；
        // 其余帧无前置窗口。
        // 翻面帧的 after 窗口静默：目录无「翻面后」触发种类，且 after
        // 再查 TurnOverBefore 会让取消技重复入队重复写回。
        if !before
            && (matches!(f.data, EventData::TurnOverBefore { .. })
                || matches!(f.data, EventData::Death { .. }))
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
            _ => EventKind::DamageBefore,
        };
        (f.data.clone(), if before { before_kind } else { f.kind })
    };
    let mut entries: Vec<QueuedTrigger> = Vec::new();
    let turn_current = core.turn.current;
    // PA01 活跃触发索引：候选遍历的代价与本局是否持有无关（目录 6215 技能
    // 档曾令 8 座提交 p50 ×3.3），改为从实例侧单遍出发，直接比对定义的
    // 触发元数据——dispatch 列表即由 event/extra_events 构建，成员关系
    // 严格等价。入队集合不变；最终顺序由下方含 def_id 的全序键决定，
    // 与遍历序无关。
    for ((owner, def_id), inst) in core.skills.range(..) {
        if !inst.enabled || !core.player(*owner).alive {
            continue;
        }
        let def = &catalog.skills[*def_id as usize];
        let Some(meta) = def.trigger.as_ref() else { continue };
        if meta.event != kind && !meta.extra_events.contains(&kind) {
            continue;
        }
        let qctx = QueryCtx { state: core, catalog };
        if !def.behavior.filter(&qctx, *owner, &event) {
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
                -meta.priority,
                u8::from(meta.optional),
                seat_rel,
                *def_id,
            ),
        });
    }
    entries.sort_by_key(|e| e.order);
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

        // A) 恢复深度匹配的挂起机器
        if res.open_choice.is_none() {
            if let Some(idx) = res
                .parks
                .iter()
                .position(|p| p.resume_depth == res.stack.len())
            {
                let mut park = res.parks.remove(idx);
                let flow = {
                    let mut ctx = Ctx { core, res, catalog, out };
                    park.machine.step(&mut ctx)
                };
                res.answer = None; // 应答只消费一次
                match flow {
                    Flow::Done => finish_slot(res, park.slot),
                    Flow::Yield => res.parks.push(park),
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
                        res.parks.push(park);
                        return StepOutcome::Pending;
                    }
                }
                continue;
            }
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
                    continue;
                }
                if queue_len > 0 {
                    run_queued_trigger(core, res, catalog, out, top);
                    continue;
                }
                let kind = res.frame(top).expect("frame").kind;
                let next = next_stage(kind, stage);
                let f = res.frame_mut(top).expect("frame");
                f.stage = next;
                f.planned = false;
            }
            Stage::Commit => {
                {
                    let mut ctx = Ctx { core, res, catalog, out };
                    kernel::commit_frame(&mut ctx, top);
                }
                let kind = res.frame(top).expect("frame").kind;
                let next = next_stage(kind, Stage::Commit);
                let f = res.frame_mut(top).expect("frame");
                f.stage = next;
                f.planned = false;
            }
            Stage::Intrinsic => {
                if intrinsic_done {
                    let kind = res.frame(top).expect("frame").kind;
                    let next = next_stage(kind, Stage::Intrinsic);
                    res.frame_mut(top).expect("frame").stage = next;
                    continue;
                }
                let machine = {
                    let mut ctx = Ctx { core, res, catalog, out };
                    kernel::intrinsic_machine(&mut ctx, top)
                };
                match machine {
                    Some(m) => res.parks.push(Park {
                        resume_depth: res.stack.len(),
                        slot: Slot::Intrinsic { frame: top },
                        machine: m,
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

/// 弹出并运行队列中的下一个触发（运行前重查实例有效性与过滤器）。
fn run_queued_trigger(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
    frame_id: EventId,
) {
    let qt = {
        let f = res.frame_mut(frame_id).expect("frame");
        f.queue.remove(0)
    };
    let instance_ok = core
        .skills
        .get(&(qt.owner, qt.def))
        .map(|i| i.enabled)
        .unwrap_or(false)
        && core.player(qt.owner).alive;
    if !instance_ok {
        return; // 结算中途被禁用/死亡的技能不再生效
    }
    let def = &catalog.skills[qt.def as usize];
    let Some(meta) = def.trigger.as_ref() else { return };
    let event = res.frame(frame_id).expect("frame").data.clone();
    let qctx = QueryCtx { state: core, catalog };
    if !def.behavior.filter(&qctx, qt.owner, &event) {
        return;
    }
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
        out.push(LogEvent::SkillTriggered {
            skill: def.name,
            owner: qt.owner,
        });
        def.behavior.start(qt.owner, &event)
    };
    res.parks.push(Park {
        resume_depth: res.stack.len(),
        slot: Slot::Trigger { frame: frame_id },
        machine,
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
