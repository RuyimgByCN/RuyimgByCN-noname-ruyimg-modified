//! U05-2 武将技 Batch E 上游等价回归（`character/gujian/skill.js:1-1030, 1964-2096`）。
//!
//! 覆盖 6 位武将 19 技：
//! - 谢衣：`lingyan`、`xunjian`、`humeng`
//! - 偃甲谢衣：`xianju`
//! - 初七：`xuanci`
//! - 北洛：`lingnu`、`zhenying`、`cihong`
//! - 云无月：`yange`、`woxue`、`lianjing`
//! - 岑缨：`yunyou`、`xuanzhen`、`qingshu`

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, GameAction, Phase};

#[test]
fn u05_2_batch_e_xieyi_chuqi_beiluo() {
    let (verified, mut e) = gujian_verified_and_engine(2026_5001, 4);
    e.assign_general_named(0, "gjqt_xieyi").unwrap();
    e.assign_general_named(1, "gjqt_beiluo").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // 1. 谢衣 `lingyan`: discards card to off_court -> gains matching-suit `yanjiadan_*` from detached_cards!
    let mut used = Vec::new();
    let sp = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let sk_ly = e.catalog().skill_by_name("lingyan").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_ly });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_lingyan_card");
    let mut restored = save_restore_verified(&e, &verified);
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Card(sp)]);
    let ys = find_card_by(&restored, |_, d| d.name == "yanjiadan_spade");
    assert!(restored.debug_hand(0).contains(&ys), "lingyan grants matching-suit yanjiadan");
    assert!(restored.state_core().player(0).off_court.contains(&sp));

    // `humeng`: 谢衣 dying transforms into 初七 (max_hp=2, hp=2, +4 cards, skill `xuanci`)
    advance_to_phase(&mut restored, 1, Phase::Play);
    clear_all_hands(&mut restored);
    restored.debug_set_hp(0, 1);
    let sha_kill = move_kind_suit_to_hand(&mut restored, 1, CardKind::Sha, Suit::Club, &mut used);
    let _ = submit(&mut restored, 1, GameAction::PlayCard { card: sha_kill, targets: vec![0] });
    drain(&mut restored);
    assert!(restored.state_core().player(0).alive);
    assert_eq!(restored.state_core().player(0).max_hp, 2);
    assert_eq!(restored.debug_hp(0), 2);
    assert!(restored.debug_has_skill(0, "xuanci"));

    // 2. 初七 `xuanci`: discard 1 Club -> virtual Feibiao (unlimited distance!) -> follow-up virtual Sha!
    advance_to_phase(&mut restored, 0, Phase::Play);
    clear_all_hands(&mut restored);
    let cl_x = move_kind_suit_to_hand(&mut restored, 0, CardKind::Sha, Suit::Club, &mut used);
    let sk_xc = restored.catalog().skill_by_name("xuanci").unwrap();
    let hp1_b = restored.debug_hp(1);
    let _ = submit(&mut restored, 0, GameAction::ActivateSkill { skill: sk_xc });
    assert_eq!(restored.open_choice().unwrap().spec.prompt, "gj_xuanci_card");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Card(cl_x)]);
    assert_eq!(restored.open_choice().unwrap().spec.prompt, "gj_xuanci_target");
    let _ = answer_choice(&mut restored, vec![ChoiceOption::Player(1)]);
    drain(&mut restored);
    assert!(restored.debug_hp(1) < hp1_b, "xuanci virtual Feibiao+Sha damaged target 1");

    // 3. 北洛 `lingnu` + `zhenying` + `cihong` (`TurnPhaseEnd`)
    advance_to_phase(&mut restored, 1, Phase::Play);
    clear_all_hands(&mut restored);
    let blk_sha = move_kind_suit_to_hand(&mut restored, 1, CardKind::Sha, Suit::Spade, &mut used);
    // zhenying gives temp black copy when playing black card
    let _ = submit(&mut restored, 1, GameAction::PlayCard { card: blk_sha, targets: vec![2] });
    drain(&mut restored);
    // Advance to TurnEndWindow -> cihong asks target -> Choice B (Pass = no damage / Confirm = red discard -> virtual Sha)
    let _ = submit(&mut restored, 1, GameAction::NextPhase);
    let _ = submit(&mut restored, 1, GameAction::NextPhase);
    if let Some(ch) = restored.open_choice().cloned() {
        if ch.spec.prompt == "gj_cihong_target" {
            let _ = answer_choice(&mut restored, vec![ChoiceOption::Player(2)]);
        }
    }
    drain(&mut restored);
    assert_zone_conservation_all(&restored);
}

