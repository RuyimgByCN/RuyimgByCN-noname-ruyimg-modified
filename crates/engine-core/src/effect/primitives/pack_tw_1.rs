//! Effect primitive implementations — tw 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) fn recover_hp(
    tx: &mut Transaction,
    source: PlayerId,
    player: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !p.alive {
        return Err(EffectError::Illegal("dead".into()));
    }
    let prev = p.hp;
    let max = p.max_hp as i32;
    let was_dying = p.dying;
    tx.journal(JournalEntry::PlayerHp { id: player, prev });
    let new_hp = (prev + amount as i32).min(max);
    if new_hp > 0 && was_dying {
        tx.journal(JournalEntry::PlayerLife {
            id: player,
            prev_alive: true,
            prev_dying: true,
        });
    }
    {
        let p = tx.state_mut().players.get_mut(player).unwrap();
        p.hp = new_hp;
        if new_hp > 0 && was_dying {
            p.dying = false;
        }
    }
    let gained = (new_hp - prev).max(0) as u32;
    if gained == 0 {
        return Ok(vec![]);
    }
    if tx.state().content.tracks_turn_hp_change_history() {
        tx.state_mut().turn.record_hp_change(player);
    }
    if tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        tx.state_mut().turn.mark_phase_hp_changed(player);
    }
    let mut events = vec![DomainEvent::HpRecovered {
        source,
        player,
        amount: gained,
    }];
    if new_hp > 0 && was_dying {
        events.push(DomainEvent::LeftDying { player });
    }
    Ok(events)
}

pub(crate) fn change_max_hp(
    tx: &mut Transaction,
    player: PlayerId,
    delta: i32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (prev_max, prev_hp) = {
        let p = tx
            .state()
            .players
            .get(player)
            .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
        (p.max_hp, p.hp)
    };
    tx.journal(JournalEntry::PlayerMaxHp {
        id: player,
        prev: prev_max,
    });
    tx.journal(JournalEntry::PlayerHp {
        id: player,
        prev: prev_hp,
    });
    let new_max = (prev_max as i32 + delta).max(0) as u32;
    {
        let p = tx.state_mut().players.get_mut(player).unwrap();
        p.max_hp = new_max;
        if p.hp > new_max as i32 {
            p.hp = new_max as i32;
        }
    }
    if prev_hp != tx.state().players.get(player).unwrap().hp {
        if tx.state().content.tracks_turn_hp_change_history() {
            tx.state_mut().turn.record_hp_change(player);
        }
        if tx.state().content.tracks_phase_target_hp_history()
            && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
        {
            tx.state_mut().turn.mark_phase_hp_changed(player);
        }
    }
    let mut events = vec![DomainEvent::MaxHpChanged { player, new_max }];
    if new_max == 0 && tx.state().players.get(player).is_some_and(|p| p.alive) {
        events.extend(kill_player(tx, player, PlayerId::NONE)?);
    }
    Ok(events)
}

pub(crate) fn add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let current = p
        .marks
        .iter()
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    if mark == crate::content::MARK_CHAIN
        && delta > 0
        && current <= 0
        && crate::query::armor_link_gain_cancelled_for(tx.state(), player)
    {
        return Ok(Vec::new());
    }
    tx.journal(JournalEntry::PlayerMarks {
        id: player,
        prev: p.marks.clone(),
    });
    let p = tx.state_mut().players.get_mut(player).unwrap();
    if let Some(slot) = p.marks.iter_mut().find(|(m, _)| *m == mark) {
        slot.1 += delta;
    } else {
        p.marks.push((mark, delta));
        p.marks.sort_by_key(|(m, _)| *m);
    }
    let count = p
        .marks
        .iter()
        .find(|(m, _)| *m == mark)
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let mut events = vec![DomainEvent::MarkChanged {
        player,
        mark_def: mark,
        count,
    }];
    if mark == crate::content::MARK_CHAIN && (current > 0) != (count > 0) {
        events.push(DomainEvent::PlayerLinkedChanged {
            player,
            linked: count > 0,
        });
    }
    Ok(events)
}

pub(crate) fn remove_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let current = p
        .marks
        .iter()
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    if mark == crate::content::MARK_CHAIN
        && current > 0
        && (amount == 0 || i64::from(current) <= i64::from(amount))
        && crate::query::link_loss_cancelled_for(tx.state(), player)
    {
        return Ok(Vec::new());
    }
    tx.journal(JournalEntry::PlayerMarks {
        id: player,
        prev: p.marks.clone(),
    });
    let p = tx.state_mut().players.get_mut(player).unwrap();
    if let Some(i) = p.marks.iter().position(|(m, _)| *m == mark) {
        if amount == 0 {
            p.marks.remove(i);
        } else {
            p.marks[i].1 -= amount as i32;
            if p.marks[i].1 <= 0 {
                p.marks.remove(i);
            }
        }
    }
    let count = p
        .marks
        .iter()
        .find(|(m, _)| *m == mark)
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let mut events = vec![DomainEvent::MarkChanged {
        player,
        mark_def: mark,
        count,
    }];
    if mark == crate::content::MARK_CHAIN && (current > 0) != (count > 0) {
        events.push(DomainEvent::PlayerLinkedChanged {
            player,
            linked: count > 0,
        });
    }
    Ok(events)
}

pub(crate) fn request_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    min_count: u8,
    max_count: u8,
    allow_cancel: bool,
    option_tokens: Vec<u32>,
    kind: ChoiceKind,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice already pending".into()));
    }
    tx.journal(JournalEntry::ResolutionCheckpoint);
    let choice_id = {
        let r = &mut tx.state_mut().resolution;
        r.next_choice_seq = r.next_choice_seq.saturating_add(1).max(1);
        r.next_choice_seq
    };
    let continuation_id = {
        let r = &mut tx.state_mut().resolution;
        r.continuation_id = r.continuation_id.saturating_add(1);
        r.continuation_id
    };
    {
        let r = &mut tx.state_mut().resolution;
        r.pending_choice_id = choice_id;
        r.pending_observer = observer.raw();
        // choice_revision stamped on commit
    }
    let options = option_tokens
        .into_iter()
        .map(|token| ChoiceOption {
            token,
            label_key: String::new(),
        })
        .collect();
    let choice = ChoiceRequest {
        choice_id: ChoiceId::new(choice_id),
        observer,
        options,
        min_count,
        max_count,
        allow_cancel,
        timeout_policy: TimeoutPolicy::None,
        revision: tx.state().revision,
        continuation_id,
        kind,
    };
    tx.set_choice(choice);
    Ok(vec![DomainEvent::ChoiceRequested {
        choice_id,
        observer,
    }])
}

pub(crate) fn request_dying_save(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let order = tx.state().players.alive_by_seat();
    let start = [tx.state().turn.current, source, player]
        .into_iter()
        .find_map(|candidate| order.iter().position(|&player| player == candidate))
        .unwrap_or(0);
    let responders = (0..order.len())
        .map(|offset| order[(start + offset) % order.len()])
        .collect();
    request_dying_save_from_queue(tx, player, source, responders)
}

pub(crate) fn request_dying_save_from_queue(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    mut responders: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(dying) = tx.state().players.get(player) else {
        return Ok(Vec::new());
    };
    if !dying.alive {
        return Ok(Vec::new());
    }
    if dying.hp > 0 || !dying.dying {
        if let Some(dying) = tx.state_mut().players.get_mut(player) {
            dying.dying = false;
        }
        return Ok(Vec::new());
    }

    loop {
        let Some(responder) = responders.first().copied() else {
            return kill_player(tx, player, source);
        };
        responders.remove(0);
        if !tx
            .state()
            .players
            .get(responder)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let has_card = crate::query::dying_save_card_for(tx.state(), responder, player).is_some();
        let mut source_free = crate::command::skill::source_free_response_tokens(
            tx.state(),
            responder,
            CardKind::Tao,
        );
        source_free.extend(crate::command::skill::multi_card_response_tokens(
            tx.state(),
            responder,
            CardKind::Tao,
        ));
        if responder == player {
            source_free.extend(crate::command::skill::source_free_response_tokens(
                tx.state(),
                responder,
                CardKind::Jiu,
            ));
            source_free.extend(crate::command::skill::multi_card_response_tokens(
                tx.state(),
                responder,
                CardKind::Jiu,
            ));
        }
        if !has_card && source_free.is_empty() {
            continue;
        }
        let mut option_tokens = vec![0];
        if has_card {
            option_tokens.push(1);
        }
        option_tokens.extend(source_free);
        let mut payload = vec![
            player.raw(),
            source.raw(),
            responder.raw(),
            responders.len() as u32,
        ];
        payload.extend(responders.into_iter().map(PlayerId::raw));
        return open_choice(
            tx,
            responder,
            1,
            1,
            false,
            option_tokens,
            ChoiceKind::Custom,
            ContinuationKind::DyingSave,
            payload,
        );
    }
}

pub(crate) fn add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    add_skill_with_state(tx, player, skill_def, None)
}

pub(crate) fn add_skill_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let already_owned = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == skill_def)
        });
    if already_owned {
        return Ok(Vec::new());
    }
    let turn_owner = tx.state().turn.current;
    if turn_owner.is_none() {
        return Err(EffectError::Illegal(
            "temporary skill requires an active turn".into(),
        ));
    }
    let events = add_skill_with_state(tx, player, skill_def, None)?;
    let skill = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied()
        .ok_or_else(|| EffectError::EngineFault("temporary skill was not added".into()))?;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .temporary_until_turn_end_of = turn_owner;
    Ok(events)
}

pub(crate) fn add_bound_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    bound_target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(bound_target)
        .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal(
            "bound skill target is not alive".into(),
        ));
    }
    add_skill_with_state(
        tx,
        player,
        skill_def,
        Some(crate::state::SkillInstance::bound_target_state(
            bound_target,
        )),
    )
}

