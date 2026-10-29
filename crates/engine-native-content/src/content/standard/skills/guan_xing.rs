use super::*;

// 观星（诸葛亮）：回合开始（判定阶段占位）时，可观看牌库顶 X 张
// （X = 存活角色数且至多 5）：所选牌按应答顺序置于牌库顶（先选先摸），
// 未选牌按原序置于牌库底。
#[derive(Clone)]
pub(in super::super) struct GuanXingBehavior;

impl SkillBehavior for GuanXingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(GuanXingMachine {
            owner,
            stage: GuanXingStage::Arrange,
            top: Vec::new(),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum GuanXingStage {
    /// 挂起询问 = guanxing_arrange（从已展示的顶牌中选任意张）。
    Arrange,
    /// 恢复读应答并落位。
    Place,
}

#[derive(Clone)]
pub(in super::super) struct GuanXingMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: GuanXingStage,
    /// Arrange 阶段取出的牌库顶牌（原序，顶部在末尾）。
    pub(in super::super) top: Vec<CardId>,
}

impl SkillMachine for GuanXingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::GuanXing {
            owner: self.owner,
            stage: match self.stage {
                GuanXingStage::Arrange => GuanXingStageSnap::Arrange,
                GuanXingStage::Place => GuanXingStageSnap::Place,
            },
            top: self.top.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            GuanXingStage::Arrange => {
                let x = ctx.core.alive_count().min(5);
                let take = x.min(ctx.core.deck.len());
                if take == 0 {
                    return Flow::Done;
                }
                let new_len = ctx.core.deck.len() - take;
                self.top = ctx.core.deck.split_off(new_len);
                let options: Vec<ChoiceOption> =
                    self.top.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = GuanXingStage::Place;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "guanxing_arrange",
                        options,
                        min: 0,
                        max: take,
                        allow_cancel: false,
                    },
                }
            }
            GuanXingStage::Place => {
                // 应答中的牌 = 置顶（按应答顺序，先选先摸：牌库顶在末尾，
                // 故逆序压栈）；未选的按原序置于牌库底（前插）。
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let rest: Vec<CardId> = self
                    .top
                    .iter()
                    .copied()
                    .filter(|c| !chosen.contains(c))
                    .collect();
                for c in &rest {
                    ctx.core.deck.insert(0, *c);
                }
                for c in chosen.iter().rev() {
                    ctx.core.deck.push(*c);
                }
                Flow::Done
            }
        }
    }
}
