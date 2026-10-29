use super::*;

// 制衡（孙权）：出牌阶段主动技——弃置任意张手牌，然后摸等量的牌
// （每回合限一次；空选择 = 不发动效果直接结束）。
#[derive(Clone)]
pub(in super::super) struct ZhiHengBehavior;

impl SkillBehavior for ZhiHengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhiHengMachine {
            owner,
            stage: ZhiHengStage::Select,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None; // 无手牌可弃：无法发动
        }
        Some(Box::new(ZhiHengMachine {
            owner,
            stage: ZhiHengStage::Select,
        }))
    }
}

#[derive(Clone)]
pub(in super::super) enum ZhiHengStage {
    /// 询问弃置哪些手牌（空应答 = 放弃发动）。
    Select,
    /// 弃置后摸等量（同一次提交内完成，无需挂起）。
    Draw,
}

#[derive(Clone)]
pub(in super::super) struct ZhiHengMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: ZhiHengStage,
}

impl SkillMachine for ZhiHengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ZhiHeng {
            owner: self.owner,
            stage: match self.stage {
                ZhiHengStage::Select => ZhiHengStageSnap::Select,
                ZhiHengStage::Draw => ZhiHengStageSnap::Draw,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ZhiHengStage::Select => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                let max = options.len();
                // 挂起询问 = zhiheng_select；恢复由 Draw 分支执行弃置与摸牌
                self.stage = ZhiHengStage::Draw;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "zhiheng_select",
                        options,
                        min: 0,
                        max,
                        allow_cancel: true,
                    },
                }
            }
            ZhiHengStage::Draw => {
                let answer = ctx.answer().clone();
                let n = answer.len();
                for c in answer {
                    if let ChoiceOption::Card(card) = c {
                        ctx.discard_from_hand(self.owner, card);
                    }
                }
                if n > 0 {
                    ctx.draw_cards(self.owner, n);
                }
                Flow::Done
            }
        }
    }
}
