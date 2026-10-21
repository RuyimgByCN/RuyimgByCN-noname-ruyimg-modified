use super::*;

/// 黑绳（十殿阎罗宋帝王，规则 v266，群雄乱入）：你死亡时，全部其他角色
/// 横置。Death 前置窗口触发（changan 氏遗言同款；forceDie 死后面=时序
/// 不敏感；死亡被取消仍已横置=登记差异）。
pub(in super::super) struct HeishengBehavior;

impl SkillBehavior for HeishengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(HeishengMachine { owner })
    }
}

/// 黑绳机器：无挂起一步机——全部其他存活横置（索命直写同款）。
#[derive(Clone)]
pub(in super::super) struct HeishengMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for HeishengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossHeisheng { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        for t in 0..seats {
            if t != self.owner && ctx.core.player(t).alive && !ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = true;
                ctx.log(LogEvent::Linked {
                    player: t,
                    linked: true,
                });
            }
        }
        Flow::Done
    }
}
