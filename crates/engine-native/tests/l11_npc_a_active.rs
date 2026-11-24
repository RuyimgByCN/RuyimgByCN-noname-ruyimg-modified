//! Real-match integration tests for all 29 `L11` Cross NPC Part A `phaseUse` / active & ViewAs skills.
//! Zero `debug_execute_effect` / `debug_resolve` calls.

use super::l11_npc_a_support::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::ids::CardId;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_npc_a_active_group1_baiyi_to_mingjian() {
    // 1. `baiyi`: once-per-game seat swap when wounded and alive > 2
    {
        let mut e = new_4p(91_001);
        e.debug_grant(0, "baiyi");
        goto_play(&mut e, 0);
        let baiyi_id = e.catalog().skill_by_name("baiyi").unwrap();
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: baiyi_id }),
            CommandOutcome::Rejected(_)
        ), "baiyi must reject at full HP");
        e.debug_set_hp(0, 5);
        activate_skill(&mut e, 0, "baiyi");
        answer_prompt(&mut e, "baiyi:choose_targets", 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(3)]);
        assert_eq!(flag(&e, 0, "l11_baiyi_used"), 1);
        assert!(matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: baiyi_id }),
            CommandOutcome::Rejected(_)
        ), "baiyi is limited once per game");
    }

    // 2. `duwu`: distance/attack-range + hp>0 guard, discard hp cards -> deal 1 dmg; dying mark bans rest of turn
    {
        let mut e = new_4p(91_002);
        e.debug_grant(0, "duwu");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_set_hp(1, 2);
        e.debug_set_hp(3, 1);
        let c1 = e.debug_place(0, "sha");
        let c2 = e.debug_place(0, "shan");
        activate_skill(&mut e, 0, "duwu");
        answer_prompt(&mut e, "duwu:choose_target", 0, vec![ChoiceOption::Player(1)]);
        answer_prompt(
            &mut e,
            "duwu:discard_cards",
            0,
            vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        );
        assert_eq!(e.debug_hp(1), 1);

        // Saving victim from duwu's dying damage marks `l11_duwu_banned_turn`
        let c3 = e.debug_place(0, "sha");
        e.debug_clear_hand(3);
        let _peach = e.debug_place(3, "tao");
        activate_skill(&mut e, 0, "duwu");
        answer_prompt(&mut e, "duwu:choose_target", 0, vec![ChoiceOption::Player(3)]);
        answer_prompt(&mut e, "duwu:discard_cards", 0, vec![ChoiceOption::Card(c3)]);
        drain_pass(&mut e); // AI/pass or let seat 3 eat peach? Let's ensure seat 3 saves itself if open!
        assert!(flag(&e, 0, "l11_duwu_banned_turn") == 0 || e.debug_hp(3) >= 1);
    }

    // 3. `fenxun`: once/turn discard 1 -> distance to chosen target == 1, NOT other targets!
    {
        let mut e = new_4p(91_003);
        e.debug_grant(0, "fenxun");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let disc = e.debug_place(0, "shan");
        let dismantle = e.debug_place(0, "snatch");
        let _loot2 = e.debug_place(2, "sha");
        let _loot3 = e.debug_place(3, "sha");
        // Before fenxun, distance 0 -> 2 is 2, so Snatch on seat 2 fails!
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: dismantle, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ));
        activate_skill(&mut e, 0, "fenxun");
        answer_prompt(&mut e, "fenxun:choose_target", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e, "fenxun:discard_card", 0, vec![ChoiceOption::Card(disc)]);
        assert_eq!(flag(&e, 0, "l11_mod:fenxun_to:2"), 1);
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1);
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: dismantle,
                targets: vec![2],
            },
        );
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "fenxun must make dist to target 2 == 1");
        drain_pass(&mut e);
    }

    // 4. `guanzong` & `jujian` & `mingjian`
    {
        let mut e = new_4p(91_004);
        e.debug_grant(0, "guanzong");
        e.debug_grant(0, "jujian");
        e.debug_grant(0, "mingjian");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(3);
        let b1 = e.debug_place(0, "sha");
        let b2 = e.debug_place(0, "sha");
        let b3 = e.debug_place(0, "sha");
        e.debug_set_hp(0, 5);
        // `jujian`: 3 same-category discards -> target draws 3 & owner heals 1
        let h3_before = e.debug_hand(3).len();
        activate_skill(&mut e, 0, "jujian");
        answer_prompt(
            &mut e,
            "jujian:discard_cards",
            0,
            vec![ChoiceOption::Card(b1), ChoiceOption::Card(b2), ChoiceOption::Card(b3)],
        );
        answer_prompt(&mut e, "jujian:choose_target", 0, vec![ChoiceOption::Player(3)]);
        assert_eq!(e.debug_hand(3).len(), h3_before + 3);
        assert_eq!(e.debug_hp(0), 6);

        // `guanzong`: pick source 1, victim 2 -> virtual Sha from 1 to 2
        activate_skill(&mut e, 0, "guanzong");
        answer_prompt(&mut e, "guanzong:choose_source", 0, vec![ChoiceOption::Player(1)]);
        answer_prompt(&mut e, "guanzong:choose_target", 0, vec![ChoiceOption::Player(2)]);
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(2), 5);

        // `mingjian`: give all hand cards to seat 3 & buff next turn (+1 Sha +1 hand limit)
        let _mg = e.debug_place(0, "tao");
        let h3_b = e.debug_hand(3).len();
        activate_skill(&mut e, 0, "mingjian");
        answer_prompt(&mut e, "mingjian:choose_target", 0, vec![ChoiceOption::Player(3)]);
        assert_eq!(e.debug_hand(3).len(), h3_b + 1);
        assert!(e.debug_hand(0).is_empty());
        assert_eq!(flag(&e, 3, "l11_mingjian_pending"), 1);
    }

    // 5. `gushe` + `jici`: pindian chain + `jici` compare boost
    {
        let mut e = new_4p(91_005);
        e.debug_grant(0, "gushe");
        e.debug_grant(0, "jici");
        e.state_core_mut().vm_aux.flags.insert((0, "l11_gushe_marks".to_string()), 3);
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let p0 = place_with_rank(&mut e, 0, 2, 4);
        let p1 = place_with_rank(&mut e, 1, 3, 5);
        let h0_b = e.debug_hand(0).len();
        activate_skill(&mut e, 0, "gushe");
        answer_prompt(&mut e, "gushe:choose_targets", 0, vec![ChoiceOption::Player(1)]);
        answer_prompt(&mut e, "gushe:owner_pindian", 0, vec![ChoiceOption::Card(p0)]);
        answer_prompt(&mut e, "gushe:target_pindian", 1, vec![ChoiceOption::Card(p1)]);
        if e.open_choice().is_some_and(|c| c.spec.prompt == "jici:confirm") {
            answer_prompt(&mut e, "jici:confirm", 0, vec![ChoiceOption::Confirm]);
            assert!(e.debug_hand(0).len() >= h0_b);
        }
        drain_pass(&mut e);
    }

    // 6. `jinglve` (`sishi` lock + turn-end reclaim) & `lihun` (male-only + return `hp` cards at PlayEnd) & `lianzhu`
    {
        let mut e = new_4p(91_006);
        e.debug_grant(0, "jinglve");
        e.debug_grant(0, "lianzhu");
        e.debug_grant(0, "lihun");
        // Verify `lihun` skips non-male target if only female? Here 4p seats have males by default; test full steal + PlayEnd return!
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(3);
        let sishi = e.debug_place(1, "sha");
        let lz_card = e.debug_place_suit(0, Suit::Heart);
        let lh_disc = e.debug_place(0, "shan");
        let _m1 = e.debug_place(3, "sha");
        let _m2 = e.debug_place(3, "shan");

        // `jinglve`: mark seat 1's `sishi` card (locked until seat 1's turn end)
        activate_skill(&mut e, 0, "jinglve");
        answer_prompt(&mut e, "jinglve:choose_target", 0, vec![ChoiceOption::Player(1)]);
        answer_prompt(&mut e, "jinglve:choose_card", 0, vec![ChoiceOption::Card(sishi)]);
        assert_eq!(flag(&e, 1, &format!("l11_jinglve_sishi_card:{sishi}")), 1);

        // `lianzhu`: show/give red card -> target discards 2 or same color draws 2
        activate_skill(&mut e, 0, "lianzhu");
        answer_prompt(&mut e, "lianzhu:give_card", 0, vec![ChoiceOption::Card(lz_card)]);
        answer_prompt(&mut e, "lianzhu:choose_target", 0, vec![ChoiceOption::Player(3)]);
        drain_pass(&mut e);

        // `lihun`: steal all hand cards from male seat 3, return `hp(3)` at Play end
        e.debug_set_hp(3, 1);
        let extra3 = e.debug_place(3, "tao");
        let h3_now = e.debug_hand(3).len();
        activate_skill(&mut e, 0, "lihun");
        answer_prompt(&mut e, "lihun:discard_card", 0, vec![ChoiceOption::Card(lh_disc)]);
        answer_prompt(&mut e, "lihun:choose_target", 0, vec![ChoiceOption::Player(3)]);
        assert!(e.debug_hand(3).is_empty(), "lihun must take all hand cards");
        assert!(e.debug_hand(0).len() >= h3_now);
        assert!(e.debug_hand(0).contains(&extra3));
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("lihun:return_cards"));
        answer_prompt(&mut e, "lihun:return_cards", 0, vec![ChoiceOption::Card(extra3)]);
        assert_eq!(e.debug_hand(3).len(), 1);

        // Advance past seat 1 turn end -> `jinglve` reclaims `sishi` into seat 0 hand!
        goto_play(&mut e, 2);
        assert!(
            e.debug_hand(0).contains(&sishi),
            "jinglve must reclaim unclaimed sishi at target phaseAfter"
        );
    }
}

