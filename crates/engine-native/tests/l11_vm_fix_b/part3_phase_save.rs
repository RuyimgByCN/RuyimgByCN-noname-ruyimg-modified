use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_vm_fix_b_qiangzhi_rejiangchi_repolu_yinghun_zaiqi_zaoxian_zhaohuo() {
    // 1. qiangzhi + qiangzhi_draw + rejiangchi (jiangchi2 & rejiangchi3) + cleanup
    let mut e = start_4p_taixu(2026_8201);
    e.debug_grant(0, "qiangzhi");
    e.debug_grant(0, "rejiangchi");
    let t_sha = e.debug_place(1, "sha"); // Basic card on seat 1
    let disc_j = e.debug_place(0, "shan");
    let play_sha = e.debug_place(0, "sha");
    let play_sha2 = e.debug_place(0, "sha");

    e.debug_set_turn(3, Phase::End, 1);
    advance_to_play(&mut e, 0);
    // qiangzhi + rejiangchi both fire at Phase::Play begin (either order)!
    for _ in 0..8 {
        let Some(ch) = e.open_choice() else {
            break;
        };
        match ch.spec.prompt {
            "trigger_order" => {
                let first = ch.spec.options[0];
                let _ = answer(&mut e, 0, vec![first]);
            }
            "l11_vm_b:qiangzhi_target" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
            }
            "l11_vm_b:qiangzhi_card" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);
            }
            "l11_vm_b:rejiangchi_mode" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(0)]); // discard 1 -> jiangchi2
            }
            "l11_vm_b:rejiangchi_discard" => {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Card(disc_j)]);
            }
            _ => break,
        }
    }
    let _ = t_sha;
    assert!(e.debug_has_skill(0, "qiangzhi_draw"));
    assert!(e.debug_has_skill(0, "jiangchi2"));

    // Using basic Sha triggers qiangzhi_draw optional draw + jiangchi2 allows 2nd Sha unlimited range!
    let _ = play_card(&mut e, 0, play_sha, vec![2]);
    for _ in 0..3 {
        if let Some(ch) = e.open_choice() {
            if ch.spec.prompt == "trigger_confirm" {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
            } else {
                drain_open_with_ai(&mut e, 4);
            }
        }
    }
    assert!(!matches!(
        play_card(&mut e, 0, play_sha2, vec![2]),
        CommandOutcome::Rejected(_)
    ), "jiangchi2 permits 2nd Sha at distance 2");
    drain_open_with_ai(&mut e, 6);

    // End Play -> jiangchi2 and qiangzhi_draw removed!
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(!e.debug_has_skill(0, "qiangzhi_draw"));
    assert!(!e.debug_has_skill(0, "jiangchi2"));

    // 2. repolu: killer draws N+1, dead owner draws N+2
    let mut e2 = start_4p_taixu(2026_8202);
    e2.debug_set_identity_roles(vec![
        engine_native::state::Role::Loyalist,
        engine_native::state::Role::Traitor,
        engine_native::state::Role::Rebel,
        engine_native::state::Role::Lord,
    ]);
    e2.debug_grant(0, "repolu");
    e2.debug_set_hp(1, 1);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(3);
    let kill_sha = e2.debug_place(0, "sha");
    let _ = play_card(&mut e2, 0, kill_sha, vec![1]);
    let _ = expect_prompt(&e2, "l11_vm_b:repolu_targets");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(0), ChoiceOption::Player(3)]);
    assert_eq!(e2.debug_hand(0).len(), 1, "first repolu draws 1");
    assert_eq!(e2.debug_hand(3).len(), 1);

    // Now seat 2 kills seat 0 -> seat 0's player:die triggers repolu drawing 2!
    advance_to_play(&mut e2, 2);
    e2.debug_clear_hand(2);
    e2.debug_clear_hand(0);
    e2.debug_set_hp(0, 1);
    e2.debug_clear_hand(3);
    let kill0 = e2.debug_place(2, "sha");
    let _ = play_card(&mut e2, 2, kill0, vec![0]);
    while let Some(ch) = e2.open_choice() {
        if ch.spec.prompt == "l11_vm_b:repolu_targets" {
            break;
        }
        drain_open_with_ai(&mut e2, 1);
    }
    let _ = expect_prompt(&e2, "l11_vm_b:repolu_targets");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(3)]);
    assert_eq!(e2.debug_hand(3).len(), 2, "second repolu draws 2");

    // 3. yinghun (lost=1 auto, lost=2 mode choice) & zaoxian & zaiqi & zhaohuo
    let mut e3 = start_4p_taixu(2026_8203);
    e3.debug_grant(0, "yinghun");
    e3.debug_grant(0, "zaoxian");
    e3.debug_grant(0, "zaiqi");
    e3.debug_grant(3, "zhaohuo");
    e3.debug_set_max_hp(0, 5);
    e3.debug_set_hp(0, 3); // lost = 2
    e3.debug_set_max_hp(3, 4);
    e3.debug_set_hp(3, 1);
    // 3 cards in retuntian pile -> zaoxian wakes at Prepare!
    let p_cards: Vec<_> = (0..3).map(|_| e3.debug_place(0, "sha")).collect();
    for &c in &p_cards {
        e3.state_core_mut().player_mut(0).hand.retain(|&x| x != c);
    }
    e3.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "retuntian".to_string()), p_cards);

    // Place Heart & Spade on top of deck for zaiqi!
    let _z_spade = e3.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let _z_heart = e3.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    e3.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e3, 3, GameAction::NextPhase);
    // Prepare triggers zaoxian + yinghun; then DrawPhase triggers zaiqi!
    let mut seen_e3 = Vec::new();
    for _ in 0..12 {
        let Some(ch) = e3.open_choice() else {
            if e3.state_core().turn.current == 0 && e3.state_core().turn.phase == Phase::Judge {
                let _ = submit(&mut e3, 0, GameAction::NextPhase);
                continue;
            }
            break;
        };
        seen_e3.push((ch.spec.prompt, ch.observer));
        match ch.spec.prompt {
            "l11_vm_b:yinghun_target" => {
                let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
            }
            "l11_vm_b:yinghun_mode" => {
                let _ = answer(&mut e3, 0, vec![ChoiceOption::Tag(0)]); // draw 2, disc 1
            }
            "l11_vm_b:yinghun_discard" => {
                let first = ch.spec.options[0];
                let _ = answer(&mut e3, 2, vec![first]);
            }
            "trigger_confirm" => {
                let _z_heart = e3.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
                let _ = answer(&mut e3, 0, vec![ChoiceOption::Confirm]); // zaiqi
            }
            _ => break,
        }
    }
    assert!(e3.debug_has_skill(0, "jixi"), "zaoxian granted jixi");
    assert_eq!(e3.state_core().player(0).max_hp, 4, "zaoxian lost 1 max_hp");
    assert!(
        e3.debug_hp(0) >= 4,
        "zaiqi recovered Heart card (hp={}, max_hp={}, phase={:?}, seen={seen_e3:?}, open={:?})",
        e3.debug_hp(0),
        e3.state_core().player(0).max_hp,
        e3.state_core().turn.phase,
        e3.open_choice().map(|c| (c.spec.prompt, c.observer))
    );

    // Dying trigger for zhaohuo on seat 3: seat 1 drops to 0 -> seat 3 max_hp becomes 1 & draws 3!
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(1);
    e3.debug_clear_hand(3);
    e3.debug_set_hp(1, 1);
    for _ in 0..3 {
        let _ = e3.debug_set_deck_top_by(|d| d.kind == CardKind::Sha);
    }
    let sha_dying = e3.debug_place(0, "sha");
    let out_dying = play_card(&mut e3, 0, sha_dying, vec![1]);
    assert!(!matches!(out_dying, CommandOutcome::Rejected(_)), "sha_dying rejected: {out_dying:?}");
    drain_open_with_ai(&mut e3, 12);
    assert_eq!(e3.state_core().player(3).max_hp, 1, "zhaohuo reduces max_hp to 1");
    assert_eq!(e3.debug_hand(3).len(), 3, "zhaohuo draws lost max_hp = 3");
}

