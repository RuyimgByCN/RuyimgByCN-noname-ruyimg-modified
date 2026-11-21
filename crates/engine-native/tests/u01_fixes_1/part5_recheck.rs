use super::common::*;

fn find_tengjia(e: &Engine) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .skip(1)
        .rfind(|(_, def)| def.name == "tengjia")
        .map(|(id, _)| id as u32)
        .expect("tengjia in catalog")
}

#[test]
fn recheck_09_dddjungui() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:384-442
    // Verify: self + 0-hand alive targets included at phaseJieshuBegin (`Phase::End`);
    // 0-hand target draws 2 & reveals BEFORE distinct suit count; effective_card_suit (xinhongyan Spade->Heart);
    // turn_flag isolates prior turn; single batch distinct-suit discard rejects duplicate suits.
    let (mut e, d3, p9, aux) = new_recheck_engine(92009, 4);
    let zhangcheng = find_general(&e, "ddd_zhangcheng");
    e.assign_general(0, zhangcheng).unwrap();
    retain_only_skill(&mut e, 0, Some("dddjungui"));
    e.debug_grant(2, "xinhongyan");

    let s1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let h1 = find_card_by(&e, |id, d| id != s1 && d.kind == CardKind::Sha && d.suit == Suit::Heart);
    let d1 = find_card_by(&e, |id, d| id != s1 && id != h1 && d.suit == Suit::Diamond);
    let s2 = find_card_by(&e, |id, d| id != s1 && id != h1 && id != d1 && d.suit == Suit::Spade);
    let s3 = find_card_by(&e, |id, d| id != s1 && id != h1 && id != d1 && id != s2 && d.suit == Suit::Spade);

    // Use a Sha during turn 1 Play phase so prior-turn CardUsed does NOT carry into turn 2 `phaseJieshuBegin`:
    e.debug_set_turn(0, Phase::Play, 1);
    put_in_hand(&mut e, 0, s3);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s3,
            targets: vec![1],
        },
    );
    drain(&mut e);

    // Enter turn 2 End phase (`phaseJieshuBegin`) on seat 0; seat 2 has 0 hand cards, deck top gives Spade + Heart.
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);
    put_deck_top(&mut e, h1);
    put_deck_top(&mut e, s1); // popped first => s1 then h1
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);

    let pick = e.open_choice().expect("dddjungui_target at phaseJieshuBegin").clone();
    assert_eq!((pick.observer, pick.spec.prompt), (0, "dddjungui_target"));
    assert!(pick.spec.options.contains(&ChoiceOption::Player(0)), "owner must be targetable");
    assert!(pick.spec.options.contains(&ChoiceOption::Player(2)), "0-hand alive target must be targetable");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Player(2)]);
    let log_start = e.log().len();
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);

    // Target 2 drew 2 BEFORE suit count; xinhongyan turns s1 (Spade) into Heart => both Heart => only 1 discard needed!
    assert!(
        e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 2, cards } if cards.len() == 2)),
        "target must draw 2 BEFORE dddjungui_discard"
    );
    let disc = e.open_choice().expect("dddjungui_discard").clone();
    assert_eq!((disc.observer, disc.spec.prompt, disc.spec.min, disc.spec.max), (2, "dddjungui_discard", 1, 1));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(s1)]);
    answer(&mut e, 2, vec![ChoiceOption::Card(s1)]);
    drain(&mut e);

    // Round 3 targeting self (Player(0)) with 2 Spades + 1 Diamond in hand => min=max=2 distinct suits;
    // duplicate Spade selection MUST be Rejected, whereas Spade+Diamond succeeds in one batch!
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, s2);
    put_deck_top(&mut e, d1);
    put_deck_top(&mut e, s3);
    e.debug_set_turn(0, Phase::Discard, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    let disc0 = e.open_choice().expect("self dddjungui_discard").clone();
    assert_eq!((disc0.observer, disc0.spec.min, disc0.spec.max), (0, 2, 2));
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: disc0.id,
                selection: vec![ChoiceOption::Card(s2), ChoiceOption::Card(s3)],
            },
        ),
        CommandOutcome::Rejected(_),
    ), "duplicate suit must be rejected in batch dddjungui_discard");
    answer(&mut e, 0, vec![ChoiceOption::Card(s3), ChoiceOption::Card(d1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand, vec![s2]);
}

#[test]
fn recheck_17_dddtongyu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1443-1583
    // Verify: target choices BEFORE direction; owner!=0 CCW order; Wugu reveals alive+extra BEFORE exclude retain;
    // paid cards remain on table during Wugu and move to discard afterward; effective_card_suit in dddtongyu_cards.
    let (mut e, d3, p9, aux) = new_recheck_engine(92017, 4);
    e.debug_grant(2, "dddtongyu");
    e.debug_grant(2, "xinhongyan");
    let tongyu = e.catalog().skill_by_name("dddtongyu").unwrap();

    let spade = find_card_by(&e, |_, d| d.suit == Suit::Spade);
    let heart = find_card_by(&e, |id, d| id != spade && d.suit == Suit::Heart);
    let diamond = find_card_by(&e, |id, d| id != spade && id != heart && d.suit == Suit::Diamond);
    let club = find_card_by(&e, |id, d| id != spade && id != heart && id != diamond && d.suit == Suit::Club);
    put_in_hand(&mut e, 2, spade);
    put_in_hand(&mut e, 2, heart);
    put_in_hand(&mut e, 2, diamond);
    put_in_hand(&mut e, 2, club);

    e.debug_set_turn(2, Phase::Play, 1);
    let _ = submit(&mut e, 2, GameAction::ActivateSkill { skill: tongyu });
    let cards_c = e.open_choice().expect("dddtongyu_cards").clone();
    assert_eq!((cards_c.observer, cards_c.spec.prompt), (2, "dddtongyu_cards"));
    // xinhongyan treats Spade as Heart => [Spade, Heart] has same effective suit => Rejected!
    assert!(matches!(
        submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: cards_c.id,
                selection: vec![ChoiceOption::Card(spade), ChoiceOption::Card(heart)],
            },
        ),
        CommandOutcome::Rejected(_),
    ), "xinhongyan Spade+Heart share Heart suit and must reject");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(spade), ChoiceOption::Card(diamond)]);
    answer(&mut e, 2, vec![ChoiceOption::Card(spade), ChoiceOption::Card(diamond)]);

    // Target choices (1036) MUST happen BEFORE direction choice (1035), in seat_order_from(2) = [2, 3, 0, 1]!
    // Paid cards MUST stay on table, NOT yet in discard!
    assert!(e.state_core().table.contains(&spade) && e.state_core().table.contains(&diamond));
    assert!(!e.state_core().discard.contains(&spade));
    for (seat, tag) in [(2u8, 0u8), (3, 0), (0, 0), (1, 1)] {
        let tc = e.open_choice().expect("dddtongyu_target_choice before direction").clone();
        assert_eq!((tc.observer, tc.spec.prompt), (seat, "dddtongyu_target_choice"));
        answer(&mut e, seat, vec![ChoiceOption::Tag(tag)]);
    }

    let dir = e.open_choice().expect("dddtongyu_direction AFTER target choices").clone();
    assert_eq!((dir.observer, dir.spec.prompt), (2, "dddtongyu_direction"));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 2, vec![ChoiceOption::Tag(1)]); // CCW from owner=2 with 1 excluded => [2, 0, 3]!

    // Wugu must reveal 4 (alive) + 2 (extra) = 6 cards BEFORE excluding seat 1, and iterate [2, 0, 3]:
    let mut pick_order = Vec::new();
    for expected_seat in [2u8, 0, 3] {
        let wp = e.open_choice().expect("wugu_pick").clone();
        assert_eq!((wp.observer, wp.spec.prompt), (expected_seat, "wugu_pick"));
        if expected_seat == 2 {
            assert_eq!(wp.spec.options.len(), 6, "Wugu must reveal alive(4) + extra(2) = 6 cards");
            assert!(e.state_core().table.contains(&spade) && e.state_core().table.contains(&diamond));
        }
        pick_order.push(wp.observer);
        let c0 = wp.spec.options[0];
        answer(&mut e, wp.observer, vec![c0]);
    }
    assert_eq!(pick_order, vec![2, 0, 3]);
    drain(&mut e);
    assert!(!e.state_core().table.contains(&spade) && e.state_core().discard.contains(&spade));
    assert!(!e.state_core().table.contains(&diamond) && e.state_core().discard.contains(&diamond));
}