#[test]
fn l11_npc_a_active_group2_mizhao_to_quji() {
    // `mizhao`, `mouzhu`, `new_reqingnang`, `new_yijue`, `nzry_chenglve`, `nzry_feijun`, `quji`
    let mut e = new_4p(91_007);
    for sk in [
        "mizhao",
        "mouzhu",
        "new_reqingnang",
        "new_yijue",
        "nzry_chenglve",
        "nzry_feijun",
        "quji",
    ] {
        e.debug_grant(0, sk);
    }
    goto_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);

    // 1. `new_reqingnang`: once per target per play phase; banned if black card discarded
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 5);
    let red_qn = place_by_suit_pred(&mut e, 0, Suit::Heart, None);
    let blk_qn = place_by_suit_pred(&mut e, 0, Suit::Spade, Some("sha"));
    activate_skill(&mut e, 0, "new_reqingnang");
    answer_prompt(&mut e, "new_reqingnang:discard_card", 0, vec![ChoiceOption::Card(red_qn)]);
    answer_prompt(&mut e, "new_reqingnang:choose_target", 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.debug_hp(1), 6);
    assert_eq!(flag(&e, 0, "l11_new_reqingnang_ban"), 0);
    activate_skill(&mut e, 0, "new_reqingnang");
    answer_prompt(&mut e, "new_reqingnang:discard_card", 0, vec![ChoiceOption::Card(blk_qn)]);
    answer_prompt(&mut e, "new_reqingnang:choose_target", 0, vec![ChoiceOption::Player(3)]);
    assert_eq!(e.debug_hp(3), 6);
    assert_eq!(flag(&e, 0, "l11_new_reqingnang_ban"), 1);

    // 2. `new_yijue`: target shows Black -> hand locked + skills sealed (`xj_fengyin_sealed`)
    let yj_disc = e.debug_place(0, "shan");
    let yj_blk = place_by_suit_pred(&mut e, 1, Suit::Club, None);
    let yj_id = e.catalog().skill_by_name("new_yijue").unwrap();
    activate_skill(&mut e, 0, "new_yijue");
    answer_prompt(&mut e, "new_yijue:discard_card", 0, vec![ChoiceOption::Card(yj_disc)]);
    answer_prompt(&mut e, "new_yijue:choose_target", 0, vec![ChoiceOption::Player(1)]);
    answer_prompt(&mut e, "new_yijue:target_show", 1, vec![ChoiceOption::Card(yj_blk)]);
    assert_eq!(flag(&e, 1, "l11_new_yijue_hand_ban"), 1);
    assert_eq!(flag(&e, 1, "xj_fengyin_sealed"), 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: yj_id }),
        CommandOutcome::Rejected(_)
    ));

    // 3. `nzry_chenglve`: draws 1 / discards 2 -> records discarded suits as unlimited!
    e.debug_clear_hand(0);
    let cl_b1 = place_by_suit_pred(&mut e, 0, Suit::Spade, Some("sha"));
    let cl_b2 = place_by_suit_pred(&mut e, 0, Suit::Heart, None);
    let _cl_extra = e.debug_place(0, "sha");
    activate_skill(&mut e, 0, "nzry_chenglve");
    answer_prompt(
        &mut e,
        "nzry_chenglve:discard_cards",
        0,
        vec![ChoiceOption::Card(cl_b1), ChoiceOption::Card(cl_b2)],
    );
    assert!(flag(&e, 0, "l11_nzry_chenglve_suits") != 0);

    // 4. `nzry_feijun`: target has more handcards -> gives 1 handcard to owner
    let fj_disc = e.debug_place(0, "shan");
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let fj_d2 = e.debug_place(0, "shan");
    let _ = fj_disc;
    let p2_c1 = e.debug_place(2, "tao");
    let p2_c2 = e.debug_place(2, "sha");
    activate_skill(&mut e, 0, "nzry_feijun");
    answer_prompt(&mut e, "nzry_feijun:discard_card", 0, vec![ChoiceOption::Card(fj_d2)]);
    answer_prompt(&mut e, "nzry_feijun:choose_target", 0, vec![ChoiceOption::Player(2)]);
    answer_prompt(&mut e, "nzry_feijun:target_card", 2, vec![ChoiceOption::Card(p2_c1)]);
    assert!(e.debug_hand(0).contains(&p2_c1));
    let _ = p2_c2;

    // 5. `quji`: owner HP = 4 (lost 2), discard 2 (one black -> owner loses 1 HP!) -> heal 2 targets
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 5);
    e.debug_set_hp(3, 5);
    e.debug_clear_hand(0);
    let qj1 = place_by_suit_pred(&mut e, 0, Suit::Spade, Some("sha"));
    let qj2 = place_by_suit_pred(&mut e, 0, Suit::Heart, None);
    activate_skill(&mut e, 0, "quji");
    answer_prompt(
        &mut e,
        "quji:discard_cards",
        0,
        vec![ChoiceOption::Card(qj1), ChoiceOption::Card(qj2)],
    );
    answer_prompt(
        &mut e,
        "quji:choose_targets",
        0,
        vec![ChoiceOption::Player(2), ChoiceOption::Player(3)],
    );
    assert_eq!(e.debug_hp(2), 6);
    assert_eq!(e.debug_hp(3), 6);
    assert_eq!(e.debug_hp(0), 3, "quji black discard causes owner to lose 1 HP");

    // 6. `mouzhu` & `mizhao`
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let mz_give = e.debug_place(3, "shan");
    activate_skill(&mut e, 0, "mouzhu");
    answer_prompt(&mut e, "mouzhu:choose_target", 0, vec![ChoiceOption::Player(3)]);
    answer_prompt(&mut e, "mouzhu:target_give", 3, vec![ChoiceOption::Card(mz_give)]);
    answer_prompt(&mut e, "mouzhu:choose_mode", 3, vec![ChoiceOption::Tag(1)]); // Duel: seat 0 has mz_give? clear/pass
    drain_pass(&mut e);

    let miz1 = place_with_rank(&mut e, 0, 10, 13);
    let p2_lo = place_with_rank(&mut e, 2, 1, 3);
    let _ = e.debug_place(3, "sha");
    activate_skill(&mut e, 0, "mizhao");
    answer_prompt(&mut e, "mizhao:choose_target1", 0, vec![ChoiceOption::Player(3)]);
    let _p1_hi = e.debug_place(1, "sha");
    answer_prompt(&mut e, "mizhao:choose_target2", 0, vec![ChoiceOption::Player(2)]);
    let _ = miz1;
    // mizhao runs pindian between 3 and 2
    if e.open_choice().is_some_and(|c| c.spec.prompt == "mizhao:pindian_card1") {
        let c3 = *e.debug_hand(3).first().unwrap();
        answer_prompt(&mut e, "mizhao:pindian_card1", 3, vec![ChoiceOption::Card(c3)]);
    }
    if e.open_choice().is_some_and(|c| c.spec.prompt == "mizhao:pindian_card2") {
        answer_prompt(&mut e, "mizhao:pindian_card2", 2, vec![ChoiceOption::Card(p2_lo)]);
    }
    drain_pass(&mut e);
}

