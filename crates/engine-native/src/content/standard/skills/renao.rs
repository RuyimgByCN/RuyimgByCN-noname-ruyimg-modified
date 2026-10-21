use super::*;

/// 热恼（十殿阎罗泰山王，规则 v268）：你死亡时，随机一名其他角色受 3
/// 点火焰伤害。Death 前置窗口触发（雷诛同位；死亡被取消仍已触发=登记
/// 差异；randomGet→座位序首个确定性近似登记）。
pub(in super::super) struct RenaoBehavior;

impl SkillBehavior for RenaoBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(RenaoMachine { owner, emitted: 0 })
    }
}

/// 热恼机器：单发 3 火伤（Yield 让濒死窗口消化；可挂起入重建表）。
#[derive(Clone)]
pub(crate) struct RenaoMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) emitted: u8,
}

impl RenaoMachine {
    pub(crate) fn new(owner: PlayerId, emitted: u8) -> Self {
        Self { owner, emitted }
    }
}

impl SkillMachine for RenaoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossRenao {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted == 0 {
            self.emitted = 1;
            let seats = ctx.core.players.len() as PlayerId;
            if let Some(t) = (0..seats).find(|&t| t != self.owner && ctx.core.player(t).alive) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    3,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}
