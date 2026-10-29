use super::*;

// ---------------------------------------------------------------------------
// 肃清：当其他玩家即将受到伤害时，其「碎盾」技能直到回合结束失效。
// （在同一个伤害前置窗口内先行结算，验证队列运行前的重查。）
// ---------------------------------------------------------------------------

pub(in super::super) struct SuQingBehavior;

impl SkillBehavior for SuQingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target != owner)
    }

    fn start(&self, _owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Damage(d) => d.target,
            _ => unreachable!("filter guarantees damage"),
        };
        Box::new(SuQingMachine { target })
    }
}

#[derive(Clone)]
pub(in super::super) struct SuQingMachine {
    pub(in super::super) target: PlayerId,
}

impl SkillMachine for SuQingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SuQing {
            target: self.target,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let suidun = ctx.catalog.skill_by_name(SUI_DUN).expect("碎盾 in catalog");
        ctx.set_skill_enabled(self.target, suidun, false);
        Flow::Done
    }
}
