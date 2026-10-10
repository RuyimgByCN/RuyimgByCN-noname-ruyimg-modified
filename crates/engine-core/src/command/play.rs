//! PlayCard command handling (Phase 5: basics + equip + simple tricks).

use crate::effect::{run_effects, EffectError};
use crate::query::{query, QueryContext};
use crate::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardDefId, CardId, CardMoveReason, PlayerId, SkillId, Suit};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

pub(crate) fn pack_damage_card_context(nature: u8, card_flags: u8) -> u32 {
    u32::from(nature) | (u32::from(card_flags) << 8)
}

pub(crate) fn unpack_damage_card_context(value: u32) -> (u8, u8) {
    (value as u8, (value >> 8) as u8)
}

const SHA_RESUME_RESPONDABLE: u32 = 1 << 0;
const SHA_RESUME_COUNTED: u32 = 1 << 1;
const SHA_RESUME_REFUNDED: u32 = 1 << 2;
const SHA_TARGET_DAMAGE_BONUS_SHIFT: u32 = 8;
const SHA_TARGET_LOW_BITS_MASK: u32 = (1 << SHA_TARGET_DAMAGE_BONUS_SHIFT) - 1;

fn sha_target_damage_bonus(flags: u32) -> u32 {
    flags >> SHA_TARGET_DAMAGE_BONUS_SHIFT
}

fn add_sha_target_damage_bonus(flags: &mut u32, amount: u32) {
    let bonus = sha_target_damage_bonus(*flags)
        .saturating_add(amount)
        .min(u32::MAX >> SHA_TARGET_DAMAGE_BONUS_SHIFT);
    *flags = (*flags & SHA_TARGET_LOW_BITS_MASK) | (bonus << SHA_TARGET_DAMAGE_BONUS_SHIFT);
}

fn record_counted_sha_use(tx: &mut Transaction) {
    let turn_used = tx
        .state()
        .turn
        .flag(crate::state::turn_flag::SHA_PLAYED)
        .max(0);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::SHA_PLAYED,
        turn_used.saturating_add(1),
    );
    if tx.state().content.tracks_opening_extra_play_lifecycle() {
        let phase_used = tx
            .state()
            .turn
            .flag(crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE)
            .max(0);
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE,
            phase_used.saturating_add(1),
        );
    }
}

fn sha_use_count_allows_targets(
    state: &crate::state::GameState,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> bool {
    crate::query::sha_play_phase_use_count(state)
        < crate::query::sha_use_limit_for_card(state, actor, card)
        || (!targets.is_empty()
            && targets
                .iter()
                .all(|target| crate::query::sha_use_unlimited_for_target(state, actor, *target)))
}

fn saved_sha_flags(tx: &Transaction, respondable: bool, counted: bool) -> u32 {
    u32::from(respondable)
        | u32::from(counted && tx.state().content.tracks_sha_use_refund_provenance())
            * SHA_RESUME_COUNTED
}

/// Revalidate and refund the currently saved proactive Sha use at most once.
fn saved_sha_refund_context(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<(usize, u32), EffectError> {
    let flag_index = match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => 4,
        crate::state::ContinuationKind::ResumePlayedShaQueue => 3,
        _ => {
            return Err(EffectError::ContentFault(
                "Sha use refund requested outside a saved proactive Sha".into(),
            ));
        }
    };
    let payload = &tx.state().resolution.resume_payload;
    let (saved_user, saved_card) = match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(5).unwrap_or(&0)),
        ),
        crate::state::ContinuationKind::ResumePlayedShaQueue => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(4).unwrap_or(&0)),
        ),
        _ => unreachable!(),
    };
    if saved_user != sha_user || saved_card != card || payload.get(flag_index).is_none() {
        return Err(EffectError::ContentFault(
            "Sha use refund source/card does not match the saved continuation".into(),
        ));
    }
    Ok((flag_index, payload[flag_index]))
}

/// Revalidate that the source/card pair belongs to the currently saved proactive Sha.
pub(crate) fn validate_saved_sha_use(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    saved_sha_refund_context(tx, sha_user, card).map(|_| ())
}

/// Revalidate one declared target of the currently saved proactive Sha.
pub(crate) fn validate_saved_sha_target(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    if saved_sha_targets(tx, sha_user, card)?.contains(&target) {
        Ok(())
    } else {
        Err(EffectError::ContentFault(
            "target does not belong to the saved Sha".into(),
        ))
    }
}

/// Revalidate and refund the currently saved proactive Sha use at most once.
pub(crate) fn refund_saved_sha_use(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let (flag_index, flags) = saved_sha_refund_context(tx, sha_user, card)?;
    let record_refunded = tx
        .state()
        .resolution
        .sha_resolution_records
        .iter()
        .find(|record| record.source == sha_user && record.card == card)
        .is_some_and(|record| record.use_refunded);
    if flags & SHA_RESUME_COUNTED == 0 || flags & SHA_RESUME_REFUNDED != 0 || record_refunded {
        return Ok(());
    }
    refund_sha_use_count(tx);
    tx.state_mut().resolution.resume_payload[flag_index] |= SHA_RESUME_REFUNDED;
    if let Some(record) = tx
        .state_mut()
        .resolution
        .sha_resolution_records
        .iter_mut()
        .find(|record| record.source == sha_user && record.card == card)
    {
        record.use_refunded = true;
    }
    Ok(())
}

/// Refund an in-flight proactive Sha after its response continuation replaced the saved resume.
pub(crate) fn refund_resolving_sha_use(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let record = tx
        .state()
        .resolution
        .sha_resolution_records
        .iter()
        .find(|record| record.source == sha_user && record.card == card)
        .ok_or_else(|| {
            EffectError::ContentFault(
                "Sha use refund source/card does not match an in-flight resolution".into(),
            )
        })?;
    if !record.use_counted || record.use_refunded {
        return Ok(());
    }
    refund_sha_use_count(tx);
    tx.state_mut()
        .resolution
        .sha_resolution_records
        .iter_mut()
        .find(|record| record.source == sha_user && record.card == card)
        .expect("validated Sha resolution record")
        .use_refunded = true;
    Ok(())
}

fn refund_sha_use_count(tx: &mut Transaction) {
    let used = tx
        .state()
        .turn
        .flag(crate::state::turn_flag::SHA_PLAYED)
        .max(0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::SHA_PLAYED, used.saturating_sub(1));
    if tx.state().content.tracks_opening_extra_play_lifecycle() {
        let phase_used = tx
            .state()
            .turn
            .flag(crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE)
            .max(0);
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE,
            phase_used.saturating_sub(1),
        );
    }
}

pub(crate) fn saved_sha_targets(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    validate_saved_sha_use(tx, sha_user, card)?;
    let resolution = &tx.state().resolution;
    match resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => Ok(vec![PlayerId::new(
            *resolution.resume_payload.get(1).unwrap_or(&0),
        )]),
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let len = *resolution.resume_payload.get(5).unwrap_or(&0) as usize;
            let end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let targets = resolution.resume_payload.get(6..end).ok_or_else(|| {
                EffectError::EngineFault("truncated saved Sha target list".into())
            })?;
            Ok(targets.iter().copied().map(PlayerId::new).collect())
        }
        _ => unreachable!(),
    }
}

/// Convert the saved ordinary Sha damage context to fire and attach the Lihuo post-use flag.
pub(crate) fn convert_saved_sha_to_fire(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    validate_saved_sha_use(tx, sha_user, card)?;
    let packed_index = match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => 3,
        crate::state::ContinuationKind::ResumePlayedShaQueue => 2,
        _ => unreachable!(),
    };
    let packed = *tx
        .state()
        .resolution
        .resume_payload
        .get(packed_index)
        .ok_or_else(|| EffectError::EngineFault("saved Sha damage context missing".into()))?;
    let (nature, flags) = unpack_damage_card_context(packed);
    if nature != rule_api::damage_nature::NORMAL {
        return Err(EffectError::ContentFault(
            "Lihuo conversion requires an ordinary Sha".into(),
        ));
    }
    tx.state_mut().resolution.resume_payload[packed_index] = pack_damage_card_context(
        rule_api::damage_nature::FIRE,
        flags | rule_api::damage_card_flag::CONVERTED_FIRE_LOSE_HP_AFTER_DAMAGE,
    );
    Ok(())
}

/// Increase the base damage stored on the currently suspended proactive Sha use.
pub(crate) fn increase_saved_sha_base_damage(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    amount: u32,
) -> Result<(), EffectError> {
    validate_saved_sha_use(tx, sha_user, card)?;
    let amount_index = match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => 2,
        crate::state::ContinuationKind::ResumePlayedShaQueue => 1,
        _ => unreachable!(),
    };
    let current = *tx
        .state()
        .resolution
        .resume_payload
        .get(amount_index)
        .ok_or_else(|| EffectError::EngineFault("saved Sha base damage missing".into()))?;
    tx.state_mut().resolution.resume_payload[amount_index] = current.saturating_add(amount);
    Ok(())
}

/// Increase damage for exactly one target of the currently suspended proactive Sha use.
pub(crate) fn increase_saved_sha_target_damage(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
    amount: u32,
) -> Result<(), EffectError> {
    validate_saved_sha_target(tx, sha_user, card, target)?;
    match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let flags = tx
                .state_mut()
                .resolution
                .resume_payload
                .get_mut(4)
                .ok_or_else(|| EffectError::EngineFault("saved Sha flags missing".into()))?;
            add_sha_target_damage_bonus(flags, amount);
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let payload = tx.state().resolution.resume_payload.clone();
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let targets_end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let index = payload
                .get(6..targets_end)
                .and_then(|targets| targets.iter().position(|raw| *raw == target.raw()))
                .ok_or_else(|| EffectError::ContentFault("saved Sha target changed".into()))?;
            if payload.get(targets_end) != Some(&u32::MAX) {
                let flags = std::iter::once(u32::MAX).chain(std::iter::repeat(1).take(len));
                tx.state_mut()
                    .resolution
                    .resume_payload
                    .splice(targets_end..targets_end, flags);
            }
            let flags = tx
                .state_mut()
                .resolution
                .resume_payload
                .get_mut(targets_end + 1 + index)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target flags missing".into()))?;
            add_sha_target_damage_bonus(flags, amount);
        }
        _ => {
            return Err(EffectError::ContentFault(
                "target damage requested outside a saved proactive Sha".into(),
            ));
        }
    }
    Ok(())
}

/// Prevent exactly one declared target from responding to the suspended proactive Sha.
pub(crate) fn make_saved_sha_target_unrespondable(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    validate_saved_sha_target(tx, sha_user, card, target)?;
    match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let flags = tx
                .state_mut()
                .resolution
                .resume_payload
                .get_mut(4)
                .ok_or_else(|| EffectError::EngineFault("saved Sha flags missing".into()))?;
            *flags &= !SHA_RESUME_RESPONDABLE;
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let payload = tx.state().resolution.resume_payload.clone();
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let targets_end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let index = payload
                .get(6..targets_end)
                .and_then(|targets| targets.iter().position(|raw| *raw == target.raw()))
                .ok_or_else(|| EffectError::ContentFault("saved Sha target changed".into()))?;
            if payload.get(targets_end) != Some(&u32::MAX) {
                let flags = std::iter::once(u32::MAX).chain(std::iter::repeat(1).take(len));
                tx.state_mut()
                    .resolution
                    .resume_payload
                    .splice(targets_end..targets_end, flags);
            }
            let flags = tx
                .state_mut()
                .resolution
                .resume_payload
                .get_mut(targets_end + 1 + index)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target flags missing".into()))?;
            *flags &= !SHA_RESUME_RESPONDABLE;
        }
        _ => {
            return Err(EffectError::ContentFault(
                "target response mutation requires a suspended Sha".into(),
            ));
        }
    }
    Ok(())
}

/// Increase the base damage stored on the currently suspended damaging card use.
///
/// ABI 104 keeps the ABI 99 Sha representation intact and adds the same mutation contract for
/// Nanman/Wanjian while they are paused in their CardUsed trigger window. ABI 229 extends that
/// contract to independently resolved Juedou.
pub(crate) fn increase_saved_card_base_damage(
    tx: &mut Transaction,
    user: PlayerId,
    card: CardId,
    amount: u32,
) -> Result<(), EffectError> {
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return increase_saved_sha_base_damage(tx, user, card, amount);
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::ContentFault(
            "current card damage mutation requires a suspended Sha, Juedou, Nanman, or Wanjian"
                .into(),
        ));
    }

    let payload = &tx.state().resolution.resume_payload;
    let effect_tag = *payload.get(2).unwrap_or(&0);
    if !matches!(
        effect_tag,
        trick_effect::HUOGONG | trick_effect::JUEDOU | trick_effect::NANMAN | trick_effect::WANJIAN
    ) || payload.get(3) != Some(&user.raw())
    {
        return Err(EffectError::ContentFault(
            "current card damage mutation does not match the suspended damaging trick".into(),
        ));
    }
    let source_marker = payload
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .ok_or_else(|| EffectError::EngineFault("suspended AOE source card missing".into()))?;
    if payload.get(source_marker + 1) != Some(&card.raw()) {
        return Err(EffectError::ContentFault(
            "current card damage mutation source/card mismatch".into(),
        ));
    }
    let args_start = 7usize;
    if matches!(effect_tag, trick_effect::HUOGONG | trick_effect::JUEDOU) {
        let args = payload.get(args_start..source_marker).ok_or_else(|| {
            EffectError::EngineFault("suspended Juedou payload is truncated".into())
        })?;
        if args.first() != Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
            return Err(EffectError::ContentFault(
                "current card damage mutation does not match the suspended Juedou".into(),
            ));
        }
        let target_count = *args.get(7).unwrap_or(&0) as usize;
        let extension_at = args_start.saturating_add(8).saturating_add(target_count);
        if extension_at > source_marker {
            return Err(EffectError::EngineFault(
                "suspended Juedou target payload is truncated".into(),
            ));
        }
        if payload.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
            let current = *payload.get(extension_at + 1).ok_or_else(|| {
                EffectError::EngineFault("suspended Juedou damage extension is truncated".into())
            })?;
            tx.state_mut().resolution.resume_payload[extension_at + 1] =
                current.saturating_add(amount);
        } else {
            tx.state_mut().resolution.resume_payload.splice(
                extension_at..extension_at,
                [SAVED_CARD_BASE_DAMAGE_MAGIC, amount],
            );
        }
        return Ok(());
    }
    let target_count = *payload.get(args_start).unwrap_or(&0) as usize;
    let extension_at = args_start.saturating_add(1).saturating_add(target_count);
    if extension_at > source_marker {
        return Err(EffectError::EngineFault(
            "suspended AOE target payload is truncated".into(),
        ));
    }
    if payload.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
        let current = *payload.get(extension_at + 1).ok_or_else(|| {
            EffectError::EngineFault("suspended AOE damage extension is truncated".into())
        })?;
        tx.state_mut().resolution.resume_payload[extension_at + 1] = current.saturating_add(amount);
    } else {
        tx.state_mut().resolution.resume_payload.splice(
            extension_at..extension_at,
            [SAVED_CARD_BASE_DAMAGE_MAGIC, amount],
        );
    }
    Ok(())
}

/// Legal extra targets for the currently saved proactive Sha, in stable seat order.
pub(crate) fn saved_sha_additional_targets(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let existing = saved_sha_targets(tx, sha_user, card)?;
    Ok(tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| !existing.contains(target))
        .filter(|target| {
            validate_sha_target_with_options(tx, sha_user, card, *target, false).is_ok()
        })
        .collect())
}

/// Legal extra targets for the saved Sha while bypassing distance only.
pub(crate) fn saved_sha_additional_targets_ignoring_distance(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let existing = saved_sha_targets(tx, sha_user, card)?;
    Ok(tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| !existing.contains(target))
        .filter(|target| {
            validate_sha_target_with_options(tx, sha_user, card, *target, true).is_ok()
        })
        .collect())
}

/// Legal replacement targets for a saved Sha that are inside `range_owner`'s attack range.
pub(crate) fn saved_sha_redirect_targets_in_attack_range(
    tx: &Transaction,
    sha_user: PlayerId,
    card: CardId,
    old_target: PlayerId,
    range_owner: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    validate_saved_sha_target(tx, sha_user, card, old_target)?;
    let existing = saved_sha_targets(tx, sha_user, card)?;
    Ok(tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| !existing.contains(target))
        .filter(|target| crate::query::player_in_attack_range(tx.state(), range_owner, *target))
        .filter(|target| {
            validate_sha_target_with_options(tx, sha_user, card, *target, true).is_ok()
        })
        .collect())
}

/// Revalidate and append one target to the saved proactive Sha.
pub(crate) fn add_saved_sha_target(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    add_saved_sha_target_with_options(tx, sha_user, card, target, false)
}

/// Append one target while bypassing distance only.
pub(crate) fn add_saved_sha_target_ignoring_distance(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    add_saved_sha_target_with_options(tx, sha_user, card, target, true)
}

fn add_saved_sha_target_with_options(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    target: PlayerId,
    ignore_distance: bool,
) -> Result<Vec<PlayerId>, EffectError> {
    let legal = if ignore_distance {
        saved_sha_additional_targets_ignoring_distance(tx, sha_user, card)?
    } else {
        saved_sha_additional_targets(tx, sha_user, card)?
    };
    if !legal.contains(&target) {
        return Err(EffectError::Illegal(
            "not a legal additional Sha target".into(),
        ));
    }
    let resolution = &mut tx.state_mut().resolution;
    let targets = match resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let payload = resolution.resume_payload.clone();
            let original = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let original_resume_flags = *payload.get(4).unwrap_or(&1);
            let original_target_flags = (original_resume_flags & SHA_RESUME_RESPONDABLE)
                | (original_resume_flags & !SHA_TARGET_LOW_BITS_MASK);
            let mut next = vec![
                sha_user.raw(),
                *payload.get(2).unwrap_or(&1),
                *payload.get(3).unwrap_or(&0),
                original_resume_flags & SHA_TARGET_LOW_BITS_MASK,
                card.raw(),
                2,
                original.raw(),
                target.raw(),
                u32::MAX,
                original_target_flags,
                1,
            ];
            next.extend_from_slice(payload.get(6..).unwrap_or_default());
            resolution.resume_kind = crate::state::ContinuationKind::ResumePlayedShaQueue;
            resolution.resume_payload = next;
            Ok(vec![original, target])
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let payload = resolution.resume_payload.clone();
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let targets_end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let targets = payload
                .get(6..targets_end)
                .ok_or_else(|| EffectError::EngineFault("truncated saved Sha targets".into()))?;
            let has_flags = payload.get(targets_end) == Some(&u32::MAX)
                && payload.len() >= targets_end.saturating_add(1).saturating_add(len);
            let flags = if has_flags {
                payload
                    .get(targets_end + 1..targets_end + 1 + len)
                    .unwrap_or_default()
            } else {
                &[]
            };
            let exclusions_start = if has_flags {
                targets_end + 1 + len
            } else {
                targets_end
            };
            let exclusions = payload.get(exclusions_start..).unwrap_or_default();
            let mut all_targets: Vec<_> = targets.iter().copied().map(PlayerId::new).collect();
            all_targets.push(target);
            let mut next = payload[..6].to_vec();
            next[5] = (len + 1) as u32;
            next.extend(all_targets.iter().map(|player| player.raw()));
            next.push(u32::MAX);
            if has_flags {
                next.extend_from_slice(flags);
            } else {
                next.extend(std::iter::repeat(1).take(len));
            }
            next.push(1);
            next.extend_from_slice(exclusions);
            resolution.resume_payload = next;
            Ok(all_targets)
        }
        _ => Err(EffectError::ContentFault(
            "additional Sha target requested outside a saved proactive Sha".into(),
        )),
    }?;
    if sha_user == tx.state().turn.current && target != sha_user {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    sync_saved_sha_resolution_targets(tx, sha_user, card, &targets);
    Ok(targets)
}

fn sync_saved_sha_resolution_targets(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) {
    let resolution = &mut tx.state_mut().resolution;
    if let Some(record) = resolution
        .sha_resolution_records
        .iter_mut()
        .find(|record| record.source == source && record.card == card)
    {
        record.targets = targets.to_vec();
    }
    if let Some(record) = resolution
        .pending_card_resolutions
        .iter_mut()
        .rev()
        .find(|record| record.source == source && record.card == card)
    {
        record.targets = targets.to_vec();
    }
}

/// Replace a saved proactive Sha's declared targets, bypassing only distance and target-count.
pub(crate) fn replace_saved_sha_targets_ignoring_distance(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    validate_saved_sha_use(tx, sha_user, card)?;
    let unique: std::collections::BTreeSet<_> = targets.iter().copied().collect();
    if targets.is_empty()
        || unique.len() != targets.len()
        || !targets.iter().all(|target| {
            validate_sha_target_with_options(tx, sha_user, card, *target, true).is_ok()
        })
    {
        return Err(EffectError::Illegal(
            "invalid replacement Sha targets".into(),
        ));
    }
    let payload = tx.state().resolution.resume_payload.clone();
    let (amount, packed, resume_flags, trailing) = match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => (
            *payload.get(2).unwrap_or(&1),
            *payload.get(3).unwrap_or(&0),
            *payload.get(4).unwrap_or(&1),
            payload.get(6..).unwrap_or_default().to_vec(),
        ),
        crate::state::ContinuationKind::ResumePlayedShaQueue => (
            *payload.get(1).unwrap_or(&1),
            *payload.get(2).unwrap_or(&0),
            *payload.get(3).unwrap_or(&1),
            Vec::new(),
        ),
        _ => {
            return Err(EffectError::ContentFault(
                "Sha target replacement requested outside a saved proactive Sha".into(),
            ));
        }
    };
    let target_flags =
        (resume_flags & SHA_RESUME_RESPONDABLE) | (resume_flags & !SHA_TARGET_LOW_BITS_MASK);
    let mut next = vec![
        sha_user.raw(),
        amount,
        packed,
        resume_flags & SHA_TARGET_LOW_BITS_MASK,
        card.raw(),
        targets.len() as u32,
    ];
    next.extend(targets.iter().map(|target| target.raw()));
    next.push(u32::MAX);
    next.extend(std::iter::repeat_n(target_flags, targets.len()));
    next.extend(trailing);
    tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::ResumePlayedShaQueue;
    tx.state_mut().resolution.resume_payload = next;
    sync_saved_sha_resolution_targets(tx, sha_user, card, targets);
    Ok(())
}

/// Revalidate and replace one declared target of the saved proactive Sha.
///
/// This preserves the original target count, bypasses only the Sha user's ordinary distance to
/// the replacement, and resets target-local response/damage flags for the new target.
pub(crate) fn redirect_saved_sha_target(
    tx: &mut Transaction,
    sha_user: PlayerId,
    card: CardId,
    old_target: PlayerId,
    new_target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    validate_saved_sha_target(tx, sha_user, card, old_target)?;
    let existing = saved_sha_targets(tx, sha_user, card)?;
    if existing.contains(&new_target) {
        return Err(EffectError::Illegal(
            "redirected Sha target is already targeted".into(),
        ));
    }
    validate_sha_target_with_options(tx, sha_user, card, new_target, true)?;

    let resolution = &mut tx.state_mut().resolution;
    let targets = match resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let payload = &mut resolution.resume_payload;
            if payload.first() != Some(&sha_user.raw())
                || payload.get(1) != Some(&old_target.raw())
                || payload.get(5) != Some(&card.raw())
            {
                return Err(EffectError::ContentFault(
                    "redirected Sha does not match the saved target".into(),
                ));
            }
            payload[1] = new_target.raw();
            payload[4] =
                (payload[4] & (SHA_RESUME_COUNTED | SHA_RESUME_REFUNDED)) | SHA_RESUME_RESPONDABLE;
            let mut next = payload[..6].to_vec();
            next.extend(
                payload[6..]
                    .iter()
                    .copied()
                    .filter(|target| *target != old_target.raw() && *target != new_target.raw()),
            );
            *payload = next;
            vec![new_target]
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let payload = resolution.resume_payload.clone();
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let targets_end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let mut targets = payload
                .get(6..targets_end)
                .ok_or_else(|| EffectError::EngineFault("truncated saved Sha targets".into()))?
                .to_vec();
            let index = targets
                .iter()
                .position(|target| *target == old_target.raw())
                .ok_or_else(|| EffectError::ContentFault("saved Sha target changed".into()))?;
            targets[index] = new_target.raw();
            let has_flags = payload.get(targets_end) == Some(&u32::MAX)
                && payload.len() >= targets_end.saturating_add(1).saturating_add(len);
            let mut flags = if has_flags {
                payload[targets_end + 1..targets_end + 1 + len].to_vec()
            } else {
                vec![SHA_RESUME_RESPONDABLE; len]
            };
            flags[index] = SHA_RESUME_RESPONDABLE;
            let exclusions_start = if has_flags {
                targets_end + 1 + len
            } else {
                targets_end
            };
            let mut next = payload[..6].to_vec();
            next.extend_from_slice(&targets);
            next.push(u32::MAX);
            next.extend(flags);
            next.extend(
                payload[exclusions_start..]
                    .iter()
                    .copied()
                    .filter(|target| *target != old_target.raw() && *target != new_target.raw()),
            );
            resolution.resume_payload = next;
            targets.into_iter().map(PlayerId::new).collect()
        }
        _ => {
            return Err(EffectError::ContentFault(
                "Sha redirect requested outside a saved proactive Sha".into(),
            ));
        }
    };
    if sha_user == tx.state().turn.current && new_target != sha_user {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    sync_saved_sha_resolution_targets(tx, sha_user, card, &targets);
    Ok(targets)
}

const TIESUO_SAVED_TARGETS_MAGIC: u32 = 0x5453_4254;

fn validate_saved_independent_trick_extra_target(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    target: PlayerId,
) -> Result<(), EffectError> {
    validate_saved_independent_trick_extra_target_with_distance(
        tx, actor, card, kind, target, false,
    )
}

fn validate_saved_independent_trick_extra_target_with_distance(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    target: PlayerId,
    ignore_distance: bool,
) -> Result<(), EffectError> {
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::ContentFault("saved trick card missing".into()))?;
    validate_explicit_target_policy(tx, actor, card_state, kind, &[target])?;
    match kind {
        CardKind::TrickGuohe => {
            validate_alive_other(tx, actor, target)?;
            if discardable_target_zone_card_tokens(tx, actor, target).is_empty() {
                return Err(EffectError::Illegal("target has nothing to discard".into()));
            }
        }
        CardKind::TrickShunshou => {
            validate_alive_other(tx, actor, target)?;
            if !ignore_distance
                && !crate::query::card_target_within_distance(tx.state(), actor, card, target, 1)
            {
                return Err(EffectError::Illegal("target out of 顺手牵羊 range".into()));
            }
            if crate::query::target_immune(
                tx.state(),
                target,
                crate::query::ModifierTarget::ImmuneShunshou,
            ) {
                return Err(EffectError::Illegal("target immune to 顺手牵羊".into()));
            }
            if target_zone_card_tokens(tx, target, true, true, false).is_empty() {
                return Err(EffectError::Illegal("target has nothing to take".into()));
            }
        }
        CardKind::TrickHuogong => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return Err(EffectError::Illegal("火攻 target has no hand".into()));
            }
        }
        CardKind::TrickJuedou => validate_alive_other(tx, actor, target)?,
        _ => {
            return Err(EffectError::ContentFault(
                "saved card is not an independently resolved ordinary trick".into(),
            ));
        }
    }
    Ok(())
}

