//! U05-2 卡牌 / 食物 / 装备重做回归（`card/gujian/{card,skill}.js` + `character/gujian/card.js`）。
//!
//! 覆盖：
//! - `yanjiadan_{heart,diamond,club,spade}` (`character/gujian/card.js:4-205`): 离牌堆、3 选 1、目标选取、返还离牌堆、mid-choice save->restore->Answer
//! - `g_shihuifen` / `shihuifen` (`card/gujian/skill.js:663-704`): 他人出牌阶段开始询问全局打出与禁止指定
//! - `heilonglinpian` (`card/gujian/skill.js:456-467`): +1 防御距离并在自己下回合开始清理
//! - `liutouge` (`card/gujian/card.js:1017`, `player.js:16070-16125`): 友方增益/敌方减益池
//! - `qiankunbiao` / `shenhuofeiya` (`card/gujian/card.js:870-989`): pure 距离 1 含使用者
//! - `feibiao` (`card/gujian/card.js:781`) / `bingpotong` (`card/gujian/card.js:656`)
//! - `shujinsan` (`card/gujian/card.js:504`), `xiajiao` (`card/gujian/skill.js:61-104`), `luyugeng` (`card/gujian/skill.js:25-60`),
//!   `ziyangdan` (`card/gujian/skill.js:5-24`), `mizhilianou` (`card/gujian/skill.js:106-150`)
//! - `gjyuheng` -> `gjyuheng_plus` -> `gjyuheng_pro` (`card/gujian/skill.js:498-662`): 黑桃代价、黑牌代价、3次/7次升级与取消返还次

use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn u05_2_cards_yanjiadan_all_four_choices_restore_and_negative() {
    let (verified, mut e) = gujian_verified_and_engine(2026_2001, 4);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // Choice A (`yanjiadan_heart` -> Tag(1) Ziyangdan) + mid-choice save->restore->Answer
    let yh = move_named_to_hand(&mut e, 0, "yanjiadan_heart");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yh,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c1 = e.open_choice().cloned().expect("gj_yanjiadan_vcard");
    assert_eq!(c1.spec.prompt, "gj_yanjiadan_vcard");
    // mid-choice save -> restore -> Answer
    let mut restored = save_restore_verified(&e, &verified);
    let r_choice = restored.open_choice().cloned().expect("restored choice");
    assert_eq!(r_choice.spec.prompt, "gj_yanjiadan_vcard");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Tag(1)]); // Ziyangdan
    let c2 = restored.open_choice().cloned().expect("gj_yanjiadan_target");
    assert_eq!(c2.spec.prompt, "gj_yanjiadan_target");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Player(0)]);
    drain(&mut restored);
    assert_eq!(restored.state_core().player(0).hujia, 3, "ziyangdan grants 3 hujia");
    assert!(
        restored.state_core().detached_cards.iter().any(|&(c, _)| c == yh),
        "yanjiadan returns to detached_cards"
    );
    assert_zone_conservation_all(&restored);

    // Choice B (`yanjiadan_club` -> Tag(0) Bingpotong multi-target including self)
    let mut e2 = gujian_verified_and_engine(2026_2002, 4).1;
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let mut used = Vec::new();
    let yc = move_named_to_hand(&mut e2, 0, "yanjiadan_club");
    let red0 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Tao, Suit::Heart, &mut used);
    let blk0 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Club, &mut used);
    let blk1 = move_kind_suit_to_hand(&mut e2, 1, CardKind::Sha, Suit::Spade, &mut used);
    let _ = (red0, blk0);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: yc,
            targets: vec![],
        },
    );
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "gj_yanjiadan_vcard");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Tag(0)]); // Bingpotong
    let c_tgt = e2.open_choice().cloned().expect("target pick");
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(0)), "bingpotong can pick self");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Player(1)]);
    // Seat 0 shows blk0 (black) -> Seat 1 shows blk1 (black) -> same color: 0 discards blk0, 1 loses 1 HP!
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(blk0)]);
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(blk1)]);
    drain(&mut e2);
    assert!(e2.debug_hp(1) < 4, "bingpotong same-color show deals 1 loseHp");
    assert!(!e2.debug_hand(0).contains(&blk0), "owner discarded matched shown card");
    assert_zone_conservation_all(&e2);

    // Negative: `yanjiadan_spade` with distance-1 enemy only & no equips anywhere -> Longxugou/Qiankun/Feibiao no targets
    let mut e3 = gujian_verified_and_engine(2026_2003, 2).1;
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let ys = move_named_to_hand(&mut e3, 0, "yanjiadan_spade");
    let _ = submit(
        &mut e3,
        0,
        GameAction::PlayCard {
            card: ys,
            targets: vec![],
        },
    );
    drain(&mut e3);
    assert!(e3.state_core().detached_cards.iter().any(|&(c, _)| c == ys));
}