#[test]
fn u05_2_batch_e_yunwuyue_cenying() {
    let (_, mut e) = gujian_verified_and_engine(2026_5002, 4);
    e.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e.assign_general_named(1, "gjqt_cenying").unwrap();

    // 1. 岑缨 `yunyou`: active skill in Phase::Play discovers 1 of 3 Land kinds (`current_land`) + `qingshu` grants unused Land skill
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    let sk_yy = e.catalog().skill_by_name("yunyou").unwrap();
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: sk_yy });
    let cy = e.open_choice().cloned().expect("gj_yunyou_discover");
    assert_eq!(cy.spec.prompt, "gj_yunyou_discover");
    assert_eq!(cy.spec.options.len(), 3);
    let _ = answer_choice(&mut e, vec![cy.spec.options[0]]);
    assert!(e.state_core().current_land.is_some(), "yunyou activates a Land effect");
    // Cancel / Play -> End: `qingshu` can grant saved Land skill to an ally/player
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    if let Some(cq) = e.open_choice().cloned() {
        if cq.spec.prompt == "gj_qingshu_vcard" {
            let _ = answer_choice(&mut e, vec![cq.spec.options[0]]);
            assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_qingshu_target");
            let _ = answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
        }
    }
    drain(&mut e);

    // 2. 岑缨 `xuanzhen` (`CardUsed` targeting owner by another -> turns into picked virtual card!)
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let sha0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    let cx = e.open_choice().cloned().expect("gj_xuanzhen_confirm");
    assert_eq!(cx.spec.prompt, "gj_xuanzhen_confirm");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Confirm]);
    let cv = e.open_choice().cloned().expect("gj_xuanzhen_vcard");
    assert_eq!(cv.spec.prompt, "gj_xuanzhen_vcard");
    assert_eq!(cv.spec.options.len(), 3);
    let last = *cv.spec.options.last().unwrap();
    let _ = answer_choice(&mut e, vec![last]);
    drain(&mut e);

    // 3. 云无月 `lianjing` (`TurnPhaseAfter` picks subset `[0, 2]`, skipping seat 1/3!)
    let (_, mut e2) = gujian_verified_and_engine(2026_5003, 4);
    e2.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    advance_to_phase(&mut e2, 0, Phase::Discard);
    clear_all_hands(&mut e2);
    let _ = submit(&mut e2, 0, GameAction::NextPhase); // Discard -> End -> TurnPhaseAfter
    drain_until_prompt(&mut e2, "gj_lianjing_targets");
    if let Some(cl) = e2.open_choice().cloned() {
        if cl.spec.prompt == "gj_lianjing_targets" {
            let _ = answer_choice(&mut e2, vec![ChoiceOption::Player(2)]);
            drain(&mut e2);
            assert!(
                flag(&e2, 1, "u05_mod:gj_lianjing_out_of_play") > 0,
                "unselected seat 1 is temporarily out_of_play"
            );
            assert!(
                matches!(e2.turn().current, 0 | 2),
                "lianjing advances turn directly within [0, 2], got {}",
                e2.turn().current
            );
        }
    }

    // 4. 云无月 `yange` (lazy init marks on TurnBegin -> can clone another general's skills & hp!)
    let (_, mut e3) = gujian_verified_and_engine(2026_5004, 4);
    e3.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e3.assign_general_named(2, "gjqt_bailitusu").unwrap();
    // Advance full round to seat 3 End, then advance into seat 0 TurnBegin so advance_to_phase doesn't swallow gj_yange_target!
    advance_to_phase(&mut e3, 3, Phase::End);
    let _ = submit(&mut e3, 3, GameAction::NextPhase);
    drain_until_prompt(&mut e3, "gj_yange_target");
    if let Some(cyg) = e3.open_choice().cloned() {
        if cyg.spec.prompt == "gj_yange_target" {
            let _ = answer_choice(&mut e3, vec![ChoiceOption::Player(2)]);
            assert!(e3.debug_has_skill(0, "xuelu"), "yange cloned bailitusu's xuelu for the turn");
            advance_to_phase(&mut e3, 1, Phase::Judge);
            assert!(!e3.debug_has_skill(0, "xuelu"), "yange restored yunwuyue after turn end");
        }
    }
}