#[test]
fn l11_vm_fix_b_zhengjian_zhiyi_rezhiyi_twshengxi_tiaojiyanmei_and_qinguo() {
    // 1. zhengjian + zhengjian_mark + zhiyi + rezhiyi
    let mut e = start_4p_taixu(2026_8204);
    e.debug_grant(0, "zhengjian");
    e.debug_grant(0, "zhiyi");
    e.debug_grant(2, "rezhiyi");

    // At seat 0's Phase::End, give zhengjian_mark to seat 1
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    if e.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    let _ = expect_prompt(&e, "l11_vm_b:zhengjian_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.debug_has_skill(1, "zhengjian_mark"));

    // Seat 1's turn: seat 1 uses 2 cards -> zhengjian_mark == 2
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let s1a = e.debug_place(1, "wuzhong");
    let _ = play_card(&mut e, 1, s1a, vec![]);
    let shan2 = e.debug_place(2, "shan");
    let s1b = e.debug_place(1, "sha");
    let _ = play_card(&mut e, 1, s1b, vec![2]);
    // Seat 2 answers Shan (1st basic -> rezhiyi records Shan!)
    if let Some(ch) = e.open_choice() {
        if ch.observer == 2 {
            let _ = answer(&mut e, 2, vec![ChoiceOption::Card(shan2)]);
        }
    }
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(1, "l11_vm_b:zhengjian_cnt".to_string())).copied(),
        Some(2)
    );

    // At seat 1's End phase -> rezhiyi triggers on seat 2; Shan has no use target -> draws 1!
    e.debug_clear_hand(2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    if e.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    assert_eq!(e.debug_hand(2).len(), 1, "rezhiyi fallback draws 1 when only Shan used/responded");

    // Advance back to seat 0's TurnBegin -> seat 1 draws 2 and zhengjian_mark clears!
    let b1 = e.debug_hand(1).len();
    advance_to_play(&mut e, 0);
    assert!(!e.debug_has_skill(1, "zhengjian_mark"));
    assert_eq!(e.debug_hand(1).len(), b1 + 2, "zhengjian draws 2 on TurnBegin");

    // Test zhiyi: 1st basic is Tao -> Tag(1) treats using extra Tao (virtual)!
    e.debug_set_hp(0, 2);
    let tao = e.debug_place(0, "tao");
    let _ = play_card(&mut e, 0, tao, vec![]);
    let _ = expect_prompt(&e, "l11_vm_b:zhiyi_mode");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.debug_hp(0), 4, "zhiyi mode 1 heals +1 extra via virtual Tao");

    // 2. twshengxi + tiaojiyanmei + xinfu_qinguo + xinfu_qinguo_recover
    let mut e2 = start_4p_taixu(2026_8205);
    e2.debug_grant(0, "twshengxi");
    e2.debug_grant(0, "xinfu_qinguo");
    e2.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e2, 3, GameAction::NextPhase);
    // Prepare: twshengxi_zhunbei gains tiaojiyanmei from detached_cards!
    let _ = expect_prompt(&e2, "l11_vm_b:twshengxi_zhunbei");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    drain_open_with_ai(&mut e2, 6);
    advance_to_play(&mut e2, 0);
    let tj_cid = e2
        .debug_hand(0)
        .into_iter()
        .find(|&c| e2.catalog().cards[c as usize].name.starts_with("tiaojiyanmei"))
        .expect("twshengxi_zhunbei gained off-deck tiaojiyanmei");

    // Use tiaojiyanmei on seat 1 (3 hand cards) and seat 2 (1 hand card) -> equalizes to 2 each -> gain card!
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(2);
    let h1a = e2.debug_place(1, "sha");
    let _h1b = e2.debug_place(1, "sha");
    let _h1c = e2.debug_place(1, "sha");
    let _h2a = e2.debug_place(2, "shan");
    assert!(!matches!(
        play_card(&mut e2, 0, tj_cid, vec![1, 2]),
        CommandOutcome::Rejected(_)
    ));
    while let Some(ch) = e2.open_choice() {
        if ch.spec.prompt == "l11_vm_b:tiaojiyanmei_discard" {
            break;
        }
        drain_open_with_ai(&mut e2, 1);
    }
    let _ = expect_prompt(&e2, "l11_vm_b:tiaojiyanmei_discard");
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(h1a)]);
    let _ = expect_prompt(&e2, "l11_vm_b:tiaojiyanmei_gain_target");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(0)]);
    assert_eq!(e2.debug_hand(1).len(), 2);
    assert_eq!(e2.debug_hand(2).len(), 2);
    assert!(e2.debug_hand(0).contains(&h1a));

    // xinfu_qinguo + xinfu_qinguo_recover: hp=1, empty equips -> equip 1st -> recovers to 2; swap 1-for-1 when hp=1 skips recover!
    e2.debug_set_hp(0, 1);
    let eq1 = e2.debug_place(0, "renwang");
    let _ = play_card(&mut e2, 0, eq1, vec![]);
    let _ = expect_prompt(&e2, "l11_vm_b:xinfu_qinguo_recover");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e2.debug_hp(0), 2, "xinfu_qinguo_recover heals when equips == hp");
    let _ = expect_prompt(&e2, "l11_vm_b:xinfu_qinguo_sha");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e2.debug_hp(1), 3, "xinfu_qinguo virtual Sha damages seat 1");

    // Set hp=1 with 1 armor equipped, swap Armor -> Bagua (eq_swap1) must NOT offer xinfu_qinguo_recover!
    e2.debug_set_hp(0, 1);
    let eq2 = place_matching(&mut e2, 0, |d| d.kind == CardKind::Armor);
    let _ = play_card(&mut e2, 0, eq2, vec![]);
    assert_ne!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("l11_vm_b:xinfu_qinguo_recover"),
        "1-for-1 slot swap must skip xinfu_qinguo_recover"
    );
    drain_open_with_ai(&mut e2, 6);

    // Phase::End -> twshengxi draws zhinang from deck + 1 card since non-damage play card used & 0 dmg by seat 0!
    advance_to_play(&mut e2, 0);
    for s in 0..4u8 {
        e2.debug_clear_hand(s);
    }
    e2.debug_set_hp(0, 4);
    let wz = place_matching(&mut e2, 0, |d| d.name == "wuzhong");
    let out_wz = play_card(&mut e2, 0, wz, vec![]);
    assert!(!matches!(out_wz, CommandOutcome::Rejected(_)), "wz rejected: {out_wz:?}");
    e2.debug_set_deck_top_by(|d| d.kind == CardKind::Nullify);
    let before_end = e2.debug_hand(0).len();
    assert_eq!(before_end, 2, "wuzhong resolved drawing 2");
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    if e2.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
    }
    let _ = expect_prompt(&e2, "l11_vm_b:twshengxi_zhinang");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Tag(2)]); // Nullify
    assert_eq!(e2.debug_hand(0).len(), before_end + 2, "twshengxi gained zhinang + drew 1");
}

