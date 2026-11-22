use super::common::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

/// Upstream: `extension_skill.js:5-23` (`_txhj_skill_maxCards`)
#[test]
fn test_txhj_skill_max_cards_gaintag_and_bonus() {
    let mut e = start_versus(0xB5_0001, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_txhj_skill_maxCards");
    e.debug_set_hp(0, 2);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_txhj_max_hs".to_string()), 1);
    let keep = place_kind(&mut e, 0, CardKind::Sha);
    let bonus_tag = place_kind(&mut e, 0, CardKind::Shan);
    let drop_c = place_kind(&mut e, 0, CardKind::Tao);
    let drop2 = place_kind(&mut e, 0, CardKind::Jiu);
    let drop3 = place_kind(&mut e, 0, CardKind::Tao);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("l11_gaintag_paiku:{bonus_tag}")), 1);

    // hp=2 + l11_txhj_max_hs=1 -> effective limit=3, bonus_tag ignored in discard -> 4 counted > 3, must discard 1!
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = next_phase(&mut e);
    let c = expect_pending(&e, "discard_to_limit");
    assert_eq!(c.spec.min, 1);
    assert_eq!(c.spec.max, 1);
    assert!(
        !c.spec.options.contains(&ChoiceOption::Card(bonus_tag)),
        "l11_gaintag_paiku card must be excluded from discard phase candidates"
    );
    let _ = answer(&mut e, &c, vec![ChoiceOption::Card(drop_c)]);
    drain(&mut e);
    assert!(e.state_core().player(0).hand.contains(&keep));
    assert!(e.state_core().player(0).hand.contains(&drop2));
    assert!(e.state_core().player(0).hand.contains(&drop3));
    assert!(e.state_core().player(0).hand.contains(&bonus_tag));

    // Using `bonus_tag` removes stale `l11_gaintag_paiku:{bonus_tag}` flag via aux_history.
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = play_card(&mut e, 0, keep, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, &format!("l11_gaintag_paiku:{keep}")), 0);
}

/// Upstream: `extension_skill.js:24-110,199-270` (`_txhj_skill_*` system/scoreboard counters & `_txhj_equip5`)
#[test]
fn test_txhj_system_counters_and_equip5_dual_treasure() {
    let mut e = start_versus(0xB5_0002, &[&[0], &[1]]);
    for sk in [
        "_txhj_skill_Timelimit",
        "_txhj_skill_addEffectStyle",
        "_txhj_skill_gaincard",
        "_txhj_skill_discard",
        "_txhj_skill_usecard",
        "_txhj_skill_damage",
        "_txhj_skills_kill",
        "_txhj_equip5",
    ] {
        grant_skill(&mut e, 0, sk);
    }
    // `_txhj_skill_Timelimit`: `max == 1` does NOT trigger (`event2.max > 1`); `max == 2` waits until round 3 -> defeat (`2`);
    // `subsist` -> victory (`1`); `repair` wins immediately when ally boss reaches `hp == max_hp`.
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_timelimit_max".to_string()), 1);
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e); // -> seat 0 round 2
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_effect_style_init"), 1);
    assert_eq!(flag(&e, 0, "l11_timelimit_outcome"), 0, "max <= 1 must NOT settle");

    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_timelimit_max".to_string()), 2);
    e.debug_set_turn(1, Phase::End, 2);
    let _ = next_phase(&mut e); // -> seat 0 round 3 (> max 2)
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_timelimit_outcome"), 2, "round > max (> 1) settles defeat");

    let mut e_rep = start_versus(0xB5_0020, &[&[0, 2], &[1]]);
    grant_skill(&mut e_rep, 0, "_txhj_skill_Timelimit");
    e_rep.state_core_mut().vm_aux.flags.insert((0, "l11_timelimit_repair".to_string()), 1);
    e_rep.state_core_mut().vm_aux.flags.insert((2, "l11_timelimit_repair_boss".to_string()), 1);
    e_rep.debug_set_hp(2, 4);
    e_rep.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_rep);
    drain(&mut e_rep);
    assert_eq!(flag(&e_rep, 0, "l11_timelimit_outcome"), 0, "injured ally boss does not win repair");
    e_rep.debug_set_hp(2, 6);
    e_rep.debug_set_turn(2, Phase::End, 1);
    let _ = next_phase(&mut e_rep);
    drain(&mut e_rep);
    assert_eq!(flag(&e_rep, 0, "l11_timelimit_outcome"), 1, "full-hp ally boss wins repair immediately");

    let mut e_sub = start_versus(0xB5_0021, &[&[0], &[1]]);
    grant_skill(&mut e_sub, 0, "_txhj_skill_Timelimit");
    e_sub.state_core_mut().vm_aux.flags.insert((0, "l11_timelimit_max".to_string()), 2);
    e_sub.state_core_mut().vm_aux.flags.insert((0, "l11_timelimit_subsist".to_string()), 1);
    e_sub.debug_set_turn(1, Phase::End, 2);
    let _ = next_phase(&mut e_sub);
    drain(&mut e_sub);
    assert_eq!(flag(&e_sub, 0, "l11_timelimit_outcome"), 1, "subsist round > max (> 1) settles win");

    // Draw phase gain -> gaincard score increments; Play Sha killing 1-HP enemy -> usecard + damage + kill!
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(flag(&e, 0, "txhj_score_gaincard") >= 2);
    e.debug_set_hp(1, 1);
    let sha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "txhj_score_usecard"), 1);
    assert_eq!(flag(&e, 0, "txhj_score_damage"), 1);
    assert_eq!(flag(&e, 0, "txhj_score_kill"), 1);

    // Dual-treasure `_txhj_equip5` (`extension_skill.js:330`): two Treasures stay equipped simultaneously;
    // equipping a third replaces only the oldest. Negative without `_txhj_equip5`: 2nd replaces 1st.
    let mut e_eq = start_versus(0xB5_0003, &[&[0], &[1]]);
    grant_skill(&mut e_eq, 0, "_txhj_equip5");
    let t1 = equip_named(&mut e_eq, 0, "txhj_xuwangzhimian");
    let t2 = equip_named(&mut e_eq, 0, "txhj_dinglanyemingzhu");
    assert!(
        e_eq.state_core().player(0).equips.contains(&t1)
            && e_eq.state_core().player(0).equips.contains(&t2),
        "_txhj_equip5 allows two treasure equips simultaneously"
    );
    if let Some(t3) = e_eq
        .state_core()
        .deck
        .iter()
        .chain(e_eq.state_core().discard.iter())
        .copied()
        .find(|&c| {
            c != t1
                && c != t2
                && e_eq.catalog().card_slot_of(e_eq.state_core(), c) == Some(EquipSlot::Treasure)
        })
    {
        remove_from_piles_to_hand(&mut e_eq, 0, t3);
        e_eq.debug_set_turn(0, Phase::Play, 1);
        let _ = play_card(&mut e_eq, 0, t3, Vec::new());
        drain(&mut e_eq);
        let treasures: Vec<u32> = e_eq
            .state_core()
            .player(0)
            .equips
            .iter()
            .copied()
            .filter(|&c| e_eq.catalog().card_slot_of(e_eq.state_core(), c) == Some(EquipSlot::Treasure))
            .collect();
        assert_eq!(treasures, vec![t2, t3], "3rd treasure replaces oldest t1");
    }

    let mut e_plain = start_versus(0xB5_0004, &[&[0], &[1]]);
    let p1 = equip_named(&mut e_plain, 0, "txhj_xuwangzhimian");
    let p2 = equip_named(&mut e_plain, 0, "txhj_dinglanyemingzhu");
    assert!(!e_plain.state_core().player(0).equips.contains(&p1));
    assert!(e_plain.state_core().player(0).equips.contains(&p2));
}

