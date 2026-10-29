use super::*;

/// 冰封（十殿阎罗楚江王，规则 v275）：你死亡时，伤害来源存活且未翻面
/// 则将其翻面。Death 前置窗口触发（忘魂同位直写一步机；无来源/已翻面
/// 不重复翻）。
pub(in super::super) struct BingfengBehavior;

impl SkillBehavior for BingfengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let killer = match event {
            EventData::Death { killer, .. } => *killer,
            _ => None,
        };
        Box::new(BingfengMachine { owner, killer })
    }
}

/// 冰封机器：直写一步机（翻面写+TurnOvered；风雪刃确认臂同款写法）。
#[derive(Clone)]
pub(in super::super) struct BingfengMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) killer: Option<PlayerId>,
}

impl SkillMachine for BingfengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossBingfeng { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 来源=死亡事件的 killer（behavior.start 时捕获）
        if let Some(src) = self.killer {
            if src != self.owner && ctx.core.player(src).alive && !ctx.core.player(src).face_down {
                let flipped = {
                    let p = ctx.core.player_mut(src);
                    p.face_down = true;
                    p.face_down
                };
                ctx.emit(EventData::TurnOvered {
                    player: src,
                    face_down: flipped,
                });
            }
        }
        Flow::Done
    }
}