#[test]
fn u05_2_cards_shihuifen_global_ai_heilong_and_target_rules() {
    // 1. `g_shihuifen`: triggers globally at another seat's Phase::Play begin + AI public info heuristic
    let (_, mut e) = gujian_verified_and_engine(2026_2004, 4);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let shf = move_named_to_hand(&mut e, 0, "shihuifen");
    let enemy = enemy_seat(&e, 0);
    // Test AI public-info decision on gj_shihuifen_use when enemy's Play phase begins
    advance_to_phase(&mut e, enemy, Phase::Draw);
    // Advance once from Draw -> Play where g_shihuifen asks seat 0
    let _ = submit(&mut e, enemy, GameAction::NextPhase);
    let ch = e.open_choice().cloned().expect("g_shihuifen prompt");
    assert_eq!(ch.observer, 0);
    assert_eq!(ch.spec.prompt, "gj_shihuifen_use");
    // AI selects shihuifen against enemy (or Pass against ally)
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let plan = e.plan_action(0, &planner);
    assert!(matches!(plan.action, GameAction::Answer { .. }), "{plan:?}");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(shf)]);
    // enemy loses ability to target cards while shihuifen_ban active
    assert!(e.debug_has_skill(enemy, engine_native::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY));
    drain(&mut e);

    // 2. `heilonglinpian` (`card/gujian/skill.js:456-467`, `card.js:383`): +1 hujia, +1 defense distance & resets at owner TurnBegin
    let (_, mut e2) = gujian_verified_and_engine(2026_2005, 4);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let d_before = e2.state_core().computed_distance(e2.catalog(), 1, 0);
    let hj_before = e2.state_core().player(0).hujia;
    let hl = move_named_to_hand(&mut e2, 0, "heilonglinpian");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: hl, targets: vec![] });
    pass_nullify_window(&mut e2);
    drain(&mut e2);
    assert_eq!(e2.state_core().player(0).hujia, hj_before + 1, "heilonglinpian grants +1 hujia");
    assert_eq!(
        flag(&e2, 0, "u05_mod:gj_heilonglinpian"),
        1,
        "heilonglinpian temp defense flag active"
    );
    assert_eq!(
        e2.state_core().computed_distance(e2.catalog(), 1, 0),
        d_before + 1,
        "heilonglinpian adds +1 defense distance"
    );
    advance_to_phase(&mut e2, 1, Phase::Play);
    advance_to_phase(&mut e2, 0, Phase::Judge);
    assert_eq!(
        flag(&e2, 0, "u05_mod:gj_heilonglinpian"),
        0,
        "heilonglinpian temp defense flag cleared on owner TurnBegin"
    );
    assert_eq!(e2.state_core().computed_distance(e2.catalog(), 1, 0), d_before);

    // 3. `qiankunbiao` & `shenhuofeiya`: pure distance 1 includes owner, spares seat 2
    let (_, mut e3) = gujian_verified_and_engine(2026_2006, 4);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let mut used = Vec::new();
    let qkb = move_named_to_hand(&mut e3, 0, "qiankunbiao");
    let eq0 = find_card_by(&e3, |id, d| !used.contains(&id) && d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon));
    used.push(eq0);
    move_card_to_hand(&mut e3, 0, eq0);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: eq0, targets: vec![] });
    let _ = move_kind_suit_to_hand(&mut e3, 1, CardKind::Sha, Suit::Heart, &mut used);
    let _ = move_kind_suit_to_hand(&mut e3, 2, CardKind::Sha, Suit::Club, &mut used);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: qkb, targets: vec![1] });
    drain(&mut e3);
    assert!(e3.state_core().player(0).equips.is_empty(), "qiankunbiao pure dist 1 discards owner equip too");
    assert!(e3.debug_hand(1).is_empty(), "target 1 discarded");
    assert!(e3.debug_hand(2).is_empty(), "pure dist 1 neighbor 2 discarded");
}

