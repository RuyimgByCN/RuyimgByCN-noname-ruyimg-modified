//! Batch 18 (`UP37` Complete Closure: `R02`, `R03`, `R04`, `R06`, `R07`, `R08`, `R09`,
//! `N02–N07`, `I02`, `I03`, `I05`) end-to-end verification suite.

use content_native_loader::{
    build_up37_closure_catalog, resolve_binding_voice_with_skin_override,
    up37_closure_extension_pack, verify_up37_i02_baseline_counts,
    verify_up37_i03_step_to_async_catalog_invariants, verify_up37_i05_presentation_bindings,
    UP37_GENERALS, UP37_STEP_TO_ASYNC_COMMITS,
};
use engine_api::PublicPresentationEvent;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent};
use engine_native_kernel::rvm_bridge::modifiers::{eval_card_use_canceled, eval_sha_use_limit};
use engine_native_kernel::rvm_ops::{
    commit_up37::{
        DCSBJUZHAN_ID, DCSBQINGSHI_ID, DCSBTAOZHOU_ID, DCSBZAOJI_ID, DCXIANGCHEN_ID, JIGE_ID,
        JISHE_ID, OLFANGMA_ID, OLHUAGUANG_ID, OLJIANGWU_ID, OLJINGXIAN_ID, OLJUNZHU_ID,
        OLREHUANSHI_ID, OLSBGUQU_ID, OLSHILUN_ID, OLXUANBAI_ID, OLZHUOMING_ID, POTCUIFENG_ID,
        RELONGHUN_ID, SHENFU_ID, STARBISHI_ID, STARDANGCHEN_ID, TT_ZHIYI_ID, TWJIANMING_ID,
        TWJUEMIE_ID, TWRENFANG_ID, UP37_STORAGE_DISPATCH_KEY, ZEXING_ID,
    },
    has_enabled_equip_slot,
};
use rule_api::{EffectRequest, PlayerId, SkillId};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_800_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: action.clone(),
    });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "submit({actor}, {action:?}) rejected: {out:?}"
    );
}

fn new_up37_engine(seats: usize, seed: u64) -> Engine {
    let pack = up37_closure_extension_pack();
    let mut e = Engine::new_with_extensions(seed, seats, &[&pack]).expect("new_with_extensions");
    for p in 0..seats as u8 {
        e.debug_clear_hand(p);
    }
    e
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn invoke_up37_op(e: &mut Engine, owner: u8, skill_name: &str, skill_id: u32) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(skill_id),
            key: UP37_STORAGE_DISPATCH_KEY,
            value: skill_id,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "UP37 effect dispatch rejected: {out:?}"
    );
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn turn_var(e: &Engine, p: u8, key: &str) -> u32 {
    flag(e, p, key)
}

fn set_turn_var(e: &mut Engine, p: u8, key: &str, val: u32) {
    set_flag(e, p, key, val);
}

#[test]
fn up37_r02_oljiangwu_authoritative_hufu_deduction_free_use_and_empty_gain_map_guard() {
    let mut e = new_up37_engine(2, 2026110701);
    let g = find_general(&e, "ol_le_zhugeliang");
    e.debug_assemble_general_def(0, g);

    // Case 1: Empty gainMap disconnect/cancel guard -> rejected without deducting hufu or granting zhanfa.
    e.state_core_mut()
        .vm_aux
        .marks
        .insert((0, OLJIANGWU_ID), 3);
    set_flag(&mut e, 0, "oljiangwu_empty_gain_map", 1);
    invoke_up37_op(&mut e, 0, "oljiangwu", OLJIANGWU_ID);
    assert_eq!(flag(&e, 0, "oljiangwu_rejected_empty_gain_map"), 1);
    assert_eq!(flag(&e, 0, "oljiangwu_acquired_count"), 0);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, OLJIANGWU_ID)).copied(),
        Some(3)
    );
    set_flag(&mut e, 0, "oljiangwu_empty_gain_map", 0);

    // Case 2: free_use == 1 -> acquires zhanfa without deducting hufu and clears free_use.
    set_turn_var(&mut e, 0, "free_use", 1);
    set_flag(&mut e, 0, "oljiangwu_selected_zhanfa", JISHE_ID);
    set_flag(&mut e, 0, "oljiangwu_cost", 2);
    invoke_up37_op(&mut e, 0, "oljiangwu", OLJIANGWU_ID);
    assert_eq!(turn_var(&e, 0, "free_use"), 0);
    assert_eq!(flag(&e, 0, "oljiangwu_acquired_count"), 1);
    assert_eq!(flag(&e, 0, "oljiangwu_zhanfa_active"), 1);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, OLJIANGWU_ID)).copied(),
        Some(3),
        "free_use must not deduct hufu marks"
    );

    // Case 3: free_use == 0 and hufu (3) >= cost (2) -> deducts 2 hufu and grants zhanfa.
    invoke_up37_op(&mut e, 0, "oljiangwu", OLJIANGWU_ID);
    assert_eq!(flag(&e, 0, "oljiangwu_acquired_count"), 2);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, OLJIANGWU_ID)).copied(),
        Some(1)
    );

    // Case 4: free_use == 0 and hufu (1) < cost (2) -> host authoritatively rejects/no-ops.
    invoke_up37_op(&mut e, 0, "oljiangwu", OLJIANGWU_ID);
    assert_eq!(flag(&e, 0, "oljiangwu_rejected_insufficient_hufu"), 1);
    assert_eq!(flag(&e, 0, "oljiangwu_acquired_count"), 2);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, OLJIANGWU_ID)).copied(),
        Some(1)
    );
}

