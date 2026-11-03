use super::*;

// 集智（黄月英）：使用普通锦囊时摸一张牌。
#[derive(Clone)]
pub(in super::super) struct JiZhiBehavior;

impl SkillBehavior for JiZhiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::CardUsed(u)
        if u.player == owner
            && matches!(
                u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)),
                crate::catalog::CardKind::SavageAttack
                    | crate::catalog::CardKind::ArcheryAttack
                    | crate::catalog::CardKind::Duel
                    | crate::catalog::CardKind::Chain
                    | crate::catalog::CardKind::Nullify
            ))
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiZhiMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct JiZhiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for JiZhiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiZhi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.start_draw(self.owner, 1);
        Flow::Done
    }
}
