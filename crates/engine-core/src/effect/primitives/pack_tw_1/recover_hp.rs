//! Effect primitives for `recover_hp` and related helpers.

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

pub(crate) fn enter_dying(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
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
        let has_zhulian = tx.state().players.get(player).is_some_and(|pp| {
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
    super::super::run_effects(
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

pub(crate) fn begin_turn(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
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