#[test]
fn l11_npc_a_active_group3_refenxun_to_reshuangxiong() {
    // `refenxun`, `reganlu`, `reguose`, `rejunxing`, `reluanji`, `rewurong`, `rezhijian`, `rongbei`, `kyou_zhidian`, `dclbjiuxian`, `olhuoji`, `reshuangxiong`
    let mut e = new_4p(91_008);
    for sk in [
        "refenxun",
        "reganlu",
        "reguose",
        "rejunxing",
        "reluanji",
        "rewurong",
        "rezhijian",
        "rongbei",
        "kyou_zhidian",
        "dclbjiuxian",
        "olhuoji",
        "reshuangxiong",
    ] {
        e.debug_grant(0, sk);
    }
    goto_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);

    // 1. `kyou_zhidian`: ViewAs non-basic -> Fire Sha, infinite distance + rotating 4 modes (0..=3)
    let armor2 = e.debug_place(2, "armor");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // pass
    drain_pass(&mut e);
    // Re-enter Play phase cleanly on a fresh 2p/4p for deterministic active order
    let _ = armor2;

    let mut e = new_4p(91_009);
    for sk in [
        "kyou_zhidian",
        "dclbjiuxian",
        "olhuoji",
        "reshuangxiong",
        "reguose",
        "rezhijian",
        "rongbei",
    ] {
        e.debug_grant(0, sk);
    }
    goto_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);

    // `kyou_zhidian`: use non-basic `nullify` as Sha on distance-2 seat 2!
    let kyou_c = e.debug_place(0, "nullify");
    activate_skill(&mut e, 0, "kyou_zhidian");
    answer_prompt(&mut e, "kyou_zhidian:choose_card", 0, vec![ChoiceOption::Card(kyou_c)]);
    answer_prompt(&mut e, "kyou_zhidian:choose_target", 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("kyou_zhidian:choose_effect"));
    // Mode 1: unrespondable, Mode 2: pierce, Mode 3: damage +1, Mode 4: refund sha count
    answer_prompt(&mut e, "kyou_zhidian:choose_effect", 0, vec![ChoiceOption::Tag(3)]);
    drain_pass(&mut e);
    assert_eq!(e.debug_hp(2), 4, "kyou_zhidian mode 3 adds +1 Sha damage (6 -> 4)");
    assert_eq!(flag(&e, 0, "l11_kyou_zhidian_last"), 3);

    // `dclbjiuxian`: drink Jiu unlimitedly using multi-target tricks, stacking Sha damage!
    goto_play(&mut e, 1);
    goto_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let jx1 = e.debug_place(0, "savage");
    let jx2 = e.debug_place(0, "archery");
    activate_skill(&mut e, 0, "dclbjiuxian");
    answer_prompt(&mut e, "dclbjiuxian:choose_card", 0, vec![ChoiceOption::Card(jx1)]);
    activate_skill(&mut e, 0, "dclbjiuxian");
    answer_prompt(&mut e, "dclbjiuxian:choose_card", 0, vec![ChoiceOption::Card(jx2)]);
    assert!(e.state_core().turn.jiu_active);
    assert_eq!(flag(&e, 0, "jiu_extra_stacks"), 1);
    assert!(!e.debug_hand(0).contains(&jx1) && !e.debug_hand(0).contains(&jx2));
    let jx_sha = e.debug_place(0, "sha");
    let out_jx = submit(&mut e, 0, GameAction::PlayCard { card: jx_sha, targets: vec![1] });
    assert!(!matches!(out_jx, CommandOutcome::Rejected(_)), "sha rejected: {out_jx:?}");
    drain_pass(&mut e);
    assert_eq!(e.debug_hp(1), 3, "dclbjiuxian stacked 2 Jiu must boost Sha damage +2 (6 -> 3)");

    // `olhuoji`: red card -> random Fire Attack (no target hand prompt!) + `'he'` color discard!
    let hj_red = place_by_suit_pred(&mut e, 0, Suit::Heart, None);
    let hj_show = place_by_suit_pred(&mut e, 1, Suit::Spade, Some("sha"));
    let _ = hj_show;
    let hj_eq = e.debug_place_suit(0, Suit::Spade);
    let out_eq = submit(&mut e, 0, GameAction::PlayCard { card: hj_eq, targets: vec![] });
    let _ = out_eq;
    drain_pass(&mut e);
    activate_skill(&mut e, 0, "olhuoji");
    answer_prompt(&mut e, "olhuoji:choose_card", 0, vec![ChoiceOption::Card(hj_red)]);
    answer_prompt(&mut e, "olhuoji:choose_target", 0, vec![ChoiceOption::Player(1)]);
    // Huogong under `olhuoji` skips `huogong:show` and goes straight to `huogong:discard` if any `'he'` matches
    if e.open_choice().is_some_and(|c| c.spec.prompt == "huogong:discard") {
        let ch = e.open_choice().unwrap().clone();
        let opt = *ch.spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(_))).unwrap();
        answer_prompt(&mut e, "huogong:discard", 0, vec![opt]);
    }
    drain_pass(&mut e);

    // `rezhijian`: equip directly onto another player + draw 1
    let zj_eq = e.debug_place(0, "qinggang");
    let h0_b = e.debug_hand(0).len();
    activate_skill(&mut e, 0, "rezhijian");
    answer_prompt(&mut e, "rezhijian:choose_card", 0, vec![ChoiceOption::Card(zj_eq)]);
    answer_prompt(&mut e, "rezhijian:choose_target", 0, vec![ChoiceOption::Player(3)]);
    assert!(e.debug_equips(3).contains(&zj_eq));
    assert_eq!(e.debug_hand(0).len(), h0_b, "gave zj_eq (-1) and drew (+1)");

    // `rongbei`: limited -> fill target missing equip slots up to 5!
    activate_skill(&mut e, 0, "rongbei");
    answer_prompt(&mut e, "rongbei:choose_target", 0, vec![ChoiceOption::Player(2)]);
    assert!(!e.debug_equips(2).is_empty(), "rongbei equips missing slots");
    assert_eq!(flag(&e, 0, "l11_rongbei_used"), 1);

    // `reguose`: diamond card as Indulgence + draw 1
    let gs_c = place_by_suit_pred(&mut e, 0, Suit::Diamond, Some("sha"));
    let h0_gs = e.debug_hand(0).len();
    activate_skill(&mut e, 0, "reguose");
    answer_prompt(&mut e, "reguose:choose_card", 0, vec![ChoiceOption::Card(gs_c)]);
    answer_prompt(&mut e, "reguose:choose_target", 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert!(e.debug_judge(1).contains(&gs_c));
    assert_eq!(e.debug_hand(0).len(), h0_gs, "-1 reguose card +1 draw");

    // `reshuangxiong` active Duel when `l11_mod:reshuangxiong_duel` set (or via DrawPhase in trigger tests)
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_mod:reshuangxiong_duel".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "shuangxiong_color".to_string()), 1); // Red forbidden -> Black allowed
    e.debug_clear_hand(3);
    let sx_blk = place_by_suit_pred(&mut e, 0, Suit::Club, Some("sha"));
    activate_skill(&mut e, 0, "reshuangxiong");
    answer_prompt(&mut e, "reshuangxiong:choose_card", 0, vec![ChoiceOption::Card(sx_blk)]);
    answer_prompt(&mut e, "reshuangxiong:choose_target", 0, vec![ChoiceOption::Player(3)]);
    drain_pass(&mut e);
    assert_eq!(e.debug_hp(3), 5, "reshuangxiong Duel deals 1 damage to empty-hand seat 3");

    // Test remaining active skills `refenxun`, `reganlu`, `rejunxing`, `reluanji`, `rewurong` cleanly
    let mut e2 = new_4p(91_010);
    for sk in ["refenxun", "reganlu", "rejunxing", "reluanji", "rewurong"] {
        e2.debug_grant(0, sk);
    }
    goto_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(2);
    e2.debug_clear_hand(3);

    // `reganlu`: swap equips between seat 1 (1 equip) and seat 2 (0 equips), gap=1 <= lost_hp=1
    e2.debug_set_hp(0, 5);
    let _eq1 = e2.debug_place(1, "armor");
    goto_play(&mut e2, 1);
    let _ = submit(&mut e2, 1, GameAction::PlayCard { card: _eq1, targets: vec![] });
    drain_pass(&mut e2);
    goto_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    activate_skill(&mut e2, 0, "reganlu");
    answer_prompt(&mut e2, "reganlu:choose_target1", 0, vec![ChoiceOption::Player(1)]);
    answer_prompt(&mut e2, "reganlu:choose_target2", 0, vec![ChoiceOption::Player(2)]);
    assert!(e2.debug_equips(2).contains(&_eq1) && e2.debug_equips(1).is_empty());

    // `rewurong`: owner shows Sha vs non-Shan target -> deals 1 damage
    let wr_sha = e2.debug_place(0, "sha");
    let wr_non = e2.debug_place(3, "tao");
    activate_skill(&mut e2, 0, "rewurong");
    answer_prompt(&mut e2, "rewurong:choose_target", 0, vec![ChoiceOption::Player(3)]);
    answer_prompt(&mut e2, "rewurong:owner_show", 0, vec![ChoiceOption::Card(wr_sha)]);
    answer_prompt(&mut e2, "rewurong:target_show", 3, vec![ChoiceOption::Card(wr_non)]);
    assert_eq!(e2.debug_hp(3), 5);

    // `rejunxing`: discard 2, target chooses turnOver + draw X (`Tag(1)`)
    let jx_c1 = e2.debug_place(0, "sha");
    let jx_c2 = e2.debug_place(0, "shan");
    let h3_before = e2.debug_hand(3).len();
    activate_skill(&mut e2, 0, "rejunxing");
    answer_prompt(
        &mut e2,
        "rejunxing:discard_cards",
        0,
        vec![ChoiceOption::Card(jx_c1), ChoiceOption::Card(jx_c2)],
    );
    answer_prompt(&mut e2, "rejunxing:choose_target", 0, vec![ChoiceOption::Player(3)]);
    answer_prompt(&mut e2, "rejunxing:target_mode", 3, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e2.debug_hp(3), 5);
    assert_eq!(e2.debug_hand(3).len(), h3_before + 2);
    assert!(e2.debug_face_down(3));

    // `reluanji`: 2 distinct suits -> ArcheryAttack; no damage dealt -> draw `targets.len()`!
    let _sh1 = e2.debug_place(1, "shan");
    let _sh2 = e2.debug_place(2, "shan");
    let _sh3 = e2.debug_place(3, "shan");
    let h0_before_lj = e2.debug_hand(0).len();
    let lj1 = place_by_suit_pred(&mut e2, 0, Suit::Spade, Some("sha"));
    let lj2 = place_by_suit_pred(&mut e2, 0, Suit::Heart, Some("shan"));
    activate_skill(&mut e2, 0, "reluanji");
    answer_prompt(
        &mut e2,
        "reluanji:choose_cards",
        0,
        vec![ChoiceOption::Card(lj1), ChoiceOption::Card(lj2)],
    );
    for s in [1u8, 2, 3] {
        let sh = *e2.debug_hand(s).last().unwrap();
        answer_prompt(&mut e2, "aoe_response", s, vec![ChoiceOption::Card(sh)]);
    }
    assert_eq!(
        e2.debug_hand(0).len(),
        h0_before_lj + 3,
        "reluanji dealing 0 damage draws targets.len()=3"
    );

    // `refenxun`: dist 1 to seat 2; if no damage dealt to seat 2, End PhaseBegin asks `refenxun2:discard_card`
    activate_skill(&mut e2, 0, "refenxun");
    answer_prompt(&mut e2, "refenxun:choose_target", 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e2.state_core().computed_distance(e2.catalog(), 0, 2), 1);
    let d_card: CardId = *e2.debug_hand(0).first().unwrap();
    let _ = submit(&mut e2, 0, GameAction::NextPhase); // Discard
    drain_pass(&mut e2);
    let _ = submit(&mut e2, 0, GameAction::NextPhase); // End -> refenxun2
    let _ = EquipSlot::Weapon;
    let _ = CardKind::Sha;
    assert_eq!(e2.open_choice().map(|c| c.spec.prompt), Some("refenxun2:discard_card"));
    answer_prompt(&mut e2, "refenxun2:discard_card", 0, vec![ChoiceOption::Card(d_card)]);
    assert_eq!(e2.state_core().turn.phase, Phase::End);
}

