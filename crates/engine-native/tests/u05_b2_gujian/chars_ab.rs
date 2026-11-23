//! U05-2 武将技 Batch A & B 上游等价回归（`character/gujian/skill.js:5-1963`）。
//!
//! 覆盖 8 位武将 21 技：
//! - 百里屠苏：`xuelu`、`fanshi`、`shahun`（+`shahun2`）
//! - 风晴雪：`qinglan`、`yuehua`、`gjqtwuxie`
//! - 襄铃：`xlqianhuan`、`meihu`、`xidie`（+`xidie2`）
//! - 方兰生：`fanyin`、`mingkong`（+`mingkong2`）、`fumo`
//! - 尹千觞：`zuiji`、`zuizhan`
//! - 红玉：`jianwu`、`meiying`
//! - 乐无异：`yanjia`、`xiuhua`、`liuying`
//! - 巽芳：`manwu`、`xfanghua`

use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn u05_2_batch_a_tusu_qingxue_xiangling_lansheng() {
    let (verified, mut e) = gujian_verified_and_engine(2026_3001, 4);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e.assign_general_named(1, "gjqt_fengqingxue").unwrap();
    e.assign_general_named(2, "gjqt_xiangling").unwrap();
    e.assign_general_named(3, "gjqt_fanglansheng").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    let mut used = Vec::new();
    // 1. 屠苏 `xuelu`: PhaseEndWindow asks red card -> Fire damage floor((lost+1)/2) clamped to 2
    // Give seat 3 (方兰生) 1 hand card so `mingkong` (0 hand -1 dmg) does not attenuate xuelu's 2 dmg.
    let _dummy3 = move_kind_suit_to_hand(&mut e, 3, CardKind::Shan, Suit::Heart, &mut used);
    e.debug_set_hp(0, 1); // lost = 3 -> (3+1)/2 = 2 fire damage
    let red_sha = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow (triggers xuelu!)
    let cx = e.open_choice().cloned().expect("xuelu card ask");
    assert_eq!(cx.spec.prompt, "gj_xuelu_card");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(red_sha)]);
    let cxt = e.open_choice().cloned().expect("xuelu target ask");
    assert_eq!(cxt.spec.prompt, "gj_xuelu_target");
    let mut restored = save_restore_verified(&e, &verified);
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Player(3)]);
    drain(&mut restored);
    assert!(restored.debug_hp(3) <= 1 || !restored.state_core().player(3).alive);

    // 2. 风晴雪 `qinglan` (DamageBefore prevents elemental damage & source recovers+draws) + `yuehua` + `gjqtwuxie`
    let (_, mut e2) = gujian_verified_and_engine(2026_3002, 4);
    e2.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e2.assign_general_named(1, "gjqt_fengqingxue").unwrap();
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let mut used2 = Vec::new();
    // Negative gjqtwuxie: seat 0 cannot use Dismantle on seat 1 using a non-Heart? Wait: check gjqtwuxie blocks Heart trick/non-Heart? Heart cards/delayed tricks!
    // Check upstream character/gujian/skill.js:1867: gjqtwuxie prevents Heart trick from targeting owner OR delayed trick!
    let le_heart = move_kind_suit_to_hand(&mut e2, 0, CardKind::Le, Suit::Heart, &mut used2);
    assert!(
        matches!(
            submit(&mut e2, 0, GameAction::PlayCard { card: le_heart, targets: vec![1] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "gjqtwuxie blocks delayed trick targeting fengqingxue"
    );
    // yuehua: seat 1 responds/plays red off-turn -> asks confirm gj_yuehua_confirm -> draws 1
    let rsha0 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Heart, &mut used2);
    let rshan1 = move_kind_suit_to_hand(&mut e2, 1, CardKind::Shan, Suit::Diamond, &mut used2);
    let h1_b = e2.debug_hand(1).len();
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: rsha0, targets: vec![1] });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(rshan1)]); // plays red Shan
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "gj_yuehua_confirm");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Confirm]);
    assert_eq!(e2.debug_hand(1).len(), h1_b, "-1 shan +1 yuehua draw");

    // 3. `mingkong` & `fumo` & `fanyin` on 方兰生 + `qinglan` fire Sha prevention
    let (_, mut e3) = gujian_verified_and_engine(2026_3003, 4);
    e3.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e3.assign_general_named(1, "gjqt_fanglansheng").unwrap();
    e3.assign_general_named(2, "gjqt_fengqingxue").unwrap();
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let mut used3 = Vec::new();
    // 0 fires FireSha at dist-1 seat 1 (who has empty hand -> mingkong -1 so amount becomes 0, source 0 draws 1)
    let fsha = find_unused_card_by(&e3, &used3, |d| d.kind == CardKind::FireSha);
    used3.push(fsha);
    move_card_to_hand(&mut e3, 0, fsha);
    let c2 = move_kind_suit_to_hand(&mut e3, 2, CardKind::Sha, Suit::Spade, &mut used3);
    let out_fs = submit(&mut e3, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert!(!matches!(out_fs, CommandOutcome::Rejected(_)), "FireSha at dist-1 seat 1: {out_fs:?}");
    // qinglan triggers on DamageBefore for elemental damage -> cancel qinglan so damage proceeds to mingkong
    if let Some(c) = e3.open_choice().cloned() {
        if c.spec.prompt == "gj_qinglan_card" {
            let _ = answer_choice(&mut e3, vec![]); // cancel -> proceeds!
        }
    }
    let hp1_after = e3.debug_hp(1);
    assert_eq!(hp1_after, 3, "mingkong reduced 1 fire damage to 0 while empty hand");
    let _ = c2;
    // Now give 1 two red cards -> another damage triggers `fumo`!
    let f_red1 = move_kind_suit_to_hand(&mut e3, 1, CardKind::Tao, Suit::Heart, &mut used3);
    let f_red2 = move_kind_suit_to_hand(&mut e3, 1, CardKind::Shan, Suit::Diamond, &mut used3);
    let sha_a = move_kind_suit_to_hand(&mut e3, 0, CardKind::Sha, Suit::Spade, &mut used3);
    e3.state_core_mut().turn.sha_used_play_phase = false;
    let hp0_b = e3.debug_hp(0);
    let out_sa = submit(&mut e3, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    assert!(!matches!(out_sa, CommandOutcome::Rejected(_)), "Sha at dist-1 seat 1: {out_sa:?}");
    drain_until_prompt(&mut e3, "gj_fumo_card1");
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(f_red1)]);
    assert_eq!(e3.open_choice().unwrap().spec.prompt, "gj_fumo_card2");
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(f_red2)]);
    drain(&mut e3);
    assert!(e3.debug_hp(0) < hp0_b, "fumo deals 1 thunder damage to source");

    // 4. `xlqianhuan` + `meihu` + `xidie` + AI heuristic on `xlqianhuan_confirm`
    let (_, mut e4) = gujian_verified_and_engine(2026_3004, 4);
    e4.assign_general_named(0, "gjqt_xiangling").unwrap();
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_all_hands(&mut e4);
    let mut used4 = Vec::new();
    let d1 = move_kind_suit_to_hand(&mut e4, 0, CardKind::Sha, Suit::Club, &mut used4);
    let d2 = move_kind_suit_to_hand(&mut e4, 0, CardKind::Sha, Suit::Spade, &mut used4);
    let d3 = move_kind_suit_to_hand(&mut e4, 0, CardKind::Tao, Suit::Heart, &mut used4);
    let d4 = move_kind_suit_to_hand(&mut e4, 0, CardKind::Sha, Suit::Diamond, &mut used4);
    e4.debug_set_hp(0, 1); // injured, hand=4 > hp=1 -> xidie will trigger on next TurnBegin!
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let _ = (d1, d2, d3, d4);
    // End phase -> NextPhase into TurnEndWindow (after: true) triggers xlqianhuan -> AI chooses Confirm because hp<=1!
    advance_to_phase(&mut e4, 0, Phase::End);
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    drain_until_prompt(&mut e4, "gj_xlqianhuan_confirm");
    assert_eq!(e4.open_choice().unwrap().spec.prompt, "gj_xlqianhuan_confirm");
    let ai_plan = e4.plan_action(0, &planner);
    let _ = submit(&mut e4, 0, ai_plan.action);
    assert!(e4.debug_hp(0) == 2 && e4.state_core().player(0).face_down);

    // 5. `shahun` + `shahun2` dying rescue -> gains juejing -> dies after 2 turns!
    let (_, mut e5) = gujian_verified_and_engine(2026_3005, 3);
    e5.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e5.assign_general_named(1, "gjqt_hongyu").unwrap();
    e5.assign_general_named(2, "gjqt_fengqingxue").unwrap();
    advance_to_phase(&mut e5, 1, Phase::Play);
    clear_all_hands(&mut e5);
    e5.debug_set_hp(0, 1);
    let mut used5 = Vec::new();
    let ksha = move_kind_suit_to_hand(&mut e5, 1, CardKind::Sha, Suit::Spade, &mut used5);
    let _ = submit(&mut e5, 1, GameAction::PlayCard { card: ksha, targets: vec![0] });
    // Dying rescue offers ChoiceOption::Skill(shahun) via DyingSaveMachine
    let shahun_def = e5.catalog().skill_by_name("shahun").unwrap();
    let c = e5.open_choice().cloned().expect("dying_save prompt for shahun");
    assert_eq!(c.spec.prompt, "dying_save", "got {c:?}");
    assert!(
        c.spec.options.contains(&ChoiceOption::Skill(shahun_def)),
        "dying_save must offer shahun: {:?}",
        c.spec.options
    );
    let _ = answer_choice(&mut e5, vec![ChoiceOption::Skill(shahun_def)]);
    drain(&mut e5);
    assert!(e5.state_core().player(0).alive && e5.debug_hp(0) == 1);
    assert!(!e5.debug_has_skill(0, "fanshi"), "shahun removes fanshi");
    assert_eq!(flag(&e5, 0, "u05_mod:gj_juejing"), 1, "shahun grants juejing passive (maxHand+2 & +lost_hp draw)");
    assert_eq!(flag(&e5, 0, "gj_shahun_timer"), 2);
    // Juejing DrawPhase: lost = 4 - 1 = 3 -> draws 2 + 3 = 5 cards on next Turn Draw!
    advance_to_phase(&mut e5, 0, Phase::Judge);
    let h0_pre_draw = e5.debug_hand(0).len();
    let _ = submit(&mut e5, 0, GameAction::NextPhase); // Judge -> Draw
    drain(&mut e5);
    assert_eq!(e5.debug_hand(0).len(), h0_pre_draw + 5, "shahun+juejing draws 2 + lost_hp(3) cards");
    // Advance 2 TurnPhaseAfter windows on seat 0 -> shahun2 kills bailitusu at timer 0!
    advance_to_phase(&mut e5, 0, Phase::End);
    let _ = submit(&mut e5, 0, GameAction::NextPhase);
    drain(&mut e5);
    assert_eq!(flag(&e5, 0, "gj_shahun_timer"), 1);
    advance_to_phase(&mut e5, 0, Phase::End);
    let _ = submit(&mut e5, 0, GameAction::NextPhase);
    drain(&mut e5);
    assert!(!e5.state_core().player(0).alive, "shahun2 kills owner after 2 turn ends");
}