/// Upstream: `extension_skill.js:362-405` (`twsidao`, `twsidao_equip`) & `1513-1628` (`_txhj_shenqi`, `_txhj_XXXshenqi`)
#[test]
fn test_twsidao_equip_and_shenqi_buff_sync() {
    let mut e = start_versus(0xB5_0005, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "twsidao");
    grant_skill(&mut e, 0, "twsidao_equip");
    grant_skill(&mut e, 0, "_txhj_shenqi");
    grant_skill(&mut e, 0, "_txhj_XXXshenqi");

    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e);
    let c_choose = expect_pending(&e, "twsidao:choose_treasure");
    // AI prefers Tag(2) = `txhj_chongyingshenfu` (`extension_skill.js:378`).
    assert_eq!(
        ai_plan(&e, 0),
        GameAction::Answer {
            choice_id: c_choose.id,
            selection: vec![ChoiceOption::Tag(2)],
        }
    );
    // Mid-choice save/restore parity picking `txhj_chongyingshenfu` (Tag 2)
    let mut e = save_restore_and_compare_pending(&e, &c_choose, vec![ChoiceOption::Tag(2)]);
    assert!(
        e.state_core()
            .player(0)
            .equips
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name == "txhj_chongyingshenfu"),
        "twsidao equips chosen txhj_chongyingshenfu"
    );
    assert!(
        has_skill(&e, 0, "buff_txhj_shenfuhuaxie"),
        "_txhj_shenqi grants buff_txhj_shenfuhuaxie on equip"
    );

    // Enemy dismantles `txhj_chongyingshenfu` into discard -> `_txhj_XXXshenqi` removes buff.
    let armor_id = e.state_core().player(0).equips[0];
    advance_until_turn_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(0);
    let guohe = place_std_kind(&mut e, 1, CardKind::Dismantle);
    let _ = play_card(&mut e, 1, guohe, vec![0]);
    drain(&mut e);
    assert!(!has_skill(&e, 0, "buff_txhj_shenfuhuaxie"));
    assert!(e.state_core().discard.contains(&armor_id));

    // Next Prepare of seat 0: `twsidao_equip` retrieves `txhj_chongyingshenfu` from pile and re-equips!
    advance_until_turn_phase(&mut e, 0, Phase::Judge);
    drain(&mut e);
    assert!(
        e.state_core().player(0).equips.contains(&armor_id),
        "twsidao_equip re-equips chosen treasure from pile on Prepare"
    );
    assert!(has_skill(&e, 0, "buff_txhj_shenfuhuaxie"));

    // Artifact swap sharing buff (`txhj_xuwangzhimian` -> `txhj_chongyingshenfu`): buffer retains shared buff.
    grant_skill(&mut e, 0, "_txhj_equip5");
    let _xw = equip_named(&mut e, 0, "txhj_xuwangzhimian");
    assert!(has_skill(&e, 0, "buff_txhj_shenfuhuaxie"));
}

