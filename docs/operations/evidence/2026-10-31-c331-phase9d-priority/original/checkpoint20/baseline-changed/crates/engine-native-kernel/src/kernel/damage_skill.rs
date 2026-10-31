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
            .and_then(|s| s.behavior.native_damage_effect())
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
            NativeDamageEffect::Zuijiu => {
                ctx.modify_damage_amount(1);
            }
        }
        Flow::Done
    }
}