fn saved_independent_trick_target_context(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(CardKind, Vec<PlayerId>), EffectError> {
    let resolution = &tx.state().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::ContentFault(
            "bound trick target requested outside a saved card use".into(),
        ));
    }
    let payload = &resolution.resume_payload;
    if payload.get(7) != Some(&INDEPENDENT_TRICK_QUEUE_MAGIC)
        || payload.get(8) != Some(&card.raw())
        || payload.get(9) != Some(&actor.raw())
    {
        return Err(EffectError::ContentFault(
            "saved trick source/card mismatch".into(),
        ));
    }
    let kind = payload
        .get(10)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad saved trick kind".into()))?;
    let len = *payload
        .get(14)
        .ok_or_else(|| EffectError::EngineFault("saved trick target count missing".into()))?
        as usize;
    let targets = payload
        .get(15..15 + len)
        .ok_or_else(|| EffectError::EngineFault("truncated saved trick targets".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    Ok((kind, targets))
}

fn saved_tiesuo_target_context(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let resolution = &tx.state().resolution;
    let payload = &resolution.resume_payload;
    if resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow
        || payload.get(2) != Some(&trick_effect::TIESUO)
        || payload.get(3) != Some(&actor.raw())
        || payload.get(7) != Some(&TIESUO_SAVED_TARGETS_MAGIC)
        || payload.get(8) != Some(&card.raw())
    {
        return Err(EffectError::ContentFault(
            "saved 铁索连环 source/card mismatch".into(),
        ));
    }
    let len = *payload
        .get(9)
        .ok_or_else(|| EffectError::EngineFault("saved 铁索 target count missing".into()))?
        as usize;
    Ok(payload
        .get(10..10 + len)
        .ok_or_else(|| EffectError::EngineFault("truncated saved 铁索 targets".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect())
}

/// Legal additions and removals for the target list of the currently suspended card use.
/// Both lists are returned in stable play/seat order.
pub(crate) fn current_card_target_adjustments(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(Vec<PlayerId>, Vec<PlayerId>), EffectError> {
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        let existing = saved_sha_targets(tx, actor, card)?;
        let add = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| !existing.contains(target))
            .filter(|target| {
                validate_sha_target_with_options(tx, actor, card, *target, false).is_ok()
            })
            .collect();
        let remove = (existing.len() > 1).then_some(existing).unwrap_or_default();
        return Ok((add, remove));
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Ok((Vec::new(), Vec::new()));
    }
    let payload = &tx.state().resolution.resume_payload;
    // 批 155：南蛮/万箭（AOE 队列族）的 wuxie 载荷目标列表可移除——啖酪族
    // 「令此牌对你无效」在挂起期内查询可移除目标。
    let effect_tag_now = *payload.get(2).unwrap_or(&0);
    if matches!(effect_tag_now, trick_effect::NANMAN | trick_effect::WANJIAN) {
        let len = payload.get(7).copied().unwrap_or(0) as usize;
        let start = 8usize;
        if start + len <= payload.len() && len > 0 {
            let existing: Vec<_> = payload[start..start + len]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let remove = (existing.len() > 1).then_some(existing).unwrap_or_default();
            return Ok((Vec::new(), remove));
        }
    }
    if payload.get(2) == Some(&trick_effect::TIAOJIYANMEI) {
        let marker = payload
            .iter()
            .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC);
        if marker.is_some_and(|index| payload.get(index + 1) == Some(&card.raw()) && index >= 9) {
            let existing: Vec<_> = payload[7..9]
                .iter()
                .copied()
                .filter(|raw| raw & SAVED_TARGET_EXCLUDED_MASK == 0)
                .map(PlayerId::new)
                .collect();
            let remove = (existing.len() > 1).then_some(existing).unwrap_or_default();
            return Ok((Vec::new(), remove));
        }
    }
    if payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC) {
        let existing = saved_tiesuo_target_context(tx, actor, card)?;
        let card_state = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::ContentFault("saved 铁索连环 card missing".into()))?;
        let add = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| !existing.contains(target))
            .filter(|target| {
                validate_explicit_target_policy(
                    tx,
                    actor,
                    card_state,
                    CardKind::TrickTiesuo,
                    &[*target],
                )
                .is_ok()
            })
            .collect();
        let remove = (existing.len() > 1).then_some(existing).unwrap_or_default();
        return Ok((add, remove));
    }
    if payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        let (kind, existing) = saved_independent_trick_target_context(tx, actor, card)?;
        let add = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| !existing.contains(target))
            .filter(|target| {
                validate_saved_independent_trick_extra_target(tx, actor, card, kind, *target)
                    .is_ok()
            })
            .collect();
        let remove = (existing.len() > 1).then_some(existing).unwrap_or_default();
        return Ok((add, remove));
    }
    Ok((Vec::new(), Vec::new()))
}

/// Revalidate that a physical Sha or ordinary trick is still the suspended card use.
pub(crate) fn validate_saved_targeted_card_use(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("saved targeted card is missing".into()))?;
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    if kind.is_sha() {
        return validate_saved_sha_use(tx, actor, card);
    }
    if !kind.is_trick() || tx.state().resolution.resume_kind != ContinuationKind::WuxieWindow {
        return Err(EffectError::Illegal(
            "no matching suspended targeted card use".into(),
        ));
    }
    let payload = &tx.state().resolution.resume_payload;
    let marker = payload
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .ok_or_else(|| EffectError::Illegal("saved targeted card marker is missing".into()))?;
    if payload.get(3) != Some(&actor.raw()) || payload.get(marker + 1) != Some(&card.raw()) {
        return Err(EffectError::Illegal(
            "saved targeted card source/card mismatch".into(),
        ));
    }
    Ok(())
}

/// Return the full target list of the exact suspended Sha or ordinary trick use.
pub(crate) fn saved_targeted_card_targets(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    validate_saved_targeted_card_use(tx, actor, card)?;
    if tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| crate::query::effective_card_kind(tx.state(), actor, card).is_sha())
    {
        return saved_sha_targets(tx, actor, card);
    }
    let payload = &tx.state().resolution.resume_payload;
    if payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return saved_independent_trick_target_context(tx, actor, card).map(|(_, targets)| targets);
    }
    if payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC) {
        return saved_tiesuo_target_context(tx, actor, card);
    }
    let marker = payload
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .ok_or_else(|| EffectError::EngineFault("saved targeted card marker missing".into()))?;
    if payload.get(marker + 1) != Some(&card.raw()) {
        return Err(EffectError::Illegal("saved targeted card changed".into()));
    }
    let effect_tag = *payload.get(2).unwrap_or(&0);
    let targets = match effect_tag {
        trick_effect::NANMAN | trick_effect::WANJIAN => {
            let len = *payload.get(7).unwrap_or(&0) as usize;
            payload.get(8..8 + len).unwrap_or_default().to_vec()
        }
        trick_effect::TAOYUAN | trick_effect::WUGU if marker == 7 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        trick_effect::TAOYUAN | trick_effect::WUGU => {
            let len = *payload.get(7).unwrap_or(&0) as usize;
            payload.get(8..8 + len).unwrap_or_default().to_vec()
        }
        trick_effect::JIEDAO | trick_effect::TIAOJIYANMEI => payload
            .get(7..9)
            .unwrap_or_default()
            .iter()
            .copied()
            .map(|raw| raw & !SAVED_TARGET_EXCLUDED_MASK)
            .filter(|raw| *raw != 0)
            .collect(),
        _ => payload
            .get(7)
            .copied()
            .into_iter()
            .filter(|raw| *raw != 0)
            .collect(),
    };
    Ok(targets.into_iter().map(PlayerId::new).collect())
}

/// Revalidate and apply one target-list adjustment to the suspended card use.
pub(crate) fn adjust_saved_card_target(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    add: bool,
) -> Result<Vec<PlayerId>, EffectError> {
    let (add_targets, remove_targets) = current_card_target_adjustments(tx, actor, card)?;
    if add {
        if !add_targets.contains(&target) {
            return Err(EffectError::Illegal(
                "not a legal additional current-card target".into(),
            ));
        }
        return add_saved_card_target(tx, actor, card, target);
    }
    if !remove_targets.contains(&target) {
        return Err(EffectError::Illegal(
            "not a legal removable current-card target".into(),
        ));
    }
    match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let payload = tx.state().resolution.resume_payload.clone();
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let targets_end = 6usize
                .checked_add(len)
                .ok_or_else(|| EffectError::EngineFault("saved Sha target overflow".into()))?;
            let targets = payload
                .get(6..targets_end)
                .ok_or_else(|| EffectError::EngineFault("truncated saved Sha targets".into()))?;
            let remove_index = targets
                .iter()
                .position(|raw| *raw == target.raw())
                .ok_or_else(|| EffectError::Illegal("saved Sha target changed".into()))?;
            let has_flags = payload.get(targets_end) == Some(&u32::MAX)
                && payload.len() >= targets_end.saturating_add(1).saturating_add(len);
            let flags = if has_flags {
                payload
                    .get(targets_end + 1..targets_end + 1 + len)
                    .unwrap_or_default()
            } else {
                &[]
            };
            let exclusions_start = if has_flags {
                targets_end + 1 + len
            } else {
                targets_end
            };
            let mut next = payload[..6].to_vec();
            next[5] = (len - 1) as u32;
            next.extend(
                targets
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != remove_index)
                    .map(|(_, raw)| *raw),
            );
            if has_flags {
                next.push(u32::MAX);
                next.extend(
                    flags
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index != remove_index)
                        .map(|(_, flag)| *flag),
                );
            }
            next.extend(
                payload[exclusions_start..]
                    .iter()
                    .copied()
                    .filter(|raw| *raw != target.raw()),
            );
            tx.state_mut().resolution.resume_payload = next;
        }
        crate::state::ContinuationKind::WuxieWindow => {
            let tiesuo =
                tx.state().resolution.resume_payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC);
            let (count_index, targets_start) = if tiesuo { (9, 10) } else { (14, 15) };
            let payload = &mut tx.state_mut().resolution.resume_payload;
            let len = payload[count_index] as usize;
            let index = payload[targets_start..targets_start + len]
                .iter()
                .position(|raw| *raw == target.raw())
                .ok_or_else(|| EffectError::Illegal("saved trick target changed".into()))?;
            payload.remove(targets_start + index);
            payload[count_index] = (len - 1) as u32;
        }
        _ => {
            return Err(EffectError::ContentFault(
                "target removal requested outside a mutable saved card use".into(),
            ));
        }
    }
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        saved_sha_targets(tx, actor, card)
    } else if tx.state().resolution.resume_payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC) {
        saved_tiesuo_target_context(tx, actor, card)
    } else if tx.state().resolution.resume_payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        saved_independent_trick_target_context(tx, actor, card).map(|(_, targets)| targets)
    } else {
        Err(EffectError::EngineFault(
            "adjusted card target context disappeared".into(),
        ))
    }
}

/// Whether `target` can be appended to the currently suspended one-target Sha or ordinary trick.
pub(crate) fn saved_card_extra_target_is_legal(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> bool {
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return saved_sha_targets(tx, actor, card).is_ok_and(|targets| {
            targets.len() == 1
                && !targets.contains(&target)
                && validate_sha_target_with_options(tx, actor, card, target, false).is_ok()
        });
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return false;
    }
    if tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO) {
        return saved_tiesuo_target_context(tx, actor, card).is_ok_and(|targets| {
            targets.len() == 1
                && !targets.contains(&target)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
                && tx.state().cards.get(card).is_some_and(|card_state| {
                    validate_explicit_target_policy(
                        tx,
                        actor,
                        card_state,
                        CardKind::TrickTiesuo,
                        &[target],
                    )
                    .is_ok()
                })
        });
    }
    saved_independent_trick_target_context(tx, actor, card).is_ok_and(|(kind, targets)| {
        targets.len() == 1
            && !targets.contains(&target)
            && validate_saved_independent_trick_extra_target(tx, actor, card, kind, target).is_ok()
    })
}

/// 专摄 legality: any non-empty mutable target list, ignoring only ordinary distance limits.
pub(crate) fn saved_card_extra_target_is_legal_ignoring_distance(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> bool {
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return saved_sha_targets(tx, actor, card).is_ok_and(|targets| {
            !targets.is_empty()
                && !targets.contains(&target)
                && validate_sha_target_with_options(tx, actor, card, target, true).is_ok()
        });
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return false;
    }
    if tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO) {
        return saved_tiesuo_target_context(tx, actor, card).is_ok_and(|targets| {
            !targets.is_empty()
                && !targets.contains(&target)
                && tx.state().cards.get(card).is_some_and(|card_state| {
                    validate_explicit_target_policy(
                        tx,
                        actor,
                        card_state,
                        CardKind::TrickTiesuo,
                        &[target],
                    )
                    .is_ok()
                })
        });
    }
    saved_independent_trick_target_context(tx, actor, card).is_ok_and(|(kind, targets)| {
        !targets.is_empty()
            && !targets.contains(&target)
            && validate_saved_independent_trick_extra_target_with_distance(
                tx, actor, card, kind, target, true,
            )
            .is_ok()
    })
}

/// Transfer the user of the currently suspended one-target Sha or ordinary trick after the
/// resolver already validated the selected player and completed their gift. The post-gift check
/// deliberately does not reinterpret that player as a target: giving their last hand card may
/// make them ineligible for e.g. Huogong, but they are becoming the user rather than a target.
pub(crate) fn transfer_saved_card_user(
    tx: &mut Transaction,
    previous: PlayerId,
    new_user: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    if previous == new_user
        || !tx
            .state()
            .players
            .get(new_user)
            .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal(
            "new card user is no longer legal for the suspended use".into(),
        ));
    }

    match tx.state().resolution.resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha
        | crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let targets = saved_sha_targets(tx, previous, card)?;
            if targets.len() != 1 || targets.contains(&new_user) {
                return Err(EffectError::Illegal(
                    "card-user transfer requires one saved Sha target".into(),
                ));
            }
            tx.state_mut().resolution.resume_payload[0] = new_user.raw();
            if let Some(record) = tx
                .state_mut()
                .resolution
                .sha_resolution_records
                .iter_mut()
                .find(|record| record.source == previous && record.card == card)
            {
                record.source = new_user;
            }
        }
        crate::state::ContinuationKind::WuxieWindow => {
            let independent =
                tx.state().resolution.resume_payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC);
            let tiesuo =
                tx.state().resolution.resume_payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC);
            if !independent && !tiesuo {
                return Err(EffectError::ContentFault(
                    "card-user transfer requires a saved ordinary trick".into(),
                ));
            }
            let targets = if independent {
                saved_independent_trick_target_context(tx, previous, card)?.1
            } else {
                saved_tiesuo_target_context(tx, previous, card)?
            };
            if targets.len() != 1 || targets.contains(&new_user) {
                return Err(EffectError::Illegal(
                    "card-user transfer requires one distinct saved trick target".into(),
                ));
            }
            let payload = &mut tx.state_mut().resolution.resume_payload;
            payload[0] = new_user.raw();
            payload[3] = new_user.raw();
            payload[4] = new_user.raw();
            if independent {
                payload[9] = new_user.raw();
            }
        }
        _ => {
            return Err(EffectError::ContentFault(
                "card-user transfer requested outside a saved card use".into(),
            ));
        }
    }
    Ok(())
}

/// Revalidate the suspended one-target Sha/ordinary-trick context used by generic card-use
/// enhancement skills and return all legal additional targets in seat order.
pub(crate) fn current_card_use_enhancement_extra_targets(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<Option<Vec<PlayerId>>, EffectError> {
    if actor != tx.state().turn.current
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|player| player.alive)
    {
        return Ok(None);
    }
    let single_target = if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        saved_sha_targets(tx, actor, card).is_ok_and(|targets| targets.len() == 1)
    } else if tx.state().resolution.resume_kind == crate::state::ContinuationKind::WuxieWindow {
        if tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO) {
            saved_tiesuo_target_context(tx, actor, card).is_ok_and(|targets| targets.len() == 1)
        } else {
            saved_independent_trick_target_context(tx, actor, card).is_ok_and(|(kind, targets)| {
                kind.is_trick() && !kind.is_delayed_trick() && targets.len() == 1
            })
        }
    } else {
        false
    };
    if !single_target {
        return Ok(None);
    }

    for other in tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|other| *other != actor)
    {
        let distance = query(
            tx.state(),
            &QueryRequest::Distance {
                from: actor,
                to: other,
            },
            &mut QueryContext::new(),
        );
        if !matches!(distance, QueryResponse::Int(value) if value <= 1) {
            return Ok(None);
        }
    }

    Ok(Some(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| saved_card_extra_target_is_legal(tx, actor, card, *target))
            .collect(),
    ))
}

/// Revalidate and append one target to the currently suspended one-target Sha or ordinary trick.
pub(crate) fn add_saved_card_target(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    if !current_card_target_adjustments(tx, actor, card)?
        .0
        .contains(&target)
    {
        return Err(EffectError::Illegal(
            "not a legal bound additional card target".into(),
        ));
    }
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return add_saved_sha_target(tx, actor, card, target);
    }
    let tiesuo = tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO);
    let (count_index, targets_start) = if tiesuo { (9, 10) } else { (14, 15) };
    let payload = &mut tx.state_mut().resolution.resume_payload;
    let len = payload[count_index] as usize;
    let mut targets: Vec<_> = payload[targets_start..targets_start + len]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    targets.push(target);
    payload[count_index] = targets.len() as u32;
    payload.insert(targets_start + len, target.raw());
    if actor == tx.state().turn.current && target != actor {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    Ok(targets)
}

pub(crate) fn add_saved_card_target_ignoring_distance(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    if !saved_card_extra_target_is_legal_ignoring_distance(tx, actor, card, target) {
        return Err(EffectError::Illegal(
            "not a legal 专摄 additional target".into(),
        ));
    }
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return add_saved_sha_target(tx, actor, card, target);
    }
    let tiesuo = tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO);
    let (count_index, targets_start) = if tiesuo { (9, 10) } else { (14, 15) };
    let payload = &mut tx.state_mut().resolution.resume_payload;
    let len = payload[count_index] as usize;
    payload.insert(targets_start + len, target.raw());
    payload[count_index] = payload[count_index].saturating_add(1);
    let targets: Vec<_> = payload[targets_start..targets_start + len + 1]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if actor == tx.state().turn.current && target != actor {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    Ok(targets)
}

fn mark_sha_damage_in_payload(
    kind: crate::state::ContinuationKind,
    payload: &mut [u32],
    source: PlayerId,
    card: CardId,
) {
    let packed_index = match kind {
        crate::state::ContinuationKind::FinishPlayedSha
            if payload.first() == Some(&source.raw()) && payload.get(1) == Some(&card.raw()) =>
        {
            Some(2)
        }
        crate::state::ContinuationKind::RespondAoeQueue
            if payload.first() == Some(&3) && payload.get(1) == Some(&source.raw()) =>
        {
            let len = *payload.get(4).unwrap_or(&0) as usize;
            (payload.get(8 + 2 * len) == Some(&card.raw())).then_some(5)
        }
        _ => None,
    };
    let Some(packed_index) = packed_index else {
        return;
    };
    let Some(packed) = payload.get(packed_index).copied() else {
        return;
    };
    let (nature, flags) = unpack_damage_card_context(packed);
    payload[packed_index] = pack_damage_card_context(
        nature,
        flags | rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE,
    );
}

/// Record committed damage on whichever continuation owns the current post-use Sha rule.
pub(crate) fn mark_current_sha_dealt_damage(tx: &mut Transaction, source: PlayerId, card: CardId) {
    let resolution = &mut tx.state_mut().resolution;
    mark_sha_damage_in_payload(
        resolution.continuation_kind,
        &mut resolution.continuation_payload,
        source,
        card,
    );
    mark_sha_damage_in_payload(
        resolution.resume_kind,
        &mut resolution.resume_payload,
        source,
        card,
    );
}

/// Preserve the post-use boundary across damage, dying, and damage-trigger choices.
pub(crate) fn save_played_sha_finish(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    packed_context: u32,
) -> Result<(), EffectError> {
    let resolution = &mut tx.state_mut().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::None {
        return Err(EffectError::EngineFault(
            "converted Sha post-use continuation would overwrite another resume".into(),
        ));
    }
    resolution.resume_kind = crate::state::ContinuationKind::FinishPlayedSha;
    resolution.resume_payload = vec![source.raw(), card.raw(), packed_context];
    Ok(())
}

/// Complete a converted Sha use after every target and nested response/dying window has ended.
pub(crate) fn finish_played_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    packed_context: u32,
) -> Result<(), EffectError> {
    let (_, flags) = unpack_damage_card_context(packed_context);
    let alive = tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive);
    let no_damage_skill = tx.state_mut().resolution.take_sha_no_damage_draw(card);
    let resolution_record = tx.state_mut().resolution.take_sha_resolution(source, card);
    let dying_cost_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.twlihuo_dying_cost_card() == Some(card))
        .map(|skill| skill.id);
    let twlihuo_caused_dying = dying_cost_skill.and_then(|skill| {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_twlihuo_dying_cost(card)
    });
    let mut effects = Vec::new();
    if alive
        && flags & rule_api::damage_card_flag::CONVERTED_FIRE_LOSE_HP_AFTER_DAMAGE != 0
        && twlihuo_caused_dying
            .unwrap_or(flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0)
    {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    if flags & rule_api::damage_card_flag::DRAW_IF_NO_DAMAGE != 0 {
        let skill = no_damage_skill.ok_or_else(|| {
            EffectError::EngineFault("Sha no-damage action lost its provenance".into())
        })?;
        if flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE == 0 {
            if skill.is_none() {
                if resolution_record
                    .as_ref()
                    .is_some_and(|record| record.use_counted && !record.use_refunded)
                {
                    refund_sha_use_count(tx);
                }
            } else if alive {
                effects.push(EffectRequest::DrawCardsFromSkill {
                    player: source,
                    count: 1,
                    skill,
                });
            }
        }
    }
    if let Some(record) = resolution_record {
        let backwater_targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                record
                    .backwater_targets
                    .iter()
                    .any(|(_, recorded)| recorded == target)
                    && !record.dying_targets.contains(target)
            })
            .collect();
        effects.push(EffectRequest::EmitShaResolved {
            source,
            card,
            targets: record.targets,
            response_cards: record.response_cards,
            card_flags: flags,
        });
        effects.extend(backwater_targets.into_iter().map(|target| {
            EffectRequest::ChooseDiscardTargetCard {
                player: target,
                target: source,
                scope: rule_api::TargetCardScope::HandEquip,
            }
        }));
    }
    if effects.is_empty() {
        crate::effect::finish_pending_card_replays_if_idle(tx)
    } else {
        run_effects(tx, effects)?;
        crate::effect::finish_pending_card_replays_if_idle(tx)
    }
}

/// Finish and clear the single-target post-use resume when no nested choice interrupted it.
pub(crate) fn finish_saved_played_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let resolution = &mut tx.state_mut().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::FinishPlayedSha {
        return Ok(());
    }
    let payload = std::mem::take(&mut resolution.resume_payload);
    resolution.resume_kind = crate::state::ContinuationKind::None;
    if payload.first() != Some(&source.raw()) || payload.get(1) != Some(&card.raw()) {
        return Err(EffectError::EngineFault(
            "converted Sha post-use continuation changed source/card".into(),
        ));
    }
    finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0))
}

/// Deal one single-target Sha hit and complete its post-use boundary if uninterrupted.
pub(crate) fn deal_single_sha_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    card: CardId,
) -> Result<(), EffectError> {
    save_played_sha_finish(
        tx,
        source,
        card,
        pack_damage_card_context(nature, card_flags),
    )?;
    deal_sha_damage(tx, source, target, amount, nature, card_flags, card)?;
    finish_saved_played_sha(tx, source, card)
}

fn remember_hand_card_suit_use(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    source_was_hand: bool,
) {
    if !source_was_hand
        || actor != tx.state().turn.current
        || !tx.state().content.tracks_turn_hand_card_suit_history()
    {
        return;
    }
    let suit = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_suit(tx.state(), actor, card));
    if let Some(suit) = suit {
        tx.state_mut()
            .turn
            .remember_hand_card_suit_used(actor, suit);
    }
}

pub(super) fn handle_play_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    _card_targets: &[CardId],
) -> Result<(), EffectError> {
    if tx.state().revision == 0 {
        return Err(EffectError::Illegal("not started".into()));
    }
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice pending".into()));
    }
    if tx.state().turn.phase != Phase::Play {
        return Err(EffectError::Illegal("not play phase".into()));
    }
    if actor.is_none() || actor != tx.state().turn.current {
        return Err(EffectError::Illegal("not current player".into()));
    }
    handle_card_from_hand(tx, actor, card, targets)
}

fn handle_card_from_hand(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    handle_card_from_hand_with_options(tx, actor, card, targets, false, true)
}

pub(crate) fn use_equipment_from_hand(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), actor, card))
        .ok_or_else(|| EffectError::Illegal("bad equipment card".into()))?;
    if !kind.is_equip() {
        return Err(EffectError::Illegal("given card is not equipment".into()));
    }
    handle_card_from_hand(tx, actor, card, &[])
}

