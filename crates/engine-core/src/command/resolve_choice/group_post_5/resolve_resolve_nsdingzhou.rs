//! Command operations for `resolve_resolve_nsdingzhou` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_nsdingzhou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Nsdingzhou continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Nsdingzhou choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Nsdingzhou context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Nsdingzhou target stale".into()));
    }
    // Gain one card from the target's zones (first by seat order, deterministic).
    let gained = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    )
    .into_iter()
    .next();
    let Some(gained) = gained else {
        return Ok(());
    };
    let gained_color = tx
        .state()
        .cards
        .get(gained)
        .map(|card| card.suit as u32 % 2)
        .unwrap_or(0);
    let from = tx
        .state()
        .cards
        .get(gained)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Nsdingzhou card missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![gained],
            to: hand,
            from: Some(from),
        }],
    )?;
    // Draw one; the drawn card is the newest hand card (after the gain).
    run_effects(
        tx,
        vec![EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill: engine_api::SkillId::new(payload[2]),
        }],
    )?;
    let drawn = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .find(|card| *card != gained);
    if let Some(drawn) = drawn {
        let drawn_color = tx
            .state()
            .cards
            .get(drawn)
            .map(|card| card.suit as u32 % 2)
            .unwrap_or(0);
        if drawn_color != gained_color {
            return run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ());
        }
    }
    Ok(())
}