#[test]
fn recheck_05_dddlangzhi() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5296-5426
    // A) 0 `he` cards on owner after reveals => auto-gains all revealed cards and permanently loses dddlangzhi.
    {
        let (mut e, _d3, _p9, _aux) = new_recheck_engine(92005, 3);
        let kebineng = find_general(&e, "ddd_kebineng");
        e.assign_general(0, kebineng).unwrap();
        let langzhi_def = e.catalog().skill_by_name("dddlangzhi").unwrap();

        let spade = find_card_by(&e, |_, d| d.suit == Suit::Spade);
        let heart = find_card_by(&e, |id, d| id != spade && d.suit == Suit::Heart);
        put_in_hand(&mut e, 1, spade);
        put_in_hand(&mut e, 2, heart);
        e.debug_set_turn(0, Phase::Discard, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let cf = e.open_choice().expect("dddlangzhi_confirm").clone();
        assert_eq!((cf.observer, cf.spec.prompt), (0, "dddlangzhi_confirm"));
        answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        // Reveal both in-range targets (seat 1 and seat 2 in 3-seat ring):
        for s in 1..=2u8 {
            let rv = e.open_choice().expect(&format!("dddlangzhi_reveal_target_card seat {s}")).clone();
            assert_eq!((rv.observer, rv.spec.prompt), (0, "dddlangzhi_reveal_target_card"));
            answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);
        }
        // Seat 0 has 0 he cards => auto-gains both cards & loses dddlangzhi!
        assert!(e.open_choice().is_none(), "0-he owner auto-gains without exchange choice");
        assert!(e.state_core().player(0).hand.contains(&spade));
        assert!(e.state_core().player(0).hand.contains(&heart));
        assert!(!e.state_core().has_skill(0, langzhi_def), "must permanently lose dddlangzhi");
    }

    // B) Swap emits simultaneous private CardsGiven (NOT CardsGivenPublic)
    {
        let (mut e, d3, p9, aux) = new_recheck_engine(92105, 3);
        let kebineng = find_general(&e, "ddd_kebineng");
        e.assign_general(0, kebineng).unwrap();
        let c0 = find_card_by(&e, |_, d| d.suit == Suit::Club);
        let c1 = find_card_by(&e, |id, d| id != c0 && d.suit == Suit::Diamond);
        put_in_hand(&mut e, 0, c0);
        put_in_hand(&mut e, 1, c1);
        e.debug_set_turn(0, Phase::Discard, 1);
        let log_start = e.log().len();
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        let rv = e.open_choice().expect("dddlangzhi_reveal_target_card").clone();
        assert_eq!(rv.spec.prompt, "dddlangzhi_reveal_target_card");
        answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);
        let own = e.open_choice().expect("dddlangzhi_own_card").clone();
        assert_eq!(own.spec.prompt, "dddlangzhi_own_card");
        assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(c0)]);
        answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
        let ex = e.open_choice().expect("dddlangzhi_swap_cards").clone();
        assert_eq!(ex.spec.prompt, "dddlangzhi_swap_cards");
        answer(&mut e, 0, vec![ChoiceOption::Card(c1)]);
        drain(&mut e);
        assert!(e.state_core().player(0).hand.contains(&c1));
        assert!(e.state_core().player(1).hand.contains(&c0));
        assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGiven { from: 0, to: 1, .. })));
        assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGiven { from: 1, to: 0, .. })));
        assert!(!e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGivenPublic { .. })));
    }
}

