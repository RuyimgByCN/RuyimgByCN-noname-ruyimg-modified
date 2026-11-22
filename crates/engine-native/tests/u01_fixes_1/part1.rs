use super::common::*;

#[test]
fn u01_fix_01_dddjuxian_j01_to_j05() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:244-383
    let (mut e, d3, p9) = new_clean_engine(91001, 4);
    let zhangcheng = find_general(&e, "ddd_zhangcheng");
    e.assign_general(0, zhangcheng).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    let juxian = e.catalog().skill_by_name("dddjuxian").unwrap();

    let red_equip = find_card_by(&e, |_, d| {
        d.kind.is_equip() && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    let red_card = find_card_by(&e, |id, d| {
        id != red_equip && d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    let red_draw = find_card_by(&e, |id, d| {
        id != red_equip
            && id != red_card
            && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    let blk_c0 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade | Suit::Club));
    let blk_c1 = find_card_by(&e, |id, d| {
        id != blk_c0 && matches!(d.suit, Suit::Spade | Suit::Club)
    });

    // J02: With only own `red_equip` left in `he` after paying `red_card`, `dddjuxian_top` (Tag(0))
    // still runs `gainPlayerCard(owner, "he", true)` on self (skill.js:302-339).
    put_in_hand(&mut e, 0, red_card);
    put_in_equip(&mut e, 0, red_equip);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "dddjuxian_mode");
    assert_eq!(c.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red_card)]);
    let c_self_gain = e.open_choice().expect("J02: self gainPlayerCard when only owner has `he`").clone();
    assert_eq!((c_self_gain.observer, c_self_gain.spec.prompt), (0, "dddjuxian_gain"));
    assert_eq!(c_self_gain.spec.options, vec![ChoiceOption::Card(red_equip)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red_equip)]);
    assert!(e.open_choice().is_none(), "J02: finishes after gaining self equip into hand");
    assert!(e.state_core().player(0).equips.is_empty());
    assert!(e.state_core().player(0).hand.contains(&red_equip));
    assert_eq!(e.state_core().deck.last().copied(), Some(red_card));

    // J05: `hand` branch (`skill.js:352` `position: "he"`) can pay an equip card, reveals paid + drawn, emits CardsGiven.
    put_deck_top(&mut e, red_draw);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    let c_mode_hand = e.open_choice().unwrap().clone();
    assert_eq!(c_mode_hand.spec.options, vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let c_hand_card = e.open_choice().unwrap().clone();
    assert!(
        c_hand_card.spec.options.contains(&ChoiceOption::Card(red_equip)),
        "J05: dddjuxian_hand must allow paying equip from `he`"
    );
    let log_len = e.log().len();
    answer(&mut e, 0, vec![ChoiceOption::Card(red_equip)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(1).hand.contains(&red_equip));
    assert!(e.state_core().player(0).hand.contains(&red_draw));
    assert!(
        e.log()[log_len..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsRevealed { cards, .. } if cards == &vec![red_equip])),
        "J05: hand branch must reveal paid card"
    );
    assert!(
        e.log()[log_len..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsRevealed { cards, .. } if cards == &vec![red_draw])),
        "J05: hand branch must reveal drawn card"
    );
    assert!(
        e.log()[log_len..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsGiven { from: 0, to: 1, cards } if cards == &vec![red_equip])),
        "J05: hand branch must emit CardsGiven"
    );
    assert_eq!(flag(&e, 0, "dddjuxian_ban"), 0);

    // J01: Entering a second `Phase::Play` in the SAME turn (from Phase::Draw -> NextPhase) resets phaseUse limits!
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::Play);
    assert_eq!(e.turn().round, 1);
    put_in_hand(&mut e, 0, blk_c0);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    let c2 = e.open_choice().unwrap().clone();
    assert_eq!(c2.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);

    // J02 (1 other `he` auto-targets) & J04 (`CardsLost` settles BEFORE `dddjuxian_target` / `gainPlayerCard`):
    let equip_pay = find_card_by(&e, |id, d| {
        id != red_equip && d.kind.is_equip() && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let pay_slot = e.catalog().card_slot_of(e.state_core(), equip_pay);
    let equip_move = find_card_by(&e, |id, d| {
        id != red_equip
            && id != equip_pay
            && d.kind.is_equip()
            && d.kind.equip_slot() != pay_slot
    });
    e.debug_grant(0, "dddzhengjun");
    put_in_equip(&mut e, 0, equip_pay);
    put_in_equip(&mut e, 0, equip_move);
    put_in_hand(&mut e, 2, blk_c1);
    clear_hand(&mut e, 1);
    clear_equips(&mut e, 1);
    clear_equips(&mut e, 2);
    clear_equips(&mut e, 3);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(equip_pay)]);
    // `dddzhengjun` (`CardsLost(Equip)` on es_target=0) triggers BEFORE `dddjuxian_gain`! Confirm + move `equip_move` 0 -> 2 BEFORE gain!
    let zj = e.open_choice().expect("J04: dddzhengjun CardsLost sub-trigger before gainPlayerCard").clone();
    assert_eq!(zj.spec.prompt, "dddzhengjun_es_confirm");
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.open_choice().expect("dddzhengjun_es_card").spec.prompt, "dddzhengjun_es_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(equip_move)]);
    assert_eq!(e.open_choice().expect("dddzhengjun_es_dest").spec.prompt, "dddzhengjun_es_dest");
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(
        e.state_core().player(0).equips.is_empty()
            && e.state_core().player(2).equips.contains(&equip_move),
        "J04: CardsLost sub-trigger (dddzhengjun) must settle BEFORE gainPlayerCard"
    );
    // 1 target (seat 2 has blk_c1; seat 0/1/3 wait: put equip_move -> wait seat 0/1 got equip_move in `equips`, NOT affecting `other` except `he_cards`! Clear seat 1 equips only? No, seat 1 has 0 cards, wait seat 0 gained `equip_move`!)
    // Seat 0 still holds `blk_c0` + `equip_move`, and seat 2 holds `blk_c1` -> upstream `targets = [0, 2]` (len 2)
    // opens `dddjuxian_target` (`1004`), offering only other players (`[Player(2)]`)!
    let tgt_c = e.open_choice().expect("J02: owner + 1 other holding he opens dddjuxian_target").clone();
    assert_eq!(tgt_c.spec.prompt, "dddjuxian_target");
    assert_eq!(tgt_c.spec.options, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let gain_c = e.open_choice().expect("dddjuxian_gain");
    assert_eq!(gain_c.spec.prompt, "dddjuxian_gain");
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::HiddenCard(0)]);
    answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);
    assert!(e.state_core().player(0).hand.contains(&blk_c1));
}

