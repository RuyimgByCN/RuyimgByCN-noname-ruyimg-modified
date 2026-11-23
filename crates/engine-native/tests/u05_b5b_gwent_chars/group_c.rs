//! U05-5b Group C (18 skills: `lingji`, `gwjinyan`, `gwshenyu`, `junchi`, `hupeng`, `hupeng2`,
//! `hupeng3`, `hupeng4`, `hunmo`, `shuijian`, `yunhuo`, `yinzhang`, `gwtianbian`, `gwxiaoshou`,
//! `kuanglie`, `gwjiquan`, plus `hunmo2` & `kuanglie2`).

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b5b_lingji_gwjinyan_gwshenyu_junchi() {
    let (v, e0) = setup_gwent_locked(5620);
    // 1. lingji (`character/gwent/skill.js:1895-1940`): draw 2, discard 2; same suit->bronze, same rank->silver
    let mut e = grant_skill(&e0, &v, "lingji", 0);
    let c1 = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let rank1 = e0.catalog().cards[c1 as usize].rank;
    let c2 = find_cards_by(&e0, |d| d.suit == Suit::Spade && d.rank == rank1 && d.name != e0.catalog().cards[c1 as usize].name, 1)
        .into_iter()
        .next()
        .unwrap_or_else(|| find_cards_by(&e0, |d| d.suit == Suit::Spade, 2)[1]);
    let rank2 = e0.catalog().cards[c2 as usize].rank;
    e = set_deck_top(&e, &v, &[c1, c2]);
    activate(&mut e, 0, "lingji");
    let ch = e.open_choice().cloned().expect("lingji discard 2");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)] });
    let expected_spells = usize::from(e.catalog().cards[c1 as usize].suit == e.catalog().cards[c2 as usize].suit)
        + usize::from(rank1 == rank2);
    assert_eq!(e.debug_hand(0).len(), expected_spells, "lingji grants bronze/silver spells on suit/rank match");

    // 2. gwjinyan (`character/gwent/skill.js:1941-1990`): round % 3 != 0 prevents damage; round % 3 == 0 gains gold spell
    let mut e = grant_skill(&e0, &v, "gwjinyan", 0);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, None);
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    assert_eq!(e.debug_hp(0), 4, "gwjinyan cancels damage on round % 3 != 0");
    // Advance until round % 3 == 0 on seat 0 Prepare -> gains gold spell & damage applies
    for _ in 0..3 {
        advance_to_seat_prepare(&mut e, 0);
        if e.turn().round % 3 == 0 {
            break;
        }
        advance_past_turn(&mut e, 0);
    }
    assert!(e.turn().round % 3 == 0);
    let target_round = e.turn().round;
    advance_to_seat_play(&mut e, 0);
    assert_eq!(e.turn().round, target_round, "same round in seat 0 Play");
    assert!(!e.debug_hand(0).is_empty(), "gwjinyan_gain gives gold spell on round 3");
    let e = inject_hand(&e, &v, &[sha], 1);
    let mut e = e;
    advance_to_seat_play(&mut e, 1);
    assert_eq!(e.turn().round, target_round, "same round in seat 1 Play");
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    assert_eq!(e.debug_hp(0), 3, "gwjinyan does NOT block damage when round % 3 == 0");

    // 3. gwshenyu (`character/gwent/skill.js:1991-2091`): recover 1 OR take untagged non-gold spell from discard
    let mut e = grant_skill(&e0, &v, "gwshenyu", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 2;
    let silver_card = find_card(&e0, "gw_youer");
    snap.core.deck.retain(|x| *x != silver_card);
    snap.core.discard.push(silver_card);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(0));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Tag(1)); // pick non-gold spell
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(silver_card));
    assert!(e.debug_hand(0).contains(&silver_card), "gwshenyu gained silver spell from discard");
    // Empty/tagged discard branch: second turn can only recover (no untagged non-gold spell left)
    advance_to_seat_play(&mut e, 3);
    let mut snap = e.save();
    let mut leftover = std::mem::take(&mut snap.core.discard);
    snap.core.deck.append(&mut leftover);
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(0));
    assert_eq!(e.debug_hp(0), 3, "gwshenyu falls back to direct recover when no untagged non-gold spell exists");

    // 4. junchi (`character/gwent/skill.js:2092-2130`): other Sha not targeting self -> use card ignoring dist on target & draw 1 (1/turn)
    let mut e = grant_skill(&e0, &v, "junchi", 0);
    let shas = find_cards_by(&e0, |d| d.kind == CardKind::Sha, 3);
    e = inject_hand(&e, &v, &[shas[0]], 1);
    e = inject_hand(&e, &v, &[shas[1], shas[2]], 0);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: shas[0], targets: vec![2] }); // enemy 1 attacks ally 2
    drain_default(&mut e); // sha resolves -> shaAfter triggers junchi on seat 0
    if let Some(ch) = e.open_choice().cloned() {
        assert_eq!(ch.observer, 0);
        submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Card(shas[1])] });
    }
    drain_default(&mut e);
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_hupeng_dudayuanshuai1_dudayuanshuai2_restrictions_and_triggers() {
    // Upstream: `character/gwent/skill.js:2131-2274`
    let (v, e0) = setup_gwent_locked(5621);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, None);
    let other_cards = find_cards_by(&e0, |d| d.kind != CardKind::Sha, 5);
    let mut e = grant_skill(&e0, &v, "hupeng", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    activate(&mut e, 0, "hupeng");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    // Give dudayuanshuai2 (tag 1) to enemy 1
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Tag(1));
    let d2 = find_card(&e0, "gw_dudayuanshuai2");
    assert!(e.debug_hand(1).contains(&d2));

    // hupeng2: gw_dudayuanshuai2 cannot be discarded in Discard phase & cannot be played;
    // hupeng3: phaseEnd discards all gw_dudayuanshuai2 + equal random other cards
    e = inject_hand(&e, &v, &[other_cards[0]], 1);
    advance_to_seat_play(&mut e, 1);
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: d2, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::WrongPhase | RejectReason::CardBlocked | RejectReason::IllegalTargets)
    ));
    submit(&mut e, 1, GameAction::NextPhase); // Play -> Discard
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "discard_to_limit" {
            assert!(!c.spec.options.contains(&ChoiceOption::Card(d2)), "hupeng2 prevents discarding d2");
        }
    }
    advance_past_turn(&mut e, 1);
    assert!(!e.debug_hand(1).contains(&d2), "hupeng3 must remove gw_dudayuanshuai2 at phaseEnd");

    // hupeng4: friend 2 receives gw_dudayuanshuai1; enemy plays duel/sha on 2 -> hupeng4 cancels & gains that card
    let mut e = grant_skill(&e0, &v, "hupeng", 0);
    let d1 = find_card(&e0, "gw_dudayuanshuai1");
    let guohe = card_of_kind_and_suit(&e0, CardKind::Dismantle, None);
    e = inject_hand(&e, &v, &[d1], 2);
    e = inject_hand(&e, &v, &[guohe], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: guohe, targets: vec![2] });
    // seat 2 gets hupeng4 prompt -> use d1 to cancel guohe and gain guohe!
    let ch = e.open_choice().cloned().expect("hupeng4 prompt for gw_dudayuanshuai1");
    assert_eq!(ch.observer, 2);
    submit(&mut e, 2, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Card(d1)] });
    drain_default(&mut e);
    assert!(e.debug_hand(2).contains(&guohe), "hupeng4 gw_dudayuanshuai1 gained incoming card");
    assert!(!e.debug_hand(2).contains(&d1) && is_vanished(&e, d1));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_hunmo_shuijian_yunhuo_yinzhang_gwtianbian_gwxiaoshou_kuanglie_gwjiquan() {
    let (v, e0) = setup_gwent_locked(5622);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let other_cards = find_cards_by(&e0, |_| true, 6);

    // 1. hunmo (`character/gwent/skill.js:2275-2354`): hand<2 -> peek top 2, take 1 give 1; hand>=2 -> discard 1, discard enemy 1 visible
    let mut e = grant_skill(&e0, &v, "hunmo", 0);
    let top2 = [other_cards[0], other_cards[1]];
    e = set_deck_top(&e, &v, &top2);
    e = inject_hand(&e, &v, &[other_cards[2]], 1);
    activate(&mut e, 0, "hunmo");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(top2[0]));
    assert!(e.debug_hand(0).contains(&top2[0]));
    assert!(e.debug_hand(1).contains(&top2[1]));
    // Now seat 1 has hunmo2 -> cannot target 1 again this turn; give 0 >=2 cards and target 3 (>=2 cards)
    e = inject_hand(&e, &v, &[other_cards[3]], 0);
    e = inject_hand(&e, &v, &[other_cards[4], other_cards[5]], 3);
    activate(&mut e, 0, "hunmo");
    let tgt = e.open_choice().cloned().expect("hunmo >=2 target");
    assert!(!tgt.spec.options.contains(&ChoiceOption::Player(1)), "hunmo2 prevents re-targeting seat 1");
    submit(&mut e, 0, GameAction::Answer { choice_id: tgt.id, selection: vec![ChoiceOption::Player(3)] });
    answer_first(&mut e, 0, |o| matches!(o, ChoiceOption::Card(_)));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(other_cards[4]));
    assert!(!e.debug_hand(3).contains(&other_cards[4]));

    // 2. shuijian (`character/gwent/skill.js:2355-2392`): phaseBegin discard 1 -> virtual wanjian ONLY on enemies (friend 2 untouched!)
    let mut e = grant_skill(&e0, &v, "shuijian", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.debug_hp(3), 3);
    assert_eq!(e.debug_hp(2), 4, "shuijian only targets enemies");

    // 3. yinzhang (`character/gwent/skill.js:2418-2445`): discard 1 -> pick 1 of 3 silver spells
    let mut e = grant_skill(&e0, &v, "yinzhang", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    activate(&mut e, 0, "yinzhang");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 0, |o| matches!(o, ChoiceOption::Tag(_)));
    assert_eq!(e.debug_hand(0).len(), 1);

    // 4. gwtianbian (`character/gwent/skill.js:2446-2516`): phaseUseBegin optional +AOE/-AOE/cancel
    let mut e = grant_skill(&e0, &v, "gwtianbian", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_play_or_choice(&mut e, 0);
    let ch = e.open_choice().cloned().expect("gwtianbian at Play begin");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Tag(1)] });
    drain_default(&mut e);
    assert!(e.debug_hp(1) < 4 || e.debug_hp(3) < 4 || !e.debug_hand(1).is_empty(), "gwtianbian negative AOE fired");

    // 5. gwxiaoshou (`character/gwent/skill.js:2517-2539`, usable: 2): discard he, damage any maxHp player
    let mut e = grant_skill(&e0, &v, "gwxiaoshou", 0);
    e = inject_hand(&e, &v, &[other_cards[0], other_cards[1], other_cards[2]], 0);
    let mut snap = e.save();
    snap.core.players[1].hp = 4;
    snap.core.players[3].hp = 2;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    activate(&mut e, 0, "gwxiaoshou");
    answer_first(&mut e, 0, |o| matches!(o, ChoiceOption::Card(_)));
    let ch = e.open_choice().cloned().expect("gwxiaoshou maxHp target");
    assert!(!ch.spec.options.contains(&ChoiceOption::Player(3)), "seat 3 is not maxHp");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3);
    // Cancel does not consume use; 2nd committed use succeeds; 3rd rejected
    assert!(matches!(activate(&mut e, 0, "gwxiaoshou"), CommandOutcome::Pending { .. }));
    answer_cancel(&mut e, 0);
    assert!(matches!(activate(&mut e, 0, "gwxiaoshou"), CommandOutcome::Pending { .. }));
    answer_first(&mut e, 0, |o| matches!(o, ChoiceOption::Card(_)));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(2));
    assert_eq!(e.debug_hp(2), 3, "gwxiaoshou 2nd committed use succeeds");
    assert!(matches!(activate(&mut e, 0, "gwxiaoshou"), CommandOutcome::Rejected(RejectReason::InvalidChoice)), "gwxiaoshou 3rd use in same turn rejected");

    // 6. kuanglie (`character/gwent/skill.js:2540-2557`): black card targeted at other w/ he -> random discard target + draw 1 every 2 procs
    let mut e = grant_skill(&e0, &v, "kuanglie", 0);
    let black_spade = card_of_kind_and_suit(&e0, CardKind::Duel, Some(Suit::Spade));
    let black_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Club));
    e = inject_hand(&e, &v, &[black_spade, black_sha], 0);
    e = inject_hand(&e, &v, &[other_cards[0]], 1);
    e = inject_hand(&e, &v, &[other_cards[1]], 3);
    submit(&mut e, 0, GameAction::PlayCard { card: black_spade, targets: vec![1] });
    drain_default(&mut e);
    assert!(e.debug_hand(1).is_empty(), "kuanglie #1 random discards target 1");
    submit(&mut e, 0, GameAction::PlayCard { card: black_sha, targets: vec![3] });
    drain_default(&mut e);
    assert!(e.debug_hand(3).is_empty(), "kuanglie #2 random discards target 3");
    assert_eq!(e.debug_hand(0).len(), 1, "kuanglie drew 1 on 2nd trigger");

    // 7. gwjiquan (`character/gwent/skill.js:2558-2612`): gain 1 he from each selected target, then target uses virtual sha on self
    let mut e = grant_skill(&e0, &v, "gwjiquan", 0);
    e = inject_hand(&e, &v, &[other_cards[0]], 1);
    activate(&mut e, 0, "gwjiquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    answer_first(&mut e, 0, |_| true);
    pass_window(&mut e);
    assert!(e.debug_hand(0).contains(&other_cards[0]), "gwjiquan gained card from 1");
    assert_eq!(e.debug_hp(0), 3, "seat 1 countered with virtual sha");

    // 8. yunhuo (`character/gwent/skill.js:2393-2417`): round % 4 == 0 grants extra turn & discards/fire damages enemies
    let mut e = grant_skill(&e0, &v, "yunhuo", 0);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 1);
    let mut snap = e.save();
    snap.core.turn.round = 3;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_prepare(&mut e, 0);
    assert_eq!(e.turn().round, 4, "wrapped into round 4");
    assert_eq!(e.debug_hp(1), 3, "yunhuo fire-damaged handless enemy on round 4");
    assert_zone_conservation(&e);
}
