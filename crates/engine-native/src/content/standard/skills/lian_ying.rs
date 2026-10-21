use super::*;

// 连营（陆逊）：失去最后一张手牌后，可摸一张牌。触发覆盖出牌
// （CardUsed）、响应（CardResponded）、给出（CardsMoved）三条路径；
// 弃牌阶段的批量弃置不产生事件，暂不触发（挂账覆盖清单）。
#[derive(Clone)]
pub(in super::super) struct LianYingBehavior;

impl SkillBehavior for LianYingBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        if ctx.state.player(owner).hand.len() > 1 {
            return false;
        }
        match event {
            EventData::CardUsed(u) => u.player == owner,
            EventData::CardResponded { player, .. } => *player == owner,
            EventData::CardsMoved { from, .. } => *from == owner,
            EventData::CardsDiscarded { player, .. } => *player == owner,
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LianYingMachine {
            owner,
            asked: false,
        })
    }
}

#[derive(Clone)]
pub(in super::super) struct LianYingMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) asked: bool,
}

impl SkillMachine for LianYingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LianYing { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            if ctx.core.player(self.owner).alive && ctx.core.player(self.owner).hand.is_empty() {
                ctx.draw_cards(self.owner, 1);
            }
        }
        Flow::Done
    }
}
