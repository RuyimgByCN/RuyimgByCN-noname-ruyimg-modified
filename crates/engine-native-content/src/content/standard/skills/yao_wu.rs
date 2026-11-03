use super::*;

// 耀武（华雄，P2/N08 标准缺失将 2026-10-07）：锁定技——你受到红色
// 【杀】的伤害时，伤害来源须选择摸一张牌或回复 1 点体力（满血时无
// 回复选项；登记差异：上游 chooseDrawRecover 由 AI/玩家确认，原生为
// 强制二选）。
#[derive(Clone)]
pub(in super::super) struct YaoWuBehavior;

impl SkillBehavior for YaoWuBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.source.is_some()
                && d.source != Some(owner)
                && ctx.state.player(d.source.unwrap()).alive
                && d.card.map(|c| {
                    let def = &ctx.catalog.cards[c as usize];
                    matches!(ctx.catalog.card_kind(c), crate::catalog::CardKind::Sha | crate::catalog::CardKind::FireSha | crate::catalog::CardKind::LeiSha)
                        && matches!(def.suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
                }).unwrap_or(false))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source.expect("filtered"),
            _ => unreachable!(),
        };
        Box::new(YaoWuMachine {
            owner,
            source,
            stage: YaoWuStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum YaoWuStage {
    /// 来源二选（挂起 = yaowu_draw_or_recover）。
    Ask,
    /// 恢复读应答：摸一张或回复 1 点体力。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct YaoWuMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) source: PlayerId,
    pub(in super::super) stage: YaoWuStage,
}

impl SkillMachine for YaoWuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YaoWu {
            owner: self.owner,
            source: self.source,
            stage: match self.stage {
                YaoWuStage::Ask => YaoWuStageSnap::Ask,
                YaoWuStage::Read => YaoWuStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            YaoWuStage::Ask => {
                let p = ctx.core.player(self.source);
                let mut options = vec![ChoiceOption::Tag(0)];
                if (p.hp as u32) < p.max_hp {
                    options.push(ChoiceOption::Tag(1));
                }
                self.stage = YaoWuStage::Read;
                Flow::WaitChoice {
                    observer: self.source,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::PickTag,
                        prompt: "yaowu_draw_or_recover",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            YaoWuStage::Read => {
                match ctx.answer_first() {
                    ChoiceOption::Tag(1) => {
                        ctx.recover(self.source, 1, None);
                    }
                    _ => {
                        ctx.start_draw(self.source, 1);
                    }
                }
                Flow::Done
            }
        }
    }
}