pub(crate) fn replace_skill_with_bound_after_resolution_if_target_alive(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    skill_def: engine_api::SkillDefId,
    bound_target: PlayerId,
    ready_to_replace: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return if ready_to_replace {
            Ok(Vec::new())
        } else {
            Err(EffectError::ContentFault(
                "post-resolution skill replacement source is missing".into(),
            ))
        };
    };
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "post-resolution skill replacement owner mismatch".into(),
        ));
    }
    if tx.state().players.get(bound_target).is_none() {
        return Err(EffectError::ContentFault(
            "post-resolution skill replacement target is missing".into(),
        ));
    }
    if !ready_to_replace {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                player,
                skill,
                skill_def,
                bound_target,
                ready_to_replace: true,
            },
        );
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(bound_target)
        .is_some_and(|target| target.alive)
    {
        return Ok(Vec::new());
    }

    let mut events = add_bound_skill(tx, player, skill_def, bound_target)?;
    events.extend(remove_skill(tx, skill)?);
    Ok(events)
}

pub(crate) fn choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    zone: u8,
    discarded_count: u32,
    ready_to_draw: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_draw {
        let Some(participant) = tx
            .state()
            .players
            .get(player)
            .filter(|participant| participant.alive)
        else {
            return Ok(Vec::new());
        };
        if discarded_count == 0 {
            return Err(EffectError::ContentFault(
                "full-zone discard draw stage requires a positive discarded count".into(),
            ));
        }
        let lost_hp = (i64::from(participant.max_hp) - i64::from(participant.hp))
            .max(0)
            .min(i64::from(u32::MAX)) as u32;
        let draw_count = discarded_count.saturating_add(lost_hp);
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            });
        return draw_cards(tx, player, draw_count);
    }

    let valid = tx.state().players.get(player).is_some_and(|participant| {
        participant.alive
            && tx.state().turn.current == player
            && tx.state().turn.phase == Phase::Prepare
    }) && tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !valid {
        return Err(EffectError::ContentFault(
            "full-zone discard requires its owner's enabled preparation-phase skill".into(),
        ));
    }

    let cards_in_zone = |state: &crate::state::GameState, zone_tag: u8| {
        let kind = match zone_tag {
            1 => ZoneKind::Hand,
            2 => ZoneKind::Equip,
            _ => return Vec::new(),
        };
        let Some(zone_id) = state.zones.find(kind, player) else {
            return Vec::new();
        };
        let discardable: std::collections::BTreeSet<_> =
            crate::query::discardable_hand_equip_cards(state, player)
                .into_iter()
                .collect();
        state
            .zones
            .get(zone_id)
            .into_iter()
            .flat_map(|zone| zone.cards.iter().copied())
            .filter(|card| discardable.contains(card))
            .collect::<Vec<_>>()
    };

    if zone == 0 {
        let hand_available = !cards_in_zone(tx.state(), 1).is_empty();
        let equip_available = !cards_in_zone(tx.state(), 2).is_empty();
        return match (hand_available, equip_available) {
            (false, false) => Ok(Vec::new()),
            (true, false) => choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
                tx, player, skill, 1, 0, false,
            ),
            (false, true) => choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
                tx, player, skill, 2, 0, false,
            ),
            (true, true) => open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp,
                vec![player.raw(), skill.raw()],
            ),
        };
    }

    let cards = cards_in_zone(tx.state(), zone);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "chosen hand or equipment zone has no discardable cards".into(),
        ));
    }
    let discarded_count = cards.len().min(u32::MAX as usize) as u32;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
            player,
            skill,
            zone,
            discarded_count,
            ready_to_draw: true,
        },
    );
    discard_cards(tx, player, cards)
}

pub(crate) fn add_skill_with_state(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    state_blob: Option<Vec<u8>>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    if let Some(existing) = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied()
    {
        let instance = tx.state_mut().skills.get_mut(existing).unwrap();
        if let Some(state_blob) = state_blob {
            instance.state_blob = state_blob;
        }
        instance.temporary_until_turn_end_of = PlayerId::NONE;
        return Ok(Vec::new());
    }
    let (_, mut events) =
        grant_skill_instance(tx, player, skill_def, state_blob.unwrap_or_default())?;
    if skill_def.raw() == crate::command::skill::DDD_FENGZHENG_DEF {
        for target in tx.state().players.alive_by_seat() {
            let exists = tx.state().players.get(target).is_some_and(|participant| {
                participant.skills.iter().any(|owned| {
                    tx.state().skills.get(*owned).is_some_and(|instance| {
                        instance.def.raw() == crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF
                    })
                })
            });
            if !exists {
                events.extend(
                    grant_skill_instance(
                        tx,
                        target,
                        engine_api::SkillDefId::new(
                            crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF,
                        ),
                        Vec::new(),
                    )?
                    .1,
                );
            }
        }
    }
    Ok(events)
}

pub(crate) fn add_luanqun_helper(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    behavior: u8,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let existing = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied();
    let (skill, events) = match existing {
        Some(skill) => (skill, Vec::new()),
        None => {
            let (skill, events) = grant_skill_instance(tx, player, skill_def, Vec::new())?;
            (skill, events)
        }
    };
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Luanqun helper disappeared".into()))?;
    if !instance.add_luanqun_source(behavior, source) {
        return Err(EffectError::Illegal("invalid Luanqun helper state".into()));
    }
    instance.temporary_until_turn_end_of = player;
    Ok(events)
}

pub(crate) fn add_mbjinzu_helper(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let existing = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied();
    let (skill, events) = match existing {
        Some(skill) => (skill, Vec::new()),
        None => {
            let (skill, events) = grant_skill_instance(tx, player, skill_def, Vec::new())?;
            (skill, events)
        }
    };
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("mbjinzu helper disappeared".into()))?;
    if !instance.add_mbjinzu_target(target) {
        return Err(EffectError::Illegal("invalid mbjinzu helper state".into()));
    }
    instance.temporary_until_turn_end_of = player;
    Ok(events)
}

pub(crate) fn grant_skill_instance(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    state_blob: Vec<u8>,
) -> Result<(engine_api::SkillId, Vec<DomainEvent>), EffectError> {
    use crate::state::SkillInstance;
    use engine_api::SkillId;

    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let sid = tx.state_mut().skills.alloc(SkillInstance {
        id: SkillId::NONE,
        def: skill_def,
        owner: player,
        enabled: true,
        use_count: 0,
        turn_fire_count: 0,
        content_flags: 0,
        play_phase_fire_count: 0,
        play_phase_use_count: 0,
        play_phase_category_mask: 0,
        active_turn_banned: false,
        turn_disabled: false,
        disabled_by_skills: Vec::new(),
        temporary_until_turn_end_of: PlayerId::NONE,
        mark: 0,
        round_fire_round: 0,
        lifetime_fire_count: 0,
        view_as_branch: 0,
        view_as_use_round: 0,
        view_as_used_branches: 0,
        state_blob,
        source_free_physical_card: CardId::NONE,
        general_slot: crate::state::GENERAL_SLOT_NONE,
        content_storage: std::collections::BTreeMap::new(),
    });
    tx.state_mut()
        .players
        .get_mut(player)
        .unwrap()
        .skills
        .push(sid);
    // Keep skills list stable-sorted by id
    tx.state_mut()
        .players
        .get_mut(player)
        .unwrap()
        .skills
        .sort_by_key(|s| s.raw());
    let content = tx.state().content.clone();
    tx.state_mut()
        .triggers
        .register_skill(sid, skill_def, player, &content);
    tx.state_mut()
        .modifiers
        .register_skill(sid, skill_def, player, &content);
    let enabled = tx
        .state()
        .skills
        .get(sid)
        .is_some_and(|skill| tx.state().skill_effectively_enabled(skill));
    tx.state_mut().triggers.set_skill_enabled(sid, enabled);
    tx.state_mut().modifiers.set_skill_enabled(sid, enabled);
    Ok((sid, vec![DomainEvent::SkillAdded { player, skill: sid }]))
}

pub(crate) fn remove_skill(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(inst) = tx.state().skills.get(skill).cloned() else {
        return Err(EffectError::Illegal("bad skill".into()));
    };
    let player = inst.owner;
    let mut events = Vec::new();
    if inst.def.raw() == 42_478_118 {
        for derived in inst.ddd_shixing_grants() {
            if tx.state().skills.get(derived).is_some() {
                events.extend(remove_skill(tx, derived)?);
            }
        }
    }
    if inst.def.raw() == crate::state::POTNIZUN_DEF {
        if let Some(pile) = tx
            .state()
            .skill_piles
            .iter()
            .find(|pile| pile.owner == player && pile.skill_def == inst.def.raw())
            .copied()
        {
            let cards = tx
                .state()
                .zones
                .get(pile.zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if !cards.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                events.extend(move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(pile.zone),
                    CardMoveReason::Discard,
                )?);
            }
            tx.state_mut()
                .skill_piles
                .retain(|reference| reference.zone != pile.zone);
        }
    }
    tx.state_mut().skills.remove(skill);
    if let Some(p) = tx.state_mut().players.get_mut(player) {
        p.skills.retain(|s| *s != skill);
    }
    tx.state_mut().triggers.unregister_skill(skill);
    tx.state_mut().modifiers.unregister_skill(skill);
    let cleared_suppression = tx.state().turn.non_locked_skill_suppression == skill;
    if cleared_suppression {
        tx.state_mut().turn.non_locked_skill_suppression = engine_api::SkillId::NONE;
    }
    let reenabled = tx.state_mut().skills.remove_disable_reason(skill);
    if reenabled || cleared_suppression {
        tx.state_mut().rebuild_derived_indexes();
    }
    events.push(DomainEvent::SkillRemoved { player, skill });
    Ok(events)
}

