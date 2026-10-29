use super::*;

/// 诛心（十殿阎罗阎罗王，规则 v271）：你死亡时，令场上血量最少的一名
/// 其他角色受到 2 点伤害。Death 前置窗口触发（热恼同位直写一步机；
/// 登记差异：上游 chooseTarget 最低体力池自选→座位序首个最低体力）。
pub(in super::super) struct ZhuxinBehavior;

impl SkillBehavior for ZhuxinBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhuxinMachine { owner, emitted: 0 })
    }
}

/// 诛心机器：直写一步机（emit+Yield 一次，emitted 防重入；热恼同构）。
#[derive(Clone)]
pub(in super::super) struct ZhuxinMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) emitted: u8,
}

impl SkillMachine for ZhuxinMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZhuxin {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted == 0 {
            self.emitted = 1;
            let seats = ctx.core.players.len() as PlayerId;
            // 座位序首个最低体力（严格更小才替换=并列取首）
            let mut best: Option<PlayerId> = None;
            for t in 0..seats {
                if t == self.owner || !ctx.core.player(t).alive {
                    continue;
                }
                let hp = ctx.core.player(t).hp;
                if best.map(|b| hp < ctx.core.player(b).hp).unwrap_or(true) {
                    best = Some(t);
                }
            }
            if let Some(t) = best {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    2,
                    crate::event::DamageNature::Normal,
                    None,
                )));
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}