#[test]
fn up37_r03_starbishi_counts_and_triggers_on_judge_zone_card_loss() {
    let mut e = new_up37_engine(2, 2026110702);
    let g = find_general(&e, "star_dongzhuo");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    // Place a delayed trick card into P0's judge zone.
    let judge_card = e.debug_place_judge(0, "le");

    // Discard all cards in P0's judge zone via host primitive.
    let out = e.debug_commit_effect(
        "starbishi",
        0,
        EffectRequest::DiscardAllOwnJudge {
            player: PlayerId::new(1),
            skill: SkillId::new(STARBISHI_ID),
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");

    // Judge-zone card loss must update vm_aux.lost, cards_lost, judge_cards_lost, and trigger starbishi.
    assert!(
        e.state_core()
            .vm_aux
            .lost
            .get(&0)
            .is_some_and(|v| v.contains(&judge_card)),
        "judge-zone card loss must be recorded in vm_aux.lost"
    );
    assert_eq!(turn_var(&e, 0, "cards_lost"), 1);
    assert_eq!(turn_var(&e, 0, "judge_cards_lost"), 1);
    assert!(
        turn_var(&e, 0, "starbishi_triggered") >= 1,
        "starbishi must trigger on judge-zone card loss"
    );
    assert_eq!(flag(&e, 0, "starbishi_from_judge"), 1);
}

#[test]
fn up37_r04_jsrgzonghai_is_non_silent_and_emits_skill_triggered() {
    let mut e = new_up37_engine(2, 2026110703);
    e.debug_grant(0, "jsrgzonghai");
    e.debug_set_turn(0, Phase::Play, 1);

    let log_before = e.log().len();
    // Inflict 4 damage on P0 (max_hp 4) so P0 enters Dying and triggers jsrgzonghai.
    let out = e.debug_deal_damage(Some(1), 0, 4, DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");

    let triggered = e.log()[log_before..].iter().any(|ev| {
        matches!(
            ev,
            LogEvent::SkillTriggered { owner: 0, skill, .. } if *skill == "jsrgzonghai"
        )
    });
    assert!(
        triggered,
        "jsrgzonghai must emit LogEvent::SkillTriggered (non-silent, upstream c1e6e7ff)"
    );
    assert_eq!(flag(&e, 0, "jsrgzonghai_invoked"), 1);

    e.release_committed_presentation();
    let batch = e
        .take_public_presentation()
        .expect("public presentation batch");
    let has_skill_activated = batch.occurrences.iter().any(|occ| {
        matches!(
            &occ.event,
            PublicPresentationEvent::SkillActivated { skill, .. }
                if format!("{:?}", skill.definition.local_id).contains("jsrgzonghai")
        )
    });
    assert!(
        has_skill_activated,
        "jsrgzonghai must emit PublicPresentationEvent::SkillActivated"
    );
}

#[test]
fn up37_r06_jige_virtual_empty_subcards_and_dcsbjuzhan_abolished_weapon_slot() {
    let mut e = new_up37_engine(2, 2026110704);
    e.debug_set_turn(0, Phase::Play, 1);

    // (a) jige handling a virtual card with empty physical subcards without panicking.
    set_flag(&mut e, 0, "jige_virtual_empty_subcards", 1);
    invoke_up37_op(&mut e, 0, "jige", JIGE_ID);
    assert_eq!(flag(&e, 0, "jige_resolved_safe"), 1);
    assert_eq!(flag(&e, 0, "jige_subcard_count"), 0);

    // (b) dcsbjuzhan respecting an abolished weapon slot (slot 1 / EquipSlot::Weapon).
    let weapon_card = e.debug_place(0, "zhuge");
    e.state_core_mut()
        .player_mut(0)
        .hand
        .retain(|&c| c != weapon_card);
    e.state_core_mut().deck.push(weapon_card);

    // Abolish P1's weapon slot (slot 1).
    set_flag(&mut e, 1, "equip_slot_disabled:1", 1);
    assert!(!has_enabled_equip_slot(
        e.state_core(),
        1,
        EquipSlot::Weapon
    ));
    set_flag(&mut e, 0, "dcsbjuzhan_target", 1);
    invoke_up37_op(&mut e, 0, "dcsbjuzhan", DCSBJUZHAN_ID);
    assert_eq!(flag(&e, 1, "dcsbjuzhan_blocked_by_abolished_slot"), 1);
    assert!(
        e.state_core().player(1).equips.is_empty(),
        "abolished weapon slot must block dcsbjuzhan weapon equip"
    );

    // Also verify EquipWeaponFromDrawPile respects abolished weapon slot.
    let out = e.debug_commit_effect(
        "dcsbjuzhan",
        1,
        EffectRequest::EquipWeaponFromDrawPile {
            player: PlayerId::new(2),
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert!(e.state_core().player(1).equips.is_empty());

    // Re-enable P1's weapon slot -> dcsbjuzhan equips the weapon.
    set_flag(&mut e, 1, "equip_slot_disabled:1", 0);
    assert!(has_enabled_equip_slot(e.state_core(), 1, EquipSlot::Weapon));
    invoke_up37_op(&mut e, 0, "dcsbjuzhan", DCSBJUZHAN_ID);
    assert_eq!(flag(&e, 1, "dcsbjuzhan_equipped"), 1);
    assert!(e.state_core().player(1).equips.contains(&weapon_card));
}

#[test]
fn up37_r07_jishe_temp_skill_persists_across_extra_phases_and_stardangchen_post_rewrite() {
    let mut e = new_up37_engine(2, 2026110705);
    e.debug_set_turn(0, Phase::Play, 1);

    // (a) Grant temp skill with expires_end_of_turn: true and queue an extra Play phase.
    set_flag(&mut e, 0, "jishe_temp_skill", STARBISHI_ID);
    set_flag(&mut e, 0, "jishe_queue_extra_play", 1);
    invoke_up37_op(&mut e, 0, "jishe", JISHE_ID);

    let starbishi_def = e.catalog().skill_by_name("starbishi").unwrap();
    assert!(e.state_core().has_skill(0, starbishi_def));
    assert_eq!(e.state_core().turn.extra_phases, vec![2]);

    // Ending the current Play phase pops extra_phases ([2] -> extra Play phase on seat 0).
    submit(&mut e, 0, GameAction::NextPhase); // -> pops extra Play phase on seat 0
    assert_eq!(e.state_core().turn.current, 0);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
    assert!(e.state_core().turn.extra_phases.is_empty());
    assert!(
        e.state_core().has_skill(0, starbishi_def),
        "temp skill with expires_end_of_turn must persist across queued extra phases"
    );

    // Finish the extra Play phase -> Discard -> End -> TurnEnd transitions to seat 1 and expires temp skill.
    submit(&mut e, 0, GameAction::NextPhase); // -> Discard
    submit(&mut e, 0, GameAction::NextPhase); // -> End
    assert!(
        e.state_core().has_skill(0, starbishi_def),
        "temp skill must still be active during End phase before TurnEnd"
    );
    submit(&mut e, 0, GameAction::NextPhase); // -> seat 1's turn
    assert_eq!(e.state_core().turn.current, 1);
    assert!(
        !e.state_core().has_skill(0, starbishi_def),
        "temp skill must expire when the turn truly ends"
    );

    // (b) stardangchen reads the post-rewrite judge result consistently across loop and post-judge check.
    let heart_card = (1..e.catalog().cards.len() as u32)
        .find(|&cid| e.catalog().cards[cid as usize].suit == Suit::Heart)
        .expect("heart card");
    set_flag(&mut e, 0, "stardangchen_rewritten_card", heart_card);
    let hand_before = e.state_core().player(0).hand.len();
    invoke_up37_op(&mut e, 0, "stardangchen", STARDANGCHEN_ID);
    assert_eq!(flag(&e, 0, "stardangchen_post_rewrite_consistent"), 1);
    assert_eq!(
        flag(&e, 0, "stardangchen_judge_suit"),
        flag(&e, 0, "stardangchen_loop_suit")
    );
    assert_eq!(flag(&e, 0, "stardangchen_judge_suit"), 2); // Heart
    assert_eq!(e.state_core().player(0).hand.len(), hand_before + 1);
}

#[test]
fn up37_r08_dcsbqingshi_dcsbzaoji_shenfu_and_olsbguqu() {
    let mut e = new_up37_engine(3, 2026110706);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1. dcsbqingshi: virtual sha on undamaged other + PhaseEnd counter-sha if target survives.
    let p1_hp_before = e.state_core().player(1).hp;
    invoke_up37_op(&mut e, 0, "dcsbqingshi", DCSBQINGSHI_ID);
    assert_eq!(flag(&e, 0, "dcsbqingshi_virtual_sha_used"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp_before - 1);

    let p0_hp_before = e.state_core().player(0).hp;
    set_flag(&mut e, 0, "dcsbqingshi_phase_end", 1);
    invoke_up37_op(&mut e, 0, "dcsbqingshi", DCSBQINGSHI_ID);
    assert_eq!(flag(&e, 0, "dcsbqingshi_counter_sha"), 1);
    assert_eq!(e.state_core().player(0).hp, p0_hp_before - 1);
    set_flag(&mut e, 0, "dcsbqingshi_phase_end", 0);

    // 2. dcsbzaoji: draw 3 & discard sha on damage_taken; discard sha for +1 damage on damage_caused.
    e.debug_clear_hand(0);
    let _sha0 = e.debug_place(0, "sha");
    invoke_up37_op(&mut e, 0, "dcsbzaoji", DCSBZAOJI_ID);
    assert_eq!(flag(&e, 0, "dcsbzaoji_damaged_draw3"), 1);
    assert!(e.state_core().player(0).hand.len() >= 2);

    let _sha1 = e.debug_place(0, "sha");
    set_flag(&mut e, 0, "dcsbzaoji_mode", 1);
    invoke_up37_op(&mut e, 0, "dcsbzaoji", DCSBZAOJI_ID);
    assert_eq!(flag(&e, 0, "dcsbzaoji_damage_boosted"), 1);
    set_flag(&mut e, 0, "dcsbzaoji_mode", 0);

    // 3. shenfu: odd hand -> thunder damage + repeat on kill; even hand -> draw 1.
    e.debug_clear_hand(0);
    let _c1 = e.debug_place(0, "shan");
    set_flag(&mut e, 0, "shenfu_repeat_on_kill", 1);
    let p1_hp = e.state_core().player(1).hp;
    let p2_hp = e.state_core().player(2).hp;
    invoke_up37_op(&mut e, 0, "shenfu", SHENFU_ID);
    assert_eq!(flag(&e, 0, "shenfu_odd_branch"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp - 1);
    assert_eq!(e.state_core().player(2).hp, p2_hp - 1);

    let _c2 = e.debug_place(0, "shan"); // even hand (2 cards)
    let p1_hand_before = e.state_core().player(1).hand.len();
    invoke_up37_op(&mut e, 0, "shenfu", SHENFU_ID);
    assert_eq!(flag(&e, 0, "shenfu_even_branch"), 1);
    assert_eq!(e.state_core().player(1).hand.len(), p1_hand_before + 1);

    // 4. olsbguqu: que card mark suit restriction + draw 2.
    e.debug_grant(0, "olsbguqu");
    e.debug_clear_hand(0);
    let spade_sha = (0..e.catalog().cards.len() as u32)
        .find(|&cid| {
            let def = &e.catalog().cards[cid as usize];
            def.suit == Suit::Spade && def.kind.is_sha_like()
        })
        .expect("spade sha");
    e.state_core_mut().player_mut(0).hand.push(spade_sha);
    set_flag(&mut e, 0, "olsbguqu_suit_bit", 1); // restrict Spade (bit 1)
    invoke_up37_op(&mut e, 0, "olsbguqu", OLSBGUQU_ID);
    assert_eq!(flag(&e, 0, "olsbguqu_drawn"), 2);
    assert!(
        eval_card_use_canceled(
            e.state_core(),
            e.catalog(),
            0,
            spade_sha,
            CardKind::Sha,
            DamageNature::Normal,
        ),
        "olsbguqu must restrict playing cards of the marked que suit"
    );
}

#[test]
fn up37_r09_dcsbtaozhou_dcxiangchen_and_potcuifeng() {
    let mut e = new_up37_engine(2, 2026110707);
    e.debug_set_turn(0, Phase::Play, 1);
    e.state_core_mut().player_mut(1).max_hp = 10;
    e.state_core_mut().player_mut(1).hp = 10;

    // 1. dcsbtaozhou: grants zijin unconditionally when diff >= 2 without canUse(sha) gate.
    // Force P1 to have sha_used_play_phase = true so P1 cannot use sha, and verify zijin is still granted.
    e.state_core_mut().turn.sha_used_play_phase = true;
    set_flag(&mut e, 0, "dcsbtaozhou_draw_count", 3);
    set_flag(&mut e, 0, "dcsbtaozhou_force_diff_ge_2", 1);
    invoke_up37_op(&mut e, 0, "dcsbtaozhou", DCSBTAOZHOU_ID);
    assert_eq!(flag(&e, 1, "zijin_granted"), 1);

    // 2. dcxiangchen: virtual card use (subcards = []), deducting mark count by cards_lost.len().
    e.state_core_mut()
        .vm_aux
        .marks
        .insert((0, DCXIANGCHEN_ID), 5);
    e.state_core_mut().vm_aux.lost.insert(1, vec![11, 12]);
    let p1_hp_before = e.state_core().player(1).hp;
    invoke_up37_op(&mut e, 0, "dcxiangchen", DCXIANGCHEN_ID);
    assert_eq!(flag(&e, 0, "dcxiangchen_virtual_used"), 1);
    assert_eq!(flag(&e, 0, "dcxiangchen_deducted"), 2);
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, DCXIANGCHEN_ID))
            .copied(),
        Some(3)
    );
    assert_eq!(e.state_core().player(1).hp, p1_hp_before - 1);

    // 3. potcuifeng: up to twice per round at TurnEnd if undamaged this turn, virtual juedou with +X damage bonus.
    e.state_core_mut().player_mut(0).turn_damage = 0;
    set_turn_var(&mut e, 0, "sha_played_this_turn", 2);
    let p1_hp_before_cf = e.state_core().player(1).hp;
    invoke_up37_op(&mut e, 0, "potcuifeng", POTCUIFENG_ID);
    assert_eq!(flag(&e, 0, "potcuifeng_damage_bonus"), 2);
    assert_eq!(
        e.state_core().player(1).hp,
        p1_hp_before_cf - 3,
        "potcuifeng must deal 1 + X (1 + 2 = 3) damage"
    );

    // Invoke a 2nd and 3rd time in the same round: 2nd succeeds, 3rd is capped at 2/round.
    let hp_after_cf1 = e.state_core().player(1).hp;
    set_turn_var(&mut e, 0, "sha_played_this_turn", 0);
    invoke_up37_op(&mut e, 0, "potcuifeng", POTCUIFENG_ID);
    assert_eq!(e.state_core().player(1).hp, hp_after_cf1 - 1);
    invoke_up37_op(&mut e, 0, "potcuifeng", POTCUIFENG_ID);
    assert_eq!(
        e.state_core().player(1).hp,
        hp_after_cf1 - 1,
        "potcuifeng must be capped at twice per round"
    );
}

#[test]
fn up37_n02_tangtang_zexing_and_tt_zhiyi() {
    let mut e = new_up37_engine(3, 2026110708);
    let g = find_general(&e, "tangtang");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    // Branch 1: show 1 card, view unvisited target's hand & gain up to 2 diff-suit cards + target gains shown card.
    // Because tangtang also has tt_zhiyi, the card movement triggers tt_zhiyi once per turn (drawing 1 for P2).
    let p2_hand_before = e.state_core().player(2).hand.len();
    set_flag(&mut e, 0, "tt_zhiyi_target", 2);
    let shown = e.debug_place(0, "sha");
    let c_spade = (0..e.catalog().cards.len() as u32)
        .find(|&cid| cid != shown && e.catalog().cards[cid as usize].suit == Suit::Spade)
        .unwrap();
    let c_heart = (0..e.catalog().cards.len() as u32)
        .find(|&cid| {
            cid != shown
                && cid != c_spade
                && e.catalog().cards[cid as usize].suit == Suit::Heart
        })
        .unwrap();
    e.state_core_mut().player_mut(1).hand = vec![c_spade, c_heart];
    set_flag(&mut e, 0, "zexing_branch", 1);
    invoke_up37_op(&mut e, 0, "zexing", ZEXING_ID);
    assert_eq!(flag(&e, 0, "zexing_branch1_gained"), 2);
    assert!(e.state_core().player(0).hand.contains(&c_spade));
    assert!(e.state_core().player(0).hand.contains(&c_heart));
    assert!(e.state_core().player(1).hand.contains(&shown));
    assert_eq!(turn_var(&e, 0, "tt_zhiyi_used"), 1);
    assert_eq!(e.state_core().player(2).hand.len(), p2_hand_before + 1);

    // Branch 2: target loses 1 HP & repeat on another unvisited player.
    set_turn_var(&mut e, 0, "zexing_visited:1", 0);
    set_turn_var(&mut e, 0, "zexing_visited:2", 0);
    set_flag(&mut e, 0, "zexing_branch", 2);
    set_flag(&mut e, 0, "zexing_repeat", 1);
    let p1_hp = e.state_core().player(1).hp;
    let p2_hp = e.state_core().player(2).hp;
    invoke_up37_op(&mut e, 0, "zexing", ZEXING_ID);
    assert_eq!(flag(&e, 0, "zexing_branch2_visits"), 2);
    assert_eq!(e.state_core().player(1).hp, p1_hp - 1);
    assert_eq!(e.state_core().player(2).hp, p2_hp - 1);

    // tt_zhiyi: second invocation in same turn is a no-op (once per turn), and works again after turn reset.
    invoke_up37_op(&mut e, 0, "tt_zhiyi", TT_ZHIYI_ID);
    assert_eq!(e.state_core().player(2).hand.len(), p2_hand_before + 1);
    set_turn_var(&mut e, 0, "tt_zhiyi_used", 0);
    invoke_up37_op(&mut e, 0, "tt_zhiyi", TT_ZHIYI_ID);
    assert_eq!(e.state_core().player(2).hand.len(), p2_hand_before + 2);
}

#[test]
fn up37_n03_tw_dm_zhangliang_twrenfang_twjuemie_twjianming_and_relonghun() {
    let mut e = new_up37_engine(2, 2026110709);
    let g = find_general(&e, "tw_dm_zhangliang");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    let base_hand_limit = e.state_core().hand_limit(e.catalog(), 0);
    let (base_sha_limit, _) = eval_sha_use_limit(
        e.state_core(),
        e.catalog(),
        0,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );

    // Accumulate 6 twrenfang marks -> +6 hand limit, +(6/5)=+1 slash limit.
    set_flag(&mut e, 0, "twrenfang_add", 6);
    invoke_up37_op(&mut e, 0, "twrenfang", TWRENFANG_ID);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, TWRENFANG_ID)).copied(),
        Some(6)
    );
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        base_hand_limit + 6
    );
    let (new_sha_limit, _) = eval_sha_use_limit(
        e.state_core(),
        e.catalog(),
        0,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    assert_eq!(new_sha_limit, base_sha_limit + 1);

    // twjuemie: consume 2 marks for 1 damage + 1 draw.
    let p1_hp = e.state_core().player(1).hp;
    let p0_hand = e.state_core().player(0).hand.len();
    set_flag(&mut e, 0, "twjuemie_consume", 2);
    invoke_up37_op(&mut e, 0, "twjuemie", TWJUEMIE_ID);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(0, TWRENFANG_ID)).copied(),
        Some(4)
    );
    assert_eq!(e.state_core().player(1).hp, p1_hp - 1);
    assert_eq!(e.state_core().player(0).hand.len(), p0_hand + 1);

    // twjianming & relonghun
    invoke_up37_op(&mut e, 0, "twjianming", TWJIANMING_ID);
    assert_eq!(flag(&e, 0, "twjianming_invoked"), 1);
    invoke_up37_op(&mut e, 0, "relonghun", RELONGHUN_ID);
    assert_eq!(flag(&e, 0, "relonghun_invoked"), 1);
}

