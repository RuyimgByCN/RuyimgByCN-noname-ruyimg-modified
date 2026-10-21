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
    // 手牌上限修正（T15②，2026-10-11 补编码）：条件编码——空表不写字节，
    // 既有哈希不变；仅当技能（如 muyin 的 +1）写入后进入哈希，修复两个
    // 仅此字段不同的状态哈希相同的一致性缺口。
    if !state.hand_limit_bonus.is_empty() {
        h.update(b"state-hand-limit-bonus/v1");
        put_u32(&mut h, state.hand_limit_bonus.len() as u32);
        for (player, bonus) in &state.hand_limit_bonus {
            put_u32(&mut h, *player);
            put_i32(&mut h, *bonus);
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
        EffectRequest::DrawCards { .. }
        | EffectRequest::ModifyCurrentDamage { .. }
        | EffectRequest::MoveCards { .. }
        | EffectRequest::MoveCardsWithReason { .. }
        | EffectRequest::UseCard { .. }
        | EffectRequest::DealDamage { .. }
        | EffectRequest::RecoverHp { .. }
        | EffectRequest::ChangeMaxHp { .. }
        | EffectRequest::AddSkill { .. }
        | EffectRequest::RemoveSkill { .. }
        | EffectRequest::AddMark { .. }
        | EffectRequest::RemoveMark { .. }
        | EffectRequest::RevealCards { .. }
        | EffectRequest::RequestChoice { .. }
        | EffectRequest::EnterDying { .. }
        | EffectRequest::KillPlayer { .. }
        | EffectRequest::SetTurnState { .. }
        | EffectRequest::EmitDebateResolved { .. }
        | EffectRequest::EmitRuleEvent { .. }
        | EffectRequest::DiscardFromHand { .. }
        | EffectRequest::DiscardFromHandEquip { .. }
        | EffectRequest::TakeFromHand { .. }
        | EffectRequest::DiscardCards { .. }
        | EffectRequest::CardResponded { .. }
        | EffectRequest::RemoveCardsByKind { .. }
        | EffectRequest::DealShaDamage { .. }
        | EffectRequest::ChooseDiscardOneHandOrEquip { .. }
        | EffectRequest::JudgeGainEquipmentByColor { .. }
        | EffectRequest::RevealDiscardSuitGainOrUse { .. }
        | EffectRequest::ChooseDiscardOrFireDamageSkipPhases { .. }
        | EffectRequest::DrawCardsFromSkill { .. }
        | EffectRequest::DiscardAllHandAndEquip { .. }
        | EffectRequest::RequestDyingSave { .. }
        | EffectRequest::DrawEachBySeat { .. }
        | EffectRequest::DrawThenGiveOne { .. }
        | EffectRequest::DrawThenTakeOne { .. }
        | EffectRequest::TakeOneCard { .. }
        | EffectRequest::TurnOver { .. }
        | EffectRequest::RemoveSkillByDef { .. }
        | EffectRequest::ChooseDiscardHandAddSkillByColor { .. }
        | EffectRequest::RevealTopGainNonBasicOrSwapBasic { .. }
        | EffectRequest::PreventDamage { .. }
        | EffectRequest::CommitDamage { .. }
        | EffectRequest::JudgeCancelCurrentShaTargetOnRed { .. }
        | EffectRequest::ChooseDrawOrRecover { .. }
        | EffectRequest::DealCardDamage { .. }
        | EffectRequest::ChooseUseVirtualSha { .. }
        | EffectRequest::ChooseUseVirtualShaWithBonus { .. }
        | EffectRequest::AdjustHandTo { .. }
        | EffectRequest::GainCardByKind { .. }
        | EffectRequest::JudgeRecoverOnSuit { .. }
        | EffectRequest::GiveCards { .. }
        | EffectRequest::UseVirtualCardOn { .. }
        | EffectRequest::UseFieldCard { .. }
        | EffectRequest::EmitDuelResolved { .. }
        | EffectRequest::JudgeDamageOtherOnSuit { .. }
        | EffectRequest::RecastCards { .. }
        | EffectRequest::DisableEquipSlots { .. }
        | EffectRequest::MoveEquipmentThenDraw { .. }
        | EffectRequest::ChooseUseVirtualShaOrDraw { .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { .. }
        | EffectRequest::DiscardTargetEquipmentGainMount { .. }
        | EffectRequest::DiscardTargetCardGainMount { .. }
        | EffectRequest::DiscardTargetCards { .. }
        | EffectRequest::MakeCurrentShaTargetUnrespondable { .. }
        | EffectRequest::ShaDodged { .. }
        | EffectRequest::ChooseDiscardTargetCard { .. }
        | EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart { .. }
        | EffectRequest::ChooseGainTopCardsBySuit { .. }
        | EffectRequest::ChooseOtherDrawByHandCount { .. }
        | EffectRequest::ChooseDiscardOneThenGainTargetEquipment { .. }
        | EffectRequest::ChooseTargetDraw { .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { .. }
        | EffectRequest::RefundCurrentShaUseAndDrawIfRed { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed { .. }
        | EffectRequest::QueueOpeningExtraPlayPhase { .. }
        | EffectRequest::BeginTurn { .. }
        | EffectRequest::FinishTurnBegin { .. }
        | EffectRequest::AdvanceOpeningExtraPlay { .. }
        | EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { .. }
        | EffectRequest::ChooseAddCurrentShaTarget { .. }
        | EffectRequest::CommitAddCurrentShaTarget { .. }
        | EffectRequest::ChooseStoreHandShaInSkillPile { .. }
        | EffectRequest::ChooseSpendSkillPileForDyingJiu { .. }
        | EffectRequest::CommitStoreHandShaInSkillPile { .. }
        | EffectRequest::CommitSpendSkillPileForDyingJiu { .. }
        | EffectRequest::GuanxingTop { .. }
        | EffectRequest::GuanxingTopByFaluMarks { .. }
        | EffectRequest::ChooseOtherGiveThenRecover { .. }
        | EffectRequest::ChooseHandCompareBetween { .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { .. }
        | EffectRequest::ChooseHandCompareGrantTargetOnWin { .. }
        | EffectRequest::ChooseCompareSelectedHandCard { .. }
        | EffectRequest::AddBoundSkill { .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { .. }
        | EffectRequest::CommitAddSkillBoundCardTarget { .. }
        | EffectRequest::ChooseDiscardMatchingEquipColorAddSkill { .. }
        | EffectRequest::AddCardColorLockSkill { .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { .. }
        | EffectRequest::AddCardUsePolicySkill { .. }
        | EffectRequest::ChooseDrawOrDiscardAddSkill { .. }
        | EffectRequest::ChooseDiscardedBlackBasicAsBingliang { .. }
        | EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::UseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { .. }
        | EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded { .. }
        | EffectRequest::FinishLinkIfTargetWounded { .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { .. }
        | EffectRequest::DrawThenGiveExactRecordPhaseTarget { .. }
        | EffectRequest::ClearPhaseGiftTarget { .. }
        | EffectRequest::EmitShaResolved { .. }
        | EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha { .. }
        | EffectRequest::ChooseUseShaAfterResolvedGift { .. }
        | EffectRequest::SkipCurrentPhase { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { .. }
        | EffectRequest::RevealUniformHandDrawSelfChooseOthers { .. }
        | EffectRequest::ChooseCardUserOrAddTarget { .. }
        | EffectRequest::CommitTransferSavedCardUser { .. }
        | EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { .. }
        | EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment { .. }
        | EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { .. }
        | EffectRequest::IncreaseCurrentDrawCount { .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
        | EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard { .. }
        | EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage { .. }
        | EffectRequest::ContinueIncreaseBoundDamage { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCard { .. }
        | EffectRequest::RandomizeCurrentHandCompareOpponent { .. }
        | EffectRequest::EmitHandCompareFinished { .. }
        | EffectRequest::GainHighestRankShaFromEventCards { .. }
        | EffectRequest::GainHandCompareResultCard { .. }
        | EffectRequest::ChooseAddOrRemoveCurrentCardTarget { .. }
        | EffectRequest::ChooseHandCompare { .. }
        | EffectRequest::ChooseOtherDamage { .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { .. }
        | EffectRequest::GainDiscardCardByKind { .. }
        | EffectRequest::RevealUniformHandChooseDamage { .. } => {
            effects_draw_cards::hash_effect(h, effect)
        }
        EffectRequest::ChooseGainDiscardCardByKind { .. }
        | EffectRequest::DrawForLifetimeCardActionDivisors { .. }
        | EffectRequest::ChooseAddMissingSkill { .. }
        | EffectRequest::AddFirstMissingSkill { .. }
        | EffectRequest::ChooseTargetsDraw { .. }
        | EffectRequest::GainEventCards { .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { .. }
        | EffectRequest::EmitJudgmentResolved { .. }
        | EffectRequest::JudgeRecoverOtherOnSuit { .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { .. }
        | EffectRequest::ContinueDyingSave { .. }
        | EffectRequest::RestoreCharacterState { .. }
        | EffectRequest::DiscardAllHandEquipJudge { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { .. }
        | EffectRequest::UseVirtualCardOnSelectedTargets { .. }
        | EffectRequest::DealDamageBanActiveIfDying { .. }
        | EffectRequest::SpjungongDiscardOrLoseThenSha { .. }
        | EffectRequest::BanActiveUntilTurnEnd { .. }
        | EffectRequest::SpwanweiActive { .. }
        | EffectRequest::SpwanweiDying { .. }
        | EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { .. }
        | EffectRequest::ChooseRecoverTarget { .. }
        | EffectRequest::GiveAllHandThenChooseUseOneOrDamage { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable { .. }
        | EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants { .. }
        | EffectRequest::ResolveSpxizhan { .. }
        | EffectRequest::ResolveSpsongshu { .. }
        | EffectRequest::ResolveSpcunsi { .. }
        | EffectRequest::ResolveYuli { .. }
        | EffectRequest::ResolveTingwei { .. }
        | EffectRequest::ResolveXinmingshi { .. }
        | EffectRequest::ResolveXunyi { .. }
        | EffectRequest::ResolveBoming { .. }
        | EffectRequest::ResolveEjian { .. }
        | EffectRequest::ResolveYiyong { .. }
        | EffectRequest::ResolveShanxie { .. }
        | EffectRequest::ResolveSheyi { .. }
        | EffectRequest::ResolveTianyin { .. }
        | EffectRequest::ResolveDbzhuifeng { .. }
        | EffectRequest::ResolveDbquedi { .. }
        | EffectRequest::ResolveSpyinju { .. }
        | EffectRequest::ResolveSpchijie { .. }
        | EffectRequest::ResolveSpshangyi { .. }
        | EffectRequest::ResolveSpjianyi { .. }
        | EffectRequest::ResolveFyjianyu { .. }
        | EffectRequest::ResolveLiubing { .. }
        | EffectRequest::ResolveHeji { .. }
        | EffectRequest::ResolveSpyajun { .. }
        | EffectRequest::ResolveSpzundi { .. }
        | EffectRequest::ResolveXinyangjie { .. }
        | EffectRequest::ResolveQingyu { .. }
        | EffectRequest::ResolveZhengsu { .. }
        | EffectRequest::ResolveHoufeng { .. }
        | EffectRequest::ResolveMbdiancai { .. }
        | EffectRequest::ResolveRechuhai { .. }
        | EffectRequest::ResolveXiongshu { .. }
        | EffectRequest::ResolveJianhui { .. }
        | EffectRequest::ResolveSpqiai { .. }
        | EffectRequest::ResolveSpshanxi { .. }
        | EffectRequest::ResolveSpshanxiSuoming { .. }
        | EffectRequest::ResolveTongduo { .. }
        | EffectRequest::ResolveYuanqing { .. }
        | EffectRequest::ResolveShuchen { .. }
        | EffectRequest::ResolveDuanbi { .. }
        | EffectRequest::ResolveZaoli { .. }
        | EffectRequest::ResolveGuying { .. }
        | EffectRequest::ResolveMbzhonghui { .. }
        | EffectRequest::ResolvePotjiejie { .. }
        | EffectRequest::ResolvePotqingshi { .. }
        | EffectRequest::ResolvePotweiyan { .. }
        | EffectRequest::ResolvePotzhouyu { .. }
        | EffectRequest::ResolvePotlvyi { .. }
        | EffectRequest::ResolvePotcaoshuang { .. }
        | EffectRequest::ResolvePotyuji { .. }
        | EffectRequest::ResolveRehengjiang { .. }
        | EffectRequest::ResolveTuntian { .. }
        | EffectRequest::ResolveZaoxian { .. }
        | EffectRequest::ResolveTiaoxin { .. }
        | EffectRequest::ResolveZhiji { .. }
        | EffectRequest::ResolveZaiqi { .. }
        | EffectRequest::ResolveTwqiushou { .. }
        | EffectRequest::ResolveZhengrong { .. }
        | EffectRequest::ResolveHongju { .. }
        | EffectRequest::ResolveQingjian { .. }
        | EffectRequest::ResolveReleiji { .. }
        | EffectRequest::ResolveHuangtian { .. }
        | EffectRequest::ResolveHaoshi { .. }
        | EffectRequest::ResolveDimeng { .. }
        | EffectRequest::ResolveQiaobian { .. }
        | EffectRequest::ExcludeSavedShaTarget { .. }
        | EffectRequest::ResolveXiangle { .. }
        | EffectRequest::ResolveFangquanSkip { .. }
        | EffectRequest::ResolveFangquanGrant { .. }
        | EffectRequest::ResolveRuoyu { .. }
        | EffectRequest::ResolveZuilun { .. }
        | EffectRequest::ResolveFuyin { .. }
        | EffectRequest::BanUseAgainst { .. }
        | EffectRequest::ResolveJuzhan { .. }
        | EffectRequest::ResolveQizhi { .. }
        | EffectRequest::ResolveJinqu { .. }
        | EffectRequest::ResolveJianxiang { .. }
        | EffectRequest::ResolveShenshi { .. }
        | EffectRequest::ResolveMingren { .. }
        | EffectRequest::ResolveZhenliang { .. }
        | EffectRequest::ResolveKuizhu { .. }
        | EffectRequest::ResolveZhizheng { .. }
        | EffectRequest::ResolveLijun { .. }
        | EffectRequest::ResolveLiangyin { .. }
        | EffectRequest::ResolveKongsheng { .. }
        | EffectRequest::ResolveDccunsi { .. }
        | EffectRequest::RecordCurrentShaAllTargets { .. }
        | EffectRequest::ChooseGiveOrDiscardHand { .. } => {
            effects_choose_gain_discard_card_by_kind::hash_effect(h, effect)
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { .. }
        | EffectRequest::ResolveDcjue { .. }
        | EffectRequest::ResolveRuanji { .. }
        | EffectRequest::ResolveZhengbi { .. }
        | EffectRequest::ResolveFengying { .. }
        | EffectRequest::ResolveQiaojian { .. }
        | EffectRequest::ResolveHuzun { .. }
        | EffectRequest::ResolveZhugemengxue { .. }
        | EffectRequest::DrawCardsFromBottom { .. }
        | EffectRequest::SetDrawPhaseFromBottom { .. }
        | EffectRequest::ResolveChenglve { .. }
        | EffectRequest::ResolveShicai { .. }
        | EffectRequest::ResolveBailingyun { .. }
        | EffectRequest::ResolveDcshaowei { .. }
        | EffectRequest::ResolveDcdichou { .. }
        | EffectRequest::ResolveDcdufeng { .. }
        | EffectRequest::ChooseDiscardHandRange { .. }
        | EffectRequest::ResolveDcctjiuxian { .. }
        | EffectRequest::ResolveBianyue { .. }
        | EffectRequest::ResolveZhupeilan { .. }
        | EffectRequest::ResolveDcSbZhangxiu { .. }
        | EffectRequest::ResolveDcSbZhouyu { .. }
        | EffectRequest::ResolveDcSbLusu { .. }
        | EffectRequest::ResolveZhangjian { .. }
        | EffectRequest::ResolveZhugeruoxue { .. }
        | EffectRequest::ResolveCaoyi { .. }
        | EffectRequest::ResolveMalingli { .. }
        | EffectRequest::ResolveNsshuaiyan { .. }
        | EffectRequest::ResolveWdbHanlong { .. }
        | EffectRequest::ResolveWdbLiufu { .. }
        | EffectRequest::ResolveWdbLiuyan { .. }
        | EffectRequest::ResolveWdbLiuzan { .. }
        | EffectRequest::ResolveWdbTianyu { .. }
        | EffectRequest::ContinueYoudishenru { .. }
        | EffectRequest::ResolveWdbXizhenxihong { .. }
        | EffectRequest::ContinueWdbFuchou { .. }
        | EffectRequest::ResolveWdbYangyi { .. }
        | EffectRequest::ContinueDiaobingqianjiang { .. }
        | EffectRequest::ResolveWdbZaozhiRenjun { .. }
        | EffectRequest::ResolveDddXianxi { .. }
        | EffectRequest::ResolveDddXiaheng { .. }
        | EffectRequest::ResolveDddLiuchong { .. }
        | EffectRequest::ResolveDddDuanbing { .. }
        | EffectRequest::ResolveDddWuzhi { .. }
        | EffectRequest::ResolveDddXujing { .. }
        | EffectRequest::ResolveDddCaomao { .. }
        | EffectRequest::ResolveDddXinxianying { .. }
        | EffectRequest::ResolveDddXianglang { .. }
        | EffectRequest::ResolveDddYujin { .. }
        | EffectRequest::ResolveDddLiuye { .. }
        | EffectRequest::ResolveDddBaosanniang { .. }
        | EffectRequest::ResolveDddZhenji { .. }
        | EffectRequest::ResolveDddZhaoang { .. }
        | EffectRequest::ResolveDddLiuba { .. }
        | EffectRequest::ResolveDddJianshuo { .. }
        | EffectRequest::ResolveDddGuanning { .. }
        | EffectRequest::ResolveDddCaoshuang { .. }
        | EffectRequest::PreventRecovery { .. }
        | EffectRequest::CommitRecovery { .. }
        | EffectRequest::ResolveDddXuelingyun { .. }
        | EffectRequest::ResolveDddLiuhong { .. }
        | EffectRequest::ResolveDddXiahouxuan { .. }
        | EffectRequest::ResolveDddZhangkai { .. }
        | EffectRequest::ResolveDddLiangxi { .. }
        | EffectRequest::GuozhanUseZhulianBihe { .. }
        | EffectRequest::ResolveShoulijian { .. }
        | EffectRequest::CancelIncomingCardUse { .. }
        | EffectRequest::ChooseOtherGainOneOfTwoSkills { .. }
        | EffectRequest::ResolveGwyewuChain { .. }
        | EffectRequest::ContinueWugu { .. }
        | EffectRequest::ResolveXiongluan { .. }
        | EffectRequest::ResolveGzbuqu { .. }
        | EffectRequest::ResolveGzbuquRecover { .. }
        | EffectRequest::ResolveSbbiyue { .. }
        | EffectRequest::ResolveSxrmqiewang { .. }
        | EffectRequest::ResolveSxrmqianliu { .. }
        | EffectRequest::ResolveTwjuxiang { .. }
        | EffectRequest::ResolveXicha { .. }
        | EffectRequest::ResolvePottuntian { .. }
        | EffectRequest::ResolvePotjixi { .. }
        | EffectRequest::ResolvePotceduan { .. }
        | EffectRequest::ResolvePottaishici { .. }
        | EffectRequest::ResolveTwfupan { .. }
        | EffectRequest::ResolveTwjiaohua { .. }
        | EffectRequest::ResolveTwkujian { .. }
        | EffectRequest::ResolveTwruilian { .. }
        | EffectRequest::EquipWeaponFromDrawPile { .. }
        | EffectRequest::ResolveTwjianwei { .. }
        | EffectRequest::ResolveTwtanfeng { .. }
        | EffectRequest::ResolveTwzhibian { .. }
        | EffectRequest::ResolveTwyuyan { .. }
        | EffectRequest::ResolveTwguoyi { .. }
        | EffectRequest::ReplayVirtualCardOnTargets { .. }
        | EffectRequest::ResolveTwchuhai { .. }
        | EffectRequest::ResolveTwyizhu { .. } => {
            effects_choose_discard_hand_equip_then_recover_target::hash_effect(h, effect)
        }
        EffectRequest::ResolveTwluanchou { .. }
        | EffectRequest::ResolveTianxiang { .. }
        | EffectRequest::ResolveTwxiawei { .. }
        | EffectRequest::ResolveTwbianfuren { .. }
        | EffectRequest::ResolveTwfeiyi { .. }
        | EffectRequest::ResolveTwchenzhen { .. }
        | EffectRequest::ResolveTwqianju { .. }
        | EffectRequest::ResolveTwqingxi { .. }
        | EffectRequest::ResolveTwzaoli { .. }
        | EffectRequest::ResolveTwzhengjian { .. }
        | EffectRequest::ResolveTwtianyu { .. }
        | EffectRequest::ResolveTwwujing { .. }
        | EffectRequest::ResolveTwshanghe { .. }
        | EffectRequest::ResolveTwwangchang { .. }
        | EffectRequest::ResolveTwcaozhao { .. }
        | EffectRequest::ResolveTwguohuai { .. }
        | EffectRequest::ResolveTwchengpu { .. }
        | EffectRequest::ResolveTwcaoang { .. }
        | EffectRequest::ResolveTwniujin { .. }
        | EffectRequest::ResolveRetianxiang { .. }
        | EffectRequest::ResolveLianzhou { .. }
        | EffectRequest::ResolveStddechong { .. }
        | EffectRequest::DiscardHandEquipCount { .. }
        | EffectRequest::ResolveJieliangGain { .. }
        | EffectRequest::ResolveStdjujian { .. }
        | EffectRequest::ResolveLulve { .. }
        | EffectRequest::ResolveDctingxian { .. }
        | EffectRequest::ResolveHfjieying { .. }
        | EffectRequest::ResolveDcnuchen { .. }
        | EffectRequest::ResolveDcdanji { .. }
        | EffectRequest::ResolveDczhiheng { .. }
        | EffectRequest::ResolveStdgaobian { .. }
        | EffectRequest::ResolveSpjiedao { .. }
        | EffectRequest::ResolveSibian { .. }
        | EffectRequest::SkipNextDiscardPhase { .. }
        | EffectRequest::ResolveSpfengyin { .. }
        | EffectRequest::ResolveXinfuLianpian { .. }
        | EffectRequest::ResolveGongjian { .. }
        | EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss { .. }
        | EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { .. }
        | EffectRequest::ResolveRefuhai { .. }
        | EffectRequest::ChooseGainTargetHandCeilHalfDifference { .. }
        | EffectRequest::ResolveGzjinfa { .. }
        | EffectRequest::ResolveWnQifeng { .. }
        | EffectRequest::ResolveXinkuangfu { .. }
        | EffectRequest::ResolveDcxiangshu { .. }
        | EffectRequest::ResolveDcxiangshuLottery { .. }
        | EffectRequest::ResolveDcyingtu { .. }
        | EffectRequest::GiveSelectedThenTargetMayUseDrawOwner { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { .. }
        | EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip { .. }
        | EffectRequest::ResolveMobiledanshou { .. }
        | EffectRequest::ResolveJsrgchiying { .. }
        | EffectRequest::EmitCardResolved { .. }
        | EffectRequest::ResolveMbqingdao { .. }
        | EffectRequest::ResolveMbxiugeng { .. }
        | EffectRequest::ResolveMbchenshe { .. }
        | EffectRequest::ResolveMbquanchong { .. }
        | EffectRequest::ResolveMbrenxing { .. }
        | EffectRequest::ResolveMbchizhang { .. }
        | EffectRequest::ResolveMbduanyang { .. }
        | EffectRequest::ResolveMbrunwei { .. }
        | EffectRequest::ResolveMbshuanghuai { .. }
        | EffectRequest::ResolveMbxiongtu { .. }
        | EffectRequest::ResolveMbganggeng { .. }
        | EffectRequest::ResolveMbsijian { .. }
        | EffectRequest::ResolveMbjianji { .. }
        | EffectRequest::ResolveMbyuanmo { .. }
        | EffectRequest::ResolveMbfeijing { .. }
        | EffectRequest::ResolveMbxiaoge { .. }
        | EffectRequest::ResolveShuangxiong { .. }
        | EffectRequest::ResolveZhijian { .. }
        | EffectRequest::ResolveGuzheng { .. }
        | EffectRequest::ResolveZhiba { .. }
        | EffectRequest::ResolveZhibaSettle { .. }
        | EffectRequest::ResolveDrltCongjian { .. }
        | EffectRequest::ResolveNzryFeijun { .. }
        | EffectRequest::ResolveDrltYongsi { .. }
        | EffectRequest::ResolveDrltWeidi { .. }
        | EffectRequest::ResolveLieren { .. }
        | EffectRequest::ResolveMbxuye { .. }
        | EffectRequest::ResolveMbkuangxiang { .. }
        | EffectRequest::ResolvePotchengpu { .. }
        | EffectRequest::ResolvePotjugu { .. }
        | EffectRequest::ResolvePotsifeng { .. }
        | EffectRequest::ResolvePotchenqun { .. }
        | EffectRequest::ResolvePotwanglie { .. }
        | EffectRequest::ResolvePothongyi { .. }
        | EffectRequest::ResolvePotguansha { .. }
        | EffectRequest::ResolvePotjiyu { .. }
        | EffectRequest::ResolveMbzhuji { .. }
        | EffectRequest::ResolvePotlibing { .. }
        | EffectRequest::ResolvePotpoxi { .. }
        | EffectRequest::ResolvePotfuan { .. }
        | EffectRequest::ResolvePotyinxian { .. }
        | EffectRequest::ResolvePothuanjie { .. }
        | EffectRequest::ResolvePotnigu { .. } => {
            effects_resolve_twluanchou::hash_effect(h, effect)
        }
        EffectRequest::ResolvePotlulian { .. }
        | EffectRequest::ResolvePotdimeng { .. }
        | EffectRequest::ResolvePothaoshi { .. }
        | EffectRequest::ResolveSpmiaolve { .. }
        | EffectRequest::ResolveSpyingjia { .. }
        | EffectRequest::ResolveMantianguohai { .. }
        | EffectRequest::ResolveXinfuQinguo { .. }
        | EffectRequest::ResolveDcmanzhi { .. }
        | EffectRequest::ResolveXinxingluan { .. }
        | EffectRequest::ResolveDckuizhen { .. }
        | EffectRequest::ResolveDckuizhenResolve { .. }
        | EffectRequest::ResolveNsdingzhou { .. }
        | EffectRequest::ResolveSxrmlanjiao { .. }
        | EffectRequest::ResolveRekuanshi { .. }
        | EffectRequest::ResolveRekuanshiResolve { .. }
        | EffectRequest::ResolveDcmoyu { .. }
        | EffectRequest::ResolveXizhen { .. }
        | EffectRequest::ResolveXuxie { .. }
        | EffectRequest::ResolveDcchiying { .. }
        | EffectRequest::ResolveXuxieAdd { .. }
        | EffectRequest::ResolveTwmoukui { .. }
        | EffectRequest::ResolveTwfuman { .. }
        | EffectRequest::ResolveCuijin { .. }
        | EffectRequest::ResolveTwqingkou { .. }
        | EffectRequest::ResolveTwchenxun { .. }
        | EffectRequest::ResolveTwfenxian { .. }
        | EffectRequest::SetSkillContentFlag { .. }
        | EffectRequest::CommitDamageBanActiveIfDying { .. }
        | EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::ChooseUseVirtualCard { .. }
        | EffectRequest::ChooseUseVirtualNanman { .. }
        | EffectRequest::ResolveQiluan { .. }
        | EffectRequest::DiscardAllOwnJudge { .. }
        | EffectRequest::StoreEventCardInSkillPile { .. }
        | EffectRequest::ResolveOllinjie { .. }
        | EffectRequest::ChooseTargetGainMaxHp { .. }
        | EffectRequest::GainTargetHandCard { .. }
        | EffectRequest::ChooseClanNotMaxHandLimitGain { .. }
        | EffectRequest::GainWholeSkillPile { .. }
        | EffectRequest::ChooseOneFromSkillPileDiscard { .. }
        | EffectRequest::PlaceHandCardsToDeckTop { .. }
        | EffectRequest::RearrangePileToDeckTop { .. }
        | EffectRequest::ResolveQianlong { .. }
        | EffectRequest::ResolveJsrgzuozhan { .. }
        | EffectRequest::ResolveJsrgzuozhanGain { .. }
        | EffectRequest::ResolveTwmouzhi { .. }
        | EffectRequest::ResolveTwdaigui { .. }
        | EffectRequest::ResolveTwmitu { .. }
        | EffectRequest::ResolveQiantun { .. }
        | EffectRequest::ResolveYangming { .. }
        | EffectRequest::ResolveCanyun { .. }
        | EffectRequest::ResolveHengwu { .. }
        | EffectRequest::ResolveBiaozhao3 { .. }
        | EffectRequest::ResolveChuanyun { .. }
        | EffectRequest::ResolveBiaozhao2 { .. }
        | EffectRequest::ResolveDcniji { .. }
        | EffectRequest::ResolveDcmoukui { .. }
        | EffectRequest::ResolveZhongyan { .. }
        | EffectRequest::ResolveChumei { .. }
        | EffectRequest::ResolveShuaiyan { .. }
        | EffectRequest::CommitGaoyuanRedirect { .. }
        | EffectRequest::ResolveTwxiongjun { .. }
        | EffectRequest::ResolveYixiang { .. }
        | EffectRequest::ResolveXinfuQiai { .. }
        | EffectRequest::ResolveSpolaige { .. }
        | EffectRequest::ResolveOlzhijie { .. }
        | EffectRequest::ResolveOlsuyi { .. }
        | EffectRequest::ResolveOlqiaozhi { .. }
        | EffectRequest::ResolveDczhee { .. }
        | EffectRequest::ResolveDchaoxian { .. }
        | EffectRequest::ResolveZhongjie { .. }
        | EffectRequest::ResolveStarcuxiaWeiming { .. }
        | EffectRequest::ResolveSanshou { .. }
        | EffectRequest::ResolveNhlundao { .. }
        | EffectRequest::ResolveXiangshu { .. }
        | EffectRequest::ResolveOlwenren { .. }
        | EffectRequest::ResolveDcqiaolve { .. }
        | EffectRequest::ResolveDingce { .. }
        | EffectRequest::ResolveQiangyong { .. }
        | EffectRequest::ResolveMeiyong { .. }
        | EffectRequest::ResolveMbzhenann { .. }
        | EffectRequest::ResolveFriendbiwei { .. }
        | EffectRequest::ResolveChulao { .. }
        | EffectRequest::ResolveDcanxu { .. }
        | EffectRequest::ResolveDuoduan { .. }
        | EffectRequest::ResolveGaoyuan { .. }
        | EffectRequest::ResolveMbbeiming { .. }
        | EffectRequest::ResolveZhengjian { .. }
        | EffectRequest::ResolveReliangyin { .. }
        | EffectRequest::ResolveQinmian { .. }
        | EffectRequest::ResolveJsrgdaoren { .. }
        | EffectRequest::ResolveDuwang { .. }
        | EffectRequest::ResolveQuanjia { .. }
        | EffectRequest::ResolvePxqixian { .. }
        | EffectRequest::ResolveDcbingji { .. }
        | EffectRequest::ChooseHandCardsToSkillPile { .. }
        | EffectRequest::ChooseTargetHandCardToPile { .. }
        | EffectRequest::PlaceTopDeckToExpansion { .. }
        | EffectRequest::DiscardSkillPile { .. }
        | EffectRequest::GainFromSkillPile { .. }
        | EffectRequest::DiscardBasicDrawEqual { .. }
        | EffectRequest::StartDebateAllOthers { .. }
        | EffectRequest::StartDebateTargets { .. }
        | EffectRequest::PinDebateOpinions { .. }
        | EffectRequest::AlignDebateOpinions { .. }
        | EffectRequest::GainShadowCards { .. }
        | EffectRequest::ChangeGroup { .. }
        | EffectRequest::DistributeShadowCards { .. }
        | EffectRequest::RegisterTurnEndDebate { .. }
        | EffectRequest::ResolveTurnEndDebates { .. }
        | EffectRequest::ChooseGroup { .. }
        | EffectRequest::ChooseGainOtherSkill { .. }
        | EffectRequest::LoseGainedSkill { .. }
        | EffectRequest::ResolvePotheyun { .. }
        | EffectRequest::PlaceXumou { .. }
        | EffectRequest::PlaceDeckTopXumou { .. }
        | EffectRequest::ChoosePlaceHandXumou { .. }
        | EffectRequest::DiscardOneOwnXumou { .. }
        | EffectRequest::PlaceDeckTopExpansion { .. }
        | EffectRequest::CityConsume { .. }
        | EffectRequest::QueueExtraTurn { .. }
        | EffectRequest::ChooseOthersInsertExtraTurn { .. }
        | EffectRequest::QueueExtraPhases { .. }
        | EffectRequest::ResolveTwjieqiu { .. }
        | EffectRequest::SkipRemainingPhases { .. }
        | EffectRequest::ChooseOthersAddMark { .. }
        | EffectRequest::ResolveJsrgchaozheng { .. }
        | EffectRequest::ResolveJsrgsheju { .. }
        | EffectRequest::ChooseOtherAddMark { .. }
        | EffectRequest::PlaceHandCardToExpansion { .. }
        | EffectRequest::ExtraResolveCurrentTrick { .. }
        | EffectRequest::FinishPhaseEndVirtualCard
        | EffectRequest::IncreaseCurrentShaBaseDamage { .. }
        | EffectRequest::SkipNextPlayPhase { .. }
        | EffectRequest::ChooseOtherDrawOrRecover { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { .. }
        | EffectRequest::DrawSelectedTargets { .. }
        | EffectRequest::RecoverIfMinCurrentHp { .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { .. }
        | EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { .. }
        | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { .. } => {
            effects_resolve_potlulian::hash_effect(h, effect)
        }
        EffectRequest::RecoverSelectedTargets { .. }
        | EffectRequest::RecordSelectedPlayPhaseCategory { .. }
        | EffectRequest::TakeTargetHandThenRepeatUntilMax { .. }
        | EffectRequest::ContinueTakeLargerHandUntilMax { .. }
        | EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark { .. }
        | EffectRequest::DrawThenChooseGiveOrDiscardHand { .. }
        | EffectRequest::GainTargetEquipment { .. }
        | EffectRequest::DiscardAllTargetEquipment { .. }
        | EffectRequest::IncreaseCurrentShaTargetDamage { .. }
        | EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp { .. }
        | EffectRequest::ChooseBoundSourceAttackRangeDamage { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill { .. }
        | EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp { .. }
        | EffectRequest::GainEquipmentFromDrawPile { .. }
        | EffectRequest::GiveAllHandRecoverToOne { .. }
        | EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { .. }
        | EffectRequest::ChooseGiveOneHandEquipOrRecover { .. }
        | EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive { .. }
        | EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { .. }
        | EffectRequest::ChooseTargetsDamage { .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { .. }
        | EffectRequest::ChooseTargetRevealHandDamageIfSha { .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { .. }
        | EffectRequest::RecoverToMaxThenOptionalExchangeSkill { .. }
        | EffectRequest::RevealHandDiscardKind { .. }
        | EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { .. }
        | EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            ..
        }
        | EffectRequest::ChooseDrawOrRecastAddSkill { .. }
        | EffectRequest::ChooseGainShaForOpeningExtraPlay { .. }
        | EffectRequest::FinishOpeningExtraPlayShaGain { .. }
        | EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { .. }
        | EffectRequest::GiveAllHand { .. }
        | EffectRequest::GainCardIfDiscard { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { .. }
        | EffectRequest::RevealHandToPlayer { .. }
        | EffectRequest::ChooseOptionalDiscardTargetCard { .. }
        | EffectRequest::RevealHandChooseDiscardColor { .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { .. }
        | EffectRequest::ChooseScskuijiDiscard { .. }
        | EffectRequest::ChoosePskuijiDiscard { .. }
        | EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCard { .. }
        | EffectRequest::RecordCompareFailureSeatDamage { .. }
        | EffectRequest::ChooseSeatPathDamage { .. }
        | EffectRequest::DealDamageEachInOrder { .. }
        | EffectRequest::RemoveAllSkills { .. }
        | EffectRequest::ChooseOtherRecover { .. }
        | EffectRequest::EnableEquipSlot { .. }
        | EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { .. }
        | EffectRequest::JudgeSourceDiscardTwoHandOrDamage { .. }
        | EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw { .. }
        | EffectRequest::DistributeTopTwoCards { .. }
        | EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { .. }
        | EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage { .. }
        | EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner { .. }
        | EffectRequest::ChooseTargetGainBottomCard { .. }
        | EffectRequest::JudgeBlackGainRepeat { .. }
        | EffectRequest::UseVirtualJuedouSecondOnFirst { .. }
        | EffectRequest::ChooseDiscardRedirectCurrentSha { .. }
        | EffectRequest::JudgeSetSkillMarkToRank { .. }
        | EffectRequest::ChooseDiscardOneFromEachLinked { .. }
        | EffectRequest::DisableSkillByDefUntilTurnEnd { .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { .. }
        | EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic { .. }
        | EffectRequest::ChooseOptionalTargetsDraw { .. }
        | EffectRequest::RevealTargetHandShaOrDiscardBind { .. }
        | EffectRequest::DrawRememberedHandCards { .. }
        | EffectRequest::DiscardRememberedHandCards { .. }
        | EffectRequest::GainDrawCardsForLifetimeCardActionDivisors { .. }
        | EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay { .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { .. }
        | EffectRequest::ChooseMoveFieldCardTo { .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { .. }
        | EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo { .. }
        | EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::AddSkillUntilTurnEnd { .. }
        | EffectRequest::ResolveMassDamageDiscardTurnOver { .. }
        | EffectRequest::ResolveMassDamageDiscardHandEquip { .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::RecordActiveTarget { .. }
        | EffectRequest::ResolveMarkParityControl { .. }
        | EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage { .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { .. }
        | EffectRequest::InspectTargetHandChooseHeartDiscardOrTop { .. }
        | EffectRequest::ResolveYeyan { .. }
        | EffectRequest::ChooseAllLoseHpOrRecover { .. }
        | EffectRequest::ResolveQixing { .. }
        | EffectRequest::ResolveQixingWeather { .. }
        | EffectRequest::ResolveNzryLongnu { .. }
        | EffectRequest::ChooseLinkOther { .. }
        | EffectRequest::ResolveDrltPoxi { .. }
        | EffectRequest::ResolveDrltJieying { .. }
        | EffectRequest::ResolveDrltDuorui { .. }
        | EffectRequest::ResolveDrltZhiti { .. }
        | EffectRequest::AdvanceTurn
        | EffectRequest::ResolveShenSimayi { .. }
        | EffectRequest::ResolveBeige { .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { .. }
        | EffectRequest::ChooseDiscardOneHeOrLoseHp { .. }
        | EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { .. }
        | EffectRequest::ChooseSourceConfirmOwnerLoseHp { .. }
        | EffectRequest::DamageThenDiscardAllJudge { .. }
        | EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors { .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
        | EffectRequest::SkipNextJudgePhase { .. }
        | EffectRequest::SkipJudgeAndDrawPhases { .. }
        | EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit { .. }
        | EffectRequest::TakeOneHandCard { .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { .. }
        | EffectRequest::GainAllSkillPileCards { .. }
        | EffectRequest::GainOtherSkillPileCards { .. }
        | EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { .. }
        | EffectRequest::ResolveHiddenSkillPile { .. }
        | EffectRequest::SetNonLockedSkillSuppression { .. } => {
            effects_recover_selected_targets::hash_effect(h, effect)
        }
        EffectRequest::ResolveDiliYuheng { .. }
        | EffectRequest::FinalizeTiaojiyanmei { .. }
        | EffectRequest::BeginTiaojiyanmeiDiscard { .. }
        | EffectRequest::DiscardSelectedDrawAllHandBonus { .. }
        | EffectRequest::ChooseOtherYinghun { .. }
        | EffectRequest::ChooseOtherBizheng { .. }
        | EffectRequest::ChooseOtherXiashu { .. }
        | EffectRequest::ResolveGuanchaoRank { .. }
        | EffectRequest::ResolveGuanwei { .. }
        | EffectRequest::ResolveLanjiang { .. }
        | EffectRequest::ResolveRebingyi { .. }
        | EffectRequest::ResolveRebotu { .. }
        | EffectRequest::ResolveOldimeng { .. }
        | EffectRequest::ResolveRejieyin { .. }
        | EffectRequest::ResolveReanxu { .. }
        | EffectRequest::ResolveDrltJueyan { .. }
        | EffectRequest::ResolveRejizhi { .. }
        | EffectRequest::ResolveJiexun { .. }
        | EffectRequest::ResolveWenguaSetup { .. }
        | EffectRequest::ResolveWengua { .. }
        | EffectRequest::ResolveXinanguo { .. }
        | EffectRequest::ResolveGanglieThree { .. }
        | EffectRequest::ResolveZhongyi { .. }
        | EffectRequest::DiscardEquipmentSlot { .. }
        | EffectRequest::ResolveOldAnxu { .. }
        | EffectRequest::StoreSelectedInSkillPile { .. }
        | EffectRequest::EndMatchForCampIfSkillPileAtLeast { .. }
        | EffectRequest::ResolveOldQinqing { .. }
        | EffectRequest::ResolveOldMingjian { .. }
        | EffectRequest::ResolveOldXuanfeng { .. }
        | EffectRequest::ResolveJianrou { .. }
        | EffectRequest::ResolveOldHuisheng { .. }
        | EffectRequest::ResolveOldJiefan { .. }
        | EffectRequest::ResolveChegu { .. }
        | EffectRequest::ResolveJunkguixin { .. }
        | EffectRequest::ResolveYaoChenwei { .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { .. }
        | EffectRequest::RefundCurrentShaUse { .. }
        | EffectRequest::ResetLinkedSelectedTargets { .. }
        | EffectRequest::DamageThenDrawSelectedTargets { .. }
        | EffectRequest::ResolveYoudi { .. }
        | EffectRequest::ResolveKuangfu { .. }
        | EffectRequest::ResolveShanjia { .. }
        | EffectRequest::ResolveNewYajiao { .. }
        | EffectRequest::ResolveDingpin { .. }
        | EffectRequest::FinishCurrentPhaseAbortResolution { .. }
        | EffectRequest::GainTopCardToSkillPile { .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { .. }
        | EffectRequest::GainOneHandFromSelectedTargets { .. }
        | EffectRequest::ChooseRepoluTargetsDraw { .. }
        | EffectRequest::ResolveXinfuGuolun { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank { .. }
        | EffectRequest::ResolveBeigeConfigured { .. }
        | EffectRequest::ResolveZhuiyi { .. }
        | EffectRequest::ResolveDecadezhenjun { .. }
        | EffectRequest::ResolveSpXiaoguo { .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { .. }
        | EffectRequest::GiveSelectedThenRecoverTarget { .. }
        | EffectRequest::ResolveDingpan { .. }
        | EffectRequest::ResolveTanlu { .. }
        | EffectRequest::JudgeSameColorGainRepeat { .. }
        | EffectRequest::ChooseCurrentTrickTargetsToNullify { .. }
        | EffectRequest::JudgeBlackGainRepeatProtectHandLimit { .. }
        | EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { .. }
        | EffectRequest::ResolveDcqiaomeng { .. }
        | EffectRequest::ResolveNewReluoyi { .. }
        | EffectRequest::ResolveRemoshi { .. }
        | EffectRequest::ResolveXinyaoming { .. }
        | EffectRequest::ResolveRehuaiyi { .. }
        | EffectRequest::ResolveReanguo { .. }
        | EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { .. }
        | EffectRequest::ResolveBeizhu { .. }
        | EffectRequest::ResolveMbshanjia { .. }
        | EffectRequest::ResolveDaoji { .. }
        | EffectRequest::ResolveJiaohua { .. }
        | EffectRequest::ResolveLuanqun { .. }
        | EffectRequest::ChooseGiveTwoDifferentColorsOrRecover { .. }
        | EffectRequest::ResolveMbzhuguo { .. }
        | EffectRequest::ResolveMbanxian { .. }
        | EffectRequest::ResolveMbjinzu { .. }
        | EffectRequest::TakeOneScopedCard { .. }
        | EffectRequest::RevealHand { .. }
        | EffectRequest::DrawLinkedPlayers { .. }
        | EffectRequest::RevealTopGainIfColor { .. }
        | EffectRequest::ResolveMuzhen { .. }
        | EffectRequest::RefreshSkill { .. }
        | EffectRequest::TagCard { .. }
        | EffectRequest::UntagCard { .. }
        | EffectRequest::ClanshenjunReveal { .. }
        | EffectRequest::ClanshenjunUse { .. }
        | EffectRequest::StorageSet { .. }
        | EffectRequest::DrawAllEventTargets { .. }
        | EffectRequest::RoundEnded { .. }
        | EffectRequest::RoundStarted { .. }
        | EffectRequest::ClanyuzhiReveal { .. }
        | EffectRequest::ClanyuzhiRoundEnd { .. }
        | EffectRequest::QieyiPhaseStart { .. }
        | EffectRequest::QieyiUseAfter { .. }
        | EffectRequest::JianzhiPhaseEnd { .. } => {
            effects_resolve_dili_yuheng::hash_effect(h, effect)
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

    /// T15②（2026-10-11）：hand_limit_bonus 进入 canonical hash。
    /// 空表不写字节（empty 哈希不变，既有 golden 兼容）；仅此字段不同
    /// 的两个状态哈希必须不同（一致性缺口回归）。
    #[test]
    fn hand_limit_bonus_enters_canonical_hash() {
        let base = canonical_state_hash(&GameState::empty());
        let mut with_bonus = GameState::empty();
        with_bonus.hand_limit_bonus.insert(0, 1);
        let hashed = canonical_state_hash(&with_bonus);
        assert_ne!(base, hashed, "非空 hand_limit_bonus 必须改变哈希");

        // 编码含玩家与修正值：不同玩家/不同值 → 不同哈希。
        let mut other_player = GameState::empty();
        other_player.hand_limit_bonus.insert(1, 1);
        assert_ne!(
            hashed,
            canonical_state_hash(&other_player),
            "不同玩家的同值修正应产生不同哈希"
        );
        let mut other_value = GameState::empty();
        other_value.hand_limit_bonus.insert(0, 2);
        assert_ne!(
            hashed,
            canonical_state_hash(&other_value),
            "同玩家不同修正值应产生不同哈希"
        );

        // 空表条件编码：与补编码前的 empty 哈希路径一致（empty_state_hash_stable 同伴）。
        assert_eq!(
            base,
            canonical_state_hash(&GameState::empty()),
            "空表不应写字节"
        );
    }
}

mod effects_choose_discard_hand_equip_then_recover_target;
mod effects_choose_gain_discard_card_by_kind;
mod effects_draw_cards;
mod effects_recover_selected_targets;
mod effects_resolve_dili_yuheng;
mod effects_resolve_potlulian;
mod effects_resolve_twluanchou;
