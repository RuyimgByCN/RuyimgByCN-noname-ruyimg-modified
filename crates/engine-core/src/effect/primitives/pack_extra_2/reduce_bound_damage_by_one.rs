//! Effect primitives for `reduce_bound_damage_by_one` and related helpers.

use super::*;

pub(crate) fn reduce_bound_damage_by_one(
    tx: &mut Transaction,
    damage_id: u32,
) -> Result<(u32, u32), EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::Illegal(
            "damage reduction context changed".into(),
        ));
    }
    match tx
        .state_mut()
        .resolution
        .deferred_tail_effects
        .iter_mut()
        .find(|request| {
            matches!(
                request,
                EffectRequest::CommitDamage { damage_id: saved, .. }
                    | EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        ..
                    } if *saved == damage_id
            )
        }) {
        Some(request) => {
            let amount = match request {
                EffectRequest::CommitDamage { amount, .. }
                | EffectRequest::CommitDamageBanActiveIfDying { amount, .. } => amount,
                _ => unreachable!(),
            };
            let before = *amount;
            let after = before.saturating_sub(1);
            *amount = after;
            Ok((before, after))
        }
        // 批 282：触发执行期 CommitDamage 尚未入列（defer_damage_tail 之前），
        // 把减量暂存到 pending 队列，deferral 时统一套用。
        None => {
            tx.state_mut()
                .resolution
                .pending_bound_damage_deltas
                .push((damage_id, -1));
            Ok((0, 0))
        }
    }
}