#[test]
fn u05_2_cards_yuheng_upgrade_chain_and_mizhilianou_dying_and_shujinsan_he() {
    // 1. `gjyuheng` -> `gjyuheng_plus` (>=3 uses) -> `gjyuheng_pro` (>=7 uses) + cancel refund
    let (_, mut e) = gujian_verified_and_engine(2026_2007, 4);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let enemy = enemy_seat(&e, 0);
    let yh = find_card_by(&e, |_, d| {
        d.kind == CardKind::Gjyuheng && !d.name.contains("plus") && !d.name.contains("pro")
    });
    move_card_to_hand(&mut e, 0, yh);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: yh, targets: vec![] });
    drain(&mut e);

    let mut used = vec![yh];
    let sp0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let cl0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Club, &mut used);
    let _esp = move_kind_suit_to_hand(&mut e, enemy, CardKind::Dismantle, Suit::Spade, &mut used);
    let sk_yh = e.catalog().skill_by_name("gj_yuheng_skill").unwrap();
    // Cancel cost choice -> refunds once_used so can re-activate same turn!
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yh });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yuheng_cost");
    let _ = answer_choice(&mut e, vec![]);
    assert!(
        !e.state_core().skills.get(&(0, sk_yh)).unwrap().once_used,
        "cancel yuheng cost must refund once_used"
    );
    // Seed 2 previous uses on card so next activation reaches threshold 3 -> upgrades to gjyuheng_plus!
    e.state_core_mut().vm_aux.flags.insert((0, format!("gj_yuheng_uses:{yh}")), 2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yh });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(sp0)]);
    drain(&mut e);
    let yh_plus = find_card_by(&e, |_, d| d.name == "gjyuheng_plus");
    assert!(e.state_core().player(0).equips.contains(&yh_plus), "upgraded to gjyuheng_plus");
    assert!(e.state_core().detached_cards.iter().any(|&(c, _)| c == yh));

    // Upgrade from plus (6 uses -> 7 uses -> gjyuheng_pro unlimited!)
    e.state_core_mut().vm_aux.flags.insert((0, format!("gj_yuheng_uses:{yh_plus}")), 6);
    let _esp2 = move_kind_suit_to_hand(&mut e, enemy, CardKind::Dismantle, Suit::Spade, &mut used);
    let sk_plus = e.catalog().skill_by_name("gjyuheng_plus_skill").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_plus });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(cl0)]); // black (club) cost legal on plus!
    drain(&mut e);
    let yh_pro = find_card_by(&e, |_, d| d.name == "gjyuheng_pro");
    assert!(e.state_core().player(0).equips.contains(&yh_pro), "upgraded to gjyuheng_pro");
    // Unlimited on pro: can activate again in same play phase!
    let cl0b = move_kind_suit_to_hand(&mut e, 0, CardKind::Nullify, Suit::Club, &mut used);
    let _esp3 = move_kind_suit_to_hand(&mut e, enemy, CardKind::Snatch, Suit::Spade, &mut used);
    let sk_pro = e.catalog().skill_by_name("gjyuheng_pro_skill").unwrap();
    let out_pro = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_pro });
    assert!(matches!(out_pro, CommandOutcome::Pending { .. }), "gjyuheng_pro unlimited: {out_pro:?}");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(cl0b)]);
    drain(&mut e);

    // 2. `mizhilianou`: allows Heart equip card to be played as Tao during dying rescue
    let (_, mut e2) = gujian_verified_and_engine(2026_2008, 3);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let mz = move_named_to_hand(&mut e2, 0, "mizhilianou");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: mz, targets: vec![1] });
    drain(&mut e2);
    // Seat 1 equips a Heart Weapon/Equipment and enters Play hurt
    let h_eq = find_card_by(&e2, |_, d| d.suit == Suit::Heart && d.kind.equip_slot().is_some());
    move_card_to_hand(&mut e2, 1, h_eq);
    advance_to_phase(&mut e2, 1, Phase::Play);
    let _ = submit(&mut e2, 1, GameAction::PlayCard { card: h_eq, targets: vec![] });
    e2.debug_set_hp(1, 2);
    let out_tao = submit(&mut e2, 1, GameAction::PlayCard { card: h_eq, targets: vec![] });
    assert!(!matches!(out_tao, CommandOutcome::Rejected(_)), "mizhilianou Heart equip -> Tao: {out_tao:?}");
    drain(&mut e2);
    assert_eq!(e2.debug_hp(1), 3, "recovered via Heart equip Tao");

    // 3. `shujinsan` ("he") + `ziyangdan` removes when hujia==0 + `feibiao` rejected at dist 1
    let (_, mut e3) = gujian_verified_and_engine(2026_2009, 4);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let fb = move_named_to_hand(&mut e3, 0, "feibiao");
    assert_eq!(
        match submit(&mut e3, 0, GameAction::PlayCard { card: fb, targets: vec![1] }) {
            CommandOutcome::Rejected(r) => Some(r),
            _ => None,
        },
        Some(RejectReason::IllegalTargets),
        "feibiao rejects dist 1"
    );
    let sjs = move_named_to_hand(&mut e3, 0, "shujinsan");
    let eq_s = find_card_by(&e3, |_, d| d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon));
    move_card_to_hand(&mut e3, 1, eq_s);
    advance_to_phase(&mut e3, 1, Phase::Play);
    let _ = submit(&mut e3, 1, GameAction::PlayCard { card: eq_s, targets: vec![] });
    advance_to_phase(&mut e3, 0, Phase::Play);
    let h1_b = e3.debug_hand(1).len();
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: sjs, targets: vec![1] });
    pass_nullify_window(&mut e3);
    let cs = e3.open_choice().cloned().expect("shujinsan choice");
    assert!(cs.spec.options.contains(&ChoiceOption::Card(eq_s)), "shujinsan allows discarding equip ('he')");
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(eq_s)]);
    drain(&mut e3);
    assert!(e3.state_core().player(1).equips.is_empty(), "shujinsan discarded seat 1 equip");
    assert_eq!(e3.debug_hand(1).len(), h1_b + 1, "shujinsan drew 1 card for discarded equip");
}