/// Use an equipment card directly from a shared source zone without fabricating a hand gain.
pub(crate) fn use_equipment_from_shared_zone(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad equipment card".into()))?;
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    let shared = tx
        .state()
        .zones
        .get(card_state.zone)
        .is_some_and(|zone| zone.owner.is_none());
    if !kind.is_equip() || !shared {
        return Err(EffectError::Illegal(
            "equipment card is not in a shared zone".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
        || !kind.equip_slot().is_some_and(|slot| {
            tx.state()
                .players
                .get(actor)
                .unwrap()
                .equip_slot_enabled(slot)
        })
        || crate::query::card_use_cancelled_for_card(tx.state(), actor, card)
    {
        return Err(EffectError::Illegal(
            "equipment card is not currently usable".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .plan_card_source_cards(actor, card, vec![card], false);
    equip_card(tx, actor, card, kind)
}

fn handle_card_from_hand_with_options(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    handle_card_from_hand_with_policies(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        false,
        source_was_hand,
    )
}

fn handle_card_from_hand_with_policies(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    handle_card_from_hand_with_policies_and_flags(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        source_was_hand,
        0,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_card_from_hand_with_policies_and_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
) -> Result<(), EffectError> {
    handle_card_with_policies_and_flags(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        source_was_hand,
        extra_damage_flags,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_card_with_policies_and_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    field_source: bool,
) -> Result<(), EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?
        .clone();
    if !player.alive || player.dying {
        return Err(EffectError::Illegal("actor not able to play".into()));
    }

    let c = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?
        .clone();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let valid_source = if field_source {
        tx.state()
            .zones
            .get(c.zone)
            .is_some_and(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
    } else {
        tx.state().playable_hand_cards(actor).contains(&card)
            || (tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32
                && tx
                    .state()
                    .zones
                    .get(c.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == actor))
    };
    if !valid_source {
        return Err(EffectError::Illegal("card not in hand".into()));
    }
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, &c);
    validate_explicit_target_policy(tx, actor, &c, kind, targets)?;
    let source_counts_as_hand =
        c.zone == hand || crate::query::borrowed_hand_source(tx.state(), actor, card).is_some();
    remember_hand_card_suit_use(tx, actor, card, source_was_hand && source_counts_as_hand);

    let mut queue: Vec<EffectRequest> = Vec::new();

    match kind {
        // 影不可响应任何窗口（无队列效果）。
        CardKind::Ying => {}
        CardKind::Shan => {
            return Err(EffectError::Illegal("cannot play shan proactively".into()));
        }
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let ignores_use_count = ignore_use_count
                || crate::query::sha_use_does_not_count_for_card(tx.state(), actor, card);
            let respondable = !crate::query::sha_unrespondable_for_card(tx.state(), actor, card);
            if !ignores_use_count && !sha_use_count_allows_targets(tx.state(), actor, card, targets)
            {
                return Err(EffectError::Illegal("sha use limit reached".into()));
            }
            validate_sha_targets_with_options(tx, actor, card, targets, ignore_distance)?;
            let nature = crate::query::effective_card_nature(tx.state(), actor, &c, kind);
            let mut amount = 1u32;
            let jiu = player
                .marks
                .iter()
                .any(|(m, c)| *m == crate::content::MARK_JIU && *c > 0);
            let card_flags = (if jiu {
                rule_api::damage_card_flag::JIU_BOOSTED
            } else {
                0
            }) | extra_damage_flags;
            if jiu {
                amount = 2;
                queue.push(EffectRequest::RemoveMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    amount: 1,
                });
            }
            // Apply jiu mark remove + use card first
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
            // Record that the current player played a 杀 this turn (克己).
            if !ignores_use_count {
                record_counted_sha_use(tx);
            }
            let resume_flags = saved_sha_flags(tx, respondable, !ignores_use_count);
            let resume = if targets.len() == 1 {
                (
                    crate::state::ContinuationKind::ResumePlayedSha,
                    vec![
                        actor.raw(),
                        targets[0].raw(),
                        amount,
                        pack_damage_card_context(nature, card_flags),
                        resume_flags,
                        card.raw(),
                    ],
                )
            } else {
                let mut payload = vec![
                    actor.raw(),
                    amount,
                    pack_damage_card_context(nature, card_flags),
                    resume_flags,
                    card.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(u32::MAX);
                payload.extend(std::iter::repeat(1).take(targets.len()));
                (
                    crate::state::ContinuationKind::ResumePlayedShaQueue,
                    payload,
                )
            };
            let (kind, payload) = run_effects_with_saved_resume(tx, queue, resume.0, resume.1)?;
            return resume_played_sha_from_payload(tx, kind, &payload);
        }
        CardKind::Tao => {
            if crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                actor,
                &c,
                CardKind::Tao,
            ) {
                return Err(EffectError::Illegal("tao target disabled".into()));
            }
            let hp = player.hp;
            let max = player.max_hp as i32;
            if hp >= max {
                return Err(EffectError::Illegal("hp full".into()));
            }
            if !targets.is_empty() && targets != [actor] {
                return Err(EffectError::Illegal("tao self only in play phase".into()));
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            queue.push(EffectRequest::RecoverHp {
                source: actor,
                player: actor,
                amount: 1,
            });
        }
        CardKind::Jiu => {
            // M1: if wounded → recover 1; else gain JIU mark (next Sha +1 damage).
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            if player.hp < player.max_hp as i32 {
                queue.push(EffectRequest::RecoverHp {
                    source: actor,
                    player: actor,
                    amount: 1,
                });
            } else {
                queue.push(EffectRequest::AddMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                });
            }
        }
        CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure => {
            return equip_card(tx, actor, card, kind);
        }
        CardKind::TrickWuxie | CardKind::TrickYoudishenru => {
            return Err(EffectError::Illegal("card is response only".into()));
        }
        CardKind::TrickWuzhong => {
            return use_card_then_wuxie(tx, actor, card, &[], trick_effect::WUZHONG, &[]);
        }
        CardKind::TrickGuohe => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "guohe")?;
            for &target in targets {
                let t = tx
                    .state()
                    .players
                    .get(target)
                    .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
                if !t.alive {
                    return Err(EffectError::Illegal("target dead".into()));
                }
                if discardable_target_zone_card_tokens(tx, actor, target).is_empty() {
                    return Err(EffectError::Illegal("target has nothing to discard".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::GUOHE,
                field_source,
            );
        }
        CardKind::TrickShunshou => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "shunshou")?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
                if !ignore_distance
                    && !crate::query::card_target_within_distance(
                        tx.state(),
                        actor,
                        card,
                        target,
                        1,
                    )
                {
                    return Err(EffectError::Illegal("target out of 顺手牵羊 range".into()));
                }
                if crate::query::target_immune(
                    tx.state(),
                    target,
                    crate::query::ModifierTarget::ImmuneShunshou,
                ) {
                    return Err(EffectError::Illegal(
                        "target immune to 顺手牵羊 (谦逊)".into(),
                    ));
                }
                if target_zone_card_tokens(tx, target, true, true, false).is_empty() {
                    return Err(EffectError::Illegal("target has nothing to take".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::SHUNSHOU,
                field_source,
            );
        }
        CardKind::TrickHuogong => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "huogong")?;
            for &target in targets {
                let t = tx
                    .state()
                    .players
                    .get(target)
                    .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
                if !t.alive {
                    return Err(EffectError::Illegal("target dead".into()));
                }
                if tx.state().hand_cards(target).is_empty() {
                    return Err(EffectError::Illegal("target has no hand".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::HUOGONG,
                field_source,
            );
        }
        CardKind::TrickJuedou => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "juedou")?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::JUEDOU,
                field_source,
            );
        }
        CardKind::TrickYuanjiao => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "yuanjiao")?;
            let target = targets[0];
            validate_alive_other(tx, actor, target)?;
            if tx.state().disabled_judge_zones.contains(&target) {
                return Err(EffectError::Illegal(
                    "target judgment zone is disabled".into(),
                ));
            }
            let actor_group = tx.state().player_group(actor);
            let target_group = tx.state().player_group(target);
            if actor_group.is_some() && actor_group == target_group {
                return Err(EffectError::Illegal(
                    "远交近攻 requires a different-group target".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::YUANJIAO,
                &[target.raw()],
            );
        }
        CardKind::TrickWangmeizhike => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "wangmeizhike")?;
            let target = targets[0];
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal("望梅止渴 target must be alive".into()));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::WANGMEIZHIKE,
                &[target.raw()],
            );
        }
        CardKind::TrickChuqibuyi => {
            // 出其不意：对一名有手牌的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "chuqibuyi")?;
            let target = targets[0];
            if target == actor
                || !tx.state().players.get(target).is_some_and(|p| p.alive)
                || tx.state().hand_count(target) == 0
            {
                return Err(EffectError::Illegal(
                    "chuqibuyi target must be another alive player with hand cards".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::CHUQIBUYI,
                &[target.raw()],
            );
        }
        CardKind::TrickYiyi => {
            // 以逸待劳（国战）：对己方全体角色使用，各摸两张再弃两张。
            // 己方 = 同势力 或 野心联盟（are_allies）。
            let allies: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    let mine = tx.state().player_group(actor).map(str::to_string);
                    let theirs = tx.state().player_group(*candidate).map(str::to_string);
                    (mine.is_some() && mine == theirs)
                        || crate::mode::are_allies(tx.state(), actor, *candidate)
                })
                .collect();
            if allies.is_empty() {
                return Err(EffectError::Illegal("以逸待劳 has no ally target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &allies,
                trick_effect::YIYI,
                field_source,
            );
        }
        CardKind::TrickShuiyanqijunx => {
            // 水淹七军（国战）：对一名装备区里有牌的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "shuiyanqijunx")?;
            let target = targets[0];
            let has_equip = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| !zone.cards.is_empty());
            if target == actor
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
                || !has_equip
            {
                return Err(EffectError::Illegal(
                    "水淹七军 target must be another living player with equipment".into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[target],
                trick_effect::SHUIYANQIJUNX,
                field_source,
            );
        }
        CardKind::TrickLulitongxin => {
            // 戮力同心（国战）：选一名角色为代表，对与其同为大势力或同为小势力的
            // 全部角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "lulitongxin")?;
            let picked = targets[0];
            if !tx
                .state()
                .players
                .get(picked)
                .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal("戮力同心 target must be alive".into()));
            }
            let major = crate::mode::is_major_faction_player(tx.state(), picked);
            let group: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    crate::mode::is_major_faction_player(tx.state(), *candidate) == major
                })
                .collect();
            if group.is_empty() {
                return Err(EffectError::Illegal("戮力同心 has no target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &group,
                trick_effect::LULITONGXIN,
                field_source,
            );
        }
        CardKind::TrickKefuzhongyuan => {
            // 克复中原（国战）：对任意名其他角色使用，各目标视为使用杀或摸牌。
            let max_targets = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|player| *player != actor)
                .count() as u32;
            validate_independent_trick_target_count(
                tx,
                actor,
                card,
                targets,
                max_targets,
                "kefuzhongyuan",
            )?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::KEFUZHONGYUAN,
                field_source,
            );
        }
        CardKind::TrickGuguoanbang => {
            // 固国安邦（国战）：对你自己使用，摸八弃≥六；吴势力可将弃牌交给其他吴角色。
            if targets != [actor] {
                return Err(EffectError::Illegal(
                    "固国安邦 targets only its user".into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[actor],
                trick_effect::GUGUOANBANG,
                field_source,
            );
        }
        CardKind::TrickHuoshaolianying => {
            // 火烧连营（国战）：对下家及其「队列」（以之为中心的连续同势力座次块）
            // 中所有角色造成 1 点火属性伤害。目标自动计算，无需选择。
            let alive = tx.state().players.alive_by_seat();
            let len = alive.len();
            if len < 2 {
                return Err(EffectError::Illegal("火烧连营 needs at least two players".into()));
            }
            let next_index = alive
                .iter()
                .position(|player| *player == actor)
                .map(|index| (index + 1) % len)
                .unwrap_or(0);
            let next = alive[next_index];
            // 队列：从 next 向两翼扩展连续同势力角色。
            let mut queue = vec![next];
            let mut index = (next_index + 1) % len;
            while index != next_index
                && crate::mode::same_side(tx.state(), next, alive[index])
            {
                queue.push(alive[index]);
                index = (index + 1) % len;
            }
            let mut index = (next_index + len - 1) % len;
            while index != next_index
                && crate::mode::same_side(tx.state(), next, alive[index])
            {
                queue.insert(0, alive[index]);
                index = (index + len - 1) % len;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &queue,
                trick_effect::HUOSHAOLIANYING,
                field_source,
            );
        }
        CardKind::TrickLianjunshengyan => {
            // 联军盛宴（国战）：选一名敌方势力角色为代表，目标 = 使用者自己 + 该
            // 势力全体（同势力/野心联盟）。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "lianjunshengyan")?;
            let picked = targets[0];
            if picked == actor
                || !tx
                    .state()
                    .players
                    .get(picked)
                    .is_some_and(|player| player.alive)
                || crate::mode::same_side(tx.state(), actor, picked)
            {
                return Err(EffectError::Illegal(
                    "联军盛宴 requires a living enemy-faction representative".into(),
                ));
            }
            let mut group = vec![actor, picked];
            for candidate in tx.state().players.alive_by_seat() {
                if candidate != picked
                    && candidate != actor
                    && crate::mode::same_side(tx.state(), picked, candidate)
                {
                    group.push(candidate);
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &group,
                trick_effect::LIANJUNSHENGYAN,
                field_source,
            );
        }
        CardKind::TrickWenheluanwu => {
            // 文和乱武（国战）：对所有其他角色使用，无手牌者自动跳过
            // （上游 ignoreTarget）。目标自动计算，无需选择。
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != actor)
                .filter(|candidate| tx.state().hand_count(*candidate) > 0)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("文和乱武 has no target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &targets,
                trick_effect::WENHELUANWU,
                field_source,
            );
        }
        CardKind::TrickHaolingtianxia => {
            // 号令天下（国战）：对一名体力值非全场最少（上游 isMinHp）的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "haolingtianxia")?;
            let target = targets[0];
            let min_hp = tx
                .state()
                .players
                .alive_by_seat()
                .iter()
                .filter_map(|candidate| tx.state().players.get(*candidate))
                .map(|player| player.hp)
                .min()
                .unwrap_or(i32::MAX);
            if target == actor
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
                || tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.hp <= min_hp)
            {
                return Err(EffectError::Illegal(
                    "号令天下 target must be another living player whose HP exceeds the minimum"
                        .into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[target],
                trick_effect::HAOLINGTIANXIA,
                field_source,
            );
        }
        CardKind::TrickChiling => {
            // 敕令（国战）：对所有有暗置武将（unseen 任一槽）的角色使用——含使用者
            // （上游 filterTarget 无自身排除）。目标自动计算，无需选择。
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|player| player.unseen[0] || player.unseen[1])
                })
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("敕令 has no unseen target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &targets,
                trick_effect::CHILING,
                field_source,
            );
        }
        CardKind::TrickTiaojiyanmei => {
            if targets.len() != 2
                || targets[0] == targets[1]
                || targets.contains(&actor)
                || targets.iter().any(|target| {
                    !tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|player| player.alive)
                })
                || tx.state().hand_count(targets[0]) == tx.state().hand_count(targets[1])
            {
                return Err(EffectError::Illegal(
                    "调剂盐梅 requires two living targets with different hand counts".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::TIAOJIYANMEI,
                &[targets[0].raw(), targets[1].raw()],
            );
        }
        CardKind::TrickMantianguohai => {
            if targets.is_empty()
                || targets.len() > 2
                || targets.iter().any(|target| {
                    *target == actor
                        || !tx
                            .state()
                            .players
                            .get(*target)
                            .is_some_and(|player| player.alive)
                        || crate::query::gainable_target_cards(
                            tx.state(),
                            actor,
                            *target,
                            rule_api::TargetCardScope::HandEquipJudge,
                        )
                        .is_empty()
                })
                || targets.len() == 2 && targets[0] == targets[1]
            {
                return Err(EffectError::Illegal(
                    "瞒天过海 requires one or two distinct living other targets with gainable cards"
                        .into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::MANTIANGUOHAI,
                field_source,
            );
        }
        CardKind::TrickNanman => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let others: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|p| *p != actor)
                .collect();
            let mut args = vec![others.len() as u32];
            args.extend(others.iter().map(|p| p.raw()));
            return use_card_then_wuxie(tx, actor, card, &others, trick_effect::NANMAN, &args);
        }
        CardKind::TrickWanjian => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let others: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|p| *p != actor)
                .collect();
            let mut args = vec![others.len() as u32];
            args.extend(others.iter().map(|p| p.raw()));
            return use_card_then_wuxie(tx, actor, card, &others, trick_effect::WANJIAN, &args);
        }
        CardKind::TrickTaoyuan => {
            let self_only = crate::query::card_targets_self_only_for(tx.state(), actor);
            let alive = if self_only {
                vec![actor]
            } else {
                tx.state().players.alive_by_seat()
            };
            let args = self_only.then(|| vec![1, actor.raw()]).unwrap_or_default();
            return use_card_then_wuxie(tx, actor, card, &alive, trick_effect::TAOYUAN, &args);
        }
        CardKind::TrickWugu => {
            let self_only = crate::query::card_targets_self_only_for(tx.state(), actor);
            let alive = if self_only {
                vec![actor]
            } else {
                tx.state().players.alive_by_seat()
            };
            return run_effects(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: alive.clone(),
                    },
                    EffectRequest::ContinueWugu {
                        action: 0,
                        actor,
                        source_card: card,
                        targets: alive,
                        excluded: Vec::new(),
                        revealed: Vec::new(),
                    },
                ],
            )
            .map(|_| ());
        }
        CardKind::TrickDiaobingqianjiang => {
            let mut alive = tx.state().players.alive_by_seat();
            if let Some(start) = alive.iter().position(|player| *player == actor) {
                alive.rotate_left(start);
            }
            alive.retain(|target| *target == actor || tx.state().hand_count(*target) > 0);
            return run_effects(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: alive.clone(),
                    },
                    EffectRequest::ResolveWdbYangyi {
                        player: actor,
                        target: PlayerId::NONE,
                        card,
                        action: 2,
                        skill: engine_api::SkillId::NONE,
                    },
                    EffectRequest::ContinueDiaobingqianjiang {
                        action: 0,
                        actor,
                        source_card: card,
                        targets: alive,
                        revealed: Vec::new(),
                    },
                ],
            )
            .map(|_| ());
        }
        CardKind::TrickJiedao => {
            // 借刀: targets [weapon_holder, victim]
            if targets.len() != 2 {
                return Err(EffectError::Illegal(
                    "jiedao needs holder and victim".into(),
                ));
            }
            let holder = targets[0];
            let victim = targets[1];
            validate_alive_other(tx, actor, holder)?;
            if victim == holder || victim == actor {
                return Err(EffectError::Illegal("bad jiedao victim".into()));
            }
            let v = tx
                .state()
                .players
                .get(victim)
                .ok_or_else(|| EffectError::Illegal("bad victim".into()))?;
            if !v.alive {
                return Err(EffectError::Illegal("victim dead".into()));
            }
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, holder)
                .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
            let has_weapon = tx
                .state()
                .zones
                .get(equip)
                .map(|z| {
                    z.cards.iter().any(|cid| {
                        tx.state()
                            .cards
                            .get(*cid)
                            .map(|c| c.kind == CardKind::Weapon)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if !has_weapon {
                return Err(EffectError::Illegal("holder has no weapon".into()));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                &[holder, victim],
                trick_effect::JIEDAO,
                &[holder.raw(), victim.raw()],
            );
        }
        CardKind::TrickTiesuo => {
            // 0 targets: 重铸 — discard and draw 1 (not nullifiable as "use effect")
            // 1–2 targets: toggle chain — goes through 无懈
            let limit = crate::query::card_target_limit_for_card(tx.state(), actor, card, 2);
            if targets.len() > limit as usize {
                return Err(EffectError::Illegal(format!(
                    "tiesuo at most {limit} targets"
                )));
            }
            ensure_distinct_targets(targets)?;
            if targets.is_empty() {
                queue.push(EffectRequest::UseCard {
                    player: actor,
                    card,
                    targets: vec![],
                });
                queue.push(EffectRequest::DrawCards {
                    player: actor,
                    count: 1,
                });
            } else {
                for &t in targets {
                    if tx.state().players.get(t).map(|p| !p.alive).unwrap_or(true) {
                        return Err(EffectError::Illegal("tiesuo target dead".into()));
                    }
                }
                let mut args = vec![TIESUO_SAVED_TARGETS_MAGIC, card.raw(), targets.len() as u32];
                args.extend(targets.iter().map(|t| t.raw()));
                return run_effects_then_wuxie(
                    tx,
                    vec![
                        EffectRequest::UseCard {
                            player: actor,
                            card,
                            targets: targets.to_vec(),
                        },
                        EffectRequest::EmitRuleEvent {
                            code: 52,
                            payload: {
                                let mut v = vec![actor.raw(), card.raw()];
                                v.extend(targets.iter().map(|t| t.raw()));
                                v
                            },
                        },
                    ],
                    actor,
                    trick_effect::TIESUO,
                    actor,
                    &args,
                );
            }
        }
        CardKind::TrickLe | CardKind::TrickBingliang => {
            if targets.len() != 1 {
                return Err(EffectError::Illegal(
                    "delayed trick needs one target".into(),
                ));
            }
            let target = targets[0];
            validate_alive_other(tx, actor, target)?;
            if kind == CardKind::TrickBingliang
                && !ignore_distance
                && !crate::query::card_target_within_distance(tx.state(), actor, card, target, 1)
            {
                return Err(EffectError::Illegal("target out of 兵粮寸断 range".into()));
            }
            if kind == CardKind::TrickLe
                && crate::query::target_immune(
                    tx.state(),
                    target,
                    crate::query::ModifierTarget::ImmuneLe,
                )
            {
                return Err(EffectError::Illegal(
                    "target immune to 乐不思蜀 (谦逊)".into(),
                ));
            }
            if has_delayed_of_kind(tx, target, kind) {
                return Err(EffectError::Illegal(
                    "target already has this delayed".into(),
                ));
            }
            // Leave hand into discard first; after 无懈, move into judge (or stay discarded if nullified).
            return run_effects_then_wuxie(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: vec![target],
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 51, // delayed used
                        payload: vec![actor.raw(), card.raw(), target.raw(), kind.tag() as u32],
                    },
                ],
                actor,
                trick_effect::DELAYED_PLACE,
                actor,
                &[card.raw(), target.raw(), kind.tag() as u32],
            );
        }
        CardKind::TrickShandian => {
            if tx.state().disabled_judge_zones.contains(&actor) {
                return Err(EffectError::Illegal("judgment zone is disabled".into()));
            }
            if has_delayed_of_kind(tx, actor, CardKind::TrickShandian) {
                return Err(EffectError::Illegal("already has lightning".into()));
            }
            return run_effects_then_wuxie(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: vec![actor],
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 51,
                        payload: vec![actor.raw(), card.raw(), actor.raw(), kind.tag() as u32],
                    },
                ],
                actor,
                trick_effect::DELAYED_PLACE,
                actor,
                &[card.raw(), actor.raw(), kind.tag() as u32],
            );
        }
        CardKind::Other => {
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
        }
    }

    run_effects(tx, queue).map(|_| ())
}

/// Target shape for a card used from a resolving skill rather than a PlayCard command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkillCardTargetSpec {
    /// No choice is needed; use these targets.
    Immediate(Vec<PlayerId>),
    /// Select distinct players from `options`.
    Select {
        min: u8,
        max: u8,
        options: Vec<PlayerId>,
    },
    /// Select one prevalidated ordered target pair (借刀杀人).
    Pairs(Vec<(PlayerId, PlayerId)>),
}

/// Proactive ordinary tricks that can currently be used source-free by `actor`.
/// `used_mask` stores permanent CardKind-tag bits for skills such as 3D精选【奋计】.
pub(crate) fn source_free_ordinary_trick_kinds(
    state: &crate::state::GameState,
    actor: PlayerId,
    used_mask: u64,
) -> Vec<CardKind> {
    let mut probe = Transaction::begin(state.clone());
    let Some(processing) = probe
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
    else {
        return Vec::new();
    };
    let mut candidates: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.def.is_some())
        .map(|card| card.kind)
        .filter(|kind| {
            kind.is_trick()
                && !kind.is_delayed_trick()
                && !matches!(kind, CardKind::TrickWuxie | CardKind::TrickYoudishenru)
                && used_mask & (1u64 << kind.tag()) == 0
        })
        .collect();
    candidates.sort_by_key(|kind| kind.tag());
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|kind| {
            let card = probe.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: *kind,
                suit: engine_api::Suit::None,
                rank: 0,
                weapon_range: 0,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            probe
                .state_mut()
                .zones
                .get_mut(processing)
                .expect("validated processing zone")
                .cards
                .push(card);
            skill_card_target_spec(&probe, actor, card).is_ok()
        })
        .collect()
}

