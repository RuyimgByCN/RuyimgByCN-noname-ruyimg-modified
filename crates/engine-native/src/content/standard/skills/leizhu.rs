use super::*;

/// 雷诛（十殿阎罗卞城王，规则 v267）：你死亡时，按座位序对全部其他角色
/// 各造成 1 点雷伤。Death 前置窗口触发（黑绳同位；forceDie 死后面=时序
/// 不敏感；死亡被取消仍已触发=登记差异）。
pub(in super::super) struct LeizhuBehavior;

impl SkillBehavior for LeizhuBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LeizhuMachine { owner, emitted: 0 })
    }
}

/// 雷诛机器：逐目标雷伤（伤害走 Damage 帧，铁索逐发同款 Yield——
/// 伤害可致死并级联遗言，逐发让管线消化窗口）。
/// 可挂起（目标濒死询问期间停机），故入快照重建表。
#[derive(Clone)]
pub(crate) struct LeizhuMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) emitted: u8,
}

impl LeizhuMachine {
    pub(crate) fn new(owner: PlayerId, emitted: u8) -> Self {
        Self { owner, emitted }
    }
}

impl SkillMachine for LeizhuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLeizhu {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        while self.emitted < seats {
            let t = self.emitted;
            self.emitted += 1;
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(self.owner),
                t,
                1,
                crate::event::DamageNature::Thunder,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}