#[test]
fn l11_vm_fix_b_ngsa_v107_save_restore_and_public_ai_selfplay() {
    let verified = taixu_verified();
    let mut e = start_4p_taixu(2026_8206);
    e.debug_grant(0, "rexianzhen");
    e.debug_grant(1, "rebiyue");
    e.debug_grant(2, "reshejian");
    let c0 = place_suit_rank(&mut e, 0, Suit::Spade, 11);
    let _ = place_suit_rank(&mut e, 1, Suit::Heart, 4);

    // Pause mid-rexianzhen target choice, NGSA v107 save/restore -> compare expected ask & resume!
    let _ = activate_skill(&mut e, 0, "rexianzhen");
    let (_, obs, opts) = expect_prompt(&e, "l11_vm_b:rexianzhen_target");
    let restored = roundtrip_archive(&e, verified);
    let (_, obs_r, opts_r) = expect_prompt(&restored, "l11_vm_b:rexianzhen_target");
    assert_eq!(obs, obs_r);
    assert_eq!(opts, opts_r);

    let mut live_a = e;
    let mut live_b = restored;
    let _ = answer(&mut live_a, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut live_b, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut live_a, 0, vec![ChoiceOption::Card(c0)]);
    let _ = answer(&mut live_b, 0, vec![ChoiceOption::Card(c0)]);
    assert_eq!(live_a.state_hash(), live_b.state_hash());

    // Multi-turn public-info AI self-play exercising L11VmFixB + rebiyue cleanly
    let mut ai_eng = start_4p_taixu(2026_8207);
    ai_eng.debug_grant(0, "sbyangwei");
    ai_eng.debug_grant(1, "rebiyue");
    ai_eng.debug_grant(2, "rezhiyi");
    ai_eng.debug_grant(3, "zhengjian");
    for s in 0..4 {
        ai_eng.debug_place(s, "sha");
        ai_eng.debug_place(s, "shan");
    }
    let planner = engine_native::ai::BoundedPlanner::new(16);
    for _ in 0..120 {
        if ai_eng.outcome().is_some() {
            break;
        }
        let actor = ai_eng
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or(ai_eng.state_core().turn.current);
        let act = ai_eng.plan_action(actor, &planner).action;
        let out = submit(&mut ai_eng, actor, act);
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
    }
}

