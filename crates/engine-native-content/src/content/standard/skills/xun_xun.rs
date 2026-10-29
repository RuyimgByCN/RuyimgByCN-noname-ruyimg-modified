use super::*;

// 恂恂（李典，P2/N08 标准缺失将第二批 2026-10-07）：摸牌阶段开始时
// 观看牌堆顶四张，选两张置顶（先选先摸）其余置底，再正常摸牌。
// 阶段机器在固定摸牌之后执行：先把固定摸的 2 张还回牌堆顶（恢复
// 摸牌前牌序），此后与上游观看对象一致（净效果等价）。
#[derive(Clone)]
pub(in super::super) struct XunXunBehavior;

impl SkillBehavior for XunXunBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Draw, begin: true }
            if *player == owner && ctx.state.deck.len() >= 4)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XunXunMachine {
            owner,
            top: Vec::new(),
            stage: XunXunStage::Undo,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum XunXunStage {
    /// 还回固定摸牌并观看顶四张（打开选择）。
    Undo,
    /// 恢复读应答：两张置顶（先选先摸）其余置底，然后摸 2。
    Arrange,
}

#[derive(Clone)]
pub(in super::super) struct XunXunMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) top: Vec<crate::ids::CardId>,
    pub(in super::super) stage: XunXunStage,
}

impl SkillMachine for XunXunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XunXun {
            owner: self.owner,
            stage: match self.stage {
                XunXunStage::Undo => XunXunStageSnap::Undo,
                XunXunStage::Arrange => XunXunStageSnap::Arrange,
            },
            top: self.top.iter().copied().map(|c| c as u32).collect(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XunXunStage::Undo => {
                // 还回固定摸的 2 张（手牌末尾=刚摸的牌；按弹出序压回牌堆
                // 即恢复摸牌前牌序，观星同款：牌库顶在末尾）。
                for _ in 0..2 {
                    if let Some(c) = ctx.core.player_mut(self.owner).hand.pop() {
                        ctx.core.deck.push(c);
                    }
                }
                let take = 4.min(ctx.core.deck.len());
                let new_len = ctx.core.deck.len() - take;
                self.top = ctx.core.deck.split_off(new_len);
                let options: Vec<ChoiceOption> =
                    self.top.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = XunXunStage::Arrange;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "xunxun_arrange",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: false,
                    },
                }
            }
            XunXunStage::Arrange => {
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
                // 未选置底（原序前插）；选中的置顶（先选先摸 → 逆序压栈）。
                for c in &rest {
                    ctx.core.deck.insert(0, *c);
                }
                for c in chosen.iter().rev() {
                    ctx.core.deck.push(*c);
                }
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
        }
    }
}
