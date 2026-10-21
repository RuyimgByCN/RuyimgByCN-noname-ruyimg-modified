//! Effect primitives for `commit_damage` and related helpers.

use super::*;

/// Commit already prepared damage and return raw chained-damage follow-ups.
pub(crate) fn commit_damage(
    tx: &mut Transaction,
    damage_id: u32,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    base_amount: u32,
    is_lose_hp: bool,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    if damage_id != 0 {
        if tx.state().resolution.current_damage_id == damage_id {
            tx.state_mut().resolution.current_damage_id = 0;
        }
        if let Some(index) = tx
            .state()
            .resolution
            .prevented_damage_ids
            .iter()
            .position(|id| *id == damage_id)
        {
            tx.state_mut().resolution.prevented_damage_ids.remove(index);
            return Ok((
                vec![DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                }],
                Vec::new(),
            ));
        }
    }
    if amount == 0 {
        return Ok((
            (!is_lose_hp)
                .then_some(DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                })
                .into_iter()
                .collect(),
            Vec::new(),
        ));
    }
    let p = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut amount = amount;
    let mut events = Vec::new();
    if !is_lose_hp {
        let hujia = tx
            .state()
            .players
            .get(target)
            .map(|p| {
                p.marks
                    .iter()
                    .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                    .map_or(0, |(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        let absorbed = hujia.min(amount);
        if absorbed > 0 {
            events.extend(remove_mark(
                tx,
                target,
                crate::content::MARK_HUJIA,
                absorbed,
            )?);
            amount -= absorbed;
            if amount == 0 {
                events.push(DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                });
                return Ok((events, Vec::new()));
            }
        }
    }
    let (prev_hp, prev_dying) = tx
        .state()
        .players
        .get(target)
        .map(|p| (p.hp, p.dying))
        .unwrap_or((0, false));

    tx.journal(JournalEntry::PlayerHp {
        id: target,
        prev: prev_hp,
    });
    {
        let p = tx
            .state_mut()
            .players
            .get_mut(target)
            .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
        // HP may go negative while dying (noname-style); do not saturating-floor at 0.
        p.hp -= amount as i32;
    }
    if tx.state().content.tracks_turn_hp_change_history() {
        tx.state_mut().turn.record_hp_change(target);
    }
    if tx.state().content.tracks_damage_taken_count() && !is_lose_hp {
        tx.state_mut().turn.record_damage_taken(target);
    }
    if tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        tx.state_mut().turn.mark_phase_hp_changed(target);
    }
    if !is_lose_hp && card.is_some() && CardKind::from_tag(card_kind).is_some_and(CardKind::is_sha)
    {
        let logical_source = if source.is_some() {
            source
        } else {
            tx.state().resolution.logical_sha_source(card)
        };
        if logical_source.is_some() {
            crate::command::play::mark_current_sha_dealt_damage(tx, logical_source, card);
        }
    }
    if !is_lose_hp && card.is_some() && CardKind::from_tag(card_kind) == Some(CardKind::TrickJuedou)
    {
        tx.state_mut().resolution.record_duel_damage(card);
    }
    if !is_lose_hp {
        if card.is_some() {
            if tx.state().content.tracks_turn_haoyi_cards() {
                let materials = tx.state().resolution.card_source_cards_for_card(card);
                tx.state_mut().turn.record_damage_card_materials(materials);
            }
            tx.state_mut()
                .resolution
                .record_card_resolution_damage(card, target);
        }
        if source.is_some() {
            tx.state_mut().damage_history.insert((source, target));
            if tx.state().content.tracks_turn_damage_pair_counts() {
                tx.state_mut().turn.record_damage_pair(source, target);
            }
            let total = tx
                .state_mut()
                .damage_dealt_totals
                .entry(source)
                .or_insert(0);
            *total = total.saturating_add(amount);
            if card_kind == CardKind::Sha.tag() {
                let key = turn_flag::sha_damage_dealt_by_player(source);
                let count = tx.state().turn.flag(key).saturating_add(1);
                tx.state_mut().turn.set_flag(key, count);
            }
        }
        let source_is_current = source.is_some() && source == tx.state().turn.current;
        let damaged_other = source_is_current && source != target;
        let damaged_other_qun =
            source_is_current && source != target && tx.state().player_group(target) == Some("qun");
        tx.journal(JournalEntry::TurnCheckpoint);
        tx.state_mut().turn.mark_player_damaged(target);
        if source.is_some() {
            tx.state_mut().turn.mark_player_dealt_damage(source);
            // ADR 0644 P3：逐源受伤位图（仅当加载内容使用 align_debate_opinions
            // 时记录——保持无关包的 canonical state hash 不变）。
            if tx.state().content.tracks_damaged_by_player_bitset() && source != target {
                if let Some(seat) = tx.state().players.get(target).map(|p| p.seat.raw()) {
                    let key = turn_flag::damaged_by_player(source);
                    let bits = tx.state().turn.flag(key) as u32;
                    tx.state_mut()
                        .turn
                        .set_flag(key, (bits | (1 << seat)) as i32);
                }
            }
        }
        if source_is_current {
            let dealt = tx
                .state()
                .turn
                .flag(turn_flag::DAMAGE_DEALT)
                .saturating_add(i32::try_from(amount).unwrap_or(i32::MAX));
            tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, dealt);
            if damaged_other {
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::DAMAGE_DEALT_OTHER, 1);
            }
            if damaged_other_qun {
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::DAMAGE_DEALT_OTHER_QUN, 1);
            }
        }
    }
    let had_chain = !is_lose_hp
        && matches!(
            nature,
            rule_api::damage_nature::FIRE | rule_api::damage_nature::THUNDER
        )
        && tx
            .state()
            .players
            .get(target)
            .map(|p| {
                p.marks
                    .iter()
                    .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .unwrap_or(false);

    events.push(DomainEvent::DamageDealt {
        source,
        target,
        amount,
        is_lose_hp,
        nature,
    });
    let p = tx.state().players.get(target).unwrap();
    if p.hp <= 0 && !prev_dying && !tx.state().resolution.gzbuqu_nodying {
        tx.journal(JournalEntry::PlayerLife {
            id: target,
            prev_alive: true,
            prev_dying: false,
        });
        tx.state_mut().players.get_mut(target).unwrap().dying = true;
        events.push(DomainEvent::EnteredDying {
            player: target,
            source,
        });
    }

    // 铁索：传导伤害一次，并解除所有横置（属性一并传导）
    let mut followups = Vec::new();
    if !is_lose_hp
        && card_flags & rule_api::damage_card_flag::BEIZHU != 0
        && card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
    {
        followups.push(EffectRequest::DrawCards {
            player: target,
            count: amount,
        });
    }
    if had_chain {
        let chained: Vec<PlayerId> = tx
            .state()
            .players
            .iter_live()
            .filter(|p| {
                p.alive
                    && p.id != target
                    && p.marks
                        .iter()
                        .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .map(|p| p.id)
            .collect();
        let all_chained: Vec<PlayerId> = tx
            .state()
            .players
            .iter_live()
            .filter(|p| {
                p.marks
                    .iter()
                    .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .map(|p| p.id)
            .collect();
        for pid in all_chained {
            let _ = remove_mark(tx, pid, crate::content::MARK_CHAIN, 0);
        }
        for o in chained {
            followups.push(EffectRequest::DealCardDamage {
                source,
                target: o,
                amount: base_amount,
                is_lose_hp: false,
                nature,
                card,
                card_kind,
                card_flags: card_flags | rule_api::damage_card_flag::CHAIN_PROPAGATED,
            });
        }
    }
    Ok((events, followups))
}

/// Commit ordinary source-free damage and ban the originating active skill only
/// when this exact commit creates the target's dying state.
pub(crate) fn commit_damage_ban_active_if_dying(
    tx: &mut Transaction,
    damage_id: u32,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    base_amount: u32,
    nature: u8,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    let instance =
        tx.state().skills.get(skill).ok_or_else(|| {
            EffectError::ContentFault("damage-linked active skill missing".into())
        })?;
    if instance.owner != source {
        return Err(EffectError::ContentFault(
            "damage-linked active skill owner mismatch".into(),
        ));
    }

    let committed = commit_damage(
        tx,
        damage_id,
        source,
        target,
        amount,
        base_amount,
        false,
        nature,
        CardId::NONE,
        0,
        0,
    )?;
    if committed
        .0
        .iter()
        .any(|event| matches!(event, DomainEvent::EnteredDying { player, .. } if *player == target))
    {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .expect("validated skill instance")
            .active_turn_banned = true;
    }
    Ok(committed)
}

/// Commit a skill owner's HP loss, charge one maximum HP only when that exact loss first enters
/// dying, and defer equal ordinary damage to another player until the dying/save window finishes.
pub(crate) fn commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    let instance =
        tx.state().skills.get(skill).ok_or_else(|| {
            EffectError::ContentFault("HP-loss-linked active skill missing".into())
        })?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "HP-loss-linked active skill owner mismatch".into(),
        ));
    }
    if player == target {
        return Err(EffectError::ContentFault(
            "HP-loss-linked damage requires another target".into(),
        ));
    }

    let (mut events, followups) = commit_damage(
        tx,
        0,
        PlayerId::NONE,
        player,
        amount,
        amount,
        true,
        rule_api::damage_nature::NORMAL,
        CardId::NONE,
        0,
        0,
    )?;
    debug_assert!(followups.is_empty(), "HP loss cannot propagate a chain");

    let entered_dying = events.iter().any(
        |event| matches!(event, DomainEvent::EnteredDying { player: dying, .. } if *dying == player),
    );
    if entered_dying {
        events.extend(change_max_hp(tx, player, -1)?);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::DealDamage {
            source: player,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    Ok((events, Vec::new()))
}
