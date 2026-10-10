//! Canonical state hash.
//!
//! Must NOT hash Rust memory layout, debug output, or ordinary Protobuf bytes.
//! Fixed field order + stable entity ids only.

use crate::state::GameState;
use engine_api::{MatchOutcome, Suit};
use rule_api::EffectRequest;

/// Compute BLAKE3 canonical hash of `GameState`.
pub fn canonical_state_hash(state: &GameState) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    // Domain separator — bump when encoding changes intentionally.
    h.update(b"noname-next/canonical-state/v3");

    put_u64(&mut h, state.revision);
    put_u32(&mut h, state.ruleset.id.raw());
    put_str(&mut h, &state.ruleset.name);
    h.update(&state.ruleset.logic_hash);

    // Turn
    put_u32(&mut h, state.turn.round);
    put_u32(&mut h, state.turn.current.raw());
    put_u8(&mut h, state.turn.phase.to_u8());
    put_u32(&mut h, state.turn.flags.len() as u32);
    for (k, v) in &state.turn.flags {
        put_u32(&mut h, *k);
        put_i32(&mut h, *v);
    }
    if !state.turn.lost_cards.is_empty() {
        h.update(b"turn-lost-cards/v1");
        put_u32(&mut h, state.turn.lost_cards.len() as u32);
        for (player, card) in &state.turn.lost_cards {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, card.raw());
        }
    }
    if !state.turn.lost_card_tags.is_empty() {
        // gaintag_map foundation: tags carried by cards lost this turn. Hash-neutral when empty.
        h.update(b"turn-lost-card-tags/v1");
        put_u32(&mut h, state.turn.lost_card_tags.len() as u32);
        for tag in &state.turn.lost_card_tags {
            put_u32(&mut h, *tag);
        }
    }
    if !state.turn.discarded_cards.is_empty() {
        h.update(b"turn-discarded-cards/v1");
        put_u32(&mut h, state.turn.discarded_cards.len() as u32);
        for card in &state.turn.discarded_cards {
            put_u32(&mut h, card.raw());
        }
    }
    if !state.turn.entered_discard_cards.is_empty() {
        h.update(b"turn-entered-discard-cards/v1");
        put_ids(
            &mut h,
            state
                .turn
                .entered_discard_cards
                .iter()
                .map(|card| card.raw()),
        );
    }
    if !state.turn.ddd_chengjing_eligible_uses.is_empty() {
        h.update(b"turn-ddd-chengjing-eligible-uses/v1");
        put_ids(
            &mut h,
            state
                .turn
                .ddd_chengjing_eligible_uses
                .iter()
                .map(|card| card.raw()),
        );
    }
    if !state.turn.damage_card_materials.is_empty() {
        h.update(b"turn-damage-card-materials/v1");
        put_ids(
            &mut h,
            state
                .turn
                .damage_card_materials
                .iter()
                .map(|card| card.raw()),
        );
    }
    if !state.turn.use_ban_pairs.is_empty() {
        h.update(b"turn-use-ban-pairs/v1");
        put_u32(&mut h, state.turn.use_ban_pairs.len() as u32);
        for (user, target) in &state.turn.use_ban_pairs {
            put_u32(&mut h, user.raw());
            put_u32(&mut h, target.raw());
        }
    }
    if !state.turn.gained_hand_cards.is_empty() {
        h.update(b"turn-gained-hand-cards/v1");
        put_u32(&mut h, state.turn.gained_hand_cards.len() as u32);
        for (player, card) in &state.turn.gained_hand_cards {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, card.raw());
        }
    }
    if !state.turn.play_phase_card_gain_players.is_empty() {
        h.update(b"play-phase-card-gain-players/v1");
        put_u32(&mut h, state.turn.play_phase_card_gain_players.len() as u32);
        for player in &state.turn.play_phase_card_gain_players {
            put_u32(&mut h, player.raw());
        }
    }
    if !state.turn.play_phase_draw_players.is_empty() {
        h.update(b"play-phase-draw-players/v1");
        put_u32(&mut h, state.turn.play_phase_draw_players.len() as u32);
        for player in &state.turn.play_phase_draw_players {
            put_u32(&mut h, player.raw());
        }
    }
    if !state.turn.hand_limit_exempt_cards.is_empty() {
        h.update(b"turn-hand-limit-exempt-cards/v1");
        put_u32(&mut h, state.turn.hand_limit_exempt_cards.len() as u32);
        for (player, card) in &state.turn.hand_limit_exempt_cards {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, card.raw());
        }
    }
    if !state.turn.damage_pair_counts.is_empty() {
        h.update(b"turn-damage-pair-counts/v1");
        put_u32(&mut h, state.turn.damage_pair_counts.len() as u32);
        for ((source, target), count) in &state.turn.damage_pair_counts {
            put_u32(&mut h, source.raw());
            put_u32(&mut h, target.raw());
            put_u32(&mut h, *count);
        }
    }
    if !state.turn.card_target_counts.is_empty() {
        h.update(b"turn-card-target-counts/v1");
        put_u32(&mut h, state.turn.card_target_counts.len() as u32);
        for ((user, target), count) in &state.turn.card_target_counts {
            put_u32(&mut h, user.raw());
            put_u32(&mut h, target.raw());
            put_u32(&mut h, *count);
        }
    }
    if !state.turn.dying_caused_targets.is_empty() {
        h.update(b"turn-dying-caused-targets/v1");
        put_u32(&mut h, state.turn.dying_caused_targets.len() as u32);
        for (source, target) in &state.turn.dying_caused_targets {
            put_u32(&mut h, source.raw());
            put_u32(&mut h, target.raw());
        }
    }
    if !state.turn.entered_dying_players.is_empty() {
        h.update(b"turn-entered-dying-players/v1");
        put_ids(
            &mut h,
            state
                .turn
                .entered_dying_players
                .iter()
                .map(|player| player.raw()),
        );
    }
    if !state.turn.used_playable_skill_pile_cards.is_empty() {
        h.update(b"turn-used-playable-skill-pile-cards/v1");
        put_u32(
            &mut h,
            state.turn.used_playable_skill_pile_cards.len() as u32,
        );
        for (player, card) in &state.turn.used_playable_skill_pile_cards {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, card.raw());
        }
    }
    if !state.turn.used_playable_skill_piles.is_empty() {
        h.update(b"turn-used-playable-skill-piles/v1");
        put_u32(&mut h, state.turn.used_playable_skill_piles.len() as u32);
        for (player, skill_def) in &state.turn.used_playable_skill_piles {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, *skill_def);
        }
    }
    if state.turn.opening_extra_play_active_skill.is_some()
        || state.turn.opening_extra_play_damage_baseline != 0
        || !state.turn.opening_extra_play_queue.is_empty()
    {
        h.update(b"opening-extra-play-origin/v1");
        put_u32(&mut h, state.turn.opening_extra_play_active_skill.raw());
        put_u32(&mut h, state.turn.opening_extra_play_damage_baseline);
        put_u32(&mut h, state.turn.opening_extra_play_queue.len() as u32);
        for skill in &state.turn.opening_extra_play_queue {
            put_u32(&mut h, skill.raw());
        }
    }
    if state.turn.extra_turn_anchor.is_some() || !state.turn.extra_turn_queue.is_empty() {
        h.update(b"full-extra-turn/v1");
        put_u32(&mut h, state.turn.extra_turn_anchor.raw());
        put_u32(&mut h, state.turn.extra_turn_queue.len() as u32);
        for player in &state.turn.extra_turn_queue {
            put_u32(&mut h, player.raw());
        }
    }
    if state.turn.extra_play_phase_active || !state.turn.extra_play_phase_queue.is_empty() {
        h.update(b"extra-play-phase/v1");
        put_bool(&mut h, state.turn.extra_play_phase_active);
        put_u32(&mut h, state.turn.extra_play_phase_queue.len() as u32);
        for player in &state.turn.extra_play_phase_queue {
            put_u32(&mut h, player.raw());
        }
    }
    if state.turn.extra_draw_phase_active || !state.turn.extra_draw_phase_queue.is_empty() {
        h.update(b"extra-draw-phase/v1");
        put_bool(&mut h, state.turn.extra_draw_phase_active);
        put_u32(&mut h, state.turn.extra_draw_phase_queue.len() as u32);
        for player in &state.turn.extra_draw_phase_queue {
            put_u32(&mut h, player.raw());
        }
    }
    if state.turn.non_locked_skill_suppression.is_some() {
        h.update(b"turn-skill-suppression/v1");
        put_u32(&mut h, state.turn.non_locked_skill_suppression.raw());
    }
    if !state.turn.non_locked_skill_suppressed_players.is_empty() {
        h.update(b"turn-player-skill-suppression/v1");
        put_ids(
            &mut h,
            state
                .turn
                .non_locked_skill_suppressed_players
                .iter()
                .map(|player| player.raw()),
        );
    }

    // Players
    put_u32(&mut h, state.players.next_id());
    let players = state.players.sorted_live();
    put_u32(&mut h, players.len() as u32);
    for p in players {
        put_u32(&mut h, p.id.raw());
        put_u8(&mut h, p.seat.raw());
        put_i32(&mut h, p.hp);
        put_u32(&mut h, p.max_hp);
        put_bool(&mut h, p.alive);
        put_bool(&mut h, p.dying);
        if p.turned_over {
            h.update(b"player-turned-over/v1");
        }
        put_u8(&mut h, identity_tag(p.identity));
        put_u32(&mut h, p.character_def);
        if p.character_def2 != 0 {
            // Guozhan lieutenant general (ADR 0598 §D2). Identity mode leaves this 0,
            // so the segment is absent and identity hashes are unchanged.
            h.update(b"player-character-def2/v1");
            put_u32(&mut h, p.character_def2);
        }
        if let Some(group) = state.group_overrides.get(&p.id) {
            h.update(b"player-group-override/v1");
            put_str(&mut h, group);
        }
        put_u32(&mut h, p.skills.len() as u32);
        for s in &p.skills {
            put_u32(&mut h, s.raw());
        }
        put_u32(&mut h, p.marks.len() as u32);
        for (m, c) in &p.marks {
            put_u32(&mut h, *m);
            put_i32(&mut h, *c);
        }
        if p.disabled_equip_slots.iter().any(|count| *count != 0) {
            h.update(b"disabled-equip-slots/v1");
            h.update(&p.disabled_equip_slots);
        }
        if p.unseen[0] || p.unseen[1] {
            // Guozhan face-down generals (ADR 0598 §D2). Identity mode defaults to
            // [false, false], so this segment is absent and identity hashes are unchanged.
            h.update(b"player-unseen/v1");
            put_u8(&mut h, p.unseen[0] as u8);
            put_u8(&mut h, p.unseen[1] as u8);
        }
    }
    if !state.ambition_alliances.is_empty() {
        // Guozhan ambition-lord alliances (ADR 0598). Identity mode keeps this empty, so the
        // segment is absent and identity hashes are unchanged.
        h.update(b"ambition-alliances/v1");
        put_u32(&mut h, state.ambition_alliances.len() as u32);
        for pair in &state.ambition_alliances {
            put_u32(&mut h, pair[0]);
            put_u32(&mut h, pair[1]);
        }
    }
    if !state.face_down_hand_cards.is_empty() {
        h.update(b"face-down-hand-cards/v1");
        put_ids(
            &mut h,
            state.face_down_hand_cards.iter().map(|card| card.raw()),
        );
    }
    if !state.lifetime_dying_players.is_empty() {
        h.update(b"lifetime-dying-players/v1");
        put_ids(
            &mut h,
            state
                .lifetime_dying_players
                .iter()
                .map(|player| player.raw()),
        );
    }
    if !state.pending_judge_skips.is_empty() {
        h.update(b"pending-judge-skips/v1");
        put_ids(
            &mut h,
            state.pending_judge_skips.iter().map(|player| player.raw()),
        );
    }
    if !state.disabled_judge_zones.is_empty() {
        h.update(b"disabled-judge-zones/v1");
        put_ids(
            &mut h,
            state.disabled_judge_zones.iter().map(|player| player.raw()),
        );
    }

    // Cards
    put_u32(&mut h, state.cards.next_id());
    let cards = state.cards.sorted_live();
    put_u32(&mut h, cards.len() as u32);
    for c in cards {
        put_u32(&mut h, c.id.raw());
        put_u32(&mut h, c.def.raw());
        put_u8(&mut h, c.kind.tag());
        put_u8(&mut h, suit_tag(c.suit));
        put_u8(&mut h, c.rank);
        put_u8(&mut h, c.weapon_range);
        put_u32(&mut h, c.zone.raw());
        put_u32(&mut h, c.owner.raw());
        put_bool(&mut h, c.face_up);
    }
    if !state.content_card_tags.is_empty() {
        h.update(b"content-card-tags/v1");
        put_u32(&mut h, state.content_card_tags.len() as u32);
        for (card, tags) in &state.content_card_tags {
            put_u32(&mut h, card.raw());
            put_u32(&mut h, tags.len() as u32);
            for tag in tags {
                put_u32(&mut h, *tag);
            }
        }
    }

    // Zones
    put_u32(&mut h, state.zones.next_id());
    let zones = state.zones.sorted_live();
    put_u32(&mut h, zones.len() as u32);
    for z in zones {
        put_u32(&mut h, z.id.raw());
        put_u8(&mut h, zone_kind_tag(z.kind));
        put_u32(&mut h, z.owner.raw());
        put_u32(&mut h, z.cards.len() as u32);
        for c in &z.cards {
            put_u32(&mut h, c.raw());
        }
    }

    // Skills
    put_u32(&mut h, state.skills.next_id());
    let skills = state.skills.sorted_live();
    put_u32(&mut h, skills.len() as u32);
    for s in skills {
        put_u32(&mut h, s.id.raw());
        put_u32(&mut h, s.def.raw());
        put_u32(&mut h, s.owner.raw());
        put_bool(&mut h, s.enabled);
        put_u32(&mut h, s.use_count);
        if s.turn_fire_count != 0 || s.round_fire_round == state.turn.round {
            h.update(b"skill-trigger-counts/v1");
            put_u32(&mut h, s.turn_fire_count);
            put_bool(&mut h, s.round_fire_round == state.turn.round);
        }
        if s.play_phase_fire_count != 0 {
            h.update(b"skill-play-phase-trigger-count/v1");
            put_u32(&mut h, s.play_phase_fire_count);
        }
        if s.play_phase_use_count != 0 {
            h.update(b"skill-play-phase-use-count/v1");
            put_u32(&mut h, s.play_phase_use_count);
        }
        if s.play_phase_category_mask != 0 {
            h.update(b"skill-play-phase-category-mask/v1");
            put_u8(&mut h, s.play_phase_category_mask);
        }
        if s.active_turn_banned {
            h.update(b"skill-active-turn-ban/v1");
        }
        if s.turn_disabled {
            h.update(b"skill-turn-disabled/v1");
        }
        if !s.disabled_by_skills.is_empty() {
            h.update(b"skill-disabled-by-skills/v1");
            put_ids(&mut h, s.disabled_by_skills.iter().map(|skill| skill.raw()));
        }
        if s.temporary_until_turn_end_of.is_some() {
            h.update(b"skill-temporary-until-turn-end/v1");
            put_u32(&mut h, s.temporary_until_turn_end_of.raw());
        }
        if s.lifetime_fire_count != 0 {
            h.update(b"skill-lifetime-trigger-count/v1");
            put_u32(&mut h, s.lifetime_fire_count);
        }
        if s.view_as_branch != 0
            || (s.view_as_use_round == state.turn.round && s.view_as_used_branches != 0)
        {
            h.update(b"source-free-view-as-state/v1");
            put_u8(&mut h, s.view_as_branch);
            put_bool(&mut h, s.view_as_use_round == state.turn.round);
            put_u32(
                &mut h,
                if s.view_as_use_round == state.turn.round {
                    s.view_as_used_branches
                } else {
                    0
                },
            );
        }
        put_u32(&mut h, s.state_blob.len() as u32);
        h.update(&s.state_blob);
        if let Some(card) = s.source_free_physical_card() {
            h.update(b"source-free-physical-card/v1");
            put_u32(&mut h, card.raw());
        }
        if s.mark != 0 {
            h.update(b"skill-mark/v1");
            put_u32(&mut h, s.mark);
        }
        if !s.content_storage.is_empty() {
            // Keyed content storage (key→value). Hash-neutral when empty.
            h.update(b"skill-content-storage/v1");
            put_u32(&mut h, s.content_storage.len() as u32);
            for (key, value) in &s.content_storage {
                put_u32(&mut h, *key);
                put_u32(&mut h, *value);
            }
        }
        if s.content_flags != 0 {
            put_u8(&mut h, 0xcf);
            h.update(&s.content_flags.to_le_bytes());
        }
        if s.general_slot != crate::state::GENERAL_SLOT_NONE {
            // Guozhan general attribution (ADR 0598 phase C). Identity skills keep the
            // default sentinel, so this segment is absent and identity hashes are unchanged.
            h.update(b"skill-general-slot/v1");
            put_u8(&mut h, s.general_slot);
        }
    }

    if !state.damage_history.is_empty() {
        h.update(b"damage-history/v1");
        put_u32(&mut h, state.damage_history.len() as u32);
        for (source, target) in &state.damage_history {
            put_u32(&mut h, source.raw());
            put_u32(&mut h, target.raw());
        }
    }
    if !state.damage_dealt_totals.is_empty() {
        h.update(b"damage-dealt-totals/v1");
        put_u32(&mut h, state.damage_dealt_totals.len() as u32);
        for (source, amount) in &state.damage_dealt_totals {
            put_u32(&mut h, source.raw());
            put_u32(&mut h, *amount);
        }
    }
    if !state.card_action_totals.is_empty() {
        h.update(b"card-action-totals/v1");
        put_u32(&mut h, state.card_action_totals.len() as u32);
        for (player, count) in &state.card_action_totals {
            put_u32(&mut h, player.raw());
            put_u32(&mut h, *count);
        }
    }
    if state.lifetime_basic_card_uses != 0 {
        put_u8(&mut h, 205);
        put_u32(&mut h, state.lifetime_basic_card_uses);
    }
    if state.lifetime_used_type_groups != 0 {
        put_u8(&mut h, 206);
        put_u8(&mut h, state.lifetime_used_type_groups);
    }
    if state.draw_reshuffle_count != 0 {
        h.update(b"draw-reshuffle-count/v1");
        put_u32(&mut h, state.draw_reshuffle_count);
    }
    if !state.skill_piles.is_empty() {
        h.update(b"skill-piles/v1");
        put_u32(&mut h, state.skill_piles.len() as u32);
        for pile in &state.skill_piles {
            put_u32(&mut h, pile.owner.raw());
            put_u32(&mut h, pile.skill_def);
            put_u32(&mut h, pile.zone.raw());
            put_u8(&mut h, u8::from(pile.usable_as_hand));
        }
    }

    // Resolution (canonical continuation + budgets config)
    let r = &state.resolution;
    put_u32(&mut h, r.pending_choice_id);
    put_u32(&mut h, r.next_choice_seq);
    if r.next_damage_seq != 0 || r.current_damage_id != 0 || !r.prevented_damage_ids.is_empty() {
        h.update(b"damage-before/v1");
        put_u32(&mut h, r.next_damage_seq);
        put_u32(&mut h, r.current_damage_id);
        put_ids(&mut h, r.prevented_damage_ids.iter().copied());
    }
    if r.next_recovery_seq != 0
        || r.current_recovery_id != 0
        || !r.prevented_recovery_ids.is_empty()
    {
        h.update(b"recover-before/v1");
        put_u32(&mut h, r.next_recovery_seq);
        put_u32(&mut h, r.current_recovery_id);
        put_ids(&mut h, r.prevented_recovery_ids.iter().copied());
    }
    if r.damage_redirect_in_progress {
        h.update(b"damage-redirect-in-progress/v1");
    }
    if r.current_hand_compare_player != 0
        || r.current_hand_compare_target != 0
        || r.current_hand_compare_random_mask != 0
    {
        h.update(b"hand-compare-before/v1");
        put_u32(&mut h, r.current_hand_compare_player);
        put_u32(&mut h, r.current_hand_compare_target);
        put_u8(&mut h, r.current_hand_compare_random_mask);
    }
    put_u64(&mut h, r.continuation_id);
    put_u32(&mut h, r.pending_observer);
    put_u64(&mut h, r.choice_revision);
    put_u8(&mut h, r.continuation_kind.tag() as u8);
    put_u32(&mut h, r.continuation_payload.len() as u32);
    for p in &r.continuation_payload {
        put_u32(&mut h, *p);
    }
    put_u32(&mut h, r.effect_budget_max);
    put_u32(&mut h, r.effect_budget);
    put_u32(&mut h, r.trigger_budget_max);
    put_u32(&mut h, r.trigger_budget);
    if !r.sha_no_damage_draw_skills.is_empty() {
        h.update(b"sha-no-damage-draw/v1");
        put_u32(&mut h, r.sha_no_damage_draw_skills.len() as u32);
        for (card, skill) in &r.sha_no_damage_draw_skills {
            put_u32(&mut h, card.raw());
            put_u32(&mut h, skill.raw());
        }
    }
    if !r.sha_resolution_records.is_empty() {
        h.update(b"sha-resolution/v1");
        put_u32(&mut h, r.sha_resolution_records.len() as u32);
        for record in &r.sha_resolution_records {
            put_u32(&mut h, record.source.raw());
            put_u32(&mut h, record.card.raw());
            put_ids(&mut h, record.targets.iter().map(|player| player.raw()));
            if record.use_counted || record.use_refunded {
                h.update(b"use-count/v1");
                put_bool(&mut h, record.use_counted);
                put_bool(&mut h, record.use_refunded);
            }
            if record.damage_source_none {
                h.update(b"damage-source-none/v1");
            }
            if !record.source_cards.is_empty() {
                h.update(b"source-cards/v1");
                put_ids(&mut h, record.source_cards.iter().map(|card| card.raw()));
            }
            put_ids(&mut h, record.response_cards.iter().map(|card| card.raw()));
            if !record.skill_targets.is_empty() {
                h.update(b"skill-targets/v1");
                put_u32(&mut h, record.skill_targets.len() as u32);
                for (skill, target) in &record.skill_targets {
                    put_u32(&mut h, skill.raw());
                    put_u32(&mut h, target.raw());
                }
            }
            if !record.backwater_targets.is_empty() {
                h.update(b"backwater-targets/v1");
                put_u32(&mut h, record.backwater_targets.len() as u32);
                for (skill, target) in &record.backwater_targets {
                    put_u32(&mut h, skill.raw());
                    put_u32(&mut h, target.raw());
                }
            }
            if !record.dying_targets.is_empty() {
                h.update(b"sha-dying-targets/v1");
                put_ids(
                    &mut h,
                    record.dying_targets.iter().map(|target| target.raw()),
                );
            }
        }
    }
    if !r.duel_damage_cards.is_empty() {
        h.update(b"duel-damage/v1");
        put_ids(&mut h, r.duel_damage_cards.iter().map(|card| card.raw()));
    }
    if !r.pending_card_resolutions.is_empty() {
        h.update(b"card-resolution/v4");
        put_u32(&mut h, r.pending_card_resolutions.len() as u32);
        for record in &r.pending_card_resolutions {
            put_u32(&mut h, record.source.raw());
            put_u32(&mut h, record.card.raw());
            put_u8(&mut h, record.kind);
            put_ids(&mut h, record.targets.iter().map(|player| player.raw()));
            put_ids(
                &mut h,
                record.damaged_targets.iter().map(|player| player.raw()),
            );
            put_bool(&mut h, record.source_was_hand);
            put_bool(&mut h, record.all_sources_were_hand);
            put_u8(&mut h, record.response_color_lock_mask);
            put_ids(&mut h, record.source_cards.iter().map(|card| card.raw()));
            put_u32(&mut h, record.originating_view_as_skill_def);
            put_ids(
                &mut h,
                record
                    .skill_excluded_targets
                    .iter()
                    .map(|player| player.raw()),
            );
        }
    }
    if !r.planned_card_source_cards.is_empty() {
        h.update(b"planned-card-sources/v2");
        put_u32(&mut h, r.planned_card_source_cards.len() as u32);
        for (source, card, cards, all_sources_were_hand) in &r.planned_card_source_cards {
            put_u32(&mut h, source.raw());
            put_u32(&mut h, card.raw());
            put_ids(&mut h, cards.iter().map(|card| card.raw()));
            put_bool(&mut h, *all_sources_were_hand);
        }
    }
    // VM continuation (canonical: pending DSL choose resume state)
    put_u32(&mut h, r.vm_cont_frames.len() as u32);
    h.update(&r.vm_cont_frames);
    h.update(r.vm_cont_module.as_bytes());
    put_u32(&mut h, r.vm_cont_pc);
    put_u32(&mut h, r.vm_cont_fuel);
    for c in &r.vm_cont_ctx {
        put_u32(&mut h, *c);
    }
    if r.vm_cont_skill != 0 {
        h.update(b"vm-skill");
        put_u32(&mut h, r.vm_cont_skill);
    }
    if r.vm_cont_trigger_marker != [0; 3] {
        h.update(b"vm-trigger-marker");
        for value in r.vm_cont_trigger_marker {
            put_u32(&mut h, value);
        }
    }
    if !r.deferred_effects.is_empty()
        || !r.deferred_tail_effects.is_empty()
        || !r.deferred_triggers.is_empty()
        || r.optional_trigger.is_some()
        || !r.pending_card_replays.is_empty()
    {
        h.update(b"deferred-resolution/v4");
        put_u32(&mut h, r.deferred_effects.len() as u32);
        for effect in &r.deferred_effects {
            hash_effect(&mut h, effect);
        }
        put_u32(&mut h, r.deferred_tail_effects.len() as u32);
        for effect in &r.deferred_tail_effects {
            hash_effect(&mut h, effect);
        }
        put_u32(&mut h, r.deferred_triggers.len() as u32);
        for task in &r.deferred_triggers {
            hash_pending_trigger(&mut h, task);
        }
        put_bool(&mut h, r.optional_trigger.is_some());
        if let Some(task) = &r.optional_trigger {
            hash_pending_trigger(&mut h, task);
        }
        put_u32(&mut h, r.pending_card_replays.len() as u32);
        for replay in &r.pending_card_replays {
            put_u32(&mut h, replay.source_card.raw());
            put_u32(&mut h, replay.player.raw());
            put_u8(&mut h, replay.kind);
            put_ids(&mut h, replay.targets.iter().map(|target| target.raw()));
            put_u32(&mut h, replay.skill.raw());
        }
    }

    // RNG
    put_u64(&mut h, state.rng.state);
    put_u64(&mut h, state.rng.call_count);

    // Outcome
    hash_outcome(&mut h, &state.outcome);

    *h.finalize().as_bytes()
}

