use super::*;

/// 枭雄（Boss 刘备，规则 v261，群雄乱入）：其他角色使用基本牌或锦囊牌
/// 结算结束后，你获得该牌的副本。触发挂 CardUsed AfterTriggers 窗口
///（上游 global useCardAfter 同口径；副本入手不 emit CardUsed=无递归）。
pub(in super::super) struct XiaoxiongBehavior;

/// 枭雄可复制的牌种：基本牌+锦囊（含延时——上游 get.type(card,"trick")
/// 延时算 trick；装备排除）。
fn xiaoxiong_copyable(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Sha
            | CardKind::Shan
            | CardKind::Tao
            | CardKind::Jiu
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::ShouLiJian
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Duel
            | CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Wuzhong
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::Jiedao
    )
}

impl SkillBehavior for XiaoxiongBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let crate::event::EventData::CardUsed(u) = event else {
            return false;
        };
        if u.cancelled || u.player == owner {
            return false;
        }
        let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
        xiaoxiong_copyable(kind)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiaoxiongMachine { owner })
    }
}

/// 枭雄机器：无挂起一步机——牌堆底实体入手（上游 createCard 凭空造牌→
/// 载体实体入手登记差异：花色点数随载体+消耗库顶实体；牌堆空静默）。
#[derive(Clone)]
pub(in super::super) struct XiaoxiongMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for XiaoxiongMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXiaoxiong { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(carrier) = ctx.core.deck.first().copied() else {
            return Flow::Done;
        };
        ctx.core.deck.remove(0);
        ctx.core.player_mut(self.owner).hand.push(carrier);
        ctx.log(LogEvent::CardsGiven {
            from: self.owner,
            to: self.owner,
            cards: vec![carrier],
        });
        Flow::Done
    }
}