#[test]
fn up37_n04_ol_sunhanhua_olhuaguang_and_olxuanbai_forms() {
    let mut e = new_up37_engine(2, 2026110710);
    let g = find_general(&e, "ol_sunhanhua");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    // Place cailian into weapon slot (1) -> dangmo form (1).
    set_flag(&mut e, 0, "olhuaguang_target_slot", 1);
    set_flag(&mut e, 0, "olhuaguang_target_suit", 2);
    invoke_up37_op(&mut e, 0, "olhuaguang", OLHUAGUANG_ID);
    assert_eq!(flag(&e, 0, "cailian_slot"), 1);
    assert_eq!(flag(&e, 0, "olxuanbai_form"), 1);

    // Place cailian into armor slot (2) -> jihui form (2).
    set_flag(&mut e, 0, "olhuaguang_target_slot", 2);
    invoke_up37_op(&mut e, 0, "olhuaguang", OLHUAGUANG_ID);
    assert_eq!(flag(&e, 0, "olxuanbai_form"), 2);

    // Place cailian into treasure slot (5) -> xiaju form (3) (+2 hand limit & draw 1 in Play phase).
    let base_hp = e.state_core().player(0).hp as usize;
    let hand_before = e.state_core().player(0).hand.len();
    set_flag(&mut e, 0, "olhuaguang_target_slot", 5);
    invoke_up37_op(&mut e, 0, "olhuaguang", OLHUAGUANG_ID);
    assert_eq!(flag(&e, 0, "olxuanbai_form"), 3);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before + 1);
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), base_hp + 2);
    let _ = OLXUANBAI_ID;
}

