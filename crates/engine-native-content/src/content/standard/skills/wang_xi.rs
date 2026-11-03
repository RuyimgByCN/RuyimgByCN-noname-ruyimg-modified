use super::*;

// 忘隙（李典，P2/N08 标准缺失将第二批 2026-10-07）：对其他角色造成
// 伤害或受到其伤害后，可与该角色各摸一张牌（登记差异：摸牌顺序为
// 座位号升序，上游按座位序自当前回合角色起算）。
#[derive(Clone)]
pub(in super::super) struct WangXiBehavior;

impl SkillBehavior for WangXiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.amount > 0 && !d.prevented
                && d.source.is_some() && d.source != Some(d.target)
                && (d.target == owner || d.source == Some(owner))
                && ctx.state.player(d.target).alive
                && ctx.state.player(d.source.unwrap()).alive)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let other = match event {
            EventData::Damage(d) => {
                if d.target == owner {
                    d.source.expect("filtered")
                } else {
                    d.target
                }
            }
            _ => unreachable!(),
        };
        Box::new(WangXiMachine { owner, other })
    }
}

#[derive(Clone)]
pub(in super::super) struct WangXiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) other: PlayerId,
}

impl SkillMachine for WangXiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::WangXi {
            owner: self.owner,
            other: self.other,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 各摸 1（座位号升序；目标已亡则只摸自己的）。
        let mut order = vec![self.owner, self.other];
        order.sort();
        order.dedup();
        for p in order {
            if ctx.core.player(p).alive {
                ctx.start_draw(p, 1);
            }
        }
        Flow::Done
    }
}
