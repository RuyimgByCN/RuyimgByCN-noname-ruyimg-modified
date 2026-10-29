//! Command operations for `resolve_resolve_dckuizhen` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_dckuizhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Dckuizhen continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Dckuizhen choice observer mismatch".into(),
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
        return Err(EffectError::Illegal("Dckuizhen context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dckuizhen target stale".into()));
    }
    // Remember the initiator and let it use a Duel on the owner. The DuelResolved event
    // later drives the outcome (resolve_dckuizhen_resolve).
    run_effects(
        tx,
        vec![
            EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(crate::effect::DCKUIZHEN_PENDING_MARK),
                delta: target.raw() as i32,
            },
            EffectRequest::UseVirtualCardOn {
                player: target,
                kind: crate::state::CardKind::TrickJuedou.tag(),
                target: player,
            },
        ],
    )
    .map(|_| ())
}
