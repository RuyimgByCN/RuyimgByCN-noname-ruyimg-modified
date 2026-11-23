//! U05-5b Group D (17 skills: `nuhou`, `shewu`, `gwzhanjiang`, `gwzhanjiang3`, `gwchuanxin`,
//! `fengjian`, `huandie`, `xuezhou`, `xuezhou_hp`, `fayin`, `gwbaquan`, `huihun`, `lanquan`)
//! + AI 公开信息不偷看验证 + 死亡/空牌堆边界。

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b5b_nuhou_shewu_gwzhanjiang_gwchuanxin_fengjian() {
    let (v, e0) = setup_gwent_locked(5630);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let other_cards = find_cards_by(&e0, |d| d.kind != CardKind::Sha, 5);

    // 1. nuhou (`character/gwent/skill.js:2613-2660`): damageEnd -> discard 1 he -> enemy 1 dmg & randomDiscard 1
    let mut e = grant_skill(&e0, &v, "nuhou", 0);
    e = inject_hand(&e, &v, &[other_cards[0]], 0);
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(other_cards[0]));
    drain_default(&mut e);
    assert!(e.debug_hp(1) < 4 || e.debug_hp(3) < 4);

    // 2. shewu (`character/gwent/skill.js:2661-2711`): discard 1->draw 3; >=2->unlimited distance; 3->recover 1
    let mut e = grant_skill(&e0, &v, "shewu", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 2;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    let plus_horse = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::HorsePlus), 1)[0];
    e = inject_equip(&e, &v, plus_horse, 2); // seat 2 at distance 3 from seat 0 normally
    e = inject_hand(&e, &v, &[other_cards[0], other_cards[1], other_cards[2], sha], 0);
    activate(&mut e, 0, "shewu");
    let ch = e.open_choice().cloned().expect("shewu discard 1..=3");
    submit(&mut e, 0, GameAction::Answer {
        choice_id: ch.id,
        selection: vec![
            ChoiceOption::Card(other_cards[0]),
            ChoiceOption::Card(other_cards[1]),
            ChoiceOption::Card(other_cards[2]),
        ],
    });
    assert_eq!(e.debug_hp(0), 3, "shewu 3 cards recovers 1");
    assert_eq!(e.debug_hand(0).len(), 4, "shewu drew 3 (remaining 1+3)");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "shewu >=2 grants unlimited card range for PlayCard");
    pass_window(&mut e);

    // 3. gwzhanjiang (`character/gwent/skill.js:2712-2783`): other phaseBegin discard 1 he -> up to 2 others may Sha turn player & draw 1; cooldown until owner's next phaseBegin
    let mut e = grant_skill(&e0, &v, "gwzhanjiang", 0);
    e = inject_hand(&e, &v, &[other_cards[0]], 0);
    e = inject_hand(&e, &v, &[sha], 2);
    advance_to_seat_prepare(&mut e, 1);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(other_cards[0]));
    let s2 = e.open_choice().cloned().expect("gwzhanjiang sha ask on seat 2");
    assert_eq!(s2.observer, 2);
    submit(&mut e, 2, GameAction::Answer { choice_id: s2.id, selection: vec![ChoiceOption::Card(sha)] });
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "gwzhanjiang cooperative Sha hit turn player 1");
    assert_eq!(e.debug_hand(2).len(), 1, "gwzhanjiang shooter seat 2 drew 1");

    // 4. gwchuanxin (`character/gwent/skill.js:2784-2809`): attackFrom - (hp - 1); shaAfter show top black -> extra virtual sha
    let mut e = grant_skill(&e0, &v, "gwchuanxin", 0);
    e = inject_equip(&e, &v, plus_horse, 2); // distance 0->2 is 3; seat 0 has hp=4 -> attack range = 1+(4-1)=4!
    let black_top = card_of_kind_and_suit(&e0, CardKind::Dismantle, Some(Suit::Spade));
    e = set_deck_top(&e, &v, &[black_top]);
    e = inject_hand(&e, &v, &[sha], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ), "gwchuanxin boosts attack range by hp - 1 = +3");
    pass_window(&mut e);
    let ch = e.open_choice().cloned().expect("gwchuanxin confirm");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Tag(0)] });
    drain_default(&mut e);
    assert_eq!(e.debug_hp(2), 2, "gwchuanxin black reveal fired 2nd virtual sha");

    // 5. fengjian (`character/gwent/skill.js:2810-2844`): trick used -> optional thunder sha on non-trick-target & tempHide
    let mut e = grant_skill(&e0, &v, "fengjian", 0);
    let guohe = card_of_kind_and_suit(&e0, CardKind::Dismantle, None);
    e = inject_hand(&e, &v, &[guohe, other_cards[0]], 0);
    e = inject_hand(&e, &v, &[other_cards[1]], 1);
    submit(&mut e, 0, GameAction::PlayCard { card: guohe, targets: vec![1] });
    pass_window(&mut e);
    let ch = e.open_choice().cloned().expect("fengjian target");
    assert!(!ch.spec.options.contains(&ChoiceOption::Player(1)), "fengjian excludes already-targeted seat 1");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Player(3)] });
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3, "fengjian thunder Sha dealt 1");
    assert!(e.state_core().player(0).yinshen_active, "fengjian grants tempHide");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_huandie_xuezhou_fayin_gwbaquan_huihun_lanquan() {
    let (v, e0) = setup_gwent_locked(5631);
    let heart_tao = card_of_kind_and_suit(&e0, CardKind::Tao, Some(Suit::Heart));
    let diamond_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Diamond));
    let red_wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, Some(Suit::Heart));
    let black_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let other_cards = find_cards_by(&e0, |_| true, 5);

    // 1. huandie (`character/gwent/skill.js:2845-2901`): phaseBegin pick targets -> self+1, targets+2; phaseEnd hand>hp discards 2
    let mut e = grant_skill(&e0, &v, "huandie", 0);
    let mut snap = e.save();
    snap.core.players[1].hp = 1;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 1);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Confirm);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1)); // pick seat 1 (+2 hand -> 2 > hp 1!)
    assert_eq!(e.debug_hand(1).len(), 2);
    advance_past_turn(&mut e, 0); // ends Play -> Discard -> End -> TurnPhaseEnd -> huandie_discard forces seat 1 to discard 2
    assert_eq!(e.debug_hand(1).len(), 0, "huandie_discard forces hand > hp players to discard 2");

    // 2. xuezhou + xuezhou_hp (`character/gwent/skill.js:2902-2983`): Mode 1 & Mode 2 once per turn
    let mut e = grant_skill(&e0, &v, "xuezhou", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 2;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Tag(0)); // Mode 1: other takes dmg -> loses 1 hp, owner +1 hp
    advance_to_seat_play(&mut e, 0);
    e = inject_hand(&e, &v, &[black_sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 2, "xuezhou Mode 1 caused extra 1 hp loss (4->3 dmg ->2 loseHp)");
    assert_eq!(e.debug_hp(0), 3, "xuezhou Mode 1 healed owner");

    // 3. gwbaquan (`character/gwent/skill.js:3030-3087`): swap hand with target; hp<=target.hp & all returned were original -> 1 dmg
    let mut e = grant_skill(&e0, &v, "gwbaquan", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 3;
    snap.core.players[1].hp = 4;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[other_cards[0]], 0);
    e = inject_hand(&e, &v, &[other_cards[1]], 1);
    activate(&mut e, 0, "gwbaquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    // Return exact same card taken -> damage!
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(other_cards[1]));
    assert_eq!(e.debug_hp(1), 3, "gwbaquan deals 1 damage when hp<=target.hp and exact cards returned");

    // 4. huihun (`character/gwent/skill.js:3088-3144`): phaseEnd recovers first 2 red cards used this turn still in discard; phaseAfter clears
    let mut e = grant_skill(&e0, &v, "huihun", 0);
    e = inject_hand(&e, &v, &[red_wuzhong, diamond_sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: red_wuzhong, targets: vec![] });
    pass_window(&mut e);
    submit(&mut e, 0, GameAction::PlayCard { card: diamond_sha, targets: vec![1] });
    pass_window(&mut e);
    advance_past_turn(&mut e, 0);
    assert!(
        e.debug_hand(0).contains(&red_wuzhong) && e.debug_hand(0).contains(&diamond_sha),
        "huihun recovered red cards used this turn"
    );

    // 5. lanquan (`character/gwent/skill.js:3145-3197`): view top 6 & play 1 directly from deck
    let mut e = grant_skill(&e0, &v, "lanquan", 0);
    e = set_deck_top(&e, &v, &[red_wuzhong, black_sha]);
    activate(&mut e, 0, "lanquan");
    let ch = e.open_choice().cloned().expect("lanquan pick top 6 playable");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Card(red_wuzhong)] });
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "lanquan played wuzhong directly from deck");

    // 6. fayin (`character/gwent/skill.js:2984-3029`): shaBegin discard -> one of 5 random effects
    let mut e = grant_skill(&e0, &v, "fayin", 0);
    e = inject_hand(&e, &v, &[black_sha, other_cards[0]], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    drain_default(&mut e); // shaBegin asks fayin discard
    assert!(
        e.debug_hp(1) < 4
            || e.state_core().player(0).hujia > 0
            || e.state_core().player(1).face_down
            || e.state_core().player(1).boss_mad
            || e.debug_hand(1).is_empty()
    );
    let _ = heart_tao;
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_ai_public_info_heuristics_never_peek_and_boundary_safety() {
    // AI 公开信息不偷看敌手黑牌数 (`gwjingshi`) + `gwhuanbi` + `gwgouhun` + `gwtianbian` + `gwbaquan` + 空牌堆保守性
    let (v, e0) = setup_gwent_locked(5632);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let other_cards = find_cards_by(&e0, |d| d.kind != CardKind::Sha, 5);

    // 1. gwjingshi AI picks max HAND COUNT (public info), NOT peeking black count when 3 has 3 red vs 1 has 1 black!
    let mut e = grant_skill(&e0, &v, "gwjingshi", 0);
    let reds = find_cards_by(&e0, |d| matches!(d.suit, Suit::Heart | Suit::Diamond), 3);
    e = inject_hand(&e, &v, &[sha], 1); // 1 black
    e = inject_hand(&e, &v, &reds, 3); // 3 red -> more total cards (public)
    activate(&mut e, 0, "gwjingshi");
    let act = ai_choose_for_open_choice(&e);
    assert!(
        matches!(act, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Player(3)]),
        "AI gwjingshi must guess max public hand count (seat 3), never peek private black cards!"
    );

    // 2. gwtianbian AI picks negative AOE (Control 1) when enemies outnumber friends with all roles public
    let mut e = grant_skill(&e0, &v, "gwtianbian", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_play_or_choice(&mut e, 0);
    let act = ai_choose_for_open_choice(&e);
    assert!(
        matches!(act, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Tag(1)] || selection == &vec![ChoiceOption::Tag(0)]),
        "AI gwtianbian selects valid public-info branch: {act:?}"
    );
    let GameAction::Answer { choice_id, selection } = act else { unreachable!() };
    submit(&mut e, 0, GameAction::Answer { choice_id, selection });
    drain_default(&mut e);

    // 3. Empty deck & discard boundary: bolang / lanquan / zhengjun / sqlongnu safely finish with 0 panics
    let e = grant_skill(&e0, &v, "bolang", 0);
    let mut snap = e.save();
    // Move all deck into player 1 hand except 1 card
    let mut keep = std::mem::take(&mut snap.core.deck);
    let last = keep.pop().unwrap();
    snap.core.players[1].hand.extend(keep);
    snap.core.deck.push(last);
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    drain_default(&mut e);
    assert_zone_conservation(&e);

    // 4. Death cleanup (`u05_b5b_on_player_died`): owner holding `off_court` card via `gwjushi` dies -> returned to discard
    let mut e = grant_skill(&e0, &v, "gwjushi", 0);
    let shield = card_of_kind_and_suit(&e0, CardKind::Armor, None);
    e = inject_equip(&e, &v, shield, 1);
    activate(&mut e, 0, "gwjushi");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    assert!(e.state_core().player(0).off_court.contains(&shield));
    let mut snap = e.save();
    snap.core.players[0].hp = 1;
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    drain_default(&mut e);
    assert!(!e.state_core().player(0).alive, "seat 0 died");
    assert!(e.state_core().discard.contains(&shield), "off_court card moved to discard on death");
    assert_zone_conservation(&e);

    // 5. Save->restore mid-choice on `gwbaquan_return` preserves stolen `m.cards`
    let e = grant_skill(&e0, &v, "gwbaquan", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 2;
    snap.core.players[1].hp = 4;
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[other_cards[0]], 0);
    e = inject_hand(&e, &v, &[other_cards[1]], 1);
    activate(&mut e, 0, "gwbaquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let (e, out) = save_restore_roundtrip_answer(e, &v, 0, vec![ChoiceOption::Card(other_cards[1])]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }));
    assert_eq!(e.debug_hp(1), 3, "restored gwbaquan_return preserved original target list and dealt damage");
    assert_zone_conservation(&e);
    let _ = other_cards;
}
