use super::*;

// 狂斧（潘凤，P2/N08 标准缺失将 2026-10-07）：出牌阶段用【杀】对目标
// 造成伤害后触发（锁定）：目标体力值小于你则你摸两张牌，否则你失去
// 1 点体力。每回合限一次（上游为出牌阶段限一次——temp 技能到阶段结束；
// 因触发仅在自身出牌阶段，两者等价）。
#[derive(Clone)]
pub(in super::super) struct KuangFuBehavior;

impl SkillBehavior for KuangFuBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && d.card.map(|c| ctx.catalog.card_kind(c) == crate::catalog::CardKind::Sha).unwrap_or(false)
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Damage(d) => d.target,
            _ => unreachable!(),
        };
        Box::new(KuangFuMachine { owner, target })
    }
}

#[derive(Clone)]
pub(in super::super) struct KuangFuMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) target: PlayerId,
}

impl SkillMachine for KuangFuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::KuangFu {
            owner: self.owner,
            target: self.target,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !ctx.core.player(self.target).alive {
            return Flow::Done;
        }
        if ctx.core.player(self.target).hp < ctx.core.player(self.owner).hp {
            ctx.start_draw(self.owner, 2);
        } else {
            // 失去 1 点体力（苦肉同款：濒死检查 + Dying 子事件）。
            let p = ctx.core.player_mut(self.owner);
            p.hp -= 1;
            let entering = p.hp <= 0 && !p.dying;
            if entering {
                p.dying = true;
                ctx.log(crate::LogEvent::DyingEntered { player: self.owner });
                ctx.emit(EventData::Dying {
                    player: self.owner,
                    source: None,
                });
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}
