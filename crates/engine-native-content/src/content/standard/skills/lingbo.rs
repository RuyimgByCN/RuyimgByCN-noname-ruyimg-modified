use super::*;

/// 凌波（Boss 甄姬，规则 v262，群雄乱入）：你使用或打出【闪】后摸两张。
/// 触发挂 CardResponded+CardUsed（extra）；filter=声明/实体牌种为闪。
pub(in super::super) struct LingboBehavior;

impl SkillBehavior for LingboBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        match event {
            crate::event::EventData::CardResponded { card, player } => {
                *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
            }
            crate::event::EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)) == CardKind::Shan
            }
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LingboMachine { owner })
    }
}

/// 凌波机器：无挂起一步机——摸 2。
#[derive(Clone)]
pub(in super::super) struct LingboMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for LingboMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLingbo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.start_draw(self.owner, 2);
        Flow::Done
    }
}
