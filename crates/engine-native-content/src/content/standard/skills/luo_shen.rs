use super::*;

// 洛神（甄姬；A2 S05）：自己准备阶段开始（上游 player phaseZhunbeiBegin；
// 原锚判定阶段开始）时可进行判定——黑色则获得此判定牌并再次判定
// （循环至翻出红色；简化契约：整链一次确认，不分次放弃）。
#[derive(Clone)]
pub(in super::super) struct LuoShenBehavior;

impl SkillBehavior for LuoShenBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Prepare,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuoShenMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct LuoShenMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for LuoShenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LuoShen { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 单次判定：黑桃/梅花 → 获得此牌并再次判定（Yield 续行）；
        // 红桃/方块 → 判定牌弃置，循环结束。
        let card = ctx.draw_judge_card();
        let black = matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        if black {
            ctx.core.player_mut(self.owner).hand.push(card);
            ctx.log(LogEvent::CardsDrawn {
                player: self.owner,
                cards: vec![card],
            });
            Flow::Yield
        } else {
            ctx.to_discard(card);
            Flow::Done
        }
    }
}