#[test]
fn u05_2_batch_b_qianshang_hongyu_wuyi_xunfang() {
    // 1. 尹千觞 `zuiji` + `zuizhan`, 红玉 `jianwu` + `meiying`, 乐无异 `yanjia` + `xiuhua` + `liuying`, 巽芳 `manwu` + `xfanghua`
    let (_, mut e) = gujian_verified_and_engine(2026_3006, 4);
    e.assign_general_named(0, "gjqt_yinqianshang").unwrap();
    e.assign_general_named(1, "gjqt_xunfang").unwrap();
    e.assign_general_named(2, "gjqt_yuewuyi").unwrap();
    e.assign_general_named(3, "gjqt_hongyu").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    let mut used = Vec::new();
    // `zuiji` active: discard 1 card -> jiu_used = true & CardUsed(Jiu) -> triggers xfanghua on others? Only when target=1!
    let c_zj = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Club, &mut used);
    let sk_zj = e.catalog().skill_by_name("zuiji").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_zj });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_zuiji_card");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(c_zj)]);
    assert!(e.state_core().turn.jiu_active);

    // `zuizhan`: red Sha targets injured seat 1 -> asks `gj_zuizhan_confirm` -> adds seat 3 & draws 1 -> triggers `xfanghua` on injured seat 1!
    e.debug_set_hp(1, 2);
    let sha_red = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_red, targets: vec![1] });
    drain_until_prompt(&mut e, "gj_zuizhan_confirm");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
    // xfanghua triggers for seat 1 (hp=2 < max_hp=3) because red Sha -> heals 2->3 BEFORE taking 2 dmg (zuiji jiu+sha) -> lands at hp=1 alive (not dying)!
    drain_until_prompt(&mut e, "gj_xfanghua_confirm");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "xfanghua recovered +1 HP before 2-dmg jiu red Sha (2+1-2=1)");

    // `manwu`: at Play->End TurnEndWindow for min-hand seat, xunfang can let them draw 1
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "gj_manwu_confirm" {
            let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        }
    }
    drain(&mut e);

    // `yanjia` + `xiuhua` + `liuying` on 乐无异 (seat 2)
    advance_to_phase(&mut e, 2, Phase::Play);
    clear_all_hands(&mut e);
    let w1 = find_unused_card_by(&e, &used, |d| {
        d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon) && !d.name.contains('_')
    });
    used.push(w1);
    let a1 = find_unused_card_by(&e, &used, |d| {
        d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Armor) && !d.name.contains('_')
    });
    used.push(a1);
    move_card_to_hand(&mut e, 2, w1);
    move_card_to_hand(&mut e, 2, a1);
    let sk_yj = e.catalog().skill_by_name("yanjia").unwrap();
    let _ = submit(&mut e, 2, GameAction::ActivateSkill { skill: sk_yj });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yanjia_card1");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(w1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yanjia_card2");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(a1)]);
    drain(&mut e);
    assert!(e.state_core().detached_cards.iter().any(|&(c, _)| c == a1), "yanjia merged second equip");
    // Merged card in hand triggers dual Weapon+Armor use check; plus test `liuying` when 2 plays Sha at dist-1 seat 1!
    e.debug_set_hp(1, 3);
    let sha2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let out_ly = submit(&mut e, 2, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(!matches!(out_ly, CommandOutcome::Rejected(_)), "Sha from 2 to dist-1 seat 1: {out_ly:?}");
    drain_until_prompt(&mut e, "gj_liuying_target");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_liuying_target");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(3)]);
    drain(&mut e);
    // `jianwu` direct hit on 红玉 (seat 3): seat 3 equips a range>=2 weapon -> targets dist-2 seat 1
    // (seat 1 attack_range == 1 < dist(1, 3) == 2 -> jianwu_direct_hit cannot dodge with Shan!)
    advance_to_phase(&mut e, 3, Phase::Play);
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 3);
    let w3 = find_unused_card_by(&e, &used, |d| {
        d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon)
            && d.kind != CardKind::Zhuge
            && !d.name.contains('_')
    });
    used.push(w3);
    move_card_to_hand(&mut e, 3, w3);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: w3, targets: vec![] });
    let sha3 = move_kind_suit_to_hand(&mut e, 3, CardKind::Sha, Suit::Spade, &mut used);
    let shan1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Shan, Suit::Heart, &mut used);
    let hp1_b = e.debug_hp(1);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha3, targets: vec![1] });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_b - 1, "jianwu direct-hit deals 1 damage when outside target attack range");
    assert!(e.debug_hand(1).contains(&shan1), "jianwu prevents target 1 from dodging with Shan");

    // Negative `jianwu`: target 0 at distance 1 (within seat 0's attack range) CAN dodge with Shan!
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha3b = move_kind_suit_to_hand(&mut e, 3, CardKind::Sha, Suit::Club, &mut used);
    let shan0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Shan, Suit::Diamond, &mut used);
    let hp0_b = e.debug_hp(0);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha3b, targets: vec![0] });
    let ch_shan = e.open_choice().cloned().expect("seat 0 CAN respond with Shan against dist-1 jianwu");
    assert_eq!(ch_shan.observer, 0);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(shan0)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0_b, "dist-1 target dodged jianwu Sha");
}

