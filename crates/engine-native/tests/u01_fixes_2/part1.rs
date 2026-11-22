use super::common::*;

#[test]
fn u01_fix2_01_dddzhengjun() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2005-2193
    let (mut e, d3, p9) = new_clean_engine(92001, 4);
    let yujin = find_general(&e, "ddd_yujin");
    e.assign_general(0, yujin).unwrap();

    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let dismantle = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle);
    let armor0 = find_card_by(&e, |_, d| d.kind == CardKind::Armor);
    let weapon2 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let horse1 = find_card_by(&e, |_, d| d.kind == CardKind::HorsePlus);
    let extra1 = find_card_by(&e, |id, d| {
        id != sha && id != wuzhong && id != dismantle && d.kind == CardKind::Shan
    });
    let extra2 = find_card_by(&e, |id, d| {
        id != sha && id != wuzhong && id != dismantle && id != extra1 && d.kind == CardKind::Tao
    });

    // Negative guard: outside owner's Phase::Play, damage matching HP must NOT trigger.
    e.state_core_mut().player_mut(0).hp = 3;
    e.state_core_mut().player_mut(1).hp = 4;
    e.debug_set_turn(1, Phase::Play, 1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "dddzhengjun only triggers while owner is in Phase::Play");

    // 1) HP branch: during owner's Play phase, P1 hp drops to P0.hp (3) -> choice Recover vs LoseHp, once per play phase.
    e.state_core_mut().player_mut(0).hp = 2;
    e.state_core_mut().player_mut(1).hp = 3;
    e.state_core_mut().player_mut(2).hp = 3;
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_hp = e.open_choice().expect("dddzhengjun_hp_choice").clone();
    assert_eq!((c_hp.observer, c_hp.spec.prompt), (0, "dddzhengjun_hp_choice"));
    assert_eq!(c_hp.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0)]);

    // Fork A: recover +1 HP
    let mut e_rec = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_rec, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e_rec.state_core().player(1).hp, 3);

    // Live B: lose 1 HP, then second matching HP event in same Phase::Play must NOT re-trigger `dddzhengjun_hp`.
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.state_core().player(1).hp, 1);
    let _ = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "dddzhengjun_hp is once per Play phase");

    // 2) Hand-size branch: affected-only filter (bystander P1 also has equal hand size but was NOT moved!).
    let (mut e_hs, d3, p9) = new_clean_engine(92002, 4);
    e_hs.assign_general(0, yujin).unwrap();
    e_hs.debug_set_turn(0, Phase::Play, 1);
    put_in_hand(&mut e_hs, 0, dismantle);
    put_in_hand(&mut e_hs, 0, extra1);
    // Owner has 1 hand card left after playing Dismantle; bystander P1 already has 1; P2 starts with 2 -> after Dismantle drops to 1!
    put_in_hand(&mut e_hs, 1, extra2);
    put_in_hand(&mut e_hs, 2, wuzhong);
    put_in_hand(&mut e_hs, 2, sha);
    let out = submit(
        &mut e_hs,
        0,
        GameAction::PlayCard {
            card: dismantle,
            targets: vec![2],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let c_hs = e_hs.open_choice().expect("dddzhengjun_hs_choice").clone();
    assert_eq!((c_hs.observer, c_hs.spec.prompt), (0, "dddzhengjun_hs_choice"));
    assert_mid_choice_archive_and_ai(&e_hs, &d3, &p9, vec![ChoiceOption::Tag(0)]);

    // Verify bystander P1 was NOT picked (`dddzhengjun` only acted on event-affected P2, even though P1.hand.len == P0.hand.len!).
    let mut e_draw = Engine::restore_with_extensions(&e_hs.save(), &[&d3, &p9]).unwrap();
    let p1_h = e_draw.state_core().player(1).hand.len();
    let p2_h = e_draw.state_core().player(2).hand.len();
    answer(&mut e_draw, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e_draw.state_core().player(1).hand.len(), p1_h);
    assert_eq!(e_draw.state_core().player(2).hand.len(), p2_h + 1);

    // Choice B: force P2 to discard 1 `he` card
    answer(&mut e_hs, 0, vec![ChoiceOption::Tag(1)]);
    let c_disc = e_hs.open_choice().expect("dddzhengjun_hs_discard").clone();
    assert_eq!((c_disc.observer, c_disc.spec.prompt), (2, "dddzhengjun_hs_discard"));
    assert!(!c_disc.spec.allow_cancel);
    let p2_card = e_hs.state_core().player(2).hand[0];
    answer(&mut e_hs, 2, vec![ChoiceOption::Card(p2_card)]);
    assert!(e_hs.state_core().player(2).hand.is_empty());

    // 3) Equip branch: legal move without slot collision
    let (mut e_es, d3, p9) = new_clean_engine(92003, 4);
    e_es.assign_general(0, yujin).unwrap();
    e_es.debug_set_turn(0, Phase::Play, 1);
    let w2 = find_card_by(&e_es, |id, d| id != weapon2 && d.kind == CardKind::Weapon);
    let d3_horse = find_card_by(&e_es, |id, d| id != horse1 && d.kind == CardKind::HorsePlus);
    // P0 has Armor; P1 has Armor+Weapon+Horse+ ; P2 loses Armor leaving 1 equip (Weapon) matching P0's 1 equip!
    put_in_equip(&mut e_es, 0, armor0);
    put_in_equip(&mut e_es, 1, w2);
    put_in_equip(&mut e_es, 1, d3_horse);
    let d_a = find_card_by(&e_es, |id, d| id != armor0 && d.kind == CardKind::Armor);
    put_in_equip(&mut e_es, 2, d_a);
    put_in_equip(&mut e_es, 2, weapon2);
    put_in_hand(&mut e_es, 0, dismantle);
    let _ = submit(
        &mut e_es,
        0,
        GameAction::PlayCard {
            card: dismantle,
            targets: vec![2],
        },
    );
    // P2 has 0 hand cards -> Dismantle discards P2's armor `d_a` leaving P2 with weapon2 (1 equip == P0's 1 equip)!
    let c_es = e_es.open_choice().expect("dddzhengjun_es_confirm").clone();
    assert_eq!((c_es.observer, c_es.spec.prompt), (0, "dddzhengjun_es_confirm"));
    assert_mid_choice_archive_and_ai(&e_es, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e_es, 0, vec![ChoiceOption::Confirm]);
    let c_es_card = e_es.open_choice().expect("dddzhengjun_es_card").clone();
    assert_eq!(c_es_card.spec.prompt, "dddzhengjun_es_card");
    assert!(c_es_card.spec.options.contains(&ChoiceOption::Card(weapon2)));
    answer(&mut e_es, 0, vec![ChoiceOption::Card(weapon2)]);
    let c_es_dest = e_es.open_choice().expect("dddzhengjun_es_dest").clone();
    assert_eq!(c_es_dest.spec.prompt, "dddzhengjun_es_dest");
    assert!(
        !c_es_dest.spec.options.contains(&ChoiceOption::Player(1)),
        "P1 already occupies Weapon slot so must NOT be valid move destination"
    );
    assert!(c_es_dest.spec.options.contains(&ChoiceOption::Player(0)));
    answer(&mut e_es, 0, vec![ChoiceOption::Player(0)]);
    assert!(e_es.state_core().player(0).equips.contains(&weapon2));
    assert!(!e_es.state_core().player(2).equips.contains(&weapon2));

    // 4) Owner-self (`p == owner`) upstream match (`skill.js:2036-2040`): P0 drawing in Play matches P0 himself!
    let (mut e_self, _, _) = new_clean_engine(92004, 3);
    e_self.assign_general(0, yujin).unwrap();
    e_self.debug_set_turn(0, Phase::Play, 1);
    put_in_hand(&mut e_self, 0, wuzhong);
    let _ = submit(&mut e_self, 0, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    let c_self = e_self.open_choice().expect("dddzhengjun_hs_choice on self draw").clone();
    assert_eq!((c_self.observer, c_self.spec.prompt), (0, "dddzhengjun_hs_choice"));
    answer(&mut e_self, 0, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e_self.state_core().player(0).hand.len(), 3);
}

#[test]
fn u01_fix2_02_dddxianxi() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2194-2295
    let (mut e, d3, p9) = new_clean_engine(92004, 4);
    let handang = find_general(&e, "ddd_handang");
    e.assign_general(0, handang).unwrap();

    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let shan = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    let pay_card = find_card_by(&e, |id, _| id != sha && id != shan);

    // Seat 0 attacks seat 1 (dist 1). Seat 2 is dist 2 (out of range 1!) and seat 3 is dist 1.
    // Both P2 and P3 must be eligible extra targets ignoring distance!
    put_in_hand(&mut e, 0, sha);
    put_in_hand(&mut e, 0, pay_card);
    put_in_hand(&mut e, 2, shan);
    e.debug_set_turn(0, Phase::Play, 1);

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let c_ex = e.open_choice().expect("dddxianxi_extra_target").clone();
    assert_eq!((c_ex.observer, c_ex.spec.prompt), (0, "dddxianxi_extra_target"));
    assert!(c_ex.spec.options.contains(&ChoiceOption::Player(2)));
    assert!(c_ex.spec.options.contains(&ChoiceOption::Player(3)));
    assert!(!c_ex.spec.options.contains(&ChoiceOption::Player(1)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);

    answer(&mut e, 0, vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    // P1 takes Sha damage (4->3); P2 dodges with Shan; P3 takes Sha damage (4->3)!
    let c_p2 = e.open_choice().expect("sha_shan for P2").clone();
    assert_eq!(c_p2.observer, 2);
    answer(&mut e, 2, vec![ChoiceOption::Card(shan)]);

    // Only P2 (added target, NO damage) qualifies -> num = 1! `dddxianxi_discard` opens!
    let c_disc = e.open_choice().expect("dddxianxi_discard").clone();
    assert_eq!((c_disc.observer, c_disc.spec.prompt), (0, "dddxianxi_discard"));
    assert!(c_disc.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(pay_card)]);

    // Fork Choice A: cancel discard -> draw 1 & lose 1 HP
    let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let h0_before = e_cancel.state_core().player(0).hand.len();
    let hp0_before = e_cancel.state_core().player(0).hp;
    answer(&mut e_cancel, 0, Vec::new());
    assert_eq!(e_cancel.state_core().player(0).hand.len(), h0_before + 1);
    assert_eq!(e_cancel.state_core().player(0).hp, hp0_before - 1);

    // Live Choice B: discard `pay_card` -> deals 1 damage to P2!
    let hp2_before = e.state_core().player(2).hp;
    answer(&mut e, 0, vec![ChoiceOption::Card(pay_card)]);
    assert_eq!(e.state_core().player(2).hp, hp2_before - 1);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(e.state_core().player(3).hp, 3);
}

#[test]
fn u01_fix2_03_dddlingyong() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2296-2332
    let (mut e, d3, p9) = new_clean_engine(92005, 3);
    let wuzhi = find_general(&e, "ddd_wuzhi");
    e.assign_general(0, wuzhi).unwrap();
    retain_only_skill(&mut e, 0, Some("dddlingyong"));

    let snatch = find_card_by(&e, |_, d| d.kind == CardKind::Snatch);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let tao = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let p1_card = find_card_by(&e, |id, _| id != snatch && id != wuzhong && id != tao && id != sha);
    let draw1 = find_card_by(&e, |id, _| {
        id != snatch && id != wuzhong && id != tao && id != sha && id != p1_card
    });
    let draw2 = find_card_by(&e, |id, _| {
        id != snatch && id != wuzhong && id != tao && id != sha && id != p1_card && id != draw1
    });

    e.state_core_mut().player_mut(0).hp = 2; // injured so Tao is usable!
    put_in_hand(&mut e, 1, p1_card);

    // Push in reverse pop order: pop 1st = snatch, pop 2nd = wuzhong, draw1/draw2 for wuzhong, pop 3rd = tao, pop 4th = sha!
    put_deck_top(&mut e, sha);
    put_deck_top(&mut e, tao);
    put_deck_top(&mut e, draw2);
    put_deck_top(&mut e, draw1);
    put_deck_top(&mut e, wuzhong);
    put_deck_top(&mut e, snatch);

    // Trigger real phase skip: seat 1 has `skip_draw_phase = true` at Phase::Judge -> NextPhase!
    e.state_core_mut().turn.skip_draw_phase = true;
    e.debug_set_turn(1, Phase::Judge, 1);
    let out = submit(&mut e, 1, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }));

    let c0 = e.open_choice().expect("dddlingyong_confirm").clone();
    assert_eq!((c0.observer, c0.spec.prompt), (0, "dddlingyong_confirm"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    // Judge #1 = snatch -> actually USE Snatch on P1 (NOT merely add to hand!)
    let c_use1 = e.open_choice().expect("dddlingyong_use_target 1").clone();
    assert_eq!((c_use1.observer, c_use1.spec.prompt), (0, "dddlingyong_use_target"));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(0).hand.contains(&p1_card));
    assert!(e.state_core().discard.contains(&snatch));

    // Repeat #1 -> Judge #2 = wuzhong -> Confirm use -> draws 2!
    let c_rep1 = e.open_choice().expect("dddlingyong_repeat 1").clone();
    assert_eq!(c_rep1.spec.prompt, "dddlingyong_repeat");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let c_use2 = e.open_choice().expect("dddlingyong_use_target 2").clone();
    assert_eq!(c_use2.spec.prompt, "dddlingyong_use_target");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(0).hand.contains(&draw1));
    assert!(e.state_core().player(0).hand.contains(&draw2));

    // Repeat #2 -> 3rd judge! (No 2-use limit!) -> Judge #3 = tao -> heals P0 2->3!
    let c_rep2 = e.open_choice().expect("dddlingyong_repeat 2").clone();
    assert_eq!(c_rep2.spec.prompt, "dddlingyong_repeat");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let c_use3 = e.open_choice().expect("dddlingyong_use_target 3").clone();
    assert_eq!(c_use3.spec.prompt, "dddlingyong_use_target");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hp, 3);

    // Repeat #3 -> Judge #4 = Sha -> fails judge, NO use option, NO repeat option -> Done!
    let _c_rep3 = e.open_choice().expect("dddlingyong_repeat 3").clone();
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(
        e.open_choice().is_none(),
        "Sha judge stops dddlingyong immediately without use or repeat"
    );
}