#[test]
fn recheck_08_dddxiaxing() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59-243
    let (mut base, d3, p9, aux) = new_recheck_engine(92008, 3);
    let zhangmiao = find_general(&base, "ddd_zhangmiao");
    base.assign_general(1, zhangmiao).unwrap();
    let xx_def = base.catalog().skill_by_name("dddxiaxing").unwrap();

    // Prepare arms dddxiaxing_xia = 1, yielding after CardsLost (verify via xingchong drawing 2 BEFORE CardUsed(Jiu)!):
    let eq_prep = find_tengjia(&base);
    base.debug_grant(1, "xingchong");
    base.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "xingchong_round_used".to_string()), 1);
    base.state_core_mut()
        .vm_aux
        .flags
        .insert((1, format!("xingchong_card:{eq_prep}")), 1);
    put_in_equip(&mut base, 1, eq_prep);
    let c_draw1 = find_card_by(&base, |id, _| id != eq_prep);
    let c_draw2 = find_card_by(&base, |id, _| id != eq_prep && id != c_draw1);
    put_deck_top(&mut base, c_draw2);
    put_deck_top(&mut base, c_draw1);
    base.debug_set_turn(0, Phase::End, 1);
    let _ = submit(&mut base, 0, GameAction::NextPhase);
    let xj_first = base.open_choice().expect("dddxiaxing_card").clone();
    assert_eq!(xj_first.spec.prompt, "dddxiaxing_card");
    answer(&mut base, 1, vec![ChoiceOption::Card(eq_prep)]);
    drain(&mut base);
    assert_eq!(flag(&base, 1, "dddxiaxing_xia"), 1);
    assert_eq!(base.state_core().player(1).hand.len(), 2, "xingchong must draw 2 on CardsLost yield BEFORE Jiu");

    // 1) NOT offered in chooseToRespond (Duel / SavageAttack):
    {
        let mut e = Engine::restore_with_extensions(&base.save(), &[&d3, &p9, &aux]).unwrap();
        let duel = find_card_by(&e, |_, d| d.kind == CardKind::Duel);
        let aoe = find_card_by(&e, |_, d| d.kind == CardKind::SavageAttack);
        put_in_hand(&mut e, 0, duel);
        put_in_hand(&mut e, 0, aoe);
        e.debug_set_turn(0, Phase::Play, 2);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
        let ch = e.open_choice().expect("duel_sha").clone();
        assert_eq!((ch.observer, ch.spec.prompt), (1, "duel_sha"));
        assert!(!ch.spec.options.contains(&ChoiceOption::Skill(xx_def)), "dddxiaxing must NOT appear in Duel chooseToRespond");
        drain(&mut e);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: aoe, targets: Vec::new() });
        let ch2 = e.open_choice().expect("aoe_response").clone();
        assert_eq!((ch2.observer, ch2.spec.prompt), (1, "aoe_response"));
        assert!(!ch2.spec.options.contains(&ChoiceOption::Skill(xx_def)), "dddxiaxing must NOT appear in SavageAttack chooseToRespond");
    }

    // 2) Exempt from dddtongyu_hand_locked + Sha dodge emits VirtualCardPlayed(Shan) + triggers releiji_on_shan + Qinglong!
    {
        let mut e = Engine::restore_with_extensions(&base.save(), &[&d3, &p9, &aux]).unwrap();
        let qinglong = find_card_by(&e, |_, d| d.kind == CardKind::Qinglong);
        let sha1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
        let sha2 = find_card_by(&e, |id, d| id != sha1 && d.kind == CardKind::Sha);
        put_in_equip(&mut e, 0, qinglong);
        e.debug_grant(1, "雷击");
        let w2 = find_card_by(&e, |id, d| id != sha1 && id != sha2 && d.kind != CardKind::Shan);
        put_in_hand(&mut e, 1, w2);
        put_in_hand(&mut e, 0, sha1);
        put_in_hand(&mut e, 0, sha2);
        let stamp = 2;
        e.state_core_mut().vm_aux.flags.insert((1, "dddtongyu_hand_locked_stamp".to_string()), stamp);
        e.debug_set_turn(0, Phase::Play, stamp);
        let club = find_card_by(&e, |id, d| id != sha1 && id != sha2 && id != w2 && d.suit == Suit::Club);
        let club2 = find_card_by(&e, |id, d| id != sha1 && id != sha2 && id != w2 && id != club && d.suit == Suit::Club);
        put_deck_top(&mut e, club);
        put_deck_bottom(&mut e, club2);
        let hp2_before = e.debug_hp(2);
        let log_start = e.log().len();
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
        let resp = e.open_choice().expect("respond_shan under hand_lock").clone();
        assert_eq!((resp.observer, resp.spec.prompt), (1, "respond_shan"));
        assert!(resp.spec.options.contains(&ChoiceOption::Skill(xx_def)), "dddxiaxing Shan exempt from hand lock");
        answer(&mut e, 1, vec![ChoiceOption::Skill(xx_def)]);
        assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Shan, player: 1, .. })));
        let ql = e.open_choice().expect("qinglong_replay_sha").clone();
        assert_eq!((ql.observer, ql.spec.prompt), (0, "qinglong_replay_sha"));
        answer(&mut e, 0, Vec::new());
        drain(&mut e);
        assert_eq!(e.debug_hp(2), hp2_before - 1, "BOSS_RELEIJI (Club) must trigger after dddxiaxing Shan");
    }

    // 3) Play Sha respects Zhuge/Paoxiao (physical Sha blocks dddxiaxing without Zhuge; Zhuge unlocks it!):
    {
        let mut e = Engine::restore_with_extensions(&base.save(), &[&d3, &p9, &aux]).unwrap();
        let zhuge = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
        let sha_hand = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
        put_in_hand(&mut e, 1, sha_hand);
        e.debug_set_turn(1, Phase::Play, 2);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_hand, targets: vec![0] });
        drain(&mut e);
        assert!(matches!(
            submit(&mut e, 1, GameAction::ActivateSkill { skill: xx_def }),
            CommandOutcome::Rejected(_),
        ), "dddxiaxing Play Sha must reject after physical Sha without Zhuge/Paoxiao");
        put_in_equip(&mut e, 1, zhuge);
        let out = submit(&mut e, 1, GameAction::ActivateSkill { skill: xx_def });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "Zhuge must unlock dddxiaxing Play Sha: {out:?}");
        let pt = e.open_choice().expect("zhangba_sha_target").clone();
        assert_eq!(pt.spec.prompt, "zhangba_sha_target");
        answer(&mut e, 1, vec![ChoiceOption::Player(0)]);
        drain(&mut e);
        assert_eq!(flag(&e, 1, "dddxiaxing_xia"), 0);
    }

    // 4) Dying seat-order loop + cancel fallback + save/restore mid dying_save and mid dddxiaxing_card:
    {
        let mut e = Engine::restore_with_extensions(&base.save(), &[&d3, &p9, &aux]).unwrap();
        let tao2 = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
        put_in_hand(&mut e, 2, tao2);
        e.debug_set_turn(0, Phase::Play, 2);
        let lethal = e.debug_hp(1) as u32;
        let _ = e.debug_deal_damage(Some(0), 1, lethal, DamageNature::Normal);
        let ds1 = e.open_choice().expect("dying_save at seat 1").clone();
        assert_eq!((ds1.observer, ds1.spec.prompt), (1, "dying_save"));
        assert!(ds1.spec.options.contains(&ChoiceOption::Skill(xx_def)));
        assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Skill(xx_def)]);
        answer(&mut e, 1, vec![ChoiceOption::Skill(xx_def)]);
        let dc = e.open_choice().expect("dddxiaxing_card for Tao").clone();
        assert_eq!((dc.observer, dc.spec.prompt), (1, "dddxiaxing_card"));
        assert_mid_choice_packs(&e, &[&d3, &p9, &aux], Vec::new());
        answer(&mut e, 1, Vec::new());
        let ds1_back = e.open_choice().expect("back to seat 1 dying_save after cancel").clone();
        assert_eq!((ds1_back.observer, ds1_back.spec.prompt), (1, "dying_save"));
        answer(&mut e, 1, vec![ChoiceOption::Pass]);
        let ds2 = e.open_choice().expect("dying_save moves to seat 2").clone();
        assert_eq!((ds2.observer, ds2.spec.prompt), (2, "dying_save"));
        answer(&mut e, 2, vec![ChoiceOption::Card(tao2)]);
        drain(&mut e);
        assert_eq!(e.debug_hp(1), 1);

        let log_start = e.log().len();
        let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        answer(&mut e, 1, vec![ChoiceOption::Skill(xx_def)]);
        let dc2 = e.open_choice().expect("dddxiaxing_card commit Tao").clone();
        let first = dc2.spec.options[0];
        answer(&mut e, 1, vec![first]);
        drain(&mut e);
        assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Tao, player: 1, .. })));
        assert_eq!(e.debug_hp(1), 1);
    }
}

