use super::*;

// 遗计（郭嘉）：受到 1 点伤害后，摸两张牌，然后可将至多两张手牌交给
// 一至两名其他玩家（每张给牌均可放弃）。
#[derive(Clone)]
pub(in super::super) struct JiJiBehavior;

impl SkillBehavior for JiJiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiJiMachine {
            owner,
            stage: JiJiStage::Draw,
            target: None,
            given: 0,
        })
    }
}

#[derive(Clone)]
pub(in super::super) enum JiJiStage {
    /// 摸两张（子事件结算后继续）。
    Draw,
    /// 挂起询问 = jiji_target（恢复读目标）。
    Target,
    /// 给牌后回到目标询问前的中转（行为同 Target：重新询问目标）。
    /// 不再构造，但保留变体：v2 字节的 jiji 挂起可能携带 Retarget=2，
    /// 恢复时映射回 Target；删除会破坏已落盘字节。
    #[allow(dead_code)]
    Retarget,
    /// 挂起询问 = jiji_card（恢复读牌并给出）。
    GiveCard,
    /// 挂起询问 = jiji_card 的给牌（恢复执行给出）。
    GiveChooseCard,
}

#[derive(Clone)]
pub(in super::super) struct JiJiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: JiJiStage,
    pub(in super::super) target: Option<PlayerId>,
    pub(in super::super) given: u8,
}

impl SkillMachine for JiJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiJi {
            owner: self.owner,
            stage: match self.stage {
                JiJiStage::Draw => JiJiStageSnap::Draw,
                JiJiStage::Target => JiJiStageSnap::Target,
                JiJiStage::Retarget => JiJiStageSnap::Retarget,
                JiJiStage::GiveCard => JiJiStageSnap::GiveCard,
                JiJiStage::GiveChooseCard => JiJiStageSnap::GiveChooseCard,
            },
            target: self.target,
            given: self.given,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiJiStage::Draw => {
                ctx.start_draw(self.owner, 2);
                self.stage = JiJiStage::Target;
                Flow::Yield
            }
            JiJiStage::Target | JiJiStage::Retarget => {
                // 至多两张；无其他存活玩家或手牌已空则结束
                if self.given >= 2 || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() {
                    return Flow::Done;
                }
                // 挂起询问 = jiji_target；恢复由 GiveCard 分支读取目标
                self.stage = JiJiStage::GiveCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "jiji_target",
                        options: others,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JiJiStage::GiveCard => {
                // 恢复：jiji_target 的应答（Player 目标 / 空应答弃权）
                let to = match ctx.answer().first() {
                    Some(ChoiceOption::Player(p)) => *p,
                    _ => return Flow::Done,
                };
                self.target = Some(to);
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
                // 挂起询问 = jiji_card；恢复由 GiveChooseCard 分支读牌并给出
                self.stage = JiJiStage::GiveChooseCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "jiji_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JiJiStage::GiveChooseCard => {
                // 恢复：jiji_card 的应答——Card 则给出并计入；空应答则放弃并结束
                if let Some(&ChoiceOption::Card(c)) = ctx.answer().first() {
                    let Some(to) = self.target else {
                        return Flow::Done;
                    };
                    ctx.give_hand_card(self.owner, to, c);
                    self.given += 1;
                } else {
                    return Flow::Done;
                }
                if self.given >= 2 || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                // 回到目标询问（第二轮，至多两张）
                self.stage = JiJiStage::Target;
                Flow::Yield
            }
        }
    }
}
