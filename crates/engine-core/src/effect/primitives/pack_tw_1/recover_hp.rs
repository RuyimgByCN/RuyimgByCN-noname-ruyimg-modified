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
    if current > 0 && count <= 0 {
        tx.state_mut()
            .turn
            .removed_marks
            .insert((mark, player));
    }
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
    if current > 0 && count <= 0 {
        tx.state_mut()
            .turn
            .removed_marks
            .insert((mark, player));
    }
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
        prompt: None,
        native_kind: None,
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
