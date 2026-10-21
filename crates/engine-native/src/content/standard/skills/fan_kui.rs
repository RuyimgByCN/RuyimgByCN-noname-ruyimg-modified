use super::*;

// ---------------------------------------------------------------------------
// 反馈（司马懿）：受到 1 点伤害后，可获得伤害来源的一张牌（手牌优先，
// 其次装备区）。来源死亡/无牌或无来源时不触发。
#[derive(Clone)]
pub(in super::super) struct FanKuiBehavior {
    pub(in super::super) _p: (),
}

impl SkillBehavior for FanKuiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
            && event_damage_source_stealable(ctx, event)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source,
            _ => None,
        };
        Box::new(FanKuiMachine {
            owner,
            source,
            asked: false,
        })
    }
}

/// 来源存活且持有至少一张可取的牌。
fn event_damage_source_stealable(ctx: &QueryCtx<'_>, event: &EventData) -> bool {
    let Some(src) = (match event {
        EventData::Damage(d) => d.source,
        _ => None,
    }) else {
        return false;
    };
    let p = ctx.state.player(src);
    p.alive && (!p.hand.is_empty() || !p.equips.is_empty())
}

#[derive(Clone)]
pub(in super::super) struct FanKuiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) source: Option<PlayerId>,
    pub(in super::super) asked: bool,
}

impl SkillMachine for FanKuiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::FanKui {
            owner: self.owner,
            source: self.source,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 可选触发由 OptionalGate 的 trigger_confirm 询问确认；
        // 本机器被启动（gate 已确认）后直接执行取牌。
        if !self.asked {
            self.asked = true;
            if let Some(src) = self.source {
                if ctx.core.player(src).alive {
                    let p = ctx.core.player(src);
                    let card = p
                        .hand
                        .first()
                        .copied()
                        .or_else(|| p.equips.first().copied());
                    if let Some(c) = card {
                        ctx.give_hand_card(src, self.owner, c);
                    }
                }
            }
        }
        Flow::Done
    }
}
