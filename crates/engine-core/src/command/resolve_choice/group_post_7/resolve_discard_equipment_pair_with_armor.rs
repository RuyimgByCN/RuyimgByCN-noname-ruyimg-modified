//! Command operations for `resolve_discard_equipment_pair_with_armor` and related helpers.

use super::*;

pub(crate) fn resolve_discard_equipment_pair_with_armor(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "equipment-pair discard requires exactly one selection".into(),
        ));
    }
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing equipment-pair stage".into()))?;
    let chooser = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let target0 = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let target1 = PlayerId::new(*payload.get(4).unwrap_or(&0));
    let targets = [target0, target1];
    if observer != chooser {
        return Err(EffectError::Illegal(
            "equipment-pair choice actor changed".into(),
        ));
    }
    if !crate::query::equipment_pair_with_armor_legal(tx.state(), chooser, &targets) {
        return Err(EffectError::Illegal(
            "equipment pair with armor is no longer legal".into(),
        ));
    }

    let selected = CardId::new(selections[0]);
    match stage {
        0 => {
            let target1_has_armor =
                !crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target1)
                    .is_empty();
            let legal = if target1_has_armor {
                crate::query::discardable_equipment_cards(tx.state(), chooser, target0)
            } else {
                crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target0)
            };
            if !legal.contains(&selected) {
                return Err(EffectError::Illegal(
                    "first equipment selection is no longer legal".into(),
                ));
            }
            let first_is_armor =
                crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target0)
                    .contains(&selected);
            let options = if first_is_armor {
                crate::query::discardable_equipment_cards(tx.state(), chooser, target1)
            } else {
                crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target1)
            };
            open_equipment_pair_choice(tx, chooser, skill, target0, target1, selected, options)
                .map(|_| ())
        }
        1 => {
            let first = CardId::new(*payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("missing first equipment selection".into())
            })?);
            let first_legal =
                crate::query::discardable_equipment_cards(tx.state(), chooser, target0);
            if !first_legal.contains(&first) {
                return Err(EffectError::Illegal(
                    "first equipment selection is no longer legal".into(),
                ));
            }
            let first_is_armor =
                crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target0)
                    .contains(&first);
            let second_legal = if first_is_armor {
                crate::query::discardable_equipment_cards(tx.state(), chooser, target1)
            } else {
                crate::query::discardable_armor_equipment_cards(tx.state(), chooser, target1)
            };
            if !second_legal.contains(&selected) {
                return Err(EffectError::Illegal(
                    "second equipment selection is no longer legal".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target0,
                        cards: vec![first],
                    },
                    EffectRequest::DiscardCards {
                        player: target1,
                        cards: vec![selected],
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown equipment-pair stage".into(),
        )),
    }
}