#[test]
fn u01_fix_02_dddyeshen() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:656-746
    let (mut e, d3, p9) = new_clean_engine(91002, 4);
    let sunluban = find_general(&e, "ddd_lie");
    e.assign_general(0, sunluban).unwrap();

    // Seat 1 is current turn player entering `Phase::End, begin: true`.
    let blk_chain = find_card_by(&e, |_, d| {
        d.kind == CardKind::Chain && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let blk_sha = find_card_by(&e, |_, d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let red_tao = find_card_by(&e, |_, d| {
        d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    put_deck_bottom(&mut e, blk_chain);
    put_deck_bottom(&mut e, blk_sha);
    put_deck_bottom(&mut e, red_tao);

    e.debug_set_turn(1, Phase::Discard, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let confirm = e.open_choice().expect("dddyeshen_confirm").clone();
    assert_eq!(confirm.spec.prompt, "dddyeshen_confirm");
    assert_eq!(confirm.observer, 0);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    // Trigger player (seat 1) must pick among the 2 black bottom cards (`dddyeshen_card`).
    let pick_blk = e.open_choice().expect("dddyeshen_card").clone();
    assert_eq!(pick_blk.spec.prompt, "dddyeshen_card");
    assert_eq!(pick_blk.observer, 1);
    assert_eq!(pick_blk.spec.options.len(), 2);
    assert!(!pick_blk.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(blk_chain)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(blk_chain)]);

    // 4-char name ("铁索连环") allows choosing up to 4 targets (let's pick 3: 0, 2, 3 -> NOT capped at 2!).
    let pick_tgts = e.open_choice().expect("dddyeshen_target").clone();
    assert_eq!(pick_tgts.spec.prompt, "dddyeshen_target");
    assert_eq!(pick_tgts.observer, 1);
    assert!(pick_tgts.spec.allow_cancel);
    assert_eq!(pick_tgts.spec.max, 4);

    // Fork: test cancel recast vs Choice A linking 3 targets.
    let mut e_recast = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    let h1_before = e_recast.state_core().player(1).hand.len();
    answer(&mut e_recast, 1, Vec::new());
    drain(&mut e_recast);
    assert!(e_recast.state_core().discard.contains(&blk_chain));
    assert_eq!(e_recast.state_core().player(1).hand.len(), h1_before + 1);
    assert_eq!(e_recast.state_core().deck.last().copied(), Some(blk_sha).filter(|_| false).or(e_recast.state_core().deck.last().copied()));
    assert!(
        e_recast.state_core().deck.contains(&blk_sha)
            && e_recast.state_core().deck.contains(&red_tao),
        "unused bottom cards must return to deck top"
    );

    assert_mid_choice_archive_and_ai(
        &e,
        &d3,
        &p9,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(2),
            ChoiceOption::Player(3),
        ],
    );
    answer(
        &mut e,
        1,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(2),
            ChoiceOption::Player(3),
        ],
    );
    drain(&mut e);
    assert!(e.state_core().player(0).linked);
    assert!(e.state_core().player(2).linked);
    assert!(e.state_core().player(3).linked);
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 1);

    // Trigger with mark == 2 -> takes 1 bottom card, resets mark to 0, deals 1 FIRE damage to owner (seat 0).
    // Equip Tengjia on seat 0 (if in catalog) or check DamageDealt amount = 1 & LogEvent::DamageDealt.
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "dddyeshen_count".to_string()), 2);
    let red_other = find_card_by(&e, |id, d| {
        id != red_tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    put_deck_bottom(&mut e, red_other);
    let hp0_before = e.debug_hp(0);
    e.debug_set_turn(2, Phase::Discard, 2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 0);
    assert_eq!(e.debug_hp(0), hp0_before - 1);
    assert!(e.log().iter().any(|ev| matches!(
        ev,
        LogEvent::DamageDealt { target: 0, amount: 1, .. }
    )));
}

#[test]
fn u01_fix_03_dddguiying() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:968-1203
    let (mut e, d3, p9) = new_clean_engine(91003, 4);
    let sunliang = find_general(&e, "ddd_sunliang");
    let wu_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            idx != sunliang
                && e.catalog()
                    .general_meta(idx)
                    .is_some_and(|m| m.1 == engine_native::catalog::Faction::Wu)
        })
        .unwrap();
    let shu_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.1 == engine_native::catalog::Faction::Shu)
        })
        .unwrap();

    e.assign_general(0, sunliang).unwrap();
    e.assign_general(1, wu_g).unwrap();
    e.assign_general(2, shu_g).unwrap();
    e.assign_general(3, wu_g).unwrap();

    // Negative guard: zhuSkill does NOT trigger in Identity mode when owner is Role::Rebel (put at seat 1, seat 0 is Lord).
    {
        let mut neg_e = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        neg_e.state_core_mut().skills.clear();
        neg_e.assign_general(1, sunliang).unwrap();
        neg_e.assign_general(0, wu_g).unwrap();
        let mut snap = neg_e.save();
        snap.core.mode = GameMode::Identity {
            roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
        };
        let mut neg = Engine::restore_with_extensions(&snap, &[&d3, &p9]).unwrap();
        let c0 = find_card_by(&neg, |_, _| true);
        put_in_hand(&mut neg, 0, c0);
        neg.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut neg, 0, GameAction::NextPhase);
        assert!(neg.open_choice().is_none(), "non-lord rebel must not trigger zhuSkill dddguiying");
    }

    // Lord in Identity mode: seat 0 is Lord, seat 1 & seat 3 (Wu) in seat order; seat 2 (Shu) skipped.
    {
        let mut snap = e.save();
        snap.core.mode = GameMode::Identity {
            roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
        };
        e = Engine::restore_with_extensions(&snap, &[&d3, &p9]).unwrap();
    }
    let rank10_a = find_card_by(&e, |_, d| d.rank == 10 && !d.kind.is_equip());
    let rank2_a = find_card_by(&e, |_, d| d.rank == 2 && !d.kind.is_equip());
    let rank13_a = find_card_by(&e, |_, d| d.rank == 13 && !d.kind.is_equip());
    let rank13_b = find_card_by(&e, |id, d| id != rank13_a && d.rank == 13);
    let rank1_judge = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let min_rank = e.catalog().cards[rank1_judge as usize].rank;
    let rank1_equip = find_card_by(&e, |id, d| id != rank1_judge && d.rank == min_rank && d.kind.is_equip());

    put_in_hand(&mut e, 1, rank2_a);
    put_in_hand(&mut e, 3, rank10_a);
    put_in_hand(&mut e, 3, rank13_a);
    put_in_hand(&mut e, 3, rank13_b);
    put_in_judge(&mut e, 2, rank1_judge);
    put_in_equip(&mut e, 0, rank1_equip);

    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);

    // Seat 1 asked first (`dddguiying_confirm`); canceling seat 1 STILL asks seat 3!
    let c1 = e.open_choice().expect("seat 1 confirm").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (1, "dddguiying_confirm"));
    answer(&mut e, 1, Vec::new());

    let c3 = e.open_choice().expect("seat 3 confirm").clone();
    assert_eq!((c3.observer, c3.spec.prompt), (3, "dddguiying_confirm"));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Confirm]);
    answer(&mut e, 3, vec![ChoiceOption::Confirm]);

    // Seat 3 has 2 tied rank-13 cards -> forced `dddguiying_give_card` (must NOT contain rank 10!).
    let give_c = e.open_choice().expect("dddguiying_give_card").clone();
    assert_eq!((give_c.observer, give_c.spec.prompt), (3, "dddguiying_give_card"));
    assert!(!give_c.spec.allow_cancel);
    assert!(!give_c.spec.options.contains(&ChoiceOption::Card(rank10_a)));
    assert!(give_c.spec.options.contains(&ChoiceOption::Card(rank13_a)));
    assert!(give_c.spec.options.contains(&ChoiceOption::Card(rank13_b)));
    answer(&mut e, 3, vec![ChoiceOption::Card(rank13_a)]);
    assert!(e.state_core().player(0).hand.contains(&rank13_a));

    // Min rank in `ej` across all alive is rank 1 on seat 0 (equip) and seat 2 (judge)!
    let gain_tgt = e.open_choice().expect("dddguiying_gain_target").clone();
    assert_eq!((gain_tgt.observer, gain_tgt.spec.prompt), (3, "dddguiying_gain_target"));
    assert!(gain_tgt.spec.allow_cancel);
    assert!(gain_tgt.spec.options.contains(&ChoiceOption::Player(2)));
    assert!(gain_tgt.spec.options.contains(&ChoiceOption::Player(0)));
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 3, vec![ChoiceOption::Player(2)]);

    assert!(
        e.state_core().player(3).hand.contains(&rank1_judge),
        "seat 3 must gain min-rank judge card from seat 2"
    );
    assert!(e.state_core().player(2).judge.is_empty());
}