#[test]
fn u01_fix2_04_dddxuxiao() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2333-2406
    let (mut e, _d3, _p9) = new_clean_engine(92006, 3);
    let wuzhi = find_general(&e, "ddd_wuzhi");
    e.assign_general(0, wuzhi).unwrap();
    retain_only_skill(&mut e, 0, Some("dddxuxiao"));

    let blk_sha1 = find_card_by(&e, |_, d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let blk_sha2 = find_card_by(&e, |id, d| {
        id != blk_sha1 && d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let blk_trick = find_card_by(&e, |_, d| {
        d.kind == CardKind::Dismantle && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let red_tao = find_card_by(&e, |_, d| {
        d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });

    // Negative guard: discarding a black trick does NOT trigger `dddxuxiao`.
    e.state_core_mut().player_mut(0).hp = 1;
    put_in_hand(&mut e, 0, blk_trick);
    put_in_hand(&mut e, 0, red_tao);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _cd = e.open_choice().expect("discard_to_limit").clone();
    answer(&mut e, 0, vec![ChoiceOption::Card(blk_trick)]);
    assert!(e.open_choice().is_none(), "black trick discard must not trigger dddxuxiao");

    // Negative guard 2: non-discard loss of a black basic card (e.g. Snatch) must NOT trigger `dddxuxiao`.
    let snatch = find_card_by(&e, |_, d| d.kind == CardKind::Snatch);
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, blk_sha1);
    put_in_hand(&mut e, 1, snatch);
    e.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: snatch, targets: vec![0] });
    drain(&mut e);
    assert!(
        !e.log().iter().any(|ev| matches!(ev, LogEvent::SkillTriggered { skill: "dddxuxiao", .. })),
        "non-discard CardsLost of black basic must not trigger dddxuxiao"
    );

    // Multi-card discard loop: discarding 2 black basic cards allows placing BOTH on targets as Bing!
    let (mut e2, d3, p9) = new_clean_engine(92007, 3);
    e2.assign_general(0, wuzhi).unwrap();
    retain_only_skill(&mut e2, 0, Some("dddxuxiao"));
    e2.state_core_mut().player_mut(0).hp = 1;
    put_in_hand(&mut e2, 0, blk_sha1);
    put_in_hand(&mut e2, 0, blk_sha2);
    put_in_hand(&mut e2, 0, red_tao);
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    let _cd2 = e2.open_choice().expect("discard_to_limit 2").clone();
    answer(&mut e2, 0, vec![ChoiceOption::Card(blk_sha1), ChoiceOption::Card(blk_sha2)]);

    // Card 1 choice
    let cx1 = e2.open_choice().expect("dddxuxiao_card 1").clone();
    assert_eq!((cx1.observer, cx1.spec.prompt), (0, "dddxuxiao_card"));
    assert_eq!(cx1.spec.options.len(), 2);
    assert_mid_choice_archive_and_ai(&e2, &d3, &p9, vec![ChoiceOption::Card(blk_sha1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Card(blk_sha1)]);

    let ct1 = e2.open_choice().expect("dddxuxiao_target 1").clone();
    assert_eq!(ct1.spec.prompt, "dddxuxiao_target");
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    assert!(e2.state_core().player(1).judge.contains(&blk_sha1));
    assert_eq!(e2.state_core().player(1).hand.len(), 2);

    // Loop automatically offers card 2 (`blk_sha2`), and P1 already has Bing so P1 is excluded from target options!
    let cx2 = e2.open_choice().expect("dddxuxiao_card 2").clone();
    assert_eq!(cx2.spec.options, vec![ChoiceOption::Card(blk_sha2)]);
    answer(&mut e2, 0, vec![ChoiceOption::Card(blk_sha2)]);
    let ct2 = e2.open_choice().expect("dddxuxiao_target 2").clone();
    assert!(!ct2.spec.options.contains(&ChoiceOption::Player(1)));
    answer(&mut e2, 0, vec![ChoiceOption::Player(2)]);
    assert!(e2.state_core().player(2).judge.contains(&blk_sha2));
    assert_eq!(e2.state_core().player(2).hand.len(), 2);
}

#[test]
fn u01_fix2_05_dddxuyu() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2407-2422
    let (mut e, d3, p9) = new_clean_engine(92008, 3);
    let xujing = find_general(&e, "ddd_xujing");
    e.assign_general(0, xujing).unwrap();
    retain_only_skill(&mut e, 0, Some("dddxuyu"));

    let le1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let le2 = find_card_by(&e, |id, d| id != le1 && d.kind == CardKind::Le);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    put_in_hand(&mut e, 0, le1);
    put_in_hand(&mut e, 0, le2);
    put_in_hand(&mut e, 0, wuzhong);
    e.debug_set_turn(0, Phase::Play, 1);

    // Card 1 (`Le1`): `dddxuyu_confirm` must fire on `useCardAfter` AFTER `Le1` enters P1's judge!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: le1, targets: vec![1] });
    assert!(
        e.state_core().player(1).judge.contains(&le1),
        "dddxuyu must trigger at useCardAfter (after Le1 finishes resolving)"
    );
    let c1 = e.open_choice().expect("dddxuyu_confirm 1").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (0, "dddxuyu_confirm"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), 3); // 2 left + 1 drawn

    // Fork A: end turn (`phaseAfter` expires `.when` WITHOUT discarding!)
    let mut e_end = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    e_end.state_core_mut().player_mut(0).hp = 4;
    e_end.debug_set_turn(0, Phase::End, 1);
    let _ = submit(&mut e_end, 0, GameAction::NextPhase);
    assert!(e_end.open_choice().is_none(), "phaseAfter clears dddxuyu delayed discard");
    // On P0's next turn, playing `le2` must only ask `dddxuyu_confirm`, NOT stale `dddxuyu_discard`!
    e_end.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e_end, 0, GameAction::PlayCard { card: le2, targets: vec![1] });
    assert_eq!(e_end.open_choice().unwrap().spec.prompt, "dddxuyu_confirm");

    // Live Path B: play Card 2 (`wuzhong`) in same turn -> forces `dddxuyu_discard`, AND still allows triggering `dddxuyu` again!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: Vec::new() });
    // Note Wuzhong has drawn 2 cards -> `useCardAfter` opens delayed discard AND new `dddxuyu_confirm`!
    let first_p = e.open_choice().expect("2nd card after").clone();
    if first_p.spec.prompt == "dddxuyu_confirm" {
        answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        let cd = e.open_choice().expect("dddxuyu_discard").clone();
        assert_eq!((cd.observer, cd.spec.prompt), (0, "dddxuyu_discard"));
        assert!(!cd.spec.allow_cancel);
        let c = e.state_core().player(0).hand[0];
        answer(&mut e, 0, vec![ChoiceOption::Card(c)]);
    } else {
        assert_eq!((first_p.observer, first_p.spec.prompt), (0, "dddxuyu_discard"));
        assert!(!first_p.spec.allow_cancel);
        let c = e.state_core().player(0).hand[0];
        answer(&mut e, 0, vec![ChoiceOption::Card(c)]);
        let c2 = e.open_choice().expect("dddxuyu_confirm 2").clone();
        assert_eq!((c2.observer, c2.spec.prompt), (0, "dddxuyu_confirm"));
        answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    }
}

