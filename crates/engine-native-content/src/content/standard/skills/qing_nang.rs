use super::*;

// 青囊（华佗）：出牌阶段限一次，弃一张手牌并令一名已受伤的其他角色回复
// 1 点体力（Target → Discard 两阶段主动技）。
#[derive(Clone)]
pub(in super::super) struct QingNangBehavior;

impl SkillBehavior for QingNangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(QingNangMachine {
            owner,
            stage: QingNangStage::TargetAsk,
            healed: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        let wounded =
            ctx.state.players.iter().enumerate().any(|(i, p)| {
                p.alive && (p.hp as i64) < (p.max_hp as i64) && i as PlayerId != owner
            });
        if !wounded {
            return None;
        }
        Some(Box::new(QingNangMachine {
            owner,
            stage: QingNangStage::TargetAsk,
            healed: None,
        }))
    }
}

#[derive(Clone)]
pub(in super::super) enum QingNangStage {
    /// 挂起询问 = qingnang_target（恢复读目标）。
    TargetAsk,
    /// 挂起询问 = qingnang_card（恢复读牌并治疗）。
    TargetRead,
    /// 挂起询问 = qingnang_card（恢复读牌并给出）。
    CardRead,
}

#[derive(Clone)]
pub(in super::super) struct QingNangMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: QingNangStage,
    pub(in super::super) healed: Option<PlayerId>,
}

impl SkillMachine for QingNangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::QingNang {
            owner: self.owner,
            stage: match self.stage {
                QingNangStage::TargetAsk => QingNangStageSnap::TargetAsk,
                QingNangStage::TargetRead => QingNangStageSnap::TargetRead,
                QingNangStage::CardRead => QingNangStageSnap::CardRead,
            },
            healed: self.healed,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            QingNangStage::TargetAsk => {
                // 构建 qingnang_target 询问（已受伤的其他角色）
                let wounded: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner && {
                            let pl = ctx.core.player(*p);
                            pl.alive && (pl.hp as i64) < (pl.max_hp as i64)
                        }
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if wounded.is_empty() {
                    return Flow::Done;
                }
                self.stage = QingNangStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "qingnang_target",
                        options: wounded,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            QingNangStage::TargetRead => {
                // 恢复：qingnang_target 的应答（Player 目标 / 空应答弃权）
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.healed = Some(target);
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = QingNangStage::CardRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qingnang_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            QingNangStage::CardRead => {
                // 恢复：qingnang_card 的应答——Card 则弃置并治疗
                if let Some(&ChoiceOption::Card(c)) = ctx.answer().first() {
                    ctx.discard_from_hand(self.owner, c);
                    if let Some(target) = self.healed {
                        ctx.recover(target, 1, None);
                    }
                }
                Flow::Done
            }
        }
    }
}
