use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome};

#[test]
fn l11_vm_fix_b_qingxi_discard_vs_plus1_damage() {
    let mut e = start_4p_taixu(2026_8101);
    e.debug_grant(0, "qingxi");
    // Equip qinggang (range 2) on seat 0
    let qg = e.debug_place(0, "qinggang");
    assert!(!matches!(play_card(&mut e, 0, qg, vec![]), CommandOutcome::Rejected(_)));
    let sha1 = e.debug_place(0, "sha");
    let d1 = e.debug_place(1, "shan");
    let d2 = e.debug_place(1, "shan");
    let hp1_b = e.debug_hp(1);

    // Seat 1 has NO shan? No, give 2 junk cards and decline Shan -> qingxi asks confirm -> seat 1 discards 2 hand cards -> weapon discarded, dmg stays 1
    e.debug_clear_hand(1);
    let j1 = e.debug_place(1, "sha");
    let j2 = e.debug_place(1, "sha");
    let _ = (d1, d2);
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    let _ = expect_prompt(&e, "trigger_confirm");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let _ = expect_prompt(&e, "l11_vm_b:qingxi_discard");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(j1), ChoiceOption::Card(j2)]);
    assert_eq!(e.debug_hp(1), hp1_b - 1, "qingxi discard keeps damage 1");
    assert!(e.debug_equips(0).is_empty(), "qingxi discard strips owner weapons");

    // Equip range-2 weapon again; target declines discard -> damage +1 (2 total)!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let qg2 = e.debug_place(0, "qinggang");
    let _ = play_card(&mut e, 0, qg2, vec![]);
    let sha2 = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    let k1 = e.debug_place(1, "sha");
    let k2 = e.debug_place(1, "sha");
    let hp1_b2 = e.debug_hp(1);
    let _ = (k1, k2);
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let _ = expect_prompt(&e, "l11_vm_b:qingxi_discard");
    let _ = answer(&mut e, 1, Vec::new()); // pass
    assert_eq!(e.debug_hp(1), hp1_b2 - 2, "declining qingxi adds +1 damage");
    assert!(!e.debug_equips(0).is_empty());
}

#[test]
fn l11_vm_fix_b_rebeige_all_four_judge_suits() {
    // Global trigger on seat 2 when seat 0 Shas seat 1!
    let mut e = start_4p_taixu(2026_8102);
    e.debug_grant(2, "rebeige");
    e.debug_grant(0, "咆哮");

    // 1. Heart -> victim (seat 1) recovers 1 HP
    let sha_h = e.debug_place(0, "sha");
    let bg_card1 = e.debug_place(2, "sha");
    e.debug_set_hp(1, 3);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let _ = play_card(&mut e, 0, sha_h, vec![1]);
    let _ = expect_prompt(&e, "l11_vm_b:rebeige_discard");
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(bg_card1)]);
    assert_eq!(e.debug_hp(1), 3, "rebeige Heart recovers 1 after taking 1 damage");

    // 2. Diamond -> victim (seat 1) draws 3
    let sha_d = e.debug_place(0, "sha");
    let bg_card2 = e.debug_place(2, "sha");
    e.debug_clear_hand(1);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Diamond);
    let _ = play_card(&mut e, 0, sha_d, vec![1]);
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(bg_card2)]);
    assert_eq!(e.debug_hand(1).len(), 3, "rebeige Diamond must draw 3 (not 2)");

    // 3. Club -> source (seat 0) discards 2
    e.debug_clear_hand(0);
    let s1 = e.debug_place(0, "sha");
    let s2 = e.debug_place(0, "sha");
    let s3 = e.debug_place(0, "sha");
    let sha_c = e.debug_place(0, "sha");
    let bg_card3 = e.debug_place(2, "sha");
    e.debug_clear_hand(1);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let _ = play_card(&mut e, 0, sha_c, vec![1]);
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(bg_card3)]);
    let _ = expect_prompt(&e, "l11_vm_b:rebeige_source_discard");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(s1), ChoiceOption::Card(s2)]);
    assert_eq!(e.debug_hand(0), vec![s3]);

    // 4. Spade -> source (seat 0) turns over!
    e.debug_set_hp(1, 4);
    assert!(!e.debug_face_down(0));
    let sha_s = e.debug_place(0, "sha");
    let bg_card4 = e.debug_place(2, "sha");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let _ = play_card(&mut e, 0, sha_s, vec![1]);
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(bg_card4)]);
    assert!(e.debug_face_down(0), "rebeige Spade must turn over source");
}