pub(crate) fn remove_all_skills(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(owner) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("bad player".into()));
    };
    let mut skills = owner.skills.clone();
    skills.sort_by_key(|skill| skill.raw());
    let mut events = Vec::with_capacity(skills.len());
    for skill in skills {
        events.extend(remove_skill(tx, skill)?);
    }
    Ok(events)
}

pub(crate) fn remove_skill_by_def(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(owner) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("bad player".into()));
    };
    let skills: Vec<_> = owner
        .skills
        .iter()
        .copied()
        .filter(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def == skill_def)
        })
        .collect();
    let mut events = Vec::new();
    for skill in skills {
        events.extend(remove_skill(tx, skill)?);
    }
    Ok(events)
}

pub(crate) fn disable_skill_by_def_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(owner) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("bad player".into()));
    };
    let skills: Vec<_> = owner
        .skills
        .iter()
        .copied()
        .filter(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def == skill_def && !skill.turn_disabled)
        })
        .collect();
    if skills.is_empty() {
        return Ok(vec![]);
    }
    for skill in skills {
        tx.state_mut().skills.get_mut(skill).unwrap().turn_disabled = true;
    }
    tx.state_mut().rebuild_derived_indexes();
    Ok(vec![])
}

pub(crate) fn enter_dying(tx: &mut Transaction, player: PlayerId) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if p.dying {
        return Ok(vec![]);
    }

    // ADR 0598 phase C: in guozhan, a 珠联璧合 (perfect pair) boon mark may be spent on
    // entering dying to recover 1 HP and cancel the dying state. Identity mode never
    // carries this mark, so identity behaviour is unchanged.
    let is_guozhan = tx.state().ruleset.name.starts_with("guozhan");
    if is_guozhan {
        let has_zhulian = tx
            .state()
            .players
            .get(player)
            .is_some_and(|pp| {
                pp.marks
                    .iter()
                    .any(|(m, c)| *m == crate::mode::ZHULIAN_BIHE_MARK && *c > 0)
            });
        if has_zhulian {
            // Spend one stack of the mark.
            if let Some(pp) = tx.state_mut().players.get_mut(player) {
                for (m, c) in &mut pp.marks {
                    if *m == crate::mode::ZHULIAN_BIHE_MARK && *c > 0 {
                        *c -= 1;
                        break;
                    }
                }
                pp.marks.retain(|(_, c)| *c > 0);
            }
            // Recover 1 HP instead of entering dying.
            return run_effects(
                tx,
                vec![EffectRequest::RecoverHp {
                    source: PlayerId::NONE,
                    player,
                    amount: 1,
                }],
            );
        }
    }

    tx.journal(JournalEntry::PlayerLife {
        id: player,
        prev_alive: p.alive,
        prev_dying: p.dying,
    });
    tx.state_mut().players.get_mut(player).unwrap().dying = true;
    Ok(vec![DomainEvent::EnteredDying {
        player,
        source: PlayerId::NONE,
    }])
}

pub(crate) fn kill_player(
    tx: &mut Transaction,
    player: PlayerId,
    killer: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (prev_alive, prev_dying, prev_hp) = {
        let p = tx
            .state()
            .players
            .get(player)
            .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
        (p.alive, p.dying, p.hp)
    };
    if prev_alive && killer.is_some() {
        let key = turn_flag::kill_count_by_player(killer);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
    }
    tx.journal(JournalEntry::PlayerLife {
        id: player,
        prev_alive,
        prev_dying,
    });
    tx.journal(JournalEntry::PlayerHp {
        id: player,
        prev: prev_hp,
    });
    {
        let p = tx.state_mut().players.get_mut(player).unwrap();
        p.alive = false;
        p.dying = false;
        p.hp = 0;
    }
    if prev_hp != 0
        && tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        tx.state_mut().turn.mark_phase_hp_changed(player);
    }
    // Capture hand + equip for death-trigger semantics before moving them to discard.
    let mut hand_equip_cards = tx.state().hand_cards(player);
    if let Some(eq) = tx.state().zones.find(ZoneKind::Equip, player) {
        if let Some(z) = tx.state().zones.get(eq) {
            hand_equip_cards.extend(z.cards.iter().copied());
        }
    }
    let mut to_discard = hand_equip_cards.clone();
    if let Some(jz) = tx.state().zones.find(ZoneKind::Judge, player) {
        if let Some(z) = tx.state().zones.get(jz) {
            to_discard.extend(z.cards.iter().copied());
        }
    }
    if !to_discard.is_empty() {
        if let Some(discard) = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE) {
            let _ = move_cards(tx, to_discard, discard, None);
        }
    }
    let mut events = vec![DomainEvent::PlayerKilled {
        player,
        killer,
        hand_equip_cards,
    }];
    // ADR 0598 §D5: guozhan matches check faction victory; identity checks role victory;
    // M4 single (单挑) checks last-living-player victory.
    let victory_ev = if tx.state().ruleset.name.starts_with("guozhan") {
        crate::mode::apply_guozhan_victory_if_any(tx.state_mut())
    } else if tx.state().ruleset.name.starts_with("single") {
        crate::mode::apply_single_victory_if_any(tx.state_mut())
    } else {
        crate::mode::apply_victory_if_any(tx.state_mut())
    };
    if let Some(ev) = victory_ev {
        events.push(ev);
    }
    if prev_alive && !tx.state().outcome.is_finished() && tx.state().turn.current == player {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::AdvanceTurn);
    }
    Ok(events)
}

pub(crate) fn set_turn(
    tx: &mut Transaction,
    current: PlayerId,
    phase: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if current.is_some() && tx.state().players.get(current).is_none() {
        return Err(EffectError::Illegal("bad current".into()));
    }
    let previous_current = tx.state().turn.current;
    let previous_phase = tx.state().turn.phase;
    let previous_damage_sources = tx.state().turn.flag(turn_flag::DAMAGE_SOURCES) as u32;
    let previous_damaged_players = tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS) as u32;
    let previous_card_targeted_other = tx.state().turn.flag(turn_flag::CARD_TARGETED_OTHER) != 0;
    let previous_current_dealt_damage_other =
        tx.state().turn.flag(turn_flag::DAMAGE_DEALT_OTHER) != 0;
    let previous_phase_targeted_players =
        tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) as u32;
    let previous_phase_hp_changed_players =
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) as u32;
    let turn_end_already_dispatched =
        previous_phase == Phase::End && tx.state().turn.flag(turn_flag::TURN_END_DISPATCHED) != 0;
    let next_phase = Phase::from_u8(phase);
    tx.journal(JournalEntry::TurnCheckpoint);
    if next_phase == Phase::Draw && (current != previous_current || previous_phase != Phase::Draw) {
        tx.state_mut().turn.clear_draw_phase_fixed_count();
    }
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = next_phase;
    if current != previous_current || next_phase != previous_phase {
        tx.state_mut().turn.clear_phase_histories();
    }
    if next_phase == Phase::Discard && previous_phase != Phase::Discard {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::DISCARD_PHASE_DISCARDED, 0);
        if tx.state().content.tracks_discard_phase_lost_cards() {
            // One-based slice base: zero means "no discard phase entered this turn".
            let lost_base = (tx.state().turn.lost_cards.len() as i32).saturating_add(1);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::DISCARD_PHASE_LOST_BASE, lost_base);
        }
    }
    if current != previous_current || next_phase != previous_phase {
        tx.state_mut().skills.reset_play_phase_fire_counts();
        let skills: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| skill.id)
            .collect();
        for skill in skills {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_bound_category_target();
            }
        }
    }
    if next_phase == Phase::Play && (current != previous_current || previous_phase != Phase::Play) {
        tx.state_mut().skills.reset_play_phase_use_counts();
        let skills: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| skill.id)
            .collect();
        for skill in skills {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_remembered_category();
            }
        }
        tx.state_mut().turn.clear_play_card_category_history();
        tx.state_mut().turn.clear_play_phase_card_use_counts();
        if tx.state().content.tracks_opening_extra_play_lifecycle() {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
        }
    }
    let starts_new_turn = current != previous_current
        || (previous_phase == Phase::End && next_phase == Phase::Prepare);
    if starts_new_turn {
        tx.state_mut().turn.clear_per_player_card_counts();
        tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::DAMAGE_DEALT_OTHER, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::DAMAGE_DEALT_OTHER_QUN, 0);
        tx.state_mut().turn.set_flag(turn_flag::CARD_USED, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::USE_RESPONSE_DISCARDED, 0);
        tx.state_mut().turn.set_flag(turn_flag::DAMAGED_PLAYERS, 0);
        tx.state_mut().turn.set_flag(turn_flag::DAMAGE_SOURCES, 0);
        tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_META, 0);
        tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_KIND, 0);
        tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_DEF, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_PLAY_CARD_META, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::OPENING_EXTRA_PLAY_PENDING, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE, 0);
        tx.state_mut().turn.opening_extra_play_queue.clear();
        tx.state_mut().turn.opening_extra_play_active_skill = engine_api::SkillId::NONE;
        tx.state_mut().turn.opening_extra_play_damage_baseline = 0;
        tx.state_mut()
            .turn
            .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT, 0);
        tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::PRE_NORMAL_EXTRA_DRAW, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::DISCARD_PILE_SUIT_MASK, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::SKIPPED_PHASE_COUNT, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CARD_TARGETED_OTHER, 0);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 0);
    } else if next_phase == Phase::Play && previous_phase != Phase::Play {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_PLAY_CARD_META, 0);
    }
    let mut events = Vec::with_capacity(2);
    if previous_phase != next_phase && previous_phase != Phase::Idle && !turn_end_already_dispatched
    {
        events.push(DomainEvent::PhaseEnded {
            current: previous_current,
            phase: previous_phase.to_tag(),
            damage_sources: previous_damage_sources,
            damaged_players: previous_damaged_players,
            card_targeted_other: previous_card_targeted_other,
            current_dealt_damage_other: previous_current_dealt_damage_other,
            phase_targeted_players: previous_phase_targeted_players,
            phase_hp_changed_players: previous_phase_hp_changed_players,
        });
    }
    if turn_end_already_dispatched {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::TURN_END_DISPATCHED, 0);
    }
    events.push(DomainEvent::TurnStateChanged {
        current,
        phase: next_phase.to_tag(),
    });
    Ok(events)
}