#[test]
fn up37_n05_ol_chitu_olfangma_and_oljingxian() {
    let mut e = new_up37_engine(2, 2026110711);
    let g = find_general(&e, "ol_chitu");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    let c1 = e.debug_place(0, "shan");
    let c2 = e.debug_place(0, "tao");
    // Start with chitu at index 0 (left=0 < right=2) -> double shan required.
    set_flag(&mut e, 0, "olfangma_chitu_index", 0);
    invoke_up37_op(&mut e, 0, "olfangma", OLFANGMA_ID);
    assert_eq!(flag(&e, 0, "olfangma_sha_double_shan"), 1);
    assert_eq!(flag(&e, 0, "olfangma_sha_unlimited"), 0);

    // Invoke oljingxian twice: moves chitu right 1 index each time, overtaken cards become sha,
    // and at rightmost (index 2 >= hand_len 2) left=2 > right=0 -> unlimited sha & extra turn at RoundEnd.
    invoke_up37_op(&mut e, 0, "oljingxian", OLJINGXIAN_ID);
    assert_eq!(flag(&e, 0, "oljingxian_overtaken_sha"), c1);
    assert_eq!(
        e.state_core().declared_kinds.get(&c1).copied(),
        Some(CardKind::Sha)
    );

    set_flag(&mut e, 0, "oljingxian_round_end", 1);
    invoke_up37_op(&mut e, 0, "oljingxian", OLJINGXIAN_ID);
    assert_eq!(flag(&e, 0, "oljingxian_overtaken_sha"), c2);
    assert_eq!(flag(&e, 0, "oljingxian_at_rightmost"), 1);
    assert_eq!(flag(&e, 0, "oljingxian_extra_turn_granted"), 1);
    assert_eq!(flag(&e, 0, "olfangma_sha_unlimited"), 1);
    let (_, unlimited) = eval_sha_use_limit(
        e.state_core(),
        e.catalog(),
        0,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    assert!(
        unlimited,
        "olfangma with left > right must grant unlimited sha"
    );
}

#[test]
fn up37_n06_ol_le_menghuo_oljunzhu_and_olrehuanshi() {
    let mut e = new_up37_engine(2, 2026110712);
    let g = find_general(&e, "ol_le_menghuo");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    // oljunzhu: deterministic mushroom roll (1 = draw 2, 3 = deal 1 damage).
    let hand_before = e.state_core().player(0).hand.len();
    set_flag(&mut e, 0, "oljunzhu_roll_override", 1);
    invoke_up37_op(&mut e, 0, "oljunzhu", OLJUNZHU_ID);
    assert_eq!(flag(&e, 0, "oljunzhu_mushroom"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before + 2);

    let p1_hp = e.state_core().player(1).hp;
    set_flag(&mut e, 0, "oljunzhu_roll_override", 3);
    invoke_up37_op(&mut e, 0, "oljunzhu", OLJUNZHU_ID);
    assert_eq!(flag(&e, 0, "oljunzhu_mushroom"), 3);
    assert_eq!(e.state_core().player(1).hp, p1_hp - 1);

    // olrehuanshi: once per suit per turn view-as.
    e.debug_clear_hand(0);
    let _c = e.debug_place(0, "sha");
    invoke_up37_op(&mut e, 0, "olrehuanshi", OLREHUANSHI_ID);
    assert_eq!(flag(&e, 0, "olrehuanshi_used_count"), 1);

    // Keep only the first card (same suit) and invoke again -> rejected for duplicate suit this turn.
    e.state_core_mut().player_mut(0).hand.truncate(1);
    invoke_up37_op(&mut e, 0, "olrehuanshi", OLREHUANSHI_ID);
    assert_eq!(flag(&e, 0, "olrehuanshi_rejected_duplicate_suit"), 1);
    assert_eq!(flag(&e, 0, "olrehuanshi_used_count"), 1);
}

#[test]
fn up37_n07_ol_xushao_olshilun_and_olzhuoming() {
    let mut e = new_up37_engine(2, 2026110713);
    let g = find_general(&e, "ol_xushao");
    e.debug_assemble_general_def(0, g);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1. Acquire a pingjian skill via olshilun (without immediate revoke), then transfer via olzhuoming on damage.
    invoke_up37_op(&mut e, 0, "olshilun", OLSHILUN_ID);
    let acquired = flag(&e, 0, "olshilun_acquired_skill");
    assert!(acquired >= 4_097_066_032);
    assert_eq!(flag(&e, 0, "pingjian_skill"), acquired);

    let hand_before_transfer = e.state_core().player(0).hand.len();
    invoke_up37_op(&mut e, 0, "olzhuoming", OLZHUOMING_ID);
    assert_eq!(flag(&e, 0, "pingjian_skill"), 0);
    assert_eq!(flag(&e, 1, "pingjian_skill"), acquired);
    assert_eq!(
        flag(&e, 0, "olzhuoming_draw_on_lose"),
        1,
        "losing/transferring a pingjian skill must trigger olzhuoming draw 1"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_transfer + 1
    );

    // 2. Acquire and immediately invoke + revoke at PhasePlay/PhaseEnd/DamageTaken -> triggers olzhuoming draw 1 again.
    set_flag(&mut e, 0, "olshilun_revoke_after_use", 1);
    invoke_up37_op(&mut e, 0, "olshilun", OLSHILUN_ID);
    assert_eq!(flag(&e, 0, "olshilun_revoked_skill"), acquired);
    assert_eq!(flag(&e, 0, "olzhuoming_draw_on_lose"), 2);
}

#[test]
fn up37_i02_death_flow_emits_die_audio_with_c29_skin_override_and_baseline_counts_hold() {
    // 1. Verify I02 baseline counts (2,487 missing audio files, 234 dynamic JS expressions, 18 non-voice assets).
    let (missing, dynamic_js, non_voice) =
        verify_up37_i02_baseline_counts().expect("I02 baseline counts must verify");
    assert_eq!(missing, 2_487);
    assert_eq!(dynamic_js, 234);
    assert_eq!(non_voice, 18);

    // 2. Verify DeathFlow (KillPlayer) emits PublicPresentationEvent::PlayerDied with die_audio,
    //    and C29 skin override (`c29_skin_id = 1`) selects the skin-specific die voice asset.
    let mut e = new_up37_engine(3, 2026110714);
    let g_tangtang = find_general(&e, "tangtang");
    let g_xushao = find_general(&e, "ol_xushao");
    e.debug_assemble_general_def(1, g_tangtang);
    e.debug_assemble_general_def(2, g_xushao);
    set_flag(&mut e, 2, "c29_skin_id", 1);

    let out1 = e.debug_commit_effect(
        "test",
        0,
        EffectRequest::KillPlayer {
            player: PlayerId::new(2),
            killer: PlayerId::new(1),
        },
    );
    assert!(!matches!(out1, CommandOutcome::Rejected(_)), "{out1:?}");
    e.release_committed_presentation();
    let batch1 = e
        .take_public_presentation()
        .expect("public presentation batch after first kill");

    let out2 = e.debug_commit_effect(
        "test",
        0,
        EffectRequest::KillPlayer {
            player: PlayerId::new(3),
            killer: PlayerId::new(1),
        },
    );
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "{out2:?}");
    e.release_committed_presentation();
    let batch2 = e
        .take_public_presentation()
        .expect("public presentation batch after second kill");

    let mut p1_die_voice = None;
    let mut p2_die_voice = None;
    for occ in batch1.occurrences.iter().chain(batch2.occurrences.iter()) {
        if let PublicPresentationEvent::PlayerDied { actor, voice, .. } = &occ.event {
            if actor.raw() == 2 {
                p1_die_voice = voice.clone();
            } else if actor.raw() == 3 {
                p2_die_voice = voice.clone();
            }
        }
    }
    let v1 = p1_die_voice.expect("P1 (tangtang) death must emit die_audio binding");
    let v2 = p2_die_voice.expect("P2 (ol_xushao with skin 1) death must emit die_audio binding");
    assert!(!v1.assets.is_empty());
    assert!(!v2.assets.is_empty());
    let ref1 = format!("{:?}", v1.assets[0].reference);
    let ref2 = format!("{:?}", v2.assets[0].reference);
    assert!(
        ref1.contains("die/tangtang"),
        "expected base die audio for tangtang, got {ref1}"
    );
    assert!(
        ref2.contains("skin/ol_xushao_skin1_die"),
        "C29 skin override on P2 must select skin 1 die asset, got {ref2}"
    );

    // Direct helper check for C29 skin override lookup.
    let binding = e
        .catalog()
        .general_presentation_binding(g_tangtang)
        .expect("tangtang presentation binding");
    let base_assets =
        resolve_binding_voice_with_skin_override(binding, "die", None, None).unwrap();
    let skin_assets =
        resolve_binding_voice_with_skin_override(binding, "die", Some(1), None).unwrap();
    assert_ne!(base_assets[0].digest, skin_assets[0].digest);
}

#[test]
fn up37_i03_step_to_async_commits_and_i05_presentation_bindings_hold() {
    // I03: 6 sp/guozhan/tw Step->Async commits preserve skill def_id and profile signatures in catalog.json.
    let verified_commits = verify_up37_i03_step_to_async_catalog_invariants()
        .expect("I03 Step->Async catalog invariants must hold");
    assert_eq!(verified_commits.len(), UP37_STEP_TO_ASYNC_COMMITS.len());
    for ((commit, _, _), &(expected_commit, pack, sym, expected_id)) in
        verified_commits.iter().zip(UP37_STEP_TO_ASYNC_COMMITS.iter())
    {
        assert_eq!(commit, expected_commit);
        assert!(verified_commits
            .iter()
            .any(|(_, ps, id)| ps == &format!("{pack}:{sym}") && *id == expected_id));
    }

    // I05: Portrait/audio/die_audio presentation bindings for N01..N07 and R07 (stardangchen).
    let catalog = build_up37_closure_catalog();
    verify_up37_i05_presentation_bindings(&catalog)
        .expect("I05 presentation bindings for N01..N07 and R07 must hold");
    assert_eq!(UP37_GENERALS.len(), 10);
}

#[test]
fn up37_r03_and_r04_cross_player_isolation_and_jsrgzonghai_dying_save_forbid() {
    // 1. starbishi must NOT trigger for P0 when P1 loses a judge card.
    let mut e = new_up37_engine(3, 2026110715);
    let g_dongzhuo = find_general(&e, "star_dongzhuo");
    e.debug_assemble_general_def(0, g_dongzhuo);
    e.debug_set_turn(0, Phase::Play, 1);
    let _p1_judge = e.debug_place_judge(1, "le");
    let p0_hand_before = e.state_core().player(0).hand.len();
    let out = e.debug_commit_effect(
        "test",
        1,
        EffectRequest::DiscardAllOwnJudge {
            player: PlayerId::new(2),
            skill: SkillId::new(STARBISHI_ID),
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert_eq!(
        turn_var(&e, 0, "starbishi_triggered"),
        0,
        "starbishi on P0 must not trigger when P1 loses a judge card"
    );
    assert_eq!(e.state_core().player(0).hand.len(), p0_hand_before);

    // 2. jsrgzonghai sets jsrgzonghai_forbid:{dying} and blocks non-self responder from offering tao in DyingSaveMachine.
    let mut e2 = new_up37_engine(3, 2026110716);
    e2.debug_grant(0, "jsrgzonghai");
    e2.debug_set_turn(0, Phase::Play, 1);
    let _p2_tao = e2.debug_place(2, "tao");
    // Inflict 4 damage on P1 so P1 enters Dying; P0 triggers jsrgzonghai and forbids non-self rescue of P1.
    let out2 = e2.debug_deal_damage(Some(0), 1, 4, DamageNature::Normal);
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "{out2:?}");
    assert_eq!(flag(&e2, 0, "jsrgzonghai_invoked"), 1);
    assert_eq!(flag(&e2, 1, "jsrgzonghai_forbid:1"), 1);
    // P2 has a tao, so DyingSaveMachine pauses at P2, but jsrgzonghai_forbid:1 filters out all Card options!
    let wait = e2.open_choice().expect("DyingSaveMachine waiting at P2");
    assert_eq!(wait.observer, 2);
    assert_eq!(
        wait.spec.options,
        vec![ChoiceOption::Pass],
        "jsrgzonghai_forbid:1 must forbid third-party P2 from offering tao to save P1"
    );
    let choice_id = wait.id;
    submit(
        &mut e2,
        2,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert!(!e2.state_core().player(1).alive, "P1 must die when rescue is forbidden");
}

#[test]
fn up37_r06_jige_mutual_sha_discard_and_dcsbjuzhan_multi_slot_disable_and_replace() {
    let mut e = new_up37_engine(2, 2026110717);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1. jige mutual sha discard: P0 has 2 sha, P1 has 1 sha -> both discard all sha, P0 deals 1 damage to P1.
    let _s0_a = e.debug_place(0, "sha");
    let _s0_b = e.debug_place(0, "sha");
    let _s1_a = e.debug_place(1, "sha");
    let p1_hp_before = e.state_core().player(1).hp;
    set_flag(&mut e, 0, "jige_mutual_sha_discard", 1);
    invoke_up37_op(&mut e, 0, "jige", JIGE_ID);
    assert_eq!(flag(&e, 0, "jige_owner_sha_discarded"), 2);
    assert_eq!(flag(&e, 0, "jige_target_sha_discarded"), 1);
    assert_eq!(flag(&e, 0, "jige_diff_damage"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp_before - 1);

    // 2. dcsbjuzhan with bit-0 mask (disabled_equip_slots = 1) and multi-slot DisableEquipSlots (weapon + armor).
    set_flag(&mut e, 1, "disabled_equip_slots", 1);
    assert!(
        !has_enabled_equip_slot(e.state_core(), 1, EquipSlot::Weapon),
        "bit 0 (disabled_equip_slots = 1) must disable weapon slot"
    );
    set_flag(&mut e, 1, "disabled_equip_slots", 0);

    let out = e.debug_commit_effect(
        "dcsbjuzhan",
        0,
        EffectRequest::DisableEquipSlots {
            player: PlayerId::new(2),
            slots: vec![1, 2],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert!(!has_enabled_equip_slot(e.state_core(), 1, EquipSlot::Weapon));
    assert!(!has_enabled_equip_slot(e.state_core(), 1, EquipSlot::Armor));
    assert!(has_enabled_equip_slot(e.state_core(), 1, EquipSlot::Treasure));

    set_flag(&mut e, 0, "dcsbjuzhan_target", 1);
    invoke_up37_op(&mut e, 0, "dcsbjuzhan", DCSBJUZHAN_ID);
    assert_eq!(flag(&e, 1, "dcsbjuzhan_blocked_by_abolished_slot"), 1);

    // Re-enable slots and verify weapon replacement + clearing dcsbjuzhan_blocked_by_abolished_slot.
    set_flag(&mut e, 1, "disabled_equip_slots", 0);
    set_flag(&mut e, 1, "equip_slot_disabled:1", 0);
    set_flag(&mut e, 1, "equip_slot_disabled:2", 0);
    let w1 = e.debug_place(0, "zhuge");
    let w2 = e.debug_place(0, "qinggang");
    e.state_core_mut()
        .player_mut(0)
        .hand
        .retain(|&c| c != w1 && c != w2);
    e.state_core_mut().deck.push(w2);
    e.state_core_mut().deck.push(w1);

    invoke_up37_op(&mut e, 0, "dcsbjuzhan", DCSBJUZHAN_ID);
    assert_eq!(flag(&e, 1, "dcsbjuzhan_blocked_by_abolished_slot"), 0);
    assert_eq!(e.state_core().player(1).equips, vec![w1]);

    invoke_up37_op(&mut e, 0, "dcsbjuzhan", DCSBJUZHAN_ID);
    assert_eq!(
        e.state_core().player(1).equips,
        vec![w2],
        "equipping a second weapon via dcsbjuzhan must replace the existing weapon"
    );
    assert!(e.state_core().discard.contains(&w1));
}

#[test]
fn up37_r09_potcuifeng_real_turn_end_window_and_damaged_this_turn_gate() {
    // 1. Undamaged turn: P0 plays 2 sha in Play phase, then advances via NextPhase to real TurnEndWindow.
    let mut e = new_up37_engine(2, 2026110718);
    e.debug_grant(0, "potcuifeng");
    e.debug_set_turn(0, Phase::Play, 1);
    e.state_core_mut().player_mut(1).max_hp = 10;
    e.state_core_mut().player_mut(1).hp = 10;

    set_turn_var(&mut e, 0, "sha_played_this_turn", 2);
    submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let p1_hp_before_end = e.state_core().player(1).hp;
    submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow (potcuifeng triggers!) -> P1's turn
    assert_eq!(e.state_core().turn.current, 1);
    assert_eq!(flag(&e, 0, "potcuifeng_damage_bonus"), 2);
    assert_eq!(
        e.state_core().player(1).hp,
        p1_hp_before_end - 3,
        "potcuifeng must trigger at real TurnEndWindow and deal 1 + 2 = 3 damage before turn handoff clears sha_played_this_turn"
    );

    // 2. Damaged turn: P0 takes 1 damage during turn -> damaged_this_turn > 0 blocks potcuifeng at TurnEndWindow.
    let mut e2 = new_up37_engine(2, 2026110719);
    e2.debug_grant(0, "potcuifeng");
    e2.debug_set_turn(0, Phase::Play, 1);
    set_turn_var(&mut e2, 0, "sha_played_this_turn", 2);
    let out = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert!(
        turn_var(&e2, 0, "damaged_this_turn") >= 1,
        "commit_damage must record damaged_this_turn on potcuifeng holder"
    );
    let p1_hp = e2.state_core().player(1).hp;
    submit(&mut e2, 0, GameAction::NextPhase); // Play -> Discard
    submit(&mut e2, 0, GameAction::NextPhase); // Discard -> End
    submit(&mut e2, 0, GameAction::NextPhase); // End -> TurnEndWindow -> P1's turn
    assert_eq!(
        e2.state_core().player(1).hp,
        p1_hp,
        "potcuifeng must not deal damage at TurnEndWindow when owner took damage this turn"
    );
}

#[test]
fn up37_n03_n05_and_ngsa_107_save_restore_roundtrip() {
    let mut e = new_up37_engine(2, 2026110720);
    let g_zhangliang = find_general(&e, "tw_dm_zhangliang");
    let g_chitu = find_general(&e, "ol_chitu");
    e.debug_assemble_general_def(0, g_chitu);
    e.debug_assemble_general_def(1, g_zhangliang);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1. twrenfang deduplication: discarding 2 cards in Discard phase must add exactly +2 marks (not double-counted).
    let d1 = e.debug_place(1, "shan");
    let d2 = e.debug_place(1, "tao");
    let out_disc = e.debug_commit_effect(
        "test",
        1,
        EffectRequest::DiscardAllHandAndEquip {
            player: PlayerId::new(2),
        },
    );
    assert!(!matches!(out_disc, CommandOutcome::Rejected(_)), "{out_disc:?}");
    assert!(e.state_core().discard.contains(&d1) && e.state_core().discard.contains(&d2));
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(1, TWRENFANG_ID)).copied(),
        Some(2),
        "twrenfang must count each discarded card exactly once across CardsDiscarded + CardsLost"
    );

    // 2. olfangma combat double-shan requirement in ShaResolveMachine + left == right flag reset.
    // P0 starts with 4 cards (sha_card + 3 others) at chitu_index = 0. When sha_card is used,
    // 3 cards remain in hand and oljingxian moves chitu_index from 0 -> 1 (left = 1 < right = 2),
    // keeping olfangma_sha_double_shan == 1 during ShaResolveMachine.
    let sha_card = e.debug_place(0, "sha");
    let keep_card = e.debug_place(0, "tao");
    let extra_card = e.debug_place(0, "shan");
    let _extra_card2 = e.debug_place(0, "jiu");
    let _p1_only_one_shan = e.debug_place(1, "shan");
    set_flag(&mut e, 0, "olfangma_chitu_index", 0);
    invoke_up37_op(&mut e, 0, "olfangma", OLFANGMA_ID);
    assert_eq!(flag(&e, 0, "olfangma_sha_double_shan"), 1);

    // When P0 plays sha against P1 (who only has 1 shan), P1 responds with 1 shan and still needs a 2nd shan!
    let p1_hp_before = e.state_core().player(1).hp;
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_card,
            targets: vec![1],
        },
    );
    // P1 is asked for 1st shan -> plays shan -> asked for 2nd shan (only Pass available) -> takes 1 damage.
    let first_wait = e.open_choice().expect("1st shan prompt").clone();
    let shan_opt = first_wait
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(_)))
        .cloned()
        .expect("P1 has 1 shan option");
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: first_wait.id,
            selection: vec![shan_opt],
        },
    );
    if let Some(second_wait) = e.open_choice().cloned() {
        assert_eq!(second_wait.observer, 1);
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: second_wait.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(
        e.state_core().player(1).hp,
        p1_hp_before - 1,
        "olfangma_sha_double_shan must require 2 shan cards in ShaResolveMachine"
    );

    // Also verify left == right (4 cards in hand, chitu_index = 2 -> left=2, right=2) clears both flags.
    let _extra_card3 = e.debug_place(0, "tao");
    assert_eq!(e.state_core().player(0).hand.len(), 4);
    set_flag(&mut e, 0, "olfangma_chitu_index", 2);
    invoke_up37_op(&mut e, 0, "olfangma", OLFANGMA_ID);
    assert_eq!(flag(&e, 0, "olfangma_sha_unlimited"), 0);
    assert_eq!(flag(&e, 0, "olfangma_sha_double_shan"), 0);

    // 3. NGSA 107 save -> encode_archive -> decode_archive -> restore_with_extensions mid-turn roundtrip after oljingxian declared_kinds & extra_phases.
    set_flag(&mut e, 0, "olfangma_chitu_index", 1);
    invoke_up37_op(&mut e, 0, "oljingxian", OLJINGXIAN_ID);
    e.state_core_mut().turn.extra_phases.push(2);
    let expected_hash = e.state_hash();
    let snap = e.save();
    let archive_bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&archive_bytes)
        .expect("NGSA 107 decode must succeed");
    let pack = up37_closure_extension_pack();
    let restored = Engine::restore_with_extensions(&decoded, &[&pack])
        .expect("NGSA 107 restore with UP37 extension pack must succeed");
    assert_eq!(restored.state_hash(), expected_hash);
    assert_eq!(
        restored.state_core().declared_kinds.get(&extra_card).copied(),
        Some(CardKind::Sha)
    );
    assert_eq!(restored.state_core().turn.extra_phases, vec![2]);
    let _ = keep_card;
}

