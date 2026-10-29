use super::*;

/// 忘魂（十殿阎罗孟婆，规则 v270）：你死亡时，清全场往事封禁，随机两
/// 名敌方角色各随机失去一个技能。Death 前置窗口触发（黑绳同位直写一
/// 步机；登记差异：随机敌将/随机技→座位序前两个其他存活各首个启用技
/// ——skills 为 BTreeMap 按 (座位,def 序) 确定性；敌方面→全体其他；回
/// 魂牌堆注入与回收面未落）。
pub(in super::super) struct WanghunBehavior;

impl SkillBehavior for WanghunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(WanghunMachine { owner })
    }
}

/// 忘魂机器：直写一步机（禁用=SkillInstance.enabled 直写+SkillToggled，
/// 隐贼同款；无子事件不挂起）。
#[derive(Clone)]
pub(in super::super) struct WanghunMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for WanghunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossWanghun { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 孟婆死亡置乱入已死旗（上游 boss_wanghun 写 shidianyanluo_mengpodie）
        if let crate::state::GameMode::Boss { stage, .. } = &mut ctx.core.mode {
            stage.mengpo_died = true;
        }
        // ① 清全场往事封禁（上游敌方 removeSkill boss_wangshi2 同口径）
        for p in ctx.core.players.iter_mut() {
            p.wangshi_ban = false;
        }
        // ② 座位序前两个其他存活各禁用首个启用技
        let seats = ctx.core.players.len() as PlayerId;
        let mut disabled = 0u8;
        for t in 0..seats {
            if disabled >= 2 {
                break;
            }
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let first_enabled = ctx
                .core
                .skills
                .iter()
                .find(|((o, _), i)| *o == t && i.enabled)
                .map(|((_, dd), _)| *dd);
            if let Some(dd) = first_enabled {
                if let Some(inst) = ctx.core.skills.get_mut(&(t, dd)) {
                    inst.enabled = false;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: t,
                    skill: ctx.catalog.skill_name(dd),
                    enabled: false,
                });
                disabled += 1;
            }
        }
        Flow::Done
    }
}
