use super::*;

/// 望乡（十殿阎罗秦广王，规则 v276）：你死亡时，全部有装备的其他角色
/// 装备全弃（撤技能+入弃堆；绝红拆装管线同款直写一步机）。
pub(in super::super) struct WangxiangBehavior;

impl SkillBehavior for WangxiangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(WangxiangMachine { owner })
    }
}

/// 望乡机器：直写一步机（逐目标装备全弃；绝红同款 revoke+入弃堆）。
#[derive(Clone)]
pub(in super::super) struct WangxiangMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for WangxiangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossWangxiang { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        for t in 0..seats {
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let equips: Vec<crate::ids::CardId> =
                ctx.core.player(t).equips.iter().copied().collect();
            for c in equips {
                ctx.core.player_mut(t).equips.retain(|x| *x != c);
                crate::kernel::revoke_equip_skills(&mut ctx.core, ctx.catalog, t, c);
                ctx.to_discard(c);
            }
        }
        Flow::Done
    }
}
