use super::*;

// 刚烈（夏侯惇）：受到伤害后，可进行判定：若判定牌不为红桃，
// 伤害来源受到你造成的 1 点伤害（递归由确认闸门天然限流）。
#[derive(Clone)]
pub(in super::super) struct GangLieBehavior;

impl SkillBehavior for GangLieBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0
                && d.source.is_some() && d.source != Some(owner))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source.expect("filtered"),
            _ => unreachable!(),
        };
        Box::new(GangLieMachine {
            owner,
            source,
            asked: false,
        })
    }
}

#[derive(Clone)]
pub(in super::super) struct GangLieMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) source: PlayerId,
    pub(in super::super) asked: bool,
}

impl SkillMachine for GangLieMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::GangLie {
            owner: self.owner,
            source: self.source,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            let card = ctx.draw_judge_card();
            let heart = ctx.catalog.cards[card as usize].suit.is_heart();
            ctx.to_discard(card);
            if !heart && ctx.core.player(self.source).alive {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(self.owner),
                    self.source,
                    1,
                )));
            }
        }
        Flow::Done
    }
}
