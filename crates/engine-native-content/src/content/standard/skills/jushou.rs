use super::*;

// ---------------------------------------------------------------------------
// 曹仁「据守」：结束阶段，你可以翻面并摸一张牌。
// 登记差异：翻面被【漩涡面具】等取消时摸牌仍执行（效果独立）。
// ---------------------------------------------------------------------------

pub(in super::super) struct JushouBehavior;

impl SkillBehavior for JushouBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::End,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JushouMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct JushouMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for JushouMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShouYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 确认门（OptionalGate）通过后执行：翻面经 TurnOverBefore 子事件
        // （面具等可取消）；摸牌独立执行（登记差异：取消翻面不影响摸牌）。
        ctx.emit(EventData::TurnOverBefore {
            player: self.owner,
            cancelled: false,
        });
        ctx.start_draw(self.owner, 1);
        Flow::Done
    }
}