pub(crate) fn queue_opening_extra_play_phase(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().content.tracks_opening_extra_play_lifecycle()
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Idle
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::ContentFault(
            "opening extra Play phase must be queued by the current player's enabled turn-begin skill"
                .into(),
        ));
    }
    let pending = tx
        .state()
        .turn
        .flag(turn_flag::OPENING_EXTRA_PLAY_PENDING)
        .max(0);
    if pending >= 8 {
        return Err(EffectError::ContentFault(
            "opening extra Play phase queue exceeds the eight-segment bound".into(),
        ));
    }
    tx.state_mut().turn.set_flag(
        turn_flag::OPENING_EXTRA_PLAY_PENDING,
        pending.saturating_add(1),
    );
    tx.state_mut().turn.opening_extra_play_queue.push(skill);
    Ok(Vec::new())
}

pub(crate) fn choose_end_extra_draw_or_play_by_used_suits(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::ContentFault(
            "extra Draw/Play choice must resolve for the current player's completed End phase"
                .into(),
        ));
    }
    let hp = tx
        .state()
        .players
        .get(player)
        .filter(|participant| participant.alive)
        .map(|participant| participant.hp.max(0) as u32)
        .ok_or_else(|| EffectError::ContentFault("extra-phase player is not alive".into()))?;
    if tx.state().turn.distinct_used_card_suits(player) >= hp {
        queue_end_extra_phases(tx, 3);
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseEndExtraDrawOrPlay,
        vec![player.raw(), skill.raw()],
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn lose_hp_double_current_damage_then_replace(
    tx: &mut Transaction,
    player: PlayerId,
    damage_id: u32,
    _amount: u32,
    skill: engine_api::SkillId,
    replacement_skill_def: engine_api::SkillDefId,
    ready_to_replace: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if replacement_skill_def.is_none()
        || tx
            .state()
            .content
            .skill_metadata(replacement_skill_def)
            .is_none()
    {
        return Err(EffectError::ContentFault(
            "damage rewrite replacement skill is unavailable".into(),
        ));
    }
    if ready_to_replace {
        let mut events = add_skill(tx, player, replacement_skill_def)?;
        if tx.state().skills.get(skill).is_some() {
            events.extend(remove_skill(tx, skill)?);
        }
        return Ok(events);
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !bound_damage_source_matches(tx, damage_id, player)
    {
        return Err(EffectError::ContentFault(
            "damage rewrite requires its originating direct damage".into(),
        ));
    }
    let amount = tx
        .state()
        .resolution
        .deferred_tail_effects
        .iter()
        .find_map(|request| match request {
            EffectRequest::CommitDamage {
                damage_id: saved,
                amount,
                ..
            }
            | EffectRequest::CommitDamageBanActiveIfDying {
                damage_id: saved,
                amount,
                ..
            } if *saved == damage_id => Some(*amount),
            _ => None,
        })
        .filter(|amount| *amount > 0)
        .ok_or_else(|| EffectError::EngineFault("bound damage amount is missing".into()))?;
    increase_bound_damage(tx, damage_id, amount)?;
    tx.state_mut().resolution.deferred_tail_effects.insert(
        0,
        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace: true,
        },
    );
    super::run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| Vec::new())
}

pub(crate) fn queue_end_extra_phases(tx: &mut Transaction, phases: u8) {
    let queued = tx.state().turn.flag(turn_flag::END_EXTRA_PHASES).max(0) as u8;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::END_EXTRA_PHASES, i32::from(queued | phases));
}

pub(crate) fn start_end_extra_phases(
    tx: &mut Transaction,
    player: PlayerId,
    phases: u8,
) -> Result<(), EffectError> {
    if !(1..=3).contains(&phases)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
    {
        return Err(EffectError::EngineFault(
            "end extra phases no longer match the completed End phase".into(),
        ));
    }
    if phases & 1 != 0 {
        crate::command::turn::enter_draw_phase(tx, player)
    } else {
        run_effects(
            tx,
            vec![EffectRequest::SetTurnState {
                current: player,
                phase: Phase::Play.to_u8(),
            }],
        )
        .map(|_| ())
    }
}

pub(crate) fn begin_turn(tx: &mut Transaction, player: PlayerId) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::EngineFault(
            "cannot begin a turn for a missing or dead player".into(),
        ));
    }
    let previous_current = tx.state().turn.current;
    let previous_phase = tx.state().turn.phase;
    let damage_sources = tx.state().turn.flag(turn_flag::DAMAGE_SOURCES) as u32;
    let damaged_players = tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS) as u32;
    let card_targeted_other = tx.state().turn.flag(turn_flag::CARD_TARGETED_OTHER) != 0;
    let current_dealt_damage_other = tx.state().turn.flag(turn_flag::DAMAGE_DEALT_OTHER) != 0;
    let phase_targeted_players = tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) as u32;
    let phase_hp_changed_players = tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) as u32;
    tx.journal(JournalEntry::TurnCheckpoint);
    tx.state_mut().turn.current = player;
    tx.state_mut().turn.phase = Phase::Idle;
    tx.state_mut().turn.clear_phase_histories();
    tx.state_mut()
        .turn
        .set_flag(turn_flag::OPENING_EXTRA_PLAY_PENDING, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE, 0);
    tx.state_mut().turn.opening_extra_play_queue.clear();
    tx.state_mut().turn.opening_extra_play_active_skill = engine_api::SkillId::NONE;
    tx.state_mut().turn.opening_extra_play_damage_baseline = 0;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT, 1);
    let mut events = Vec::with_capacity(2);
    if previous_phase != Phase::Idle {
        events.push(DomainEvent::PhaseEnded {
            current: previous_current,
            phase: previous_phase.to_tag(),
            damage_sources,
            damaged_players,
            card_targeted_other,
            current_dealt_damage_other,
            phase_targeted_players,
            phase_hp_changed_players,
        });
    }
    events.push(DomainEvent::TurnBegan { player });
    Ok(events)
}

pub(crate) fn finish_turn_begin(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player || tx.state().turn.phase != Phase::Idle {
        return Err(EffectError::EngineFault(
            "finish-turn-begin continuation no longer matches the current turn".into(),
        ));
    }
    let pending = tx
        .state()
        .turn
        .flag(turn_flag::OPENING_EXTRA_PLAY_PENDING)
        .max(0);
    if pending == 0 {
        return set_turn(tx, player, Phase::Prepare.to_u8());
    }
    let skill = if tx.state().turn.opening_extra_play_queue.is_empty() {
        engine_api::SkillId::NONE
    } else {
        tx.state_mut().turn.opening_extra_play_queue.remove(0)
    };
    tx.state_mut().turn.set_flag(
        turn_flag::OPENING_EXTRA_PLAY_PENDING,
        pending.saturating_sub(1),
    );
    tx.state_mut()
        .turn
        .set_flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE, 1);
    let damage_baseline = tx
        .state()
        .damage_dealt_totals
        .get(&player)
        .copied()
        .unwrap_or(0);
    tx.state_mut().turn.opening_extra_play_active_skill = skill;
    tx.state_mut().turn.opening_extra_play_damage_baseline = damage_baseline;
    set_turn(tx, player, Phase::Play.to_u8())
}

pub(crate) fn advance_opening_extra_play(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx.state().turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE) == 0
    {
        return Err(EffectError::EngineFault(
            "opening extra Play continuation no longer matches the current phase".into(),
        ));
    }
    let pending = tx
        .state()
        .turn
        .flag(turn_flag::OPENING_EXTRA_PLAY_PENDING)
        .max(0);
    if pending == 0 {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE, 0);
        tx.state_mut().turn.opening_extra_play_active_skill = engine_api::SkillId::NONE;
        tx.state_mut().turn.opening_extra_play_damage_baseline = 0;
        return set_turn(tx, player, Phase::Prepare.to_u8());
    }
    let skill = if tx.state().turn.opening_extra_play_queue.is_empty() {
        engine_api::SkillId::NONE
    } else {
        tx.state_mut().turn.opening_extra_play_queue.remove(0)
    };
    tx.state_mut().turn.set_flag(
        turn_flag::OPENING_EXTRA_PLAY_PENDING,
        pending.saturating_sub(1),
    );
    let damage_baseline = tx
        .state()
        .damage_dealt_totals
        .get(&player)
        .copied()
        .unwrap_or(0);
    tx.state_mut().turn.opening_extra_play_active_skill = skill;
    tx.state_mut().turn.opening_extra_play_damage_baseline = damage_baseline;
    restart_play_phase(tx, player)
}

pub(crate) fn restart_play_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let damage_sources = tx.state().turn.flag(turn_flag::DAMAGE_SOURCES) as u32;
    let damaged_players = tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS) as u32;
    let card_targeted_other = tx.state().turn.flag(turn_flag::CARD_TARGETED_OTHER) != 0;
    let current_dealt_damage_other = tx.state().turn.flag(turn_flag::DAMAGE_DEALT_OTHER) != 0;
    let phase_targeted_players = tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) as u32;
    let phase_hp_changed_players = tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) as u32;
    tx.journal(JournalEntry::TurnCheckpoint);
    reset_restarted_play_phase(tx);
    Ok(vec![
        DomainEvent::PhaseEnded {
            current: player,
            phase: Phase::Play.to_tag(),
            damage_sources,
            damaged_players,
            card_targeted_other,
            current_dealt_damage_other,
            phase_targeted_players,
            phase_hp_changed_players,
        },
        DomainEvent::TurnStateChanged {
            current: player,
            phase: Phase::Play.to_tag(),
        },
    ])
}