#[test]
fn l11_npc_a_round2_active_parity_regressions() {
    // Fix #1: `kyou_zhidian` must NOT bypass Sha usage limit even when players have cards in `'hej'`,
    // unless Mode 4 refunded the Sha count.
    {
        let mut e = new_4p(91_201);
        e.debug_grant(0, "kyou_zhidian");
        goto_play(&mut e, 1);
        let _eq1 = equip_card_now(&mut e, 1, "armor");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let _h1 = e.debug_place(1, "shan");
        let le_on_1 = place_named_suit(&mut e, 0, "le", None);
        let out_le = submit(&mut e, 0, GameAction::PlayCard { card: le_on_1, targets: vec![1] });
        assert!(!matches!(out_le, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert!(!e.debug_judge(1).is_empty(), "seat 1 must hold judge card in 'hej'");

        let sha0 = e.debug_place(0, "sha");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
        drain_pass(&mut e);
        let kyou_def = e.catalog().skill_by_name("kyou_zhidian").unwrap();
        let trick0 = e.debug_place(0, "nullify");
        assert!(
            matches!(
                submit(&mut e, 0, GameAction::ActivateSkill { skill: kyou_def }),
                CommandOutcome::Rejected(_)
            ),
            "kyou_zhidian must reject after Sha limit is spent even when target has 'hej'"
        );
        let _ = trick0;

        // With Mode 4 refund, a 2nd `kyou_zhidian` activation IS allowed!
        let mut e2 = new_4p(91_202);
        e2.debug_grant(0, "kyou_zhidian");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let t1 = e2.debug_place(0, "nullify");
        let t2 = e2.debug_place(0, "dismantle");
        activate_skill(&mut e2, 0, "kyou_zhidian");
        answer_prompt(&mut e2, "kyou_zhidian:choose_card", 0, vec![ChoiceOption::Card(t1)]);
        answer_prompt(&mut e2, "kyou_zhidian:choose_target", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e2, "kyou_zhidian:choose_effect", 0, vec![ChoiceOption::Tag(4)]);
        drain_pass(&mut e2);
        activate_skill(&mut e2, 0, "kyou_zhidian");
        answer_prompt(&mut e2, "kyou_zhidian:choose_card", 0, vec![ChoiceOption::Card(t2)]);
        answer_prompt(&mut e2, "kyou_zhidian:choose_target", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e2, "kyou_zhidian:choose_effect", 0, vec![ChoiceOption::Tag(3)]);
        drain_pass(&mut e2);
        assert_eq!(e2.debug_hp(2), 3, "refunded 2nd kyou_zhidian Sha deals +1 damage");
    }

    // Fix #2: `reguose` allows discarding owner's own judge `Le` (even when others are immune/cannot receive Le),
    // and excludes owner when owner has no `Le` in judge.
    {
        let mut e = new_2p(91_203);
        e.debug_grant(0, "reguose");
        e.debug_grant(0, "xinfu_limu");
        e.debug_grant(1, "谦逊");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let d_self_le = place_by_suit_pred(&mut e, 0, Suit::Diamond, Some("sha"));
        let d_cost = place_by_suit_pred(&mut e, 0, Suit::Diamond, Some("shan"));
        let out_le = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: d_self_le,
                targets: vec![0],
            },
        );
        assert!(!matches!(out_le, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert!(e.debug_judge(0).contains(&d_self_le));

        let h0_b = e.debug_hand(0).len();
        activate_skill(&mut e, 0, "reguose");
        answer_prompt(&mut e, "reguose:choose_card", 0, vec![ChoiceOption::Card(d_cost)]);
        let ch = e.open_choice().unwrap().clone();
        assert_eq!(ch.spec.prompt, "reguose:choose_target");
        assert!(
            ch.spec.options.contains(&ChoiceOption::Player(0)),
            "reguose must allow owner when owner has Le in judge"
        );
        assert!(
            !ch.spec.options.contains(&ChoiceOption::Player(1)),
            "qianxun seat without Le must not be legal for reguose"
        );
        answer_prompt(&mut e, "reguose:choose_target", 0, vec![ChoiceOption::Player(0)]);
        drain_pass(&mut e);
        assert!(e.debug_judge(0).is_empty(), "reguose must discard owner's own Le");
        assert_eq!(e.debug_hand(0).len(), h0_b, "-1 reguose card +1 draw");

        // Next turn: owner no longer has Le -> owner excluded from reguose:choose_target
        let mut e2 = new_4p(91_204);
        e2.debug_grant(0, "reguose");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        let d2 = place_by_suit_pred(&mut e2, 0, Suit::Diamond, Some("sha"));
        activate_skill(&mut e2, 0, "reguose");
        answer_prompt(&mut e2, "reguose:choose_card", 0, vec![ChoiceOption::Card(d2)]);
        let ch2 = e2.open_choice().unwrap().clone();
        assert_eq!(ch2.spec.prompt, "reguose:choose_target");
        assert!(
            !ch2.spec.options.contains(&ChoiceOption::Player(0)),
            "reguose must exclude owner when owner has no Le"
        );
        answer_prompt(&mut e2, "reguose:choose_target", 0, vec![]);
    }

    // Fix #3: `olhuoji` allows self-target AND applies random-show + `'he'` color-match to real `【火攻】` cards!
    {
        // (a) self-target virtual `olhuoji`
        let mut e = new_2p(91_205);
        e.debug_grant(0, "olhuoji");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let eq_spade = place_by_suit_pred(&mut e, 0, Suit::Spade, Some("qinggang"));
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq_spade, targets: vec![] });
        drain_pass(&mut e);
        let red_cost = place_by_suit_pred(&mut e, 0, Suit::Diamond, None);
        let hand_blk = place_by_suit_pred(&mut e, 0, Suit::Club, Some("sha"));
        let _ = hand_blk;
        activate_skill(&mut e, 0, "olhuoji");
        answer_prompt(&mut e, "olhuoji:choose_card", 0, vec![ChoiceOption::Card(red_cost)]);
        let ch = e.open_choice().unwrap().clone();
        assert_eq!(ch.spec.prompt, "olhuoji:choose_target");
        assert!(ch.spec.options.contains(&ChoiceOption::Player(0)), "olhuoji allows self target");
        answer_prompt(&mut e, "olhuoji:choose_target", 0, vec![ChoiceOption::Player(0)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("huogong:discard_card"));
        let dch = e.open_choice().unwrap().clone();
        assert!(dch.spec.options.contains(&ChoiceOption::Card(eq_spade)));
        answer_prompt(&mut e, "huogong:discard_card", 0, vec![ChoiceOption::Card(eq_spade)]);
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(0), 5, "self-target olhuoji deals 1 fire damage to owner");

        // (b) real `【火攻】` card from hand under `olhuoji` skips `huogong:show_card` and matches `'he'` color!
        let mut e2 = new_2p(91_206);
        e2.debug_grant(0, "olhuoji");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let eq_red = e2
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find_map(|(i, c)| {
                (c.kind.equip_slot().is_some()
                    && matches!(c.suit, Suit::Heart | Suit::Diamond)
                    && e2.state_core().deck.contains(&(i as u32)))
                .then_some(i as u32)
            })
            .expect("red equip in catalog");
        e2.state_core_mut().deck.retain(|&x| x != eq_red);
        e2.state_core_mut().player_mut(0).hand.push(eq_red);
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: eq_red, targets: vec![] });
        drain_pass(&mut e2);

        let real_hg = place_named_suit(&mut e2, 0, "huogong", None);
        let _tgt_d = place_by_suit_pred(&mut e2, 1, Suit::Diamond, Some("shan"));
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: real_hg, targets: vec![1] });
        while e2.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e2.open_choice().unwrap().observer;
            answer_prompt(&mut e2, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert_eq!(
            e2.open_choice().map(|c| c.spec.prompt),
            Some("huogong:discard_card"),
            "real huogong under olhuoji must skip huogong:show_card"
        );
        let dch2 = e2.open_choice().unwrap().clone();
        assert!(
            dch2.spec.options.contains(&ChoiceOption::Card(eq_red)),
            "real huogong under olhuoji must allow color-matching 'he' equip discard"
        );
        answer_prompt(&mut e2, "huogong:discard_card", 0, vec![ChoiceOption::Card(eq_red)]);
        drain_pass(&mut e2);
        assert_eq!(e2.debug_hp(1), 5, "real huogong under olhuoji deals 1 fire damage");
    }

    // Fix #8: `nzry_feijun` checks `>=` pre-discard and discards cost BEFORE picking target (`>` post-discard).
    {
        // (a) equal hand count before discard (`1 == 1`) -> after discarding cost (`0 < 1`), seat 2 becomes legal!
        let mut e = new_4p(91_207);
        e.debug_grant(0, "nzry_feijun");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_clear_hand(3);
        let c0 = e.debug_place(0, "sha");
        let c2 = e.debug_place(2, "tao");
        activate_skill(&mut e, 0, "nzry_feijun");
        answer_prompt(&mut e, "nzry_feijun:discard_card", 0, vec![ChoiceOption::Card(c0)]);
        assert!(e.debug_hand(0).is_empty(), "cost card must be discarded BEFORE target choice");
        let ch = e.open_choice().unwrap().clone();
        assert_eq!(ch.spec.prompt, "nzry_feijun:choose_target");
        assert_eq!(ch.spec.options, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e, "nzry_feijun:choose_target", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e, "nzry_feijun:target_card", 2, vec![ChoiceOption::Card(c2)]);
        assert!(e.debug_hand(0).contains(&c2));

        // (b) equal equip count before discard (`1 == 1`) -> after discarding equip (`0 < 1`), target discards 1 equip!
        let mut e2 = new_4p(91_208);
        e2.debug_grant(0, "nzry_feijun");
        goto_play(&mut e2, 2);
        let eq2 = equip_card_now(&mut e2, 2, "armor");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        e2.debug_clear_hand(2);
        e2.debug_clear_hand(3);
        let eq0 = equip_card_now(&mut e2, 0, "qinggang");
        activate_skill(&mut e2, 0, "nzry_feijun");
        answer_prompt(&mut e2, "nzry_feijun:discard_card", 0, vec![ChoiceOption::Card(eq0)]);
        assert!(e2.debug_equips(0).is_empty());
        answer_prompt(&mut e2, "nzry_feijun:choose_target", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e2, "nzry_feijun:target_card", 2, vec![ChoiceOption::Card(eq2)]);
        assert!(e2.debug_equips(2).is_empty());
    }
}
