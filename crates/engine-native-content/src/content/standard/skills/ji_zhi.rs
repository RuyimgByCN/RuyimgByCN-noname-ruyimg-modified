use super::*;

// 集智（黄月英）：使用普通锦囊时摸一张牌。
#[derive(Clone)]
pub(in super::super) struct JiZhiBehavior;

fn is_ordinary_trick(kind: crate::catalog::CardKind) -> bool {
    use crate::catalog::CardKind;
    if matches!(
        kind,
        CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Duel
            | CardKind::Chain
            | CardKind::Nullify
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Wuzhong
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::Jiedao
    ) {
        return true;
    }
    !kind.is_basic_kind()
        && !kind.is_equip()
        && !matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
        && kind.is_basic_trick_or_equip()
}

impl SkillBehavior for JiZhiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::CardUsed(u)
            if u.player == owner
                && is_ordinary_trick(u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog))))
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