fn hash_pending_trigger(h: &mut blake3::Hasher, task: &crate::trigger::PendingTrigger) {
    put_u32(h, task.active.skill_id.raw());
    put_u32(h, task.active.skill_def.raw());
    put_u32(h, task.active.owner.raw());
    put_u32(h, task.active.trigger_def.raw());
    put_u32(h, task.active.event.tag());
    put_u32(h, task.active.timing as u32);
    put_i32(h, task.active.priority);
    put_bool(h, task.active.optional);
    put_bool(h, task.active.enabled);
    put_u32(h, task.context.kind.tag());
    put_u32(h, task.context.player.raw());
    put_u32(h, task.context.source.raw());
    put_u32(h, task.context.respond_to.raw());
    put_u32(h, task.context.amount);
    put_u8(h, task.context.nature);
    put_u32(h, task.context.turn_damage_sources);
    put_u32(h, task.context.turn_damaged_players);
    put_u8(h, card_move_reason_tag(task.context.move_reason));
    put_bool(h, task.context.moved_has_basic);
    put_u8(h, task.context.from_zone.map(zone_kind_tag).unwrap_or(0));
    put_u32(h, task.context.to_zone.map(|zone| zone.raw()).unwrap_or(0));
    put_u32(h, task.context.card.map(|card| card.raw()).unwrap_or(0));
    put_u8(h, task.context.card_kind.unwrap_or(0));
    put_u8(h, task.context.card_flags);
    put_u32(h, task.context.target_count);
    put_ids(h, task.context.targets.iter().map(|player| player.raw()));
    put_ids(h, task.context.cards.iter().map(|card| card.raw()));
    put_u32(h, task.view_as_skill);
    put_u32(h, task.view_as_card);
    put_bool(h, task.history.is_some());
    if let Some(history) = &task.history {
        put_u32(h, history.previous_card_meta);
        put_u8(h, history.previous_card_kind);
        put_u32(h, history.previous_card_def);
        put_u32(h, history.previous_play_card_meta);
        put_u32(h, history.previous_game_card_meta);
        put_u32(h, history.owner_sha_used);
        put_u32(h, history.owner_juedou_used);
        put_i32(h, history.source_card_used);
        put_i32(h, history.owner_card_action_count);
        put_i32(h, history.owner_play_phase_card_used);
        put_u32(h, history.event_player_play_phase_card_used);
        put_u32(h, history.event_player_play_phase_suit_count);
        put_u32(h, history.view_as_skill_def);
        put_u32(h, history.view_as_skill_owner);
        put_bool(h, history.event_player_had_other_red_card_target);
    }
}

fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::DrawCards { player, count } => {
            put_u8(h, 1);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *delta as u32);
            put_u32(h, skill.raw());
        }
        EffectRequest::MoveCards { cards, to, from } => {
            put_u8(h, 2);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.map(|id| id.raw()).unwrap_or(0));
        }
        EffectRequest::MoveCardsWithReason {
            cards,
            to,
            from,
            reason,
        } => {
            put_u8(h, 0);
            h.update(&385u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.map(|id| id.raw()).unwrap_or(0));
            put_u8(h, card_move_reason_tag(*reason));
        }
        EffectRequest::UseCard {
            player,
            card,
            targets,
        } => {
            put_u8(h, 3);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
        }
        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => {
            put_u8(h, 4);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
        }
        EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } => {
            put_u8(h, 5);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }
        EffectRequest::ChangeMaxHp { player, delta } => {
            put_u8(h, 6);
            put_u32(h, player.raw());
            put_i32(h, *delta);
        }
        EffectRequest::AddSkill { player, skill_def } => {
            put_u8(h, 7);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }
        EffectRequest::RemoveSkill { skill } => {
            put_u8(h, 8);
            put_u32(h, skill.raw());
        }
        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => {
            put_u8(h, 9);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_i32(h, *delta);
        }
        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => {
            put_u8(h, 10);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *amount);
        }
        EffectRequest::RevealCards { cards, to } => {
            put_u8(h, 11);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_ids(h, to.iter().map(|id| id.raw()));
        }
        EffectRequest::RequestChoice {
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
        } => {
            put_u8(h, 12);
            put_u32(h, observer.raw());
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_bool(h, *allow_cancel);
            put_ids(h, option_tokens.iter().copied());
        }
        EffectRequest::EnterDying { player } => {
            put_u8(h, 13);
            put_u32(h, player.raw());
        }
        EffectRequest::KillPlayer { player, killer } => {
            put_u8(h, 14);
            put_u32(h, player.raw());
            put_u32(h, killer.raw());
        }
        EffectRequest::SetTurnState { current, phase } => {
            put_u8(h, 15);
            put_u32(h, current.raw());
            put_u8(h, *phase);
        }
        EffectRequest::EmitDebateResolved { .. } => {
            // Engine-internal event emission; no content hash contribution.
        }
        EffectRequest::EmitRuleEvent { code, payload } => {
            put_u8(h, 16);
            put_u32(h, *code);
            put_ids(h, payload.iter().copied());
        }
        EffectRequest::DiscardFromHand { player, count } => {
            put_u8(h, 17);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::DiscardFromHandEquip { player, count } => {
            put_u8(h, 0);
            h.update(&359u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::TakeFromHand { from, to } => {
            put_u8(h, 18);
            put_u32(h, from.raw());
            put_u32(h, to.raw());
        }
        EffectRequest::DiscardCards { player, cards } => {
            put_u8(h, 19);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }
        EffectRequest::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => {
            put_u8(h, 20);
            put_u32(h, player.raw());
            put_u32(h, respond_to.raw());
            put_u32(h, card.raw());
            put_u8(h, *response_kind);
        }
        EffectRequest::RemoveCardsByKind { kind } => {
            put_u8(h, 21);
            put_u8(h, *kind);
        }
        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => {
            put_u8(h, 22);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u8(h, *card_flags);
        }
        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            put_u8(h, 23);
            put_u32(h, player.raw());
        }
        EffectRequest::JudgeGainEquipmentByColor { player } => {
            put_u8(h, 24);
            put_u32(h, player.raw());
        }
        EffectRequest::RevealDiscardSuitGainOrUse {
            player,
            mark,
            base_count,
            max_mark,
            revealed,
            completed,
            step,
            data,
        } => {
            put_u8(h, 25);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u8(h, *base_count);
            put_u8(h, *max_mark);
            put_ids(h, revealed.iter().map(|card| card.raw()));
            put_u8(h, *completed);
            put_u8(h, *step);
            put_u32(h, *data);
        }
        EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        } => {
            put_u8(h, 26);
            put_u32(h, player.raw());
            put_u8(h, *max_discard);
            put_u8(h, *damage);
            put_u8(h, *selected);
            put_u8(h, *step);
            put_u32(h, target.raw());
        }
        EffectRequest::DrawCardsFromSkill {
            player,
            count,
            skill,
        } => {
            put_u8(h, 27);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardAllHandAndEquip { player } => {
            put_u8(h, 28);
            put_u32(h, player.raw());
        }
        EffectRequest::RequestDyingSave { player, source } => {
            put_u8(h, 29);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
        }
        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            skill,
        } => {
            put_u8(h, 30);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill,
            drawn,
            ready_to_give,
        } => {
            put_u8(h, 31);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_bool(h, *drawn_only);
            put_bool(h, *use_given_equipment);
            put_u32(h, skill.raw());
            put_ids(h, drawn.iter().map(|card| card.raw()));
            put_bool(h, *ready_to_give);
        }
        EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count,
            skill,
            ready_to_take,
        } => {
            put_u8(h, 32);
            put_u32(h, player.raw());
            put_u32(h, taker.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_take);
        }
        EffectRequest::TakeOneCard {
            from,
            to,
            equip_only,
        } => {
            put_u8(h, 33);
            put_u32(h, from.raw());
            put_u32(h, to.raw());
            put_bool(h, *equip_only);
        }
        EffectRequest::TurnOver { player } => {
            put_u8(h, 34);
            put_u32(h, player.raw());
        }
        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            put_u8(h, 35);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }
        EffectRequest::ChooseDiscardHandAddSkillByColor {
            player,
            red_skill_def,
            black_skill_def,
        } => {
            put_u8(h, 36);
            put_u32(h, player.raw());
            put_u32(h, red_skill_def.raw());
            put_u32(h, black_skill_def.raw());
        }
        EffectRequest::RevealTopGainNonBasicOrSwapBasic { player } => {
            put_u8(h, 37);
            put_u32(h, player.raw());
        }
        EffectRequest::PreventDamage { damage_id } => {
            put_u8(h, 38);
            put_u32(h, *damage_id);
        }
        EffectRequest::CommitDamage {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            put_u8(h, 39);
            put_u32(h, *damage_id);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, *base_amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
        }
        EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        } => {
            put_u8(h, 40);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => {
            put_u8(h, 41);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *recover_count);
            put_bool(h, *allow_cancel);
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
            put_u8(h, 42);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_bool(h, *is_lose_hp);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
        }
        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } => {
            put_u8(h, 43);
            put_u32(h, player.raw());
            put_bool(h, *ignore_distance);
        }
        EffectRequest::ChooseUseVirtualShaWithBonus {
            player,
            ignore_distance,
            damage_bonus,
        } => {
            put_u8(h, 120);
            put_u32(h, player.raw());
            put_bool(h, *ignore_distance);
            put_u32(h, *damage_bonus);
        }
        EffectRequest::AdjustHandTo { player, count } => {
            put_u8(h, 44);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::GainCardByKind { player, kind } => {
            put_u8(h, 45);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }
        EffectRequest::JudgeRecoverOnSuit {
            source,
            player,
            suit,
            amount,
        } => {
            put_u8(h, 46);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
        }
        EffectRequest::GiveCards { cards, to, from } => {
            put_u8(h, 47);
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, to.raw());
            put_u32(h, from.raw());
        }
        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => {
            put_u8(h, 48);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, target.raw());
        }
        EffectRequest::UseFieldCard {
            player,
            card,
            targets,
        } => {
            put_u8(h, 0);
            h.update(&438u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
        }
        EffectRequest::EmitDuelResolved {
            card,
            initiator,
            target,
            winner,
            loser,
        } => {
            put_u8(h, 49);
            put_u32(h, card.raw());
            put_u32(h, initiator.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_u32(h, loser.raw());
        }
        EffectRequest::JudgeDamageOtherOnSuit {
            player,
            suit,
            amount,
            nature,
        } => {
            put_u8(h, 50);
            put_u32(h, player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
            put_u8(h, *nature);
        }
        EffectRequest::RecastCards { player, cards } => {
            put_u8(h, 51);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }
        EffectRequest::DisableEquipSlots { player, slots } => {
            put_u8(h, 52);
            put_u32(h, player.raw());
            put_u32(h, slots.len() as u32);
            for slot in slots {
                put_u8(h, *slot);
            }
        }
        EffectRequest::MoveEquipmentThenDraw { player, skill } => {
            put_u8(h, 53);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseUseVirtualShaOrDraw { player, skill } => {
            put_u8(h, 54);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 55);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardTargetEquipmentGainMount { player, target } => {
            put_u8(h, 56);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => {
            put_u8(h, 57);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }
        EffectRequest::DiscardTargetCards {
            actor,
            target,
            cards,
            scope,
        } => {
            put_u8(h, 58);
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, scope.tag());
        }
        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => {
            put_u8(h, 59);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::ShaDodged {
            source,
            target,
            card,
        } => {
            put_u8(h, 60);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
        }
        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            put_u8(h, 61);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }
        EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id,
        } => {
            put_u8(h, 62);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
        }
        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => {
            put_u8(h, 63);
            put_u32(h, player.raw());
            put_u8(h, *count);
            put_u8(h, suit_tag(*suit));
        }
        EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            skill,
        } => {
            put_u8(h, 64);
            put_u32(h, player.raw());
            put_u32(h, *normal_count);
            put_u32(h, *empty_count);
            put_u32(h, *times);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
            player,
            target,
            slot,
        } => {
            put_u8(h, 65);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *slot);
        }
        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            skill,
        } => {
            put_u8(h, 66);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, skill } => {
            put_u8(h, 67);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => {
            put_u8(h, 68);
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => {
            put_u8(h, 69);
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
            put_u8(h, 70);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::BeginTurn { player } => {
            put_u8(h, 71);
            put_u32(h, player.raw());
        }
        EffectRequest::FinishTurnBegin { player } => {
            put_u8(h, 72);
            put_u32(h, player.raw());
        }
        EffectRequest::AdvanceOpeningExtraPlay { player } => {
            put_u8(h, 73);
            put_u32(h, player.raw());
        }
        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
            player,
            card,
            skill,
        } => {
            put_u8(h, 74);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseAddCurrentShaTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 75);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitAddCurrentShaTarget {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 76);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseStoreHandShaInSkillPile { player, skill } => {
            put_u8(h, 77);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseSpendSkillPileForDyingJiu {
            player,
            dying,
            skill,
        } => {
            put_u8(h, 78);
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 79);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitSpendSkillPileForDyingJiu {
            player,
            dying,
            card,
            skill,
        } => {
            put_u8(h, 80);
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GuanxingTop {
            player,
            count,
            round: _,
            flipped,
            top: _,
            bottom: _,
            skill,
        } => {
            put_u8(h, 132);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::GuanxingTopByFaluMarks {
            player,
            marks,
            skill,
        } => {
            put_u8(h, 134);
            put_u32(h, player.raw());
            for def in marks.iter() {
                put_u32(h, def.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherGiveThenRecover { player, skill } => {
            put_u8(h, 133);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseHandCompareBetween {
            player,
            first,
            skill,
        } => {
            put_u8(h, 131);
            put_u32(h, player.raw());
            put_u32(h, first.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill,
        } => {
            put_u8(h, 130);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            for card in cards.iter() {
                put_u32(h, card.raw());
            }
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 129);
            put_u32(h, player.raw());
            put_u32(h, win_grant_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 81);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => {
            put_u8(h, 82);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, bound_target.raw());
        }
        EffectRequest::ChooseAddSkillBoundCardTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 83);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 84);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            skill,
        } => {
            put_u8(h, 85);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::AddCardColorLockSkill {
            player,
            source,
            color_mask,
            skill_def,
        } => {
            put_u8(h, 86);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *color_mask);
            put_u32(h, skill_def.raw());
        }
        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            put_u8(h, 87);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 88);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => {
            put_u8(h, 89);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *policy_mask);
            put_u32(h, skill_def.raw());
        }
        EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        } => {
            put_u8(h, 90);
            put_u32(h, player.raw());
            put_u32(h, draw_skill_def.raw());
            put_u32(h, discard_skill_def.raw());
            put_u32(h, skill.raw());
            if *three_way {
                put_u8(h, 1);
            }
        }
        EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 91);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 92);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::UseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 93);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 94);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            toggle_link,
            skill,
        } => {
            put_u8(h, 95);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            if *toggle_link {
                put_u8(h, 1);
            }
        }
        EffectRequest::FinishLinkIfTargetWounded {
            player,
            target,
            toggle_link,
            skill,
        } => {
            put_u8(h, 96);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            if *toggle_link {
                put_u8(h, 1);
            }
        }
        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            put_u8(h, 97);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill,
            ready_to_give,
        } => {
            put_u8(h, 98);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_give);
        }
        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            put_u8(h, 99);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::EmitShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => {
            put_u8(h, 100);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, response_cards.iter().map(|card| card.raw()));
            put_u8(h, *card_flags);
        }
        EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => {
            put_u8(h, 101);
            put_u32(h, player.raw());
            put_u32(h, sha_card.raw());
            put_ids(h, shan_cards.iter().map(|card| card.raw()));
            put_ids(h, excluded_recipients.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseUseShaAfterResolvedGift {
            owner,
            player,
            skill,
        } => {
            put_u8(h, 102);
            put_u32(h, owner.raw());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::SkipCurrentPhase { player } => {
            put_u8(h, 103);
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 104);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, skill } => {
            put_u8(h, 105);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCardUserOrAddTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 106);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitTransferSavedCardUser {
            previous,
            new_user,
            card,
            skill,
        } => {
            put_u8(h, 107);
            put_u32(h, previous.raw());
            put_u32(h, new_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id,
            skill,
        } => {
            put_u8(h, 108);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&434u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player,
            target,
            skill,
        } => {
            put_u8(h, 109);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 110);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
            owner,
            recipient,
            count,
            skill,
        } => {
            put_u8(h, 111);
            put_u32(h, owner.raw());
            put_u32(h, recipient.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 112);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, skill } => {
            put_u8(h, 113);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            put_u8(h, 114);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, skill } => {
            put_u8(h, 115);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 116);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 117);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            put_u8(h, 118);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id,
            amount,
            skill,
        } => {
            put_u8(h, 119);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            skill,
        } => {
            put_u8(h, 120);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 121);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::RandomizeCurrentHandCompareOpponent { player, skill } => {
            put_u8(h, 122);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::EmitHandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => {
            put_u8(h, 123);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, winner.raw());
        }
        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 124);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => {
            put_u8(h, 125);
            put_u32(h, player.raw());
            put_u32(h, initiator.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 126);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 127);
            put_u32(h, player.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 128);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherDiscardTargetCard {
            player,
            scope,
            skill,
        } => {
            put_u8(h, 129);
            put_u32(h, player.raw());
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainDiscardCardByKind { player, kind } => {
            put_u8(h, 130);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }
        EffectRequest::RevealUniformHandChooseDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 131);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainDiscardCardByKind {
            player,
            kind,
            skill,
        } => {
            put_u8(h, 132);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            put_u8(h, 133);
            put_u32(h, player.raw());
            for divisor in divisors {
                put_u32(h, *divisor);
            }
        }
        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 134);
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => {
            put_u8(h, 135);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, *target_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainEventCards { player, cards } => {
            put_u8(h, 136);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            put_u8(h, 137);
            put_u32(h, player.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            put_u8(h, 138);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::EmitJudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        } => {
            put_u8(h, 139);
            put_u32(h, player.raw());
            put_u32(h, result_card.raw());
            put_u8(h, suit_tag(*suit));
            put_u8(h, *rank);
            put_u32(h, cause_card.raw());
        }
        EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit,
            amount,
        } => {
            put_u8(h, 140);
            put_u32(h, source.raw());
            put_u32(h, judge_player.raw());
            put_u32(h, recover_player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
        }
        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => {
            put_u8(h, 141);
            put_u32(h, player.raw());
            put_ids(h, remaining.iter().map(|player| player.raw()));
            put_bool(h, *started);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => {
            put_u8(h, 142);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_ids(h, responders.iter().map(|player| player.raw()));
        }
        EffectRequest::RestoreCharacterState { player } => {
            put_u8(h, 143);
            put_u32(h, player.raw());
        }
        EffectRequest::DiscardAllHandEquipJudge { player } => {
            put_u8(h, 144);
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => {
            put_u8(h, 145);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            put_u8(h, 146);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
        }
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 147);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&363u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&364u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::SpwanweiActive { player, target } => {
            put_u8(h, 0);
            h.update(&365u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::SpwanweiDying { player, target } => {
            put_u8(h, 0);
            h.update(&366u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            put_u8(h, 0);
            h.update(&367u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            put_u8(h, 0);
            h.update(&368u16.to_le_bytes());
            put_u32(h, chooser.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
        }
        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&369u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&370u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&371u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, *damage_bonus);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&372u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, fangzong_skill_def.raw());
        }
        EffectRequest::ResolveSpsongshu {
            player,
            phase_player,
            renku_skill_def,
            block_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&384u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, renku_skill_def.raw());
            put_u32(h, block_skill_def.raw());
        }
        EffectRequest::ResolveSpcunsi { player, target } => {
            put_u8(h, 0);
            h.update(&390u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::ResolveYuli {
            player,
            mode,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTingwei {
            player,
            mode,
            card,
            targets,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinmingshi {
            player,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXunyi {
            player,
            mode,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBoming {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveEjian {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&414u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveYiyong {
            player,
            card,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShanxie { player, skill } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSheyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTianyin { player, skill } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDbzhuifeng { player, skill } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDbquedi {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpyinju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&421u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpchijie {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpshangyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpjianyi { player, skill } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveFyjianyu {
            player,
            source,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&425u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLiubing {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&426u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHeji {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpyajun {
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&428u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, less_def.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpzundi {
            player,
            target,
            card,
            destination,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, destination.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinyangjie {
            player,
            target,
            card,
            target_card,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, source.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQingyu {
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&431u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, xuancun_def.raw());
            put_u32(h, mbyongjue_def.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&433u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbdiancai {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&434u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRechuhai {
            player,
            target,
            card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXiongshu {
            player,
            target,
            card,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJianhui {
            player,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpqiai {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTongduo {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveYuanqing { player, skill } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShuchen {
            player,
            dying,
            renku_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, renku_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&445u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
            for target in targets {
                put_u32(h, target.raw());
            }
        }
        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&446u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGuying {
            player,
            actor,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&447u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRechuhai { .. } => {
            // 其他 agent 在建批次：落盘前暂以桩哈希。
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
        }
        EffectRequest::ResolveXiongshu { .. } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
        }
        EffectRequest::ResolveSpshanxi { .. } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
        }
        EffectRequest::ResolveSpshanxiSuoming { .. } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
        }
        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&545u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, *amount);
            put_u32(h, mbsizi_def.raw());
            put_u32(h, mbkechang_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, used_def.raw());
            for def in suit_defs {
                put_u32(h, def.raw());
            }
            put_u32(h, qingshi_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotqingshi { player, skill } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&546u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&547u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u8(h, *card_flags);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&548u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, event_cards.len() as u32);
            for card in event_cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&549u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&550u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&551u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, hand_limit_def.raw());
            put_u32(h, draw_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTuntian { player, skill } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jixi_def.raw());
            put_u32(h, tuntian_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, guanxing_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZaiqi {
            player,
            extra,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *extra);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwqiushou { player, skill } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, candidates.iter().map(|candidate| candidate.raw()));
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qingjian_def.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&513u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHaoshi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&514u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&515u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *phase);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&518u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveFangquanSkip { player, skill } => {
            put_u8(h, 0);
            h.update(&519u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&520u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&521u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jijiang_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZuilun {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&522u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&523u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::BanUseAgainst { user, target } => {
            put_u8(h, 0);
            h.update(&524u16.to_le_bytes());
            put_u32(h, user.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&525u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&526u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&527u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qizhi_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJianxiang {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&528u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&529u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&531u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, mingren_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&532u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&533u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&534u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&535u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, kongsheng_def.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&536u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&537u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, guixiu_def.raw());
            put_u32(h, yongjue_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&538u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&570u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *payment_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&571u16.to_le_bytes());
            put_u32(h, chooser.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&572u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *remaining);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&573u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&574u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&575u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards,
            split,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&576u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_def.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *split);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&582u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&583u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawCardsFromBottom { player, count } => {
            put_u8(h, 0);
            h.update(&584u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
        }
        EffectRequest::SetDrawPhaseFromBottom { player } => {
            put_u8(h, 0);
            h.update(&587u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::ResolveChenglve {
            player,
            stage,
            suits,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&585u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *suits);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShicai {
            player,
            card,
            placed,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&586u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, placed.iter().map(|card| card.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&588u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcshaowei {
            player,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&591u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&592u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&593u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardHandRange {
            player,
            min,
            max,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&594u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *min);
            put_u32(h, *max);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&595u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&598u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&600u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&601u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&602u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&603u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&604u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, grant_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&605u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, *remaining);
            put_u32(h, *used_mask);
            put_u8(h, u8::from(*repeated));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&606u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            source,
            target,
            nature,
            damage_id,
            event_cards,
            event_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&607u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u8(h, *nature);
            put_u32(h, *damage_id);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&608u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, recovered_player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&609u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&610u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, zhucheng_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbLiuyan {
            player,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&611u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&612u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&613u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueYoudishenru {
            action,
            protected,
            source,
            forced_card,
            saved,
        } => {
            put_u8(h, 1);
            h.update(&613u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, protected.raw());
            put_u32(h, source.raw());
            put_u32(h, forced_card.raw());
            put_ids(h, saved.iter().copied());
        }
        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&614u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueWdbFuchou {
            action,
            player,
            target,
            remaining,
            skill,
        } => {
            put_u8(h, 1);
            h.update(&614u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, remaining.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&615u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueDiaobingqianjiang {
            action,
            actor,
            source_card,
            targets,
            revealed,
        } => {
            put_u8(h, 1);
            h.update(&615u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, actor.raw());
            put_u32(h, source_card.raw());
            put_ids(h, targets.iter().map(|player| player.raw()));
            put_ids(h, revealed.iter().map(|card| card.raw()));
        }
        EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&616u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&617u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&618u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, discard_target.raw());
            put_u32(h, damage_target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&619u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&621u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&622u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&623u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&624u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&625u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&626u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddYujin {
            player,
            actor,
            target,
            move_mask,
            action,
            candidates,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&627u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
            put_u8(h, *move_mask);
            put_u8(h, *action);
            put_ids(h, candidates.iter().map(|candidate| candidate.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&628u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&629u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *used_kind);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&630u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, piles.iter().map(|pile| u32::from(*pile)));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&631u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *kind);
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&632u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&633u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&634u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&635u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::PreventRecovery { recovery_id } => {
            put_u8(h, 1);
            h.update(&635u16.to_le_bytes());
            put_u32(h, *recovery_id);
        }
        EffectRequest::CommitRecovery {
            recovery_id,
            source,
            player,
            amount,
        } => {
            put_u8(h, 2);
            h.update(&635u16.to_le_bytes());
            put_u32(h, *recovery_id);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }
        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&636u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&637u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddXiahouxuan {
            player,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&638u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards,
            amount,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&639u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDddLiangxi {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&640u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::GuozhanUseZhulianBihe { player } => {
            // ABI 641: VM-driven guozhan 珠联璧合 spend-as-无中生有 primitive.
            put_u8(h, 0);
            h.update(&641u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::ResolveShoulijian {
            player,
            target,
            skill,
        } => {
            // ABI 642: 欢乐卡牌手里剑 custom-basic-card settlement.
            put_u8(h, 0);
            h.update(&642u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => {
            // ABI 643: 取消使用牌 (gwent 杜达元帅1 special-card cancel-use primitive).
            put_u8(h, 0);
            h.update(&643u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherGainOneOfTwoSkills {
            player,
            target,
            first_def,
            second_def,
            skill,
        } => {
            // ABI 644: 择一加技能 (gwent 呼朋 choose-one-of-two-skill grant primitive).
            put_u8(h, 0);
            h.update(&644u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, first_def.raw());
            put_u32(h, second_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGwyewuChain {
            player,
            skill,
            cost_cards,
            wuyao_def,
            lang_def,
        } => {
            // ABI 645: 叶舞 (gwent) random-discard chain + 雾妖/狼 skill gains.
            put_u8(h, 0);
            h.update(&645u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, cost_cards.len() as u32);
            for card in cost_cards {
                put_u32(h, card.raw());
            }
            put_u32(h, wuyao_def.raw());
            put_u32(h, lang_def.raw());
        }
        EffectRequest::ContinueWugu {
            action,
            actor,
            source_card,
            targets,
            excluded,
            revealed,
        } => {
            put_u8(h, 1);
            h.update(&616u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, actor.raw());
            put_u32(h, source_card.raw());
            put_ids(h, targets.iter().map(|player| player.raw()));
            put_ids(h, excluded.iter().map(|player| player.raw()));
            put_ids(h, revealed.iter().map(|card| card.raw()));
        }
        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&589u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&591u16.to_le_bytes());
            put_u32(h, player.raw());
            put_i32(h, *delta);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGzbuquRecover {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&592u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSbbiyue { player, skill } => {
            put_u8(h, 0);
            h.update(&599u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&596u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&597u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&590u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&577u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *amount);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, zaoxian_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&519u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, hanzhan_def.raw());
            put_u32(h, zhanlie_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&373u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_amount);
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&375u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&386u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&387u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u8(h, *category);
            put_u32(h, skill.raw());
        }
        EffectRequest::EquipWeaponFromDrawPile { player } => {
            put_u8(h, 0);
            h.update(&388u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&389u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, winner.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&391u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *phase);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&392u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&393u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&394u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *hand_lock_skill_def);
            put_u32(h, skill.raw());
        }
        EffectRequest::ReplayVirtualCardOnTargets {
            player,
            kind,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&395u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&396u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&397u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&398u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, draw_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&399u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *wangxing_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&400u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&401u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&402u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u32(h, *record_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u8(h, *card_kind);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwqianju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&403u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&404u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *count);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&405u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, slots.len() as u32);
            h.update(slots);
            put_u8(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&406u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&407u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&408u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *target_count);
            put_u8(h, *action);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, remaining.iter().map(|target| target.raw()));
            put_bool(h, *got_jiu);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, selected_card.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *selected_skill_def);
            put_u32(h, *twfuzuan_skill_def);
            put_u32(h, *twfeifu_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *mark_def);
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *chunlao_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, selected_card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, limited_skill_def.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card,
            branch,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u8(h, *branch);
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardHandEquipCount {
            player,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mover.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLulve {
            player,
            target,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDctingxian {
            player,
            card,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, grant_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcnuchen {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcdanji { player, skill } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDczhiheng { player, skill } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveStdgaobian {
            player,
            chooser,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, chooser.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpjiedao { player, skill } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSibian { player, skill } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::SkipNextDiscardPhase { player } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::ResolveSpfengyin {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinfuLianpian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGongjian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&450u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *reveal_count);
            put_u8(h, *recipient_cap);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, skill } => {
            put_u8(h, 0);
            h.update(&451u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRefuhai { player, skill } => {
            put_u8(h, 0);
            h.update(&456u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 0);
            h.update(&461u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u8(h, u8::from(*ready_to_choose));
        }
        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&464u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, cost_card.raw());
            put_u32(h, given_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&466u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinkuangfu {
            player,
            target,
            sha_target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&467u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, sha_target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcxiangshu {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&469u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcxiangshuLottery {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&470u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcyingtu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&471u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&468u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&472u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&473u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&474u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started,
            remaining,
            accum,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&475u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_bool(h, *started);
            put_u32(h, remaining.len() as u32);
            for participant in remaining {
                put_u32(h, participant.raw());
            }
            put_u32(h, accum.len() as u32);
            for card in accum {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => {
            put_u8(h, 0);
            h.update(&476u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_ids(h, source_cards.iter().map(|card| card.raw()));
            put_bool(h, *all_sources_were_hand);
        }
        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&477u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_bool(h, *damaged);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&478u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, hand_limit_def.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage,
            roles,
            suits,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&479u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_ids(h, roles.iter().map(|target| target.raw()));
            put_u32(h, suits.len() as u32);
            for suit in suits {
                put_u8(h, *suit);
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&480u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbrenxing {
            player,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&481u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbchizhang {
            player,
            card,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&482u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&483u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&484u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *value);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbshuanghuai {
            player,
            target,
            stage,
            choice,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&485u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&486u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&487u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&488u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbjianji {
            player,
            source,
            target,
            stage,
            card,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&489u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbyuanmo {
            player,
            target,
            stage,
            card,
            value,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&490u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, *value);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage,
            targets,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&491u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&492u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShuangxiong { player, skill } => {
            put_u8(h, 0);
            h.update(&493u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&494u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&495u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().copied());
            put_u32(h, return_card.raw());
            put_u8(h, u8::from(*take_rest));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhiba {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, winner.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltCongjian {
            player,
            targets,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltYongsi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbxuye {
            player,
            target,
            stage,
            was_max,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, u8::from(*was_max));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotjugu { player, skill } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *choice);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotpoxi {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, range_helper_def.raw());
            put_u32(h, yinxian_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, qice_def.raw());
            put_u32(h, kanpo_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *index);
            put_u8(h, *declined);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&513u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePotdimeng {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&514u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&515u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpmiaolve {
            player,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            card_kind,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_def.raw());
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMantianguohai {
            player,
            stage,
            payload,
        } => {
            put_u8(h, 0);
            h.update(&518u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, payload.iter().copied());
        }
        EffectRequest::ResolveXinfuQinguo {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&445u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcmanzhi { player, skill } => {
            put_u8(h, 0);
            h.update(&446u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinxingluan { player, skill } => {
            put_u8(h, 0);
            h.update(&447u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDckuizhen { player, skill } => {
            put_u8(h, 0);
            h.update(&448u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDckuizhenResolve {
            player,
            winner,
            loser,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&449u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, winner.raw());
            put_u32(h, loser.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNsdingzhou { player, skill } => {
            put_u8(h, 0);
            h.update(&454u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSxrmlanjiao {
            player,
            target,
            stage,
            view,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&455u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, view.len() as u32);
            for card in view {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRekuanshi { player, skill } => {
            put_u8(h, 0);
            h.update(&457u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRekuanshiResolve {
            player,
            damaged,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&458u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, damaged.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcmoyu { player, skill } => {
            put_u8(h, 0);
            h.update(&459u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXizhen { player, skill } => {
            put_u8(h, 0);
            h.update(&460u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXuxie { player, skill } => {
            put_u8(h, 0);
            h.update(&462u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&465u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, accum.len() as u32);
            for card in accum {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXuxieAdd { player, skill } => {
            put_u8(h, 0);
            h.update(&463u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&421u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&425u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&426u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&428u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, winner.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&431u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&452u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&453u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *bit);
            put_u8(h, u8::from(*value));
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitDamageBanActiveIfDying {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            nature,
            skill,
        } => {
            put_u8(h, 148);
            put_u32(h, *damage_id);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, *base_amount);
            put_u8(h, *nature);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            skill,
        } => {
            put_u8(h, 149);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            card,
            before,
            skill,
        } => {
            put_u8(h, 150);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *before);
            put_u32(h, skill.raw());
        }
        EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            before,
            skill,
        } => {
            put_u8(h, 151);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *before);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseUseVirtualCard {
            player,
            kind,
            skill,
        } => {
            put_u8(h, 152);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseUseVirtualNanman { player, skill } => {
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQiluan { player, skill } => {
            put_u8(h, 155);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardAllOwnJudge { player, skill } => {
            put_u8(h, 156);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::StoreEventCardInSkillPile {
            player,
            card,
            skill,
        } => {
            put_u8(h, 157);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOllinjie {
            player,
            source,
            skill,
        } => {
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetGainMaxHp { player, skill } => {
            // Reuses digest tag 159 with a distinct 2x u32 payload shape.
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainTargetHandCard { player, skill } => {
            // Reuses digest tag 154 (existing uses are 4x u32) with a distinct
            // 2x u32 payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseClanNotMaxHandLimitGain { player, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 payload shape
            // (existing 154 uses are 4x u32 and 2x u32).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4D55_5949);
        }
        EffectRequest::GainWholeSkillPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt GWHP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4757_5048);
        }
        EffectRequest::ChooseOneFromSkillPileDiscard {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHOD).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4348_5044);
        }
        EffectRequest::PlaceHandCardsToDeckTop {
            player,
            skill,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt PHDT).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *count);
            put_u32(h, 0x5048_4454);
        }
        EffectRequest::RearrangePileToDeckTop {
            player,
            skill,
            pile_def,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, *count);
            put_u32(h, 0x5250_5444);
        }
        EffectRequest::ResolveQianlong { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QLN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x51_4C_4E);
        }
        EffectRequest::ResolveJsrgzuozhan { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt ZUZH).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5A55_5A48);
        }
        EffectRequest::ResolveJsrgzuozhanGain {
            player,
            skill,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt ZUZG).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, 0x5A55_5A47);
        }
        EffectRequest::ResolveTwmouzhi {
            player,
            skill,
            mode,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt MOUZ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *event_card);
            put_u32(h, 0x4D4F_555A);
        }
        EffectRequest::ResolveTwdaigui {
            player,
            skill,
            stage,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt TWDG).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *count);
            put_u32(h, 0x5457_4447);
        }
        EffectRequest::ResolveTwmitu {
            player,
            skill,
            stage,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt TWMT).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *count);
            put_u32(h, 0x5457_4D54);
        }
        EffectRequest::ResolveQiantun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QTN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5147_544E);
        }
        EffectRequest::ResolveYangming { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt YML).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x594D_4C);
        }
        EffectRequest::ResolveCanyun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt CYY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4359_59);
        }
        EffectRequest::ResolveHengwu {
            player,
            skill,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt HENW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *event_card);
            put_u32(h, 0x4845_4E57);
        }
        EffectRequest::ResolveBiaozhao3 { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt BZ3).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x425A_33);
        }
        EffectRequest::ResolveChuanyun {
            player,
            skill,
            event_player,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt CYN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, *event_card);
            put_u32(h, 0x4359_4E);
        }
        EffectRequest::ResolveBiaozhao2 {
            player,
            skill,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt BZ2).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, event_player.raw());
            put_u32(h, 0x425A_32);
        }
        EffectRequest::ResolveDcniji {
            player,
            skill,
            mode,
            event_card,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt NJI).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *event_card);
            put_u32(h, 0x4E4A_49);
        }
        EffectRequest::ResolveDcmoukui {
            player,
            skill,
            stage,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt MKU).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, 0x4D4B_55);
        }
        EffectRequest::ResolveZhongyan { player, skill, stage } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt ZZY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x5A5A_59);
        }
        EffectRequest::ResolveChumei { player, skill, stage } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x4348_4D);
        }
        EffectRequest::ResolveShuaiyan { player, skill, stage } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt SHY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x5348_59);
        }
        EffectRequest::CommitGaoyuanRedirect {
            player,
            source,
            card,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt GY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x4747_59);
        }
        EffectRequest::ResolveTwxiongjun { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt XJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x584A);
        }
        EffectRequest::ResolveYixiang { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt YX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5958);
        }
        EffectRequest::ResolveXinfuQiai { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt FQ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4651);
        }
        EffectRequest::ResolveSpolaige { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt PL).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x504C);
        }
        EffectRequest::ResolveOlzhijie { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4A);
        }
        EffectRequest::ResolveOlsuyi { player, target } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt SY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x5359);
        }
        EffectRequest::ResolveOlqiaozhi { player, stage } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QZ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *stage);
            put_u32(h, 0x515A);
        }
        EffectRequest::ResolveDczhee { player, mode } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt ZE).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mode);
            put_u32(h, 0x5A45);
        }
        EffectRequest::ResolveDchaoxian { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt CX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4358);
        }
        EffectRequest::ResolveZhongjie { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZJ2).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4A);
        }
        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt XW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *variant);
            put_u32(h, 0x5857);
        }
        EffectRequest::ResolveSanshou { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt SS).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5353);
        }
        EffectRequest::ResolveNhlundao { player, source } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt LD).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, 0x4C44);
        }
        EffectRequest::ResolveXiangshu { player, targets } => {
            // Reuses digest tag 154 with a list + 1x u32 + salt payload shape (salt XS).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, 0x5853);
        }
        EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill,
        } => {
            // Digest tag 158-shaped list payload (targets + skill, salt WR).
            put_u8(h, 158);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_u32(h, 0x5752);
        }
        EffectRequest::ResolveDcqiaolve { player, target } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt QL).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x514C);
        }
        EffectRequest::ResolveDingce { player, source } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt DCE).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, 0x4443_45);
        }
        EffectRequest::ResolveQiangyong {
            player,
            target,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt QY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5159);
        }
        EffectRequest::ResolveMeiyong { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt MY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x4D5_9);
        }
        EffectRequest::ResolveMbzhenann { player } => {
            // Reuses digest tag 154 with a distinct 1x u32 + salt payload shape (salt ZNN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, 0x5A4E_4E);
        }
        EffectRequest::ResolveFriendbiwei {
            player,
            target,
            card,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt FBW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4642_57);
        }
        EffectRequest::ResolveChulao { player, targets } => {
            // Reuses digest tag 154 with a distinct ids-list + salt payload shape (salt CLC).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, 0x434C_43);
        }
        EffectRequest::ResolveDcanxu {
            player,
            targets,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct ids-list + salt payload shape (salt DCX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_u32(h, 0x4443_58);
        }
        EffectRequest::ResolveDuoduan { player, source, card, skill } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DDN).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4444_4E);
        }
        EffectRequest::ResolveGaoyuan { player, source, card } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt GY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, 0x4747_59);
        }
        EffectRequest::ResolveMbbeiming { player, skill, stage } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt BBM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, 0x4242_4D);
        }

        EffectRequest::ResolveZhengjian {
            player,
            skill,
            stage,
            event_player,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt ZJZJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, event_player.raw());
            put_u32(h, 0x5A4A_5A4A);
        }
        EffectRequest::ResolveReliangyin { player, skill, mode } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt RLY).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, 0x524C_59);
        }
        EffectRequest::ResolveQinmian {
            player,
            skill,
            mode,
            amount,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt QMM).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *mode);
            put_u32(h, *amount);
            put_u32(h, 0x514D_4D);
        }
        EffectRequest::ResolveJsrgdaoren { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt JSDR).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4A53_4452);
        }
        EffectRequest::ResolveDuwang { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape (salt DUW).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4455_57);
        }
        EffectRequest::ResolveQuanjia {
            player,
            skill,
            source,
            target,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt QJA).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, 0x514A_41);
        }
        EffectRequest::ResolvePxqixian {
            player,
            skill,
            stage,
            kind,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt PXQX).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *kind);
            put_u32(h, 0x5058_5158);
        }
        EffectRequest::ResolveDcbingji {
            player,
            skill,
            stage,
            kind,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DCBJ).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *stage);
            put_u32(h, *kind);
            put_u32(h, 0x4443_424A);
        }
        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            skill,
            min_count,
            max_count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt CHSP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *min_count);
            put_u32(h, *max_count);
            put_u32(h, 0x4348_5350);
        }
        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt CHTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4348_5450);
        }
        EffectRequest::PlaceTopDeckToExpansion {
            player,
            skill,
            pile_def,
            count,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape (salt DPTP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, *count);
            put_u32(h, 0x4450_5450);
        }
        EffectRequest::DiscardSkillPile {
            player,
            skill,
            pile_def,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape (salt DSKP).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *pile_def);
            put_u32(h, 0x4453_4B50);
        }
        EffectRequest::GainFromSkillPile { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x474169_4e);
        }
        EffectRequest::DiscardBasicDrawEqual { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5943_4841);
        }
        EffectRequest::StartDebateAllOthers { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5344_4231);
        }
        EffectRequest::StartDebateTargets { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5344_4232);
        }
        EffectRequest::PinDebateOpinions { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5044_4F33);
        }
        EffectRequest::AlignDebateOpinions { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4144_4F34);
        }
        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4753_4335);
        }
        EffectRequest::ChangeGroup {
            player,
            group,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, group.len() as u32);
            put_u32(h, skill.raw());
            put_u32(h, 0x4347_5236);
        }
        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4453_4337);
        }
        EffectRequest::RegisterTurnEndDebate { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5244_4238);
        }
        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5254_4439);
        }
        EffectRequest::ChooseGroup { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_4750);
        }
        EffectRequest::ChooseGainOtherSkill { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4347_4F53);
        }
        EffectRequest::LoseGainedSkill { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4C47_534B);
        }
        EffectRequest::ResolvePotheyun { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5054_4859);
        }
        EffectRequest::PlaceXumou { player, card, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D31);
        }
        EffectRequest::PlaceDeckTopXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D32);
        }
        EffectRequest::ChoosePlaceHandXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D33);
        }
        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x5058_4D34);
        }
        EffectRequest::PlaceDeckTopExpansion { player, count, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_5431);
        }
        EffectRequest::CityConsume { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4348_5432);
        }
        EffectRequest::QueueExtraTurn { player, check, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *check);
            put_u32(h, skill.raw());
            put_u32(h, 0x4554_5232);
        }
        EffectRequest::ChooseOthersInsertExtraTurn { player, check, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *check);
            put_u32(h, skill.raw());
            put_u32(h, 0x4945_5431);
        }
        EffectRequest::QueueExtraPhases { player, phases_mask, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *phases_mask);
            put_u32(h, skill.raw());
            put_u32(h, 0x5048_4153);
        }
        EffectRequest::ResolveTwjieqiu {
            player,
            target,
            subaction,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *subaction);
            put_u32(h, skill.raw());
            put_u32(h, 0x4A51_4A31);
        }
        EffectRequest::SkipRemainingPhases { player, phases_mask, skill } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *phases_mask);
            put_u32(h, skill.raw());
            put_u32(h, 0x5352_5031);
        }
        EffectRequest::ChooseOthersAddMark {
            player,
            mark,
            delta,
            max_count,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 5x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, *delta as u32);
            put_u32(h, *max_count);
            put_u32(h, skill.raw());
            put_u32(h, 0x4F41_4D31);
        }
        EffectRequest::ResolveJsrgchaozheng { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x435A_5A47);
        }
        EffectRequest::ResolveJsrgsheju {
            player,
            counterpart,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 3x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, counterpart.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x534A_4A55);
        }
        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill,
        } => {
            // Reuses digest tag 154 with a distinct 4x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, *delta as u32);
            put_u32(h, skill.raw());
            put_u32(h, 0x4F41_4D4B);
        }
        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            // Reuses digest tag 154 with a distinct 2x u32 + salt payload shape.
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x504C_4143);
        }
        EffectRequest::ExtraResolveCurrentTrick { player, card, skill } => {
            // Reuses digest tag 154 with a distinct 4x u32 payload shape
            // (existing 154 uses are 4x/2x/3x u32 with different salts/fields).
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u32(h, 0x4558_5452);
        }
        EffectRequest::FinishPhaseEndVirtualCard => {
            put_u8(h, 153);
        }
        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            skill,
        } => {
            put_u8(h, 154);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::SkipNextPlayPhase { player } => {
            put_u8(h, 155);
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill,
        } => {
            put_u8(h, 156);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *recover_count);
            put_u32(h, *times);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => {
            put_u8(h, 157);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill,
        } => {
            put_u8(h, 158);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            put_u8(h, 159);
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }
        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            skill,
        } => {
            put_u8(h, 160);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 161);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 162);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill,
        } => {
            put_u8(h, 163);
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::RecordSelectedPlayPhaseCategory { card, skill } => {
            put_u8(h, 164);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::TakeTargetHandThenRepeatUntilMax {
            player,
            target,
            skill,
        } => {
            put_u8(h, 165);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueTakeLargerHandUntilMax { player, skill } => {
            put_u8(h, 166);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            skill,
        } => {
            put_u8(h, 167);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *damage_amount);
            put_u32(h, mark_def.raw());
            put_u32(h, *mark_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 168);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *payment_count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::GainTargetEquipment {
            player,
            target,
            slot,
        } => {
            put_u8(h, 169);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *slot);
        }
        EffectRequest::DiscardAllTargetEquipment { actor, target } => {
            put_u8(h, 170);
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::IncreaseCurrentShaTargetDamage {
            source,
            card,
            target,
            amount,
        } => {
            put_u8(h, 171);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
        }
        EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            skill,
            ready_to_check,
        } => {
            put_u8(h, 172);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_check);
        }
        EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            skill,
        } => {
            put_u8(h, 173);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill,
            ready_to_discard,
        } => {
            put_u8(h, 174);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_discard);
        }
        EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
            player,
            target,
            cap,
            skill,
            add_def,
        } => {
            put_u8(h, 0);
            h.update(&414u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
            put_u32(h, add_def.raw());
        }
        EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            player,
            target,
            card,
            suit,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 175);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::GainEquipmentFromDrawPile { player } => {
            put_u8(h, 176);
            put_u32(h, player.raw());
        }
        EffectRequest::GiveAllHandRecoverToOne {
            player,
            target,
            ready_to_recover,
        } => {
            put_u8(h, 177);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_bool(h, *ready_to_recover);
        }
        EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
            player,
            target,
            skill,
            ready_to_check,
        } => {
            put_u8(h, 178);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_check);
        }
        EffectRequest::ChooseGiveOneHandEquipOrRecover {
            owner,
            giver,
            target,
            skill,
        } => {
            put_u8(h, 179);
            put_u32(h, owner.raw());
            put_u32(h, giver.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
            player,
            skill,
            skill_def,
            bound_target,
            ready_to_replace,
        } => {
            put_u8(h, 180);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, bound_target.raw());
            put_bool(h, *ready_to_replace);
        }
        EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
            player,
            skill,
            zone,
            discarded_count,
            ready_to_draw,
        } => {
            put_u8(h, 181);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u8(h, *zone);
            put_u32(h, *discarded_count);
            put_bool(h, *ready_to_draw);
        }
        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 182);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill,
        } => {
            put_u8(h, 183);
            put_u32(h, player.raw());
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_u8(h, scope.tag());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        } => {
            put_u8(h, 184);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetRevealHandDamageIfSha {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 185);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherGiveOneHandEquip { player, skill } => {
            put_u8(h, 186);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        } => {
            put_u8(h, 187);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        } => {
            put_u8(h, 188);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, removable_skill_def.raw());
            put_u32(h, restore_skill_def.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::RevealHandDiscardKind { player, kind } => {
            put_u8(h, 189);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }
        EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
            player,
            target,
            skill,
        } => {
            put_u8(h, 190);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 191);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill,
        } => {
            put_u8(h, 192);
            put_u32(h, player.raw());
            put_u32(h, draw_skill_def.raw());
            put_u32(h, recast_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainShaForOpeningExtraPlay { player, skill } => {
            put_u8(h, 193);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::FinishOpeningExtraPlayShaGain { player, skill } => {
            put_u8(h, 194);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            skill,
        } => {
            put_u8(h, 195);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GiveAllHand { player, target } => {
            put_u8(h, 196);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }
        EffectRequest::GainCardIfDiscard { player, card } => {
            put_u8(h, 197);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
        }
        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 198);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::RevealHandToPlayer { player, observer } => {
            put_u8(h, 199);
            put_u32(h, player.raw());
            put_u32(h, observer.raw());
        }
        EffectRequest::ChooseOptionalDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            put_u8(h, 200);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }
        EffectRequest::RevealHandChooseDiscardColor { player } => {
            put_u8(h, 201);
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 202);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseScskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&360u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChoosePskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&361u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 203);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 204);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDrawOrDiscardTargetCard {
            player,
            target,
            draw_count,
            skill,
        } => {
            put_u8(h, 205);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *draw_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 206);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseSeatPathDamage {
            player,
            first,
            second,
            amount,
            skill,
        } => {
            put_u8(h, 207);
            put_u32(h, player.raw());
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::DealDamageEachInOrder {
            source,
            targets,
            amount,
            skill,
        } => {
            put_u8(h, 208);
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::RemoveAllSkills { player } => {
            put_u8(h, 209);
            put_u32(h, player.raw());
        }
        EffectRequest::ChooseOtherRecover { player, amount } => {
            put_u8(h, 210);
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }
        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => {
            put_u8(h, 211);
            put_u32(h, player.raw());
            put_u8(h, *slot);
            put_u32(h, source.raw());
        }
        EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
            player,
            target,
            skill,
        } => {
            put_u8(h, 212);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeSourceDiscardTwoHandOrDamage { player, source } => {
            put_u8(h, 213);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
        }
        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        } => {
            put_u8(h, 214);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
            put_u8(h, u8::from(*ready_to_recover));
        }
        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            put_u8(h, 215);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 216);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, selected.iter().map(|card| card.raw()));
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => {
            put_u8(h, 217);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            if *retieji {
                put_bool(h, true);
                put_u32(h, skill.raw());
                put_bool(h, *ready_to_choose);
                put_u8(h, *judgment_suit);
            }
        }
        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 218);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        } => {
            put_u8(h, 219);
            put_u32(h, owner.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_redirect);
        }
        EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 220);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 221);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card } => {
            put_u8(h, 222);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, card.raw());
        }
        EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            skill,
        } => {
            put_u8(h, 223);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeSetSkillMarkToRank { player, skill } => {
            put_u8(h, 224);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardOneFromEachLinked { player, skill } => {
            put_u8(h, 225);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            put_u8(h, 226);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }
        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => {
            put_u8(h, 227);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            skill,
            stage,
            discarded_basic,
        } => {
            put_u8(h, 228);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_u8(h, *stage);
            put_u8(h, u8::from(*discarded_basic));
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill,
        } => {
            put_u8(h, 229);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        } => {
            put_u8(h, 230);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
            put_u8(h, *stage);
        }
        EffectRequest::DrawRememberedHandCards {
            player,
            count,
            skill,
        } => {
            put_u8(h, 231);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardRememberedHandCards { player, skill } => {
            put_u8(h, 232);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors,
            groups,
        } => {
            put_u8(h, 233);
            put_u32(h, player.raw());
            for divisor in divisors {
                put_u32(h, *divisor);
            }
            for group in groups {
                put_u8(h, *group);
            }
        }
        EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            skill,
        } => {
            put_u8(h, 234);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, mandatory_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, skill } => {
            put_u8(h, 235);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseMoveFieldCardTo {
            player,
            target,
            skill,
        } => {
            put_u8(h, 236);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill,
            ready_to_choose,
            finish,
        } => {
            put_u8(h, 237);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *give_cap);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
            put_bool(h, *finish);
        }
        EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
            player,
            target,
            cards,
            used_targets,
            total_given,
            skill,
            continue_only,
        } => {
            put_u8(h, 238);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, used_targets.iter().map(|target| target.raw()));
            put_u32(h, *total_given);
            put_u32(h, skill.raw());
            put_bool(h, *continue_only);
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            skill,
        } => {
            put_u8(h, 239);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        } => {
            put_u8(h, 240);
            put_u32(h, player.raw());
            put_u8(h, *priority);
            put_ids(h, remaining_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
            put_u8(h, 241);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }
        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 242);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *hand_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&270u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *discard_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&271u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, u8::from(*initialized));
            put_ids(h, remaining_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::RecordActiveTarget {
            player,
            target,
            skill,
        } => {
            put_u8(h, 243);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 244);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mass_threshold);
            put_u32(h, *damage_amount);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 245);
            put_u32(h, player.raw());
            put_u32(h, *damage_amount);
            put_u8(h, *nature);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 246);
            put_u32(h, player.raw());
            put_u8(h, *count);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 247);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill,
        } => {
            put_u8(h, 248);
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *data);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseAllLoseHpOrRecover { player, skill } => {
            put_u8(h, 249);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill,
        } => {
            put_u8(h, 250);
            put_u32(h, player.raw());
            put_u8(h, u8::from(*initialize));
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 251);
            put_u32(h, player.raw());
            put_u32(h, qixing_skill_def.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *max_targets);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 252);
            put_u32(h, player.raw());
            put_u32(h, fire_skill_def.raw());
            put_u32(h, thunder_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseLinkOther { player, skill } => {
            put_u8(h, 253);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        } => {
            put_u8(h, 254);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, hand_limit_skill_def.raw());
            put_u8(h, *stage);
            put_u8(h, *own_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill,
        } => {
            put_u8(h, 255);
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&256u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, selected_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&257u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, event_source.raw());
            put_u32(h, *event_amount);
            put_u8(h, *mode);
            put_u32(h, skill.raw());
        }
        EffectRequest::AdvanceTurn => {
            put_u8(h, 0);
            h.update(&258u16.to_le_bytes());
        }
        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&259u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, *event_amount);
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&260u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, result_card.raw());
            put_u8(h, crate::command::judge::suit_tag(*suit) as u8);
            put_u8(h, *rank);
            put_ids(h, rewriters.iter().map(|player| player.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, skill } => {
            put_u8(h, 0);
            h.update(&261u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardOneHeOrLoseHp {
            player,
            lose_amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&601u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *lose_amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&602u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseSourceConfirmOwnerLoseHp {
            owner,
            source,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&603u16.to_le_bytes());
            put_u32(h, owner.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::DamageThenDiscardAllJudge {
            player,
            ready_to_discard,
        } => {
            put_u8(h, 0);
            h.update(&262u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *ready_to_discard);
        }
        EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
            player,
            first_skill_def,
            second_skill_def,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&263u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, first_skill_def.raw());
            put_u32(h, second_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&264u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::SkipNextJudgePhase { player } => {
            put_u8(h, 0);
            h.update(&265u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
            put_u8(h, 0);
            h.update(&433u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id,
            suit,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&266u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u8(h, *suit);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::TakeOneHandCard { from, to } => {
            put_u8(h, 0);
            h.update(&267u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
        }
        EffectRequest::ChooseOtherHandStoreSkillPile {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&567u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainAllSkillPileCards { player, skill } => {
            put_u8(h, 0);
            h.update(&569u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&600u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *skill_def);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            put_u8(h, 0);
            h.update(&268u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => {
            put_u8(h, 0);
            h.update(&269u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
            put_u32(h, replacement_skill_def.raw());
            put_u8(h, u8::from(*ready_to_replace));
        }
        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&272u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&273u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *enabled);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&274u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::FinalizeTiaojiyanmei {
            actor,
            left,
            right,
            card,
        } => {
            put_u8(h, 0);
            h.update(&275u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, card.raw());
        }
        EffectRequest::BeginTiaojiyanmeiDiscard {
            actor,
            left,
            right,
            higher,
        } => {
            put_u8(h, 0);
            h.update(&276u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, higher.raw());
        }
        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&277u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&278u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u8(h, *branch);
            put_bool(h, *ready_to_discard);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&279u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&280u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGuanchaoRank {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&281u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&282u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&283u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, *index);
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRebingyi { player, skill } => {
            put_u8(h, 0);
            h.update(&284u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRebotu { player, skill } => {
            put_u8(h, 0);
            h.update(&285u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&286u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&287u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_bool(h, *equip);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill,
        } => {
            h.update(&288u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, card.raw());
            put_bool(h, *from_hand);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        } => {
            h.update(&289u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *slot);
            put_bool(h, *first);
            put_u32(h, sha_skill_def.raw());
            put_u32(h, weapon_skill_def.raw());
            put_u32(h, mount_skill_def.raw());
            put_u32(h, armor_skill_def.raw());
            put_u32(h, treasure_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill,
        } => {
            h.update(&290u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        } => {
            h.update(&291u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, *draw_count);
            put_u32(h, *discard_count);
            put_u32(h, funan_helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill,
        } => {
            h.update(&292u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        } => {
            h.update(&293u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, main_skill_def.raw());
            put_u32(h, marker_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => {
            h.update(&294u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *index);
            put_u8(h, *remaining);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveGanglieThree {
            player,
            target,
            skill,
        } => {
            h.update(&295u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill,
        } => {
            h.update(&296u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            h.update(&297u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *slot);
        }
        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill,
        } => {
            h.update(&298u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, giver.raw());
            put_u32(h, receiver.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => {
            h.update(&299u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::EndMatchForCampIfSkillPileAtLeast {
            player,
            pile_skill_def,
            count,
        } => {
            h.update(&300u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u32(h, *count);
        }
        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill,
        } => {
            h.update(&301u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, lord.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&302u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill,
        } => {
            h.update(&303u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJianrou {
            player,
            damage_id,
            skill,
        } => {
            h.update(&304u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id,
            stage,
            cards,
            skill,
        } => {
            h.update(&305u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *damage_id);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            skill,
        } => {
            h.update(&306u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, current.raw());
            put_u32(h, card.raw());
            put_u32(h, *damage_id);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            skill,
        } => {
            h.update(&307u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *branch);
            put_u8(h, *index);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill,
        } => {
            h.update(&308u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *group);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill,
        } => {
            h.update(&309u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => {
            h.update(&310u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *max_distance);
            put_u32(h, skill.raw());
        }
        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill,
        } => {
            h.update(&311u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            h.update(&374u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
        }
        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill,
        } => {
            h.update(&312u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *damage);
            put_u32(h, *draw);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            h.update(&313u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            h.update(&314u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill,
        } => {
            h.update(&315u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
            h.update(&[*stage]);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        } => {
            h.update(&316u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source_card.raw());
            put_u32(h, revealed.raw());
            put_u32(h, target.raw());
            h.update(&[*source_category, *stage]);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDingpin {
            player,
            target,
            skill,
        } => {
            h.update(&317u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            h.update(&318u16.to_le_bytes());
            put_u32(h, player.raw());
            h.update(&[*stage]);
        }
        EffectRequest::GainTopCardToSkillPile { player, skill } => {
            h.update(&319u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => {
            h.update(&320u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            h.update(&321u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseRepoluTargetsDraw { player, skill } => {
            h.update(&322u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            skill,
        } => {
            h.update(&323u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        } => {
            h.update(&324u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, restore_skill_def.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        } => {
            h.update(&325u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, result_card.raw());
            put_u8(h, crate::command::judge::suit_tag(*suit) as u8);
            put_u8(h, *rank);
            put_ids(h, rewriters.iter().map(|player| player.raw()));
            put_u32(h, *heart_recover_amount);
            put_u32(h, *diamond_draw_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveZhuiyi {
            player,
            killer,
            skill,
        } => {
            h.update(&326u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, killer.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill,
        } => {
            h.update(&327u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&328u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            h.update(&329u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *initialized);
            put_ids(h, remaining_targets.iter().map(|player| player.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        } => {
            h.update(&330u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_recover);
        }
        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&331u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            skill,
        } => {
            h.update(&332u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
            color,
        } => {
            h.update(&333u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
            put_u8(h, *color);
        }
        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill,
        } => {
            h.update(&334u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        } => {
            h.update(&335u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }
        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => {
            h.update(&336u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_card.raw());
            put_ids(h, shan_cards.iter().map(|card| card.raw()));
            put_ids(h, excluded_recipients.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill,
        } => {
            h.update(&337u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, *draw_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => {
            h.update(&338u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, bonus_skill_def.raw());
        }
        EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        } => {
            h.update(&339u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, target.raw());
            put_u32(h, selected.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            skill,
        } => {
            h.update(&340u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, buff_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_bool(h, *accepted);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => {
            h.update(&341u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            skill,
        } => {
            h.update(&342u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        } => {
            h.update(&343u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *color);
            put_u32(h, *discarded_count);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, *gained_count);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => {
            h.update(&344u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *index);
            put_u8(h, *remaining);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => {
            h.update(&345u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        } => {
            h.update(&346u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            h.update(&347u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        } => {
            h.update(&348u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *lost_equipment_count);
            put_u32(h, distance_skill_def.raw());
            put_u8(h, *stage);
            put_u8(h, *reward_mask);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            skill,
        } => {
            h.update(&349u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            if *tw {
                h.update(b"tw-daoji/v1");
            }
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            skill,
        } => {
            h.update(&350u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *category);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        } => {
            h.update(&351u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, effect_skill_def.raw());
            put_u32(h, direct_hit_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, participants.iter().map(|player| player.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill,
        } => {
            h.update(&352u16.to_le_bytes());
            put_u32(h, owner.raw());
            put_u32(h, giver.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill,
        } => {
            h.update(&353u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill,
        } => {
            h.update(&354u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            h.update(&355u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        EffectRequest::TakeOneScopedCard { from, to, scope } => {
            h.update(&356u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
            put_u8(h, scope.tag());
        }
        EffectRequest::RevealHand { player } => {
            h.update(&357u16.to_le_bytes());
            put_u32(h, player.raw());
        }
        EffectRequest::DrawLinkedPlayers { count } => {
            h.update(&358u16.to_le_bytes());
            put_u32(h, *count);
        }
        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            put_u8(h, 0);
            h.update(&362u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *color_mask);
        }
        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&562u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, delta.unsigned_abs());
            put_u8(h, u8::from(*delta < 0));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMuzhen { .. } => {
            put_u8(h, 0);
            h.update(&993u16.to_le_bytes());
        }
        EffectRequest::RefreshSkill { skill } => {
            put_u8(h, 0);
            h.update(&994u16.to_le_bytes());
            put_u32(h, skill.raw());
        }
        EffectRequest::TagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }
        EffectRequest::UntagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }
        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ClanshenjunUse { player, skill } => {
            put_u8(h, 0);
            h.update(&998u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::StorageSet { skill, key, value } => {
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
            put_u32(h, skill.raw());
            put_u32(h, *key);
            put_u32(h, *value);
        }
        EffectRequest::DrawAllEventTargets { players, count } => {
            put_u8(h, 0);
            h.update(&1000u16.to_le_bytes());
            put_u32(h, players.len() as u32);
            for player in players {
                put_u32(h, player.raw());
            }
            put_u32(h, *count);
        }
        EffectRequest::RoundEnded { round, current } => {
            put_u8(h, 0);
            h.update(&1001u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }
        EffectRequest::RoundStarted { round, current } => {
            put_u8(h, 0);
            h.update(&1002u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }
        EffectRequest::ClanyuzhiReveal { player, skill } => {
            put_u8(h, 0);
            h.update(&1003u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1004u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::QieyiPhaseStart { player, skill } => {
            put_u8(h, 0);
            h.update(&1005u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::QieyiUseAfter { player, card, skill } => {
            put_u8(h, 0);
            h.update(&1006u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::JianzhiPhaseEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1007u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
    }
}

fn put_ids(h: &mut blake3::Hasher, values: impl IntoIterator<Item = u32>) {
    let values: Vec<_> = values.into_iter().collect();
    put_u32(h, values.len() as u32);
    for value in values {
        put_u32(h, value);
    }
}

fn identity_tag(i: crate::state::Identity) -> u8 {
    match i {
        crate::state::Identity::Lord => 1,
        crate::state::Identity::Loyalist => 2,
        crate::state::Identity::Rebel => 3,
        crate::state::Identity::Renegade => 4,
    }
}

fn suit_tag(s: Suit) -> u8 {
    match s {
        Suit::None => 0,
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

fn card_move_reason_tag(reason: engine_api::CardMoveReason) -> u8 {
    match reason {
        engine_api::CardMoveReason::Unspecified => 0,
        engine_api::CardMoveReason::Discard => 1,
        engine_api::CardMoveReason::Give => 2,
        engine_api::CardMoveReason::Recast => 3,
        engine_api::CardMoveReason::UseOrRespond => 4,
        engine_api::CardMoveReason::Use => 5,
        engine_api::CardMoveReason::Respond => 6,
        engine_api::CardMoveReason::Potduzuo => 7,
        engine_api::CardMoveReason::WuguRemained => 8,
    }
}

fn zone_kind_tag(k: crate::state::ZoneKind) -> u8 {
    match k {
        crate::state::ZoneKind::DrawPile => 1,
        crate::state::ZoneKind::DiscardPile => 2,
        crate::state::ZoneKind::Hand => 3,
        crate::state::ZoneKind::Equip => 4,
        crate::state::ZoneKind::Judge => 5,
        crate::state::ZoneKind::Processing => 6,
        crate::state::ZoneKind::Removed => 7,
        crate::state::ZoneKind::Expansion => 8,
    }
}

fn hash_outcome(h: &mut blake3::Hasher, o: &MatchOutcome) {
    match o {
        MatchOutcome::InProgress => put_u8(h, 0),
        MatchOutcome::RebelWin => put_u8(h, 1),
        MatchOutcome::LoyalistWin => put_u8(h, 2),
        MatchOutcome::RenegadeWin { player } => {
            put_u8(h, 3);
            put_u32(h, player.raw());
        }
        MatchOutcome::Aborted { reason_code } => {
            put_u8(h, 4);
            put_u32(h, *reason_code);
        }
        MatchOutcome::Draw => put_u8(h, 5),
        MatchOutcome::GroupWin { group } => {
            // Guozhan faction victory (ADR 0598 §D4). Identity matches never take this
            // variant, so identity outcome hashes are unchanged.
            put_u8(h, 6);
            put_str(h, group);
        }
        MatchOutcome::SingleWin { player } => {
            // M4 单挑（single）: last living player wins.
            put_u8(h, 7);
            put_u32(h, player.raw());
        }
    }
}

fn put_u8(h: &mut blake3::Hasher, v: u8) {
    h.update(&[v]);
}
fn put_bool(h: &mut blake3::Hasher, v: bool) {
    put_u8(h, u8::from(v));
}
fn put_u32(h: &mut blake3::Hasher, v: u32) {
    h.update(&v.to_le_bytes());
}
fn put_i32(h: &mut blake3::Hasher, v: i32) {
    h.update(&v.to_le_bytes());
}
fn put_u64(h: &mut blake3::Hasher, v: u64) {
    h.update(&v.to_le_bytes());
}
fn put_str(h: &mut blake3::Hasher, s: &str) {
    put_u32(h, s.len() as u32);
    h.update(s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameState;

    #[test]
    fn empty_state_hash_stable() {
        let a = canonical_state_hash(&GameState::empty());
        let b = canonical_state_hash(&GameState::empty());
        assert_eq!(a, b);
    }
}