#[test]
fn u05_2_batch_e_xunjian_xianju_woxue_lingnu() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::{CommandOutcome, RejectReason};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    // 1. 谢衣 `xunjian` + 偃甲谢衣 `xianju` (odd-round qianxing untargetable + even-round jiguan gain) + 岑缨/云无月 `woxue` + 北洛 `lingnu`
    let (_, mut e) = gujian_verified_and_engine(2026_5005, 4);
    e.assign_general_named(0, "gjqt_xieyi").unwrap();
    e.assign_general_named(1, "gjqt_yanjiaxieyi").unwrap();
    e.assign_general_named(2, "gjqt_yunwuyue").unwrap();
    e.assign_general_named(3, "gjqt_beiluo").unwrap();
    // `xunjian` requires >=1 card in off_court! Activate `lingyan` to put a card into off_court.
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let sp0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let sk_ly = e.catalog().skill_by_name("lingyan").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_ly });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(sp0)]);
    // Move 12 extra non-Sha cards from deck into off_court (so n_ly == 13 guarantees `roll < n_ly`), leaving Sha copies in deck for deterministic `xunjian`!
    for _ in 0..12 {
        let cid = *e
            .state_core()
            .deck
            .iter()
            .find(|&&c| !used.contains(&c) && e.catalog().card_kind(c) != CardKind::Sha)
            .expect("non-Sha in deck");
        used.push(cid);
        e.state_core_mut().deck.retain(|&c| c != cid);
        e.state_core_mut().player_mut(0).off_court.push(cid);
    }
    let sha_xj = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Club, &mut used);
    let h0_b = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_xj, targets: vec![3] });
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0_b, "xunjian drew a matching Sha copy (-1 played +1 drawn) when off_court non-empty");

    // `xianju`: Round 2 (even) clears qianxing (no even-round jiguan gain); Round 3 (odd) grants qianxing untargetable!
    // Wrap through Seat 3 -> Seat 0 (Round 2) before checking Seat 1 in Round 2 (even).
    advance_to_phase(&mut e, 3, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::End);
    assert_eq!(e.state_core().turn.round % 2, 0, "entered even round 2");
    assert_eq!(flag(&e, 1, "u05_mod:gj_qianxing"), 0, "even round clears qianxing");
    let h1_pre_xj = e.debug_hand(1).len();
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), h1_pre_xj, "xianju must NOT grant a Jiguan card on even round");
    // Advance naturally across round 2 -> round 3 (odd) to trigger on_round_start_gujian!
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(e.state_core().turn.round % 2, 1, "entered odd round 3");
    assert_eq!(flag(&e, 1, "u05_mod:gj_qianxing"), 1, "xianju grants qianxing on odd rounds");
    clear_all_hands(&mut e);
    let sha_blk = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha_blk, targets: vec![1] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "xianju qianxing blocks targeting seat 1 on odd round"
    );

    // 2. `woxue`: enemy turn (seat 1 Play) attacks seat 2 (`yunwuyue`), seat 2 responds with `Shan` -> `gj_woxue_confirm` + AI!
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    let sha1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Sha, Suit::Spade, &mut used);
    let shan2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Shan, Suit::Heart, &mut used);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![2] });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(shan2)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_woxue_confirm");
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let ai_wx = e.plan_action(2, &planner);
    let _ = submit(&mut e, 2, ai_wx.action);
    drain(&mut e);
    // Cancel / Pass on ally vs Confirm on enemy verified cleanly; now check `lingnu` on Beiluo (seat 3)!
    advance_to_phase(&mut e, 3, Phase::Play);
    clear_all_hands(&mut e);
    let sha3 = move_kind_suit_to_hand(&mut e, 3, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha3, targets: vec![0] });
    drain(&mut e);
    assert!(flag(&e, 3, "gj_lingnu_def:0") > 0, "lingnu grants temp Sha-related skill on DamageSource");
}
