use super::*;

/// 火刑（十殿阎罗都市王，规则 v273）：你死亡时，全部其他存活角色按座
/// 位序各受 1 点火焰伤害（热恼同位直写逐座机；emitted=下一待扫描座位
/// 索引，中途死亡由存活过滤跳过）。
pub(in super::super) struct HuoxingBehavior;

impl SkillBehavior for HuoxingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(HuoxingMachine { owner, emitted: 0 })
    }
}

/// 火刑机器：直写逐座机（emit+Yield 逐目标，emitted=下一座位索引）。
#[derive(Clone)]
pub(in super::super) struct HuoxingMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) emitted: u8,
}

impl SkillMachine for HuoxingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossHuoxing {
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
                None,
                t,
                1,
                crate::event::DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}
