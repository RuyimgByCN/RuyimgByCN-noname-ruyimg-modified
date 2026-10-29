use super::*;

// 骁果（乐进，P2/N08 标准缺失将 2026-10-07）：其他角色的结束阶段
// 开始时，可弃一张基本手牌，对其造成 1 点伤害（登记差异：上游
// chooseToDiscard 自选，原生为确认门 + 手牌基本牌选择）。
#[derive(Clone)]
pub(in super::super) struct XiaoGuoBehavior;

impl SkillBehavior for XiaoGuoBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::End, begin: true }
        if *player != owner
            && ctx.state.player(*player).alive
            && ctx.state.player(owner).alive
            && ctx.state.player(owner).hand.iter().any(|c| {
                matches!(ctx.catalog.card_kind(*c), crate::catalog::CardKind::Sha | crate::catalog::CardKind::Shan | crate::catalog::CardKind::Tao | crate::catalog::CardKind::Jiu)
            }))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Phase { player, .. } => *player,
            _ => unreachable!(),
        };
        Box::new(XiaoGuoMachine {
            owner,
            target,
            stage: XiaoGuoStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum XiaoGuoStage {
    /// 弃牌选择（挂起 = xiaoguo_discard_basic）。
    Ask,
    /// 恢复读应答：弃置所选基本牌并对目标造成 1 点伤害。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct XiaoGuoMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) target: PlayerId,
    pub(in super::super) stage: XiaoGuoStage,
}

impl SkillMachine for XiaoGuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XiaoGuo {
            owner: self.owner,
            target: self.target,
            stage: match self.stage {
                XiaoGuoStage::Ask => XiaoGuoStageSnap::Ask,
                XiaoGuoStage::Read => XiaoGuoStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XiaoGuoStage::Ask => {
                if !ctx.core.player(self.target).alive {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            crate::catalog::CardKind::Sha
                                | crate::catalog::CardKind::Shan
                                | crate::catalog::CardKind::Tao
                                | crate::catalog::CardKind::Jiu
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = XiaoGuoStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "xiaoguo_discard_basic",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            XiaoGuoStage::Read => {
                if let ChoiceOption::Card(card) = ctx.answer_first() {
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(self.owner),
                            self.target,
                            1,
                        )));
                    }
                }
                Flow::Done
            }
        }
    }
}
