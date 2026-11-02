//! One admitted, atomic native source unit; children complete after this body has finished.
use super::*;
use crate::event::{DamageWindowKind, NativeDamageEffect};
#[derive(Clone)]
pub struct NativeDamageSkill {
    pub owner: PlayerId,
    pub def: crate::ids::SkillDefId,
    pub window: DamageWindowKind,
}
impl SkillMachine for NativeDamageSkill {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::NativeDamageSkill {
            owner: self.owner,
            def: self.def,
            window: self.window,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(effect) = ctx
            .catalog
            .skills
            .get(self.def as usize)
            .and_then(|s| s.behavior.native_damage_effect(self.window.event_kind()))
        else {
            return Flow::Done;
        };
        let d = ctx.current_damage().expect("checked window root").clone();
        match effect {
            NativeDamageEffect::Jueqing | NativeDamageEffect::Wuxin => {
                let root = ctx.current_event_frame_id();
                let amount = if effect == NativeDamageEffect::Jueqing {
                    d.amount as u32
                } else {
                    1
                };
                finish_damage(
                    ctx,
                    root,
                    d.clone(),
                    crate::event::DamageOutcome::ReplacedByLoseHp,
                );
                ctx.emit(EventData::LoseHp {
                    player: d.target,
                    amount,
                });
            }
            NativeDamageEffect::JidingRecover=>{let named=ctx.catalog.jinshenrushou_general_id();for player in ctx.core.seat_ring.clone(){if ctx.core.player(player).alive&&ctx.core.player(player).general_slots[0]==Some(named){ctx.recover(player,1,None);}}},
            NativeDamageEffect::Zuijiu => {
                ctx.modify_damage_amount(1);
            }
            NativeDamageEffect::ZhaohuoSource => {
                let root = ctx.current_event_frame_id();
                if d.target == self.owner {
                    ctx.log(LogEvent::DamagePrevented { target: d.target });
                    finish_damage(ctx, root, d, crate::event::DamageOutcome::Cancelled);
                } else {
                    let mut d = d;
                    d.nature = crate::event::DamageNature::Fire;
                    ctx.res.frame_mut(root).unwrap().data = EventData::Damage(d);
                }
            }
            effect => {
                let root = ctx.current_event_frame_id();
                match self.window {
                    DamageWindowKind::End => post_damage(ctx, d, effect, self.owner),
                    DamageWindowKind::After => {
                        damage_after::post_damage_after(ctx, d, effect, self.owner)
                    }
                    _ => damage_windows::apply_native_damage(
                        ctx,
                        root,
                        self.window,
                        effect,
                        self.def,
                    ),
                }
            }
        }
        Flow::Done
    }
}