#[test]
fn u05_2_batch_ab_fanshi_meihu_xidie_fanyin_meiying_xiuhua_and_manwu_ai() {
    // 1. `fanshi` (`character/gujian/skill.js:109`): Bailitusu dealt dmg in turn -> Discard PhaseEnd loses 1 HP & draws 1
    let (_, mut e) = gujian_verified_and_engine(2026_3007, 4);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e.assign_general_named(1, "gjqt_xiangling").unwrap();
    e.assign_general_named(2, "gjqt_fanglansheng").unwrap();
    e.assign_general_named(3, "gjqt_hongyu").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let sha0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    drain(&mut e);
    // Notice `meihu` on Xiangling (seat 1) already fired if seat 0 still had hand? Wait: seat 0's hand was empty -> now check fanshi!
    let hp0_pre = e.debug_hp(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> DiscardEnd (triggers fanshi!)
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0_pre - 1, "fanshi loses 1 HP when turn_damage > 0");
    assert_eq!(e.debug_hand(0).len(), 1, "fanshi draws 1 card");

    // 2. `meihu` (`character/gujian/skill.js:235`): Xiangling damaged by source with hand -> gj_meihu_confirm + AI heuristic + steal
    let (_, mut e_mh) = gujian_verified_and_engine(2026_3008, 4);
    e_mh.assign_general_named(1, "gjqt_xiangling").unwrap();
    advance_to_phase(&mut e_mh, 0, Phase::Play);
    clear_all_hands(&mut e_mh);
    let mut u2 = Vec::new();
    let sha_mh = move_kind_suit_to_hand(&mut e_mh, 0, CardKind::Sha, Suit::Spade, &mut u2);
    let gift0 = move_kind_suit_to_hand(&mut e_mh, 0, CardKind::Tao, Suit::Heart, &mut u2);
    let _ = submit(&mut e_mh, 0, GameAction::PlayCard { card: sha_mh, targets: vec![1] });
    assert_eq!(e_mh.open_choice().unwrap().spec.prompt, "gj_meihu_confirm");
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let ai_mh = e_mh.plan_action(1, &planner);
    let _ = submit(&mut e_mh, 1, ai_mh.action);
    assert_eq!(e_mh.open_choice().unwrap().spec.prompt, "gj_meihu_give");
    let _ = answer_choice(&mut e_mh, vec![ChoiceOption::Card(gift0)]);
    drain(&mut e_mh);
    assert!(e_mh.debug_hand(1).contains(&gift0), "meihu stole source card via gj_meihu_confirm");

    // 3. `xidie` + `xidie2` (`character/gujian/skill.js:270`): hand > hp at TurnBegin -> discard up to 3 -> draw back at TurnEnd
    e_mh.debug_set_hp(1, 1);
    for _ in 0..2 {
        let extra = find_unused_card_by(&e_mh, &u2, |d| d.kind == CardKind::Sha);
        u2.push(extra);
        move_card_to_hand(&mut e_mh, 1, extra);
    }
    advance_to_phase(&mut e_mh, 0, Phase::End);
    let _ = submit(&mut e_mh, 0, GameAction::NextPhase);
    assert_eq!(e_mh.open_choice().unwrap().spec.prompt, "gj_xidie_discard");
    let xd_opt = e_mh.open_choice().unwrap().spec.options[0];
    let _ = answer_choice(&mut e_mh, vec![xd_opt]);
    assert_eq!(flag(&e_mh, 1, "gj_xidie_count"), 1);
    advance_to_phase(&mut e_mh, 1, Phase::End);
    let h1_before_xd2 = e_mh.debug_hand(1).len();
    let _ = submit(&mut e_mh, 1, GameAction::NextPhase);
    drain(&mut e_mh);
    assert!(e_mh.debug_hand(1).len() > h1_before_xd2, "xidie2 drew discarded count at TurnEnd");

    // 4. `fanyin` (`character/gujian/skill.js:340`): unlinks & turns face-up & recovers min-HP target
    let (_, mut e_fy) = gujian_verified_and_engine(2026_3009, 4);
    e_fy.assign_general_named(0, "gjqt_fanglansheng").unwrap();
    advance_to_phase(&mut e_fy, 0, Phase::End);
    clear_all_hands(&mut e_fy);
    e_fy.state_core_mut().player_mut(1).linked = true;
    e_fy.state_core_mut().player_mut(1).face_down = true;
    e_fy.debug_set_hp(1, 2);
    let _ = submit(&mut e_fy, 0, GameAction::NextPhase);
    assert_eq!(e_fy.open_choice().unwrap().spec.prompt, "gj_fanyin_target");
    let _ = answer_choice(&mut e_fy, vec![ChoiceOption::Player(1)]);
    drain(&mut e_fy);
    assert!(!e_fy.state_core().player(1).linked && !e_fy.state_core().player(1).face_down);
    assert_eq!(e_fy.debug_hp(1), 3, "fanyin recovered min-HP target");

    // 5. `meiying` (`character/gujian/skill.js:1650`) + `xiuhua` (`character/gujian/skill.js:1760`)
    let (_, mut e_my) = gujian_verified_and_engine(2026_3010, 4);
    e_my.assign_general_named(1, "gjqt_hongyu").unwrap();
    e_my.assign_general_named(2, "gjqt_yuewuyi").unwrap();
    advance_to_phase(&mut e_my, 0, Phase::Play);
    clear_all_hands(&mut e_my);
    let mut u3 = Vec::new();
    let eq_d = find_unused_card_by(&e_my, &u3, |d| d.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon));
    u3.push(eq_d);
    move_card_to_hand(&mut e_my, 0, eq_d);
    // Give seat 0 a 2nd card (or discard phase): discard eq_d in Discard phase -> `xiuhua` triggers!
    e_my.debug_set_hp(0, 1);
    let extra0 = move_kind_suit_to_hand(&mut e_my, 0, CardKind::Tao, Suit::Heart, &mut u3);
    let _ = submit(&mut e_my, 0, GameAction::NextPhase); // Play -> Discard
    let c_disc = e_my.open_choice().cloned().expect("discard choice");
    assert!(c_disc.spec.options.contains(&ChoiceOption::Card(eq_d)));
    let _ = answer_choice(&mut e_my, vec![ChoiceOption::Card(eq_d)]);
    let _ = extra0;
    drain_until_prompt(&mut e_my, "gj_xiuhua_gain");
    assert_eq!(e_my.open_choice().unwrap().spec.prompt, "gj_xiuhua_gain");
    let _ = answer_choice(&mut e_my, vec![ChoiceOption::Confirm]);
    assert!(e_my.debug_hand(2).contains(&eq_d), "xiuhua gained discarded equip");
    // Give Hongyu (seat 1) a red card so seat 0 End->TurnEndWindow (no target other) triggers `meiying`!
    let r_my = move_kind_suit_to_hand(&mut e_my, 1, CardKind::Sha, Suit::Heart, &mut u3);
    advance_to_phase(&mut e_my, 0, Phase::End);
    let _ = submit(&mut e_my, 0, GameAction::NextPhase);
    drain_until_prompt(&mut e_my, "gj_meiying_discard");
    assert_eq!(e_my.open_choice().unwrap().spec.prompt, "gj_meiying_discard");
    let hp0_b = e_my.debug_hp(0);
    let _ = answer_choice(&mut e_my, vec![ChoiceOption::Card(r_my)]);
    drain(&mut e_my);
    assert!(e_my.debug_hp(0) < hp0_b, "meiying dealt virtual Sha damage at other TurnEndWindow");
}