#[test]
fn recheck_11_dddmoyan() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:472-509
    let (mut e, d3, p9, aux) = new_recheck_engine(92011, 3);
    let liuchong = find_general(&e, "ddd_liuchong");
    e.assign_general(0, liuchong).unwrap();

    // 1) Declining trigger_confirm leaves top 3 deck cards untouched:
    let deck_snap = e.save().core.deck.clone();
    e.debug_set_turn(2, Phase::End, 1);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let cf = e.open_choice().expect("trigger_confirm").clone();
    assert_eq!((cf.observer, cf.spec.prompt), (0, "trigger_confirm"));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], Vec::new());
    answer(&mut e, 0, Vec::new());
    assert!(e.open_choice().is_none());
    assert_eq!(e.save().core.deck, deck_snap);

    // 2) Confirming RoundStart sets MOYAN_TARGET (hand <= 1) -> DamageBefore stays forced +1 WITHOUT trigger_confirm popup!
    e.debug_set_turn(2, Phase::End, 2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.open_choice().expect("d3_moyan_target").spec.prompt, "d3_moyan_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    let pick_c = e.open_choice().expect("d3_moyan_gain").clone();
    let c0 = pick_c.spec.options[0];
    answer(&mut e, 0, vec![c0]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1);
    let hp0 = e.debug_hp(0);
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "DamageBefore must NOT ask trigger_confirm");
    assert_eq!(e.debug_hp(0), hp0 - 2);
}