#[test]
fn l11_vm_fix_b_r2_xinfu_qinguo_and_zhiyi_range_and_no_count() {
    // 1. xinfu_qinguo: addCount=false & check_dist=true
    let mut e = start_4p_taixu(2026_8215);
    e.debug_grant(0, "xinfu_qinguo");
    let sha1 = e.debug_place(0, "sha");
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    assert_eq!(e.debug_hp(1), 3);

    // Even AFTER playing Sha (sha_used_play_phase == true), equipping triggers xinfu_qinguo_sha!
    // No weapon equipped -> seat 2 (distance 2) must NOT be in options.
    let eq_armor = place_matching(&mut e, 0, |d| d.kind == CardKind::Armor);
    let _ = play_card(&mut e, 0, eq_armor, vec![]);
    let (_, _, q_opts) = expect_prompt(&e, "l11_vm_b:xinfu_qinguo_sha");
    assert!(q_opts.contains(&ChoiceOption::Player(1)));
    assert!(
        !q_opts.contains(&ChoiceOption::Player(2)),
        "xinfu_qinguo virtual Sha must respect attack range"
    );
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.debug_hp(1), 2);

    // Next turn: xinfu_qinguo first must NOT consume Play-phase Sha quota!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let eq_mount = place_matching(&mut e, 0, |d| {
        d.kind == CardKind::Armor || d.kind == CardKind::HorseMinus
    });
    let _ = play_card(&mut e, 0, eq_mount, vec![]);
    drain_open_with_ai(&mut e, 6);
    if e.open_choice().is_some_and(|c| c.spec.prompt == "l11_vm_b:xinfu_qinguo_sha") {
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    }
    drain_open_with_ai(&mut e, 4);
    let sha_after = e.debug_place(0, "sha");
    let out_sha = play_card(&mut e, 0, sha_after, vec![3]);
    assert!(
        !matches!(out_sha, CommandOutcome::Rejected(_)),
        "xinfu_qinguo virtual Sha must not consume Play-phase Sha count: {out_sha:?}"
    );

    // 2. zhiyi: check_dist=true (without tianyi2!), check_count=false, and does NOT consume Sha quota
    let mut e2a = start_4p_taixu(2026_8216);
    e2a.debug_grant(0, "zhiyi");
    let a_sha1 = e2a.debug_place(0, "sha");
    let out_a1 = play_card(&mut e2a, 0, a_sha1, vec![1]);
    assert!(!matches!(out_a1, CommandOutcome::Rejected(_)), "a_sha1 rejected: {out_a1:?}");
    let _ = expect_prompt(&e2a, "l11_vm_b:zhiyi_mode");
    let _ = answer(&mut e2a, 0, vec![ChoiceOption::Tag(1)]); // virtual Sha after sha_used_play_phase == true!
    let (_, _, z_opts) = expect_prompt(&e2a, "l11_vm_b:zhiyi_target");
    assert!(z_opts.contains(&ChoiceOption::Player(1)));
    assert!(z_opts.contains(&ChoiceOption::Player(3)));
    assert!(
        !z_opts.contains(&ChoiceOption::Player(2)),
        "zhiyi virtual Sha must respect attack range (no nodistance)"
    );
    let _ = answer(&mut e2a, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e2a.debug_hp(1), 2);

    // Next: with tianyi2 (+1 Sha limit -> 2), 1st Sha + zhiyi virtual Sha still allows 2nd entity Sha!
    let mut e2 = start_4p_taixu(2026_8219);
    e2.debug_grant(1, "zhiyi");
    e2.debug_grant(1, "tianyi");
    advance_to_play(&mut e2, 1);
    for s in 0..4 {
        e2.debug_clear_hand(s);
    }
    let _win1 = place_suit_rank(&mut e2, 1, Suit::Heart, 12);
    let _lose0 = place_suit_rank(&mut e2, 0, Suit::Club, 2);
    let _ = activate_skill(&mut e2, 1, "tianyi");
    let _ = expect_prompt(&e2, "l11_vm_b:tianyi_target");
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Player(0)]);
    drain_open_with_ai(&mut e2, 6);
    assert!(e2.debug_has_skill(1, "tianyi2"), "tianyi win grants +1 Sha limit (tianyi2)");

    let z_sha1 = e2.debug_place(1, "sha");
    let z_sha2 = e2.debug_place(1, "sha");
    let out_z1 = play_card(&mut e2, 1, z_sha1, vec![0]);
    assert!(!matches!(out_z1, CommandOutcome::Rejected(_)), "z_sha1 rejected: {out_z1:?}");
    let _ = expect_prompt(&e2, "l11_vm_b:zhiyi_mode");
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Tag(1)]); // virtual Sha!
    let _ = expect_prompt(&e2, "l11_vm_b:zhiyi_target");
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Player(2)]);
    let out_z2 = play_card(&mut e2, 1, z_sha2, vec![2]);
    assert!(
        !matches!(out_z2, CommandOutcome::Rejected(_)),
        "zhiyi virtual Sha must not consume Play-phase Sha quota: {out_z2:?}"
    );
}