/// Determine whether a revealed card has a legal use and which target choice it needs.
pub(crate) fn skill_card_target_spec(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<SkillCardTargetSpec, EffectError> {
    skill_card_target_spec_with_options(tx, actor, card, false, false)
}

/// Resolve the ordinary target shape for a hand card used by a skill, optionally ignoring
/// distance while preserving every other target and use-count rule.
pub(crate) fn skill_card_target_spec_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    ignore_distance: bool,
    ignore_use_count: bool,
) -> Result<SkillCardTargetSpec, EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?;
    if !player.alive || player.dying {
        return Err(EffectError::Illegal("actor not able to play".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor);
    let equip = tx.state().zones.find(ZoneKind::Equip, actor);
    let playable_as_hand = tx.state().playable_hand_cards(actor).contains(&card);
    let view_as_material =
        tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32;
    if !((card_state.zone == processing && card_state.owner.is_none())
        || (hand == Some(card_state.zone) && card_state.owner == actor)
        || (view_as_material && equip == Some(card_state.zone) && card_state.owner == actor)
        || playable_as_hand)
    {
        return Err(EffectError::Illegal(
            "skill card not in processing or user hand".into(),
        ));
    }
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    let alive = tx.state().players.alive_by_seat();
    let target_allowed = |target: PlayerId| {
        validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
    };
    let select_one = |options: Vec<PlayerId>| {
        if options.is_empty() {
            Err(EffectError::Illegal("card has no legal target".into()))
        } else {
            Ok(SkillCardTargetSpec::Select {
                min: 1,
                max: 1,
                options,
            })
        }
    };
    let select_independent = |options: Vec<PlayerId>| {
        if options.is_empty() {
            return Err(EffectError::Illegal("card has no legal target".into()));
        }
        let max = crate::query::card_target_limit_for_card(tx.state(), actor, card, 1)
            .min(options.len() as u32)
            .min(u32::from(u8::MAX)) as u8;
        if max == 0 {
            Err(EffectError::Illegal("card has no legal target".into()))
        } else {
            Ok(SkillCardTargetSpec::Select {
                min: 1,
                max,
                options,
            })
        }
    };

    match kind {
        CardKind::Ying => Err(EffectError::Illegal("shadow card cannot be played".into())),
        CardKind::Shan | CardKind::TrickWuxie => {
            Err(EffectError::Illegal("response-only card".into()))
        }
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let within_use_count = ignore_use_count
                || crate::query::sha_play_phase_use_count(tx.state())
                    < crate::query::sha_use_limit_for_card(tx.state(), actor, card);
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| {
                    within_use_count
                        || crate::query::sha_use_unlimited_for_target(tx.state(), actor, *target)
                })
                .filter(|target| {
                    validate_sha_target_with_options(tx, actor, card, *target, ignore_distance)
                        .is_ok()
                })
                .collect();
            if crate::query::sha_targets_all_for(tx.state(), actor) {
                if options.is_empty() {
                    Err(EffectError::Illegal("sha has no legal target".into()))
                } else {
                    Ok(SkillCardTargetSpec::Immediate(options))
                }
            } else {
                let max = usize::try_from(crate::query::sha_target_limit_for(tx.state(), actor))
                    .unwrap_or(usize::MAX)
                    .min(options.len())
                    .min(u8::MAX as usize) as u8;
                if max == 0 {
                    Err(EffectError::Illegal("sha has no legal target".into()))
                } else {
                    Ok(SkillCardTargetSpec::Select {
                        min: 1,
                        max,
                        options,
                    })
                }
            }
        }
        CardKind::Tao => {
            if player.hp >= player.max_hp as i32 {
                return Err(EffectError::Illegal("hp full".into()));
            }
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::Jiu => {
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure
        | CardKind::TrickWuzhong
        | CardKind::TrickYoudishenru
        | CardKind::Other => Ok(SkillCardTargetSpec::Immediate(Vec::new())),
        CardKind::TrickGuohe => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    !discardable_target_zone_card_tokens(tx, actor, *target).is_empty()
                })
                .collect(),
        ),
        CardKind::TrickShunshou => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    ignore_distance
                        || crate::query::card_target_within_distance(
                            tx.state(),
                            actor,
                            card,
                            *target,
                            1,
                        )
                })
                .filter(|target| {
                    !crate::query::target_immune(
                        tx.state(),
                        *target,
                        crate::query::ModifierTarget::ImmuneShunshou,
                    )
                })
                .filter(|target| {
                    !target_zone_card_tokens(tx, *target, true, true, false).is_empty()
                })
                .collect(),
        ),
        CardKind::TrickHuogong => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .filter(|target| !tx.state().hand_cards(*target).is_empty())
                .collect(),
        ),
        CardKind::TrickJuedou => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickYuanjiao => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    let actor_group = tx.state().player_group(actor);
                    let target_group = tx.state().player_group(*target);
                    actor_group.is_none() || actor_group != target_group
                })
                .collect(),
        ),
        CardKind::TrickWangmeizhike => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickChuqibuyi => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| {
                    *target != actor
                        && target_allowed(*target)
                        && tx.state().hand_count(*target) > 0
                })
                .collect(),
        ),
        CardKind::TrickYiyi => {
            // 以逸待劳（国战）：自动目标 = 己方全体（同势力或野心联盟，含自己）。
            let allies: Vec<_> = alive
                .iter()
                .copied()
                .filter(|candidate| {
                    let mine = tx.state().player_group(actor).map(str::to_string);
                    let theirs = tx.state().player_group(*candidate).map(str::to_string);
                    (mine.is_some() && mine == theirs)
                        || crate::mode::are_allies(tx.state(), actor, *candidate)
                })
                .collect();
            if allies.is_empty() {
                Err(EffectError::Illegal("以逸待劳 has no ally target".into()))
            } else {
                Ok(SkillCardTargetSpec::Immediate(allies))
            }
        }
        CardKind::TrickShuiyanqijunx => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    tx.state()
                        .zones
                        .find(crate::state::ZoneKind::Equip, *target)
                        .and_then(|zone| tx.state().zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty())
                })
                .collect(),
        ),
        CardKind::TrickLulitongxin => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickKefuzhongyuan => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect();
            if options.is_empty() {
                Err(EffectError::Illegal("克复中原 has no target".into()))
            } else {
                Ok(SkillCardTargetSpec::Select {
                    min: 1,
                    max: options.len().min(u8::MAX as usize) as u8,
                    options,
                })
            }
        }
        CardKind::TrickGuguoanbang => {
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::TrickHuoshaolianying => {
            // 火烧连营（国战）：目标自动 = 下家及其队列，无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickLianjunshengyan => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| !crate::mode::same_side(tx.state(), actor, *target))
                .collect(),
        ),
        CardKind::TrickWenheluanwu => {
            // 文和乱武（国战）：目标自动 = 除自己外全部有手牌角色，无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickHaolingtianxia => {
            // 号令天下（国战）：选一名体力值非全场最少的其他角色。
            let min_hp = alive
                .iter()
                .filter_map(|candidate| tx.state().players.get(*candidate))
                .map(|player| player.hp)
                .min()
                .unwrap_or(i32::MAX);
            select_one(
                alive
                    .iter()
                    .copied()
                    .filter(|target| *target != actor)
                    .filter(|target| target_allowed(*target))
                    .filter(|target| {
                        tx.state()
                            .players
                            .get(*target)
                            .is_some_and(|player| player.hp > min_hp)
                    })
                    .collect(),
            )
        }
        CardKind::TrickChiling => {
            // 敕令（国战）：目标自动 = 所有有暗置武将的角色（含使用者），无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickTiaojiyanmei => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .collect();
            if options.iter().any(|left| {
                options.iter().any(|right| {
                    left != right && tx.state().hand_count(*left) != tx.state().hand_count(*right)
                })
            }) {
                Ok(SkillCardTargetSpec::Select {
                    min: 2,
                    max: 2,
                    options,
                })
            } else {
                Err(EffectError::Illegal(
                    "调剂盐梅 has no legal target pair".into(),
                ))
            }
        }
        CardKind::TrickMantianguohai => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    !crate::query::gainable_target_cards(
                        tx.state(),
                        actor,
                        *target,
                        rule_api::TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                Err(EffectError::Illegal("瞒天过海 has no legal target".into()))
            } else {
                Ok(SkillCardTargetSpec::Select {
                    min: 1,
                    max: 2,
                    options,
                })
            }
        }
        CardKind::TrickNanman | CardKind::TrickWanjian => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let targets: Vec<_> = alive
                .into_iter()
                .filter(|target| *target != actor)
                .collect();
            validate_explicit_target_policy(tx, actor, card_state, kind, &targets)?;
            Ok(SkillCardTargetSpec::Immediate(targets))
        }
        CardKind::TrickTaoyuan | CardKind::TrickWugu | CardKind::TrickDiaobingqianjiang => {
            let targets = if crate::query::card_targets_self_only_for(tx.state(), actor) {
                vec![actor]
            } else {
                let mut targets = alive;
                if kind == CardKind::TrickDiaobingqianjiang {
                    if let Some(start) = targets.iter().position(|player| *player == actor) {
                        targets.rotate_left(start);
                    }
                    targets.retain(|target| *target == actor || tx.state().hand_count(*target) > 0);
                }
                targets
            };
            validate_explicit_target_policy(tx, actor, card_state, kind, &targets)?;
            Ok(SkillCardTargetSpec::Immediate(targets))
        }
        CardKind::TrickJiedao => {
            let has_weapon = |holder: PlayerId| {
                tx.state()
                    .zones
                    .find(ZoneKind::Equip, holder)
                    .and_then(|zone| tx.state().zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.kind == CardKind::Weapon)
                        })
                    })
            };
            let mut pairs = Vec::new();
            for holder in alive.iter().copied().filter(|holder| {
                validate_alive_other(tx, actor, *holder).is_ok() && has_weapon(*holder)
            }) {
                for victim in alive
                    .iter()
                    .copied()
                    .filter(|victim| *victim != actor && *victim != holder)
                {
                    if validate_explicit_target_policy(
                        tx,
                        actor,
                        card_state,
                        kind,
                        &[holder, victim],
                    )
                    .is_ok()
                    {
                        pairs.push((holder, victim));
                    }
                }
            }
            if pairs.is_empty() {
                Err(EffectError::Illegal("jiedao has no legal targets".into()))
            } else {
                Ok(SkillCardTargetSpec::Pairs(pairs))
            }
        }
        CardKind::TrickTiesuo => {
            let options = alive
                .into_iter()
                .filter(|target| target_allowed(*target))
                .collect::<Vec<_>>();
            Ok(SkillCardTargetSpec::Select {
                min: 0,
                max: options
                    .len()
                    .min(
                        crate::query::card_target_limit_for_card(tx.state(), actor, card, 2)
                            as usize,
                    )
                    .min(u8::MAX as usize) as u8,
                options,
            })
        }
        CardKind::TrickLe | CardKind::TrickBingliang => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    ignore_distance
                        || kind != CardKind::TrickBingliang
                        || crate::query::card_target_within_distance(
                            tx.state(),
                            actor,
                            card,
                            *target,
                            1,
                        )
                })
                .filter(|target| {
                    kind != CardKind::TrickLe
                        || !crate::query::target_immune(
                            tx.state(),
                            *target,
                            crate::query::ModifierTarget::ImmuneLe,
                        )
                })
                .filter(|target| !tx.state().disabled_judge_zones.contains(target))
                .filter(|target| !has_delayed_of_kind(tx, *target, kind))
                .collect(),
        ),
        CardKind::TrickShandian => {
            if tx.state().disabled_judge_zones.contains(&actor) {
                return Err(EffectError::Illegal("judgment zone is disabled".into()));
            }
            if has_delayed_of_kind(tx, actor, kind) {
                return Err(EffectError::Illegal("already has lightning".into()));
            }
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
    }
}

/// Use a shared-processing card through the ordinary card-resolution pipeline.
pub(crate) fn handle_skill_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    handle_skill_card_with_options(tx, actor, card, targets, false)
}

/// Use a shared-processing card while optionally ignoring ordinary distance.
pub(crate) fn handle_skill_card_with_options(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
) -> Result<(), EffectError> {
    handle_skill_card_with_policies(tx, actor, card, targets, ignore_distance, false)
}

/// Use a shared-processing card with explicit distance and use-count policies.
pub(crate) fn handle_skill_card_with_policies(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad skill card".into()))?;
    if card_state.zone != processing || card_state.owner.is_some() {
        return Err(EffectError::Illegal(
            "skill card not in shared processing zone".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, actor, false)?;
    handle_card_from_hand_with_policies(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        false,
    )
}

/// Use a materialized virtual Sha with explicit use policies and a fixed base-damage bonus.
pub(crate) fn handle_virtual_sha_with_policies_and_bonus(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    damage_bonus: u32,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, actor, false)?;
    play_as_kind_with_extra_damage_flags(
        tx,
        actor,
        card,
        CardKind::Sha,
        targets,
        ignore_distance,
        false,
        &[],
        ignore_use_count,
        false,
        0,
        &[],
        damage_bonus,
    )
}

/// Play a hand card as a different kind (view-as). Consumes the source card.
pub(crate) fn play_as_kind(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    play_as_kind_with_extra_damage_flags(
        tx,
        actor,
        card,
        as_kind,
        targets,
        ignore_distance,
        force_unrespondable,
        &[],
        ignore_use_count,
        source_was_hand,
        0,
        &[],
        0,
    )
}

/// Use a physical equipment/judgment card as an independent trick without first gaining it.
pub(crate) fn play_field_card_as_kind(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !matches!(
        as_kind,
        CardKind::TrickGuohe
            | CardKind::TrickShunshou
            | CardKind::TrickHuogong
            | CardKind::TrickJuedou
    ) {
        return Err(EffectError::ContentFault(
            "field view-as only supports independent tricks".into(),
        ));
    }
    let source = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad field card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &source, as_kind) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.kind = as_kind;
    }
    handle_card_with_policies_and_flags(tx, actor, card, targets, true, true, false, 0, true)
}

/// Play a view-as card while attaching engine-owned post-use damage flags.
pub(crate) fn play_as_kind_with_extra_damage_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    forced_unrespondable_targets: &[PlayerId],
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    extra_cards: &[CardId],
    multi_card_bonus: u32,
) -> Result<(), EffectError> {
    play_as_kind_with_extra_damage_flags_target_bonus(
        tx,
        actor,
        card,
        as_kind,
        targets,
        ignore_distance,
        force_unrespondable,
        forced_unrespondable_targets,
        ignore_use_count,
        source_was_hand,
        extra_damage_flags,
        extra_cards,
        multi_card_bonus,
        0,
    )
}

/// Play a view-as card with one content-authorized target-limit bonus.
pub(crate) fn play_as_kind_with_extra_damage_flags_target_bonus(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    forced_unrespondable_targets: &[PlayerId],
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    extra_cards: &[CardId],
    multi_card_bonus: u32,
    target_limit_bonus: u8,
) -> Result<(), EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?
        .clone();
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &source_card, as_kind) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &source_card, as_kind, targets)?;
    let converted_use_unlimited = extra_cards.is_empty()
        && crate::query::card_use_unlimited_for_logical_kind(tx.state(), actor, card, as_kind);
    remember_hand_card_suit_use(tx, actor, card, source_was_hand);
    let mut queue: Vec<EffectRequest> = Vec::new();
    if !extra_cards.is_empty() {
        let hand = tx.state().zones.find(ZoneKind::Hand, actor);
        let mut source_cards = Vec::with_capacity(extra_cards.len() + 1);
        source_cards.push(card);
        source_cards.extend_from_slice(extra_cards);
        let all_sources_were_hand = source_was_hand
            && extra_cards.iter().all(|source| {
                tx.state().cards.get(*source).is_some_and(|source_card| {
                    Some(source_card.zone) == hand
                        || crate::query::borrowed_hand_source(tx.state(), actor, *source).is_some()
                })
            });
        tx.state_mut().resolution.plan_card_source_cards(
            actor,
            card,
            source_cards,
            all_sources_were_hand,
        );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
        let refills = crate::effect::pothaoshi_refill_effects(tx.state(), actor, extra_cards);
        for extra in extra_cards {
            let from = tx
                .state()
                .cards
                .get(*extra)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("view-as source disappeared".into()))?;
            queue.push(EffectRequest::MoveCardsWithReason {
                cards: vec![*extra],
                to: discard,
                from: Some(from),
                reason: CardMoveReason::Use,
            });
        }
        queue.extend(refills);
    }
    match as_kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let ignores_use_count = ignore_use_count
                || crate::query::sha_use_does_not_count_for_card(tx.state(), actor, card)
                || converted_use_unlimited;
            let respondable = !force_unrespondable
                && !crate::query::sha_unrespondable_for_logical_kind(
                    tx.state(),
                    actor,
                    &source_card,
                    as_kind,
                );
            if !ignores_use_count && !sha_use_count_allows_targets(tx.state(), actor, card, targets)
            {
                return Err(EffectError::Illegal("sha use limit reached".into()));
            }
            validate_sha_targets_with_options_target_bonus(
                tx,
                actor,
                card,
                targets,
                ignore_distance,
                target_limit_bonus,
            )?;
            let mut amount = 1u32.saturating_add(multi_card_bonus);
            let jiu = player
                .marks
                .iter()
                .any(|(m, c)| *m == crate::content::MARK_JIU && *c > 0);
            let card_flags = (if jiu {
                rule_api::damage_card_flag::JIU_BOOSTED
            } else {
                0
            }) | extra_damage_flags;
            if jiu {
                amount = amount.saturating_add(1);
                queue.push(EffectRequest::RemoveMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    amount: 1,
                });
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
            // Proactive 杀 (view-as) during own turn counts for 克己.
            if tx.state().turn.current == actor && !ignores_use_count {
                record_counted_sha_use(tx);
            }
            let counted = tx.state().turn.current == actor && !ignores_use_count;
            let resume = if targets.len() == 1 {
                let resume_flags = saved_sha_flags(
                    tx,
                    respondable && !forced_unrespondable_targets.contains(&targets[0]),
                    counted,
                );
                (
                    crate::state::ContinuationKind::ResumePlayedSha,
                    vec![
                        actor.raw(),
                        targets[0].raw(),
                        amount,
                        pack_damage_card_context(as_kind.sha_nature(), card_flags),
                        resume_flags,
                        card.raw(),
                    ],
                )
            } else {
                let resume_flags = saved_sha_flags(tx, respondable, counted);
                let mut payload = vec![
                    actor.raw(),
                    amount,
                    pack_damage_card_context(as_kind.sha_nature(), card_flags),
                    resume_flags,
                    card.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(u32::MAX);
                payload.extend(
                    targets
                        .iter()
                        .map(|target| u32::from(!forced_unrespondable_targets.contains(target))),
                );
                (
                    crate::state::ContinuationKind::ResumePlayedShaQueue,
                    payload,
                )
            };
            if !extra_cards.is_empty() {
                let mut source_cards = Vec::with_capacity(extra_cards.len() + 1);
                source_cards.push(card);
                source_cards.extend_from_slice(extra_cards);
                let resolution = &mut tx.state_mut().resolution;
                resolution.register_sha_resolution(actor, card, targets, counted);
                debug_assert!(resolution.register_sha_source_cards(actor, card, &source_cards));
            }
            let (kind, payload) = run_effects_with_saved_resume(tx, queue, resume.0, resume.1)?;
            return resume_played_sha_from_payload(tx, kind, &payload);
        }
        CardKind::Tao => {
            if crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                actor,
                &source_card,
                CardKind::Tao,
            ) {
                return Err(EffectError::Illegal("tao target disabled".into()));
            }
            if player.hp >= player.max_hp as i32 {
                return Err(EffectError::Illegal("hp full".into()));
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            queue.push(EffectRequest::RecoverHp {
                source: actor,
                player: actor,
                amount: 1u32.saturating_add(multi_card_bonus),
            });
        }
        CardKind::Jiu => {
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            if player.hp < player.max_hp as i32 {
                queue.push(EffectRequest::RecoverHp {
                    source: actor,
                    player: actor,
                    amount: 1,
                });
            } else {
                queue.push(EffectRequest::AddMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                });
            }
        }
        CardKind::Shan => {
            return Err(EffectError::Illegal(
                "view_as shan not usable proactively (no response phase yet)".into(),
            ));
        }
        // Trick view-as (奇袭 → 过河拆桥, 国色 → 乐不思蜀, …): retag the source
        // card to the trick kind and delegate to the full play pipeline, reusing
        // its target validation and wuxie windows. No effect logic duplicated.
        CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickWuzhong
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickHuogong
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickWangmeizhike
        | CardKind::TrickDiaobingqianjiang => {
            run_effects(tx, queue)?;
            if let Some(c) = tx.state_mut().cards.get_mut(card) {
                c.kind = as_kind;
            }
            return handle_card_from_hand_with_options(
                tx,
                actor,
                card,
                targets,
                ignore_distance,
                source_was_hand,
            );
        }
        _ => {
            return Err(EffectError::Illegal("unsupported view_as kind".into()));
        }
    }
    run_effects(tx, queue).map(|_| ())
}

pub(super) fn validate_alive_other(
    tx: &Transaction,
    actor: PlayerId,
    target: PlayerId,
) -> Result<(), EffectError> {
    if target == actor {
        return Err(EffectError::Illegal("cannot target self".into()));
    }
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !t.alive {
        return Err(EffectError::Illegal("target dead".into()));
    }
    Ok(())
}

/// Legal targets when one physical card is used as a distance-unlimited Bingliang.
pub(crate) fn view_as_bingliang_targets(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some(actor_state) = tx.state().players.get(actor) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    if !actor_state.alive
        || actor_state.dying
        || crate::query::card_use_cancelled_for_logical_kind(
            tx.state(),
            actor,
            card_state,
            CardKind::TrickBingliang,
        )
    {
        return Vec::new();
    }
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
        .filter(|target| {
            validate_explicit_target_policy(
                tx,
                actor,
                card_state,
                CardKind::TrickBingliang,
                &[*target],
            )
            .is_ok()
        })
        .filter(|target| !has_delayed_of_kind(tx, *target, CardKind::TrickBingliang))
        .collect()
}

pub(super) fn validate_sha_target(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    validate_sha_target_with_options(tx, actor, card, target, false)
}

/// Validate a Sha target while preserving every policy except ordinary range.
pub(crate) fn validate_sha_target_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    ignore_distance: bool,
) -> Result<(), EffectError> {
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !t.alive {
        return Err(EffectError::Illegal("target dead".into()));
    }
    if target == actor {
        return Err(EffectError::Illegal("cannot sha self".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
    if crate::query::card_target_cancelled_for_logical_kind(
        tx.state(),
        actor,
        target,
        card_state,
        CardKind::Sha,
    ) {
        return Err(EffectError::Illegal("sha target disabled".into()));
    }
    if ignore_distance
        || crate::query::sha_distance_unlimited_for(tx.state(), actor)
        || crate::query::card_distance_unlimited_for_target(tx.state(), actor, card, target)
    {
        return Ok(());
    }
    let mut ctx = QueryContext::new();
    let dist = query(
        tx.state(),
        &QueryRequest::Distance {
            from: actor,
            to: target,
        },
        &mut ctx,
    );
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: actor },
        &mut ctx,
    );
    match (dist, range) {
        (QueryResponse::Int(d), QueryResponse::Int(r)) => {
            if d > r {
                return Err(EffectError::Illegal("target out of range".into()));
            }
        }
        _ => return Err(EffectError::Illegal("distance query failed".into())),
    }
    Ok(())
}

/// Whether one exact physical hand card can be used by `actor` so that `target` is affected,
/// ignoring only ordinary distance. Used by XIANDING 英谋's forced serial card uses.
pub(crate) fn physical_damage_card_can_target_without_distance(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> bool {
    if actor == target
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive && !participant.dying)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().hand_cards(actor).contains(&card)
    {
        return false;
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return false;
    };
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return false;
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            sha_use_count_allows_targets(tx.state(), actor, card, &[target])
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
                && validate_sha_target_with_options(tx, actor, card, target, true).is_ok()
        }
        CardKind::TrickJuedou => {
            crate::query::card_target_limit_for_card(tx.state(), actor, card, 1) > 0
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
        }
        CardKind::TrickHuogong => {
            crate::query::card_target_limit_for_card(tx.state(), actor, card, 1) > 0
                && tx.state().hand_count(target) > 0
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
        }
        CardKind::TrickNanman | CardKind::TrickWanjian => {
            !crate::query::card_targets_self_only_for(tx.state(), actor)
        }
        _ => false,
    }
}

fn validate_sha_targets_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
) -> Result<(), EffectError> {
    validate_sha_targets_with_options_target_bonus(tx, actor, card, targets, ignore_distance, 0)
}

fn validate_sha_targets_with_options_target_bonus(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    target_limit_bonus: u8,
) -> Result<(), EffectError> {
    if crate::query::sha_targets_all_for(tx.state(), actor) {
        let expected: Vec<PlayerId> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != actor)
            .filter(|target| {
                validate_sha_target_with_options(tx, actor, card, *target, ignore_distance).is_ok()
            })
            .collect();
        if targets != expected {
            return Err(EffectError::Illegal(
                "sha must target every player in range".into(),
            ));
        }
        return Ok(());
    }
    let limit = crate::query::sha_target_limit_for(tx.state(), actor)
        .saturating_add(u32::from(target_limit_bonus)) as usize;
    if targets.is_empty() || targets.len() > limit {
        return Err(EffectError::Illegal(format!(
            "sha needs 1 to {limit} targets"
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for target in targets {
        if !seen.insert(*target) {
            return Err(EffectError::Illegal("duplicate sha target".into()));
        }
        validate_sha_target_with_options(tx, actor, card, *target, ignore_distance)?;
    }
    Ok(())
}

/// Validate a source-free logical Sha before asking another player to provide its physical card.
pub(crate) fn validate_source_free_sha_use(
    tx: &Transaction,
    actor: PlayerId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return Err(EffectError::Illegal(
            "source-free Sha user is not able to play".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no source-free Sha user hand".into()))?;
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &card, CardKind::Sha) {
        return Err(EffectError::Illegal("source-free Sha use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &card, CardKind::Sha, targets)?;
    let target_legal = |target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
            && target != actor
            && !crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                target,
                &card,
                CardKind::Sha,
            )
            && (crate::query::sha_distance_unlimited_for(tx.state(), actor)
                || crate::query::card_distance_unlimited_for_target(
                    tx.state(),
                    actor,
                    CardId::NONE,
                    target,
                )
                || crate::query::player_in_attack_range(tx.state(), actor, target))
    };
    if crate::query::sha_targets_all_for(tx.state(), actor) {
        let expected: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| target_legal(*target))
            .collect();
        if targets != expected {
            return Err(EffectError::Illegal(
                "sha must target every player in range".into(),
            ));
        }
    } else {
        let limit = crate::query::sha_target_limit_for(tx.state(), actor) as usize;
        if targets.is_empty() || targets.len() > limit {
            return Err(EffectError::Illegal(format!(
                "sha needs 1 to {limit} targets"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        if targets
            .iter()
            .any(|target| !seen.insert(*target) || !target_legal(*target))
        {
            return Err(EffectError::Illegal(
                "invalid source-free Sha target".into(),
            ));
        }
    }
    if crate::query::sha_play_phase_use_count(tx.state())
        >= crate::query::sha_use_limit_for(tx.state(), actor)
        && !targets
            .iter()
            .all(|target| crate::query::sha_use_unlimited_for_target(tx.state(), actor, *target))
    {
        return Err(EffectError::Illegal("sha use limit reached".into()));
    }
    Ok(())
}

fn validate_explicit_target_policy(
    tx: &Transaction,
    actor: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if logical_kind.is_sha()
        && !crate::query::luanqun_sha_targets_allowed(tx.state(), actor, targets)
    {
        return Err(EffectError::Illegal(
            "first Sha after Luanqun may only target its owners".into(),
        ));
    }
    if crate::query::card_targets_self_only_for(tx.state(), actor)
        && targets.iter().any(|target| *target != actor)
    {
        return Err(EffectError::Illegal("card may only target its user".into()));
    }
    if targets.iter().any(|target| {
        crate::query::card_target_cancelled_for_logical_kind(
            tx.state(),
            actor,
            *target,
            card,
            logical_kind,
        )
    }) {
        return Err(EffectError::Illegal("card target disabled".into()));
    }
    Ok(())
}

pub(crate) fn validate_source_free_card_on_target(
    tx: &Transaction,
    actor: PlayerId,
    kind: CardKind,
    target: PlayerId,
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return Err(EffectError::Illegal(
            "source-free card user is not able to play".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no source-free card user hand".into()))?;
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &card, kind) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &card, kind, &[target])?;
    match kind {
        CardKind::TrickJuedou => {
            if crate::query::card_target_limit_for_kind(tx.state(), actor, kind, 1) == 0 {
                return Err(EffectError::Illegal(
                    "source-free card has no legal target".into(),
                ));
            }
            validate_alive_other(tx, actor, target)
        }
        CardKind::TrickHuogong => {
            if tx.state().hand_cards(target).is_empty() {
                Err(EffectError::Illegal("target has no hand".into()))
            } else {
                Ok(())
            }
        }
        _ => Err(EffectError::ContentFault(
            "unsupported source-free active target card kind".into(),
        )),
    }
}

/// Whether a living actor can currently use one of the source-free card kinds needed by
/// phase-end skills. This is the non-mutating counterpart of `skill_card_target_spec`.
pub(crate) fn source_free_card_has_legal_use(
    state: &crate::state::GameState,
    actor: PlayerId,
    kind: CardKind,
) -> bool {
    if !state
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return false;
    }
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, kind)
        || crate::query::card_targets_self_only_for(state, actor)
    {
        return false;
    }
    let legal_other = |target: PlayerId| {
        target != actor
            && state
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && !crate::query::card_target_cancelled_for_logical_kind(
                state, actor, target, &card, kind,
            )
    };
    match kind {
        CardKind::TrickWanjian | CardKind::TrickNanman => {
            let targets: Vec<_> = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != actor)
                .collect();
            !targets.is_empty() && targets.iter().copied().all(legal_other)
        }
        CardKind::TrickJuedou => {
            crate::query::card_target_limit_for_kind(state, actor, kind, 1) > 0
                && state.players.alive_by_seat().into_iter().any(legal_other)
        }
        _ => false,
    }
}

/// Whether an ordinary source-free Sha may target one player while ignoring only distance.
pub(crate) fn source_free_sha_can_target_without_distance(
    state: &crate::state::GameState,
    actor: PlayerId,
    target: PlayerId,
) -> bool {
    let Some(owner) = state.players.get(actor) else {
        return false;
    };
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    if !owner.alive
        || owner.dying
        || actor == target
        || !state.players.get(target).is_some_and(|player| player.alive)
    {
        return false;
    }
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    let use_count_allowed = crate::query::sha_play_phase_use_count(state)
        < crate::query::sha_use_limit_for(state, actor)
        || crate::query::sha_use_unlimited_for_target(state, actor, target);
    let only_target = !crate::query::sha_targets_all_for(state, actor)
        || state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != actor)
            .count()
            == 1;
    use_count_allowed
        && only_target
        && crate::query::sha_target_limit_for(state, actor) > 0
        && crate::query::luanqun_sha_targets_allowed(state, actor, &[target])
        && !crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, CardKind::Sha)
        && !crate::query::card_targets_self_only_for(state, actor)
        && !crate::query::card_target_cancelled_for_logical_kind(
            state,
            actor,
            target,
            &card,
            CardKind::Sha,
        )
}

/// Whether a living actor may use the source-free, self-resolving Wuzhong needed by 颂谳.
pub(crate) fn source_free_wuzhong_has_legal_use(
    state: &crate::state::GameState,
    actor: PlayerId,
) -> bool {
    let Some(player) = state.players.get(actor) else {
        return false;
    };
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    if !player.alive || player.dying {
        return false;
    }
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::TrickWuzhong,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    !crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, CardKind::TrickWuzhong)
}

/// Use one suitless virtual Duel on the active skill's already-authoritative target list.
///
/// The active target contract replaces Duel's ordinary one-target cap; all other card-use,
/// target, Wuxie, and per-target Duel rules remain on the ordinary resolution path.
pub(crate) fn play_source_free_multi_target_juedou(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return Err(EffectError::Illegal(
            "source-free card user is not able to play".into(),
        ));
    }
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad source-free card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(
        tx.state(),
        actor,
        &source_card,
        CardKind::TrickJuedou,
    ) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    if targets.is_empty() {
        return Err(EffectError::Illegal(
            "source-free duel needs at least one target".into(),
        ));
    }
    ensure_distinct_targets(targets)?;
    validate_explicit_target_policy(tx, actor, &source_card, CardKind::TrickJuedou, targets)?;
    for &target in targets {
        validate_alive_other(tx, actor, target)?;
    }
    use_independent_trick_targets(tx, actor, card, targets, trick_effect::JUEDOU, false)
}

/// Use one suitless virtual Shunshou on an authoritative target list while bypassing only its
/// ordinary distance and one-target caps. Each target still receives an independent Wuxie window
/// and must expose a gainable hand/equipment card when the use starts.
pub(crate) fn play_source_free_multi_target_shunshou(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
        || targets.is_empty()
    {
        return Err(EffectError::Illegal(
            "source-free Shunshou user or targets are invalid".into(),
        ));
    }
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad source-free card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(
        tx.state(),
        actor,
        &source_card,
        CardKind::TrickShunshou,
    ) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    ensure_distinct_targets(targets)?;
    validate_explicit_target_policy(tx, actor, &source_card, CardKind::TrickShunshou, targets)?;
    for &target in targets {
        validate_alive_other(tx, actor, target)?;
        if crate::query::gainable_target_cards(
            tx.state(),
            actor,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty()
        {
            return Err(EffectError::Illegal(
                "source-free Shunshou target has no gainable card".into(),
            ));
        }
    }
    use_independent_trick_targets(tx, actor, card, targets, trick_effect::SHUNSHOU, false)
}

/// Use one exact suitless virtual Sha on a fixed target and draw for `skill` if that use
/// commits no damage. Ordinary distance and target policies apply; the play-phase use count does
/// not because the use is produced by a triggered skill.
pub(crate) fn use_virtual_sha_on_draw_if_no_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?
        .cards
        .push(card);

    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && !participant.dying);
    let legal = player_alive
        && tx.state().cards.get(card).is_some_and(|card_state| {
            !crate::query::card_use_cancelled_for_logical_kind(
                tx.state(),
                player,
                card_state,
                CardKind::Sha,
            ) && validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[target])
                .is_ok()
        })
        && validate_sha_targets_with_options(tx, player, card, &[target], false).is_ok();
    if !legal {
        crate::effect::move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        if !player_alive {
            return Ok(());
        }
        return run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: 1,
                skill,
            }],
        )
        .map(|_| ());
    }

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no virtual Sha user hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, player, false)?;
    tx.state_mut()
        .resolution
        .register_sha_no_damage_draw(card, skill);
    play_as_kind_with_extra_damage_flags(
        tx,
        player,
        card,
        CardKind::Sha,
        &[target],
        false,
        false,
        &[],
        true,
        false,
        rule_api::damage_card_flag::DRAW_IF_NO_DAMAGE,
        &[],
        0,
    )
}