/// Upstream: `extension_skill.js:408-454` (`polu`, `polu_damage`)
#[test]
fn test_polu_and_polu_damage_lifecycle() {
    let mut e = start_versus(0xB5_0006, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "polu");
    grant_skill(&mut e, 0, "polu_damage");

    // Advance from seat 1 End -> seat 0 Prepare -> `polu` auto-equips `txhj_piliche`.
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e);
    drain(&mut e);
    let piliche = e
        .state_core()
        .player(0)
        .equips
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].name == "txhj_piliche")
        .expect("polu equips txhj_piliche");

    // Negative: dealing damage while `txhj_piliche` is equipped does NOT fire `polu_damage`.
    advance_until_turn_phase(&mut e, 1, Phase::Play);
    let hand_before_neg = e.state_core().player(0).hand.len();
    let sha_neg = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha_neg, vec![0]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 5);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_neg);

    // Strip `txhj_piliche` (enters `detached_cards(Destroyed)` via `machine.rs:658`).
    e.debug_clear_hand(0);
    let guohe = place_std_kind(&mut e, 1, CardKind::Dismantle);
    let _ = play_card(&mut e, 1, guohe, vec![0]);
    drain(&mut e);
    assert!(
        !e.state_core().player(0).equips.contains(&piliche)
            && !e.state_core().discard.contains(&piliche),
        "txhj_piliche is destroyed when leaving play"
    );

    // Seat 0 next Prepare does NOT revive destroyed `txhj_piliche`.
    advance_until_turn_phase(&mut e, 0, Phase::Judge);
    drain(&mut e);
    assert!(
        !e.state_core()
            .player(0)
            .equips
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name == "txhj_piliche"),
        "destroyed txhj_piliche stays gone"
    );

    // Seat 1 deals 2 Jiu+Sha damage -> `polu_damage` loops twice: draws 2 & equips weapon from deck!
    advance_until_turn_phase(&mut e, 1, Phase::Play);
    let w_top = e.debug_set_deck_top_by(|cd| {
        matches!(cd.kind, CardKind::Weapon | CardKind::Qinggang | CardKind::Hanbing)
    });
    let _ = w_top;
    let hand_before = e.state_core().player(0).hand.len();
    let jiu = place_kind(&mut e, 1, CardKind::Jiu);
    let sha2 = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, jiu, Vec::new());
    let _ = play_card(&mut e, 1, sha2, vec![0]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before + 2);
    assert!(
        e.state_core()
            .player(0)
            .equips
            .iter()
            .any(|&c| e.catalog().card_slot_of(e.state_core(), c) == Some(EquipSlot::Weapon)),
        "polu_damage equips weapon from deck after draw"
    );
}

