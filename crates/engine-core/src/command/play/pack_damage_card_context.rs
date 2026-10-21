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

pub(crate) fn validate_saved_independent_trick_extra_target(
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

pub(crate) fn validate_saved_independent_trick_extra_target_with_distance(
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

pub(crate) fn saved_independent_trick_target_context(
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

pub(crate) fn saved_tiesuo_target_context(
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