#[test]
fn l11_vm_fix_b_rejianxiong_and_rejieming_parity() {
    // rejianxiong is OPTIONAL (can pass or confirm to gain dmg card + draw 1)
    let mut e = start_4p_taixu(2026_8103);
    e.debug_grant(1, "rejianxiong");
    e.debug_grant(1, "rejieming");
    e.debug_grant(0, "咆哮");

    let sha1 = e.debug_place(0, "sha");
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    for _ in 0..6 {
        let Some(ch) = e.open_choice() else {
            break;
        };
        match ch.spec.prompt {
            "trigger_order" => {
                let first = ch.spec.options[0];
                let _ = answer(&mut e, 1, vec![first]);
            }
            "trigger_confirm" => {
                let _ = answer(&mut e, 1, vec![ChoiceOption::Confirm]); // rejianxiong
            }
            "l11_vm_b:rejieming_target" => {
                let _ = answer(&mut e, 1, vec![ChoiceOption::Player(1)]);
            }
            _ => break,
        }
    }
    // rejianxiong gained sha1 + drew 1 = 2 cards, PLUS rejieming drew 2 (+ owner drew 1 since 4 < max_hp=5)
    assert!(e.debug_hand(1).contains(&sha1));
    assert!(e.debug_hand(1).len() >= 4);

    // Next: jiu-buffed Sha (dmg = 2) -> rejieming loops TWICE!
    advance_to_play(&mut e, 0);
    e.debug_remove_skill(1, "rejianxiong");
    e.debug_clear_hand(2);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(2, 5);
    let jiu = e.debug_place(0, "jiu");
    let _ = play_card(&mut e, 0, jiu, vec![]);
    let sha2 = e.debug_place(0, "sha");
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    // Step 1 of rejieming (dmg=2): seat 2 draws 2 -> 2 < 5 -> seat 1 draws 1!
    let _ = expect_prompt(&e, "l11_vm_b:rejieming_target");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    // Step 2 of rejieming: seat 2 draws 2 -> 4 < 5 -> seat 1 draws 1 more!
    let _ = expect_prompt(&e, "l11_vm_b:rejieming_target");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hand(2).len(), 4);
    assert_eq!(e.debug_hand(1).len(), 2);
}

