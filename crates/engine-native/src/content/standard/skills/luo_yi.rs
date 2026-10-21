use super::*;

// 裸衣（许褚）：摸牌阶段开始时，可少摸一张牌，然后本回合
// 【杀】或【决斗】造成的伤害 +1。加成旗标复用 once_per_turn 的
// `once_used`（确认时置位；下回合判定阶段开始时随
// `reset_turn_limits` 清除），伤害结算侧由 kernel `luoyi_bonus` 读取。
#[derive(Clone)]
pub(in super::super) struct LuoYiBehavior;

impl SkillBehavior for LuoYiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuoYiMachine {
            owner,
            asked: false,
        })
    }
}

#[derive(Clone)]
pub(in super::super) struct LuoYiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) asked: bool,
}

impl SkillMachine for LuoYiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LuoYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 少摸一张：弃置本阶段固定摸到的最后一张（手牌末尾）
            let dropped = ctx.core.player_mut(self.owner).hand.pop();
            if let Some(c) = dropped {
                ctx.to_discard(c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![c],
                });
            }
        }
        Flow::Done
    }
}
