use super::*;

// 妄尊（袁术，P2/N08 标准缺失将收尾批 2026-10-07）：锁定技——其他
// 角色的回合开始（判定阶段开始锚点）时，若其体力值大于你，你摸一张；
// 该角色为主公时再摸一张并给其「妄尊印」（手牌上限 -1 到其回合结束）。
// 主公资格沿激将 ADR 0776 惯例（原型模式视为真）。
#[derive(Clone)]
pub(in super::super) struct WangZunBehavior;

impl SkillBehavior for WangZunBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true }
            if *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(owner).alive
                && ctx.state.player(*player).hp > ctx.state.player(owner).hp)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Phase { player, .. } => *player,
            _ => unreachable!(),
        };
        Box::new(WangZunMachine { owner, target })
    }
}

#[derive(Clone)]
pub(in super::super) struct WangZunMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) target: PlayerId,
}

impl SkillMachine for WangZunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::WangZun {
            owner: self.owner,
            target: self.target,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        if ctx.core.is_lord(self.target) {
            ctx.draw_cards(self.owner, 1);
            if let Some(mark) = ctx.catalog.skill_by_name(WANG_ZUN_MARK) {
                ctx.insert_skill(
                    (self.target, mark),
                    crate::state::SkillInstance {
                        owner: self.target,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    },
                );
            }
        }
        Flow::Done
    }
}