#[test]
fn recheck_12_dddshilie() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:511-655
    let (mut e, _d3, _p9, _aux) = new_recheck_engine(92012, 3);
    let luoxian = find_general(&e, "ddd_luoxian");
    e.assign_general(0, luoxian).unwrap();
    let sl_def = e.catalog().skill_by_name("dddshilie").unwrap();

    let r5 = find_card_by(&e, |_, d| d.rank == 5);
    let r3 = find_card_by(&e, |id, d| id != r5 && d.rank == 3);
    let r7 = find_card_by(&e, |id, d| id != r5 && id != r3 && d.rank == 7);
    let r2 = find_card_by(&e, |id, d| id != r5 && id != r3 && id != r7 && d.rank == 2);
    let zhuge = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    let qinglong = find_card_by(&e, |_, d| d.kind == CardKind::Qinglong);
    put_in_hand(&mut e, 0, r5);
    put_in_hand(&mut e, 0, r3);
    put_in_hand(&mut e, 0, r7);
    put_in_hand(&mut e, 0, r2);

    // 1) Cancelling dddshilie_target does NOT consume turn flag, mark shown cards, or draw:
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sl_def });
    answer(&mut e, 0, vec![ChoiceOption::Card(r5), ChoiceOption::Card(r3)]);
    let tc = e.open_choice().expect("dddshilie_target").clone();
    assert_eq!(tc.spec.prompt, "dddshilie_target");
    answer(&mut e, 0, Vec::new());
    assert_eq!(e.state_core().player(0).hand.len(), 4, "cancel must not draw");
    assert_eq!(flag(&e, 0, &format!("dddshilie_shown_card:{r5}")), 0);

    // Re-activate immediately & commit 1st Play Sha:
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sl_def });
    answer(&mut e, 0, vec![ChoiceOption::Card(r5), ChoiceOption::Card(r3)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, &format!("dddshilie_shown_card:{r5}")), 1);

    // 2) Card leaving hand via Dismantle (`t.hand.first() == r5`) clears dddshilie_shown_card:{c}:
    let dism = find_card_by(&e, |id, d| {
        id != r5 && id != r3 && id != r7 && id != r2 && d.kind == CardKind::Dismantle
    });
    assert_eq!(e.state_core().player(0).hand.first().copied(), Some(r5));
    put_in_hand(&mut e, 2, dism);
    e.debug_set_turn(2, Phase::Play, 1);
    let out_dism = submit(&mut e, 2, GameAction::PlayCard { card: dism, targets: vec![0] });
    assert!(!matches!(out_dism, CommandOutcome::Rejected(_)), "PlayCard Dismantle must succeed: {out_dism:?}");
    drain(&mut e);
    assert!(!e.state_core().player(0).hand.contains(&r5));
    assert_eq!(flag(&e, 0, &format!("dddshilie_shown_card:{r5}")), 0, "leaving hand clears shown marker");

    // 3) Play Sha respects Zhuge/Paoxiao (physical Sha blocks dddshilie without Zhuge; Zhuge unlocks dddshilie,
    //    while cleared `r5` is selectable again & shown `r3` stays excluded; dddshilie_used stays once per turn!):
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_phys = find_card_by(&e, |id, d| {
        id != r5 && id != r3 && id != r7 && id != r2 && id != dism && id != zhuge && d.kind == CardKind::Sha
    });
    put_in_hand(&mut e, 0, r5);
    put_in_hand(&mut e, 0, sha_phys);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_phys, targets: vec![1] });
    drain(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: sl_def }),
        CommandOutcome::Rejected(_),
    ), "dddshilie Play Sha must reject after physical Sha without Zhuge/Paoxiao");
    put_in_equip(&mut e, 0, zhuge);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sl_def });
    let sc2 = e.open_choice().expect("Zhuge unlocks dddshilie_cards").clone();
    assert!(sc2.spec.options.contains(&ChoiceOption::Card(r5)), "cleared r5 must be selectable again");
    assert!(!sc2.spec.options.contains(&ChoiceOption::Card(r3)), "r3 still shown must remain excluded");
    answer(&mut e, 0, vec![ChoiceOption::Card(r5), ChoiceOption::Card(r7)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: sl_def }),
        CommandOutcome::Rejected(_),
    ), "2nd dddshilie in same turn rejected by dddshilie_used (skill.js:524)");

    // 4) Exempt from hand lock + defense Sha triggers VirtualCardPlayed(Shan) + releiji_on_shan + Qinglong:
    let (mut def_e, _d3, _p9, _aux) = new_recheck_engine(92112, 3);
    def_e.assign_general(0, luoxian).unwrap();
    let sha = find_card_by(&def_e, |id, d| id != r7 && id != r2 && d.kind == CardKind::Sha);
    let sha_follow = find_card_by(&def_e, |id, d| id != r7 && id != r2 && id != sha && d.kind == CardKind::Sha);
    let club = find_card_by(&def_e, |id, d| {
        id != r7 && id != r2 && id != sha && id != sha_follow && d.suit == Suit::Club
    });
    let club2 = find_card_by(&def_e, |id, d| {
        id != r7 && id != r2 && id != sha && id != sha_follow && id != club && d.suit == Suit::Club
    });
    put_in_equip(&mut def_e, 1, qinglong);
    put_in_hand(&mut def_e, 1, sha);
    put_in_hand(&mut def_e, 1, sha_follow);
    def_e.debug_grant(0, "雷击");
    put_deck_top(&mut def_e, club);
    put_deck_bottom(&mut def_e, club2);
    let stamp = 3;
    def_e.state_core_mut().vm_aux.flags.insert((0, "dddtongyu_hand_locked_stamp".to_string()), stamp);
    def_e.debug_set_turn(1, Phase::Play, stamp);
    put_in_hand(&mut def_e, 0, r7);
    put_in_hand(&mut def_e, 0, r2);
    let hp1 = def_e.debug_hp(1);
    let log_start = def_e.log().len();
    let _ = submit(&mut def_e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    let resp = def_e.open_choice().expect("respond_shan").clone();
    assert!(resp.spec.options.contains(&ChoiceOption::Skill(sl_def)));
    answer(&mut def_e, 0, vec![ChoiceOption::Skill(sl_def)]);
    let _pick2 = def_e.open_choice().expect("dddshilie_cards defense").clone();
    answer(&mut def_e, 0, vec![ChoiceOption::Card(r7), ChoiceOption::Card(r2)]);
    assert!(def_e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Shan, player: 0, .. })));
    assert_eq!(def_e.open_choice().expect("qinglong_replay_sha").spec.prompt, "qinglong_replay_sha");
    answer(&mut def_e, 1, Vec::new());
    drain(&mut def_e);
    assert_eq!(def_e.debug_hp(1), hp1 - 1, "BOSS_RELEIJI (Club) hits next_alive(0) = seat 1");
}