fn validate_independent_trick_target_count(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    base: u32,
    label: &str,
) -> Result<(), EffectError> {
    let limit = crate::query::card_target_limit_for_card(tx.state(), actor, card, base);
    if targets.is_empty() || targets.len() > limit as usize {
        return Err(EffectError::Illegal(format!(
            "{label} needs 1 to {limit} targets"
        )));
    }
    ensure_distinct_targets(targets)
}

fn ensure_distinct_targets(targets: &[PlayerId]) -> Result<(), EffectError> {
    let mut seen = std::collections::BTreeSet::new();
    if targets
        .iter()
        .any(|target| target.is_none() || !seen.insert(target.raw()))
    {
        return Err(EffectError::Illegal(
            "card targets must be distinct players".into(),
        ));
    }
    Ok(())
}

fn reject_other_targeting_card_if_self_only(
    tx: &Transaction,
    actor: PlayerId,
) -> Result<(), EffectError> {
    if crate::query::card_targets_self_only_for(tx.state(), actor) {
        Err(EffectError::Illegal(
            "card requires another player as target".into(),
        ))
    } else {
        Ok(())
    }
}

fn run_effects_with_saved_resume(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    resume_kind: crate::state::ContinuationKind,
    resume_payload: Vec<u32>,
) -> Result<(crate::state::ContinuationKind, Vec<u32>), EffectError> {
    let sha = match resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => Some((
            PlayerId::new(*resume_payload.first().unwrap_or(&0)),
            CardId::new(*resume_payload.get(5).unwrap_or(&0)),
            vec![PlayerId::new(*resume_payload.get(1).unwrap_or(&0))],
            resume_payload
                .get(4)
                .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
        )),
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let len = *resume_payload.get(5).unwrap_or(&0) as usize;
            resume_payload
                .get(6..6usize.saturating_add(len))
                .map(|targets| {
                    (
                        PlayerId::new(*resume_payload.first().unwrap_or(&0)),
                        CardId::new(*resume_payload.get(4).unwrap_or(&0)),
                        targets.iter().copied().map(PlayerId::new).collect(),
                        resume_payload
                            .get(3)
                            .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
                    )
                })
        }
        _ => None,
    };
    {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = resume_kind;
        resolution.resume_payload = resume_payload;
        if let Some((source, card, targets, use_counted)) = sha {
            resolution.register_sha_resolution(source, card, &targets, use_counted);
        }
    }
    match run_effects(tx, effects) {
        Ok(_) => {
            let resolution = &mut tx.state_mut().resolution;
            let kind = resolution.resume_kind;
            resolution.resume_kind = crate::state::ContinuationKind::None;
            Ok((kind, std::mem::take(&mut resolution.resume_payload)))
        }
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Err(error) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            Err(error)
        }
    }
}

pub(super) fn resume_played_sha_from_payload(
    tx: &mut Transaction,
    kind: crate::state::ContinuationKind,
    payload: &[u32],
) -> Result<(), EffectError> {
    let (source, card) = match kind {
        crate::state::ContinuationKind::ResumePlayedSha => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(5).unwrap_or(&0)),
        ),
        crate::state::ContinuationKind::ResumePlayedShaQueue => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(4).unwrap_or(&0)),
        ),
        _ => (PlayerId::NONE, CardId::NONE),
    };
    if crate::effect::pottaishici_response_cost_armed(tx, source, card) {
        {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = kind;
            resolution.resume_payload = payload.to_vec();
        }
        crate::effect::begin_pottaishici_response_cost(tx, source, card)?;
        if tx.state().has_pending_choice() {
            return Err(EffectError::ChoiceRequested);
        }
        let resolution = &mut tx.state_mut().resolution;
        let next_kind = resolution.resume_kind;
        resolution.resume_kind = crate::state::ContinuationKind::None;
        let next = std::mem::take(&mut resolution.resume_payload);
        return resume_played_sha_from_payload(tx, next_kind, &next);
    }
    match kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let card = CardId::new(*payload.get(5).unwrap_or(&0));
            tx.state_mut().resolution.register_sha_resolution(
                source,
                card,
                &[target],
                payload
                    .get(4)
                    .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
            );
            if payload.iter().skip(6).any(|value| *value == target.raw()) {
                return finish_played_sha(tx, source, card, *payload.get(3).unwrap_or(&0));
            }
            let amount =
                payload
                    .get(2)
                    .copied()
                    .unwrap_or(1)
                    .saturating_add(sha_target_damage_bonus(
                        payload.get(4).copied().unwrap_or(1),
                    ));
            let (nature, card_flags) = unpack_damage_card_context(*payload.get(3).unwrap_or(&0));
            let respondable = *payload.get(4).unwrap_or(&1) & SHA_RESUME_RESPONDABLE != 0;
            begin_sha_response(
                tx,
                source,
                target,
                amount,
                nature,
                card_flags,
                respondable,
                Some(card),
            )
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let amount = *payload.get(1).unwrap_or(&1);
            let (nature, card_flags) = unpack_damage_card_context(*payload.get(2).unwrap_or(&0));
            let respondable = *payload.get(3).unwrap_or(&1) & SHA_RESUME_RESPONDABLE != 0;
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let declared_targets: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .copied()
                .map(PlayerId::new)
                .collect();
            tx.state_mut().resolution.register_sha_resolution(
                source,
                card,
                &declared_targets,
                payload
                    .get(3)
                    .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
            );
            let targets_end = 6usize.saturating_add(len);
            let has_response_flags = payload.get(targets_end) == Some(&u32::MAX)
                && payload.len() >= targets_end.saturating_add(1).saturating_add(len);
            let flags_start = targets_end.saturating_add(1);
            let exclusions_start = if has_response_flags {
                flags_start.saturating_add(len)
            } else {
                targets_end
            };
            let exclusions = payload.get(exclusions_start..).unwrap_or_default();
            let forced_unrespondable: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .enumerate()
                .filter(|(index, target)| {
                    !exclusions.contains(target)
                        && has_response_flags
                        && payload
                            .get(flags_start + index)
                            .is_some_and(|flags| flags & SHA_RESUME_RESPONDABLE == 0)
                })
                .map(|(_, target)| PlayerId::new(*target))
                .collect();
            let target_damage_bonuses: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .enumerate()
                .filter(|(_, target)| !exclusions.contains(target))
                .map(|(index, _)| {
                    has_response_flags
                        .then(|| payload.get(flags_start + index).copied())
                        .flatten()
                        .map(sha_target_damage_bonus)
                        .unwrap_or(0)
                })
                .collect();
            let targets: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .copied()
                .filter(|target| !exclusions.contains(target))
                .map(PlayerId::new)
                .collect();
            if targets.is_empty() {
                return finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0));
            }
            begin_sha_queue(
                tx,
                source,
                &targets,
                amount,
                nature,
                card_flags,
                respondable,
                Some(card),
                &forced_unrespondable,
                &target_damage_bonuses,
            )
        }
        _ => Err(EffectError::EngineFault(
            "invalid played Sha continuation".into(),
        )),
    }
}

fn run_effects_then_wuxie(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    ask_start: PlayerId,
    effect_tag: u32,
    actor: PlayerId,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    let card = effects
        .iter()
        .find_map(|effect| match effect {
            EffectRequest::UseCard { player, card, .. }
            | EffectRequest::UseFieldCard { player, card, .. }
                if *player == actor =>
            {
                Some(*card)
            }
            _ => None,
        })
        .ok_or_else(|| EffectError::EngineFault("wuxie window missing source card".into()))?;
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::EngineFault("wuxie window source card missing".into()))?;
    let response_kind = crate::query::effective_card_kind(tx.state(), actor, source_card);
    let response_suit = crate::query::effective_card_suit(tx.state(), actor, source_card);
    let mut resume_payload = vec![
        ask_start.raw(),
        0,
        effect_tag,
        actor.raw(),
        actor.raw(),
        response_kind.tag() as u32,
        super::judge::suit_tag(response_suit),
    ];
    resume_payload.extend_from_slice(effect_args);
    resume_payload.push(SAVED_WUXIE_SOURCE_CARD_MAGIC);
    resume_payload.push(card.raw());
    let (resume_kind, resume_payload) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::WuxieWindow,
        resume_payload,
    )?;
    if resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::EngineFault(
            "card-used triggers changed the Wuxie resume kind".into(),
        ));
    }
    let saved_args = resume_payload
        .get(7..)
        .ok_or_else(|| EffectError::EngineFault("truncated Wuxie resume payload".into()))?;
    resume_wuxie_after_card_used_triggers(
        tx,
        ask_start,
        *resume_payload.get(1).unwrap_or(&0),
        effect_tag,
        actor,
        actor,
        response_kind,
        response_suit,
        saved_args,
    )?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

pub(crate) const SAVED_WUXIE_SOURCE_CARD_MAGIC: u32 = 0x5758_4344;
const SAVED_CARD_BASE_DAMAGE_MAGIC: u32 = 0x4244_4d47;
const SAVED_TARGET_EXCLUDED_MASK: u32 = 1 << 31;

fn remove_saved_target(payload: &mut Vec<u32>, count_index: usize, start: usize, target: PlayerId) {
    let count = payload.get(count_index).copied().unwrap_or(0) as usize;
    let Some(offset) = payload
        .get(start..start.saturating_add(count))
        .and_then(|targets| targets.iter().position(|value| *value == target.raw()))
    else {
        return;
    };
    payload.remove(start + offset);
    payload[count_index] = payload[count_index].saturating_sub(1);
}

/// Exclude one target from the currently suspended physical card resolution.
pub(crate) fn exclude_saved_card_target(
    tx: &mut Transaction,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    if tx
        .state_mut()
        .resolution
        .exclude_saved_sha_target(card, target)
    {
        return Ok(());
    }
    let resolution = &tx.state().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::Illegal(
            "no matching currently resolving card target".into(),
        ));
    }
    let marker = resolution
        .resume_payload
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .ok_or_else(|| EffectError::Illegal("saved card marker is missing".into()))?;
    if resolution.resume_payload.get(marker + 1) != Some(&card.raw()) {
        return Err(EffectError::Illegal(
            "saved card target does not match triggering card".into(),
        ));
    }
    let effect_tag = *resolution.resume_payload.get(2).unwrap_or(&0);
    let default_global_targets = (marker == 7
        && matches!(effect_tag, trick_effect::TAOYUAN | trick_effect::WUGU))
    .then(|| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != target)
            .map(PlayerId::raw)
            .collect::<Vec<_>>()
    });
    let payload = &mut tx.state_mut().resolution.resume_payload;
    if payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        remove_saved_target(payload, 14, 15, target);
    } else {
        match effect_tag {
            trick_effect::NANMAN | trick_effect::WANJIAN | trick_effect::TIESUO => {
                let (count, start) = if effect_tag == trick_effect::TIESUO
                    && payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC)
                {
                    (9, 10)
                } else {
                    (7, 8)
                };
                remove_saved_target(payload, count, start, target);
            }
            trick_effect::TAOYUAN | trick_effect::WUGU => {
                if let Some(targets) = default_global_targets {
                    payload.insert(7, targets.len() as u32);
                    for (offset, player) in targets.into_iter().enumerate() {
                        payload.insert(8 + offset, player);
                    }
                } else {
                    remove_saved_target(payload, 7, 8, target);
                }
            }
            trick_effect::JIEDAO => {
                for index in 7..=8 {
                    if payload.get(index) == Some(&target.raw()) {
                        payload[index] = PlayerId::NONE.raw();
                    }
                }
            }
            trick_effect::TIAOJIYANMEI => {
                for index in 7..=8 {
                    if payload
                        .get(index)
                        .map(|raw| raw & !SAVED_TARGET_EXCLUDED_MASK)
                        == Some(target.raw())
                    {
                        payload[index] |= SAVED_TARGET_EXCLUDED_MASK;
                    }
                }
            }
            trick_effect::DELAYED_PLACE => {
                if payload.get(8) == Some(&target.raw()) {
                    payload[8] = PlayerId::NONE.raw();
                }
            }
            _ => {
                return Err(EffectError::Illegal(
                    "current card target cannot be excluded".into(),
                ));
            }
        }
    }
    Ok(())
}

/// Exclude every target of the currently suspended card use.
pub(crate) fn exclude_all_saved_card_targets(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let mut excluded = 0usize;
    for target in targets {
        if exclude_saved_card_target(tx, card, *target).is_ok() {
            excluded += 1;
        }
    }
    if excluded == targets.len() {
        return Ok(());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
        .ok_or_else(|| EffectError::Illegal("missing current card".into()))?;
    let before = tx.state().resolution.deferred_tail_effects.len();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .retain(|effect| match effect {
            EffectRequest::RecoverHp {
                source,
                player: target,
                ..
            } if matches!(kind, CardKind::Tao | CardKind::Jiu)
                && *source == player
                && targets.contains(target) =>
            {
                false
            }
            EffectRequest::AddMark {
                player: target,
                mark,
                ..
            } if kind == CardKind::Jiu
                && *target == player
                && mark.raw() == crate::content::MARK_JIU =>
            {
                false
            }
            _ => true,
        });
    if excluded == 0 && before == tx.state().resolution.deferred_tail_effects.len() {
        return Err(EffectError::ContentFault(
            "current card targets cannot be cancelled".into(),
        ));
    }
    Ok(())
}

/// Resume the original trick after card-used triggers, suppressing its Wuxie window when an
/// exact-card policy was installed while those triggers were resolving.
pub(crate) fn resume_wuxie_after_card_used_triggers(
    tx: &mut Transaction,
    start: PlayerId,
    nullified: u32,
    effect_tag: u32,
    actor: PlayerId,
    respond_to: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    args: &[u32],
) -> Result<(), EffectError> {
    let marker = args
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC);
    let (effect_args, card) = marker.map_or((args, CardId::NONE), |index| {
        (
            &args[..index],
            CardId::new(*args.get(index + 1).unwrap_or(&0)),
        )
    });
    if card.is_some()
        && crate::query::card_use_unrespondable_for_card(
            tx.state(),
            respond_to,
            card,
            response_kind,
            response_suit,
        )
    {
        return apply_deferred_trick_with_extras(tx, effect_tag, actor, effect_args, card);
    }
    begin_wuxie_window_with_parity(
        tx,
        start,
        nullified,
        effect_tag,
        actor,
        respond_to,
        response_kind,
        response_suit,
        args,
    )
}

fn effect_args_source_card(args: &[u32]) -> CardId {
    if args.first() == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return CardId::new(*args.get(1).unwrap_or(&0));
    }
    args.iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .and_then(|index| args.get(index + 1))
        .copied()
        .map(CardId::new)
        .unwrap_or(CardId::NONE)
}

fn use_card_then_wuxie(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    run_effects_then_wuxie(
        tx,
        vec![EffectRequest::UseCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }],
        actor,
        effect_tag,
        actor,
        effect_args,
    )
}

const INDEPENDENT_TRICK_QUEUE_MAGIC: u32 = 0x4e54_5155;
pub(crate) const MANTIANGUOHAI_CUSTOM_CHOICE: u32 = 0x4d54_5391;
pub(crate) const MANTIANGUOHAI_SUCCESS_MASK: u32 = 1 << 30;
pub(crate) const DIAOBING_POOL_MAGIC: u32 = 0x4442_504c;
pub(crate) const WUGU_POOL_MAGIC: u32 = 0x5747_504c;

fn saved_independent_trick_damage_bonus(args: &[u32]) -> u32 {
    if args.first() != Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return 0;
    }
    let payload = &args[1..];
    let target_count = *payload.get(6).unwrap_or(&0) as usize;
    let extension_at = 7usize.saturating_add(target_count);
    if payload.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
        *payload.get(extension_at + 1).unwrap_or(&0)
    } else {
        0
    }
}

pub(crate) fn saved_independent_trick_damage_card_and_bonus(args: &[u32]) -> (CardId, u32) {
    (
        queued_independent_trick(args)
            .map(|(payload, _)| CardId::new(payload[0]))
            .unwrap_or(CardId::NONE),
        saved_independent_trick_damage_bonus(args),
    )
}

fn use_independent_trick_targets(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
    field_source: bool,
) -> Result<(), EffectError> {
    let effect_args = independent_trick_effect_args(tx, actor, card, targets, effect_tag)?;
    let use_effect = if field_source {
        EffectRequest::UseFieldCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }
    } else {
        EffectRequest::UseCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }
    };
    run_effects_then_wuxie(tx, vec![use_effect], actor, effect_tag, actor, &effect_args)
}

fn independent_trick_effect_args(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
) -> Result<Vec<u32>, EffectError> {
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad trick card".into()))?;
    let kind = crate::query::effective_card_kind(tx.state(), actor, source_card);
    let suit = crate::query::effective_card_suit(tx.state(), actor, source_card);
    let mut payload = vec![
        card.raw(),
        actor.raw(),
        kind.tag() as u32,
        super::judge::suit_tag(suit),
        effect_tag,
        0,
        targets.len() as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    let mut effect_args = Vec::with_capacity(payload.len() + 1);
    effect_args.push(INDEPENDENT_TRICK_QUEUE_MAGIC);
    effect_args.extend_from_slice(&payload);
    Ok(effect_args)
}

/// Resolve an already-used independent trick without opening a Wuxie window.
pub(crate) fn resolve_independent_trick_without_wuxie(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
) -> Result<(), EffectError> {
    let effect_args = independent_trick_effect_args(tx, actor, card, targets, effect_tag)?;
    apply_deferred_trick(tx, effect_tag, actor, &effect_args, card)
}

/// Continue the next target of a non-global ordinary trick.
pub(crate) fn advance_independent_trick_queue(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "malformed independent trick queue".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let kind = CardKind::from_tag(payload[2] as u8)
        .ok_or_else(|| EffectError::EngineFault("bad queued trick kind".into()))?;
    let suit = super::judge::suit_from_tag(payload[3]);
    let effect_tag = payload[4];
    let index = payload[5] as usize;
    let len = payload[6] as usize;
    if payload.len() < 7 + len {
        return Err(EffectError::EngineFault(
            "truncated independent trick queue".into(),
        ));
    }
    if index >= len {
        let resolution = &mut tx.state_mut().resolution;
        resolution.continuation_kind = crate::state::ContinuationKind::None;
        resolution.continuation_payload.clear();
        if effect_tag == trick_effect::MANTIANGUOHAI {
            return begin_mantianguohai_gives(tx, payload);
        }
        if effect_tag == trick_effect::DIAOBINGQIANJIANG {
            let extension = 7usize.saturating_add(len);
            if payload.get(extension) != Some(&DIAOBING_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "调兵遣将 pool marker is missing".into(),
                ));
            }
            let count = *payload.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = payload
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("调兵遣将 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            return run_effects(
                tx,
                vec![EffectRequest::ContinueDiaobingqianjiang {
                    action: 1,
                    actor,
                    source_card: CardId::new(payload[0]),
                    targets: Vec::new(),
                    revealed,
                }],
            )
            .map(|_| ());
        }
        if effect_tag == trick_effect::WUGU {
            let extension = 7usize.saturating_add(len);
            if payload.get(extension) != Some(&WUGU_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "五谷丰登 pool marker is missing".into(),
                ));
            }
            let count = *payload.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = payload
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            return run_effects(
                tx,
                vec![EffectRequest::ContinueWugu {
                    action: 2,
                    actor,
                    source_card: CardId::new(payload[0]),
                    targets: Vec::new(),
                    excluded: Vec::new(),
                    revealed,
                }],
            )
            .map(|_| ());
        }
        return Ok(());
    }
    let mut effect_args = Vec::with_capacity(payload.len() + 1);
    effect_args.push(INDEPENDENT_TRICK_QUEUE_MAGIC);
    effect_args.extend_from_slice(payload);
    if effect_tag == trick_effect::HAOLINGTIANXIA {
        // 号令天下：非标准逐目标锦囊——链目标由 advance_haolingtianxia_player
        // 依次二选一驱动，不开启无懈窗口。
        return advance_haolingtianxia_player(tx, payload);
    }
    begin_wuxie_window(tx, actor, effect_tag, actor, kind, suit, &effect_args)
}

fn queued_independent_trick(args: &[u32]) -> Option<(&[u32], PlayerId)> {
    if args.first().copied() != Some(INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return None;
    }
    let payload = args.get(1..)?;
    let index = *payload.get(5)? as usize;
    let len = *payload.get(6)? as usize;
    if index >= len || payload.len() < 7 + len {
        return None;
    }
    Some((payload, PlayerId::new(payload[7 + index])))
}

/// Finish one independently resolved trick target and advance to the next target.
pub(crate) fn finish_independent_trick_target(
    tx: &mut Transaction,
    args: &[u32],
) -> Result<(), EffectError> {
    let Some((payload, _)) = queued_independent_trick(args) else {
        return Ok(());
    };
    let mut next = payload.to_vec();
    next[5] = next[5].saturating_add(1);
    advance_independent_trick_queue(tx, &next)
}

fn begin_mantianguohai_gives(tx: &mut Transaction, queue: &[u32]) -> Result<(), EffectError> {
    if queue.len() < 7 {
        return Err(EffectError::EngineFault(
            "malformed 瞒天过海 target queue".into(),
        ));
    }
    let actor = PlayerId::new(queue[1]);
    let len = queue[6] as usize;
    if queue.len() < 7 + len {
        return Err(EffectError::EngineFault(
            "truncated 瞒天过海 target queue".into(),
        ));
    }
    let successful: std::collections::BTreeSet<_> = queue[7..7 + len]
        .iter()
        .filter(|raw| **raw & MANTIANGUOHAI_SUCCESS_MASK != 0)
        .map(|raw| PlayerId::new(*raw & !MANTIANGUOHAI_SUCCESS_MASK))
        .collect();
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| successful.contains(target))
        .collect();
    resolve_mantianguohai_give(tx, actor, targets)
}

pub(crate) fn resolve_mantianguohai_give(
    tx: &mut Transaction,
    actor: PlayerId,
    mut targets: Vec<PlayerId>,
) -> Result<(), EffectError> {
    loop {
        let Some(target) = targets.first().copied() else {
            return Ok(());
        };
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        {
            targets.remove(0);
            continue;
        }
        let mut options = tx.state().hand_cards(actor);
        if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, actor) {
            options.extend(
                tx.state()
                    .zones
                    .get(equip)
                    .into_iter()
                    .flat_map(|zone| zone.cards.iter().copied()),
            );
        }
        if options.is_empty() {
            return Ok(());
        }
        let mut payload = vec![
            MANTIANGUOHAI_CUSTOM_CHOICE,
            1,
            actor.raw(),
            target.raw(),
            targets.len() as u32,
        ];
        payload.extend(targets.iter().map(|target| target.raw()));
        crate::effect::open_choice(
            tx,
            actor,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::Custom,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

/// Apply one target's effects while preserving the remaining trick queue across choices/dying.
pub(crate) fn run_effects_then_finish_independent_trick_target(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    args: &[u32],
) -> Result<(), EffectError> {
    let Some((payload, _)) = queued_independent_trick(args) else {
        return run_effects(tx, effects).map(|_| ());
    };
    let mut next = payload.to_vec();
    next[5] = next[5].saturating_add(1);
    let (_, next) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::IndependentTrickQueue,
        next,
    )?;
    advance_independent_trick_queue(tx, &next)
}

/// Run effects that may suspend on a nested choice (e.g. a damage trigger),
/// resuming into the independent-trick queue once the whole nested chain completes.
/// Used by 号令天下's per-player chain so each step can continue to the next actor.
pub(crate) fn run_effects_then_resume_trick_queue(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    queue: Vec<u32>,
) -> Result<(), EffectError> {
    let (_, next) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::IndependentTrickQueue,
        queue,
    )?;
    advance_independent_trick_queue(tx, &next)
}