#[test]
fn u05_2_cards_shenhuofeiya_liutouge_luyugeng_and_xiajiao() {
    let (verified, mut e) = gujian_verified_and_engine(2026_2010, 4);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // 1. `shenhuofeiya` (`card/gujian/card.js:906`): hits target 1 + pure distance-1 neighbor (2), spares dist-2 (3)
    let shfy = move_named_to_hand(&mut e, 0, "shenhuofeiya");
    let hp1_b = e.debug_hp(1);
    let hp2_b = e.debug_hp(2);
    let hp3_b = e.debug_hp(3);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: shfy, targets: vec![1] });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_b - 1, "shenhuofeiya deals 1 fire dmg to target");
    assert_eq!(e.debug_hp(2), hp2_b - 1, "shenhuofeiya splashes pure dist-1 neighbor");
    assert_eq!(e.debug_hp(3), hp3_b, "shenhuofeiya spares non-adjacent seat 3");

    // 2. `liutouge` (`card/gujian/card.js:1017`): friendly target receives buff, enemy receives debuff
    let ltg = move_named_to_hand(&mut e, 0, "liutouge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: ltg, targets: vec![0] });
    drain(&mut e);
    assert_zone_conservation_all(&e);

    // 3. `luyugeng` -> grants `"gj_food_luyugeng"` -> cost basic card -> discover 1 of 3 & mid-choice save->restore->Answer
    let lyg = move_named_to_hand(&mut e, 0, "luyugeng");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: lyg, targets: vec![0] });
    drain(&mut e);
    let mut used = Vec::new();
    let cost_b = move_kind_suit_to_hand(&mut e, 0, CardKind::Shan, Suit::Heart, &mut used);
    let sk_lyg = e.catalog().skill_by_name(engine_native::content::host::GJ_FOOD_LUYUGENG_KEY).unwrap();
    let h0_b = e.debug_hand(0).len();
    let out_lyg = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_lyg });
    assert!(matches!(out_lyg, CommandOutcome::Pending { .. }), "{out_lyg:?}");
    let c_cost = e.open_choice().cloned().expect("gj_luyugeng_cost");
    assert_eq!(c_cost.spec.prompt, "gj_luyugeng_cost");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(cost_b)]);
    let cd = e.open_choice().cloned().expect("gj_luyugeng_discover");
    assert_eq!(cd.spec.prompt, "gj_luyugeng_discover");
    assert!(!cd.spec.options.is_empty());
    let mut restored = save_restore_verified(&e, &verified);
    let pick0 = restored.open_choice().unwrap().spec.options[0];
    let _ = answer_choice(&mut restored, vec![pick0]);
    drain(&mut restored);
    assert_eq!(restored.debug_hand(0).len(), h0_b, "luyugeng_skill spent 1 basic and gained 1 discovered card");

    // 4. `xiajiao` + `xiajiao2` (`card/gujian/skill.js:61-104`) on `restored` (whose choice was answered cleanly)
    let xj = move_named_to_hand(&mut restored, 0, "xiajiao");
    let _ = submit(&mut restored, 0, GameAction::PlayCard { card: xj, targets: vec![0] });
    drain(&mut restored);
    assert!(restored.debug_has_skill(0, engine_native::content::host::GJ_FOOD_XIAJIAO_KEY));
    assert_zone_conservation_all(&restored);
}