#[test]
fn l11_vm_fix_b_releiji_club_and_spade_and_reweimu() {
    // 1. releiji Club: 1 thunder dmg + owner recovers 1; Spade: 2 thunder dmg
    let mut e = start_4p_taixu(2026_8104);
    e.debug_grant(1, "releiji");
    e.debug_grant(0, "咆哮");
    e.debug_set_hp(1, 2);
    let sha1 = e.debug_place(0, "sha");
    let shan1 = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let hp2_b = e.debug_hp(2);
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    // Seat 1 answers Shan
    let _ch = e.open_choice().unwrap();
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(shan1)]);
    let _ = expect_prompt(&e, "l11_vm_b:releiji_target");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(2), hp2_b - 1, "releiji Club deals 1 thunder");
    assert_eq!(e.debug_hp(1), 3, "releiji Club recovers 1 HP for owner");

    // Spade judge -> 2 thunder dmg!
    let sha2 = e.debug_place(0, "sha");
    let shan2 = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let hp2_b2 = e.debug_hp(2);
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(shan2)]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(2), hp2_b2 - 2, "releiji Spade deals 2 thunder");

    // 2. reweimu: black trick target immunity + own-turn damage prevent & draw 2*amount
    let mut e2 = start_4p_taixu(2026_8105);
    e2.debug_grant(1, "reweimu");
    let black_dism = place_matching(&mut e2, 0, |d| d.kind == CardKind::Dismantle && matches!(d.suit, Suit::Spade | Suit::Club));
    let _ = e2.debug_place(1, "sha");
    assert!(
        matches!(play_card(&mut e2, 0, black_dism, vec![1]), CommandOutcome::Rejected(_)),
        "reweimu must forbid black trick targeting owner"
    );
    let red_dism = place_matching(&mut e2, 0, |d| d.kind == CardKind::Dismantle && matches!(d.suit, Suit::Heart | Suit::Diamond));
    assert!(!matches!(
        play_card(&mut e2, 0, red_dism, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e2, 12);

    // Own turn damage prevented + draw 2*amount (use jiu+duel self-harm / opponent fankui/ganglie / damage test via Duel!)
    advance_to_play(&mut e2, 1);
    e2.debug_grant(2, "quhu");
    e2.debug_clear_hand(1);
    let duel = place_matching(&mut e2, 1, |d| d.kind == CardKind::Duel && matches!(d.suit, Suit::Heart | Suit::Diamond));
    let _s2 = e2.debug_place(2, "sha");
    // Seat 1 duels seat 2, seat 2 plays Sha, seat 1 declines -> seat 1 takes 1 dmg ON SEAT 1's OWN TURN -> reweimu prevents & draws 2!
    let hp1_b = e2.debug_hp(1);
    let _ = play_card(&mut e2, 1, duel, vec![2]);
    drain_open_with_ai(&mut e2, 16);
    assert_eq!(e2.debug_hp(1), hp1_b, "reweimu must prevent own-turn damage");
    assert_eq!(e2.debug_hand(1).len(), 2, "reweimu must draw 2 * amount");
}

#[test]
fn l11_vm_fix_b_reshejian_spchijie_spdengli_sphuangen_xinliegong_yiyong_zhuandui() {
    // 1. spchijie: judge > 6 cancels incoming single-target card (once per turn!)
    let mut e = start_4p_taixu(2026_8106);
    e.debug_grant(1, "spchijie");
    e.debug_grant(0, "咆哮");
    let sha1 = e.debug_place(0, "sha");
    e.debug_set_deck_top_by(|d| d.rank == 7);
    let hp1_b = e.debug_hp(1);
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    let _ = expect_prompt(&e, "trigger_confirm");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hp(1), hp1_b, "spchijie rank>6 must cancel Sha");
    // 2nd card same turn -> spchijie already used -> does NOT ask again!
    let sha2 = e.debug_place(0, "sha");
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    assert!(e.open_choice().is_none(), "spchijie is usable 1/turn");
    assert_eq!(e.debug_hp(1), hp1_b - 1);

    // 2. spdengli & xinliegong & zhuandui & reshejian
    let mut e2 = start_4p_taixu(2026_8107);
    e2.debug_grant(0, "spdengli");
    e2.debug_grant(0, "xinliegong");
    e2.debug_grant(0, "zhuandui");
    e2.debug_grant(1, "reshejian");
    e2.debug_set_hp(0, 4);
    e2.debug_set_hp(1, 4); // equal HP -> spdengli AND xinliegong(+dmg)!
    let s0 = place_suit_rank(&mut e2, 0, Suit::Spade, 13); // big pindian & distance 13
    let c0_h1 = place_matching(&mut e2, 0, |d| d.kind == CardKind::Sha && d.suit == Suit::Heart);
    let r1_a = place_suit_rank(&mut e2, 1, Suit::Club, 2); // small pindian card kept after reshejian!
    let r1_b = e2.debug_place(1, "shan");
    let r1_c = e2.debug_place(1, "shan");
    let _ = play_card(&mut e2, 0, c0_h1, vec![1]);
    // Trigger windows on CardUsed:
    // reshejian: seat 1 discards r1_b, r1_c -> deals 1 dmg to seat 0!
    // Check prompt order/names cleanly by responding per open observer:
    let planner2 = engine_native::ai::BoundedPlanner::new(16);
    let mut seen_prompts = Vec::new();
    for _ in 0..16 {
        let Some(ch) = e2.open_choice() else {
            break;
        };
        let p = ch.spec.prompt;
        let obs = ch.observer;
        seen_prompts.push((p, obs));
        if p == "l11_vm_b:reshejian_discard" {
            let out = answer(&mut e2, obs, vec![ChoiceOption::Card(r1_b), ChoiceOption::Card(r1_c)]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "reshejian_discard rejected: {out:?}");
        } else if p == "l11_vm_b:reshejian_mode" {
            let out = answer(&mut e2, obs, vec![ChoiceOption::Tag(1)]); // 1 dmg to seat 0
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "reshejian_mode rejected: {out:?}");
        } else if p == "trigger_order" {
            let first = ch.spec.options[0];
            let out = answer(&mut e2, obs, vec![first]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "trigger_order rejected: {out:?}");
        } else if p == "trigger_confirm" {
            let out = answer(&mut e2, obs, vec![ChoiceOption::Confirm]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "trigger_confirm rejected: {out:?}");
        } else if p == "l11_vm_b:zhuandui_confirm" {
            let out = answer(&mut e2, obs, vec![ChoiceOption::Confirm]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "zhuandui_confirm rejected: {out:?}");
        } else if p == "l11_vm_b:pindian_card" || p == "l11_vm_b:target_pindian_card" {
            let card_opt = if obs == 0 {
                ChoiceOption::Card(s0)
            } else {
                ChoiceOption::Card(r1_a)
            };
            let out = answer(&mut e2, obs, vec![card_opt]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "pindian rejected: {out:?}");
        } else {
            let act = e2.plan_action(obs, &planner2).action;
            let out = submit(&mut e2, obs, act);
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "fallback {p} rejected: {out:?}");
        }
    }
    assert!(
        e2.debug_hp(0) < 4,
        "reshejian Tag(1) dealt 1 dmg to seat 0 (prompts: {seen_prompts:?}, hand0: {:?}, hand1: {:?})",
        e2.debug_hand(0),
        e2.debug_hand(1)
    );
    assert!(e2.debug_hp(1) < 4, "xinliegong hit seat 1 unrespondably (+dmg)");

    // 3. sphuangen: ordinary multi-target trick (tiaojiyanmei / tiesuo) excludes 1 target & owner draws 1
    let mut e3 = start_4p_taixu(2026_8108);
    e3.debug_grant(2, "sphuangen");
    let ts = place_matching(&mut e3, 0, |d| d.kind == CardKind::Chain);
    let _ = play_card(&mut e3, 0, ts, vec![1, 3]);
    let _ = expect_prompt(&e3, "l11_vm_b:sphuangen_targets");
    let _ = answer(&mut e3, 2, vec![ChoiceOption::Player(1)]);
    drain_open_with_ai(&mut e3, 12);
    assert!(!e3.state_core().player(1).linked, "seat 1 excluded by sphuangen");
    assert!(e3.state_core().player(3).linked, "seat 3 still linked");
    assert_eq!(e3.debug_hand(2).len(), 1, "sphuangen owner draws 1");

    // 4. yiyong: Sha damage -> defender with Weapon gains Sha & plays on source (+1 dmg if source has no weapon!)
    let mut e4 = start_4p_taixu(2026_8109);
    e4.debug_grant(1, "yiyong");
    let wep = e4.debug_place(1, "qinggang");
    advance_to_play(&mut e4, 1);
    let _ = play_card(&mut e4, 1, wep, vec![]);
    advance_to_play(&mut e4, 0);
    e4.debug_clear_hand(0);
    e4.debug_clear_hand(1);
    let sha_from_0 = e4.debug_place(0, "sha");
    let hp0_b = e4.debug_hp(0);
    let _ = play_card(&mut e4, 0, sha_from_0, vec![1]);
    let _ = expect_prompt(&e4, "trigger_confirm");
    let _ = answer(&mut e4, 1, vec![ChoiceOption::Confirm]);
    assert_eq!(e4.debug_hp(0), hp0_b - 2, "yiyong counter-Sha must deal +1 dmg when source has no weapon");
}