#[test]
fn u01_fix2_01b_dddzhengjun_branch3_judge_and_source() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2182-2186
    let (mut e, d3, p9) = new_clean_engine(92101, 4);
    let yujin = find_general(&e, "ddd_yujin");
    e.assign_general(0, yujin).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);

    let dism1 = find_card_by(&e, |_, d| d.kind == CardKind::Dismantle);
    let dism2 = find_card_by(&e, |id, d| id != dism1 && d.kind == CardKind::Dismantle);
    let lebu = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let p1_eq = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let p2_renwang = find_card_by(&e, |_, d| d.kind == CardKind::Armor);
    let p2_eq2 = find_card_by(&e, |id, d| id != p1_eq && d.kind == CardKind::Weapon);

    // Counterexample (#148 Item 1): P0 has 0 equips, P1 (es_target) loses only equip leaving 0 equips + judge=[lebu],
    // while bystander P2 holds equip=[p2_renwang]. Ask 2005 MUST offer `lebu` (P1.judge) and MUST NOT offer `p2_renwang`!
    remove_card_from_all_zones(&mut e, lebu);
    e.state_core_mut().player_mut(1).judge.push(lebu);
    put_in_equip(&mut e, 1, p1_eq);
    put_in_equip(&mut e, 2, p2_renwang);
    put_in_hand(&mut e, 0, dism1);
    put_in_hand(&mut e, 0, dism2);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: dism1, targets: vec![1] });
    assert_eq!(e.open_choice().expect("dddzhengjun_es_confirm").spec.prompt, "dddzhengjun_es_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    let cc = e.open_choice().expect("dddzhengjun_es_card on P1 judge").clone();
    assert_eq!(cc.spec.prompt, "dddzhengjun_es_card");
    assert!(cc.spec.options.contains(&ChoiceOption::Card(lebu)), "must include es_target P1's judge card");
    assert!(!cc.spec.options.contains(&ChoiceOption::Card(p2_renwang)), "must NOT include bystander P2's equip");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(lebu)]);

    // Fork A: Cancel Ask 2005 -> `used_b3` IS already consumed on Confirm, so P3 losing its equip down to 0==P0 must NOT re-open `2004`!
    let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_cancel, 0, Vec::new());
    assert!(e_cancel.state_core().player(1).judge.contains(&lebu));
    put_in_equip(&mut e_cancel, 3, p2_eq2);
    let _ = submit(&mut e_cancel, 0, GameAction::PlayCard { card: dism2, targets: vec![3] });
    assert!(e_cancel.open_choice().is_none(), "used_b3 consumed on Confirm even if Ask 2005 was cancelled");

    // Live B: Move `lebu` from P1.judge -> P2.judge (does NOT re-trigger branch 2 hand choice!)
    answer(&mut e, 0, vec![ChoiceOption::Card(lebu)]);
    let cd = e.open_choice().expect("dddzhengjun_es_dest").clone();
    assert_eq!(cd.spec.prompt, "dddzhengjun_es_dest");
    assert!(!cd.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(cd.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(e.state_core().player(1).judge.is_empty());
    assert_eq!(e.state_core().player(2).judge, vec![lebu]);
    assert!(e.open_choice().is_none(), "moving judge card must NOT re-trigger dddzhengjun hs/es");

    // Empty movable Confirm test: Confirm when es_target has 0 movable cards ALSO consumes `used_b3`!
    let (mut e2, _, _) = new_clean_engine(92102, 2);
    e2.assign_general(0, yujin).unwrap();
    e2.debug_set_turn(0, Phase::Play, 1);
    put_in_equip(&mut e2, 1, p1_eq);
    put_in_hand(&mut e2, 0, dism1);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: dism1, targets: vec![1] });
    // P1 drops to 0 equips & 0 judge, P0 has 0 equips & 0 judge -> movable on es_target P1 is empty!
    assert_eq!(e2.open_choice().expect("es_confirm when movable empty").spec.prompt, "dddzhengjun_es_confirm");
    answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert!(e2.open_choice().is_none());
    put_in_equip(&mut e2, 0, p2_renwang);
    put_in_equip(&mut e2, 1, p2_eq2);
    put_in_hand(&mut e2, 0, dism2);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: dism2, targets: vec![1] });
    assert!(e2.open_choice().is_none(), "used_b3 consumed on Confirm even when es_target had 0 movable cards");
}