pub(crate) fn reset_restarted_play_phase(tx: &mut Transaction) {
    tx.state_mut().skills.reset_play_phase_fire_counts();
    tx.state_mut().skills.reset_play_phase_use_counts();
    tx.state_mut().turn.clear_play_card_category_history();
    tx.state_mut().turn.clear_play_phase_card_use_counts();
    tx.state_mut().turn.clear_phase_histories();
    tx.state_mut()
        .turn
        .set_flag(turn_flag::LAST_PLAY_CARD_META, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
}

pub(crate) fn insert_extra_play_after_completed_play(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Discard
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    tx.journal(JournalEntry::TurnCheckpoint);
    tx.state_mut().turn.phase = Phase::Play;
    reset_restarted_play_phase(tx);
    Ok(vec![DomainEvent::TurnStateChanged {
        current: player,
        phase: Phase::Play.to_tag(),
    }])
}

/// Bootstrap identity match used by StartGame (Phase 5).
///
/// Setup: identities, characters + skills, standard deck, initial 4 cards,
/// lord starts at Draw and draws 2.
pub fn bootstrap_match(
    tx: &mut Transaction,
    player_count: u8,
    rng_seed: u64,
    ruleset_name: &str,
) -> Result<(), EffectError> {
    use crate::mode::distribute_identities;
    use crate::state::{RulesetRef, SkillInstance};
    use engine_api::{RulesetId, SkillDefId, SkillId};

    if !(2..=8).contains(&player_count) {
        return Err(EffectError::Illegal("player_count must be 2..=8".into()));
    }
    // M4: 单挑（single）为严格 1v1；其余人数拒绝。
    if ruleset_name.starts_with("single") && player_count != 2 {
        return Err(EffectError::Illegal(
            "single mode requires exactly 2 players".into(),
        ));
    }

    // ADR 0598 §D5: guozhan ruleset dispatches to the guozhan bootstrap path; identity
    // rulesets keep the existing path below, so identity behaviour is unchanged.
    if ruleset_name.starts_with("guozhan") {
        return bootstrap_guozhan(tx, player_count, rng_seed, ruleset_name);
    }

    // M1: auto-load official standard skill pack when not already present.
    crate::content::ensure_standard_package(std::sync::Arc::make_mut(&mut tx.state_mut().content));

    let st = tx.state_mut();
    // Preserve loaded content lock hash (packages loaded before StartGame).
    let content_hash = st.content.combined_logic_hash;
    st.ruleset = RulesetRef {
        id: RulesetId::new(1),
        name: ruleset_name.to_string(),
        logic_hash: content_hash,
    };
    st.rng = RuleRng::from_seed(rng_seed).into_state();

    let draw = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::DrawPile,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _discard = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::DiscardPile,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _processing = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Processing,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _removed = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Removed,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });

    // Identities: seat 0 = Lord; shuffle the rest deterministically.
    // 单挑（single）：无主公，双方中立身份，避免主公技误触发。
    let mut identities = distribute_identities(player_count);
    if ruleset_name.starts_with("single") {
        identities = vec![Identity::Loyalist; player_count as usize];
    }
    if identities.len() > 1 {
        let mut rest = identities[1..].to_vec();
        let mut rng = RuleRng::from_state(st.rng);
        rng.shuffle(&mut rest);
        st.rng = rng.into_state();
        identities[1..].copy_from_slice(&rest);
    }

    // Characters: shuffle roster and assign one each
    let mut char_ids: Vec<u32> = st.content.characters().map(|c| c.id).collect();
    if char_ids.is_empty() {
        return Err(EffectError::Illegal(
            "loaded content has no character definitions".into(),
        ));
    }
    {
        let mut rng = RuleRng::from_state(st.rng);
        rng.shuffle(&mut char_ids);
        st.rng = rng.into_state();
    }

    let mut player_ids = Vec::new();
    for seat in 0..player_count {
        let id_role = identities[seat as usize];
        let char_id = char_ids[seat as usize % char_ids.len()];
        let ch = st
            .content
            .character(char_id)
            .cloned()
            .ok_or_else(|| EffectError::Illegal(format!("unknown character {char_id}")))?;
        let mut max_hp = ch.max_hp;
        let mut hp = if ch.initial_hp == 0 {
            ch.max_hp
        } else {
            ch.initial_hp
        };
        if matches!(id_role, Identity::Lord) {
            max_hp = max_hp.saturating_add(1); // 主公 +1 体力上限
            hp = hp.saturating_add(1);
        }
        let pid = st.players.alloc(Player {
            id: PlayerId::NONE,
            seat: engine_api::SeatIndex::new(seat),
            hp: hp as i32,
            max_hp,
            alive: true,
            dying: false,
            turned_over: false,
            identity: id_role,
            character_def: char_id,
            character_def2: 0,
            skills: Vec::new(),
            marks: Vec::new(),
            disabled_equip_slots: [0; 5],
            unseen: [false, false],
        });
        player_ids.push(pid);
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Hand,
            owner: pid,
            cards: Vec::new(),
        });
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Equip,
            owner: pid,
            cards: Vec::new(),
        });
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Judge,
            owner: pid,
            cards: Vec::new(),
        });

        // Grant character skills
        for &sdef in &ch.skills {
            let sid = st.skills.alloc(SkillInstance {
                id: SkillId::NONE,
                def: SkillDefId::new(sdef),
                owner: pid,
                enabled: true,
                use_count: 0,
                turn_fire_count: 0,
                content_flags: 0,
                play_phase_fire_count: 0,
                play_phase_use_count: 0,
                play_phase_category_mask: 0,
                active_turn_banned: false,
                turn_disabled: false,
                disabled_by_skills: Vec::new(),
                temporary_until_turn_end_of: PlayerId::NONE,
                mark: 0,
                round_fire_round: 0,
                lifetime_fire_count: 0,
                view_as_branch: 0,
                view_as_use_round: 0,
                view_as_used_branches: 0,
                state_blob: Vec::new(),
                source_free_physical_card: CardId::NONE,
                general_slot: crate::state::GENERAL_SLOT_NONE,
                content_storage: std::collections::BTreeMap::new(),
            });
            if let Some(p) = st.players.get_mut(pid) {
                p.skills.push(sid);
                p.skills.sort_by_key(|s| s.raw());
            }
            let content = st.content.clone();
            st.triggers
                .register_skill(sid, SkillDefId::new(sdef), pid, &content);
            st.modifiers
                .register_skill(sid, SkillDefId::new(sdef), pid, &content);
            let enabled = st
                .skills
                .get(sid)
                .is_some_and(|skill| st.skill_effectively_enabled(skill));
            st.triggers.set_skill_enabled(sid, enabled);
            st.modifiers.set_skill_enabled(sid, enabled);
        }
    }

    // Standard deck — config-driven (§48), falls back to hardcoded if config missing
    let specs = crate::content::build_deck_specs();
    let mut deck_cards = Vec::new();
    for spec in specs {
        let cid = st.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(spec.def),
            kind: spec.kind,
            suit: spec.suit,
            rank: spec.rank,
            weapon_range: spec.weapon_range,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: false,
        });
        deck_cards.push(cid);
    }
    {
        let mut rng = RuleRng::from_state(st.rng);
        rng.shuffle(&mut deck_cards);
        st.rng = rng.into_state();
    }
    if let Some(z) = st.zones.get_mut(draw) {
        z.cards = deck_cards;
    }

    // Initial 4 cards each (classic)
    let pids = player_ids.clone();
    let first = pids.first().copied().unwrap_or(PlayerId::NONE);
    st.turn.round = 1;
    st.turn.current = first;
    st.turn.phase = Phase::Draw;

    let game_started = DomainEvent::GameStarted { player_count };
    let mut setup_events = vec![game_started.clone()];

    for pid in &pids {
        let count = crate::query::initial_hand_count_for(tx.state(), *pid, 4);
        let drawn = draw_cards(tx, *pid, count)?;
        setup_events.extend(drawn);
    }
    // 开局发牌不计入“本回合获得”（旧 JS 的 gain 历史自 gameStart 后开始）。
    tx.state_mut().turn.gained_hand_cards.clear();
    for pid in &pids {
        let tags_initial_hand = tx
            .state()
            .players
            .get(*pid)
            .into_iter()
            .flat_map(|player| player.skills.iter())
            .filter_map(|skill| tx.state().skills.get(*skill))
            .any(|skill| {
                tx.state()
                    .content
                    .view_as_defs_for_skill(skill.def)
                    .iter()
                    .any(|rule| rule.cost.as_deref() == Some("face_down_hand"))
            });
        if tags_initial_hand {
            let cards = tx.state().hand_cards(*pid);
            tx.state_mut().face_down_hand_cards.extend(cards);
        }
    }

    for event in setup_events {
        tx.push_event(event);
    }
    let mut fires = crate::trigger::TriggerFireRecord::new();
    let startup_effects = crate::trigger::dispatch_for_events(tx, &[game_started], &mut fires)?;
    let _ = crate::effect::run_effects(tx, startup_effects)?;

    tx.push_event(DomainEvent::TurnStateChanged {
        current: first,
        phase: TurnPhaseTag::Draw,
    });
    // Lord's phaseDraw
    let draw_events = draw_cards(tx, first, DEFAULT_DRAW_COUNT)?;
    for event in draw_events {
        tx.push_event(event);
    }

    Ok(())
}

