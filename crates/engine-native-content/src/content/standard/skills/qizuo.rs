use super::*;

/// 奇佐（Boss 郭嘉，规则 v259，群雄乱入）：你使用锦囊牌结算结束后，你
/// 使用一张同名牌。触发挂 CardUsed AfterTriggers 窗口（上游 useCardAfter
/// →前置近似：副本效果先于原牌效果，登记）；无懈不可复制；铁索不复制
///（上游 AI check 排除同口径）；副本载体槽守卫递归（上游 event.parent
/// name 守卫同口径）。
pub(in super::super) struct QizuoBehavior;

/// 奇佐可复制的锦囊种（标准锦囊集；G1 扩展锦囊不入 boss 场）：
/// 延时锦囊含判定区同名去重门（上游 canUse 同口径）。
fn qizuo_copyable(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::SavageAttack
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

impl SkillBehavior for QizuoBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let crate::event::EventData::CardUsed(u) = event else {
            return false;
        };
        if u.cancelled || u.player != owner {
            return false;
        }
        // 副本载体自身的使用不再触发（上游 event.parent name 守卫同口径）
        if ctx.state.turn.qizuo_copy == Some(u.card) {
            return false;
        }
        let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
        if !qizuo_copyable(kind) {
            return false;
        }
        // 延时锦囊：目标判定区无同名（上游 canUse 同口径）
        if matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian) {
            for t in &u.targets {
                if ctx
                    .state
                    .player(*t)
                    .judge
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == kind)
                {
                    return false;
                }
            }
        }
        // 原目标全体存活（上游 isIn 同口径）
        u.targets.iter().all(|t| ctx.state.player(*t).alive)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let crate::event::EventData::CardUsed(u) = event else {
            unreachable!("qizuo trigger event");
        };
        Box::new(QizuoMachine {
            owner,
            targets: u.targets.clone(),
        })
    }
}

/// 奇佐机器：无挂起一步机——以牌堆底实体为载体按原牌种复用（use_
/// virtual_card 完整管线），载体实体入副本槽供其 AfterTriggers 过滤。
#[derive(Clone)]
pub(in super::super) struct QizuoMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) targets: Vec<PlayerId>,
}

impl SkillMachine for QizuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossQizuo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 牌堆空无载体（上游凭空造牌不依赖实体，登记）；过滤后目标离场
        //（同窗口其他触发先行结算所致）→ 不发动
        if ctx.core.deck.is_empty() || !self.targets.iter().all(|t| ctx.core.player(*t).alive) {
            return Flow::Done;
        }
        let u = ctx
            .current_card_use()
            .expect("qizuo machine: card use frame");
        let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
        let carrier = ctx.use_virtual_card(self.owner, kind, self.targets.clone());
        ctx.core.turn.qizuo_copy = Some(carrier);
        Flow::Done
    }
}