/// Advance / open the 号令天下 (guozhan) per-player option chain.
///
/// Queue layout (an independent-trick-queue superset):
/// `[0] card, [1] actor, [2] kind, [3] suit, [4] HAOLINGTIANXIA, [5] index, [6] len,
/// [7..7+len] chain players (seat order, excluding the trick target),
/// [7+len] trick target, [8+len..] original trick args tail (finish resume).`
///
/// Each live chain player with at least one legal option gets a stage-1 choice
/// (① 弃牌视为对目标用杀 / ② 弃置或获得目标一张牌); the trick user is a chain
/// member too when not the target. Dead / option-less players are skipped.
pub(crate) fn advance_haolingtianxia_player(
    tx: &mut Transaction,
    queue: &[u32],
) -> Result<(), EffectError> {
    if queue.len() < 8 {
        return Err(EffectError::EngineFault(
            "malformed 号令天下 chain queue".into(),
        ));
    }
    let index = queue[5] as usize;
    let len = queue[6] as usize;
    if index >= len {
        let tail = queue.get(8 + len..).unwrap_or_default();
        return finish_independent_trick_target(tx, tail);
    }
    let current = PlayerId::new(queue[7 + index]);
    let trick_target = PlayerId::new(queue[7 + len]);
    let advance = |tx: &mut Transaction| -> Result<(), EffectError> {
        let mut next = queue.to_vec();
        next[5] = next[5].saturating_add(1);
        advance_haolingtianxia_player(tx, &next)
    };
    if !tx
        .state()
        .players
        .get(current)
        .is_some_and(|player| player.alive)
    {
        return advance(tx);
    }
    let is_wei = tx
        .state()
        .player_group(current)
        .map(str::to_string)
        .as_deref()
        == Some("wei");
    let mut choices: Vec<u32> = Vec::new();
    // ① 视为对目标使用杀：需在攻击范围内，且（魏则免弃牌）或本人有可弃的「he」牌。
    if sha_in_range(tx, current, trick_target)
        && (is_wei || !target_zone_card_tokens(tx, current, true, true, false).is_empty())
    {
        choices.push(1);
    }
    // ② 弃置（或魏势力获得）目标一张牌：目标需有「he」牌。
    if !target_zone_card_tokens(tx, trick_target, true, true, false).is_empty() {
        choices.push(2);
    }
    if choices.is_empty() {
        return advance(tx);
    }
    let mut payload = vec![crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE, 1];
    payload.extend_from_slice(queue);
    crate::effect::open_choice(
        tx,
        current,
        1,
        1,
        true,
        choices,
        engine_api::ChoiceKind::Custom,
        crate::state::ContinuationKind::Custom,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

fn equip_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
) -> Result<(), EffectError> {
    let slot = kind
        .equip_slot()
        .ok_or_else(|| EffectError::Illegal("card has no equipment slot".into()))?;
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.equip_slot_enabled(slot))
    {
        return Err(EffectError::Illegal("equipment slot disabled".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, actor)
        .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let source = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("equipment card disappeared".into()))?;
    if source != hand
        && crate::query::borrowed_hand_source(tx.state(), actor, card).is_none()
        && !tx.state().resolution.has_planned_card_source(actor, card)
    {
        return Err(EffectError::Illegal(
            "equipment card is not usable from hand".into(),
        ));
    }
    let borrowed_refills = crate::effect::pothaoshi_refill_effects(tx.state(), actor, &[card]);

    // Unequip same slot first
    let existing: Vec<CardId> = tx
        .state()
        .zones
        .get(equip)
        .map(|z| z.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|cid| {
            tx.state()
                .cards
                .get(*cid)
                .map(|c| same_equip_slot(c.kind, kind))
                .unwrap_or(false)
        })
        .collect();

    let mut queue = vec![EffectRequest::UseCard {
        player: actor,
        card,
        targets: vec![],
    }];
    if !existing.is_empty() {
        queue.push(EffectRequest::MoveCards {
            cards: existing,
            to: discard,
            from: Some(equip),
        });
    }
    queue.push(EffectRequest::MoveCardsWithReason {
        cards: vec![card],
        to: equip,
        from: Some(source),
        reason: CardMoveReason::Use,
    });
    queue.extend(borrowed_refills);
    queue.push(EffectRequest::EmitRuleEvent {
        code: 50, // equipped
        payload: vec![actor.raw(), card.raw()],
    });
    run_effects(tx, queue)?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

/// Cancel the exact card use currently suspended behind its CardUsed triggers.
pub(crate) fn cancel_saved_card_use(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|state| crate::query::effective_card_kind(tx.state(), actor, state))
        .ok_or_else(|| EffectError::Illegal("current card disappeared".into()))?;
    if kind.is_sha() {
        let targets = saved_targeted_card_targets(tx, actor, card)?;
        return exclude_all_saved_card_targets(tx, actor, card, &targets);
    }
    if kind.is_trick() {
        let resolution = &mut tx.state_mut().resolution;
        if resolution.resume_kind != ContinuationKind::WuxieWindow
            || !resolution
                .resume_payload
                .windows(2)
                .any(|pair| pair == [SAVED_WUXIE_SOURCE_CARD_MAGIC, card.raw()])
        {
            return Err(EffectError::Illegal(
                "no matching trick use to cancel".into(),
            ));
        }
        resolution.resume_payload[1] = 1;
        return Ok(());
    }
    if kind.is_equip() {
        let tail = &mut tx.state_mut().resolution.deferred_tail_effects;
        let Some(end) = tail.iter().position(|effect| {
            matches!(effect, EffectRequest::EmitRuleEvent { code: 50, payload } if payload.get(1) == Some(&card.raw()))
        }) else {
            return Err(EffectError::Illegal("no matching equipment use to cancel".into()));
        };
        tail.drain(..=end);
        return Ok(());
    }
    let before = tx.state().resolution.deferred_tail_effects.len();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .retain(|effect| match effect {
            EffectRequest::RecoverHp { source, player, .. }
                if *source == actor && *player == actor =>
            {
                false
            }
            EffectRequest::AddMark { player, mark, .. }
                if *player == actor && mark.raw() == crate::content::MARK_JIU =>
            {
                false
            }
            _ => true,
        });
    if before == tx.state().resolution.deferred_tail_effects.len() {
        return Err(EffectError::Illegal(
            "current card use cannot be cancelled".into(),
        ));
    }
    Ok(())
}

fn same_equip_slot(a: CardKind, b: CardKind) -> bool {
    matches!(
        (a, b),
        (CardKind::Weapon, CardKind::Weapon)
            | (CardKind::Armor, CardKind::Armor)
            | (CardKind::HorseOff, CardKind::HorseOff)
            | (CardKind::HorseDef, CardKind::HorseDef)
            | (CardKind::Treasure, CardKind::Treasure)
    )
}

fn has_delayed_of_kind(tx: &Transaction, player: PlayerId, kind: CardKind) -> bool {
    let Some(jz) = tx.state().zones.find(ZoneKind::Judge, player) else {
        return false;
    };
    let Some(z) = tx.state().zones.get(jz) else {
        return false;
    };
    z.cards.iter().any(|cid| {
        tx.state()
            .cards
            .get(*cid)
            .map(|c| c.kind == kind)
            .unwrap_or(false)
    })
}

/// After 杀 is used: ask target to play 闪 (or view_as 闪). Cancel = damage.
/// 无双: need 2 闪. `nature` = damage_nature tag for the eventual damage.
pub(super) fn begin_sha_response(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: Option<CardId>,
) -> Result<(), EffectError> {
    let need = crate::query::sha_response_count_for(tx.state(), source, target, 1);
    let respondable = respondable
        && !crate::query::sha_use_unrespondable_within_distance(tx.state(), source, &[target])
        && !crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
        && card.is_none_or(|card| {
            !crate::query::sha_unrespondable_for_card_target(tx.state(), source, target, card)
                && !crate::query::potshishi_blocks_response(tx.state(), source, card, target)
        });
    // Source damage modifiers (裸衣 +1 etc.) apply to 杀 damage.
    let amount = card.map_or_else(
        || crate::query::sha_damage_for(tx.state(), source, amount),
        |card| crate::query::sha_damage_for_card(tx.state(), source, card, amount),
    );
    open_sha_response_step(
        tx,
        source,
        target,
        amount,
        need,
        0,
        nature,
        card_flags,
        respondable,
        card.unwrap_or(CardId::NONE),
    )
}

/// Resolve a multi-target 杀 through the existing ordered response queue.
/// Queue payload: `[3, source, amount, idx, len, nature, need, have, t0, ..., r0, ..., card]`.
pub(super) fn begin_sha_queue(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
    amount: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: Option<CardId>,
    forced_unrespondable: &[PlayerId],
    target_damage_bonuses: &[u32],
) -> Result<(), EffectError> {
    let need = 1;
    let use_locked =
        crate::query::sha_use_unrespondable_within_distance(tx.state(), source, targets);
    let amount = card.map_or_else(
        || crate::query::sha_damage_for(tx.state(), source, amount),
        |card| crate::query::sha_damage_for_card(tx.state(), source, card, amount),
    );
    let mut payload = vec![
        3,
        source.raw(),
        amount,
        0,
        targets.len() as u32,
        pack_damage_card_context(nature, card_flags),
        need,
        0,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    payload.extend(targets.iter().enumerate().map(|(index, target)| {
        let target_locked = card.is_some_and(|card| {
            crate::query::sha_unrespondable_for_card_target(tx.state(), source, *target, card)
                || crate::query::potshishi_blocks_response(tx.state(), source, card, *target)
        }) || forced_unrespondable.contains(target);
        let distance_locked =
            crate::query::card_use_unrespondable_within_distance(tx.state(), source, *target);
        let response_flag =
            u32::from(respondable && !use_locked && !target_locked && !distance_locked);
        let bonus = target_damage_bonuses
            .get(index)
            .copied()
            .unwrap_or(0)
            .min(u32::MAX >> SHA_TARGET_DAMAGE_BONUS_SHIFT);
        response_flag | (bonus << SHA_TARGET_DAMAGE_BONUS_SHIFT)
    }));
    payload.push(card.unwrap_or(CardId::NONE).raw());
    let resolution = &mut tx.state_mut().resolution;
    resolution.continuation_kind = crate::state::ContinuationKind::RespondAoeQueue;
    resolution.continuation_payload = payload;
    advance_aoe_queue(tx)
}

/// Open one step of 杀-闪 response (supports multi-shan for 无双 + 八卦).
/// payload: `[source, target, amount, need, have, nature, respondable, card]`
pub(crate) fn open_sha_response_step(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    need: u32,
    have: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: CardId,
) -> Result<(), EffectError> {
    let tokens = if respondable {
        let mut tokens = response_shan_options(tx.state(), target);
        if have == 0 {
            tokens.extend(response_youdishenru_tokens(tx.state(), target));
        }
        tokens
    } else {
        Vec::new()
    };
    if tokens.is_empty() {
        return deal_single_sha_damage(tx, source, target, amount, nature, card_flags, card);
    }
    crate::effect::open_choice(
        tx,
        target,
        0,
        1,
        true,
        tokens,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::RespondToSha,
        vec![
            source.raw(),
            target.raw(),
            amount,
            need,
            have,
            pack_damage_card_context(nature, card_flags),
            u32::from(respondable),
            card.raw(),
        ],
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(super) fn deal_sha_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    card: CardId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let amount =
        crate::query::sha_damage_outside_target_range_for(tx.state(), source, target, amount);
    let amount = crate::query::sha_damage_taken_for(tx.state(), target, amount);
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        }],
    )
}

/// 闪 response options: 八卦 token (if equipped) + hand 闪 / view_as 闪.
pub(crate) fn response_shan_options(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if crate::content::player_has_bagua(state, player) {
        out.push(crate::content::BAGUA_RESPONSE_TOKEN);
    }
    out.extend(response_shan_tokens(state, player));
    out
}

/// Shan options a different player may physically provide for a delegated response.
pub(crate) fn delegated_shan_options(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if crate::content::player_has_bagua(state, player) {
        out.push(crate::content::BAGUA_RESPONSE_TOKEN);
    }
    if !crate::query::hand_card_use_response_cancelled_for(state, player) {
        out.extend(response_card_tokens(state, player, "shan", |kind| {
            kind == CardKind::Shan
        }));
    }
    out
}

/// Physical Sha options another player may provide without recursively delegating again.
pub(crate) fn delegated_sha_options(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::sha_response_cancelled_for(state, player)
        || crate::query::hand_card_use_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    response_card_tokens(state, player, "sha", CardKind::is_sha)
}

/// Run 八卦判定：红心视为打出 闪。Returns `true` if successful.
pub(crate) fn try_bagua_as_shan(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<(bool, EffectRequest), EffectError> {
    let bagua = crate::content::player_bagua_source(tx.state(), player)
        .ok_or_else(|| EffectError::Illegal("no bagua".into()))?;
    let result = super::judge::flip_judgment(tx, player, super::judge::JudgmentFavor::Red)?;
    let suit = result.suit;
    let rank = result.rank;
    let ok = matches!(suit, engine_api::Suit::Heart | engine_api::Suit::Diamond);
    run_effects(
        tx,
        vec![EffectRequest::EmitRuleEvent {
            code: 231, // bagua judge
            payload: vec![
                player.raw(),
                super::judge::suit_tag(suit),
                rank as u32,
                u32::from(ok),
            ],
        }],
    )?;
    Ok((
        ok,
        EffectRequest::EmitJudgmentResolved {
            player,
            result_card: result.card,
            suit,
            rank,
            cause_card: bagua,
        },
    ))
}

/// Ordered response queue: kind 1=南蛮 need 杀, 2=万箭 need 闪, 3=multi-target 杀.
pub(crate) fn begin_aoe_queue(
    tx: &mut Transaction,
    kind: u32,
    source: PlayerId,
    amount: u32,
    targets: &[PlayerId],
    card: CardId,
) -> Result<(), EffectError> {
    if targets.is_empty() {
        return Ok(());
    }
    let mut payload = vec![
        kind,
        source.raw(),
        amount,
        0, // idx
        targets.len() as u32,
    ];
    for t in targets {
        payload.push(t.raw());
    }
    payload.push(card.raw());
    // Store payload then process from idx 0
    eprintln!("DBG begin_aoe_queue kind={} targets={:?} card={}", kind, targets, card.raw());
    {
        let r = &mut tx.state_mut().resolution;
        r.continuation_kind = crate::state::ContinuationKind::RespondAoeQueue;
        r.continuation_payload = payload;
    }
    advance_aoe_queue(tx)
}

/// Bump AOE queue index on resume payload and/or live continuation.
fn bump_aoe_resume_idx(tx: &mut Transaction, next_idx: usize) {
    let r = &mut tx.state_mut().resolution;
    if r.resume_kind == crate::state::ContinuationKind::RespondAoeQueue
        && r.resume_payload.len() > 3
    {
        r.resume_payload[3] = next_idx as u32;
        if r.resume_payload.first() == Some(&3) && r.resume_payload.len() > 7 {
            r.resume_payload[7] = 0;
        }
    }
    if r.continuation_kind == crate::state::ContinuationKind::RespondAoeQueue
        && r.continuation_payload.len() > 3
    {
        r.continuation_payload[3] = next_idx as u32;
        if r.continuation_payload.first() == Some(&3) && r.continuation_payload.len() > 7 {
            r.continuation_payload[7] = 0;
        }
    }
}

/// Process current AOE target or finish.
pub(crate) fn advance_aoe_queue(tx: &mut Transaction) -> Result<(), EffectError> {
    loop {
        let mut payload = tx.state().resolution.continuation_payload.clone();
        if payload.len() < 6 {
            tx.state_mut().resolution.clear_choice();
            return Ok(());
        }
        let kind = payload[0];
        let source = PlayerId::new(payload[1]);
        let amount = payload[2];
        let idx = payload[3] as usize;
        let len = payload[4] as usize;
        let target_offset = if kind == 3 { 8 } else { 5 };
        if idx >= len || target_offset + idx >= payload.len() {
            tx.state_mut().resolution.clear_choice();
            if kind == 3 {
                let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
                return finish_played_sha(tx, source, card, *payload.get(5).unwrap_or(&0));
            }
            return Ok(());
        }
        let target = PlayerId::new(payload[target_offset + idx]);
        let source_card = if kind == 3 {
            CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0))
        } else {
            CardId::new(*payload.get(5 + len).unwrap_or(&0))
        };
        // Skip dead
        if tx
            .state()
            .players
            .get(target)
            .map(|p| !p.alive)
            .unwrap_or(true)
        {
            let queue = &mut tx.state_mut().resolution.continuation_payload;
            queue[3] = (idx + 1) as u32;
            if kind == 3 {
                queue[7] = 0;
            }
            continue;
        }
        if kind == 3 {
            payload[6] = crate::query::sha_response_count_for(tx.state(), source, target, 1);
        }
        // 批 155：消费暂存排除（啖酪族「令此牌对你无效」——触发效果执行时 AOE 队列
        // 尚未建立，排除意图经 deferred_card_exclusions 传递至此）。
        let deferred_hit = tx
            .state()
            .deferred_card_exclusions
            .iter()
            .any(|(card_raw, target_raw)| {
                *card_raw == source_card.raw() && *target_raw == target.raw()
            });
        if deferred_hit && (kind == 1 || kind == 2) {
            tx.state_mut()
                .deferred_card_exclusions
                .retain(|(card_raw, target_raw)| {
                    !(*card_raw == source_card.raw() && *target_raw == target.raw())
                });
            tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
            continue;
        }
        // 藤甲：南蛮入侵对该角色无效（无需出杀，不受伤）
        if kind == 1 && crate::content::player_has_tengjia(tx.state(), target) {
            let _ = run_effects(
                tx,
                vec![EffectRequest::EmitRuleEvent {
                    code: 230, // tengjia blocks nanman
                    payload: vec![target.raw()],
                }],
            );
            tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
            continue;
        }
        let respondable = (kind != 3
            || sha_queue_target_respondable(&payload, target_offset, len, idx))
            && !crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
            && !crate::query::potshishi_blocks_response(tx.state(), source, source_card, target);
        let tokens = match kind {
            1 if respondable => response_sha_tokens(tx.state(), target),
            2 | 3 if respondable => {
                let mut options = response_shan_options(tx.state(), target);
                if kind == 3 && payload[7] == 0 {
                    options.extend(response_youdishenru_tokens(tx.state(), target));
                }
                options
            }
            _ => vec![],
        };
        if kind == 3 && payload[6] == 0 {
            let queue = &mut tx.state_mut().resolution.continuation_payload;
            queue[3] = (idx + 1) as u32;
            queue[7] = 0;
            continue;
        }
        eprintln!("DBG aoe idx={} target={} tokens={} respondable={}", idx, target.raw(), tokens.len(), respondable);
        if tokens.is_empty() {
            eprintln!("DBG aoe dealing damage to {}", target.raw());
            // Fail → damage, next
            let dmg = if kind == 3 {
                let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
                let (nature, card_flags) = unpack_damage_card_context(payload[5]);
                let amount = amount.saturating_add(sha_queue_target_damage_bonus(
                    &payload,
                    target_offset,
                    len,
                    idx,
                ));
                deal_sha_damage(tx, source, target, amount, nature, card_flags, card)
            } else {
                eprintln!("DBG aoe DealDamage src={} tgt={} amount={}", source.raw(), target.raw(), amount);
                let res = run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source,
                        target,
                        amount,
                        is_lose_hp: false,
                        nature: 0,
                    }],
                );
                eprintln!("DBG aoe after dmg hp={}", tx.state().players.get(target).map(|p| p.hp).unwrap_or(-1));
                res
            };
            match dmg {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {
                    // Damage applied; dying (or nested choice) interrupted.
                    // Always advance past this target so resume does not re-hit.
                    bump_aoe_resume_idx(tx, idx + 1);
                    return Err(EffectError::ChoiceRequested);
                }
                Err(e) => return Err(e),
            }
            if tx.state().has_pending_choice() {
                bump_aoe_resume_idx(tx, idx + 1);
                return Ok(());
            }
            // Still in AOE: advance idx on stored payload
            if tx.state().resolution.continuation_kind
                == crate::state::ContinuationKind::RespondAoeQueue
                && tx.state().resolution.continuation_payload.len() > 3
            {
                tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
                if kind == 3 {
                    tx.state_mut().resolution.continuation_payload[7] = 0;
                }
            } else {
                // restore queue progress
                let mut p = payload;
                p[3] = (idx + 1) as u32;
                if kind == 3 {
                    p[7] = 0;
                }
                tx.state_mut().resolution.continuation_kind =
                    crate::state::ContinuationKind::RespondAoeQueue;
                tx.state_mut().resolution.continuation_payload = p;
            }
            continue;
        }
        // Open response for this target
        crate::effect::open_choice(
            tx,
            target,
            0,
            1,
            true,
            tokens,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::RespondAoeQueue,
            // keep full queue payload
            {
                let mut p = payload;
                p[3] = idx as u32; // current
                p
            },
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

fn sha_queue_target_respondable(
    payload: &[u32],
    target_offset: usize,
    len: usize,
    idx: usize,
) -> bool {
    let response_offset = target_offset + len;
    if payload.len() >= response_offset + len {
        payload.get(response_offset + idx).copied().unwrap_or(1) & SHA_RESUME_RESPONDABLE != 0
    } else {
        // Accept the previous single-flag in-flight queue shape.
        payload.get(response_offset).copied().unwrap_or(1) & SHA_RESUME_RESPONDABLE != 0
    }
}

pub(crate) fn sha_queue_target_damage_bonus(
    payload: &[u32],
    target_offset: usize,
    len: usize,
    idx: usize,
) -> u32 {
    let response_offset = target_offset.saturating_add(len);
    if payload.len() >= response_offset.saturating_add(len) {
        payload
            .get(response_offset + idx)
            .copied()
            .map(sha_target_damage_bonus)
            .unwrap_or(0)
    } else {
        0
    }
}

pub(crate) fn begin_juedou_with_tail(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    tail: &[u32],
) -> Result<(), EffectError> {
    open_juedou_step_n_with_tail(tx, source, target, target, 0, tail)
}

pub(crate) fn open_juedou_step_n_with_tail(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    // Safety: prevent pathological view_as 杀 loops (max 32 exchanges).
    if exchanges >= 32 {
        return finish_juedou_with_tail(tx, source, target, next, 1, tail);
    }
    let opponent = if next == target { source } else { target };
    let need = crate::query::duel_response_count_for(tx.state(), opponent, 1);
    open_juedou_response_step(tx, source, target, next, exchanges, need, 0, tail)
}

fn open_juedou_response_step(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    need: u32,
    have: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    let source_card = queued_independent_trick(tail)
        .map(|(payload, _)| CardId::new(payload[0]))
        .unwrap_or_else(|| effect_args_source_card(tail));
    let tokens = if next == target
        && (crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
            || crate::query::potshishi_blocks_response(tx.state(), source, source_card, target))
    {
        Vec::new()
    } else {
        response_sha_tokens(tx.state(), next)
    };
    if tokens.is_empty() {
        return finish_juedou_with_tail(tx, source, target, next, 1, tail);
    }
    let mut payload = vec![
        source.raw(),
        target.raw(),
        next.raw(),
        exchanges,
        need,
        have,
    ];
    payload.extend_from_slice(tail);
    crate::effect::open_choice(
        tx,
        next,
        0,
        1,
        true,
        tokens,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::JuedouDuel,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn juedou_payload_after_response(
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    need: u32,
    have: u32,
    tail: &[u32],
) -> Vec<u32> {
    let have = have.saturating_add(1);
    let (next, exchanges, need, have) = if have < need {
        (next, exchanges, need, have)
    } else {
        (
            if next == target { source } else { target },
            exchanges.saturating_add(1),
            0,
            0,
        )
    };
    let mut payload = vec![
        source.raw(),
        target.raw(),
        next.raw(),
        exchanges,
        need,
        have,
    ];
    payload.extend_from_slice(tail);
    payload
}

pub(crate) fn continue_juedou_payload(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let source = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let next = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let exchanges = *payload.get(3).unwrap_or(&0);
    let need = *payload.get(4).unwrap_or(&0);
    let have = *payload.get(5).unwrap_or(&0);
    let tail = payload.get(6..).unwrap_or_default();
    if need == 0 {
        open_juedou_step_n_with_tail(tx, source, target, next, exchanges, tail)
    } else {
        open_juedou_response_step(tx, source, target, next, exchanges, need, have, tail)
    }
}

pub(crate) fn finish_juedou_with_tail(
    tx: &mut Transaction,
    initiator: PlayerId,
    target: PlayerId,
    loser: PlayerId,
    amount: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    let winner = if loser == target { initiator } else { target };
    let amount = amount.saturating_add(saved_independent_trick_damage_bonus(tail));
    let queued = queued_independent_trick(tail);
    let card = queued
        .map(|(payload, _)| CardId::new(payload[0]))
        .unwrap_or(CardId::NONE);
    let damage = if let Some((payload, _)) = queued {
        EffectRequest::DealCardDamage {
            source: winner,
            target: loser,
            amount,
            is_lose_hp: false,
            nature: 0,
            card: CardId::new(payload[0]),
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }
    } else {
        EffectRequest::DealDamage {
            source: winner,
            target: loser,
            amount,
            is_lose_hp: false,
            nature: 0,
        }
    };
    run_effects_then_finish_independent_trick_target(
        tx,
        vec![
            damage,
            EffectRequest::EmitDuelResolved {
                card,
                initiator,
                target,
                winner,
                loser,
            },
        ],
        tail,
    )
}

fn player_has_mark(tx: &Transaction, player: PlayerId, mark: u32) -> bool {
    tx.state()
        .players
        .get(player)
        .map(|p| p.marks.iter().any(|(m, c)| *m == mark && *c > 0))
        .unwrap_or(false)
}

/// Deferred non-delayed trick effect tags (after 无懈 window).
pub(crate) mod trick_effect {
    pub const WUZHONG: u32 = 1;
    pub const GUOHE: u32 = 2;
    pub const SHUNSHOU: u32 = 3;
    pub const HUOGONG: u32 = 4;
    pub const JUEDOU: u32 = 5;
    pub const NANMAN: u32 = 6;
    pub const WANJIAN: u32 = 7;
    pub const TAOYUAN: u32 = 8;
    pub const WUGU: u32 = 9;
    pub const TIESUO: u32 = 10;
    pub const JIEDAO: u32 = 11;
    /// Place delayed trick into judge zone: args `[card, target, kind_tag]`.
    pub const DELAYED_PLACE: u32 = 12;
    pub const YUANJIAO: u32 = 13;
    pub const TIAOJIYANMEI: u32 = 14;
    pub const MANTIANGUOHAI: u32 = 15;
    pub const YOUDISHENRU: u32 = 16;
    pub const DIAOBINGQIANJIANG: u32 = 17;
    pub const WANGMEIZHIKE: u32 = 18;
    /// 以逸待劳（国战：对己方全体，各摸两张再弃两张）.
    pub const YIYI: u32 = 19;
    /// 水淹七军（国战：目标弃置全部装备牌或受 1 点雷电伤害）.
    pub const SHUIYANQIJUNX: u32 = 20;
    /// 戮力同心（国战：全部大势力或全部小势力；未横置→横置，已横置→摸一张）.
    pub const LULITONGXIN: u32 = 21;
    /// 克复中原（国战：任意目标，各目标视为使用杀或摸牌；蜀势力伤害+1/摸两张）.
    pub const KEFUZHONGYUAN: u32 = 22;
    /// 固国安邦（国战：自用，摸八弃≥六；吴势力可将弃牌交给其他吴角色）.
    pub const GUGUOANBANG: u32 = 23;
    /// 火烧连营（国战：对下家及其队列所有角色造成 1 点火属性伤害）.
    pub const HUOSHAOLIANYING: u32 = 24;
    /// 联军盛宴（国战：对己方及一敌方势力全体；己方分配摸/回，敌方摸1并重置）.
    pub const LIANJUNSHENGYAN: u32 = 25;
    /// 文和乱武（国战：全角色；使用者选弃两张类型不同手牌或弃其一张手牌，
    /// 群势力空手牌则补至体力至多 5）.
    pub const WENHELUANWU: u32 = 26;
    /// 号令天下（国战：对一名体力值非全场最少者使用；其他角色依次二选一）.
    pub const HAOLINGTIANXIA: u32 = 27;
    /// 敕令（国战：对所有有暗置武将的角色使用；各目标选明置并摸一、失去1点体力
    /// 或弃置一张装备牌）.
    pub const CHILING: u32 = 28;
    /// 出其不意（展示目标一张手牌，与此牌花色不同则造成 1 点伤害）.
    pub const CHUQIBUYI: u32 = 29;
}

const YOUDISHENRU_SAVED_MAGIC: u32 = 0x5944_5352;
pub(crate) const YOUDISHENRU_FORCE_CHOICE: u32 = 0x5944_5343;

pub(crate) fn encode_youdishenru_saved(
    protected: PlayerId,
    source: PlayerId,
    incoming: CardId,
    kind: crate::state::ContinuationKind,
    payload: &[u32],
) -> Vec<u32> {
    let mut saved = vec![
        YOUDISHENRU_SAVED_MAGIC,
        protected.raw(),
        source.raw(),
        incoming.raw(),
        u32::from(kind.tag()),
        payload.len().min(u32::MAX as usize) as u32,
    ];
    saved.extend_from_slice(payload);
    saved
}

fn parse_youdishenru_saved(
    saved: &[u32],
) -> Result<
    (
        PlayerId,
        PlayerId,
        CardId,
        crate::state::ContinuationKind,
        &[u32],
    ),
    EffectError,
> {
    if saved.first() != Some(&YOUDISHENRU_SAVED_MAGIC) {
        return Err(EffectError::EngineFault(
            "bad 诱敌深入 continuation marker".into(),
        ));
    }
    let len = *saved.get(5).unwrap_or(&u32::MAX) as usize;
    if saved.len() != 6usize.saturating_add(len) {
        return Err(EffectError::EngineFault(
            "truncated 诱敌深入 continuation".into(),
        ));
    }
    Ok((
        PlayerId::new(*saved.get(1).unwrap_or(&0)),
        PlayerId::new(*saved.get(2).unwrap_or(&0)),
        CardId::new(*saved.get(3).unwrap_or(&0)),
        crate::state::ContinuationKind::from_tag(*saved.get(4).unwrap_or(&0) as u16),
        &saved[6..],
    ))
}

/// Use a physical or materialized 诱敌深入 against the exact suspended incoming Sha.
pub(crate) fn use_youdishenru_for_saved_sha(
    tx: &mut Transaction,
    protected: PlayerId,
    source: PlayerId,
    incoming: CardId,
    youdi_card: CardId,
    saved_kind: crate::state::ContinuationKind,
    saved_payload: &[u32],
) -> Result<(), EffectError> {
    let valid = tx.state().cards.get(youdi_card).is_some_and(|card| {
        card.kind == CardKind::TrickYoudishenru
            && tx
                .state()
                .playable_hand_cards(protected)
                .contains(&youdi_card)
    });
    if !valid || source == protected || incoming.is_none() {
        return Err(EffectError::Illegal(
            "invalid 诱敌深入 response context".into(),
        ));
    }
    let saved = encode_youdishenru_saved(protected, source, incoming, saved_kind, saved_payload);
    run_effects_then_wuxie(
        tx,
        vec![EffectRequest::UseCard {
            player: protected,
            card: youdi_card,
            targets: Vec::new(),
        }],
        protected,
        trick_effect::YOUDISHENRU,
        protected,
        &saved,
    )
}

fn resume_youdishenru_incoming(
    tx: &mut Transaction,
    saved: &[u32],
    cancel_target: bool,
) -> Result<(), EffectError> {
    let (protected, source, incoming, kind, payload) = parse_youdishenru_saved(saved)?;
    match kind {
        crate::state::ContinuationKind::ResumePlayedSha
        | crate::state::ContinuationKind::ResumePlayedShaQueue => {
            {
                let resolution = &mut tx.state_mut().resolution;
                resolution.resume_kind = kind;
                resolution.resume_payload = payload.to_vec();
            }
            if cancel_target
                && !tx
                    .state_mut()
                    .resolution
                    .exclude_saved_sha_target(incoming, protected)
            {
                return Err(EffectError::EngineFault(
                    "诱敌深入 lost its incoming Sha target".into(),
                ));
            }
            Ok(())
        }
        crate::state::ContinuationKind::RespondToSha => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault(
                    "bad saved single-target Sha response".into(),
                ));
            }
            if cancel_target {
                tx.state_mut().resolution.resume_kind =
                    crate::state::ContinuationKind::FinishPlayedSha;
                tx.state_mut().resolution.resume_payload =
                    vec![source.raw(), incoming.raw(), payload[5]];
            } else {
                tx.state_mut().resolution.resume_kind = kind;
                tx.state_mut().resolution.resume_payload = payload.to_vec();
            }
            Ok(())
        }
        crate::state::ContinuationKind::RespondAoeQueue => {
            let mut next = payload.to_vec();
            if next.first() != Some(&3) || next.len() < 8 {
                return Err(EffectError::EngineFault(
                    "bad saved multi-target Sha response".into(),
                ));
            }
            if cancel_target {
                next[3] = next[3].saturating_add(1);
                next[7] = 0;
            }
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::RespondAoeQueue;
            tx.state_mut().resolution.resume_payload = next;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "诱敌深入 cannot resume this continuation".into(),
        )),
    }
}

