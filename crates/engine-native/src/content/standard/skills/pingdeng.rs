use super::*;

/// 平等（十殿阎罗平等王，规则 v272）：你死亡时——最高体力其他存活
///（座位序首个）先受 2 点火焰伤害、重算后再受 1 点火焰伤害。Death 前置
/// 窗口触发（诛心同位直写两波机；登记差异：上游两波 randomGet×
/// lib.linked randomGet→座位序首个+恒定火）。
pub(in super::super) struct PingdengBehavior;

impl SkillBehavior for PingdengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(PingdengMachine { owner, emitted: 0 })
    }
}

/// 平等机器：直写两波机（emit+Yield 两次，emitted 防重入；诛心同构）。
#[derive(Clone)]
pub(in super::super) struct PingdengMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) emitted: u8,
}

impl SkillMachine for PingdengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossPingdeng {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted >= 2 {
            return Flow::Done;
        }
        let seats = ctx.core.players.len() as PlayerId;
        // 最高体力池=体力等于其他存活最大者；取座位序首个
        let mut max_hp: Option<i32> = None;
        for t in 0..seats {
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let hp = ctx.core.player(t).hp;
            if max_hp.map(|m| hp > m).unwrap_or(true) {
                max_hp = Some(hp);
            }
        }
        let target = (0..seats).find(|&t| {
            t != self.owner && ctx.core.player(t).alive && Some(ctx.core.player(t).hp) == max_hp
        });
        self.emitted += 1;
        if let Some(t) = target {
            let amount = if self.emitted == 1 { 2 } else { 1 };
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                t,
                amount,
                crate::event::DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}
