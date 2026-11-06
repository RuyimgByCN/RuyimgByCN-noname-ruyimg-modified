//! Semantic identity for direct native extension declarations (without an npack manifest).
use super::*;

fn bytes(h: &mut blake3::Hasher, value: &[u8]) {
    h.update(&(value.len() as u64).to_le_bytes());
    h.update(value);
}
fn program(h: &mut blake3::Hasher, p: &VmSkillProgram) {
    bytes(h, p.skill_name.as_bytes());
    bytes(h, &p.module.to_bytes());
}
/// Encode the complete trigger tuple with fixed-width, portable fields.
pub fn hash_trigger_meta(h: &mut blake3::Hasher, meta: &crate::catalog::TriggerMeta) {
    h.update(&[meta.event as u8, match meta.timing { crate::event::Timing::First => 0, crate::event::Timing::Normal => 1, crate::event::Timing::Last => 2 }]);
    h.update(&meta.priority.to_le_bytes());
    h.update(&[u8::from(meta.order_selectable), u8::from(meta.optional)]);
    h.update(&(meta.extra_events.len() as u64).to_le_bytes());
    for event in &meta.extra_events { h.update(&[*event as u8]); }
}
impl VmFilterPolicy {
    /// Hash all rule-visible policy and handler identity fields, retaining declaration order.
    pub fn hash_semantics(&self, h: &mut blake3::Hasher) {
        use VmFilterPolicy::*;
        let tag = match self {
            OwnerIsTarget => 0, OwnerAlive => 1, OwnerIsSource => 2,
            OwnerAliveGaveCardsToOtherAlive => 3, Always => 4, OwnerWounded => 5,
            OwnerLinked => 6, OwnerHandNonempty => 7, OwnerHandEmpty => 8,
            OwnerNoTurnDamage => 9, OwnerSkippedPhasePositive => 10, OwnerTurnDamage => 11,
            OwnerHandBelowMaxHp => 12, OwnerHandAboveHp => 13, OwnerHandAtLeastHp => 14,
            OwnerOtherHandNonempty => 15, OwnerTurnDamageAtLeastThree => 16,
            OwnerHandAtLeastHpOtherAlive => 17, OwnerPlayPhaseBteUsed => 18,
            OwnerAlivePlayPhaseBteUsed => 19, OwnerHpBelow(_) => 20, OwnerPlayCountEq(_) => 21,
            OwnerPlayCountEqHp => 22, OwnerCardUsedAtLeastHp => 23,
            OwnerHandEmptyElementalOrTrickDamageBefore => 24, OwnerSourceMovableFieldCard => 25,
            OwnerTiedMaxHand => 26, OwnerTiedMaxHp => 27, OwnerTiedMaxEquipNonempty => 28,
            OwnerNotUniqueMaxHp => 29, OwnerTargetedByOtherSha => 30, Named(_) => 31,
            OwnerSourceSha => 32, OwnerSourceFireSha => 33, OwnerSourceOrdinaryTrick => 34,
            OwnerAliveNotEventPlayer => 35, OwnerTargetSourceAlive => 36,
            OwnerSourceOrTarget => 37, OwnerTargetOtherSourceAlive => 38,
            OwnerSourceTargetMarkPositive(_) => 39, AtPhase { .. } => 40,
            Declared { .. } => 41, Handlers(_) => 42,
            OwnerSkillAddedSelf(_) => 43, OwnerTurnedOver => 44,
            OwnerBecameFaceUpOrLinked => 45, OwnerTurnChangedOrBecameLinked => 46,
            DrawPhaseSkipped { .. } => 47, GameStarted { .. } => 48,
            EventViewAsSkill(_) => 49, PhaseAnySkipped { .. } => 50,
        };
        h.update(&[tag]);
        match self {
            OwnerHpBelow(n)
            | OwnerPlayCountEq(n)
            | OwnerSourceTargetMarkPositive(n)
            | OwnerSkillAddedSelf(n)
            | EventViewAsSkill(n) => {
                h.update(&n.to_le_bytes());
            }
            AtPhase { phase, inner } => { h.update(&[*phase as u8]); inner.hash_semantics(h); }
            DrawPhaseSkipped { inner } | GameStarted { inner } | PhaseAnySkipped { inner } => {
                inner.hash_semantics(h);
            }
            Declared { skill_def_id, hand_limit_base_max_hp, modifiers, choice_actor, inner } => {
                h.update(&skill_def_id.to_le_bytes());
                h.update(&[u8::from(*hand_limit_base_max_hp), *choice_actor as u8]);
                h.update(&(modifiers.len() as u64).to_le_bytes());
                for m in modifiers {
                    h.update(&[m.target as u8, m.mode as u8, m.selector.map_or(255, |s| s as u8)]);
                    h.update(&m.value.to_le_bytes());
                }
                inner.hash_semantics(h);
            }
            Named(p) => p.hash_semantics(h),
            Handlers(set) => {
                h.update(&set.skill_def_id.to_le_bytes());
                h.update(&[u8::from(set.active.is_some())]);
                if let Some(active) = &set.active { program(h, active); }
                let l = &set.limits;
                h.update(&[
                    u8::from(l.once_per_turn),
                    u8::from(l.once_per_play_phase),
                    u8::from(l.once_per_game),
                    l.uses_per_turn.unwrap_or(0),
                    l.uses_per_play_phase.unwrap_or(0),
                    l.uses_per_turn_source.map_or(0, |s| s as u8 + 1),
                    u8::from(l.hand_nonempty),
                    u8::from(l.equip_nonempty),
                    u8::from(l.hand_gt_hp),
                    u8::from(l.hand_gt_max_hp),
                    u8::from(l.face_up),
                    u8::from(l.owner_unlinked),
                    u8::from(l.target_other),
                    l.target_max_hp_gt.unwrap_or(0),
                ]);
                h.update(&[u8::from(l.mark_at_least.is_some())]);
                if let Some((mark, count)) = l.mark_at_least { h.update(&mark.to_le_bytes()); h.update(&count.to_le_bytes()); }
                h.update(&(set.modifiers.len() as u64).to_le_bytes());
                for m in &set.modifiers {
                    h.update(&[m.target as u8, m.mode as u8, m.selector.map_or(255, |s| s as u8)]);
                    h.update(&m.value.to_le_bytes());
                }
                h.update(&(set.triggers.len() as u64).to_le_bytes());
                for trigger in &set.triggers {
                    program(h, &trigger.program);
                    hash_trigger_meta(h, &trigger.meta);
                    h.update(&[trigger.choice_actor as u8]);
                    trigger.filter.hash_semantics(h);
                }
            }
            _ => {}
        }
    }
}
