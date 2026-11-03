use super::*;

// 神智（甘夫人，P2/N08 标准缺失将第二批 2026-10-07；A2 S09）：自己
// 准备阶段开始（上游 player phaseZhunbeiBegin；原差异锚＝判定阶段开
// 始，R05 后准备阶段真实存在、过滤收紧 Phase::Prepare）可弃置全部手
// 牌，若数量不少于体力值则回复 1 点体力。
#[derive(Clone)]
pub(in super::super) struct ShenZhiBehavior;

impl SkillBehavior for ShenZhiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true }
            if *player == owner && !ctx.state.player(owner).hand.is_empty())
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShenZhiMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct ShenZhiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for ShenZhiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShenZhi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let hand: Vec<crate::ids::CardId> = ctx.core.player(self.owner).hand.clone();
        if hand.is_empty() {
            return Flow::Done;
        }
        let enough = hand.len() as i32 >= ctx.core.player(self.owner).hp;
        for card in hand {
            ctx.discard_from_hand(self.owner, card);
        }
        if enough && ctx.core.player(self.owner).alive {
            ctx.recover(self.owner, 1, None);
        }
        Flow::Done
    }
}
