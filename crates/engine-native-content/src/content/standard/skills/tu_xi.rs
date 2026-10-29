use super::*;

// 突袭（张辽）：摸牌阶段开始时，可放弃摸牌，改为获得至多两名
// 其他角色的各一张手牌（各区首张，Dismantle 同款确定性取牌；
// 目标询问弃权 = 不发动，保留摸牌）。
#[derive(Clone)]
pub(in super::super) struct TuXiBehavior;
impl SkillBehavior for TuXiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TuXiMachine {
            owner,
            stage: TuXiStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum TuXiStage {
    /// 挂起询问 = tuxi_target（选择 1–2 名有手牌的其他角色）。
    Ask,
    /// 恢复读应答：弃固定摸牌并取牌。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct TuXiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: TuXiStage,
}

impl SkillMachine for TuXiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::TuXi {
            owner: self.owner,
            stage: match self.stage {
                TuXiStage::Ask => TuXiStageSnap::Ask,
                TuXiStage::Read => TuXiStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            TuXiStage::Ask => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = TuXiStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "tuxi_target",
                        options: candidates,
                        min: 1,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            TuXiStage::Read => {
                let targets: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if targets.is_empty() {
                    return Flow::Done; // 弃权：保留本阶段摸牌
                }
                // 放弃摸牌：弃置本阶段固定摸到的牌（手牌末尾至多 2 张，
                // 触发机器在阶段固定摸牌之后执行，末尾即刚摸的牌）
                let mut dropped = Vec::new();
                {
                    let p = ctx.core.player_mut(self.owner);
                    for _ in 0..2 {
                        if let Some(c) = p.hand.pop() {
                            dropped.push(c);
                        }
                    }
                }
                if !dropped.is_empty() {
                    for c in &dropped {
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: dropped,
                    });
                }
                let mut gained = Vec::new();
                for t in targets {
                    if t == self.owner || !ctx.core.player(t).alive {
                        continue;
                    }
                    if let Some(c) = ctx.core.player(t).hand.first().copied() {
                        ctx.give_hand_card(t, self.owner, c);
                        gained.push(c);
                    }
                }
                if !gained.is_empty() {
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: gained,
                    });
                }
                Flow::Done
            }
        }
    }
}