/// Guozhan 珠联璧合 spend-as-无中生有 primitive (ABI 641, ADR 0604).
///
/// Consumes one stack of the `ZHULIAN_BIHE_MARK` boon and draws two cards, emulating the
/// 【无中生有】 effect. Returns `Illegal` if the player has no boon mark to spend.
pub fn resolve_guozhan_use_zhulian_bihe(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let has_mark = tx
        .state()
        .players
        .get(player)
        .is_some_and(|p| {
            p.marks
                .iter()
                .any(|(m, c)| *m == crate::mode::ZHULIAN_BIHE_MARK && *c > 0)
        });
    if !has_mark {
        return Err(EffectError::Illegal(
            "no 珠联璧合 mark to spend".into(),
        ));
    }
    // Spend one stack.
    if let Some(p) = tx.state_mut().players.get_mut(player) {
        for (m, c) in &mut p.marks {
            if *m == crate::mode::ZHULIAN_BIHE_MARK && *c > 0 {
                *c -= 1;
                break;
            }
        }
        p.marks.retain(|(_, c)| *c > 0);
    }
    // Draw two cards (无中生有 effect).
    run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])
}

/// Bootstrap a guozhan (国战) match — double-general dealing + faction assignment.
///
/// ADR 0598 §6 phase B skeleton: both generals start revealed (`unseen = [false, false]`),
/// faction is computed from the pair and written into `group_overrides`. Phase C will add
/// hidden-general mechanics, 珠联璧合, and 野心家 nation-building.
pub fn bootstrap_guozhan(
    tx: &mut Transaction,
    player_count: u8,
    rng_seed: u64,
    ruleset_name: &str,
) -> Result<(), EffectError> {
    use crate::mode::{compute_group_from_pair, guozhan_max_hp, guozhan_player};
    use crate::state::{RulesetRef, SkillInstance};
    use engine_api::{RulesetId, SkillDefId, SkillId};

    crate::content::ensure_standard_package(std::sync::Arc::make_mut(&mut tx.state_mut().content));
    // ADR 0604: deal guozhan matches from the 256-character guozhan pool.
    crate::content::ensure_guozhan_package(std::sync::Arc::make_mut(&mut tx.state_mut().content));

    let st = tx.state_mut();
    let content_hash = st.content.combined_logic_hash;
    st.ruleset = RulesetRef {
        id: RulesetId::new(1),
        name: ruleset_name.to_string(),
        logic_hash: content_hash,
    };
    st.rng = RuleRng::from_seed(rng_seed).into_state();

    let draw = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::DrawPile,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _discard = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::DiscardPile,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _processing = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Processing,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });
    let _removed = st.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Removed,
        owner: PlayerId::NONE,
        cards: Vec::new(),
    });

    // Build the general pool: only characters with a recognised faction are eligible.
    let mut char_pool: Vec<u32> = st
        .content
        .characters()
        .filter(|c| {
            crate::mode::FACTIONS.iter().any(|f| c.group == *f)
        })
        .map(|c| c.id)
        .collect();
    if char_pool.len() < usize::from(player_count) * 2 {
        return Err(EffectError::Illegal(format!(
            "guozhan needs at least {} faction-tagged characters, found {}",
            usize::from(player_count) * 2,
            char_pool.len()
        )));
    }
    {
        let mut rng = RuleRng::from_state(st.rng);
        rng.shuffle(&mut char_pool);
        st.rng = rng.into_state();
    }

    let mut player_ids = Vec::new();
    for seat in 0..player_count {
        let main_id = char_pool[(seat as usize * 2) % char_pool.len()];
        let lt_id = char_pool[(seat as usize * 2 + 1) % char_pool.len()];
        let main_ch = st
            .content
            .character(main_id)
            .cloned()
            .ok_or_else(|| EffectError::Illegal(format!("unknown character {main_id}")))?;
        let lt_ch = st
            .content
            .character(lt_id)
            .cloned()
            .ok_or_else(|| EffectError::Illegal(format!("unknown character {lt_id}")))?;
        let max_hp = guozhan_max_hp(
            if main_ch.max_hp == 0 { 4 } else { main_ch.max_hp },
            if lt_ch.max_hp == 0 { 4 } else { lt_ch.max_hp },
        );
        // ADR 0598 phase C: 珠联璧合 (perfect pair) grants +1 max HP at deal time.
        let perfect = crate::mode::is_perfect_pair(&main_ch.name_key, &lt_ch.name_key);
        let max_hp = if perfect { max_hp.saturating_add(1) } else { max_hp };

        let pid = st.players.alloc(guozhan_player(seat, main_id, lt_id, max_hp));
        player_ids.push(pid);
        if perfect {
            // Award the 珠联璧合 boon mark (spend effects arrive with content migration).
            if let Some(p) = st.players.get_mut(pid) {
                p.marks.push((crate::mode::ZHULIAN_BIHE_MARK, 1));
                p.marks.sort_by_key(|(m, _)| *m);
            }
        }
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Hand,
            owner: pid,
            cards: Vec::new(),
        });
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Equip,
            owner: pid,
            cards: Vec::new(),
        });
        st.zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Judge,
            owner: pid,
            cards: Vec::new(),
        });

        // Grant both generals' skills. Phase C: tag each with its general slot so that
        // face-down (暗置) generals' skills can be gated by `skill_effectively_enabled`.
        // Both generals start revealed (unseen = [false, false]); the gate is inert until
        // a general is flipped face-down.
        let general_pair = [(&main_ch, 0u8), (&lt_ch, 1u8)];
        for &(ch, slot) in &general_pair {
            for &sdef in &ch.skills {
                let sid = st.skills.alloc(SkillInstance {
                    id: SkillId::NONE,
                    def: SkillDefId::new(sdef),
                    owner: pid,
                    enabled: true,
                    use_count: 0,
                    turn_fire_count: 0,
                    content_flags: 0,
                    play_phase_fire_count: 0,
                    play_phase_use_count: 0,
                    play_phase_category_mask: 0,
                    active_turn_banned: false,
                    turn_disabled: false,
                    disabled_by_skills: Vec::new(),
                    temporary_until_turn_end_of: PlayerId::NONE,
                    mark: 0,
                    round_fire_round: 0,
                    lifetime_fire_count: 0,
                    view_as_branch: 0,
                    view_as_used_branches: 0,
                    view_as_use_round: 0,
                    state_blob: Vec::new(),
                    source_free_physical_card: CardId::NONE,
                    general_slot: slot as u8,
                    content_storage: std::collections::BTreeMap::new(),
                });
                if let Some(p) = st.players.get_mut(pid) {
                    p.skills.push(sid);
                    p.skills.sort_by_key(|s| s.raw());
                }
                let content = st.content.clone();
                st.triggers
                    .register_skill(sid, SkillDefId::new(sdef), pid, &content);
                st.modifiers
                    .register_skill(sid, SkillDefId::new(sdef), pid, &content);
                let enabled = st
                    .skills
                    .get(sid)
                    .is_some_and(|skill| st.skill_effectively_enabled(skill));
                st.triggers.set_skill_enabled(sid, enabled);
                st.modifiers.set_skill_enabled(sid, enabled);
            }
        }

        // Resolve faction from the pair and store it in group_overrides.
        let group = compute_group_from_pair(Some(&main_ch.group), Some(&lt_ch.group));
        if let Some(g) = group {
            st.group_overrides.insert(pid, g);
        }
    }

    // Guozhan deck: identity deck + 12 guozhan equipment cards (ADR 0604).
    let specs = crate::content::guozhan_deck_specs();
    let mut deck_cards = Vec::new();
    for spec in specs {
        let cid = st.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(spec.def),
            kind: spec.kind,
            suit: spec.suit,
            rank: spec.rank,
            weapon_range: spec.weapon_range,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: false,
        });
        deck_cards.push(cid);
    }
    {
        let mut rng = RuleRng::from_state(st.rng);
        rng.shuffle(&mut deck_cards);
        st.rng = rng.into_state();
    }
    if let Some(z) = st.zones.get_mut(draw) {
        z.cards = deck_cards;
    }

    let pids = player_ids.clone();
    let first = pids.first().copied().unwrap_or(PlayerId::NONE);
    st.turn.round = 1;
    st.turn.current = first;
    st.turn.phase = Phase::Draw;

    let game_started = DomainEvent::GameStarted { player_count };
    let mut setup_events = vec![game_started.clone()];

    for pid in &pids {
        let count = crate::query::initial_hand_count_for(tx.state(), *pid, 4);
        let drawn = draw_cards(tx, *pid, count)?;
        setup_events.extend(drawn);
    }
    tx.state_mut().turn.gained_hand_cards.clear();

    for event in setup_events {
        tx.push_event(event);
    }
    let mut fires = crate::trigger::TriggerFireRecord::new();
    let startup_effects = crate::trigger::dispatch_for_events(tx, &[game_started], &mut fires)?;
    let _ = crate::effect::run_effects(tx, startup_effects)?;

    tx.push_event(DomainEvent::TurnStateChanged {
        current: first,
        phase: TurnPhaseTag::Draw,
    });
    let draw_events = draw_cards(tx, first, DEFAULT_DRAW_COUNT)?;
    for event in draw_events {
        tx.push_event(event);
    }

    Ok(())
}

/// Public helper to open a typed choice with continuation already set.
pub fn open_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    min_count: u8,
    max_count: u8,
    allow_cancel: bool,
    option_tokens: Vec<u32>,
    kind: ChoiceKind,
    cont: ContinuationKind,
    payload: Vec<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    {
        let r = &mut tx.state_mut().resolution;
        r.continuation_kind = cont;
        r.continuation_payload = payload;
    }
    request_choice(
        tx,
        observer,
        min_count,
        max_count,
        allow_cancel,
        option_tokens,
        kind,
    )
}