#[test]
fn l11_vm_fix_b_r2_rejianxiong_cardless_and_zhuandui_txyonglve() {
    // 1. rejianxiong must NOT trigger or draw on cardless damage (shenfen)
    let mut e = start_4p_taixu(2026_8212);
    e.debug_grant(0, "shenfen");
    e.debug_grant(1, "rejianxiong");
    e.state_core_mut().vm_aux.marks.insert((0, 1_252_226_867), 6);
    assert!(!matches!(
        activate_skill(&mut e, 0, "shenfen"),
        CommandOutcome::Rejected(_)
    ));
    assert!(
        e.open_choice().is_none(),
        "rejianxiong must not prompt on cardless shenfen damage"
    );
    assert_eq!(e.debug_hp(1), 3, "shenfen dealt 1 cardless dmg");
    assert!(
        e.debug_hand(1).is_empty(),
        "rejianxiong must not draw on cardless damage"
    );
    let sha_real = e.debug_place(0, "sha");
    let _ = play_card(&mut e, 0, sha_real, vec![1]);
    let _ = expect_prompt(&e, "trigger_confirm");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Confirm]);
    assert!(e.debug_hand(1).contains(&sha_real), "rejianxiong gains table/discard Sha");
    assert_eq!(e.debug_hand(1).len(), 2, "rejianxiong gains Sha + draws 1");

    // 2. zhuandui with empty hand + txyonglve (attacker empty after playing last Sha)
    let mut e2 = start_4p_taixu(2026_8213);
    e2.debug_grant(0, "zhuandui");
    e2.debug_grant(0, "txyonglve");
    let sha_last = e2.debug_place(0, "sha");
    let r1 = place_suit_rank(&mut e2, 1, Suit::Club, 2);
    let _shan1 = e2.debug_place(1, "shan");
    let _top12 = place_suit_rank(&mut e2, 3, Suit::Heart, 12);
    // Put rank-12 back on deck top for txyonglve!
    e2.state_core_mut().player_mut(3).hand.retain(|&c| c != _top12);
    e2.state_core_mut().deck.push(_top12);
    let _ = r1;
    let out = play_card(&mut e2, 0, sha_last, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    let _ = expect_prompt(&e2, "l11_vm_b:zhuandui_confirm");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    let _ = expect_prompt(&e2, "txyonglve:confirm");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    // Target chooses pindian card (r1=2 vs deck top=12 -> seat 0 wins -> Sha unrespondable!)
    drain_open_with_ai(&mut e2, 8);
    assert_eq!(
        e2.debug_hp(1),
        3,
        "empty-hand attacker zhuandui via txyonglve must hit unrespondably"
    );

    // 3. zhuandui with empty hand + txyonglve on DEFENDING target
    let mut e3 = start_4p_taixu(2026_8214);
    e3.debug_grant(1, "zhuandui");
    e3.debug_grant(1, "txyonglve");
    let sha0 = e3.debug_place(0, "sha");
    let _r0 = place_suit_rank(&mut e3, 0, Suit::Club, 2);
    let top13 = place_suit_rank(&mut e3, 3, Suit::Spade, 13);
    e3.state_core_mut().player_mut(3).hand.retain(|&c| c != top13);
    e3.state_core_mut().deck.push(top13);
    let _ = play_card(&mut e3, 0, sha0, vec![1]);
    let _ = expect_prompt(&e3, "l11_vm_b:zhuandui_confirm");
    let _ = answer(&mut e3, 1, vec![ChoiceOption::Confirm]);
    let _ = expect_prompt(&e3, "txyonglve:confirm");
    let _ = answer(&mut e3, 1, vec![ChoiceOption::Confirm]);
    assert_eq!(
        e3.debug_hp(1),
        4,
        "empty-hand target zhuandui via txyonglve must cancel Sha"
    );
}