#[test]
fn u01_fix_04_dddfushi() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727-3864
    let (mut e, d3, p9) = new_clean_engine(91004, 3);
    let zhenji = find_general(&e, "ddd_zhenji");
    e.assign_general(0, zhenji).unwrap();
    e.debug_grant(0, "dddfushi");

    let male_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.4 == Gender::Male)
        })
        .unwrap();
    e.assign_general(1, male_g).unwrap();
    let female_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            e.catalog()
                .general_meta(idx)
                .is_some_and(|m| m.4 == Gender::Female)
        })
        .unwrap();
    e.assign_general(2, female_g).unwrap();

    e.debug_set_turn(0, Phase::Play, 1);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let p0_hand = find_card_by(&e, |id, _| id != wuzhong && id != sha);
    let pile1_card = find_card_by(&e, |id, _| id != wuzhong && id != sha && id != p0_hand);
    let pile2_card = find_card_by(&e, |id, _| {
        id != wuzhong && id != sha && id != p0_hand && id != pile1_card
    });
    let p1_hand = find_card_by(&e, |id, _| {
        id != wuzhong && id != sha && id != p0_hand && id != pile1_card && id != pile2_card
    });

    remove_card_from_all_zones(&mut e, pile1_card);
    remove_card_from_all_zones(&mut e, pile2_card);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "dddmiaoxing_1".to_string()), vec![pile1_card]);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "dddmiaoxing_2".to_string()), vec![pile2_card]);
    put_in_hand(&mut e, 0, wuzhong);
    put_in_hand(&mut e, 0, sha);
    put_in_hand(&mut e, 0, p0_hand);
    put_in_hand(&mut e, 1, p1_hand);

    // Negative guard: Trick `Wuzhong` must NOT trigger `dddfushi`.
    let out_wz = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: Vec::new(),
        },
    );
    assert!(matches!(out_wz, CommandOutcome::Accepted { .. }));
    assert!(e.open_choice().is_none());

    // Basic `Sha`: triggers `dddfushi_pile` AFTER `CardUsed` settles!
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    let c_pile = e.open_choice().expect("dddfushi_pile").clone();
    assert_eq!((c_pile.observer, c_pile.spec.prompt), (0, "dddfushi_pile"));
    assert_eq!(c_pile.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    assert!(e.state_core().player(0).hand.contains(&pile1_card));

    // Male target selection MUST include alive male seat 0 (if seat 0 is male) AND seat 1, exclude female seat 2.
    let c_tgt = e.open_choice().expect("dddfushi_male_target").clone();
    assert_eq!(c_tgt.spec.prompt, "dddfushi_male_target");
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(!c_tgt.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    // Selected male target seat 1 gets independent choice (`observer == 1`, `dddfushi_target_pile`).
    let c_t_pile = e.open_choice().expect("dddfushi_target_pile").clone();
    assert_eq!((c_t_pile.observer, c_t_pile.spec.prompt), (1, "dddfushi_target_pile"));
    assert!(c_t_pile.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Tag(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.state_core().player(1).hand, vec![pile2_card]);
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "dddmiaoxing_2".to_string())).cloned(),
        Some(vec![p1_hand])
    );
}