pub(crate) fn resolve_twfupan(
    tx: &mut Transaction,
    player: PlayerId,
    damage_amount: u32,
    card: CardId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !alive || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let eligible_targets = |tx: &Transaction| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.twfupan_target_state(*candidate) < 2)
            })
            .collect::<Vec<_>>()
    };
    match stage {
        0 => {
            if damage_amount == 0 {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount,
                    card: CardId::NONE,
                    target: PlayerId::NONE,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: damage_amount,
                    skill,
                }],
            )
        }
        1 => {
            let cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, player))
                .filter_map(|zone| tx.state().zones.get(zone))
                .flat_map(|zone| zone.cards.iter().copied())
                .collect();
            if cards.is_empty() || eligible_targets(tx).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwfupan,
                vec![1, player.raw(), damage_amount, 0, 0, skill.raw()],
            )
        }
        2 => {
            if card.is_none() {
                return Err(EffectError::Illegal("twfupan selected card missing".into()));
            }
            let options = eligible_targets(tx);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwfupan,
                vec![2, player.raw(), damage_amount, card.raw(), 0, skill.raw()],
            )
        }
        3 => {
            let previous = tx
                .state()
                .skills
                .get(skill)
                .map_or(2, |instance| instance.twfupan_target_state(target));
            if previous == 0 {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated twfupan skill")
                    .set_twfupan_target_state(target, 1);
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 2,
                        skill,
                    }],
                );
            }
            if previous == 1 {
                return open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    vec![1],
                    ChoiceKind::Confirm,
                    ContinuationKind::ResolveTwfupan,
                    vec![3, player.raw(), damage_amount, 0, target.raw(), skill.raw()],
                );
            }
            Ok(Vec::new())
        }
        4 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_none_or(|instance| instance.twfupan_target_state(target) != 1)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated twfupan skill")
                .set_twfupan_target_state(target, 2);
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown twfupan stage".into())),
    }
}

pub(crate) fn resolve_twjiaohua(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    event_cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_is_min_hp = tx
        .state()
        .players
        .get(target)
        .filter(|target| target.alive)
        .is_some_and(|target_state| {
            tx.state()
                .players
                .iter_live()
                .filter(|other| other.alive)
                .all(|other| target_state.hp <= other.hp)
        });
    if !originating_skill_is_enabled(tx, player, skill)
        || target.is_none()
        || (target != player && !target_is_min_hp)
    {
        return Err(EffectError::Illegal("invalid TW Jiaohua context".into()));
    }

    let category = |card: &crate::state::Card| {
        if card.kind.is_basic() {
            0b001
        } else if card.kind.is_trick() {
            0b010
        } else if card.kind.is_equip() {
            0b100
        } else {
            0
        }
    };
    let drawn_mask = event_cards.iter().fold(0u8, |mask, card| {
        mask | tx.state().cards.get(*card).map_or(0, category)
    });
    let key = crate::state::turn_flag::twjiaohua_category_by_player(target);
    let gained_mask = tx.state().turn.flag(key) as u8;
    let available = !(drawn_mask | gained_mask) & 0b111;
    if available == 0 {
        return Err(EffectError::Illegal(
            "TW Jiaohua has no available category".into(),
        ));
    }

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let candidate = [draw, discard].into_iter().find_map(|zone| {
        tx.state().zones.get(zone).and_then(|zone_state| {
            zone_state.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| category(card) & available != 0)
            })
        })
    });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("TW Jiaohua card disappeared".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("TW Jiaohua target hand missing".into()))?;
    let gained_category = tx
        .state()
        .cards
        .get(card)
        .map(category)
        .ok_or_else(|| EffectError::Illegal("TW Jiaohua card disappeared".into()))?;
    tx.state_mut()
        .turn
        .set_flag(key, i32::from(gained_mask | gained_category));
    move_cards(tx, vec![card], hand, Some(from))
}

pub(crate) fn resolve_twkujian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: &[CardId],
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid TW Kujian context".into()));
    }
    match action {
        0 => {
            if !(1..=3).contains(&cards.len()) {
                return Err(EffectError::Illegal(
                    "TW Kujian requires one to three cards".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Kujian owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("TW Kujian target hand missing".into()))?;
            let mut selected = cards.to_vec();
            selected.sort_unstable_by_key(|card| card.raw());
            selected.dedup();
            if selected.len() != cards.len()
                || selected.iter().any(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_none_or(|card| card.zone != hand)
                })
            {
                return Err(EffectError::Illegal("invalid TW Kujian gift cards".into()));
            }
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Kujian skill")
                .add_twkujian_cards(&selected)
            {
                return Err(EffectError::ContentFault(
                    "TW Kujian state is occupied".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: selected,
                    to: target_hand,
                    from: hand,
                }],
            )
        }
        1 | 2 => {
            let tracked = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Kujian skill")
                .take_twkujian_cards(cards);
            if tracked.is_empty() {
                return Ok(Vec::new());
            }
            let count = tracked.len() as u32;
            if action == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawEachBySeat {
                        first: player,
                        second: target,
                        count,
                        skill,
                    }],
                );
            }
            let seats = tx.state().players.alive_by_seat();
            let start = seats
                .iter()
                .position(|candidate| *candidate == tx.state().turn.current)
                .unwrap_or(0);
            let mut participants = [player, target];
            participants.sort_by_key(|candidate| {
                seats
                    .iter()
                    .position(|seated| seated == candidate)
                    .map_or(usize::MAX, |index| {
                        (index + seats.len() - start) % seats.len()
                    })
            });
            let mut effects = Vec::with_capacity(tracked.len() * 2);
            for _ in tracked {
                effects.extend(participants.iter().map(|participant| {
                    EffectRequest::ChooseDiscardOneHandOrEquip {
                        player: *participant,
                    }
                }));
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown TW Kujian action".into())),
    }
}

pub(crate) fn resolve_twruilian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    event_cards: &[CardId],
    action: u8,
    category: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .set_twruilian_prompt_round(round);
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwruilian,
                vec![0, player.raw(), 0, skill.raw(), 0],
            )
        }
        1 => {
            let mask = event_cards.iter().fold(0, |mask, card| {
                mask | tx.state().cards.get(*card).map_or(0, |card| {
                    if card.kind.is_basic() {
                        1
                    } else if card.kind.is_trick() {
                        2
                    } else if card.kind.is_equip() {
                        4
                    } else {
                        0
                    }
                })
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .record_twruilian_discard(target, event_cards.len() as u32, mask);
            Ok(Vec::new())
        }
        2 => {
            let Some((count, mask)) = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .take_twruilian_summary(target)
            else {
                return Ok(Vec::new());
            };
            if count < 2 || mask == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = [(1, 1), (2, 2), (3, 4)]
                .into_iter()
                .filter_map(|(token, bit)| (mask & bit != 0).then_some(token))
                .collect();
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwruilian,
                vec![1, player.raw(), target.raw(), skill.raw(), u32::from(mask)],
            )
        }
        3 => {
            let matches = |kind: CardKind| match category {
                1 => kind.is_basic(),
                2 => kind.is_trick(),
                3 => kind.is_equip(),
                _ => false,
            };
            if !matches!(category, 1..=3) {
                return Err(EffectError::ContentFault(
                    "unknown TW Ruilian category".into(),
                ));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(discard)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| matches(card.kind))
                })
                .take(2)
                .collect();
            let effects = [target, player]
                .into_iter()
                .zip(cards)
                .filter_map(|(recipient, card)| {
                    tx.state()
                        .zones
                        .find(ZoneKind::Hand, recipient)
                        .map(|hand| EffectRequest::GiveCards {
                            cards: vec![card],
                            to: hand,
                            from: discard,
                        })
                })
                .collect();
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Ruilian action".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twjianwei(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    target: PlayerId,
    _card: CardId,
    winner: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let can_compare = |tx: &Transaction, left: PlayerId, right: PlayerId| {
        left != right
            && tx
                .state()
                .players
                .get(left)
                .is_some_and(|participant| participant.alive)
            && tx
                .state()
                .players
                .get(right)
                .is_some_and(|participant| participant.alive)
            && !tx.state().hand_cards(left).is_empty()
            && !tx.state().hand_cards(right).is_empty()
    };
    match stage {
        0 => {
            if actor != player || physical_equipped_weapons(tx, player).is_empty() {
                return Ok(Vec::new());
            }
            let range = match crate::query::query(
                tx.state(),
                &QueryRequest::AttackRange { player },
                &mut crate::query::QueryContext::new(),
            ) {
                QueryResponse::Int(value) => value,
                _ => 0,
            };
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| can_compare(tx, player, *target))
                .filter(|target| {
                    matches!(
                        crate::query::query(
                            tx.state(),
                            &QueryRequest::Distance {
                                from: player,
                                to: *target,
                            },
                            &mut crate::query::QueryContext::new(),
                        ),
                        QueryResponse::Int(distance) if distance <= range
                    )
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwjianwei,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if actor == player || !can_compare(tx, actor, player) {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                actor,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwjianwei,
                vec![1, player.raw(), actor.raw(), skill.raw()],
            )
        }
        2 => {
            if !can_compare(tx, actor, target) {
                return Ok(Vec::new());
            }
            let options = tx.state().hand_cards(actor);
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwjianwei,
                vec![2, player.raw(), actor.raw(), target.raw(), skill.raw()],
            )
        }
        3 => {
            let comparison = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twjianwei_compare());
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Jianwei skill")
                .clear_twjianwei_compare();
            let Some((actor, target)) = comparison else {
                return Ok(Vec::new());
            };
            if winner == player {
                let loser = if player == actor { target } else { actor };
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    loser,
                    TargetCardScope::HandEquipJudge,
                );
                let zone_count = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .filter(|kind| {
                        tx.state().zones.find(*kind, loser).is_some_and(|zone| {
                            options.iter().any(|card| {
                                tx.state()
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.zone == zone)
                            })
                        })
                    })
                    .count();
                if zone_count == 0 {
                    return Ok(Vec::new());
                }
                return open_choice(
                    tx,
                    player,
                    zone_count as u8,
                    zone_count as u8,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveTwjianwei,
                    vec![3, player.raw(), loser.raw(), skill.raw()],
                );
            }
            if winner.is_none() || player != actor && player != target {
                return Ok(Vec::new());
            }
            let weapons = physical_equipped_weapons(tx, player);
            if weapons.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, winner)
                .ok_or_else(|| EffectError::EngineFault("winner hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: weapons,
                    to,
                    from,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown TW Jianwei stage".into())),
    }
}

