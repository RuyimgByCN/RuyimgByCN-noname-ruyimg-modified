//! Command operations for `resolve_potqingshi_choice` and related helpers.

use super::*;

pub(crate) fn resolve_potqingshi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad 清识 payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("invalid 清识选择者".into()));
    }
    if cancelled {
        return Ok(());
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Ok(());
    }
    let [target] = selections else {
        return Err(EffectError::Illegal("清识需要一名目标".into()));
    };
    let target = PlayerId::new(*target);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|state| state.alive)
    {
        return Err(EffectError::Illegal("清识目标已失效".into()));
    }
    use crate::state::Identity::{Lord, Loyalist, Rebel};
    let allied = player == target
        || tx
            .state()
            .players
            .get(player)
            .zip(tx.state().players.get(target))
            .is_some_and(|(a, b)| {
                matches!(
                    (a.identity, b.identity),
                    (Lord, Lord | Loyalist) | (Loyalist, Lord | Loyalist) | (Rebel, Rebel)
                )
            });
    let effects = if allied {
        vec![
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ]
    } else {
        vec![
            EffectRequest::ChooseDiscardOneHandOrEquip { player },
            EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: rule_api::TargetCardScope::HandEquip,
            },
        ]
    };
    run_effects(tx, effects).map(|_| ())
}
