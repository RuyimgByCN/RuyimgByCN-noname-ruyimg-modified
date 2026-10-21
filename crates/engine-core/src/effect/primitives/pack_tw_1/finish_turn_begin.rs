//! Effect primitives for `finish_turn_begin` and related helpers.

use super::*;

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
    let has_mark = tx.state().players.get(player).is_some_and(|p| {
        p.marks
            .iter()
            .any(|(m, c)| *m == crate::mode::ZHULIAN_BIHE_MARK && *c > 0)
    });
    if !has_mark {
        return Err(EffectError::Illegal("no 珠联璧合 mark to spend".into()));
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
        .filter(|c| crate::mode::FACTIONS.iter().any(|f| c.group == *f))
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
            if main_ch.max_hp == 0 {
                4
            } else {
                main_ch.max_hp
            },
            if lt_ch.max_hp == 0 { 4 } else { lt_ch.max_hp },
        );
        // ADR 0598 phase C: 珠联璧合 (perfect pair) grants +1 max HP at deal time.
        let perfect = crate::mode::is_perfect_pair(&main_ch.name_key, &lt_ch.name_key);
        let max_hp = if perfect {
            max_hp.saturating_add(1)
        } else {
            max_hp
        };

        let pid = st
            .players
            .alloc(guozhan_player(seat, main_id, lt_id, max_hp));
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
