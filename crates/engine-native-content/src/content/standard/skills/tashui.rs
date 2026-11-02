use super::*;

/// 踏水（Boss 甄姬，规则 v262，群雄乱入）：你使用或打出黑色牌后，将一名
/// 其他角色翻面。触发挂 CardResponded+CardUsed（extra）；filter=实体花
/// 色黑（上游 get.color(event.card) 实体同口径）。
pub(in super::super) struct TashuiBehavior;

impl SkillBehavior for TashuiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let card = match event {
            crate::event::EventData::CardResponded { card, player } => {
                if *player != owner {
                    return false;
                }
                *card
            }
            crate::event::EventData::CardUsed(u) => {
                if u.player != owner || u.cancelled {
                    return false;
                }
                let Some(card) = u.card else { return false };
                card
            }
            _ => return false,
        };
        matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TashuiMachine { owner })
    }
}

/// 踏水机器：无挂起一步机——座位序首个其他存活角色翻面（chooseTarget
/// AI 态度面→首个确定性近似登记；经 TurnOverBefore 走翻面提交，虚失同
/// 款通道；无其他存活静默）。
#[derive(Clone)]
pub(in super::super) struct TashuiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for TashuiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossTashui { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        if let Some(t) = (0..seats).find(|&t| t != self.owner && ctx.core.player(t).alive) {
            ctx.emit(EventData::TurnOverBefore {
                player: t,
                cancelled: false,
            });
        }
        Flow::Done
    }
}