#[test]
fn u01_fix2_03b_dddlingyong_equip() {
    // upstream: archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2306-2318
    let (mut e, d3, p9) = new_clean_engine(92103, 3);
    let wuzhi = find_general(&e, "ddd_wuzhi");
    e.assign_general(0, wuzhi).unwrap();
    retain_only_skill(&mut e, 0, Some("dddlingyong"));

    let old_weapon = find_card_by(&e, |_, d| d.kind == CardKind::Weapon);
    let new_weapon = find_card_by(&e, |id, d| id != old_weapon && d.kind == CardKind::Weapon);
    let armor = find_card_by(&e, |_, d| d.kind == CardKind::Armor);

    put_in_equip(&mut e, 0, old_weapon);
    put_deck_top(&mut e, new_weapon);
    put_deck_top(&mut e, armor);

    // Phase skip triggers dddlingyong; judging Equip must equip onto owner (NOT vanish/discard).
    e.state_core_mut().turn.skip_draw_phase = true;
    e.debug_set_turn(1, Phase::Judge, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.open_choice().expect("dddlingyong_confirm").spec.prompt, "dddlingyong_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    let cu1 = e.open_choice().expect("dddlingyong_use_target armor").clone();
    assert_eq!((cu1.observer, cu1.spec.prompt), (0, "dddlingyong_use_target"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(0).equips.contains(&armor), "judged Armor must be equipped on P0");
    assert!(!e.state_core().discard.contains(&armor), "judged Armor must NOT sit in discard");
    assert!(!e.state_core().table.contains(&armor), "judged Armor must NOT linger on table");

    // Repeat #1 -> Judge #2 = new_weapon -> replaces old_weapon cleanly into discard!
    let rep1 = e.open_choice().expect("dddlingyong_repeat").clone();
    assert_eq!(rep1.spec.prompt, "dddlingyong_repeat");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let cu2 = e.open_choice().expect("dddlingyong_use_target weapon").clone();
    assert_eq!(cu2.spec.prompt, "dddlingyong_use_target");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(0).equips.contains(&new_weapon), "judged Weapon must equip onto P0");
    assert!(e.state_core().player(0).equips.contains(&armor));
    assert!(!e.state_core().player(0).equips.contains(&old_weapon), "replaced weapon removed from equips");
    assert!(e.state_core().discard.contains(&old_weapon), "replaced weapon moved to discard");
    assert!(!e.state_core().discard.contains(&new_weapon));
    assert!(!e.state_core().table.contains(&new_weapon));

    let rep2 = e.open_choice().expect("dddlingyong_repeat after weapon").clone();
    assert_eq!(rep2.spec.prompt, "dddlingyong_repeat");
    answer(&mut e, 0, Vec::new());
    assert!(e.open_choice().is_none());
}