#[test]
fn l11_vm_fix_b_r2_equip_uses_and_twshengxi_pile_fallback() {
    // 1. qiangzhi_draw, twshengxi, zhengjian_mark on played EventData::Equipped
    let mut e = start_4p_taixu(2026_8217);
    e.debug_grant(0, "qiangzhi");
    e.debug_grant(0, "twshengxi");
    e.debug_grant(1, "zhengjian_mark");
    advance_to_play(&mut e, 0);
    let _ = expect_prompt(&e, "l11_vm_b:qiangzhi_target");
    e.debug_clear_hand(1);
    let t1_eq = place_matching(&mut e, 1, |d| d.kind == CardKind::Armor);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = expect_prompt(&e, "l11_vm_b:qiangzhi_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);
    let _ = t1_eq;

    // Seat 0 plays an equip card from hand -> triggers qiangzhi_draw & records twshengxi!
    e.debug_clear_hand(0);
    let eq0 = place_matching(&mut e, 0, |d| d.kind == CardKind::Weapon);
    let _ = play_card(&mut e, 0, eq0, vec![]);
    let _ = expect_prompt(&e, "trigger_confirm"); // qiangzhi_draw!
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hand(0).len(), 1, "qiangzhi_draw must fire on played equip");

    // Seat 0 dealt 0 dmg -> End phase triggers twshengxi_zhinang!
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    if e.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    let _ = expect_prompt(&e, "l11_vm_b:twshengxi_zhinang");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(2)]);

    // Seat 1's turn: plays equip -> zhengjian_cnt increments!
    advance_to_play(&mut e, 1);
    let _ = play_card(&mut e, 1, t1_eq, vec![]);
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(1, "l11_vm_b:zhengjian_cnt".to_string())).copied(),
        Some(1),
        "zhengjian_mark must increment on played equip use"
    );

    // 2. twshengxi_zhunbei fallback to draw pile (deck) when 4 detached tiaojiyanmei are exhausted
    let mut e2 = start_4p_taixu(2026_8218);
    e2.debug_grant(0, "twshengxi");
    let planner2 = engine_native::ai::BoundedPlanner::new(8);
    let step_to_zhunbei = |eng: &mut engine_native::Engine| {
        for _ in 0..160 {
            if eng
                .open_choice()
                .is_some_and(|c| c.spec.prompt == "l11_vm_b:twshengxi_zhunbei")
            {
                return;
            }
            if let Some(ch) = eng.open_choice() {
                let act = eng.plan_action(ch.observer, &planner2).action;
                let _ = submit(eng, ch.observer, act);
            } else {
                let cur = eng.state_core().turn.current;
                let _ = submit(eng, cur, GameAction::NextPhase);
            }
        }
    };
    let mut gained_ty = Vec::new();
    for _ in 0..4 {
        advance_to_play(&mut e2, 1);
        for s in 0..4 {
            e2.debug_clear_hand(s);
        }
        step_to_zhunbei(&mut e2);
        let _ = expect_prompt(&e2, "l11_vm_b:twshengxi_zhunbei");
        let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
        gained_ty.extend(e2.debug_hand(0));
    }
    assert_eq!(gained_ty.len(), 4);
    assert!(
        !e2.state_core().detached_cards.iter().any(|&(c, _)| {
            e2.catalog()
                .cards
                .get(c as usize)
                .is_some_and(|d| d.name.starts_with("tiaojiyanmei"))
        }),
        "all 4 off-deck tiaojiyanmei cards must be exhausted from detached_cards"
    );

    // Advance past seat 0's DrawPhase to seat 1 Play, then put a taken tiaojiyanmei into deck!
    advance_to_play(&mut e2, 1);
    for s in 0..4 {
        e2.debug_clear_hand(s);
    }
    let ty_5th = gained_ty[0];
    e2.state_core_mut().player_mut(0).hand.retain(|&c| c != ty_5th);
    e2.state_core_mut().discard.retain(|&c| c != ty_5th);
    e2.state_core_mut().deck.insert(0, ty_5th);
    step_to_zhunbei(&mut e2);
    let _ = expect_prompt(&e2, "l11_vm_b:twshengxi_zhunbei");
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert!(
        e2.debug_hand(0).contains(&ty_5th),
        "twshengxi_zhunbei must gain tiaojiyanmei from draw pile after 4 off-deck cards are taken"
    );
}
