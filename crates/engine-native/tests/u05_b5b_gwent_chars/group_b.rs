//! U05-5b Group B (19 skills: `shuangxi`, `yangfan`, `gwfengshi`, `gwchenshui`, `gwliedi`,
//! `julian`, `gwfusheng`, `gwqinwu`, `gwqinwu2`, `huanshu`, `huanshu2`, `huanshu3`, `gwjieyin`,
//! `zhengjun`, `gwxuezhan`, `bolang`, `gwjushi`, `gwjushi2`, `gwfengchi`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b5b_shuangxi_yangfan_gwfengshi_julian_gwxuezhan() {
    let (v, e0) = setup_gwent_locked(5610);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Spade));
    let club_shan = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Club));

    // 1. shuangxi (`character/gwent/skill.js:1134-1180`, round: 2): no damage -> 1 target ciguhanshuang;
    // dealt damage -> 1..=3 baishuang; cooldown 2 rounds
    let mut e = grant_skill(&e0, &v, "shuangxi", 0);
    activate(&mut e, 0, "shuangxi");
    let ch = e.open_choice().cloned().expect("shuangxi targets");
    assert_eq!(ch.spec.max, 1, "without dealt damage shuangxi acts as single-target ciguhanshuang");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Player(1)] });
    assert!(has_skill(&e, 1, "gw_cigu_state") || has_skill(&e, 1, "gw_ciguhanshuang"));
    // Cooldown across next round (round+1 rejected, round+2 allowed)
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    assert!(matches!(
        activate(&mut e, 0, "shuangxi"),
        CommandOutcome::Rejected(
            RejectReason::SkillAlreadyUsed
                | RejectReason::WrongPhase
                | RejectReason::SkillDisabled
                | RejectReason::InvalidChoice
        )
    ));
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![3] });
    pass_window(&mut e);
    activate(&mut e, 0, "shuangxi");
    let ch2 = e.open_choice().cloned().expect("shuangxi baishuang after dealing damage");
    assert_eq!(ch2.spec.max, 3, "after dealing damage shuangxi lets choose up to 3 for gw_baishuang");
    submit(&mut e, 0, GameAction::Answer { choice_id: ch2.id, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(3)] });
    assert!(has_skill(&e, 3, "gw_cigu_state") || has_skill(&e, 3, "gw_ciguhanshuang"));

    // 2. yangfan (`character/gwent/skill.js:1181-1200`): using non-equip recasts 1 same-suit/same-color hand card
    let mut e = grant_skill(&e0, &v, "yangfan", 0);
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, Some(engine_native::catalog::Suit::Spade));
    let second_spade = find_cards_by(&e0, |d| d.suit == engine_native::catalog::Suit::Spade && d.kind == CardKind::Sha, 2)[0];
    e = inject_hand(&e, &v, &[wuzhong, second_spade, club_shan], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: vec![] });
    pass_window(&mut e);
    assert!(!e.debug_hand(0).contains(&second_spade), "yangfan prefers same suit recast");

    // 3. gwfengshi (`character/gwent/skill.js:1201-1242`): phaseEnd choice 0=debuff / 1=buff / cancel
    let mut e = grant_skill(&e0, &v, "gwfengshi", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 2;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    for _ in 0..8 {
        if e.turn().current != 0 || e.open_choice().is_some() {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    let c = e.open_choice().cloned().expect("gwfengshi choice at TurnPhaseEnd");
    submit(&mut e, 0, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Tag(1)] });
    advance_past_turn(&mut e, 0);
    assert!(e.debug_hand(0).len() + e.debug_hp(0) as usize + e.state_core().player(0).hujia as usize > 2);

    // 4. julian (`character/gwent/skill.js:1363-1381`): phaseUseBegin draw up to max other hand
    let mut e = grant_skill(&e0, &v, "julian", 0);
    let h3 = find_cards_by(&e0, |_| true, 3);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 1);
    e = clear_hand(&e, &v, 2);
    e = clear_hand(&e, &v, 3);
    e = inject_hand(&e, &v, &h3, 1);
    advance_to_seat_play(&mut e, 0);
    assert_eq!(e.debug_hand(0).len(), 3, "julian draws up to other max hand (3)");

    // 5. gwxuezhan (`character/gwent/skill.js:1701-1710`): phaseBegin when min handcard -> gain gw_shizizhaohuan
    let mut e = grant_skill(&e0, &v, "gwxuezhan", 0);
    e = inject_hand(&e, &v, &[sha], 1);
    e = inject_hand(&e, &v, &[club_shan], 2);
    e = inject_hand(&e, &v, &[wuzhong], 3);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    assert!(e.debug_hand(0).iter().any(|&c| e.catalog().cards[c as usize].name == "gw_shizizhaohuan"));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwchenshui_awakens_gwliedi_and_sleeps_back() {
    // Upstream: `character/gwent/skill.js:1243-1362`
    let (v, e0) = setup_gwent_locked(5611);
    let mut e = grant_skill(&e0, &v, "gwchenshui", 0);
    let shas = find_cards_by(&e0, |d| d.kind == CardKind::Sha, 4);
    e = inject_hand(&e, &v, &[shas[0], shas[1], shas[2]], 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 4;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();

    // Seat 1 attacks seat 0 once (#1), seat 0 attacks seat 1 (#2), seat 3 attacks seat 0 (#3)
    e = inject_hand(&e, &v, &[shas[3]], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: shas[3], targets: vec![0] });
    pass_window(&mut e);
    assert_eq!(e.debug_hp(0), 4, "gwchenshui cancels damage");
    // Give 2 more prevented damages using 1->3->0 aoe or 0's own turn aoe/nanman
    let nanman = card_of_kind_and_suit(&e0, CardKind::SavageAttack, None);
    advance_to_seat_play(&mut e, 0);
    e = inject_hand(&e, &v, &[nanman], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: nanman, targets: vec![] });
    drain_default(&mut e); // cancels damage to 1, 2, 3 -> total prevented >= 3!
    advance_past_turn(&mut e, 0); // phaseEnd -> awakening damages all enemies & swaps to gwliedi
    assert!(!has_skill(&e, 0, "gwchenshui"));
    assert!(has_skill(&e, 0, "gwliedi"));
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.debug_hp(3), 3);

    // gwliedi: distance >= 2 to seat 3 (dist = 1 normally, let's inject +1 horse on seat 3 so dist = 2 -> +1 damage)
    let plus_horse = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::HorsePlus), 1)[0];
    e = inject_equip(&e, &v, plus_horse, 3);
    advance_to_seat_play(&mut e, 0);
    let duel = card_of_kind_and_suit(&e0, CardKind::Duel, None);
    e = inject_hand(&e, &v, &[duel], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![3] });
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 1, "gwliedi adds floor(2/2)=+1 damage to distance 2 enemy (3->1)");

    // Two turns without dealing damage -> gwliedi reverts to gwchenshui
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    advance_past_turn(&mut e, 0);
    assert!(has_skill(&e, 0, "gwchenshui") && !has_skill(&e, 0, "gwliedi"));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwfusheng_gwqinwu_huanshu_gwjieyin_zhengjun_bolang_gwjushi_gwfengchi() {
    let (v, e0) = setup_gwent_locked(5612);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Spade));
    let red_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Heart));
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, Some(engine_native::catalog::Suit::Heart));

    // 1. gwfusheng (`character/gwent/skill.js:1382-1413`): dying save when !face_down -> turn over, recover 1, both draw 1 (self draws 2)
    let mut e = grant_skill(&e0, &v, "gwfusheng", 0);
    let mut snap = e.save();
    snap.core.players[1].hp = 1;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    let d_opt = e.open_choice().cloned().expect("gwfusheng dying choice");
    let sk_def = skill_id(&e, "gwfusheng");
    assert!(d_opt.spec.options.contains(&ChoiceOption::Skill(sk_def)));
    submit(&mut e, 0, GameAction::Answer { choice_id: d_opt.id, selection: vec![ChoiceOption::Skill(sk_def)] });
    assert_eq!(e.debug_hp(1), 1);
    assert!(e.state_core().player(1).face_down);
    assert_eq!(e.debug_hand(0).len(), 1);
    assert_eq!(e.debug_hand(1).len(), 1);

    // 2. gwqinwu (`character/gwent/skill.js:1414-1460`): basic card -> target draws 1 & temp gwqinwu+gwqinwu2
    let mut e = grant_skill(&e0, &v, "gwqinwu", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(2));
    assert_eq!(e.debug_hand(2).len(), 1);
    assert!(has_skill(&e, 2, "gwqinwu") && has_skill(&e, 2, "gwqinwu2"));

    // 3. huanshu / huanshu2 / huanshu3 (`character/gwent/skill.js:1461-1529`): store red card; cancel enemy red trick + draw 2; else phaseBegin discards
    let mut e = grant_skill(&e0, &v, "huanshu", 0);
    e = inject_hand(&e, &v, &[wuzhong], 0);
    for _ in 0..8 {
        if e.turn().current != 0 || e.open_choice().is_some() {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(wuzhong));
    assert!(has_skill(&e, 0, "huanshu2"));
    e = inject_hand(&e, &v, &[wuzhong], 1); // enemy seat 1 uses red trick -> canceled, seat 0 draws 2, huanshu2 removed
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: vec![] });
    drain_default(&mut e);
    assert!(!has_skill(&e, 0, "huanshu2"));
    assert_eq!(e.debug_hand(0).len(), 2, "huanshu2 drew 2");
    assert!(!e.debug_hand(1).contains(&wuzhong) && e.debug_hand(1).len() == 2, "enemy red trick was canceled by huanshu2");

    // 4. gwjieyin (`character/gwent/skill.js:1530-1635`): Play virtual gw_yanziyaoshui / gw_kunenfayin / gw_wenyi, reset every 2 rounds
    let mut e = grant_skill(&e0, &v, "gwjieyin", 0);
    activate(&mut e, 0, "gwjieyin");
    let jy = e.open_choice().cloned().expect("gwjieyin pick");
    assert_eq!(jy.spec.options.len(), 3);
    submit(&mut e, 0, GameAction::Answer { choice_id: jy.id, selection: vec![ChoiceOption::Tag(1)] }); // yanziyaoshui
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(0));
    assert!(!e.debug_hand(0).is_empty(), "gwjieyin virtual yanziyaoshui drew cards");

    // 5. zhengjun (`character/gwent/skill.js:1636-1700`): 2nd use/respond of same card name -> +1 maxHp, recover 1, phaseEnd draw/pick
    let mut e = grant_skill(&e0, &v, "zhengjun", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 3;
    snap.core.players[0].max_hp = 4;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[sha, red_sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    submit(&mut e, 0, GameAction::PlayCard { card: red_sha, targets: vec![3] });
    pass_window(&mut e);
    // zhengjun might need unlimited sha or across turns -> use sha on next turn if limited! Let's also allow response/next turn:
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    if e.debug_hand(0).contains(&red_sha) {
        submit(&mut e, 0, GameAction::PlayCard { card: red_sha, targets: vec![3] });
        pass_window(&mut e);
    }
    assert!(e.state_core().player(0).max_hp >= 5 && e.debug_hp(0) >= 4, "zhengjun raised maxHp & recovered");

    // 6. bolang (`character/gwent/skill.js:1711-1774`): phaseBegin discard 1..=3 of top 6; damageEnd retrieve 1
    let mut e = grant_skill(&e0, &v, "bolang", 0);
    let tops = find_cards_by(&e, |d| d.kind != CardKind::Sha, 3);
    advance_to_seat_play(&mut e, 3);
    e = set_deck_top(&e, &v, &tops);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(tops[0]));
    advance_to_seat_play(&mut e, 0);
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(tops[0]));
    assert!(e.debug_hand(0).contains(&tops[0]), "bolang reclaimed discarded card on damageEnd");

    // 7. gwjushi (`character/gwent/skill.js:1775-1834`): hold neighbor equip -> damage returns it equipped; else phaseBegin gains
    let mut e = grant_skill(&e0, &v, "gwjushi", 0);
    let shield = card_of_kind_and_suit(&e0, CardKind::Armor, None);
    e = inject_equip(&e, &v, shield, 1);
    activate(&mut e, 0, "gwjushi");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    assert!(e.state_core().player(1).equips.is_empty());
    assert!(has_skill(&e, 0, "gwjushi2"));
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    assert!(e.state_core().player(1).equips.contains(&shield), "gwjushi2 returned equip onto seat 1");
    assert!(!has_skill(&e, 0, "gwjushi2"));

    // 8. gwfengchi (`character/gwent/skill.js:1835-1894`): phaseUseBegin grants 1 of 3 phaseUse skills until phaseUseEnd
    let mut e = grant_skill(&e0, &v, "gwfengchi", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_play_or_choice(&mut e, 0);
    let ch = e.open_choice().cloned().expect("gwfengchi pick skill");
    let ChoiceOption::Skill(picked_def) = ch.spec.options[0] else {
        panic!("expected ChoiceOption::Skill");
    };
    submit(&mut e, 0, GameAction::Answer { choice_id: ch.id, selection: vec![ChoiceOption::Skill(picked_def)] });
    let picked_name = e.catalog().skill_name(picked_def);
    assert!(has_skill(&e, 0, picked_name));
    submit(&mut e, 0, GameAction::NextPhase);
    drain_default(&mut e);
    assert!(!has_skill(&e, 0, picked_name), "gwfengchi temp skill expires on Play end");
    assert_zone_conservation(&e);
}
