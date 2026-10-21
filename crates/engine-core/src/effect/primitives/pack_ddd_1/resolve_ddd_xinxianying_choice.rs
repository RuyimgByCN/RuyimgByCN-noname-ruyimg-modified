//! Effect primitives for `resolve_ddd_xinxianying_choice` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_xinxianying_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XINXIANYING_CHOICE) || cancelled {
        return Err(EffectError::Illegal("bad 3D精选辛宪英 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let [raw] = selections else {
        return Err(EffectError::Illegal("洞察须选择一张牌".into()));
    };
    let card = CardId::new(*raw);
    match stage {
        0 if observer == player => {
            ddd_dongcha_move_to_pile(tx, player, player, card)?;
            let options = tx.state().hand_cards(actor);
            if options.is_empty() {
                return Ok(());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        1 if observer == actor => ddd_dongcha_move_to_pile(tx, player, actor, card),
        10 if observer == player => {
            ddd_dongcha_gain(tx, player, player, card)?;
            let remaining = tx.state().skill_pile_cards(player, DDD_DONGCHA_DEF);
            if remaining.is_empty()
                || !tx
                    .state()
                    .players
                    .get(actor)
                    .is_some_and(|state| state.alive)
            {
                return Ok(());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                remaining.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    11,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        11 if observer == actor => ddd_dongcha_gain(tx, player, actor, card),
        _ => Err(EffectError::Illegal(
            "辛宪英 choice observer or stage changed".into(),
        )),
    }
}