#[test]
fn recheck_16_dddbingjian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1204-1442
    let (mut e, d3, p9, aux) = new_recheck_engine(92016, 4);
    let wk = find_general(&e, "ddd_wangkanglvkai");
    e.assign_general(0, wk).unwrap();
    retain_only_skill(&mut e, 0, Some("dddbingjian"));
    let bj_def = e.catalog().skill_by_name("dddbingjian").unwrap();
    e.debug_grant(3, "dddzhilian");

    // 1) Cancelling dddbingjian_sha_target (1033) aborts BEFORE drawing/discarding or marking partner:
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: bj_def });
    let tgt = e.open_choice().expect("dddbingjian_sha_target first").clone();
    assert_eq!(tgt.spec.prompt, "dddbingjian_sha_target");
    answer(&mut e, 0, Vec::new());
    assert!(e.open_choice().is_none(), "cancel 1033 must abort immediately");
    assert!(e.state_core().player(0).hand.is_empty());

    // 2) Play Sha commits draw + partner draw, and 2nd Play Sha is blocked without Zhuge:
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: bj_def });
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    let p_c = e.open_choice().expect("dddbingjian_partner").clone();
    assert_eq!(e.state_core().player(0).hand.len(), 2);
    assert_eq!(p_c.spec.prompt, "dddbingjian_partner");
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().player(2).hand.len(), 2);
    drain(&mut e);
    clear_hand(&mut e, 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: bj_def }),
        CommandOutcome::Rejected(_),
    ), "2nd dddbingjian Play Sha blocked without Zhuge/Paoxiao");

    // 3) Exempt from hand lock (< 2 cards ok) + batch discard (>2) + Sha defense emits VirtualCardPlayed(Shan) + releiji + Qinglong:
    let qinglong = find_card_by(&e, |_, d| d.kind == CardKind::Qinglong);
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let sha_follow = find_card_by(&e, |id, d| id != sha && d.kind == CardKind::Sha);
    let club = find_card_by(&e, |id, d| id != sha && id != sha_follow && d.suit == Suit::Club);
    put_in_equip(&mut e, 3, qinglong);
    put_in_hand(&mut e, 3, sha);
    put_in_hand(&mut e, 3, sha_follow);
    e.debug_grant(0, "雷击");
    put_deck_top(&mut e, club);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);
    let stamp = 2;
    e.state_core_mut().vm_aux.flags.insert((0, "dddtongyu_hand_locked_stamp".to_string()), stamp);
    e.debug_set_turn(3, Phase::Play, stamp);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let log_start = e.log().len();
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha, targets: vec![0] });
    let resp = e.open_choice().expect("respond_shan with dddbingjian").clone();
    assert!(resp.spec.options.contains(&ChoiceOption::Skill(bj_def)));
    answer(&mut e, 0, vec![ChoiceOption::Skill(bj_def)]);
    let _part = e.open_choice().expect("dddbingjian_partner defense").clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Shan, player: 0, .. })));
    assert_eq!(e.open_choice().expect("qinglong_replay_sha").spec.prompt, "qinglong_replay_sha");
    drain(&mut e);

    // 4) AOE virtual respond via dddbingjian (>2 batch discard!) marks u01_turn_actor_mask cleanly:
    let aoe = find_card_by(&e, |_, d| d.kind == CardKind::ArcheryAttack);
    let c1 = find_card_by(&e, |id, _| id != aoe);
    let c2 = find_card_by(&e, |id, _| id != aoe && id != c1);
    let c3 = find_card_by(&e, |id, _| id != aoe && id != c1 && id != c2);
    let c4 = find_card_by(&e, |id, _| id != aoe && id != c1 && id != c2 && id != c3);
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, c1);
    put_in_hand(&mut e, 0, c2);
    put_in_hand(&mut e, 0, c3);
    put_in_hand(&mut e, 0, c4);
    put_in_hand(&mut e, 3, aoe);
    e.debug_set_turn(3, Phase::Play, 3);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: aoe, targets: Vec::new() });
    let resp0 = e.open_choice().expect("aoe_response with dddbingjian").clone();
    assert!(resp0.spec.options.contains(&ChoiceOption::Skill(bj_def)));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Skill(bj_def)]);
    answer(&mut e, 0, vec![ChoiceOption::Skill(bj_def)]);
    let disc0 = e.open_choice().expect("dddbingjian batch discard 2").clone();
    assert_eq!((disc0.spec.min, disc0.spec.max), (2, 2));
    answer(&mut e, 0, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    answer(&mut e, 0, Vec::new()); // optional partner cancel
    drain(&mut e);
    assert_ne!(e.state_core().vm_aux.flags.get(&(0, "turn_use_resp_mask".to_string())).copied().unwrap_or(0), 0);
}