fn begin_youdishenru_force_loop(
    tx: &mut Transaction,
    protected: PlayerId,
    source: PlayerId,
    saved: &[u32],
) -> Result<(), EffectError> {
    let both_alive = tx
        .state()
        .players
        .get(protected)
        .is_some_and(|player| player.alive)
        && tx
            .state()
            .players
            .get(source)
            .is_some_and(|player| player.alive && !player.dying);
    if !both_alive {
        return resume_youdishenru_incoming(tx, saved, true);
    }
    let options: Vec<_> = prompted_sha_source_cards(tx.state(), source)
        .into_iter()
        .filter(|card| prompted_sha_targets_unlimited(tx, source, *card).contains(&protected))
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ContinueYoudishenru {
                action: 2,
                protected,
                source,
                forced_card: CardId::NONE,
                saved: saved.to_vec(),
            });
        return run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: protected,
                target: source,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    let mut payload = vec![
        YOUDISHENRU_FORCE_CHOICE,
        protected.raw(),
        source.raw(),
        saved.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend_from_slice(saved);
    crate::effect::open_choice(
        tx,
        source,
        0,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn resolve_youdishenru_force_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let protected = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let len = *payload.get(3).unwrap_or(&u32::MAX) as usize;
    let saved = payload
        .get(4..4usize.saturating_add(len))
        .ok_or_else(|| EffectError::Illegal("bad 诱敌深入 forced-Sha choice".into()))?;
    if payload.first() != Some(&YOUDISHENRU_FORCE_CHOICE)
        || payload.len() != 4 + len
        || observer != source
        || selections.len() > 1
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal(
            "stale 诱敌深入 forced-Sha choice".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ContinueYoudishenru {
                action: 2,
                protected,
                source,
                forced_card: CardId::NONE,
                saved: saved.to_vec(),
            });
        return run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: protected,
                target: source,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    let forced_card = CardId::new(selections[0]);
    if !prompted_sha_source_cards(tx.state(), source).contains(&forced_card)
        || !prompted_sha_targets_unlimited(tx, source, forced_card).contains(&protected)
    {
        return Err(EffectError::Illegal("invalid 诱敌深入 forced Sha".into()));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ContinueYoudishenru {
            action: 1,
            protected,
            source,
            forced_card,
            saved: saved.to_vec(),
        });
    play_prompted_sha_source_unlimited(tx, source, forced_card, &[protected])
}

pub(crate) fn continue_youdishenru(
    tx: &mut Transaction,
    action: u8,
    protected: PlayerId,
    source: PlayerId,
    forced_card: CardId,
    saved: &[u32],
) -> Result<(), EffectError> {
    match action {
        0 => {
            let (saved_protected, saved_source, incoming, saved_kind, saved_payload) =
                parse_youdishenru_saved(saved)?;
            if saved_protected != protected || saved_source != source {
                return Err(EffectError::EngineFault(
                    "诱敌深入 virtual response context changed".into(),
                ));
            }
            let card =
                crate::effect::materialize_virtual_card(tx, protected, CardKind::TrickYoudishenru)?;
            use_youdishenru_for_saved_sha(
                tx,
                protected,
                source,
                incoming,
                card,
                saved_kind,
                saved_payload,
            )
        }
        1 => {
            let damaged = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| record.source == source && record.card == forced_card)
                .is_some_and(|record| record.damaged_targets.contains(&protected));
            if damaged {
                resume_youdishenru_incoming(tx, saved, true)
            } else {
                begin_youdishenru_force_loop(tx, protected, source, saved)
            }
        }
        2 => resume_youdishenru_incoming(tx, saved, true),
        _ => Err(EffectError::EngineFault(
            "unknown 诱敌深入 continuation action".into(),
        )),
    }
}

/// Cards that are or can be converted to 无懈可击.
pub(crate) fn wuxie_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    wuxie_tokens_for_card(state, player, CardId::NONE)
}

pub(crate) fn wuxie_tokens_for_card(
    state: &crate::state::GameState,
    player: PlayerId,
    root_card: CardId,
) -> Vec<u32> {
    let mut out = response_tokens(state, player, "wuxie", |kind| kind == CardKind::TrickWuxie);
    let colors = crate::query::potsongyan_response_colors(state, root_card, player);
    if colors == 0 {
        return out;
    }
    for zone_kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Expansion] {
        let Some(zone) = state
            .zones
            .find(zone_kind, player)
            .and_then(|id| state.zones.get(id))
        else {
            continue;
        };
        for card in &zone.cards {
            if !out.contains(&card.raw())
                && !crate::query::card_color_locked_for_card(state, player, *card)
                && !crate::query::response_color_locked_for_current_card(state, player, *card)
                && crate::query::effective_card_color_mask(state, player, *card) & colors != 0
            {
                out.push(card.raw());
            }
        }
    }
    out
}

pub(crate) fn wuxie_tokens_for_payload(
    state: &crate::state::GameState,
    player: PlayerId,
    payload: &[u32],
) -> Vec<u32> {
    let ask_len = payload.get(2).copied().unwrap_or(0) as usize;
    let effect_off = 3usize.saturating_add(ask_len);
    wuxie_tokens_for_card(
        state,
        player,
        effect_args_source_card(payload.get(effect_off + 5..).unwrap_or_default()),
    )
}

/// Seat-ordered alive players starting from `start` (inclusive).
fn alive_from_seat(tx: &Transaction, start: PlayerId) -> Vec<PlayerId> {
    let order = tx.state().players.alive_by_seat();
    if order.is_empty() {
        return order;
    }
    let pos = order.iter().position(|p| *p == start).unwrap_or(0);
    let mut out = order[pos..].to_vec();
    out.extend_from_slice(&order[..pos]);
    out
}

/// Open 无懈 window before deferred trick effect resolves.
///
/// payload: `[nullified, ask_idx, ask_len, p0.., effect_tag, actor, respond_to, response_kind,
/// response_suit, ...args]`
pub(crate) fn begin_wuxie_window(
    tx: &mut Transaction,
    ask_start: PlayerId,
    effect_tag: u32,
    actor: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    begin_wuxie_window_with_parity(
        tx,
        ask_start,
        0,
        effect_tag,
        actor,
        actor,
        response_kind,
        response_suit,
        effect_args,
    )
}

/// Like [`begin_wuxie_window`] but carries current nullify parity (after a 无懈).
pub(crate) fn begin_wuxie_window_with_parity(
    tx: &mut Transaction,
    ask_start: PlayerId,
    nullified: u32,
    effect_tag: u32,
    actor: PlayerId,
    respond_to: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    let askers = alive_from_seat(tx, ask_start);
    let mut payload = vec![nullified & 1, 0, askers.len() as u32];
    payload.extend(askers.iter().map(|p| p.raw()));
    payload.push(effect_tag);
    payload.push(actor.raw());
    payload.push(respond_to.raw());
    payload.push(response_kind.tag() as u32);
    payload.push(super::judge::suit_tag(response_suit));
    payload.extend_from_slice(effect_args);
    tx.state_mut().resolution.continuation_kind = crate::state::ContinuationKind::WuxieWindow;
    tx.state_mut().resolution.continuation_payload = payload;
    advance_wuxie_window(tx)
}

/// Advance / open next 无懈 ask, or finish into deferred effect.
pub(crate) fn advance_wuxie_window(tx: &mut Transaction) -> Result<(), EffectError> {
    loop {
        let payload = tx.state().resolution.continuation_payload.clone();
        if payload.len() < 5 {
            tx.state_mut().resolution.clear_choice();
            return Ok(());
        }
        let ask_idx = payload[1] as usize;
        let ask_len = payload[2] as usize;
        if ask_idx >= ask_len || 3 + ask_len > payload.len() {
            return finish_wuxie_window(tx, &payload);
        }
        let player = PlayerId::new(payload[3 + ask_idx]);
        let alive = tx
            .state()
            .players
            .get(player)
            .map(|p| p.alive && !p.dying)
            .unwrap_or(false);
        if !alive {
            tx.state_mut().resolution.continuation_payload[1] = (ask_idx + 1) as u32;
            continue;
        }
        let effect_off = 3 + ask_len;
        let respond_to = PlayerId::new(*payload.get(effect_off + 2).unwrap_or(&0));
        let response_kind = payload
            .get(effect_off + 3)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .unwrap_or(CardKind::Other);
        let response_suit = super::judge::suit_from_tag(*payload.get(effect_off + 4).unwrap_or(&1));
        let source_card =
            effect_args_source_card(payload.get(effect_off + 5..).unwrap_or_default());
        // tanhu3（批 170）：独立锦囊队列的 effect_args 布局统一
        // `[MAGIC, card, actor, kind, suit, effect_tag, 0, len, targets...]`——
        // 目标集合含「拥有被声明技能且挂有对应声明」的角色时，无懈不可响应。
        let wuxie_immune_targets = (|| {
            let args = payload.get(effect_off + 5..).unwrap_or_default();
            if args.first().copied() != Some(crate::command::play::INDEPENDENT_TRICK_QUEUE_MAGIC)
            {
                return None;
            }
            let len = *args.get(7).unwrap_or(&0) as usize;
            let targets: Vec<_> = args
                .get(8..8 + len)?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            Some(targets)
        })();
        let tokens = if crate::query::card_use_unrespondable_for_kind(
            tx.state(),
            respond_to,
            response_kind,
            response_suit,
        ) || crate::query::card_use_unrespondable_within_distance(
            tx.state(),
            respond_to,
            player,
        ) || crate::query::potshishi_blocks_response(
            tx.state(),
            respond_to,
            source_card,
            player,
        ) {
            Vec::new()
        } else if let Some(targets) = wuxie_immune_targets {
            if crate::query::wuxie_blocked_by_target_skill_declarations(tx.state(), &targets) {
                Vec::new()
            } else {
                wuxie_tokens_for_card(tx.state(), player, source_card)
            }
        } else {
            wuxie_tokens_for_card(tx.state(), player, source_card)
        };
        if tokens.is_empty() {
            tx.state_mut().resolution.continuation_payload[1] = (ask_idx + 1) as u32;
            continue;
        }
        let mut p = payload;
        p[1] = ask_idx as u32;
        crate::effect::open_choice(
            tx,
            player,
            0,
            1,
            true,
            tokens,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::WuxieWindow,
            p,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

fn finish_wuxie_window(tx: &mut Transaction, payload: &[u32]) -> Result<(), EffectError> {
    let nullified = *payload.first().unwrap_or(&0);
    let ask_len = *payload.get(2).unwrap_or(&0) as usize;
    let effect_off = 3 + ask_len;
    let effect_tag = *payload.get(effect_off).unwrap_or(&0);
    let actor = PlayerId::new(*payload.get(effect_off + 1).unwrap_or(&0));
    let args = if payload.len() > effect_off + 5 {
        &payload[effect_off + 5..]
    } else {
        &[]
    };
    let marker = args
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC);
    let (effect_args, source_card) = marker.map_or((args, CardId::NONE), |index| {
        (
            &args[..index],
            CardId::new(*args.get(index + 1).unwrap_or(&0)),
        )
    });

    tx.state_mut().resolution.continuation_kind = crate::state::ContinuationKind::None;
    tx.state_mut().resolution.continuation_payload.clear();
    tx.state_mut().resolution.clear_choice();

    if nullified % 2 == 1 {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 220, // trick nullified by 无懈
                payload: vec![effect_tag, actor.raw()],
            }],
        )?;
        if effect_tag == trick_effect::YOUDISHENRU {
            return resume_youdishenru_incoming(tx, effect_args, false);
        }
        finish_independent_trick_target(tx, effect_args)?;
        return crate::effect::finish_pending_card_replays_if_idle(tx);
    }
    apply_deferred_trick_with_extras(tx, effect_tag, actor, effect_args, source_card)?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

/// Apply trick effect after 无懈 window clears without odd nullify, honoring
/// queued 额外结算 (extra_effect_count) passes (clanfenshi family, ABI 649).
pub(crate) fn apply_deferred_trick_with_extras(
    tx: &mut Transaction,
    effect_tag: u32,
    actor: PlayerId,
    args: &[u32],
    source_card: CardId,
) -> Result<(), EffectError> {
    apply_deferred_trick(tx, effect_tag, actor, args, source_card)?;
    let extras = tx
        .state()
        .extra_effect_count
        .get(&source_card.raw())
        .copied()
        .unwrap_or(0);
    for _ in 0..extras.min(2) {
        apply_deferred_trick(tx, effect_tag, actor, args, source_card)?;
    }
    if source_card.is_some() {
        tx.state_mut().extra_effect_count.remove(&source_card.raw());
    }
    Ok(())
}

