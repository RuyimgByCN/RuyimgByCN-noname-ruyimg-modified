//! Command operations for `pack_damage_card_context` and related helpers.

use super::*;

pub(crate) fn pack_damage_card_context(nature: u8, card_flags: u8) -> u32 {
    u32::from(nature) | (u32::from(card_flags) << 8)
}

pub(crate) fn unpack_damage_card_context(value: u32) -> (u8, u8) {
    (value as u8, (value >> 8) as u8)
}

pub(crate) fn sha_target_damage_bonus(flags: u32) -> u32 {
    flags >> SHA_TARGET_DAMAGE_BONUS_SHIFT
}

pub(crate) fn add_sha_target_damage_bonus(flags: &mut u32, amount: u32) {
    let bonus = sha_target_damage_bonus(*flags)
        .saturating_add(amount)
        .min(u32::MAX >> SHA_TARGET_DAMAGE_BONUS_SHIFT);
    *flags = (*flags & SHA_TARGET_LOW_BITS_MASK) | (bonus << SHA_TARGET_DAMAGE_BONUS_SHIFT);
}

pub(crate) fn record_counted_sha_use(tx: &mut Transaction) {
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

pub(crate) fn sha_use_count_allows_targets(
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

pub(crate) fn saved_sha_flags(tx: &Transaction, respondable: bool, counted: bool) -> u32 {
    u32::from(respondable)
        | u32::from(counted && tx.state().content.tracks_sha_use_refund_provenance())
            * SHA_RESUME_COUNTED
}

/// Revalidate and refund the currently saved proactive Sha use at most once.
pub(crate) fn saved_sha_refund_context(
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

pub(crate) fn refund_sha_use_count(tx: &mut Transaction) {
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

pub(crate) fn add_saved_sha_target_with_options(
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

pub(crate) fn sync_saved_sha_resolution_targets(
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