#[test]
fn recheck_02_dddyeshen() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:656-746
    // card_zh_name_len exact mapping (LeiSha=1, tengjia=2, Zhangba=4, Qinglong=5) & 3-mark Fire damage trigger!
    let (mut e, d3, p9, aux) = new_recheck_engine(92002, 5);
    let lie = find_general(&e, "ddd_lie");
    e.assign_general(0, lie).unwrap();

    let leisha = find_card_by(&e, |_, d| d.kind == CardKind::LeiSha && matches!(d.suit, Suit::Spade | Suit::Club));
    let tengjia = find_tengjia(&e);
    let zhangba = find_card_by(&e, |_, d| d.kind == CardKind::Zhangba && matches!(d.suit, Suit::Spade | Suit::Club));
    let qinglong = find_card_by(&e, |_, d| d.kind == CardKind::Qinglong && matches!(d.suit, Suit::Spade | Suit::Club));
    let r1 = find_card_by(&e, |_, d| d.suit == Suit::Heart);
    let r2 = find_card_by(&e, |id, d| id != r1 && d.suit == Suit::Heart);
    let r3 = find_card_by(&e, |id, d| id != r1 && id != r2 && d.suit == Suit::Diamond);
    let r4 = find_card_by(&e, |id, d| id != r1 && id != r2 && id != r3 && d.suit == Suit::Diamond);

    // 1st time (mark=0, draws 3 bottom): LeiSha (len 1) => target max == 1:
    put_deck_bottom(&mut e, r2);
    put_deck_bottom(&mut e, r1);
    put_deck_bottom(&mut e, leisha);
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let tg1 = e.open_choice().expect("dddyeshen chain targets leisha").clone();
    assert_eq!(tg1.spec.max, 1, "LeiSha zh name len must be 1");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);

    // 2nd time (mark=1, draws 2 bottom): tengjia (len 2) => target max == 2:
    put_deck_bottom(&mut e, r3);
    put_deck_bottom(&mut e, tengjia);
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let tg2 = e.open_choice().expect("dddyeshen chain targets tengjia").clone();
    assert_eq!(tg2.spec.max, 2, "tengjia zh name len must be 2");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);

    // 3rd time (mark=2, draws 1 bottom): qinglong (len 5) => target max == 5, resets mark=0 & deals 1 Fire damage to owner!
    put_deck_bottom(&mut e, qinglong);
    let hp0 = e.debug_hp(0);
    e.debug_set_turn(0, Phase::Discard, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let tg3 = e.open_choice().expect("dddyeshen chain targets qinglong").clone();
    assert_eq!(tg3.spec.max, 5, "qinglong zh name len must be 5");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Player(2)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "3rd dddyeshen mark must deal 1 Fire damage");
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 0);

    // 4th time: Zhangba (len 4) => target max == 4:
    clear_hand(&mut e, 1);
    put_deck_bottom(&mut e, r4);
    put_deck_bottom(&mut e, r1);
    put_deck_bottom(&mut e, zhangba);
    e.debug_set_turn(0, Phase::Discard, 4);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let tg4 = e.open_choice().expect("dddyeshen chain targets zhangba").clone();
    assert_eq!(tg4.spec.max, 4, "zhangba zh name len must be 4");
}