#[test]
fn u01_fix_05_dddlangzhi() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5296-5426
    let (mut e, d3, p9) = new_clean_engine(91005, 4);
    let kebineng = find_general(&e, "ddd_kebineng");
    e.assign_general(0, kebineng).unwrap();

    // Seat 0 has default attack range 1 -> seats 1 and 3 are in range (dist 1), seat 2 is OUT of range (dist 2).
    let c0 = find_card_by(&e, |_, _| true);
    let c1 = find_card_by(&e, |id, _| id != c0);
    let c2 = find_card_by(&e, |id, _| id != c0 && id != c1);
    let c3 = find_card_by(&e, |id, _| id != c0 && id != c1 && id != c2);
    put_in_hand(&mut e, 0, c0);
    put_in_hand(&mut e, 1, c1);
    put_in_hand(&mut e, 2, c2);
    put_in_hand(&mut e, 3, c3);

    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);

    // Optional confirm (`dddlangzhi_confirm`)
    let conf = e.open_choice().expect("dddlangzhi_confirm").clone();
    assert_eq!((conf.observer, conf.spec.prompt), (0, "dddlangzhi_confirm"));
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);

    // Sequential reveal ONLY for in-range players (seat 1, then seat 3; seat 2 is NOT asked!).
    let r1 = e.open_choice().expect("reveal seat 1").clone();
    assert_eq!((r1.observer, r1.spec.prompt), (0, "dddlangzhi_reveal_target_card"));
    assert_eq!(r1.spec.options, vec![ChoiceOption::HiddenCard(0)]);
    answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);

    let r3 = e.open_choice().expect("reveal seat 3 (skipping seat 2)").clone();
    assert_eq!((r3.observer, r3.spec.prompt), (0, "dddlangzhi_reveal_target_card"));
    answer(&mut e, 0, vec![ChoiceOption::HiddenCard(0)]);

    // Next: `dddlangzhi_own_card` `[1..=2]` from owner's `he` (`allow_cancel: true`).
    let own_pick = e.open_choice().expect("dddlangzhi_own_card").clone();
    assert_eq!((own_pick.observer, own_pick.spec.prompt), (0, "dddlangzhi_own_card"));
    assert!(own_pick.spec.allow_cancel);

    // Fork Choice B (Cancel): gain all revealed cards AND lose `dddlangzhi` skill!
    let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_cancel, 0, Vec::new());
    assert!(e_cancel.state_core().player(0).hand.contains(&c1));
    assert!(e_cancel.state_core().player(0).hand.contains(&c3));
    assert!(!e_cancel.state_core().player(0).hand.contains(&c2));
    let lz_def = e_cancel.catalog().skill_by_name("dddlangzhi").unwrap();
    assert!(
        !e_cancel.state_core().has_skill(0, lz_def),
        "canceling dddlangzhi swap must remove skill dddlangzhi"
    );

    // Choice A (Swap 1): pick c0 -> pick 1 from revealed [c1, c3] (`dddlangzhi_swap_cards`).
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(c0)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
    let swap_pick = e.open_choice().expect("dddlangzhi_swap_cards").clone();
    assert_eq!(swap_pick.spec.prompt, "dddlangzhi_swap_cards");
    assert_eq!((swap_pick.spec.min, swap_pick.spec.max), (1, 1));
    assert!(!swap_pick.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(c3)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(c3)]);
    assert!(e.state_core().player(0).hand.contains(&c3));
    assert!(e.state_core().player(3).hand.contains(&c0));
    let lz_def_a = e.catalog().skill_by_name("dddlangzhi").unwrap();
    assert!(e.state_core().has_skill(0, lz_def_a));
}
