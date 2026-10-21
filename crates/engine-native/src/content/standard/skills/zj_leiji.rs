use super::*;

/// 雷击·张角（Boss 张角，规则 v263，群雄乱入）：你使用或打出【闪】后，
/// 一名其他角色判定：黑则其受到 1 点雷伤且你摸一张牌。触发挂 CardResponded
/// +CardUsed（extra）；chooseTarget AI 态度面→座位序首个确定性近似。
pub(in super::super) struct ZjLeijiBehavior;

impl SkillBehavior for ZjLeijiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        match event {
            crate::event::EventData::CardResponded { card, player } => {
                *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
            }
            crate::event::EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card)) == CardKind::Shan
            }
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZjLeijiMachine { owner })
    }
}

/// 雷击·张角机器：无挂起一步机——座位序首个其他存活角色判定（内联确定
/// 性口径=铁骑先例，无改判窗）：黑则 1 雷伤+持有者摸 1；红色无伤。
#[derive(Clone)]
pub(in super::super) struct ZjLeijiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for ZjLeijiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZjLeiji { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        if let Some(t) = (0..seats).find(|&t| t != self.owner && ctx.core.player(t).alive) {
            let judged = ctx.draw_judge_card();
            let black = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            );
            ctx.to_discard(judged);
            if black {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    1,
                    crate::event::DamageNature::Thunder,
                    None,
                )));
                ctx.draw_cards(self.owner, 1);
            }
        }
        Flow::Done
    }
}
