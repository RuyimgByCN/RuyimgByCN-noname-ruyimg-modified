use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b4_feizhua_diesha_fenxing_and_xfenxin() {
    // 1. feizhua: 4 seats, 0->1 single Sha -> confirm adds 2 (pure dist 1 from 1)
    let (_, mut ef) = start_xianjian_only(8410, 4);
    advance_to_play(&mut ef, 0);
    for p in 0..4 {
        ef.debug_clear_hand(p);
        ef.debug_set_hp(p, 5);
    }
    ef.debug_grant(0, "feizhua");
    let sha = ef.debug_place(0, "sha");
    let out = submit(&mut ef, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let fc = ef.open_choice().expect("feizhua confirm").clone();
    assert_eq!(fc.spec.prompt, "xj_u05_feizhua_confirm");
    answer_open(&mut ef, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut ef);
    assert_eq!(ef.debug_hp(1), 4, "primary target damaged");
    assert_eq!(ef.debug_hp(2), 4, "pure distance-1 neighbor also damaged");
    assert_eq!(ef.debug_hp(3), 5, "player 3 untouched");

    // 2. diesha: Sha damage -> target LoseHp(1) + owner recover(1); non-Sha ignores
    let (_, mut ed) = start_xianjian_only(8411, 2);
    advance_to_play(&mut ed, 0);
    ed.debug_clear_hand(0);
    ed.debug_clear_hand(1);
    ed.debug_set_max_hp(0, 5);
    ed.debug_set_hp(0, 3);
    ed.debug_set_hp(1, 5);
    ed.debug_grant(0, "diesha");
    let dl = ed.debug_place(0, "duel");
    submit(&mut ed, 0, GameAction::PlayCard { card: dl, targets: vec![1] });
    assert_eq!((ed.debug_hp(1), ed.debug_hp(0)), (4, 3), "duel damage does NOT trigger diesha");
    let sha_d = ed.debug_place(0, "sha");
    submit(&mut ed, 0, GameAction::PlayCard { card: sha_d, targets: vec![1] });
    assert_eq!(ed.debug_hp(1), 2, "target took 1 sha damage + 1 LoseHp = 4->2");
    assert_eq!(ed.debug_hp(0), 4, "owner recovered 1 (3->4)");

    // 3. fenxing: pal_longkui toggles diewu/lingyu <-> guijiang/diesha at TurnBegin (seed finds toggle); non-longkui ignored
    let (_, mut en) = start_xianjian_only(8412, 2);
    advance_to_play(&mut en, 1);
    en.debug_grant(0, "fenxing");
    en.debug_grant(0, "diewu");
    step_full_turn_to(&mut en, 0);
    assert!(en.state_core().skills.contains_key(&(0, en.catalog().skill_by_name("diewu").unwrap())), "non-pal_longkui keeps diewu");

    // Test pal_longkui hero toggle across TurnBegin rounds
    let mut toggled = false;
    for seed in 8413..8425 {
        let (v_l, mut el) = start_xianjian_only(seed, 2);
        let lk_idx = el.catalog().extension_generals.iter().position(|g| g.name == "pal_longkui").unwrap();
        let lk_gid = (el.catalog().standard_generals.len() + lk_idx) as u16;
        advance_to_play(&mut el, 1);
        let mut snap = el.save();
        snap.core.players[0].general_slots = [Some(lk_gid), None];
        let mut el = content_native_loader::restore_locked_engine(&snap, &v_l).unwrap();
        el.debug_grant(0, "fenxing");
        el.debug_grant(0, "diewu");
        el.debug_grant(0, "lingyu");
        step_full_turn_to(&mut el, 0);
        if el.catalog().skill_by_name("guijiang").is_some_and(|d| el.state_core().skills.contains_key(&(0, d))) {
            assert!(!el.state_core().skills.contains_key(&(0, el.catalog().skill_by_name("diewu").unwrap())));
            assert!(el.state_core().skills.contains_key(&(0, el.catalog().skill_by_name("diesha").unwrap())));
            toggled = true;
            break;
        }
    }
    assert!(toggled, "pal_longkui toggled via fenxing");

    // 4. xfenxin + xfenxin2: LoseHp & Recover draw delta; kill boosts max_hp + recover
    let (_, mut ex) = start_xianjian_only(8426, 2);
    advance_to_play(&mut ex, 0);
    ex.debug_clear_hand(0);
    ex.debug_clear_hand(1);
    assemble_general_by_name(&mut ex, 0, "pal_xuanxiao");
    ex.debug_set_max_hp(0, 6);
    ex.debug_set_hp(0, 6);
    ex.debug_set_hp(1, 1);
    deal_damage(&mut ex, Some(1), 0, 2, DamageNature::Normal);
    assert_eq!(ex.debug_hand(0).len(), 2, "took 2 damage -> xfenxin draws 2");
    let tao = ex.debug_place(0, "tao");
    submit(&mut ex, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    assert_eq!(ex.debug_hand(0).len(), 3, "recovered 1 -> xfenxin draws 1 more");
    deal_damage(&mut ex, Some(0), 1, 1, DamageNature::Normal);
    drain_pass_all(&mut ex);
    assert!(!ex.debug_alive(1));
    assert_eq!(ex.state_core().player(0).max_hp, 7, "xfenxin2 +1 max_hp on kill");
    assert_eq!(ex.debug_hp(0), 6, "xfenxin2 +1 recover AND xfenxin +1 draw");
    assert_eq!(ex.debug_hand(0).len(), 4);
}

#[test]
fn u05_b4_qijian_qianfang2_poyun2_longxi_xjzhimeng3() {
    // 1. qijian + save/restore mid ASK_QIJIAN_TARGETS + qianfang2 + poyun2
    let (v, mut e) = start_xianjian_only(8430, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "qijian");
    e.debug_grant(0, "poyun2");
    e.debug_set_hp(0, 1);
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let c = e.open_choice().expect("discard 2 ask").clone();
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: picks });
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End (triggers qijian)
    let qjc = e.open_choice().expect("qijian targets ask").clone();
    assert_eq!(qjc.spec.prompt, "xj_u05_qijian_targets");
    assert_eq!(qjc.spec.max, 2);
    let mut e2 = roundtrip_choice(e, &v);
    answer_open(&mut e2, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    assert_eq!(e2.debug_hp(1), 5, "virtual sha hit target 1");
    assert_eq!(e2.debug_hp(2), 5, "virtual sha hit target 2 ignoring distance");
    assert_eq!(e2.debug_hand(0).len(), 2, "poyun2 drew 1 (once) after damage");
    assert!(!e2.state_core().skills.contains_key(&(0, e2.catalog().skill_by_name("poyun2").unwrap())));

    // qianfang2: DrawPhase +1 (and verify zhuyue2 subskill binding)
    assert!(e2.catalog().skill_by_name("zhuyue2").is_some());
    e2.debug_grant(1, "qianfang2");
    e2.debug_clear_hand(1);
    step_full_turn_to(&mut e2, 1);
    assert_eq!(e2.debug_hand(1).len(), 3, "qianfang2 draws 3 instead of 2");

    // 2. longxi: off-turn Shan ask checks top 2 of deck & gains matching Shan; no trigger on own turn
    let (vl, mut el) = start_xianjian_only(8431, 2);
    advance_to_play(&mut el, 0);
    el.debug_clear_hand(0);
    el.debug_clear_hand(1);
    el.debug_set_hp(1, 4);
    el.debug_grant(1, "longxi");
    let top_shan = (0..el.catalog().cards.len() as u32).find(|&c| el.catalog().card_kind(c) == CardKind::Shan).unwrap();
    let second_tao = (0..el.catalog().cards.len() as u32).find(|&c| el.catalog().card_kind(c) == CardKind::Tao).unwrap();
    place_top_of_deck(&mut el, &vl, &[top_shan, second_tao]);
    let sha0 = el.debug_place(0, "sha");
    submit(&mut el, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(el.debug_hand(1).contains(&top_shan), "longxi pulled Shan from top 2 of deck before ask");
    assert!(!el.debug_hand(1).contains(&second_tao), "non-Shan ignored by longxi");
    drain_pass_all(&mut el);

    // 3. xjzhimeng + xjzhimeng3: dark card returned at owner's next TurnBegin
    let (vz, mut ez) = start_xianjian_only(8432, 2);
    advance_to_play(&mut ez, 0);
    ez.debug_clear_hand(0);
    ez.debug_clear_hand(1);
    ez.debug_grant(0, "xjzhimeng");
    let mark_cid = (0..ez.catalog().cards.len() as u32).find(|&c| ez.catalog().card_kind(c) == CardKind::Wuzhong).unwrap();
    place_top_of_deck(&mut ez, &vz, &[mark_cid]);
    let _ = submit(&mut ez, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut ez, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut ez, 0, GameAction::NextPhase); // End triggers xjzhimeng
    answer_open(&mut ez, vec![ChoiceOption::Player(1)]);
    assert_eq!(ez.state_core().player(1).off_court, vec![mark_cid]);
    assert!(ez.state_core().skills.contains_key(&(0, ez.catalog().skill_by_name("xjzhimeng3").unwrap())));
    step_full_turn_to(&mut ez, 1);
    assert!(ez.state_core().player(1).off_court.contains(&mark_cid), "still off_court during player 1 turn");
    ez.debug_clear_hand(1);
    step_full_turn_to(&mut ez, 0);
    assert!(ez.state_core().player(1).off_court.is_empty(), "xjzhimeng3 cleared off_court at owner next TurnBegin");
    assert!(ez.debug_hand(1).contains(&mark_cid), "player 1 gained dark card");
    assert!(!ez.state_core().skills.contains_key(&(1, ez.catalog().skill_by_name("xjzhimeng2").unwrap())));
}
