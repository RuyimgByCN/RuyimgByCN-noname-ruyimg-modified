use super::*;

// 奸雄（曹操）：受到伤害后，可获得造成此伤害的牌（牌已进弃牌堆）。
#[derive(Clone)]
pub(in super::super) struct JianXiongBehavior;

impl SkillBehavior for JianXiongBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0 && d.card.is_some())
            && d_card_in_discard(ctx, event)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let card = match event {
            EventData::Damage(d) => d.card,
            _ => None,
        };
        Box::new(JianXiongMachine {
            owner,
            card: card.expect("filtered"),
            asked: false,
        })
    }
}

fn d_card_in_discard(ctx: &QueryCtx<'_>, event: &EventData) -> bool {
    let Some(card) = (match event {
        EventData::Damage(d) => d.card,
        _ => None,
    }) else {
        return false;
    };
    ctx.state.discard.contains(&card)
}

#[derive(Clone)]
pub(in super::super) struct JianXiongMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) card: CardId,
    pub(in super::super) asked: bool,
}

impl SkillMachine for JianXiongMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JianXiong {
            owner: self.owner,
            card: self.card,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // OptionalGate 已确认；牌仍在弃牌堆时收入手牌（竞态防御：不在则跳过）
        if !self.asked {
            self.asked = true;
            if let Some(pos) = ctx.core.discard.iter().position(|c| *c == self.card) {
                ctx.core.discard.remove(pos);
                ctx.core.player_mut(self.owner).hand.push(self.card);
            }
        }
        Flow::Done
    }
}
