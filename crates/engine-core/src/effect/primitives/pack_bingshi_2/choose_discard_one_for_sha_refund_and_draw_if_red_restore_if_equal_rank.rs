//! Effect primitives for `resolve_potlvyi` and related helpers.

use super::*;

pub(crate) fn choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
    tx: &mut Transaction,
    player: PlayerId,
    sha_user: PlayerId,
    card: CardId,
    restore_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|value| value.alive)
    {
        return Ok(Vec::new());
    }
    crate::command::play::validate_saved_sha_use(tx, sha_user, card)?;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed,
        vec![
            player.raw(),
            sha_user.raw(),
            card.raw(),
            skill.raw(),
            restore_skill_def.raw(),
        ],
    )
}

pub(crate) fn turn_over(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !participant.alive {
        return Err(EffectError::Illegal("dead player cannot turn over".into()));
    }
    if crate::query::turn_over_cancelled(tx.state(), player) {
        return Ok(Vec::new());
    }
    if !participant.turned_over && crate::query::turn_face_down_cancelled(tx.state(), player) {
        return Ok(Vec::new());
    }
    let turned_over = {
        let player = tx
            .state_mut()
            .players
            .get_mut(player)
            .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
        player.turned_over = !player.turned_over;
        player.turned_over
    };
    Ok(vec![DomainEvent::PlayerTurnedOver {
        player,
        turned_over,
    }])
}

pub(crate) fn restore_character_state(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !participant.alive {
        return Err(EffectError::Illegal(
            "dead player character state cannot be restored".into(),
        ));
    }
    let linked = participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0);
    let turned_over = participant.turned_over;
    let mut events = Vec::new();
    if linked {
        events.extend(remove_mark(tx, player, crate::content::MARK_CHAIN, 0)?);
    }
    if turned_over {
        events.extend(turn_over(tx, player)?);
    }
    Ok(events)
}

pub(crate) fn gain_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    if let Some((from, card)) = first_shared_pile_card_by_kind(tx, kind)? {
        return move_cards(tx, vec![card], hand, Some(from));
    }
    Ok(Vec::new())
}