#[test]
fn recheck_18_dddzhilian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1584-1675
    // 1) Targets ordered by seat_order_from(turn.current), and PlayCard equip counts!
    let (mut e, d3, p9, aux) = new_recheck_engine(92018, 4);
    let bao = find_general(&e, "ddd_baosanniang");
    e.assign_general(0, bao).unwrap();
    retain_only_skill(&mut e, 0, Some("dddzhilian"));

    let qinglong = find_card_by(&e, |_, d| d.kind == CardKind::Qinglong);
    let sha1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let shan0 = find_card_by(&e, |_, d| d.kind == CardKind::Shan);
    put_in_hand(&mut e, 2, qinglong);
    put_in_hand(&mut e, 2, sha1);
    put_in_hand(&mut e, 0, shan0);
    e.debug_set_turn(2, Phase::Play, 1);
    // Seat 2 plays equip (counts toward zhilian & gives range 3!) then Sha -> Seat 0 (owner!) responds Shan => [2, 0]!
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: qinglong, targets: Vec::new() });
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: sha1, targets: vec![0] });
    answer(&mut e, 0, vec![ChoiceOption::Card(shan0)]);
    drain(&mut e);
    e.debug_set_turn(2, Phase::Discard, 1);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let c = e.open_choice().expect("dddzhilian_mode at TurnPhaseEnd").clone();
    assert_eq!((c.observer, c.spec.prompt), (0, "dddzhilian_mode"));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Tag(0)]);
    let h2_before = e.state_core().player(2).hand.len();
    let h0_before = e.state_core().player(0).hand.len();
    let log_start = e.log().len();
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    let drawn_order: Vec<u8> = e.log()[log_start..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardsDrawn { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(&drawn_order[..2], &[2, 0], "dddzhilian draw order must follow seat_order_from(turn.current)");
    assert!(e.state_core().player(0).hand.contains(&sha1));
    assert_eq!(e.state_core().player(2).hand.len(), h2_before + 1);
    assert_eq!(e.state_core().player(0).hand.len(), h0_before + 2);
    drain(&mut e);

    // 2) Disqualified when a dead 3rd player used/responded earlier this turn (3 total actors > 2!):
    let (mut e2, _d3, _p9, _aux) = new_recheck_engine(92118, 4);
    e2.assign_general(0, bao).unwrap();
    retain_only_skill(&mut e2, 0, Some("dddzhilian"));
    let aoe = find_card_by(&e2, |_, d| d.kind == CardKind::ArcheryAttack);
    let shan_p1 = find_card_by(&e2, |_, d| d.kind == CardKind::Shan);
    let shan_p2 = find_card_by(&e2, |id, d| id != shan_p1 && d.kind == CardKind::Shan);
    put_in_hand(&mut e2, 0, aoe);
    put_in_hand(&mut e2, 1, shan_p1);
    put_in_hand(&mut e2, 2, shan_p2);
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: aoe, targets: Vec::new() });
    answer(&mut e2, 1, vec![ChoiceOption::Card(shan_p1)]);
    answer(&mut e2, 2, vec![ChoiceOption::Card(shan_p2)]);
    drain(&mut e2);
    let lethal = e2.debug_hp(1) as u32;
    let _ = e2.debug_deal_damage(Some(0), 1, lethal, DamageNature::Normal);
    drain(&mut e2);
    assert!(!e2.state_core().player(1).alive);
    e2.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    assert!(e2.open_choice().is_none(), "dead 3rd actor (0,1,2) must disqualify dddzhilian");
}