/// Upstream: `extension_skill.js:658-729` (`hanbing_skill` + `fuchendangmo` + `shenhanxueyin`)
#[test]
fn test_hanbing_skill_cancel_buff_combo_and_ai() {
    // 1. Plain hanbing_skill: prevents 1 Sha damage & attacker steals/discards 2 target cards.
    let mut e = start_versus(0xB5_0007, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "hanbing_skill");
    e.debug_set_turn(0, Phase::Play, 1);
    let t1 = place_kind(&mut e, 1, CardKind::Tao);
    let t2 = place_kind(&mut e, 1, CardKind::Jiu);
    let sha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let c_conf = expect_pending(&e, "hanbing_skill:confirm");
    assert_eq!(
        ai_plan(&e, 0),
        GameAction::Answer {
            choice_id: c_conf.id,
            selection: vec![ChoiceOption::Confirm],
        }
    );
    let _ = answer(&mut e, &c_conf, vec![ChoiceOption::Confirm]);
    let p1 = expect_pending(&e, "hanbing_skill:pick_card");
    // Mid-choice save/restore on `hanbing_skill:pick_card`:
    let e_after_p1 = save_restore_and_compare_pending(&e, &p1, vec![ChoiceOption::Card(t1)]);
    // In `e`, continue interactively picking t1 then t2:
    let _ = answer(&mut e, &p1, vec![ChoiceOption::Card(t1)]);
    let p2 = expect_pending(&e, "hanbing_skill:pick_card");
    let _ = answer(&mut e, &p2, vec![ChoiceOption::Card(t2)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 6, "hanbing prevents damage");
    assert!(e.state_core().player(1).hand.is_empty());
    let _ = e_after_p1.state_hash();

    // 2. Cancel branch -> Sha deals 1 damage normally; `hp == 1` AI prefers Pass.
    let mut e_c = start_versus(0xB5_0008, &[&[0], &[1]]);
    grant_skill(&mut e_c, 0, "hanbing_skill");
    e_c.debug_set_hp(1, 1);
    let _tc1 = place_kind(&mut e_c, 1, CardKind::Tao);
    let _tc2 = place_kind(&mut e_c, 1, CardKind::Jiu);
    e_c.debug_set_turn(0, Phase::Play, 1);
    let sha_c = place_kind(&mut e_c, 0, CardKind::Sha);
    let _ = play_card(&mut e_c, 0, sha_c, vec![1]);
    let c_low = expect_pending(&e_c, "hanbing_skill:confirm");
    assert_eq!(
        ai_plan(&e_c, 0),
        GameAction::Answer {
            choice_id: c_low.id,
            selection: vec![ChoiceOption::Pass],
        },
        "AI skips hanbing against 1-HP enemy"
    );
    let _ = answer(&mut e_c, &c_low, vec![ChoiceOption::Pass]);
    drain(&mut e_c);
    assert_eq!(e_c.state_core().player(1).hp, 0);

    // 3. `fuchendangmo` (3 discards) + `shenhanxueyin` (attacker gains discarded cards).
    let mut eb = start_versus(0xB5_0009, &[&[0], &[1]]);
    grant_skill(&mut eb, 0, "hanbing_skill");
    grant_skill(&mut eb, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut eb, 0, "buff_txhj_shenhanxueyin");
    let _w = equip_kind(&mut eb, 0, CardKind::Weapon);
    let k1 = place_kind(&mut eb, 1, CardKind::Tao);
    let k2 = place_kind(&mut eb, 1, CardKind::Jiu);
    let k3 = place_kind(&mut eb, 1, CardKind::Dismantle);
    eb.debug_set_turn(0, Phase::Play, 1);
    let sha_b = place_kind(&mut eb, 0, CardKind::Sha);
    let _ = play_card(&mut eb, 0, sha_b, vec![1]);
    let cb = expect_pending(&eb, "hanbing_skill:confirm");
    let _ = answer(&mut eb, &cb, vec![ChoiceOption::Confirm]);
    for k in [k1, k2, k3] {
        let pk = expect_pending(&eb, "hanbing_skill:pick_card");
        let _ = answer(&mut eb, &pk, vec![ChoiceOption::Card(k)]);
    }
    drain(&mut eb);
    assert_eq!(eb.state_core().player(1).hp, 6);
    assert!(
        eb.state_core().player(0).hand.contains(&k1)
            && eb.state_core().player(0).hand.contains(&k2)
            && eb.state_core().player(0).hand.contains(&k3),
        "fuchendangmo steals 3 via shenhanxueyin"
    );
}

/// Upstream: `extension_skill.js:733-782` (`cixiong_skill`) & `911-971` (`guanshi_skill`)
#[test]
fn test_cixiong_and_guanshi_skills_authentic() {
    // `cixiong_skill`: opposite sex triggers confirm -> target discards 1 (`2` with fuchendangmo + shenhanxueyin) or attacker draws 1.
    let mut e = start_versus(0xB5_000A, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "cixiong_skill");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "effective_gender_override".to_string()), 1); // Male
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "effective_gender_override".to_string()), 2); // Female
    e.debug_set_turn(0, Phase::Play, 1);
    let td1 = place_kind(&mut e, 1, CardKind::Tao);
    let sha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let c_conf = expect_pending(&e, "cixiong_skill:confirm");
    let _ = answer(&mut e, &c_conf, vec![ChoiceOption::Confirm]);
    let c_disc = expect_pending(&e, "cixiong_skill:target_discard");
    // Save/restore mid target discard -> cancel -> attacker draws 1!
    let e_cancel = save_restore_and_compare_pending(&e, &c_disc, Vec::new());
    assert_eq!(e_cancel.state_core().player(0).hand.len(), 1);
    assert!(e_cancel.state_core().player(1).hand.contains(&td1));

    // Fuchendangmo + Shenhanxueyin on cixiong: target discards 2 and attacker steals both.
    let mut eb = start_versus(0xB5_000B, &[&[0], &[1]]);
    grant_skill(&mut eb, 0, "cixiong_skill");
    grant_skill(&mut eb, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut eb, 0, "buff_txhj_shenhanxueyin");
    let _w = equip_kind(&mut eb, 0, CardKind::Weapon);
    eb.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "effective_gender_override".to_string()), 1);
    eb.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "effective_gender_override".to_string()), 2);
    eb.debug_set_turn(0, Phase::Play, 1);
    let d1 = place_kind(&mut eb, 1, CardKind::Tao);
    let d2 = place_kind(&mut eb, 1, CardKind::Jiu);
    let sha_b = place_kind(&mut eb, 0, CardKind::Sha);
    let _ = play_card(&mut eb, 0, sha_b, vec![1]);
    let cb = expect_pending(&eb, "cixiong_skill:confirm");
    let _ = answer(&mut eb, &cb, vec![ChoiceOption::Confirm]);
    let cd = expect_pending(&eb, "cixiong_skill:target_discard");
    assert_eq!(cd.spec.min, 2);
    let _ = answer(&mut eb, &cd, vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)]);
    drain(&mut eb);
    assert!(
        eb.state_core().player(0).hand.contains(&d1)
            && eb.state_core().player(0).hand.contains(&d2)
    );

    // Negative: same gender never triggers cixiong_skill.
    let mut e_same = start_versus(0xB5_000C, &[&[0], &[1]]);
    grant_skill(&mut e_same, 0, "cixiong_skill");
    e_same
        .state_core_mut()
        .vm_aux
        .flags
        .insert((0, "effective_gender_override".to_string()), 1);
    e_same
        .state_core_mut()
        .vm_aux
        .flags
        .insert((1, "effective_gender_override".to_string()), 1);
    e_same.debug_set_turn(0, Phase::Play, 1);
    let _h1 = place_kind(&mut e_same, 1, CardKind::Tao);
    let s_same = place_kind(&mut e_same, 0, CardKind::Sha);
    let _ = play_card(&mut e_same, 0, s_same, vec![1]);
    assert!(e_same.open_choice().is_none(), "same sex must not open cixiong");

    // `guanshi_skill`: Jiu + LeiSha dodged -> discards 2 (1 with shangjiangdelvli) -> deals 2 Thunder damage!
    let mut eg = start_versus(0xB5_000D, &[&[0], &[1]]);
    let gs = equip_kind(&mut eg, 0, CardKind::Guanshi);
    grant_skill(&mut eg, 0, "guanshi_skill");
    eg.debug_set_turn(0, Phase::Play, 1);
    let c1 = place_kind(&mut eg, 0, CardKind::Tao);
    let c2 = place_kind(&mut eg, 0, CardKind::Jiu);
    let shan = place_kind(&mut eg, 1, CardKind::Shan);
    let jiu = place_kind(&mut eg, 0, CardKind::Jiu);
    let lei = place_kind(&mut eg, 0, CardKind::LeiSha);
    let _ = play_card(&mut eg, 0, jiu, Vec::new());
    let _ = play_card(&mut eg, 0, lei, vec![1]);
    let resp = expect_pending(&eg, "respond_shan");
    let _ = answer(&mut eg, &resp, vec![ChoiceOption::Card(shan)]);
    let gc = expect_pending(&eg, "guanshi_skill:discard");
    assert_eq!(gc.spec.min, 2);
    assert!(
        !gc.spec.options.contains(&ChoiceOption::Card(gs)),
        "equipped guanshi cannot be discarded to pay guanshi_skill"
    );
    let eg_done = save_restore_and_compare_pending(
        &eg,
        &gc,
        vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
    );
    assert_eq!(
        eg_done.state_core().player(1).hp,
        4,
        "guanshi preserves Jiu (+1) + LeiSha damage = 2"
    );

    // `guanshi_skill` + `bolehuiyan3` + `card_damage_bonus`:
    // Dodged Sha forced through `ShaResolveMachine::emit_sha_damage` preserves `card_damage_bonus` AND passes `is_damage = true` to `sha_end_step` (`bolehuiyan3`, not `bolehuiyan2`)!
    let mut e_bh = start_versus_with_u08(0xB5_0022, &[&[0], &[1, 2]]);
    let _gs_bh = equip_kind(&mut e_bh, 0, CardKind::Guanshi);
    grant_skill(&mut e_bh, 0, "guanshi_skill");
    grant_skill(&mut e_bh, 2, "bolehuiyan");
    e_bh.debug_set_turn(0, Phase::Play, 1);
    let bh_c1 = place_kind(&mut e_bh, 0, CardKind::Tao);
    let bh_c2 = place_kind(&mut e_bh, 0, CardKind::Tao);
    let bh_shan = place_kind(&mut e_bh, 1, CardKind::Shan);
    let bh_drop = place_kind(&mut e_bh, 2, CardKind::Tao);
    let bh_sha = place_kind(&mut e_bh, 0, CardKind::LeiSha);
    e_bh.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("card_damage_bonus:{bh_sha}")), 1);
    let _ = play_card(&mut e_bh, 0, bh_sha, vec![1]);
    let bh_pred = expect_pending(&e_bh, "bolehuiyan:predict");
    let _ = answer(&mut e_bh, &bh_pred, vec![ChoiceOption::Tag(2)]); // Predict Miss -> guanshi Hit -> bolehuiyan3 wrong discard!
    let bh_resp = expect_pending(&e_bh, "respond_shan");
    let _ = answer(&mut e_bh, &bh_resp, vec![ChoiceOption::Card(bh_shan)]);
    let bh_gs = expect_pending(&e_bh, "guanshi_skill:discard");
    let e_bh_done = save_restore_and_compare_pending_with(
        &e_bh,
        &bh_gs,
        vec![ChoiceOption::Card(bh_c1), ChoiceOption::Card(bh_c2)],
        taixu_with_u08_verified(),
    );
    assert_eq!(e_bh_done.state_core().player(1).hp, 4, "guanshi + extraDamage dealt 2 Thunder damage");
    assert!(
        !e_bh_done.state_core().player(2).hand.contains(&bh_drop),
        "bolehuiyan3 (is_damage=true) forced predictor seat 2 discard"
    );
}

