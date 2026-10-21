use super::*;

// 机捷（伊籍，P2/N08 标准缺失将第四批 2026-10-07）：出牌阶段限一次
// 主动技——取牌堆底一张牌交给一名任意存活角色（上游 chooseTarget 无
// 过滤器，AI 含自引加成，原生同允许自选）。
#[derive(Clone)]
pub(in super::super) struct JiJieBehavior;

impl SkillBehavior for JiJieBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiJieMachine {
            owner,
            card: None,
            stage: JiJieStage::Ask,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        if !ctx.state.player(owner).alive || ctx.state.deck.is_empty() {
            return None; // 无牌可授
        }
        Some(Box::new(JiJieMachine {
            owner,
            card: None,
            stage: JiJieStage::Ask,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum JiJieStage {
    /// 目标选择（挂起 = jijie_give_target；牌堆底牌已取入手上）。
    Ask,
    /// 恢复读应答：交牌。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct JiJieMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) card: Option<crate::ids::CardId>,
    pub(in super::super) stage: JiJieStage,
}

impl SkillMachine for JiJieMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiJie {
            owner: self.owner,
            card: self.card,
            stage: match self.stage {
                JiJieStage::Ask => JiJieStageSnap::Ask,
                JiJieStage::Read => JiJieStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiJieStage::Ask => {
                let candidates: Vec<ChoiceOption> =
                    ctx.core.alive_players().map(ChoiceOption::Player).collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                // 取牌堆底（deck 首位）。
                self.card = Some(ctx.core.deck.remove(0));
                self.stage = JiJieStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectPlayers,
                        prompt: "jijie_give_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            JiJieStage::Read => {
                if let (ChoiceOption::Player(target), Some(card)) = (ctx.answer_first(), self.card)
                {
                    if ctx.core.player(target).alive {
                        ctx.core.player_mut(target).hand.push(card);
                        ctx.out.push(crate::LogEvent::CardsDrawn {
                            player: target,
                            cards: vec![card],
                        });
                    } else {
                        ctx.to_discard(card);
                    }
                }
                self.card = None;
                Flow::Done
            }
        }
    }
}
