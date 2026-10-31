//! 伤害请求的预处理：分配编号、附加牌与禁用标记、推迟伤害尾段到调度队列。

use super::*;

pub(super) fn defer_scheduler_front(tx: &mut Transaction, sched: &mut EffectScheduler) {
    let mut remaining = sched.drain_remaining();
    if remaining.is_empty() {
        return;
    }
    let deferred = &mut tx.state_mut().resolution.deferred_effects;
    remaining.append(&mut std::mem::take(deferred));
    *deferred = remaining;
}

pub(super) fn defer_scheduler_tail(tx: &mut Transaction, sched: &mut EffectScheduler) {
    let mut remaining = sched.drain_remaining();
    if remaining.is_empty() {
        return;
    }
    remaining.append(&mut std::mem::take(
        &mut tx.state_mut().resolution.deferred_tail_effects,
    ));
    tx.state_mut().resolution.deferred_tail_effects = remaining;
}

pub(super) fn prepare_damage_request(
    tx: &Transaction,
    request: &EffectRequest,
) -> Result<Option<EffectRequest>, EffectError> {
    match request {
        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => primitives::prepare_damage(
            tx.state(),
            *source,
            *target,
            *amount,
            *is_lose_hp,
            *nature,
            false,
            engine_api::CardId::NONE,
        ),
        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => {
            let damage_source = tx.state().resolution.sha_damage_source(*source, *card);
            let prepared = primitives::prepare_damage(
                tx.state(),
                damage_source,
                *target,
                *amount,
                false,
                *nature,
                true,
                *card,
            )?;
            Ok(attach_damage_card(
                prepared,
                *card,
                CardKind::Sha.tag(),
                *card_flags,
            ))
        }
        EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            let prepared = primitives::prepare_damage(
                tx.state(),
                *source,
                *target,
                *amount,
                *is_lose_hp,
                *nature,
                false,
                *card,
            )?;
            Ok(attach_damage_card(prepared, *card, *card_kind, *card_flags))
        }
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            let prepared = primitives::prepare_damage(
                tx.state(),
                *source,
                *target,
                *amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                engine_api::CardId::NONE,
            )?;
            Ok(attach_active_damage_ban(prepared, *skill))
        }
        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let prepared = primitives::prepare_damage(
                tx.state(),
                PlayerId::NONE,
                *player,
                *amount,
                true,
                rule_api::damage_nature::NORMAL,
                false,
                engine_api::CardId::NONE,
            )?;
            Ok(attach_lose_hp_then_damage_other(prepared, *target, *skill))
        }
        EffectRequest::CommitDamage { damage_id: 0, .. }
        | EffectRequest::CommitDamageBanActiveIfDying { damage_id: 0, .. }
        | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { .. } => {
            Ok(Some(request.clone()))
        }
        _ => Ok(None),
    }
}

pub(super) fn attach_active_damage_ban(
    prepared: Option<EffectRequest>,
    skill: engine_api::SkillId,
) -> Option<EffectRequest> {
    let EffectRequest::CommitDamage {
        damage_id,
        source,
        target,
        amount,
        base_amount,
        nature,
        ..
    } = prepared?
    else {
        unreachable!();
    };
    Some(EffectRequest::CommitDamageBanActiveIfDying {
        damage_id,
        source,
        target,
        amount,
        base_amount,
        nature,
        skill,
    })
}

pub(super) fn attach_lose_hp_then_damage_other(
    prepared: Option<EffectRequest>,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Option<EffectRequest> {
    let EffectRequest::CommitDamage {
        target: player,
        amount,
        ..
    } = prepared?
    else {
        unreachable!();
    };
    Some(EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
        player,
        target,
        amount,
        skill,
    })
}

pub(super) fn attach_damage_card(
    mut prepared: Option<EffectRequest>,
    card: engine_api::CardId,
    card_kind: u8,
    card_flags: u8,
) -> Option<EffectRequest> {
    if let Some(EffectRequest::CommitDamage {
        card: prepared_card,
        card_kind: prepared_kind,
        card_flags: prepared_flags,
        ..
    }) = &mut prepared
    {
        *prepared_card = card;
        *prepared_kind = card_kind;
        *prepared_flags = card_flags;
    }
    prepared
}

pub(super) fn allocate_damage_id(tx: &mut Transaction) -> u32 {
    let resolution = &mut tx.state_mut().resolution;
    resolution.next_damage_seq = resolution.next_damage_seq.wrapping_add(1).max(1);
    resolution.next_damage_seq
}

pub(super) fn allocate_recovery_id(tx: &mut Transaction) -> u32 {
    let resolution = &mut tx.state_mut().resolution;
    resolution.next_recovery_seq = resolution.next_recovery_seq.wrapping_add(1).max(1);
    resolution.next_recovery_seq
}

pub(super) fn defer_damage_tail(
    tx: &mut Transaction,
    prepared: EffectRequest,
    sched: &mut EffectScheduler,
) {
    // 批 282：damage_before 触发执行期的 bound 增减暂存，在这里随 prepared
    // 入列时统一套用（触发执行先于本函数运行）。
    let pending = std::mem::take(&mut tx.state_mut().resolution.pending_bound_damage_deltas);
    let mut prepared = prepared;
    if !pending.is_empty() {
        let damage_id = tx.state().resolution.current_damage_id;
        let total: i32 = pending
            .iter()
            .filter(|(saved, _)| *saved == damage_id)
            .map(|(_, delta)| *delta)
            .sum();
        if total != 0 {
            if let EffectRequest::CommitDamage {
                amount,
                base_amount,
                ..
            }
            | EffectRequest::CommitDamageBanActiveIfDying {
                amount,
                base_amount,
                ..
            } = &mut prepared
            {
                *amount = if total > 0 {
                    amount.saturating_add(total as u32)
                } else {
                    amount.saturating_sub(total.unsigned_abs())
                };
                if total > 0 {
                    *base_amount = base_amount.saturating_add(total as u32);
                }
            }
        }
    }
    let mut tail = vec![prepared];
    tail.extend(sched.drain_remaining());
    tail.append(&mut std::mem::take(
        &mut tx.state_mut().resolution.deferred_tail_effects,
    ));
    tx.state_mut().resolution.deferred_tail_effects = tail;
}