/// Apply trick effect after 无懈 window clears without odd nullify.
pub(crate) fn apply_deferred_trick(
    tx: &mut Transaction,
    effect_tag: u32,
    actor: PlayerId,
    args: &[u32],
    source_card: CardId,
) -> Result<(), EffectError> {
    let source_card = if source_card.is_none() {
        effect_args_source_card(args)
    } else {
        source_card
    };
    match effect_tag {
        trick_effect::WUZHONG => run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: actor,
                count: 2,
            }],
        )
        .map(|_| ()),
        trick_effect::YUANJIAO => {
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    },
                    EffectRequest::DrawCards {
                        player: actor,
                        count: 3,
                    },
                ],
            )
            .map(|_| ())
        }
        trick_effect::TIAOJIYANMEI => {
            let [left_raw, right_raw] = args else {
                return Err(EffectError::ContentFault(
                    "调剂盐梅 target payload is invalid".into(),
                ));
            };
            let left_excluded = left_raw & SAVED_TARGET_EXCLUDED_MASK != 0;
            let right_excluded = right_raw & SAVED_TARGET_EXCLUDED_MASK != 0;
            let left = PlayerId::new(left_raw & !SAVED_TARGET_EXCLUDED_MASK);
            let right = PlayerId::new(right_raw & !SAVED_TARGET_EXCLUDED_MASK);
            let left_hand = tx.state().hand_count(left);
            let right_hand = tx.state().hand_count(right);
            let (lower, higher) = if left_hand < right_hand {
                (left, right)
            } else if right_hand < left_hand {
                (right, left)
            } else {
                return Ok(());
            };
            let mut effects = Vec::new();
            let lower_excluded = if lower == left {
                left_excluded
            } else {
                right_excluded
            };
            let higher_excluded = if higher == left {
                left_excluded
            } else {
                right_excluded
            };
            if !lower_excluded {
                effects.push(EffectRequest::DrawCards {
                    player: lower,
                    count: 1,
                });
            }
            if !higher_excluded {
                effects.push(EffectRequest::BeginTiaojiyanmeiDiscard {
                    actor,
                    left,
                    right,
                    higher,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        trick_effect::MANTIANGUOHAI => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "瞒天过海 missing independent target queue".into(),
                ));
            };
            let options: Vec<_> = crate::query::gainable_target_cards(
                tx.state(),
                actor,
                target,
                rule_api::TargetCardScope::HandEquipJudge,
            )
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                MANTIANGUOHAI_CUSTOM_CHOICE,
                0,
                actor.raw(),
                target.raw(),
                queue.len() as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::YOUDISHENRU => {
            let (protected, source, _, _, _) = parse_youdishenru_saved(args)?;
            begin_youdishenru_force_loop(tx, protected, source, args)
        }
        trick_effect::DIAOBINGQIANJIANG => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "调兵遣将 missing independent target queue".into(),
                ));
            };
            let len = *queue.get(6).unwrap_or(&0) as usize;
            let extension = 7usize.saturating_add(len);
            if queue.get(extension) != Some(&DIAOBING_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "调兵遣将 pool marker is missing".into(),
                ));
            }
            let count = *queue.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = queue
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("调兵遣将 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        tx.state()
                            .zones
                            .get(state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
                .collect();
            if revealed.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::RESOLVE_WDB_YANGYI_CHOICE,
                2,
                queue.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                target,
                0,
                1,
                true,
                revealed.iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WUGU => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "五谷丰登 missing independent target queue".into(),
                ));
            };
            let len = *queue.get(6).unwrap_or(&0) as usize;
            let extension = 7usize.saturating_add(len);
            if queue.get(extension) != Some(&WUGU_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "五谷丰登 pool marker is missing".into(),
                ));
            }
            let count = *queue.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = queue
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        tx.state()
                            .zones
                            .get(state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
                .collect();
            if revealed.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::RESOLVE_WDB_ZAOZHIRENJUN_CHOICE,
                2,
                queue.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                revealed.iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WANGMEIZHIKE => {
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            run_effects(
                tx,
                vec![EffectRequest::ResolveWdbZaozhiRenjun {
                    player: actor,
                    target,
                    card: source_card,
                    cards: Vec::new(),
                    action: 2,
                    skill: engine_api::SkillId::NONE,
                }],
            )
            .map(|_| ())
        }
        trick_effect::CHUQIBUYI => {
            // 出其不意：展示目标一张手牌（确定性首张近似），与源牌花色
            // 不同则对其造成 1 点伤害。
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let shown = tx.state().hand_cards(target).first().copied();
            let source_suit = tx
                .state()
                .cards
                .get(source_card)
                .map(|card| card.suit);
            if let (Some(shown), Some(source_suit)) = (shown, source_suit) {
                let shown_suit = tx
                    .state()
                    .cards
                    .get(shown)
                    .map(|card| card.suit);
                if shown_suit.is_some_and(|suit| suit != source_suit) {
                    return run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: actor,
                            target,
                            amount: 1,
                            is_lose_hp: false,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                        args,
                    );
                }
            }
            finish_independent_trick_target(tx, args)
        }
        trick_effect::YIYI => {
            // 以逸待劳（国战）：目标摸两张牌，然后弃置两张牌。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            // 弃牌 choice 挂起时，已保存的 IndependentTrickQueue resume 会在 choice
            // 完成后经 resume_interrupted_continuation 推进到下一目标。
            run_effects_then_finish_independent_trick_target(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player: target,
                        count: 2,
                    },
                    EffectRequest::DiscardHandEquipCount {
                        player: target,
                        count: 2,
                        skill: engine_api::SkillId::NONE,
                    },
                ],
                args,
            )
        }
        trick_effect::SHUIYANQIJUNX => {
            // 水淹七军（国战）：目标选择弃置全部装备牌或受 1 点雷电伤害。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            if crate::query::discardable_equipment_cards(tx.state(), target, target).is_empty() {
                // 无装备可弃：直接受 1 点雷电伤害。
                return run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: actor,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::THUNDER,
                    }],
                    args,
                );
            }
            let mut payload = vec![
                crate::effect::SHUIYANQIJUNX_CUSTOM_CHOICE,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::LULITONGXIN => {
            // 戮力同心（国战）：目标未横置 → 横置；已横置 → 摸一张牌。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let linked = player_has_mark(tx, target, crate::content::MARK_CHAIN);
            let effects = if linked {
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                }]
            } else {
                vec![EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                    delta: 1,
                }]
            };
            run_effects_then_finish_independent_trick_target(tx, effects, args)
        }
        trick_effect::KEFUZHONGYUAN => {
            // 克复中原（国战）：目标选择 1=视为使用杀（蜀势力伤害基数+1）；2=摸牌
            // （蜀势力摸两张，否则一张）。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::KEFUZHONGYUAN_CUSTOM_CHOICE,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::GUGUOANBANG => {
            // 固国安邦（国战）：摸八张牌，然后弃置至少六张手牌；吴势力可选将弃牌
            // 交给其他吴角色（每名至多两张，单轮近似）。三阶段 Custom 续体。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: 8,
                }],
            )?;
            let hand = tx.state().hand_cards(target);
            if hand.len() < 6 {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                1,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                6,
                hand.len().min(u8::MAX as usize) as u8,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HUOSHAOLIANYING => {
            // 火烧连营（国战）：对目标造成 1 点火属性伤害。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            run_effects_then_finish_independent_trick_target(
                tx,
                vec![EffectRequest::DealDamage {
                    source: actor,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                }],
                args,
            )
        }
        trick_effect::LIANJUNSHENGYAN => {
            // 联军盛宴（国战）：非己目标摸 1 并重置（解除横置）；自己按敌方势力人数
            // 分配「摸 Y 张 + 回 X−Y 点」（Y∈[0,X]，回复量以缺体为上限）。
            let (queue_payload, target) = queued_independent_trick(args)
                .map(|(payload, target)| (Some(payload), target))
                .unwrap_or((None, PlayerId::new(*args.first().unwrap_or(&0))));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            if target != actor {
                let mut effects = vec![EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                }];
                if player_has_mark(tx, target, crate::content::MARK_CHAIN) {
                    effects.push(EffectRequest::RemoveMark {
                        player: target,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        amount: 0,
                    });
                }
                return run_effects_then_finish_independent_trick_target(tx, effects, args);
            }
            // 自己：X = 敌方势力人数 = 队列目标数 − 1（含自己）。
            let x = queue_payload
                .and_then(|payload| payload.get(6).copied())
                .map(|len| len.saturating_sub(1))
                .unwrap_or(0);
            if x == 0 {
                return finish_independent_trick_target(tx, args);
            }
            let participant = tx
                .state()
                .players
                .get(actor)
                .ok_or_else(|| EffectError::Illegal("user missing".into()))?;
            let damaged = participant
                .max_hp
                .saturating_sub(participant.hp.max(0) as u32);
            if damaged == 0 {
                return run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player: actor,
                        count: x,
                    }],
                    args,
                );
            }
            let cap = x.min(damaged);
            let mut payload = vec![
                crate::effect::LIANJUNSHENGYAN_CUSTOM_CHOICE,
                actor.raw(),
                x,
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                (0..=cap).collect(),
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WENHELUANWU => {
            // 文和乱武（国战）：目标展示手牌；使用者二选一：①令其弃置两张类型不同的
            // 手牌（仅一种类型时退化为弃一张）②弃置其一张手牌；然后若目标为群势力
            // 且空手，则摸至当前体力（至多5）。选择权在使用者，故 choice 挂到 actor。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::WENHELUANWU_CUSTOM_CHOICE,
                1, // stage 1 = 使用者二选一
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HAOLINGTIANXIA => {
            // 号令天下（国战）：对目标外所有角色（按座次）依次二选一。链状态以
            // 独立锦囊队列格式持久化（effect_tag = HAOLINGTIANXIA），每个步骤经
            // IndependentTrickQueue resume 续链（杀以直接伤害近似，ADR 0623）。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let chain: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != target)
                .collect();
            if chain.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let source_card_state = tx
                .state()
                .cards
                .get(source_card)
                .ok_or_else(|| EffectError::Illegal("bad trick card".into()))?;
            let kind = crate::query::effective_card_kind(tx.state(), actor, source_card_state);
            let suit = crate::query::effective_card_suit(tx.state(), actor, source_card_state);
            let mut queue = vec![
                source_card.raw(),
                actor.raw(),
                kind.tag() as u32,
                super::judge::suit_tag(suit),
                trick_effect::HAOLINGTIANXIA,
                0,
                chain.len() as u32,
            ];
            queue.extend(chain.iter().map(|candidate| candidate.raw()));
            queue.push(target.raw());
            queue.extend_from_slice(args);
            advance_haolingtianxia_player(tx, &queue)
        }
        trick_effect::CHILING => {
            // 敕令（国战）：目标选 ①明置一张武将牌并摸一张 ②失去1点体力 ③弃置一张
            // 装备牌（有装备牌时可选）。选择权在目标自身，choice 挂到 target。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.unseen[0] || player.unseen[1])
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut choices = vec![1, 2];
            if !equip_typed_zone_cards(tx, target).is_empty() {
                choices.push(3);
            }
            let mut payload = vec![
                crate::effect::CHILING_CUSTOM_CHOICE,
                1, // stage 1 = 目标三选一
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                choices,
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::GUOHE => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let tokens = discardable_target_zone_card_tokens(tx, actor, target);            if tokens.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![target.raw(), actor.raw(), 0];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::PickTargetCard,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::SHUNSHOU => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let tokens = target_zone_card_tokens(tx, target, true, true, false);
            if tokens.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![target.raw(), actor.raw(), 1];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::PickTargetCard,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HUOGONG => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let thand = tx.state().hand_cards(target);
            if thand.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let tokens: Vec<u32> = thand.iter().map(|c| c.raw()).collect();
            let mut payload = vec![actor.raw(), target.raw()];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::HuogongShow,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::JUEDOU => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if tx
                .state()
                .players
                .get(target)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return finish_independent_trick_target(tx, args);
            }
            let saved_tail;
            let tail: &[u32] = if queued_independent_trick(args).is_some() {
                args
            } else if source_card != CardId::NONE {
                saved_tail = vec![SAVED_WUXIE_SOURCE_CARD_MAGIC, source_card.raw()];
                &saved_tail
            } else {
                &[]
            };
            begin_juedou_with_tail(tx, actor, target, tail)
        }
        trick_effect::NANMAN => {
            let len = *args.first().unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(1..1 + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .filter(|p| tx.state().players.get(*p).map(|x| x.alive).unwrap_or(false))
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            begin_aoe_queue(
                tx,
                1,
                actor,
                saved_aoe_base_damage(args, len),
                &targets,
                source_card,
            )
        }
        trick_effect::WANJIAN => {
            let len = *args.first().unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(1..1 + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .filter(|p| tx.state().players.get(*p).map(|x| x.alive).unwrap_or(false))
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            begin_aoe_queue(
                tx,
                2,
                actor,
                saved_aoe_base_damage(args, len),
                &targets,
                source_card,
            )
        }
        trick_effect::TAOYUAN => {
            let alive = optional_trick_targets(tx, args)
                .unwrap_or_else(|| tx.state().players.alive_by_seat());
            let mut q = Vec::new();
            for p in alive {
                let pl = tx.state().players.get(p).unwrap();
                if pl.hp < pl.max_hp as i32 {
                    q.push(EffectRequest::RecoverHp {
                        source: actor,
                        player: p,
                        amount: 1,
                    });
                }
            }
            if q.is_empty() {
                return Ok(());
            }
            run_effects(tx, q).map(|_| ())
        }
        trick_effect::TIESUO => {
            let (len_index, targets_start) = if args.first() == Some(&TIESUO_SAVED_TARGETS_MAGIC) {
                (2, 3)
            } else {
                (0, 1)
            };
            let len = *args.get(len_index).unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(targets_start..targets_start + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .collect();
            let mut q = Vec::new();
            for t in targets {
                if tx.state().players.get(t).map(|p| !p.alive).unwrap_or(true) {
                    continue;
                }
                if player_has_mark(tx, t, crate::content::MARK_CHAIN) {
                    q.push(EffectRequest::RemoveMark {
                        player: t,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        amount: 0,
                    });
                } else {
                    q.push(EffectRequest::AddMark {
                        player: t,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        delta: 1,
                    });
                }
            }
            if q.is_empty() {
                return Ok(());
            }
            run_effects(tx, q).map(|_| ())
        }
        trick_effect::JIEDAO => {
            let holder = PlayerId::new(*args.first().unwrap_or(&0));
            let victim = PlayerId::new(*args.get(1).unwrap_or(&0));
            if tx
                .state()
                .players
                .get(holder)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return Ok(());
            }
            let mut tokens = vec![0u32];
            if sha_in_range(tx, holder, victim) {
                tokens.push(1);
            }
            crate::effect::open_choice(
                tx,
                holder,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::JiedaoChoice,
                vec![actor.raw(), holder.raw(), victim.raw()],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::DELAYED_PLACE => {
            // args: [card, target, kind_tag] — card currently in discard after use
            let cid = CardId::new(*args.first().unwrap_or(&0));
            let target = PlayerId::new(*args.get(1).unwrap_or(&0));
            let kind_tag = *args.get(2).unwrap_or(&0) as u8;
            let kind = match kind_tag {
                30 => CardKind::TrickLe,
                31 => CardKind::TrickBingliang,
                32 => CardKind::TrickShandian,
                _ => CardKind::Other,
            };
            if tx
                .state()
                .players
                .get(target)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return Ok(()); // target dead → stay discarded
            }
            if tx.state().disabled_judge_zones.contains(&target) {
                return Ok(());
            }
            // Re-check duplicate delayed (race with other effects)
            if kind != CardKind::Other && has_delayed_of_kind(tx, target, kind) {
                return Ok(());
            }
            let Some(card) = tx.state().cards.get(cid).cloned() else {
                return Ok(());
            };
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            if card.zone != discard {
                return Ok(()); // already moved elsewhere
            }
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::Illegal("no judge".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![cid],
                    to: judge,
                    from: Some(discard),
                }],
            )?;
            // `UseCard` temporarily stages delayed tricks in the discard pile. The generic
            // use/response-discard counter observes that staging move; a successfully placed
            // delayed trick did not actually finish in discard and must therefore be removed
            // from the turn total. A cancelled/invalid delayed trick stays discarded and keeps
            // the count.
            if tx.state().content.tracks_turn_use_response_discard_count() {
                let count = tx
                    .state()
                    .turn
                    .flag(crate::state::turn_flag::USE_RESPONSE_DISCARDED);
                tx.state_mut().turn.set_flag(
                    crate::state::turn_flag::USE_RESPONSE_DISCARDED,
                    count.saturating_sub(1),
                );
            }
            if let Some(c) = tx.state_mut().cards.get_mut(cid) {
                c.face_up = true;
                c.owner = target;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn saved_aoe_base_damage(args: &[u32], target_count: usize) -> u32 {
    let extension_at = 1usize.saturating_add(target_count);
    if args.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
        1u32.saturating_add(*args.get(extension_at + 1).unwrap_or(&0))
    } else {
        1
    }
}

fn optional_trick_targets(tx: &Transaction, args: &[u32]) -> Option<Vec<PlayerId>> {
    let len = *args.first()? as usize;
    Some(
        args.get(1..1 + len)?
            .iter()
            .map(|raw| PlayerId::new(*raw))
            .filter(|player| {
                tx.state()
                    .players
                    .get(*player)
                    .map(|state| state.alive)
                    .unwrap_or(false)
            })
            .collect(),
    )
}

/// Collect card tokens from target zones for 过河/顺手.
pub(crate) fn target_zone_card_tokens(
    tx: &Transaction,
    target: PlayerId,
    hand: bool,
    equip: bool,
    judge: bool,
) -> Vec<u32> {
    let mut out = Vec::new();
    if hand {
        out.extend(tx.state().hand_cards(target).iter().map(|c| c.raw()));
    }
    if equip {
        if let Some(eq) = tx.state().zones.find(ZoneKind::Equip, target) {
            if let Some(z) = tx.state().zones.get(eq) {
                out.extend(z.cards.iter().map(|c| c.raw()));
            }
        }
    }
    if judge {
        if let Some(jz) = tx.state().zones.find(ZoneKind::Judge, target) {
            if let Some(z) = tx.state().zones.get(jz) {
                out.extend(z.cards.iter().map(|c| c.raw()));
            }
        }
    }
    out
}

pub(crate) fn discardable_target_zone_card_tokens(
    tx: &Transaction,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<u32> {
    target_zone_card_tokens(tx, target, true, true, true)
        .into_iter()
        .filter(|raw| {
            !crate::query::card_discard_protected_from(tx.state(), actor, target, CardId::new(*raw))
        })
        .collect()
}

/// Equipment-typed cards in a player's hand/equip zones (上游
/// `hasCards("he", { type: "equip" })`), used by 敕令's discard-equipment option.
pub(crate) fn equip_typed_zone_cards(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    target_zone_card_tokens(tx, player, true, true, false)
        .into_iter()
        .filter(|raw| {
            tx.state()
                .cards
                .get(CardId::new(*raw))
                .is_some_and(|card| card.kind.is_equip())
        })
        .collect()
}

pub(crate) fn sha_in_range(tx: &Transaction, from: PlayerId, to: PlayerId) -> bool {
    if crate::query::card_distance_unlimited_for(tx.state(), from) {
        return true;
    }
    let mut ctx = QueryContext::new();
    let dist = query(tx.state(), &QueryRequest::Distance { from, to }, &mut ctx);
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: from },
        &mut ctx,
    );
    matches!(
        (dist, range),
        (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
    )
}

fn prompted_sha_source(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> Option<(CardKind, Option<(SkillId, crate::content::ViewAsDef)>)> {
    let participant = state.players.get(player)?;
    if !participant.alive || participant.dying {
        return None;
    }
    let card_state = state.cards.get(card)?;
    if crate::query::card_color_locked_for_card(state, player, card) {
        return None;
    }
    if state.playable_hand_cards(player).contains(&card) {
        let kind = crate::query::effective_card_kind(state, player, card_state);
        if kind.is_sha() && !crate::query::card_use_cancelled_for_card(state, player, card) {
            return Some((kind, None));
        }
    }
    for skill in &participant.skills {
        let Some(instance) = state.skills.get(*skill) else {
            continue;
        };
        if !state.skill_effectively_enabled(instance) {
            continue;
        }
        for rule in state.content.view_as_defs_for_skill(instance.def) {
            if rule.as_kind != "sha"
                || rule.source_zone.as_deref() == Some("none")
                || !matches!(rule.window.as_str(), "any" | "play_phase")
                || rule
                    .uses_per_turn
                    .is_some_and(|limit| instance.use_count >= u32::from(limit))
                || rule
                    .branch
                    .is_some_and(|branch| branch != instance.view_as_branch)
                || (rule.once_per_round
                    && instance.view_as_use_round == state.turn.round
                    && rule.branch.is_some_and(|branch| {
                        instance.view_as_used_branches & (1u32 << branch) != 0
                    }))
                || !crate::query::view_as_source_zone_matches(
                    state,
                    player,
                    rule.source_zone.as_deref(),
                    card_state,
                )
                || !crate::query::view_as_filter_matches(state, player, &rule.filter, card_state)
                || crate::query::card_use_cancelled_for_logical_kind(
                    state,
                    player,
                    card_state,
                    CardKind::Sha,
                )
            {
                continue;
            }
            return Some((CardKind::Sha, Some((*skill, rule))));
        }
    }
    None
}

/// Physical cards the player can currently supply as a prompted logical Sha.
pub(crate) fn prompted_sha_source_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .flat_map(|zone| {
            state
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .filter(|card| prompted_sha_source(state, player, *card).is_some())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

fn twtanfeng_sha_source_is_owned(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> bool {
    let Some(card_state) = state.cards.get(card) else {
        return false;
    };
    let in_hand_or_equip = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .any(|zone| zone == card_state.zone);
    in_hand_or_equip
        && card_state.owner == player
        && state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
        && !crate::query::card_use_cancelled_for_logical_kind(
            state,
            player,
            card_state,
            CardKind::Sha,
        )
}

/// Physical hand/equipment cards 探锋 may convert to Sha against `required_target`.
pub(crate) fn twtanfeng_sha_source_cards(
    tx: &Transaction,
    player: PlayerId,
    required_target: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .filter(|card| !twtanfeng_sha_targets(tx, player, required_target, *card).is_empty())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Legal normal-distance Sha targets for one 探锋 physical source; empty unless the owner is legal.
pub(crate) fn twtanfeng_sha_targets(
    tx: &Transaction,
    player: PlayerId,
    required_target: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    if !twtanfeng_sha_source_is_owned(tx.state(), player, card) {
        return Vec::new();
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let legal: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[*target])
                .is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, false).is_ok()
        })
        .collect();
    legal
        .contains(&required_target)
        .then_some(legal)
        .unwrap_or_default()
}

/// Use one 探锋 source as Sha after revalidating its mandatory owner target and normal range.
pub(crate) fn play_twtanfeng_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    required_target: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = twtanfeng_sha_targets(tx, player, required_target, card);
    let valid_shape = if crate::query::sha_targets_all_for(tx.state(), player) {
        targets == legal
    } else {
        !targets.is_empty()
            && targets.contains(&required_target)
            && targets.len() <= crate::query::sha_target_limit_for(tx.state(), player) as usize
            && targets.iter().all(|target| legal.contains(target))
            && targets
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == targets.len()
    };
    if !valid_shape {
        return Err(EffectError::Illegal("invalid Tanfeng Sha targets".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no Tanfeng user hand".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Tanfeng source disappeared".into()))?;
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card).is_some();
    let source_was_hand = card_state.zone == hand || borrowed;
    if !source_was_hand {
        crate::effect::move_one_card(tx, card, card_state.zone, hand, player, false)?;
    }
    play_as_kind(
        tx,
        player,
        card,
        CardKind::Sha,
        targets,
        true,
        false,
        true,
        source_was_hand,
    )
}

fn oldjiefan_sha_spec(
    state: &crate::state::GameState,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
) -> Option<(bool, Vec<PlayerId>)> {
    let (_, view_as) = prompted_sha_source(state, player, card)?;
    let card_state = state.cards.get(card)?;
    if current.is_none()
        || current == player
        || crate::query::card_targets_self_only_for(state, player)
        || crate::query::card_target_cancelled_for_logical_kind(
            state,
            player,
            current,
            card_state,
            CardKind::Sha,
        )
    {
        return None;
    }
    let ignore_distance = view_as
        .as_ref()
        .is_some_and(|(_, rule)| rule.ignore_distance);
    let target_legal = |target: PlayerId| {
        target != player
            && state
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && !crate::query::card_target_cancelled_for_logical_kind(
                state,
                player,
                target,
                card_state,
                CardKind::Sha,
            )
            && (ignore_distance
                || crate::query::sha_distance_unlimited_for(state, player)
                || crate::query::card_distance_unlimited_for_target(state, player, card, target)
                || crate::query::player_in_attack_range(state, player, target))
    };
    if !target_legal(current) {
        return None;
    }
    let targets = if crate::query::sha_targets_all_for(state, player) {
        state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| target_legal(*target))
            .collect()
    } else if crate::query::sha_target_limit_for(state, player) > 0 {
        vec![current]
    } else {
        return None;
    };
    targets
        .contains(&current)
        .then_some((ignore_distance, targets))
}

/// Physical cards that can supply old 解烦's Sha against the current turn player.
pub(crate) fn oldjiefan_sha_source_cards(
    state: &crate::state::GameState,
    player: PlayerId,
    current: PlayerId,
) -> Vec<CardId> {
    prompted_sha_source_cards(state, player)
        .into_iter()
        .filter(|card| oldjiefan_sha_spec(state, player, current, *card).is_some())
        .collect()
}

/// Use old 解烦's selected physical or view-as Sha through the ordinary response pipeline.
pub(crate) fn play_oldjiefan_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let (ignore_distance, targets) = oldjiefan_sha_spec(tx.state(), player, current, card)
        .ok_or_else(|| EffectError::Illegal("old Jiefan Sha source became illegal".into()))?;
    let (_, view_as) = prompted_sha_source(tx.state(), player, card)
        .ok_or_else(|| EffectError::Illegal("old Jiefan Sha source disappeared".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("missing old Jiefan Sha source".into()))?;
    let kind = view_as.as_ref().map_or_else(
        || crate::query::effective_card_kind(tx.state(), player, card_state),
        |_| CardKind::Sha,
    );
    validate_explicit_target_policy(tx, player, card_state, kind, &targets)?;
    validate_sha_targets_with_options(tx, player, card, &targets, ignore_distance)?;
    play_prompted_sha_source_impl(
        tx,
        player,
        card,
        &targets,
        ignore_distance,
        true,
        rule_api::damage_card_flag::OLDJIEFAN,
    )
}

/// Legal targets for a Xinzhongyong-prompted Sha. Ordinary target policies apply, but ordinary
/// user distance is replaced by the original skill owner's attack range.
pub(crate) fn prompted_sha_targets(
    tx: &Transaction,
    owner: PlayerId,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, _)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != owner)
        .filter(|target| sha_in_range(tx, owner, *target))
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, true).is_ok()
        })
        .collect()
}

fn clockwise_living_others(state: &crate::state::GameState, player: PlayerId) -> Vec<PlayerId> {
    let order = state.players.alive_by_seat();
    let Some(start) = order.iter().position(|candidate| *candidate == player) else {
        return Vec::new();
    };
    (1..order.len())
        .map(|offset| order[(start + offset) % order.len()])
        .collect()
}

/// Legal targets for a 乱武-prompted Sha. Only players at the minimum final distance from the
/// prompted user are eligible; ordinary Sha range and target policies still apply.
pub(crate) fn luanwu_sha_targets(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, _)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let candidates = clockwise_living_others(tx.state(), player);
    let distance = |target| {
        let mut context = QueryContext::new();
        match query(
            tx.state(),
            &QueryRequest::Distance {
                from: player,
                to: target,
            },
            &mut context,
        ) {
            QueryResponse::Int(value) => Some(value),
            _ => None,
        }
    };
    let Some(minimum) = candidates
        .iter()
        .filter_map(|target| distance(*target))
        .min()
    else {
        return Vec::new();
    };
    candidates
        .into_iter()
        .filter(|target| distance(*target) == Some(minimum))
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, false).is_ok()
        })
        .collect()
}

/// Use one revalidated physical source as a logical Sha without consuming the ordinary use count.
pub(crate) fn play_prompted_sha_source(
    tx: &mut Transaction,
    owner: PlayerId,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets(tx, owner, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// Use one revalidated physical or view-as source for 乱武 without consuming the ordinary Sha
/// use count.
pub(crate) fn play_luanwu_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = luanwu_sha_targets(tx, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// List legal targets for a prompted physical or view-as Sha while ignoring ordinary distance.
pub(crate) fn prompted_sha_targets_unlimited(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| validate_sha_target_with_options(tx, player, card, *target, true).is_ok())
        .collect()
}

/// Legal targets for a prompted physical or one-card view-as Sha under ordinary distance and
/// use-count rules.
pub(crate) fn prompted_sha_targets_normal(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, view_as)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let ignore_distance = view_as
        .as_ref()
        .is_some_and(|(_, rule)| rule.ignore_distance);
    let within_use_count = crate::query::sha_play_phase_use_count(tx.state())
        < crate::query::sha_use_limit_for_card(tx.state(), player, card);
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            within_use_count
                || crate::query::sha_use_unlimited_for_target(tx.state(), player, *target)
        })
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, ignore_distance)
                    .is_ok()
        })
        .collect()
}

/// Use a prompted physical or one-card view-as Sha with ordinary distance and use-count rules.
pub(crate) fn play_prompted_sha_source_normal(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets_normal(tx, player, card);
    if targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
    {
        return Err(EffectError::Illegal(
            "invalid ordinary prompted Sha targets".into(),
        ));
    }
    let ignore_distance = prompted_sha_source(tx.state(), player, card)
        .and_then(|(_, view_as)| view_as.map(|(_, rule)| rule.ignore_distance))
        .unwrap_or(false);
    play_prompted_sha_source_impl(tx, player, card, targets, ignore_distance, false, 0)
}

/// Use one revalidated physical or view-as Sha against arbitrary legal targets while ignoring
/// ordinary distance and use count.
pub(crate) fn play_prompted_sha_source_unlimited(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets_unlimited(tx, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// Force one exact physical Sha from the discard pile, with only the supplied targets unable to
/// respond. Used by 破袭; ordinary distance and use count are ignored.
pub(crate) fn play_discard_sha_unlimited(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    unrespondable: &[PlayerId],
) -> Result<(), EffectError> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("discarded Sha missing".into()))?
        .clone();
    let kind = crate::query::effective_card_kind(tx.state(), player, &card_state);
    let legal = prompted_sha_targets_unlimited(tx, player, card);
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive && !player.dying)
        || card_state.zone != discard
        || !kind.is_sha()
        || targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
        || crate::query::card_use_cancelled_for_card(tx.state(), player, card)
    {
        return Err(EffectError::Illegal("invalid 破袭 Sha use".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Sha user hand missing".into()))?;
    crate::effect::move_one_card(tx, card, discard, hand, player, false)?;
    play_as_kind_with_extra_damage_flags(
        tx,
        player,
        card,
        kind,
        targets,
        true,
        false,
        unrespondable,
        true,
        false,
        0,
        &[],
        0,
    )
}

/// Use one exact hand Sha forced by 备诛, ignoring distance and the ordinary use count.
pub(crate) fn play_beizhu_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    if !beizhu_sha_usable(tx, player, target, card) {
        return Err(EffectError::Illegal(
            "Beizhu Sha is no longer usable on the skill owner".into(),
        ));
    }
    play_prompted_sha_source_impl(
        tx,
        player,
        card,
        &[target],
        true,
        true,
        rule_api::damage_card_flag::BEIZHU,
    )
}

pub(crate) fn beizhu_sha_usable(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> bool {
    let Some(hand) = tx.state().zones.find(ZoneKind::Hand, player) else {
        return false;
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return false;
    };
    if card_state.zone != hand
        || card_state.owner != player
        || !crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|state| state.alive && !state.dying)
        || crate::query::card_color_locked_for_card(tx.state(), player, card)
        || crate::query::card_use_cancelled_for_card(tx.state(), player, card)
    {
        return false;
    }
    validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[target]).is_ok()
        && validate_sha_targets_with_options(tx, player, card, &[target], true).is_ok()
}

fn play_prompted_sha_source_with_legal_targets(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    legal: &[PlayerId],
) -> Result<(), EffectError> {
    if targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
    {
        return Err(EffectError::Illegal(
            "invalid prompted Sha target selection".into(),
        ));
    }
    play_prompted_sha_source_impl(tx, player, card, targets, true, true, 0)
}

fn play_prompted_sha_source_impl(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    extra_damage_flags: u8,
) -> Result<(), EffectError> {
    let (kind, view_as) = prompted_sha_source(tx.state(), player, card)
        .ok_or_else(|| EffectError::Illegal("prompted Sha source is no longer legal".into()))?;
    if view_as.is_none() {
        return handle_card_from_hand_with_policies_and_flags(
            tx,
            player,
            card,
            targets,
            ignore_distance,
            ignore_use_count,
            true,
            extra_damage_flags,
        );
    }
    let (skill, rule) = view_as.unwrap();
    let card_state = tx
        .state()
        .cards
        .get(card)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("missing prompted Sha source".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no prompted Sha user hand".into()))?;
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card).is_some();
    let source_was_hand = card_state.zone == hand || borrowed;
    if !source_was_hand {
        crate::effect::move_one_card(tx, card, card_state.zone, hand, player, false)?;
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def)
        .ok_or_else(|| EffectError::Illegal("missing prompted Sha view-as skill".into()))?;
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.use_count = instance.use_count.saturating_add(1);
    }
    let source_suit = crate::query::effective_card_suit(
        tx.state(),
        player,
        tx.state()
            .cards
            .get(card)
            .expect("prompted source preserved"),
    );
    let force_unrespondable =
        rule.red_unrespondable && matches!(source_suit, Suit::Heart | Suit::Diamond);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = (|| {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 120,
                payload: vec![
                    skill.raw(),
                    skill_def.raw(),
                    player.raw(),
                    card.raw(),
                    kind.tag() as u32,
                ],
            }],
        )?;
        play_as_kind_with_extra_damage_flags(
            tx,
            player,
            card,
            kind,
            targets,
            ignore_distance,
            force_unrespondable,
            &[],
            ignore_use_count,
            source_was_hand,
            extra_damage_flags,
            &[],
            0,
        )
    })();
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

/// Cards that can answer 杀 as 闪 (natural 闪 or view_as 闪).
pub(crate) fn response_shan_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::hand_card_use_response_cancelled_for(state, player) {
        return super::skill::source_free_response_tokens(state, player, CardKind::Shan);
    }
    response_tokens(state, player, "shan", |kind| kind == CardKind::Shan)
}

/// Physical 诱敌深入 cards usable before an incoming respondable Sha takes effect.
pub(crate) fn response_youdishenru_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::hand_card_use_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    response_card_tokens(state, player, "youdishenru", |kind| {
        kind == CardKind::TrickYoudishenru
    })
    .into_iter()
    .filter(|token| !crate::query::card_use_cancelled_for_card(state, player, CardId::new(*token)))
    .collect()
}

/// Cards that can answer 南蛮/决斗 as 杀 (natural 杀 or view_as 杀).
pub(crate) fn response_sha_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::sha_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    if crate::query::hand_card_use_response_cancelled_for(state, player) {
        return super::skill::source_free_response_tokens(state, player, CardKind::Sha);
    }
    response_tokens(state, player, "sha", CardKind::is_sha)
}

fn response_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    as_kind: &str,
    natural: impl Fn(CardKind) -> bool,
) -> Vec<u32> {
    let mut out = response_card_tokens(state, player, as_kind, natural);
    let dynamic_kind = match as_kind {
        "sha" => CardKind::Sha,
        "shan" => CardKind::Shan,
        "wuxie" => CardKind::TrickWuxie,
        _ => CardKind::Other,
    };
    if dynamic_kind != CardKind::Other {
        out.extend(super::skill::source_free_response_tokens(
            state,
            player,
            dynamic_kind,
        ));
        out.extend(super::skill::multi_card_response_tokens(
            state,
            player,
            dynamic_kind,
        ));
    }
    out
}

fn response_card_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    as_kind: &str,
    natural: impl Fn(CardKind) -> bool,
) -> Vec<u32> {
    let mut out = Vec::new();
    for cid in state.playable_hand_cards(player) {
        let Some(c) = state.cards.get(cid) else {
            continue;
        };
        let natural_kind = crate::query::effective_card_kind(state, player, c);
        if !crate::query::card_color_locked_for_card(state, player, cid)
            && !crate::query::response_color_locked_for_current_card(state, player, cid)
            && (natural(natural_kind)
                && !crate::query::card_response_cancelled_for_logical_kind(
                    state,
                    player,
                    c,
                    natural_kind,
                )
                || view_as_response_skill(state, player, c, as_kind).is_some())
        {
            out.push(cid.raw());
        }
    }
    if let Some(equip) = state.zones.find(ZoneKind::Equip, player) {
        if let Some(zone) = state.zones.get(equip) {
            for cid in &zone.cards {
                if state.cards.get(*cid).is_some_and(|card| {
                    !crate::query::card_color_locked_for_card(state, player, *cid)
                        && !crate::query::response_color_locked_for_current_card(
                            state, player, *cid,
                        )
                        && view_as_response_skill(state, player, card, as_kind).is_some()
                }) {
                    out.push(cid.raw());
                }
            }
        }
    }
    out
}

pub(crate) fn view_as_response_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    c: &crate::state::Card,
    as_kind: &str,
) -> Option<engine_api::SkillId> {
    let Some(logical_kind) = (match as_kind {
        "sha" => Some(CardKind::Sha),
        "shan" => Some(CardKind::Shan),
        "wuxie" => Some(CardKind::TrickWuxie),
        _ => None,
    }) else {
        return None;
    };
    let Some(p) = state.players.get(player) else {
        return None;
    };
    for sid in &p.skills {
        let Some(inst) = state.skills.get(*sid) else {
            continue;
        };
        if !state.skill_effectively_enabled(inst) {
            continue;
        }
        for v in state.content.view_as_defs_for_skill(inst.def) {
            if v.as_kind != as_kind || !matches!(v.window.as_str(), "any" | "response") {
                continue;
            }
            if crate::query::view_as_source_zone_matches(state, player, v.source_zone.as_deref(), c)
                && crate::query::view_as_filter_matches(state, player, &v.filter, c)
                && !crate::query::card_response_cancelled_for_logical_kind(
                    state,
                    player,
                    c,
                    logical_kind,
                )
            {
                return Some(*sid);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        pack_damage_card_context, sha_queue_target_respondable, unpack_damage_card_context,
    };

    #[test]
    fn damage_card_context_keeps_zero_flag_payloads_backward_compatible() {
        assert_eq!(pack_damage_card_context(2, 0), 2);
        let packed = pack_damage_card_context(1, rule_api::damage_card_flag::JIU_BOOSTED);
        assert_eq!(
            unpack_damage_card_context(packed),
            (1, rule_api::damage_card_flag::JIU_BOOSTED)
        );
    }

    #[test]
    fn multi_sha_queue_keeps_one_response_flag_per_target() {
        let payload = [3, 1, 1, 0, 2, 0, 1, 0, 10, 11, 0, 1];
        assert!(!sha_queue_target_respondable(&payload, 8, 2, 0));
        assert!(sha_queue_target_respondable(&payload, 8, 2, 1));

        let old_payload = [3, 1, 1, 0, 2, 0, 1, 0, 10, 11, 0];
        assert!(!sha_queue_target_respondable(&old_payload, 8, 2, 0));
        assert!(!sha_queue_target_respondable(&old_payload, 8, 2, 1));
    }
}
