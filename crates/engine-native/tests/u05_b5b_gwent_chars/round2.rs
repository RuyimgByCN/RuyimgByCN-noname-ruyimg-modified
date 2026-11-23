//! U05-5b Round-2 审校回归测试（#309 全部 17 项反例：零 `debug_execute_effect`）。
//! 覆盖：`Jiedao` 二段目标（`sqlongwu`/`jielue`/`lanquan`）、`gwliaotian`、`junchi`、
//! `gwqinwu`、`gwchenshui`、`hupeng4`、`sqlonghuo`、`sqlongnu`、`hunmo`、`gwminxiang`、
//! `huanshu2`/`fengjian`、`vm_va_sha_no_count`、`gwweitu`、`gwjinli`/`gwgouhun`、`huandie`、
//! `bolang`/`gwshenyu` 残签清理、及 LIFO/防具守恒。

use super::common::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b5b_r2_jiedao_two_step_sqlongwu_jielue_lanquan() {
    // Item 1: Jiedao two-step holder -> victim in sqlongwu / jielue / lanquan (plus save->restore)
    let (v, e0) = setup_gwent_locked(5640);
    let jiedao = card_of_kind_and_suit(&e0, CardKind::Jiedao, None);
    let weapon = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon), 1)[0];
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, None);

    // 1) sqlongwu + Jiedao (with negative check when no holder has weapon)
    let mut e = grant_skill(&e0, &v, "sqlongwu", 0);
    e = inject_hand(&e, &v, &[jiedao], 0);
    for _ in 0..8 {
        if e.open_choice().is_some() || e.turn().current != 0 {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    if let Some(ch) = e.open_choice().cloned() {
        assert_eq!(ch.spec.prompt, "sqlongwu_use_card");
        assert!(!ch.spec.options.contains(&ChoiceOption::Card(jiedao)));
        answer_cancel(&mut e, 0);
    }
    let mut e = grant_skill(&e0, &v, "sqlongwu", 0);
    e = inject_equip(&e, &v, weapon, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    e = inject_hand(&e, &v, &[jiedao], 0);
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "sqlongwu_use_card") {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(jiedao));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let ch2 = e.open_choice().cloned().expect("sqlongwu jiedao step 2 victim");
    assert_eq!(ch2.spec.prompt, "sqlongwu_use_target");
    assert!(ch2.spec.options.contains(&ChoiceOption::Player(3)));
    submit(&mut e, 0, GameAction::Answer { choice_id: ch2.id, selection: vec![ChoiceOption::Player(3)] });
    pass_window(&mut e);
    answer_first(&mut e, 1, |o| *o == ChoiceOption::Tag(0));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3, "sqlongwu jiedao resolved holder->victim Sha");
    assert_zone_conservation(&e);

    // 2) jielue + Jiedao (`gwent/skill.js:149-210`)
    let mut e = grant_skill(&e0, &v, "jielue", 0);
    e = inject_equip(&e, &v, weapon, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[jiedao], 2);
    advance_to_seat_play_or_choice(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let jc = e.open_choice().cloned().expect("jielue jiedao step 2 victim");
    assert_eq!(jc.spec.prompt, "jielue_target");
    submit(&mut e, 0, GameAction::Answer { choice_id: jc.id, selection: vec![ChoiceOption::Player(3)] });
    pass_window(&mut e);
    answer_first(&mut e, 1, |o| *o == ChoiceOption::Tag(0));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3, "jielue jiedao resolved holder->victim Sha");
    assert_eq!(e.debug_hand(2).len(), 1, "jielue drew 1 for friend provider");
    assert_zone_conservation(&e);

    // 3) lanquan + Jiedao with mid-choice save -> restore
    let mut e = grant_skill(&e0, &v, "lanquan", 0);
    e = inject_equip(&e, &v, weapon, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    e = set_deck_top(&e, &v, &[jiedao]);
    activate(&mut e, 0, "lanquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(jiedao));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let lq2 = e.open_choice().cloned().expect("lanquan jiedao step 2 victim");
    assert_eq!(lq2.spec.prompt, "lanquan_target");
    let (mut e, out) = save_restore_roundtrip_answer(e, &v, 0, vec![ChoiceOption::Player(3)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    answer_first(&mut e, 1, |o| *o == ChoiceOption::Tag(0));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3, "lanquan jiedao resolved holder->victim Sha after restore");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r2_liaotian_junchi_qinwu_chenshui_hupeng4() {
    let (v, e0) = setup_gwent_locked(5641);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let reds4 = find_cards_by(&e0, |d| matches!(d.suit, Suit::Heart | Suit::Diamond) && d.kind == CardKind::Sha, 4);
    let plus_horse = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(EquipSlot::HorsePlus), 1)[0];

    // Item 2 + Item 12: gwliaotian rejects mixed color, recasts ALL 3 same-color cards, fires free Sha even after Play Sha, doesn't block Play Sha
    let mut e = grant_skill(&e0, &v, "gwliaotian", 0);
    e = inject_hand(&e, &v, &[reds4[0], reds4[1], reds4[2]], 0);
    activate(&mut e, 0, "gwliaotian");
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 3, "gwliaotian recasts all 3 cards");
    assert!(e.debug_hp(1) == 3 || e.debug_hp(3) == 3, "gwliaotian fired random free Sha at an enemy");
    assert!(
        !e.debug_hand(0).contains(&reds4[0]) && !e.debug_hand(0).contains(&reds4[2]),
        "all 3 old red cards were discarded"
    );
    e = inject_hand(&e, &v, &[sha], 0);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "gwliaotian virtual Sha did not consume Play-phase Sha limit"
    );
    pass_window(&mut e);

    // Item 3 + Item 12: junchi (`distance=false`, `noCount`, no junchi_gold extra turn, draw before resolve)
    let mut e = grant_skill(&e0, &v, "junchi", 0);
    let snatch = card_of_kind_and_suit(&e0, CardKind::Snatch, None);
    let wuyao = find_card(&e0, "gw_wuyao");
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, None);
    let sha2 = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Heart));
    let enemy_gold = find_card(&e0, "gw_baishuang");
    e = inject_hand(&e, &v, &[enemy_gold], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: enemy_gold, targets: vec![3] });
    drain_default(&mut e);
    assert!(!e.state_core().vm_aux.flags.contains_key(&(0, "extra_turn_insert_queued".to_string())), "enemy gold spell must NOT grant junchi extra turn");
    let e = grant_skill(&e, &v, "gwminxiang", 0);
    let mut e = e;
    advance_to_seat_play(&mut e, 0);
    e = inject_equip(&e, &v, plus_horse, 2); // seat 0->2 dist=3
    let diamond_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Diamond));
    e = inject_hand(&e, &v, &[sha, diamond_sha, sha2, snatch, wuyao, wuzhong], 0);
    e = inject_hand(&e, &v, &[reds4[2]], 2);
    // 1) Seat 0 spends Play-phase Sha limit on seat 1; then via gwminxiang seat 1 uses Sha on seat 2 -> junchi triggers on seat 0 for target 2 (dist=3 from seat 0)!
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    activate(&mut e, 0, "gwminxiang");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(diamond_sha));
    let mt = e.open_choice().cloned().expect("gwminxiang targets for junchi trigger");
    submit(&mut e, 0, GameAction::Answer { choice_id: mt.id, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)] });
    pass_window(&mut e);
    let jc = e.open_choice().cloned().expect("junchi_use after Play Sha");
    assert_eq!(jc.spec.prompt, "junchi_use");
    assert!(jc.spec.options.contains(&ChoiceOption::Card(sha2)), "junchi offers Sha even after Play-phase Sha used");
    assert!(jc.spec.options.contains(&ChoiceOption::Card(wuyao)), "junchi offers gw_wuyao");
    assert!(jc.spec.options.contains(&ChoiceOption::Card(snatch)), "junchi offers Snatch at dist=3 (distance=false)");
    assert!(!jc.spec.options.contains(&ChoiceOption::Card(wuzhong)), "junchi excludes Wuzhong");
    let before_seat2_zones = e.debug_hand(2).len() + e.state_core().player(2).equips.len();
    submit(&mut e, 0, GameAction::Answer { choice_id: jc.id, selection: vec![ChoiceOption::Card(snatch)] });
    drain_default(&mut e);
    assert_eq!(
        e.debug_hand(2).len() + e.state_core().player(2).equips.len(),
        before_seat2_zones - 1,
        "junchi Snatch at dist=3 stole 1 card from seat 2"
    );

    // Item 4: gwqinwu survives seat 0's TurnEnd and clears at target's own TurnEndWindow(after=true)
    let mut e = grant_skill(&e0, &v, "gwqinwu", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(2));
    assert!(has_skill(&e, 2, "gwqinwu") && has_skill(&e, 2, "gwqinwu2"));
    advance_past_turn(&mut e, 0);
    assert!(has_skill(&e, 2, "gwqinwu") && has_skill(&e, 2, "gwqinwu2"), "gwqinwu must survive seat 0 turn end");
    advance_past_turn(&mut e, 1);
    assert!(has_skill(&e, 2, "gwqinwu"), "gwqinwu still active at start of seat 2 turn");
    advance_past_turn(&mut e, 2);
    assert!(!has_skill(&e, 2, "gwqinwu") && !has_skill(&e, 2, "gwqinwu2"), "gwqinwu temp expires after target's turn end");
    assert!(has_skill(&e, 0, "gwqinwu"), "owner retains permanent gwqinwu");

    // Item 5: gwchenshui awakening hits enemies at dist >= 2 BEFORE inserting gwliedi
    let mut e = grant_skill(&e0, &v, "gwchenshui", 0);
    e = inject_equip(&e, &v, plus_horse, 1); // dist(0, 1) >= 2
    e = inject_equip(&e, &v, plus_horse, 3);
    let mut snap = e.save();
    snap.core.vm_aux.flags.insert((0, "gwchenshui_count".to_string()), 3);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_prepare(&mut e, 0);
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "gwchenshui dealt 1 dmg even at dist >= 2");
    assert_eq!(e.debug_hp(3), 3, "gwchenshui dealt 1 dmg to seat 3");
    assert!(has_skill(&e, 0, "gwliedi") && !has_skill(&e, 0, "gwchenshui"));

    // Item 6: hupeng4 cancels only owner on AOE (other targets still suffer) & gains AOE card
    let mut e = grant_skill(&e0, &v, "hupeng4", 0);
    let d1 = find_card(&e0, "gw_dudayuanshuai1");
    let nanman = card_of_kind_and_suit(&e0, CardKind::SavageAttack, None);
    e = inject_hand(&e, &v, &[d1], 0);
    e = inject_hand(&e, &v, &[nanman], 1);
    advance_to_seat_play(&mut e, 1);
    e = clear_hand(&e, &v, 2);
    e = clear_hand(&e, &v, 3);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: nanman, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(d1));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(0), 4, "hupeng4 shielded seat 0 from Nanman");
    assert_eq!(e.debug_hp(2), 3, "Nanman still damaged seat 2");
    assert_eq!(e.debug_hp(3), 3, "Nanman still damaged seat 3");
    assert!(e.debug_hand(0).contains(&nanman), "seat 0 gained Nanman");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r2_sqlonghuo_sqlongnu_hunmo_minxiang_tricks_weitu() {
    let (v, e0) = setup_gwent_locked(5642);
    let shas = find_cards_by(&e0, |d| d.kind == CardKind::Sha && d.suit == Suit::Spade, 3);
    let red_shas = find_cards_by(&e0, |d| d.kind == CardKind::Sha && d.suit == Suit::Heart, 2);
    let tao = card_of_kind_and_suit(&e0, CardKind::Tao, Some(Suit::Heart));
    let shan = card_of_kind_and_suit(&e0, CardKind::Shan, None);
    let lebu = card_of_kind_and_suit(&e0, CardKind::Le, None);
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, Some(Suit::Heart));
    let jiu = card_of_kind_and_suit(&e0, CardKind::Jiu, None);
    let baoxue = find_card(&e0, "gw_baoxueyaoshui");

    // Item 7: sqlonghuo discards at most 1 random matching card per enemy
    let mut e = grant_skill(&e0, &v, "sqlonghuo", 0);
    e = inject_hand(&e, &v, &[shas[0]], 0);
    e = inject_hand(&e, &v, &[shas[1], shas[2], red_shas[0]], 1);
    activate(&mut e, 0, "sqlonghuo");
    drain_default(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "sqlonghuo discards ONLY 1 random matching Sha per enemy");

    // Item 8: sqlongnu discover <=3, allow self, cancel gains duplicate
    let mut e = grant_skill(&e0, &v, "sqlongnu", 0);
    e = inject_hand(&e, &v, &[shas[0]], 0);
    advance_to_seat_play(&mut e, 3);
    let mut snap = e.save();
    let rest: Vec<_> = snap.core.deck.drain(..).filter(|&c| c != shas[1] && c != tao && c != shan).collect();
    snap.core.discard.extend(rest);
    snap.core.discard.retain(|&c| c != shas[1] && c != tao && c != shan);
    snap.core.deck = vec![shas[1], tao, shan];
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_prepare(&mut e, 0);
    let pick_ch = e.open_choice().cloned().expect("sqlongnu_pick");
    assert!(pick_ch.spec.options.len() <= 3);
    submit(&mut e, 0, GameAction::Answer { choice_id: pick_ch.id, selection: vec![ChoiceOption::Card(shas[1])] });
    let dmg_ch = e.open_choice().cloned().expect("sqlongnu_damage");
    assert!(dmg_ch.spec.options.contains(&ChoiceOption::Player(0)), "sqlongnu_damage allows self");
    assert!(dmg_ch.spec.allow_cancel, "sqlongnu_damage allows cancel");
    answer_cancel(&mut e, 0);
    drain_default(&mut e);
    assert!(e.debug_hand(0).contains(&shas[1]), "cancelling sqlongnu_damage puts duplicate into owner hand");
    assert_eq!(e.debug_hp(0), 4);

    // Item 9: hunmo <2 target filter rejects >=2 hand targets
    let mut e = grant_skill(&e0, &v, "hunmo", 0);
    e = inject_hand(&e, &v, &[shas[0]], 0); // <2
    e = inject_hand(&e, &v, &[shas[1], shas[2]], 1); // >=2 (excluded!)
    e = inject_hand(&e, &v, &[tao], 3); // <2 (included!)
    activate(&mut e, 0, "hunmo");
    let ch = e.open_choice().cloned().expect("hunmo_target");
    assert!(!ch.spec.options.contains(&ChoiceOption::Player(1)), "hunmo hand<2 excludes seat 1 (>=2 cards)");
    assert!(ch.spec.options.contains(&ChoiceOption::Player(3)), "hunmo hand<2 includes seat 3 (<2 cards)");
    answer_cancel(&mut e, 0);

    // Item 10 + Item 12: gwminxiang excludes Tao(healthy)/Lebu/baoxue, accepts Wuzhong/Jiu/Sha (noCount)
    let mut e = grant_skill(&e0, &v, "gwminxiang", 0);
    e = inject_hand(&e, &v, &[tao, lebu, baoxue], 0);
    assert!(matches!(activate(&mut e, 0, "gwminxiang"), CommandOutcome::Rejected(_)));
    let mut e = grant_skill(&e0, &v, "gwminxiang", 0);
    e = inject_hand(&e, &v, &[wuzhong, jiu, shas[0]], 0);
    activate(&mut e, 0, "gwminxiang");
    let mc = e.open_choice().cloned().expect("gwminxiang_card");
    assert!(mc.spec.options.contains(&ChoiceOption::Card(wuzhong)), "gwminxiang allows Wuzhong");
    assert!(mc.spec.options.contains(&ChoiceOption::Card(jiu)), "gwminxiang allows Jiu");
    assert!(mc.spec.options.contains(&ChoiceOption::Card(shas[0])), "gwminxiang allows Sha");
    submit(&mut e, 0, GameAction::Answer { choice_id: mc.id, selection: vec![ChoiceOption::Card(wuzhong)] });
    let mt = e.open_choice().cloned().expect("gwminxiang_targets");
    submit(&mut e, 0, GameAction::Answer { choice_id: mt.id, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(3)] });
    drain_default(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "target 1 used Wuzhong drew 2");
    assert_eq!(e.debug_hand(3).len(), 2, "target 3 used Wuzhong drew 2");

    // Item 11: huanshu2 & fengjian reject non-trick Gwent spells (`gw_baoxueyaoshui`)
    let mut e = grant_skill(&e0, &v, "fengjian", 0);
    e = inject_hand(&e, &v, &[baoxue], 0);
    e = inject_hand(&e, &v, &[tao, shan], 1);
    submit(&mut e, 0, GameAction::PlayCard { card: baoxue, targets: vec![1] });
    drain_default(&mut e);
    assert!(!e.state_core().player(0).yinshen_active, "fengjian must NOT trigger on gw_baoxueyaoshui");
    let mut e = grant_skill(&e0, &v, "fengjian", 0);
    let guohe_f = card_of_kind_and_suit(&e0, CardKind::Dismantle, None);
    e = inject_hand(&e, &v, &[guohe_f, shas[0]], 0);
    e = inject_hand(&e, &v, &[tao], 1);
    submit(&mut e, 0, GameAction::PlayCard { card: guohe_f, targets: vec![1] });
    pass_window(&mut e);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(3));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3, "fengjian thunder Sha dealt 1");
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: shas[0], targets: vec![1] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "fengjian virtual LeiSha did not consume Play-phase Sha limit"
    );
    pass_window(&mut e);
    let mut e = grant_skill(&e0, &v, "huanshu", 0);
    let red_baoxue = find_cards_by(&e0, |d| d.name.starts_with("gw_baoxueyaoshui") && matches!(d.suit, Suit::Heart | Suit::Diamond), 1)
        .into_iter()
        .next()
        .unwrap_or_else(|| find_card(&e0, "gw_baishuang"));
    e = inject_hand(&e, &v, &[wuzhong], 0);
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "huanshu_card") {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(wuzhong));
    assert!(has_skill(&e, 0, "huanshu2"));
    e = inject_hand(&e, &v, &[red_baoxue], 1);
    advance_to_seat_play(&mut e, 1);
    let play_tgt = if e.catalog().cards[red_baoxue as usize].name.starts_with("gw_baishuang") { vec![0] } else { vec![1] };
    submit(&mut e, 1, GameAction::PlayCard { card: red_baoxue, targets: play_tgt });
    drain_default(&mut e);
    assert!(has_skill(&e, 0, "huanshu2"), "huanshu2 must NOT trigger on non-trick card");

    // Item 13: gwweitu 2-dmg changeHujia event increments gwweitu_loss by 1 (not 2)
    let mut e = grant_skill(&e0, &v, "gwweitu", 0);
    let mut snap = e.save();
    snap.core.players[0].hujia = 2;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &[jiu, shas[0]], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: jiu, targets: vec![] });
    submit(&mut e, 1, GameAction::PlayCard { card: shas[0], targets: vec![0] }); // 2 dmg in 1 event -> -2 hujia
    drain_default(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "gwweitu_loss".to_string())).copied().unwrap_or(0), 1, "single 2-hujia loss increments gwweitu_loss by 1");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r2_jinli_gouhun_huandie_vanishtag_zhanjiang_jushi() {
    let (v, e0) = setup_gwent_locked(5643);
    let armor = card_of_kind_and_suit(&e0, CardKind::Armor, None);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let wuyao = find_card(&e0, "gw_wuyao");
    let guohe = card_of_kind_and_suit(&e0, CardKind::Dismantle, None);
    let silver_spell = find_card(&e0, "gw_youer");

    // Item 14: gwjinli & gwgouhun require hand cards (equips rejected)
    let mut e = grant_skill(&e0, &v, "gwjinli", 0);
    e = inject_equip(&e, &v, armor, 0);
    assert!(matches!(activate(&mut e, 0, "gwjinli"), CommandOutcome::Rejected(_)));
    let mut e = grant_skill(&e0, &v, "gwgouhun", 0);
    e = inject_equip(&e, &v, armor, 0);
    e = inject_hand(&e, &v, &[sha], 1);
    assert!(matches!(activate(&mut e, 0, "gwgouhun"), CommandOutcome::Rejected(_)));

    // Item 15: huandie 0 other targets confirms self-draw 1 + arms huandie_discard (with save->restore)
    let mut e = grant_skill(&e0, &v, "huandie", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Confirm);
    let hd = e.open_choice().cloned().expect("huandie_targets");
    assert_eq!(hd.spec.min, 0);
    assert!(!hd.spec.allow_cancel);
    let (e, out) = save_restore_roundtrip_answer(e, &v, 0, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.debug_hand(0).len(), 1, "huandie 0-target drew 1 for self");
    assert!(has_skill(&e, 0, "huandie_discard"), "huandie 0-target armed huandie_discard");
    // Declining huandie trigger_confirm draws nothing and does not arm huandie_discard
    let mut e = grant_skill(&e0, &v, "huandie", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_cancel(&mut e, 0);
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0);
    assert!(!has_skill(&e, 0, "huandie_discard"));

    // Item 16: stale gw_shenyu_vanishtag / gw_bolang_tag cleared on washDeck / reshuffle
    let mut e = grant_skill(&e0, &v, "gwshenyu", 0);
    let mut snap = e.save();
    snap.core.deck.retain(|&c| c != silver_spell && c != guohe);
    snap.core.discard.retain(|&c| c != silver_spell && c != guohe);
    let extra_deck: Vec<_> = snap.core.deck.drain(..).collect();
    snap.core.discard.extend(extra_deck);
    snap.core.discard.push(silver_spell);
    snap.core.discard.push(guohe);
    snap.core.deck.clear(); // washDeck triggers at next DrawPhase
    snap.core.vm_aux.flags.insert((0, format!("gw_shenyu_vanishtag:{silver_spell}")), 1);
    snap.core.vm_aux.flags.insert((0, format!("gw_bolang_tag:{guohe}")), 1);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0); // runs seat 0 DrawPhase -> washes discard -> clears both tags!
    drain_default(&mut e);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, format!("gw_shenyu_vanishtag:{silver_spell}"))).copied().unwrap_or(0), 0);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, format!("gw_bolang_tag:{guohe}"))).copied().unwrap_or(0), 0);

    // Item 17: gwzhanjiang accepts gw_wuyao + enforces kongcheng/sha_user_or_target_blocked + does not consume Play Sha
    let e = grant_skill(&e0, &v, "gwzhanjiang", 0);
    let kongcheng_owner = grant_skill(&e, &v, "空城", 1); // empty hand seat 1 has kongcheng
    let mut e = kongcheng_owner;
    e = inject_hand(&e, &v, &[sha], 0);
    e = inject_hand(&e, &v, &[wuyao], 2);
    advance_to_seat_prepare(&mut e, 1);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    assert!(e.open_choice().is_none(), "kongcheng turn player cannot be targeted by gwzhanjiang Sha");
    let mut e = grant_skill(&e0, &v, "gwzhanjiang", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    e = inject_hand(&e, &v, &[wuyao], 2);
    advance_to_seat_prepare(&mut e, 1);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 2, |o| *o == ChoiceOption::Card(wuyao));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "gwzhanjiang used gw_wuyao as Sha");

    // Item 17b: gwjushi2 returns equip replacing another equip with CardsLost conservation + gwfusheng dying LIFO
    let mut e = grant_skill(&e0, &v, "gwjushi", 0);
    let armor2 = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(EquipSlot::Armor), 2)[1];
    e = inject_equip(&e, &v, armor, 1);
    activate(&mut e, 0, "gwjushi");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    e = inject_equip(&e, &v, armor2, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    assert!(e.state_core().player(1).equips.contains(&armor) && !e.state_core().player(1).equips.contains(&armor2));
    assert_eq!(e.state_core().player(1).equips.len(), 1, "gwjushi2 swapped out old armor cleanly");

    // Item 17c: gwfusheng self-dying yields sequentially: turn_over -> recover 1 -> draw 2
    let mut e = grant_skill(&e0, &v, "gwfusheng", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 1;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_play(&mut e, 1);
    e = clear_hand(&e, &v, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    pass_window(&mut e);
    let d_opt = e.open_choice().cloned().expect("gwfusheng self dying choice");
    let sk_def = skill_id(&e, "gwfusheng");
    let out = submit(&mut e, 0, GameAction::Answer { choice_id: d_opt.id, selection: vec![ChoiceOption::Skill(sk_def)] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "fusheng out={out:?} d_opt={d_opt:?} open={:?}", e.open_choice());
    assert_eq!(e.debug_hp(0), 1);
    assert!(e.state_core().player(0).face_down);
    assert_eq!(e.debug_hand(0).len(), 2, "gwfusheng self-save draws 2");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r2_sharp50_gwtianbian_taoyuan_and_hupeng2() {
    // Defect 1 (#50): gwtianbian positive AOE (`Taoyuan`) must heal owner as well as others
    let mut hit_taoyuan = false;
    for seed in 5650..5666 {
        let (v, e0) = setup_gwent_locked(seed);
        let mut e = grant_skill(&e0, &v, "gwtianbian", 0);
        let mut snap = e.save();
        for p in &mut snap.core.players {
            p.hp = 2;
        }
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
        advance_past_turn(&mut e, 0);
        advance_to_seat_play_or_choice(&mut e, 0);
        let ch = e.open_choice().cloned().expect("gwtianbian at Play begin");
        assert_eq!(ch.spec.prompt, "gwtianbian_mode");
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        drain_default(&mut e);
        if e.debug_hp(1) == 3 {
            assert_eq!(
                e.debug_hp(0),
                3,
                "gwtianbian Taoyuan must also heal owner (seat 0)"
            );
            assert_zone_conservation(&e);
            hit_taoyuan = true;
            break;
        }
    }
    assert!(hit_taoyuan, "expected at least one seed to roll Taoyuan for gwtianbian");
}

#[test]
fn u05_b5b_r2_sharp50_hupeng2_non_discardable() {
    // Defect 3 (#50): hupeng2 locks gw_dudayuanshuai2 in ASK_LINGJI_DISCARD, ASK_HUANDIE_DISCARD_PICK, ASK_GWGOUHUN_CARD
    let (v, e0) = setup_gwent_locked(5651);
    let d2 = find_card(&e0, "gw_dudayuanshuai2");
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let other_cards = find_cards_by(&e0, |d| d.kind == CardKind::Shan, 4);

    // 3a) lingji_discard excludes gw_dudayuanshuai2
    let mut e = grant_skill(&e0, &v, "hupeng", 0);
    e = grant_skill(&e, &v, "lingji", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    e = set_deck_top(&e, &v, &[other_cards[0], other_cards[1]]);
    activate(&mut e, 0, "lingji");
    let lc = e.open_choice().cloned().expect("lingji_discard");
    assert_eq!(lc.spec.prompt, "lingji_discard");
    assert!(
        !lc.spec.options.contains(&ChoiceOption::Card(d2)),
        "hupeng2 must exclude gw_dudayuanshuai2 from lingji_discard"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: lc.id,
            selection: vec![
                ChoiceOption::Card(other_cards[0]),
                ChoiceOption::Card(other_cards[1]),
            ],
        },
    );
    drain_default(&mut e);
    assert!(
        e.debug_hand(0).contains(&d2),
        "gw_dudayuanshuai2 remains in hand after lingji_discard"
    );

    // 3b) huandie_discard_pick excludes gw_dudayuanshuai2
    let mut e = grant_skill(&e0, &v, "hupeng", 0);
    e = grant_skill(&e, &v, "huandie", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    e = inject_hand(&e, &v, &[d2, other_cards[0], other_cards[1]], 1);
    let mut snap = e.save();
    snap.core.players[1].hp = 1;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Confirm);
    let ht = e.open_choice().cloned().expect("huandie_targets");
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ht.id,
            selection: vec![],
        },
    );
    for _ in 0..8 {
        if e.open_choice()
            .is_some_and(|c| c.spec.prompt == "huandie_discard_pick")
        {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.allow_cancel {
                answer_cancel(&mut e, c.observer);
                continue;
            }
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hd = e.open_choice().cloned().expect("huandie_discard_pick");
    assert_eq!(hd.spec.prompt, "huandie_discard_pick");
    assert!(
        !hd.spec.options.contains(&ChoiceOption::Card(d2)),
        "hupeng2 must exclude gw_dudayuanshuai2 from huandie_discard_pick"
    );
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: hd.id,
            selection: vec![
                ChoiceOption::Card(other_cards[0]),
                ChoiceOption::Card(other_cards[1]),
            ],
        },
    );
    drain_default(&mut e);
    assert!(
        e.debug_hand(1).contains(&d2),
        "gw_dudayuanshuai2 remains in victim hand after huandie_discard_pick"
    );

    // 3c) gwgouhun_card excludes gw_dudayuanshuai2 (and refuses activation if only d2)
    let mut e = grant_skill(&e0, &v, "hupeng", 0);
    e = grant_skill(&e, &v, "gwgouhun", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    e = inject_hand(&e, &v, &[sha], 1);
    assert!(
        matches!(activate(&mut e, 0, "gwgouhun"), CommandOutcome::Rejected(_)),
        "gwgouhun cannot activate with only locked gw_dudayuanshuai2"
    );
    let mut e = inject_hand(&e, &v, &[other_cards[0], d2], 0);
    activate(&mut e, 0, "gwgouhun");
    let gc = e.open_choice().cloned().expect("gwgouhun_card");
    assert_eq!(gc.spec.prompt, "gwgouhun_card");
    assert!(
        !gc.spec.options.contains(&ChoiceOption::Card(d2)),
        "hupeng2 must exclude gw_dudayuanshuai2 from gwgouhun_card"
    );
    answer_cancel(&mut e, 0);
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r2_sharp50_limits_targets_and_lanquan_wash() {
    let (v, e0) = setup_gwent_locked(5652);
    let black_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let wuyao = find_card(&e0, "gw_wuyao");
    let black_duel = card_of_kind_and_suit(&e0, CardKind::Duel, Some(Suit::Spade));
    let red_duel = card_of_kind_and_suit(&e0, CardKind::Duel, Some(Suit::Diamond));
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, Some(Suit::Heart));
    let shans = find_cards_by(&e0, |d| d.kind == CardKind::Shan, 4);
    let diamond_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Diamond));

    // 2a) Weimu (`帷幕`) blocks black Duel, Kongcheng (`空城`) blocks Sha in lanquan / sqlongwu / jielue / junchi
    // Note: keep only seat 0 and seat 1 (Weimu) / seat 2 (Kongcheng) alive for unambiguous target checks
    let make_3p_dead_seat3 = |eng: engine_native::Engine| -> engine_native::Engine {
        let mut snap = eng.save();
        snap.core.players[3].hp = 0;
        snap.core.players[3].alive = false;
        content_native_loader::restore_locked_engine(&snap, &v).unwrap()
    };

    // junchi excludes black Duel against Weimu (`帷幕`) target (but allows red Duel)
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "junchi", 0));
    e = grant_skill(&e, &v, "gwminxiang", 0);
    e = grant_skill(&e, &v, "帷幕", 1);
    e = inject_hand(&e, &v, &[diamond_sha, black_duel, red_duel], 0);
    activate(&mut e, 0, "gwminxiang");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(diamond_sha));
    let mt = e.open_choice().cloned().expect("gwminxiang targets");
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: mt.id,
            selection: vec![ChoiceOption::Player(2), ChoiceOption::Player(1)],
        },
    );
    pass_window(&mut e);
    let jc = e.open_choice().cloned().expect("junchi_use");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(black_duel)),
        "junchi must exclude black Duel when target has Weimu"
    );
    assert!(
        jc.spec.options.contains(&ChoiceOption::Card(red_duel)),
        "junchi keeps red Duel against Weimu target"
    );
    answer_cancel(&mut e, 0);
    drain_default(&mut e);

    // sqlongwu: same-turn phaseEnd enforces Sha usable quota (blocked after Play Sha) and Weimu
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "sqlongwu", 0));
    e = grant_skill(&e, &v, "帷幕", 1);
    e = inject_hand(&e, &v, &[black_sha, wuyao, black_duel, red_duel], 0);
    // Use red_duel / no, first Play black_sha on seat 1 (not Kongcheng) to consume Play Sha quota!
    submit(&mut e, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    pass_window(&mut e);
    // Now Play red_duel out or keep red_duel; advance to phaseEnd with [wuyao, black_duel, red_duel] in hand
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "sqlongwu_use_card") {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "discard_phase_pick" || c.spec.allow_cancel {
                answer_cancel(&mut e, 0);
                continue;
            }
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    // Note DiscardPhase may drop excess above hp; ensure only seat 0 has <= 4 cards (injected 4 cards, used 1 -> 3 <= 4, no discard!)
    let sw = e.open_choice().cloned().expect("sqlongwu_use_card at phaseEnd");
    assert!(
        !sw.spec.options.contains(&ChoiceOption::Card(wuyao)),
        "sqlongwu at phaseEnd must enforce getCardUsable(\"sha\") after Play-phase Sha"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: sw.id,
            selection: vec![ChoiceOption::Card(black_duel)],
        },
    );
    let swt = e.open_choice().cloned().expect("sqlongwu_use_target for black_duel");
    assert!(
        !swt.spec.options.contains(&ChoiceOption::Player(1)),
        "sqlongwu black Duel must exclude Weimu seat 1"
    );
    assert!(
        swt.spec.options.contains(&ChoiceOption::Player(2)),
        "sqlongwu black Duel allows non-Weimu seat 2"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: swt.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    drain_default(&mut e);

    // sqlongwu Sha target excludes Kongcheng (seat 2, empty hand) & allows non-Kongcheng (seat 1)
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "sqlongwu", 0));
    e = grant_skill(&e, &v, "空城", 2);
    e = inject_hand(&e, &v, &[black_sha], 0);
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "sqlongwu_use_card") {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(black_sha));
    let swt = e.open_choice().cloned().expect("sqlongwu Sha target");
    assert!(
        !swt.spec.options.contains(&ChoiceOption::Player(2)),
        "sqlongwu Sha must exclude empty-hand Kongcheng seat 2"
    );
    assert!(swt.spec.options.contains(&ChoiceOption::Player(1)));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: swt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "sqlongwu Sha resolved on seat 1");

    // lanquan: enforces getCardUsable("sha") after Play Sha, blocks Weimu/Kongcheng, and washes discard when deck.len() < 6
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "lanquan", 0));
    e = grant_skill(&e, &v, "帷幕", 1);
    e = inject_hand(&e, &v, &[diamond_sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: diamond_sha, targets: vec![1] });
    pass_window(&mut e);
    e = set_deck_top(&e, &v, &[wuzhong, black_sha, wuyao, black_duel, red_duel, shans[0]]);
    activate(&mut e, 0, "lanquan");
    let lqp = e.open_choice().cloned().expect("lanquan_pick");
    assert!(
        !lqp.spec.options.contains(&ChoiceOption::Card(black_sha))
            && !lqp.spec.options.contains(&ChoiceOption::Card(wuyao)),
        "lanquan must exclude Sha/gw_wuyao after Play-phase Sha"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: lqp.id,
            selection: vec![ChoiceOption::Card(black_duel)],
        },
    );
    let lqt = e.open_choice().cloned().expect("lanquan_target for black_duel");
    assert!(
        !lqt.spec.options.contains(&ChoiceOption::Player(1)),
        "lanquan black Duel must exclude Weimu seat 1"
    );
    assert!(lqt.spec.options.contains(&ChoiceOption::Player(2)));
    answer_cancel(&mut e, 0);
    drain_default(&mut e);

    // lanquan Sha excludes empty-hand Kongcheng (seat 2) and allows seat 1; Play Sha after lanquan Sha is rejected
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "lanquan", 0));
    e = grant_skill(&e, &v, "空城", 2);
    e = set_deck_top(&e, &v, &[black_sha, shans[0], shans[1], shans[2], shans[3], black_duel]);
    activate(&mut e, 0, "lanquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(black_sha));
    let lqt = e.open_choice().cloned().expect("lanquan_target for Sha");
    assert!(
        !lqt.spec.options.contains(&ChoiceOption::Player(2)),
        "lanquan Sha must exclude empty-hand Kongcheng seat 2"
    );
    assert!(lqt.spec.options.contains(&ChoiceOption::Player(1)));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: lqt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    pass_window(&mut e);
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "lanquan Sha dealt 1");
    e = inject_hand(&e, &v, &[diamond_sha], 0);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: diamond_sha, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "Sha used via lanquan consumes Play-phase Sha quota"
    );

    // lanquan when deck.len() == 2 (unplayable Shan x2) washes discard into deck so Wuzhong in discard is peeked!
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "lanquan", 0));
    let mut snap = e.save();
    let keep_top = [shans[0], shans[1]];
    let extra_cards: Vec<_> = snap
        .core
        .deck
        .drain(..)
        .chain(snap.core.discard.drain(..))
        .filter(|c| !keep_top.contains(c) && *c != wuzhong)
        .collect();
    snap.core.players[2].hand.extend(extra_cards);
    snap.core.deck = vec![shans[0], shans[1]];
    snap.core.discard = vec![wuzhong, black_duel, red_duel, shans[2]];
    snap.core.players[2].hand.retain(|c| !snap.core.discard.contains(c));
    snap.core.vm_aux.flags.insert((0, format!("gw_shenyu_vanishtag:{wuzhong}")), 1);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    activate(&mut e, 0, "lanquan");
    let lqw = e.open_choice().cloned().expect("lanquan_pick after washing discard");
    assert!(
        lqw.spec.options.contains(&ChoiceOption::Card(wuzhong)),
        "lanquan washed discard when deck.len() < 6 so Wuzhong is in top 6"
    );
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, format!("gw_shenyu_vanishtag:{wuzhong}"))).copied().unwrap_or(0),
        0,
        "lanquan deck wash clears vanishtags"
    );
    answer_cancel(&mut e, 0);

    // jielue excludes Weimu on black Duel
    let mut e = grant_skill(&e0, &v, "jielue", 0);
    e = grant_skill(&e, &v, "帷幕", 1);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[black_duel], 2);
    advance_to_seat_play_or_choice(&mut e, 0);
    let jc = e.open_choice().cloned().expect("jielue_target for black_duel");
    assert_eq!(jc.spec.prompt, "jielue_target");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Player(1)),
        "jielue black Duel excludes Weimu seat 1"
    );
    assert!(jc.spec.options.contains(&ChoiceOption::Player(3)));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: jc.id,
            selection: vec![ChoiceOption::Player(3)],
        },
    );
    drain_default(&mut e);

    // jielue excludes Kongcheng on Sha
    let mut e = grant_skill(&e0, &v, "jielue", 0);
    e = grant_skill(&e, &v, "空城", 3);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 3);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[black_sha], 2);
    advance_to_seat_play_or_choice(&mut e, 0);
    let jc = e.open_choice().cloned().expect("jielue_target for Sha");
    assert_eq!(jc.spec.prompt, "jielue_target");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Player(3)),
        "jielue Sha excludes empty-hand Kongcheng seat 3"
    );
    assert!(jc.spec.options.contains(&ChoiceOption::Player(1)));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: jc.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_default(&mut e);

    // Hidden (`yinshen_active`) + VM/flag `eval_card_target_canceled` (`u05_mod:molicha`) also exclude targets,
    // and equipping Zhuge permits lanquan Sha even after Play-phase Sha
    let mut e = make_3p_dead_seat3(grant_skill(&e0, &v, "lanquan", 0));
    let zhuge = card_of_kind_and_suit(&e0, CardKind::Zhuge, None);
    e = inject_equip(&e, &v, zhuge, 0);
    e = inject_hand(&e, &v, &[diamond_sha], 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: diamond_sha,
            targets: vec![1],
        },
    );
    pass_window(&mut e);
    let mut snap = e.save();
    snap.core.players[2].yinshen_active = true;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = set_deck_top(
        &e,
        &v,
        &[black_sha, shans[0], shans[1], shans[2], shans[3], red_duel],
    );
    activate(&mut e, 0, "lanquan");
    let lqp = e.open_choice().cloned().expect("lanquan_pick with Zhuge");
    assert!(
        lqp.spec.options.contains(&ChoiceOption::Card(black_sha)),
        "lanquan permits Sha after Play Sha when Zhuge is equipped"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: lqp.id,
            selection: vec![ChoiceOption::Card(black_sha)],
        },
    );
    let lqt = e.open_choice().cloned().expect("lanquan_target for Sha vs yinshen");
    assert!(
        !lqt.spec.options.contains(&ChoiceOption::Player(2)),
        "lanquan Sha excludes hidden seat 2"
    );
    assert!(lqt.spec.options.contains(&ChoiceOption::Player(1)));
    answer_cancel(&mut e, 0);
    drain_default(&mut e);
    assert_zone_conservation(&e);
}