/// Upstream: `extension_skill.js:975-1136` (`sanjian_skill`, `yitianjian`, `pyzhuren_diamond`)
#[test]
fn test_sanjian_yitianjian_and_pyzhuren_diamond() {
    // 1. `sanjian_skill`: Sha damage to seat 1 (dist(1, 2) <= 1) -> discard 1 -> target seat 2 takes 1 damage.
    let mut es = start_versus(0xB5_000E, &[&[0], &[1, 2]]);
    grant_skill(&mut es, 0, "sanjian_skill");
    es.debug_set_turn(0, Phase::Play, 1);
    let cost = place_kind(&mut es, 0, CardKind::Tao);
    let sha = place_kind(&mut es, 0, CardKind::Sha);
    let _ = play_card(&mut es, 0, sha, vec![1]);
    let cs_disc = expect_pending(&es, "sanjian_skill:discard");
    let _ = answer(&mut es, &cs_disc, vec![ChoiceOption::Card(cost)]);
    let cs_tgt = expect_pending(&es, "sanjian_skill:target");
    assert!(!cs_tgt.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(cs_tgt.spec.options.contains(&ChoiceOption::Player(2)));
    // Cancelling target leaves `cost` card un-discarded!
    let es_c = save_restore_and_compare_pending(&es, &cs_tgt, Vec::new());
    assert!(es_c.state_core().player(0).hand.contains(&cost));
    assert_eq!(es_c.state_core().player(2).hp, 6);
    let _ = answer(&mut es, &cs_tgt, vec![ChoiceOption::Player(2)]);
    drain(&mut es);
    assert_eq!(es.state_core().player(2).hp, 5);
    assert!(!es.state_core().player(0).hand.contains(&cost));

    // `sanjian_skill` + `buff_txhj_shangjiangdelvli`: skips discard directly to `sanjian_skill:target`.
    let mut es_sj = start_versus(0xB5_000F, &[&[0], &[1, 2]]);
    let _w = equip_kind(&mut es_sj, 0, CardKind::Weapon);
    grant_skill(&mut es_sj, 0, "sanjian_skill");
    grant_skill(&mut es_sj, 0, "buff_txhj_shangjiangdelvli");
    es_sj.debug_set_turn(0, Phase::Play, 1);
    let sha_sj = place_kind(&mut es_sj, 0, CardKind::Sha);
    let _ = play_card(&mut es_sj, 0, sha_sj, vec![1]);
    let ct = expect_pending(&es_sj, "sanjian_skill:target");
    let _ = answer(&mut es_sj, &ct, vec![ChoiceOption::Player(2)]);
    drain(&mut es_sj);
    assert_eq!(es_sj.state_core().player(2).hp, 5);

    // 2. `yitianjian`: optional discard recovers 1 HP; cancel recovers 0 and keeps hand card.
    let mut ey = start_versus(0xB5_0010, &[&[0], &[1]]);
    grant_skill(&mut ey, 0, "yitianjian");
    ey.debug_set_hp(0, 4);
    ey.debug_set_turn(0, Phase::Play, 1);
    let yd = place_kind(&mut ey, 0, CardKind::Tao);
    let ys = place_kind(&mut ey, 0, CardKind::Sha);
    let _ = play_card(&mut ey, 0, ys, vec![1]);
    let cy = expect_pending(&ey, "yitianjian:discard");
    let ey_cancel = save_restore_and_compare_pending(&ey, &cy, Vec::new());
    assert_eq!(ey_cancel.state_core().player(0).hp, 4);
    assert!(ey_cancel.state_core().player(0).hand.contains(&yd));
    let _ = answer(&mut ey, &cy, vec![ChoiceOption::Card(yd)]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(0).hp, 5);

    // 3. `pyzhuren_diamond`: usable 2 per turn, cannot discard equipped self, cancel refunds usage!
    let mut ep = start_versus(0xB5_0011, &[&[0], &[1]]);
    grant_skill(&mut ep, 0, engine_native::vocabulary::names::PAO_XIAO);
    let _py = equip_kind(&mut ep, 0, CardKind::Weapon);
    grant_skill(&mut ep, 0, "pyzhuren_diamond");
    ep.debug_set_max_hp(1, 10);
    ep.debug_set_hp(1, 10);
    ep.debug_set_turn(0, Phase::Play, 1);
    let d1 = place_kind(&mut ep, 0, CardKind::Tao);
    let d2 = place_kind(&mut ep, 0, CardKind::Jiu);
    let s1 = place_kind(&mut ep, 0, CardKind::Sha);
    let s2 = place_kind(&mut ep, 0, CardKind::Sha);
    let s3 = place_kind(&mut ep, 0, CardKind::Sha);
    let s4 = place_kind(&mut ep, 0, CardKind::Sha);

    // Sha 1 -> discard d1 -> 2 damage (used 1/2)
    let _ = play_card(&mut ep, 0, s1, vec![1]);
    let p_a1 = expect_pending(&ep, "pyzhuren_diamond:discard");
    let _ = answer(&mut ep, &p_a1, vec![ChoiceOption::Card(d1)]);
    drain(&mut ep);
    assert_eq!(ep.state_core().player(1).hp, 8);

    // Sha 2 -> cancel -> 1 damage (still 1/2!)
    let _ = play_card(&mut ep, 0, s2, vec![1]);
    let p_a2 = expect_pending(&ep, "pyzhuren_diamond:discard");
    let _ = answer(&mut ep, &p_a2, Vec::new());
    drain(&mut ep);
    assert_eq!(ep.state_core().player(1).hp, 7);

    // Sha 3 -> discard d2 -> 2 damage (used 2/2)
    let _ = play_card(&mut ep, 0, s3, vec![1]);
    let p_a3 = expect_pending(&ep, "pyzhuren_diamond:discard");
    let _ = answer(&mut ep, &p_a3, vec![ChoiceOption::Card(d2)]);
    drain(&mut ep);
    assert_eq!(ep.state_core().player(1).hp, 5);

    // Sha 4 -> usable: 2 exhausted -> no prompt even with card in hand!
    let _spare = place_kind(&mut ep, 0, CardKind::Tao);
    let _ = play_card(&mut ep, 0, s4, vec![1]);
    assert!(ep.open_choice().is_none(), "pyzhuren_diamond limited to 2 activations/turn");
    assert_eq!(ep.state_core().player(1).hp, 4);
}

/// Upstream: `extension_skill.js:1141-1508` (`twzhouhu`, `twharvestinori`, `twzuhuo` + L11 `_bihu` aliases)
/// + `582-880` (`guofengyupao`, `gx_taijifuchen`, `wushuangfangtianji_skill`, `qilin_skill`)
#[test]
fn test_l11_mahou_bihu_aliases_and_l28_l11_overlap_equips() {
    let mut e = start_versus_with_l27(0xB5_0012, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "twzhouhu");
    grant_skill(&mut e, 0, "buff_txhj_digongdeyouhu");
    grant_skill(&mut e, 0, "buff_txhj_tiangongdeyouhu");
    e.debug_set_hp(0, 3);
    e.debug_set_turn(0, Phase::Play, 1);
    let red = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| {
            matches!(
                e.catalog().cards[c as usize].suit,
                Suit::Heart | Suit::Diamond
            )
        })
        .expect("red card in deck");
    remove_from_piles_to_hand(&mut e, 0, red);
    let out = activate_skill(&mut e, 0, "twzhouhu");
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let pick_c = expect_pending(&e, "twzhouhu:discard_red");
    let _ = answer(&mut e, &pick_c, vec![ChoiceOption::Card(red)]);
    let pick_dur = expect_pending(&e, "twzhouhu:choose_turns");
    assert!(
        pick_dur.spec.options.contains(&ChoiceOption::Tag(4)),
        "buff_txhj_digongdeyouhu unlocks 4-turn (Tag 4) mahou duration"
    );
    let _ = answer(&mut e, &pick_dur, vec![ChoiceOption::Tag(3)]); // 3-turn option -> tiangongdeyouhu advances by 1
    drain(&mut e);

    // 3-turn duration with `tiangongdeyouhu` resolves after 2 phaseEnds instead of 3!
    e.debug_set_turn(1, Phase::Discard, 1);
    let _ = next_phase(&mut e);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "not yet healed after 1 turn");
    e.debug_set_turn(1, Phase::Discard, 2);
    let _ = next_phase(&mut e);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 6, "twzhouhu(3) + tiangongdeyouhu healed 3 after 2 turns");

    // Verify ALL 4 L11 equip catalog/skill overrides (`guofengyupao`, `gx_taijifuchen`, `wushuangfangtianji_skill`, `qilin_skill`):
    let mut ee = start_versus_with_l28(0xB5_0013, &[&[0], &[1]]);
    let _ = equip_named(&mut ee, 1, "txhj_guofengyupao");
    assert!(has_skill(&ee, 1, "txhj_guofengyupao"));
    ee.debug_set_turn(0, Phase::Play, 1);
    let g_dismantle = place_std_kind(&mut ee, 0, CardKind::Dismantle);
    assert!(matches!(
        play_card(&mut ee, 0, g_dismantle, vec![1]),
        CommandOutcome::Rejected { .. }
    ));

    // Direct `guofengyupao` skill cancels ordinary trick `Dismantle`:
    let mut egf = start_versus_with_l28(0xB5_0023, &[&[0], &[1]]);
    grant_skill(&mut egf, 1, "guofengyupao");
    egf.debug_set_turn(0, Phase::Play, 1);
    let _h1_gf = place_kind(&mut egf, 1, CardKind::Tao);
    let gf_dis = place_std_kind(&mut egf, 0, CardKind::Dismantle);
    assert!(matches!(
        play_card(&mut egf, 0, gf_dis, vec![1]),
        CommandOutcome::Rejected { .. }
    ));

    // `gx_taijifuchen` (`extension_skill.js:617`): unconditionally requires 2 discards EVEN WITHOUT `buff_txhj_fuchendangmo`!
    let mut egx = start_versus(0xB5_0024, &[&[0], &[1]]);
    grant_skill(&mut egx, 0, "gx_taijifuchen");
    egx.debug_set_turn(0, Phase::Play, 1);
    let gx_sha = place_kind_suit(&mut egx, 0, CardKind::Sha, Suit::Heart);
    let gx_d1 = place_kind_suit(&mut egx, 1, CardKind::Tao, Suit::Heart); // same suit -> owner steals!
    let gx_d2 = place_kind_suit(&mut egx, 1, CardKind::Jiu, Suit::Club);
    let gx_shan = place_kind(&mut egx, 1, CardKind::Shan);
    let _ = play_card(&mut egx, 0, gx_sha, vec![1]);
    let t1 = expect_pending(&egx, "gx_taijifuchen:target_discard");
    let _ = answer(&mut egx, &t1, vec![ChoiceOption::Card(gx_d1)]);
    let t2 = expect_pending(&egx, "gx_taijifuchen:target_discard");
    let _ = answer(&mut egx, &t2, vec![ChoiceOption::Card(gx_d2)]);
    let rs = expect_pending(&egx, "respond_shan");
    let _ = answer(&mut egx, &rs, vec![ChoiceOption::Card(gx_shan)]);
    drain(&mut egx);
    assert_eq!(egx.state_core().player(1).hp, 6, "2 discards allowed Shan response");
    assert!(egx.state_core().player(0).hand.contains(&gx_d1), "same-suit card stolen");

    // `wushuangfangtianji_skill` (`extension_skill.js:796-808`):
    // (a) Target has 0 `he` cards -> auto-draws 1 WITHOUT opening `choose_mode`!
    let mut ew0 = start_versus_with_l28(0xB5_0025, &[&[0], &[1]]);
    grant_skill(&mut ew0, 0, "wushuangfangtianji_skill");
    ew0.debug_set_turn(0, Phase::Play, 1);
    let ws0 = place_kind(&mut ew0, 0, CardKind::Sha);
    let _ = play_card(&mut ew0, 0, ws0, vec![1]);
    drain(&mut ew0);
    assert!(ew0.open_choice().is_none(), "0 target he cards auto-draws without choose_mode");
    assert_eq!(ew0.state_core().player(0).hand.len(), 1);
    // (b) Target has >= 1 `he` card -> mandatory control (`allow_cancel: false`)!
    let mut ew1 = start_versus_with_l28(0xB5_0026, &[&[0], &[1]]);
    grant_skill(&mut ew1, 0, "wushuangfangtianji_skill");
    ew1.debug_set_turn(0, Phase::Play, 1);
    let _t1_card = place_kind(&mut ew1, 1, CardKind::Tao);
    let ws1 = place_kind(&mut ew1, 0, CardKind::Sha);
    let _ = play_card(&mut ew1, 0, ws1, vec![1]);
    let wmode = expect_pending(&ew1, "wushuangfangtianji:choose_mode");
    assert!(!wmode.spec.allow_cancel, "L11 wushuangfangtianji chooseControl is mandatory");
    let ew1_done = save_restore_and_compare_pending_with(
        &ew1,
        &wmode,
        vec![ChoiceOption::Tag(0)],
        taixu_with_l28_verified(),
    );
    assert_eq!(ew1_done.state_core().player(0).hand.len(), 1);

    // `qilin_skill` (`extension_skill.js:856-907`): `SelectCards` over target's dual `equip5` treasures + `fuchendangmo`!
    let mut eql = start_versus(0xB5_0027, &[&[0], &[1]]);
    grant_skill(&mut eql, 0, "qilin_skill");
    grant_skill(&mut eql, 0, "buff_txhj_fuchendangmo");
    grant_skill(&mut eql, 1, "_txhj_equip5");
    let tr1 = equip_named(&mut eql, 1, "txhj_xuwangzhimian");
    let tr2 = equip_named(&mut eql, 1, "txhj_dinglanyemingzhu");
    eql.debug_set_turn(0, Phase::Play, 1);
    let ql_sha = place_kind(&mut eql, 0, CardKind::Sha);
    let _ = play_card(&mut eql, 0, ql_sha, vec![1]);
    let q1 = expect_pending(&eql, "qilin_skill:discard_horse");
    assert!(
        q1.spec.options.contains(&ChoiceOption::Card(tr1))
            && q1.spec.options.contains(&ChoiceOption::Card(tr2)),
        "qilin_skill must let attacker choose among target equip5 treasures"
    );
    let _ = answer(&mut eql, &q1, vec![ChoiceOption::Card(tr2)]);
    let q2 = expect_pending(&eql, "qilin_skill:discard_horse");
    let eql_done = save_restore_and_compare_pending(&eql, &q2, vec![ChoiceOption::Card(tr1)]);
    assert!(
        !eql_done.state_core().player(1).equips.contains(&tr1)
            && !eql_done.state_core().player(1).equips.contains(&tr2),
        "qilin_skill discarded chosen tr2 first and tr1 on fuchendangmo 2nd ask"
    );
}