pub(crate) fn resolve_twtanfeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    phase: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Prepare
    {
        return Ok(Vec::new());
    }
    let alive = |tx: &Transaction, participant: PlayerId| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    };
    match stage {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .filter(|candidate| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        *candidate,
                        TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwtanfeng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquipJudge,
            );
            if !alive(tx, target) || options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwtanfeng,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::command::play::twtanfeng_sha_source_cards(tx, target, player);
            if options.is_empty() {
                return resolve_twtanfeng(tx, player, target, CardId::NONE, 4, 0, skill);
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwtanfeng,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        3 => {
            let legal = crate::command::play::twtanfeng_sha_targets(tx, target, player, card);
            if legal.is_empty() {
                return Err(EffectError::Illegal(
                    "Tanfeng Sha source became illegal".into(),
                ));
            }
            if crate::query::sha_targets_all_for(tx.state(), target) {
                crate::command::play::play_twtanfeng_sha_source(tx, target, player, card, &legal)?;
                return Ok(Vec::new());
            }
            let max = crate::query::sha_target_limit_for(tx.state(), target) as usize;
            if max <= 1 {
                crate::command::play::play_twtanfeng_sha_source(
                    tx,
                    target,
                    player,
                    card,
                    &[player],
                )?;
                return Ok(Vec::new());
            }
            let extras: Vec<_> = legal
                .into_iter()
                .filter(|candidate| *candidate != player)
                .collect();
            if extras.is_empty() {
                crate::command::play::play_twtanfeng_sha_source(
                    tx,
                    target,
                    player,
                    card,
                    &[player],
                )?;
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                0,
                extras.len().min(max - 1).min(u8::MAX as usize) as u8,
                true,
                extras.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwtanfeng,
                vec![3, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        4 => run_effects(
            tx,
            vec![
                EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                },
                EffectRequest::ResolveTwtanfeng {
                    player,
                    target,
                    card: CardId::NONE,
                    stage: 5,
                    phase: 0,
                    skill,
                },
            ],
        ),
        5 => {
            if !alive(tx, target) {
                return Ok(Vec::new());
            }
            let mut phases = Vec::new();
            if !tx.state().pending_judge_skips.contains(&player) {
                phases.push(Phase::Judge.to_u8() as u32);
            }
            if !tx.state().turn.skip_draw() {
                phases.push(Phase::Draw.to_u8() as u32);
            }
            if !tx.state().turn.skip_play() {
                phases.push(Phase::Play.to_u8() as u32);
            }
            if !tx.state().turn.skip_discard() {
                phases.push(Phase::Discard.to_u8() as u32);
            }
            if phases.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                phases,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwtanfeng,
                vec![4, player.raw(), target.raw(), skill.raw()],
            )
        }
        6 => match Phase::from_u8(phase) {
            Phase::Judge => skip_next_judge_phase(tx, player),
            Phase::Draw => {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
                Ok(Vec::new())
            }
            Phase::Play => skip_next_play_phase(tx, player),
            Phase::Discard => {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
                Ok(Vec::new())
            }
            _ => Err(EffectError::Illegal("invalid Tanfeng skipped phase".into())),
        },
        _ => Err(EffectError::ContentFault("unknown TW Tanfeng stage".into())),
    }
}

pub(crate) fn twzhibian_movable_cards(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        TargetCardScope::HandEquipJudge,
    );
    let player_judge = tx.state().zones.find(ZoneKind::Judge, player);
    gainable
        .into_iter()
        .filter(|card| {
            let Some(card_state) = tx.state().cards.get(*card) else {
                return false;
            };
            match tx.state().zones.get(card_state.zone).map(|zone| zone.kind) {
                Some(ZoneKind::Hand) => true,
                Some(ZoneKind::Equip) => card_state.kind.equip_slot().is_some_and(|slot| {
                    tx.state()
                        .players
                        .get(player)
                        .is_some_and(|owner| owner.equip_slot_enabled(slot))
                }),
                Some(ZoneKind::Judge) => {
                    card_state.kind.is_delayed_trick()
                        && player_judge.is_some_and(|zone| {
                            tx.state().zones.get(zone).is_some_and(|zone| {
                                zone.cards.iter().all(|existing| {
                                    tx.state()
                                        .cards
                                        .get(*existing)
                                        .is_none_or(|existing| existing.kind != card_state.kind)
                                })
                            })
                        })
                }
                _ => false,
            }
        })
        .collect()
}

pub(crate) fn twzhibian_move_effects(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if !twzhibian_movable_cards(tx, player, target).contains(&card) {
        return Err(EffectError::Illegal(
            "TW Zhibian card is no longer movable".into(),
        ));
    }
    let card_state = tx.state().cards.get(card).unwrap();
    let source = card_state.zone;
    let source_kind = tx.state().zones.get(source).unwrap().kind;
    let destination_kind = source_kind;
    let destination = tx
        .state()
        .zones
        .find(destination_kind, player)
        .ok_or_else(|| EffectError::EngineFault("TW Zhibian destination missing".into()))?;
    let mut effects = Vec::new();
    if source_kind == ZoneKind::Equip {
        let slot = card_state.kind.equip_slot().unwrap();
        let replaced: Vec<_> = tx
            .state()
            .zones
            .get(destination)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|existing| {
                tx.state()
                    .cards
                    .get(*existing)
                    .and_then(|card| card.kind.equip_slot())
                    == Some(slot)
            })
            .collect();
        if !replaced.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards: replaced,
                to: tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .unwrap(),
                from: Some(destination),
            });
        }
    }
    effects.push(if source_kind == ZoneKind::Hand {
        EffectRequest::GiveCards {
            cards: vec![card],
            to: destination,
            from: source,
        }
    } else {
        EffectRequest::MoveCards {
            cards: vec![card],
            to: destination,
            from: Some(source),
        }
    });
    Ok(effects)
}

pub(crate) fn resolve_twzhibian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player
                || !matches!(tx.state().turn.phase, Phase::Prepare | Phase::Play)
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player && !tx.state().hand_cards(*target).is_empty())
                .filter(|_| !tx.state().hand_cards(player).is_empty())
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwzhibian,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Zhibian skill")
                .clear_twzhibian_compare();
            if winner != player {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            let can_move = !twzhibian_movable_cards(tx, player, target).is_empty();
            let can_recover = tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.hp < owner.max_hp as i32);
            if !can_move && !can_recover {
                return Ok(Vec::new());
            }
            let can_backwater = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .any(|card| {
                    tx.state().cards.get(card).is_some_and(|card| {
                        !crate::query::effective_card_kind(tx.state(), player, card).is_basic()
                    })
                });
            let mut options = Vec::new();
            if can_move {
                options.push(1);
            }
            if can_recover {
                options.push(2);
            }
            if can_backwater {
                options.push(3);
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwzhibian,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            let options = twzhibian_movable_cards(tx, player, target);
            if options.is_empty() {
                return recover_hp(tx, player, player, 1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzhibian,
                vec![3, player.raw(), target.raw(), skill.raw(), 3],
            )
        }
        _ => Err(EffectError::ContentFault("unknown TW Zhibian stage".into())),
    }
}


/// 劫囚·装备栏编排（ABI 673，tw twjieqiu 多路器）：
/// sub 0：选择一名其他存活角色 → 禁用其全部装备槽（现有装备入弃牌堆）并令其
///        摸 k 张（k=禁用前装备区牌数）→ 打上 BUFF 标记；
/// sub 2：若目标仍有禁用槽，为持有者入队一个纯额外回合（check=0 语义）。
// TWJIEQIU_BUFF_MARK 常量上移至 crate::content（dispatch 恢复钩子共用）。

pub(crate) fn resolve_twjieqiu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    subaction: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    match subaction {
        0 => {
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::TwjieqiuSelect,
                vec![player.raw(), 0, skill.raw()],
            )
        }
        2 => {
            let disabled = tx
                .state()
                .players
                .get(target)
                .is_some_and(|p| {
                    p.alive && p.disabled_equip_slots.iter().any(|&count| count > 0)
                });
            if !disabled {
                return Ok(Vec::new());
            }
            queue_extra_turn(tx, player, 0, skill)
        }
        _ => Err(EffectError::Illegal("bad twjieqiu subaction".into())),
    }
}

/// Cont 282 resolver（劫囚·选目标段）：禁用目标全部装备槽（装备入弃牌堆），
/// 令其摸 k 张（k=禁用前装备区牌数），并打上 BUFF 标记。
pub(crate) fn resolve_twjieqiu_select(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _sub, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad twjieqiu-select payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("twjieqiu chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "twjieqiu must select exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal("bad twjieqiu target".into()));
    }
    // k = 禁用前装备区牌数。
    let k = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| zone.cards.len() as u32)
        .unwrap_or(0);
    let slots: Vec<u8> = (1u8..=5u8).collect();
    disable_equip_slots(tx, target, slots)?;
    let mut effects = Vec::new();
    if k > 0 {
        effects.push(EffectRequest::DrawCards {
            player: target,
            count: k,
        });
    }
    effects.push(EffectRequest::AddMark {
        player: target,
        mark: engine_api::MarkDefId::new(crate::content::TWJIEQIU_BUFF_MARK),
        delta: 1,
    });
    run_effects(tx, effects)?;
    Ok(())
}

