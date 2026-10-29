//! 效果队列空闲时的续行：待回放卡牌与乱武失去体力后的续结算。

use super::*;

/// Run skill-armed card replays only after no card/choice continuation remains.
pub(crate) fn finish_pending_card_replays_if_idle(tx: &mut Transaction) -> Result<(), EffectError> {
    let resolution = &tx.state().resolution;
    if resolution.pending_choice_id != 0
        || resolution.continuation_kind != ContinuationKind::None
        || resolution.resume_kind != ContinuationKind::None
        || !resolution.deferred_effects.is_empty()
        || !resolution.deferred_tail_effects.is_empty()
        || !resolution.deferred_triggers.is_empty()
        || resolution.optional_trigger.is_some()
        || resolution.pending_card_replays.iter().any(|replay| {
            resolution
                .sha_resolution_records
                .iter()
                .any(|record| record.card == replay.source_card)
        })
    {
        return Ok(());
    }
    if let Some(record) = tx.state_mut().resolution.pending_card_resolutions.pop() {
        let source_cards: Vec<_> = record
            .source_cards
            .iter()
            .copied()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.def.is_some())
            })
            .collect();
        let all_sources_were_hand = record.all_sources_were_hand && !source_cards.is_empty();
        run_effects(
            tx,
            vec![EffectRequest::EmitCardResolved {
                source: record.source,
                card: record.card,
                kind: record.kind,
                targets: record.targets,
                damaged_targets: record.damaged_targets,
                source_cards,
                all_sources_were_hand,
            }],
        )?;
        return finish_pending_card_replays_if_idle(tx);
    }
    let replays = std::mem::take(&mut tx.state_mut().resolution.pending_card_replays);
    if replays.is_empty() {
        return Ok(());
    }
    let effects = replays
        .into_iter()
        .map(|replay| EffectRequest::ReplayVirtualCardOnTargets {
            player: replay.player,
            kind: replay.kind,
            targets: replay.targets,
            skill: replay.skill,
        })
        .collect();
    run_effects(tx, effects).map(|_| ())
}

/// Resolve one 乱武 participant's refusal/impossibility branch, then continue the snapshotted
/// clockwise queue after any dying-save interruption has completed.
pub(crate) fn continue_luanwu_after_hp_loss(
    tx: &mut Transaction,
    owner: PlayerId,
    current: PlayerId,
    skill: engine_api::SkillId,
    remaining: Vec<PlayerId>,
) -> Result<(), EffectError> {
    if !remaining.is_empty() {
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                player: owner,
                remaining,
                started: true,
                skill,
            },
        );
    }
    match run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: current,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    ) {
        Ok(_) => resume_deferred_resolution(tx),
        Err(error) => Err(error),
    }
}
