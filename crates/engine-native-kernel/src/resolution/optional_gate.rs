//! 可选触发的确认闸门（自 resolution.rs 拆分）：确认后启动对应技能机器并透明委托步进。

use super::{native_machine, trigger_order};
use crate::catalog::SkillMachine;
use crate::choice::ChoiceOption;
use crate::event::QueuedTrigger;
use crate::ids::{PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow};

/// 可选触发的确认闸门：确认后才启动真正的技能机器。
pub struct OptionalGate {
    pub(super) owner: PlayerId,
    pub(super) def: SkillDefId,
    /// 计划时的事件载荷（触发语义以进入时为准）。
    pub(super) event: crate::event::EventData,
    pub(super) asked: bool,
    pub(super) inner: Option<Box<dyn SkillMachine>>,
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
    fn actor_owner(&self) -> Option<PlayerId> {
        Some(self.owner)
    }

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
        let actual_ev = crate::event::damage_event(&ctx.res.stack, frame);
        let observer = ctx.catalog.skills[self.def as usize]
            .behavior
            .confirmation_actor(self.owner, actual_ev, kind);
        self.event = if observer != self.owner {
            actual_ev.clone()
        } else {
            frame.data.clone()
        };
        if !self.asked {
            self.asked = true;
            return Flow::WaitChoice {
                observer,
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
            ctx.log_skill_triggered(def.name, self.owner, self.def);
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
