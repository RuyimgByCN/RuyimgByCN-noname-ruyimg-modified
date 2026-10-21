//! Command operations for `append_pothaoshi_refills` and related helpers.

use super::*;

pub(crate) fn append_pothaoshi_refills(
    state: &crate::state::GameState,
    player: PlayerId,
    cards: &[CardId],
    effects: &mut Vec<EffectRequest>,
) {
    effects.extend(crate::effect::pothaoshi_refill_effects(
        state, player, cards,
    ));
}

pub(crate) fn resolve_choose_take_larger_hand_until_max(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match (payload.first().copied(), payload) {
        (Some(0), [_, player, skill]) => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            if observer != player {
                return Err(EffectError::Illegal(
                    "larger-hand target choice belongs to another player".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled larger-hand target choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "select exactly one larger-hand target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            crate::effect::open_take_larger_hand_card_choice(tx, player, target, skill).map(|_| ())
        }
        (Some(1), [_, player, target, skill]) => {
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            let skill = engine_api::SkillId::new(*skill);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "larger-hand card choice requires exactly one card".into(),
                ));
            }
            if tx.state().hand_count(target) <= tx.state().hand_count(player) {
                return Err(EffectError::Illegal(
                    "chosen target no longer has more hand cards".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| {
                    EffectError::Illegal("larger-hand target has no hand zone".into())
                })?;
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is no longer in the larger hand".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("skill owner has no hand zone".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ContinueTakeLargerHandUntilMax { player, skill });
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad larger-hand take continuation payload".into(),
        )),
    }
}
