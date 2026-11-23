//! U05-2 Round 3 parity counterexample regression suite (board #23).
//!
//! Covers all 8 + 1 Round 3 findings using real event flows (`PlayCard`/`ActivateSkill`/`Answer`/`NextPhase`),
//! public-info AI (`BoundedPlanner`), save/restore roundtrips, and zone conservation:
//! 1. `humeng`: `gjqt_yanjiaxieyi` SubPlayer lifecycle (`xianju`, 2 HP, 2 hs; exits on death to `gjqt_chuqi` with stashed 4 cards & `xuanci`).
//! 2. `yange`: SubPlayer exits at `TurnEndWindow { after: true }` (`phaseAfter`) BEFORE `lianjing` and AFTER copied `phaseEnd` stock skills (`biyue`); `clone_card_like` instantiates matching card definitions without dropping unique equips or substituting unrelated cards.
//! 3. `yanjia`: merged non-skill `c2` equips (`Zhuge`, `Qinggang`, `Renwang`, `Bagua`, `Cixiong`, `Qinglong`, `Guanshi`, `Fangtian`, `Zhangba`) work across kernel/engine.
//! 4. `mapodoufu`: targets enemies (`!is_friend_of`) only and skips allies.
//! 5. `yuling`: `gameDrawAfter` opening refill to 5 cards and `yuling`/`yuling3` non-draw `CardsGiven` (`gainEnd`) discard to 5.
//! 6. `xianju`: no `xianju_gain` / no even-round jiguan gain at `roundStart` or `TurnEndWindow`.
//! 7. `dinvxuanshuang` & `tanhuadong`: `dinvxuanshuang` rejected in `Phase::Play` (`WrongPhase`) while savable in `Dying`; `tanhuadong` ticks countdown at `phaseEnd` even when `Phase::Play` is skipped.
//! 8. `mingkong`/`mingkong2` & `duhun`: `mingkong` triggers on `DamageBegin` (after `DamageBefore`), `mingkong2` draws on `DamageEnd` / `DamageZero` (not on cancelled damage); `duhun` clamps target HP to `max_hp`.

use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase, RejectReason};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn play_yanjiadan_choose_any(e: &mut engine_native::Engine, owner: u8, card: u32) {
    let out = submit(e, owner, GameAction::PlayCard { card, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(ch) = e.open_choice().cloned() {
        if !ch.spec.prompt.starts_with("gj_yanjiadan") {
            break;
        }
        let opt = ch.spec.options.first().cloned().expect("valid yanjiadan option");
        let _ = answer_choice(e, vec![opt]);
    }
    drain(e);
}

#[test]
fn round3_defect1_humeng_subplayer_lifecycle() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9301, 4);
    e.assign_general_named(0, "gjqt_xieyi").unwrap();
    e.assign_general_named(1, "gjqt_beiluo").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // Unlock all 4 `yanjiadan_*` suits via `lingyan` (distinct ranks required!) + PlayCard -> sets `gj_humeng_sub == 1`.
    let mut used = Vec::new();
    let mut used_ranks = Vec::new();
    let sk_ly = e.catalog().skill_by_name("lingyan").unwrap();
    for (i, suit) in [Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade].into_iter().enumerate() {
        if i > 0 {
            let _ = submit(&mut e, 0, GameAction::NextPhase);
            drain(&mut e);
            advance_to_phase(&mut e, 0, Phase::Play);
            clear_all_hands(&mut e);
        }
        e.debug_set_hp(0, 3);
        let dummy_eq = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Zhuge);
        let dummy_hand = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Sha && d.suit != suit);
        e.state_core_mut().deck.retain(|&x| x != dummy_eq);
        if !e.state_core().player(1).equips.contains(&dummy_eq) {
            e.state_core_mut().player_mut(1).equips.push(dummy_eq);
        }
        move_card_to_hand(&mut e, 1, dummy_hand);
        let c = find_unused_card_by(&e, &used, |d| d.suit == suit && !d.off_deck && !used_ranks.contains(&d.rank));
        used.push(c);
        used_ranks.push(e.catalog().cards[c as usize].rank);
        move_card_to_hand(&mut e, 0, c);
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_ly });
        let _ = answer_choice(&mut e, vec![ChoiceOption::Card(c)]);
        let yj_name = match suit {
            Suit::Heart => "yanjiadan_heart",
            Suit::Diamond => "yanjiadan_diamond",
            Suit::Club => "yanjiadan_club",
            Suit::Spade => "yanjiadan_spade",
        };
        let yj = find_card_by(&e, |_, d| d.name == yj_name);
        play_yanjiadan_choose_any(&mut e, 0, yj);
    }
    assert_eq!(flag(&e, 0, "gj_humeng_sub"), 1, "all 4 yanjiadan suits unlock humeng_sub");

    // Seat 1 kills Xieyi (hp=1 -> dying) -> triggers `humeng`, draws 4 onto stashed `gjqt_chuqi`, calls SubPlayer `gjqt_yanjiaxieyi` (2 HP, 2 hs, `xianju`)!
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    e.state_core_mut().player_mut(0).hujia = 0;
    e.debug_set_hp(0, 1);
    let sha1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    drain(&mut e);

    assert!(e.state_core().player(0).alive, "humeng prevents death and enters SubPlayer");
    let sub_gid = e.state_core().player(0).general_slots[0].expect("subplayer general");
    assert_eq!(
        e.catalog().general_meta(sub_gid).map(|m| m.0),
        Some("gjqt_yanjiaxieyi"),
        "active general during SubPlayer must be gjqt_yanjiaxieyi"
    );
    assert_eq!(e.state_core().player(0).max_hp, 2);
    assert_eq!(e.debug_hp(0), 2);
    assert_eq!(e.debug_hand(0).len(), 2, "gjqt_yanjiaxieyi starts with 2 hand cards");
    assert!(e.debug_has_skill(0, "xianju"), "gjqt_yanjiaxieyi has xianju");
    assert!(!e.debug_has_skill(0, "xuanci"), "chuqi's xuanci must stay stashed while in SubPlayer");
    assert_zone_conservation_all(&e);

    // Save/restore while SubPlayer is active.
    let mut r = save_restore_verified(&e, &verified);
    assert_eq!(r.debug_hand(0).len(), 2);
    assert!(r.debug_has_skill(0, "xianju"));

    // Kill SubPlayer `gjqt_yanjiaxieyi` via seat 1 (distance 1) -> Death exits SubPlayer cleanly into stashed `gjqt_chuqi`!
    r.debug_set_hp(0, 1);
    clear_all_hands(&mut r);
    // Ensure round 2 so `xianju` odd-round `qianxing` does not block targeting seat 0.
    advance_to_phase(&mut r, 1, Phase::Play);
    if r.state_core().turn.round % 2 == 1 {
        advance_to_phase(&mut r, 2, Phase::Play);
        advance_to_phase(&mut r, 1, Phase::Play);
    }
    clear_all_hands(&mut r);
    let _extra_card = move_kind_suit_to_hand(&mut r, 0, CardKind::Sha, Suit::Diamond, &mut used);
    let kill_sub = move_kind_suit_to_hand(&mut r, 1, CardKind::Sha, Suit::Club, &mut used);
    let out_kill = submit(&mut r, 1, GameAction::PlayCard { card: kill_sub, targets: vec![0] });
    assert!(!matches!(out_kill, CommandOutcome::Rejected(_)), "seat 1 kill Sha rejected: {out_kill:?}");
    drain(&mut r);

    assert!(r.state_core().player(0).alive, "dying in SubPlayer exits SubPlayer keeping seat 0 alive");
    let main_gid = r.state_core().player(0).general_slots[0].expect("restored general");
    assert_eq!(
        r.catalog().general_meta(main_gid).map(|m| m.0),
        Some("gjqt_chuqi"),
        "exiting SubPlayer restores gjqt_chuqi"
    );
    assert_eq!(r.state_core().player(0).max_hp, 2);
    assert_eq!(r.debug_hp(0), 2);
    assert!(r.debug_has_skill(0, "xuanci"), "restored gjqt_chuqi has xuanci");
    assert!(!r.debug_has_skill(0, "xianju"), "xianju removed upon SubPlayer exit");
    assert_eq!(r.debug_hand(0).len(), 4, "restored gjqt_chuqi receives stashed 4 cards from humeng");
    assert_zone_conservation_all(&r);
}

#[test]
fn round3_defect2_yange_timing_and_definition_cloning() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9302, 4);
    e.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e.assign_general_named(1, "貂蝉").unwrap();
    e.assign_general_named(2, "gjqt_bailitusu").unwrap();

    // Put unique `Renwang` (only 1 copy in catalog!) into seat 1 equips, and `Tao` into seat 1 hand,
    // moving ALL other `Tao`s in catalog out of deck/discard into seat 3 off_court so 0 `Renwang` and 0 `Tao` exist in deck/discard!
    let rw = find_card_by(&e, |_, d| d.kind == CardKind::Renwang);
    let all_taos: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, d)| d.kind == CardKind::Tao)
        .map(|(i, _)| i as u32)
        .collect();
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    for &t in &all_taos {
        move_card_to_hand(&mut e, 3, t);
    }
    let seat1_tao = all_taos[0];
    move_card_to_hand(&mut e, 1, rw);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: rw, targets: vec![] });
    move_card_to_hand(&mut e, 1, seat1_tao);
    // Keep other Taos in seat 3 off_court so deck & discard contain ZERO Renwang and ZERO Tao!
    for &t in &all_taos[1..] {
        e.state_core_mut().player_mut(3).hand.retain(|&c| c != t);
        e.state_core_mut().player_mut(3).off_court.push(t);
    }
    e.debug_clear_hand(3);

    // Put a Le in seat 0 judge zone to verify SubPlayer preserves judge zone.
    let le0 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    {
        let core = e.state_core_mut();
        core.deck.retain(|&c| c != le0);
        core.discard.retain(|&c| c != le0);
        for p in 0..4 {
            core.player_mut(p).hand.retain(|&c| c != le0);
        }
        core.player_mut(0).judge.push(le0);
    }

    advance_to_phase(&mut e, 3, Phase::End);
    clear_all_hands(&mut e);
    // Restore seat 1's hand (only seat1_tao) cleanly since clear_all_hands cleared it.
    move_card_to_hand(&mut e, 1, seat1_tao);
    let mut used = vec![rw, le0];
    used.extend(all_taos.iter().copied());
    let orig_sha = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);

    let _ = submit(&mut e, 3, GameAction::NextPhase); // wraps into seat 0 TurnBegin -> yange!
    drain_until_prompt(&mut e, "gj_yange_target");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_yange_target");
    // Place 4 Heart cards on top of deck BEFORE answering `gj_yange_target`, because cloning
    // may pop up to 2 carriers off deck top before `Phase::Judge` judges `le0`!
    let heart_cards: Vec<u32> = e
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&c| e.catalog().cards[c as usize].suit == Suit::Heart)
        .take(4)
        .collect();
    assert!(!heart_cards.is_empty());
    for &hc in &heart_cards {
        e.state_core_mut().deck.retain(|&c| c != hc);
        e.state_core_mut().deck.push(hc);
    }
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(1)]);

    // Cloned state check: seat 0 is Diaochan SubPlayer with 1 Renwang-equivalent equip & 1 Tao-equivalent hand card;
    // `le0` must be preserved across `yange` entry (still in `judge` during Prepare or judged cleanly into discard by Judge)!
    assert!(e.debug_has_skill(0, "闭月"), "yange cloned Diaochan's 闭月");
    assert!(!e.debug_has_skill(0, "lianjing"), "main player's lianjing stashed while in SubPlayer");
    assert!(
        e.state_core().player(0).judge == vec![le0]
            || (e.state_core().discard.contains(&le0)
                && heart_cards.iter().any(|hc| e.state_core().discard.contains(hc))),
        "judge card le0 must be preserved across yange entry (phase={:?}, judge={:?}, le_in_discard={})",
        e.turn().phase,
        e.state_core().player(0).judge,
        e.state_core().discard.contains(&le0),
    );
    assert_eq!(e.state_core().player(0).equips.len(), 1, "unique Renwang cloned rather than dropped to None");
    assert_eq!(e.debug_hand(0).len(), 1, "cloned 1 Tao card");

    // Verify cloned hand card actually works as `Tao` (heals wounded owner in Play phase)!
    drain(&mut e);
    if e.turn().phase == Phase::Prepare {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Judge);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw (Le Heart passed -> Play NOT skipped!)
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play
    drain(&mut e);
    e.debug_set_hp(0, 2); // wounded (max 3) so cloned Tao is playable!
    let cloned_hand_c = e.debug_hand(0)[0];
    let hp_pre_tao = e.debug_hp(0);
    let out_tao = submit(&mut e, 0, GameAction::PlayCard { card: cloned_hand_c, targets: vec![] });
    assert!(!matches!(out_tao, CommandOutcome::Rejected(_)), "cloned hand card must be playable as Tao: {out_tao:?}");
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_pre_tao + 1, "cloned Tao healed +1 HP");

    // Re-place `le0` in seat 0's judge zone during Play to verify `yange_exit` also preserves judge zone.
    e.state_core_mut().discard.retain(|&c| c != le0);
    e.state_core_mut().player_mut(0).judge.push(le0);

    // Advance Play -> Discard -> End -> TurnEndWindow:
    // (1) Phase::End triggers Diaochan's `闭月` while STILL in SubPlayer (Phase::End does NOT exit `yange`);
    // (2) TurnEndWindow { after: false } (`phaseEnd`) also runs before `yange_exit`.
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End -> asks `闭月` trigger_confirm!
    let mut saw_biyue_during_subplayer = false;
    for _ in 0..8 {
        let Some(ch) = e.open_choice().cloned() else { break };
        if ch.spec.prompt == "trigger_confirm" {
            saw_biyue_during_subplayer = true;
            assert!(e.debug_has_skill(0, "闭月"));
            assert!(!e.debug_has_skill(0, "lianjing"), "yange must stay active during Phase::End 闭月");
            let mut r_mid = save_restore_verified(&e, &verified);
            let _ = answer_choice(&mut r_mid, vec![ChoiceOption::Confirm]);
            e = r_mid;
            break;
        }
        let _ = answer_choice(&mut e, vec![ChoiceOption::Pass]);
    }
    assert!(saw_biyue_during_subplayer, "copied general's Phase::End skill `闭月` must trigger before `yange_exit`");
    drain(&mut e);
    assert!(e.debug_has_skill(0, "闭月"), "Phase::End completion must not exit yange before phaseEnd");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow { after: false } -> { after: true } (`yange_exit`) -> `lianjing`!

    // Now `TurnEndWindow { after: true }` (`phaseAfter`) exits SubPlayer (`yange_exit`) BEFORE `lianjing` opens!
    drain_until_prompt(&mut e, "gj_lianjing_targets");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_lianjing_targets", "yange_exit restores main player so lianjing prompts at phaseAfter");
    assert!(e.debug_has_skill(0, "woxue"), "main player skills restored before lianjing");
    assert!(!e.debug_has_skill(0, "闭月"), "cloned skills removed by yange_exit");
    assert!(e.debug_hand(0).contains(&orig_sha), "main player hand restored");
    let _ = answer_choice(&mut e, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).judge.len(), 1, "judge card preserved after yange_exit");
    assert_zone_conservation_all(&e);

    // Also verify a cloned unique Weapon (`Qinggang`, with all copies out of deck/discard!) actually
    // works in real combat (piercing Seat 2's `Bagua` without triggering `ArmorJudged`) while in SubPlayer!
    let (_, mut e_rw) = gujian_verified_and_engine(2026_9303, 3);
    e_rw.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e_rw.assign_general_named(1, "貂蝉").unwrap();
    let all_qg: Vec<u32> = e_rw
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, d)| d.kind == CardKind::Qinggang)
        .map(|(i, _)| i as u32)
        .collect();
    let qg1 = all_qg[0];
    let bg_s2 = find_unused_card_by(&e_rw, &all_qg, |d| d.kind == CardKind::Armor && d.name == "armor");
    advance_to_phase(&mut e_rw, 1, Phase::Play);
    clear_all_hands(&mut e_rw);
    move_card_to_hand(&mut e_rw, 1, qg1);
    let _ = submit(&mut e_rw, 1, GameAction::PlayCard { card: qg1, targets: vec![] });
    // Stash any extra Qinggang copies in seat 1 off_court so deck & discard contain ZERO Qinggang!
    for &extra_qg in &all_qg[1..] {
        move_card_to_hand(&mut e_rw, 1, extra_qg);
        e_rw.state_core_mut().player_mut(1).hand.retain(|&c| c != extra_qg);
        e_rw.state_core_mut().player_mut(1).off_court.push(extra_qg);
    }
    advance_to_phase(&mut e_rw, 2, Phase::Play);
    clear_all_hands(&mut e_rw);
    move_card_to_hand(&mut e_rw, 2, bg_s2);
    let _ = submit(&mut e_rw, 2, GameAction::PlayCard { card: bg_s2, targets: vec![] });
    advance_to_phase(&mut e_rw, 2, Phase::End);
    let _ = submit(&mut e_rw, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e_rw, "gj_yange_target");
    let _ = answer_choice(&mut e_rw, vec![ChoiceOption::Player(1)]);
    assert_eq!(e_rw.state_core().player(0).equips.len(), 1);
    advance_to_phase(&mut e_rw, 0, Phase::Play);
    clear_all_hands(&mut e_rw);
    let mut u2 = all_qg;
    u2.push(bg_s2);
    let sha_qg = move_kind_suit_to_hand(&mut e_rw, 0, CardKind::Sha, Suit::Spade, &mut u2);
    let hp2_pre = e_rw.debug_hp(2);
    let _ = submit(&mut e_rw, 0, GameAction::PlayCard { card: sha_qg, targets: vec![2] });
    drain(&mut e_rw);
    assert!(
        !e_rw.log().iter().any(|l| matches!(l, LogEvent::ArmorJudged { player: 2, .. })),
        "cloned Qinggang must pierce seat 2's Bagua without ArmorJudged"
    );
    assert_eq!(e_rw.debug_hp(2), hp2_pre - 1, "cloned Qinggang dealt 1 damage through Bagua");
    assert_zone_conservation_all(&e_rw);

    // Also verify a copied `TurnPhaseEnd` (`phaseEnd`) skill (`fanyin` on `gjqt_fanglansheng`) STILL fires
    // in `TurnEndWindow { after: false }` BEFORE `yange_exit` (`TurnPhaseAfter`) exits SubPlayer!
    let (_, mut e_pe) = gujian_verified_and_engine(2026_9313, 3);
    e_pe.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e_pe.assign_general_named(1, "gjqt_fanglansheng").unwrap();
    advance_to_phase(&mut e_pe, 2, Phase::End);
    clear_all_hands(&mut e_pe);
    e_pe.state_core_mut().player_mut(1).linked = true;
    let _ = submit(&mut e_pe, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e_pe, "gj_yange_target");
    let _ = answer_choice(&mut e_pe, vec![ChoiceOption::Player(1)]);
    assert!(e_pe.debug_has_skill(0, "fanyin"));
    advance_to_phase(&mut e_pe, 0, Phase::End);
    let _ = submit(&mut e_pe, 0, GameAction::NextPhase); // End -> TurnEndWindow { after: false } -> asks `gj_fanyin_target`!
    assert_eq!(
        e_pe.open_choice().expect("copied fanyin must prompt at phaseEnd").spec.prompt,
        "gj_fanyin_target",
        "copied phaseEnd skill fanyin must fire BEFORE yange_exit"
    );
    assert!(!e_pe.debug_has_skill(0, "lianjing"), "yange still active during phaseEnd fanyin");
    let _ = answer_choice(&mut e_pe, vec![]);
    drain_until_prompt(&mut e_pe, "gj_lianjing_targets");
    assert_eq!(e_pe.open_choice().unwrap().spec.prompt, "gj_lianjing_targets", "yange_exit runs at phaseAfter before lianjing");
    assert!(!e_pe.debug_has_skill(0, "fanyin"), "fanyin stripped by yange_exit");
    drain(&mut e_pe);
    assert_zone_conservation_all(&e_pe);
}

#[test]
fn round3_defect3_yanjia_merged_non_skill_equips() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9304, 3);
    e.assign_general_named(0, "gjqt_yuewuyi").unwrap();
    e.assign_general_named(1, "貂蝉").unwrap(); // female for Cixiong test
    e.assign_general_named(2, "gjqt_bailitusu").unwrap(); // male
    let sk_yj = e.catalog().skill_by_name("yanjia").unwrap();

    // Part A: Merge `Renwang` (c1) + `Zhuge` (c2) -> equip merged card -> unlimited Sha!
    // Also test `Bagua` (c1) + `Qinggang` (c2) -> ignores defender Renwang!
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let rw = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Renwang);
    used.push(rw);
    let zhuge = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Zhuge);
    used.push(zhuge);
    move_card_to_hand(&mut e, 0, rw);
    move_card_to_hand(&mut e, 0, zhuge);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yj });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(rw)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(zhuge)]);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: rw, targets: vec![] });
    let sha_a = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    let sha_b = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Diamond, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    drain(&mut e);
    let out_2nd = submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
    assert!(
        !matches!(out_2nd, CommandOutcome::Rejected(_)),
        "c2=Zhuge merged onto c1=Renwang must allow 2nd Sha: {out_2nd:?}"
    );
    drain(&mut e);

    // Part B: Merge `Zhuge` (c1) + `Renwang` (c2) on seat 2 (Wait: merge on seat 0 and equip on seat 1? Or `yanjia` on seat 0 gives Bagua+Qinggang on seat 0, and seat 1 tests Bagua/Renwang!)
    // Put Bagua (c1) + Qinggang (c2) merged by seat 0, while seat 0 ALSO equips Fangtian (c1) + Cixiong (c2) and Guanshi (c1) + Qinglong (c2)!
    // Specifically test EACH of the 9 non-skill equips as `c2`:
    // 1. c2 = Renwang (merged onto Bagua -> `c1 = Bagua, c2 = Renwang`): black Sha from seat 1 is negated!
    clear_all_hands(&mut e);
    let bagua = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Armor && d.name == "armor");
    used.push(bagua);
    let qinggang = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Qinggang);
    used.push(qinggang);
    move_card_to_hand(&mut e, 0, bagua);
    move_card_to_hand(&mut e, 0, qinggang);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_yj });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(bagua)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(qinggang)]);
    // Equip bagua_qinggang on seat 0: replaces renwang_zhuge cleanly, grants BOTH bagua armor & qinggang pierce!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bagua, targets: vec![] });
    // Put a second Bagua on seat 1 so we verify c2=Qinggang pierces seat 1's Bagua!
    let bagua2 = find_unused_card_by(&e, &used, |d| d.kind == CardKind::Armor && d.name == "armor");
    used.push(bagua2);
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    move_card_to_hand(&mut e, 1, bagua2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: bagua2, targets: vec![] });
    // Also merge Zhuge2 + Renwang on seat 0? First test Bagua2 is pierced by seat 0's c2=Qinggang!
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let sha_p = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let hp1_b = e.debug_hp(1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_p, targets: vec![1] });
    while let Some(c) = e.open_choice().cloned() {
        if matches!(c.spec.prompt, "gj_liuying_target" | "gj_xiuhua_gain") {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                vec![]
            };
            let _ = answer_choice(&mut e, sel);
        } else {
            break;
        }
    }
    assert!(
        e.open_choice().is_none(),
        "c2=Qinggang must pierce seat 1's Bagua without judge prompt, got: {:?}",
        e.open_choice().map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert!(
        !e.log().iter().any(|l| matches!(l, LogEvent::ArmorJudged { player: 1, .. })),
        "c2=Qinggang must prevent ArmorJudged on seat 1 Bagua"
    );
    assert_eq!(e.debug_hp(1), hp1_b - 1, "c2=Qinggang dealt damage through armor");
    let _ = save_restore_verified(&e, &verified);

    // Fresh engine verifying c2 = Renwang, Bagua, Cixiong, Qinglong, Guanshi, Fangtian, Zhangba!
    let (_, mut e2) = gujian_verified_and_engine(2026_9305, 3);
    e2.assign_general_named(0, "gjqt_yuewuyi").unwrap();
    e2.assign_general_named(1, "貂蝉").unwrap(); // female
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_all_hands(&mut e2);
    let mut u2 = Vec::new();
    let sk_yj2 = e2.catalog().skill_by_name("yanjia").unwrap();

    // c1 = Zhuge, c2 = Cixiong -> equip -> Sha on female seat 1 triggers `cixiong_discard_choose`!
    let zg1 = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Zhuge);
    u2.push(zg1);
    let cx = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Cixiong);
    u2.push(cx);
    move_card_to_hand(&mut e2, 0, zg1);
    move_card_to_hand(&mut e2, 0, cx);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: sk_yj2 });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(zg1)]);
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(cx)]);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: zg1, targets: vec![] });
    let sh1_cx = move_kind_suit_to_hand(&mut e2, 1, CardKind::Shan, Suit::Heart, &mut u2);
    let sha_cx = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Heart, &mut u2);
    let _ = sh1_cx;
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_cx, targets: vec![1] });
    drain_until_prompt(&mut e2, "cixiong_discard_choose");
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "cixiong_discard_choose", "c2=Cixiong triggers vs female target");
    drain(&mut e2);

    // c1 = Zhuge2, c2 = Fangtian -> last hand Sha targets BOTH seat 1 & seat 2!
    clear_all_hands(&mut e2);
    let zg2 = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Zhuge);
    u2.push(zg2);
    let ft = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Fangtian);
    u2.push(ft);
    move_card_to_hand(&mut e2, 0, zg2);
    move_card_to_hand(&mut e2, 0, ft);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: sk_yj2 });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(zg2)]);
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(ft)]);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: zg2, targets: vec![] });
    let last_sha = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Spade, &mut u2);
    let out_ft = submit(&mut e2, 0, GameAction::PlayCard { card: last_sha, targets: vec![1, 2] });
    assert!(!matches!(out_ft, CommandOutcome::Rejected(_)), "c2=Fangtian must allow 2+ targets on last Sha: {out_ft:?}");
    drain(&mut e2);

    // c1 = Hanbing, c2 = Qinglong -> dodged Sha prompts `qinglong_replay_sha`!
    clear_all_hands(&mut e2);
    let hb = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Hanbing);
    u2.push(hb);
    let ql = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Qinglong);
    u2.push(ql);
    move_card_to_hand(&mut e2, 0, hb);
    move_card_to_hand(&mut e2, 0, ql);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: sk_yj2 });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(hb)]);
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(ql)]);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: hb, targets: vec![] });
    e2.debug_set_hp(1, 3);
    let sha_ql1 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Spade, &mut u2);
    let sha_ql2 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Club, &mut u2);
    let shan_ql = move_kind_suit_to_hand(&mut e2, 1, CardKind::Shan, Suit::Diamond, &mut u2);
    e2.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_ql1, targets: vec![1] });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(shan_ql)]);
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "qinglong_replay_sha", "c2=Qinglong must prompt replay after dodged Sha");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(sha_ql2)]);
    drain(&mut e2);

    // c1 = Qilin, c2 = Guanshi -> dodged Sha prompts `guanshi_force_damage` AND excludes merged Qilin_Guanshi from discard options!
    clear_all_hands(&mut e2);
    let qilin = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Qilin);
    u2.push(qilin);
    let gs = find_unused_card_by(&e2, &u2, |d| d.kind == CardKind::Guanshi);
    u2.push(gs);
    move_card_to_hand(&mut e2, 0, qilin);
    move_card_to_hand(&mut e2, 0, gs);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: sk_yj2 });
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(qilin)]);
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(gs)]);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: qilin, targets: vec![] });
    e2.debug_set_hp(1, 3);
    let sha_gs = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Spade, &mut u2);
    let f1 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Tao, Suit::Heart, &mut u2);
    let f2 = move_kind_suit_to_hand(&mut e2, 0, CardKind::Sha, Suit::Diamond, &mut u2);
    let shan_gs = move_kind_suit_to_hand(&mut e2, 1, CardKind::Shan, Suit::Heart, &mut u2);
    e2.state_core_mut().turn.sha_used_play_phase = false;
    let out_gs = submit(&mut e2, 0, GameAction::PlayCard { card: sha_gs, targets: vec![1] });
    assert!(!matches!(out_gs, CommandOutcome::Rejected(_)), "sha_gs rejected: {out_gs:?}, p1_alive={}", e2.state_core().player(1).alive);
    drain_until_prompt(&mut e2, "respond_shan");
    assert_eq!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("respond_shan"),
        "expected respond_shan after PlayCard(sha_gs), out={out_gs:?}, shan_in_h1={}",
        e2.debug_hand(1).contains(&shan_gs)
    );
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(shan_gs)]);
    drain_until_prompt(&mut e2, "guanshi_force_damage");
    let c_gs = e2.open_choice().cloned().expect("c2=Guanshi must prompt guanshi_force_damage");
    assert_eq!(c_gs.spec.prompt, "guanshi_force_damage");
    assert!(!c_gs.spec.options.contains(&ChoiceOption::Card(qilin)), "merged qilin_guanshi itself excluded");
    let _ = answer_choice(&mut e2, vec![ChoiceOption::Card(f1), ChoiceOption::Card(f2)]);
    drain(&mut e2);

    // Fresh engine verifying c2 = Renwang, Bagua, Zhangba!
    let (_, mut e3) = gujian_verified_and_engine(2026_9306, 3);
    e3.assign_general_named(0, "gjqt_yuewuyi").unwrap();
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let mut u3 = Vec::new();
    let sk_yj3 = e3.catalog().skill_by_name("yanjia").unwrap();
    let hm3 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::HorseMinus);
    u3.push(hm3);
    let rw3 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Renwang);
    u3.push(rw3);
    let zg3 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Zhuge);
    u3.push(zg3);
    let zb3 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Zhangba);
    u3.push(zb3);
    move_card_to_hand(&mut e3, 0, hm3);
    move_card_to_hand(&mut e3, 0, rw3);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: sk_yj3 });
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(hm3)]);
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(rw3)]);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: hm3, targets: vec![] });
    move_card_to_hand(&mut e3, 0, zg3);
    move_card_to_hand(&mut e3, 0, zb3);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: sk_yj3 });
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(zg3)]);
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(zb3)]);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: zg3, targets: vec![] });
    assert!(e3.debug_has_skill(0, "丈八蛇矛"), "c2=Zhangba grants 丈八蛇矛 skill");
    // Use c2=Zhangba to convert 2 cards into Sha against seat 1!
    let zc1 = move_kind_suit_to_hand(&mut e3, 0, CardKind::Shan, Suit::Heart, &mut u3);
    let zc2 = move_kind_suit_to_hand(&mut e3, 0, CardKind::Shan, Suit::Diamond, &mut u3);
    let sk_zb = e3.catalog().skill_by_name("丈八蛇矛").unwrap();
    let hp1_zb = e3.debug_hp(1);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: sk_zb });
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(zc1), ChoiceOption::Card(zc2)]);
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Player(1)]);
    drain(&mut e3);
    assert_eq!(e3.debug_hp(1), hp1_zb - 1, "c2=Zhangba dealt 1 Sha damage");

    // Verify c2=Renwang on seat 0 negates black Sha from seat 1 (even with Shan in seat 0 hand, no respond_shan prompts)!
    advance_to_phase(&mut e3, 1, Phase::Play);
    clear_all_hands(&mut e3);
    let shan_decoy = move_kind_suit_to_hand(&mut e3, 0, CardKind::Shan, Suit::Diamond, &mut u3);
    let _ = shan_decoy;
    let blk_sha3 = move_kind_suit_to_hand(&mut e3, 1, CardKind::Sha, Suit::Spade, &mut u3);
    let hp0_rw3 = e3.debug_hp(0);
    let out_rw3 = submit(&mut e3, 1, GameAction::PlayCard { card: blk_sha3, targets: vec![0] });
    assert!(!matches!(out_rw3, CommandOutcome::Rejected(_)), "blk_sha3 rejected: {out_rw3:?}");
    assert!(e3.open_choice().is_none(), "c2=Renwang negated black Sha without prompting respond_shan: {:?}", e3.open_choice().map(|c| c.spec.prompt));
    drain(&mut e3);
    assert_eq!(e3.debug_hp(0), hp0_rw3, "c2=Renwang negated black Sha");

    // Swap seat 0 to Bagua (c1=HorseMinus2, c2=Bagua) and verify red Sha triggers Bagua Heart judge!
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_all_hands(&mut e3);
    let hm_bg = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::HorseMinus);
    u3.push(hm_bg);
    let bg3 = find_unused_card_by(&e3, &u3, |d| d.kind == CardKind::Armor && d.name == "armor");
    u3.push(bg3);
    move_card_to_hand(&mut e3, 0, hm_bg);
    move_card_to_hand(&mut e3, 0, bg3);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: sk_yj3 });
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(hm_bg)]);
    let _ = answer_choice(&mut e3, vec![ChoiceOption::Card(bg3)]);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: hm_bg, targets: vec![] });
    advance_to_phase(&mut e3, 1, Phase::Play);
    clear_all_hands(&mut e3);
    let red_sha3 = move_kind_suit_to_hand(&mut e3, 1, CardKind::Sha, Suit::Heart, &mut u3);
    let j_heart = *e3.state_core().deck.iter().find(|&&c| e3.catalog().cards[c as usize].suit == Suit::Heart).unwrap();
    e3.state_core_mut().deck.retain(|&c| c != j_heart);
    e3.state_core_mut().deck.push(j_heart);
    let hp0_bg3 = e3.debug_hp(0);
    let out_bg3 = submit(&mut e3, 1, GameAction::PlayCard { card: red_sha3, targets: vec![0] });
    assert!(!matches!(out_bg3, CommandOutcome::Rejected(_)), "red_sha3 rejected: {out_bg3:?}");
    drain(&mut e3);
    assert!(
        e3.log().iter().any(|l| matches!(l, LogEvent::ArmorJudged { player: 0, success: true, .. })),
        "c2=Bagua must trigger ArmorJudged on defender"
    );
    assert_eq!(e3.debug_hp(0), hp0_bg3, "c2=Bagua dodged red Sha via Heart judge");
    assert_zone_conservation_all(&e3);
}

#[test]
fn round3_defect4_mapodoufu_enemies_only() {
    let (_, mut e) = gujian_verified_and_engine(2026_9307, 4);
    e.assign_general_named(0, "gjqt_yunwuyue").unwrap(); // Wei
    e.assign_general_named(1, "gjqt_fengqingxue").unwrap(); // Wu (enemy)
    e.assign_general_named(2, "曹操").unwrap(); // Wei (ally!)
    e.assign_general_named(3, "刘备").unwrap(); // Shu (enemy)

    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let mp = move_named_to_hand(&mut e, 0, "mapodoufu");
    let ally_card = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: mp, targets: vec![0] });
    drain(&mut e);
    advance_to_phase(&mut e, 0, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert!(e.debug_hand(2).contains(&ally_card), "mapodoufu must skip ally seat 2 when no enemy has cards");

    // Next turn: give enemy seat 1 a card, keep ally seat 2 holding ally_card -> discards from enemy seat 1!
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_clear_hand(0);
    let enemy_card = move_kind_suit_to_hand(&mut e, 1, CardKind::Shan, Suit::Heart, &mut used);
    advance_to_phase(&mut e, 0, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert!(e.debug_hand(2).contains(&ally_card), "ally seat 2 still untouched");
    assert!(!e.debug_hand(1).contains(&enemy_card), "enemy seat 1 discarded by mapodoufu");
    assert_zone_conservation_all(&e);
}

#[test]
fn round3_defect5_yuling_opening_refill_and_non_draw_gain() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9308, 4);
    e.assign_general_named(0, "gjqt_aruan").unwrap();
    e.assign_general_named(1, "刘备").unwrap();

    // 5a. `gameDrawAfter`: starting Aruan refills from 4 opening cards to 5!
    assert_eq!(e.debug_hand(0).len(), 5, "yuling2 gameDrawAfter must refill Aruan to 5 cards at start");

    // 5b. Non-draw `gainEnd` (`CardsGiven` from Liubei's `rende`): Aruan goes 5 -> 6 and `yuling`/`yuling3` forces discard back to 5!
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(1);
    let mut used = Vec::new();
    let gift = move_kind_suit_to_hand(&mut e, 1, CardKind::Sha, Suit::Spade, &mut used);
    let sk_rd = e.catalog().skill_by_name("仁德").unwrap();
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: sk_rd });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(gift)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(0)]);
    let ch = e.open_choice().cloned().expect("yuling3 discard prompt on non-draw CardsGiven");
    assert_eq!(ch.spec.prompt, "gj_yuling3_discard");
    assert_eq!(ch.observer, 0);
    let mut r = save_restore_verified(&e, &verified);
    let pick = r.open_choice().unwrap().spec.options[0];
    let _ = answer_choice(&mut r, vec![pick]);
    drain(&mut r);
    assert_eq!(r.debug_hand(0).len(), 5, "yuling3 discarded back down to 5 cards after non-draw gainEnd");
    assert_zone_conservation_all(&r);
}

#[test]
fn round3_defect6_xianju_no_even_round_gain() {
    let (_, mut e) = gujian_verified_and_engine(2026_9309, 4);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e.assign_general_named(1, "gjqt_yanjiaxieyi").unwrap();
    assert!(!e.debug_has_skill(1, "xianju_gain"), "no xianju_gain skill on gjqt_yanjiaxieyi");

    // Advance naturally to Round 2 (even), clear seat 1 hand, and end seat 1's turn: ZERO cards gained!
    advance_to_phase(&mut e, 3, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::End);
    assert_eq!(e.state_core().turn.round % 2, 0);
    e.debug_clear_hand(1);
    assert_eq!(flag(&e, 1, "u05_mod:gj_qianxing"), 0, "qianxing cleared on even roundStart");
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "xianju must NOT grant any jiguan card on even rounds");

    // Odd round 3 grants qianxing again.
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(e.state_core().turn.round % 2, 1);
    assert_eq!(flag(&e, 1, "u05_mod:gj_qianxing"), 1, "xianju restores qianxing on odd roundStart");
}

#[test]
fn round3_defect7_dinvxuanshuang_play_rejected_and_tanhuadong_phase_end_tick() {
    let (_, mut e) = gujian_verified_and_engine(2026_9310, 3);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // 7a. `dinvxuanshuang` CANNOT be used in Phase::Play even when wounded!
    e.debug_set_hp(0, 2);
    let dvx = move_named_to_hand(&mut e, 0, "dinvxuanshuang");
    let out_dvx = submit(&mut e, 0, GameAction::PlayCard { card: dvx, targets: vec![] });
    assert!(
        matches!(out_dvx, CommandOutcome::Rejected(RejectReason::WrongPhase)),
        "dinvxuanshuang has no enable:true -> rejected in Phase::Play, got {out_dvx:?}"
    );
    assert!(
        !e.legal_actions(0).iter().any(|a| matches!(a, GameAction::PlayCard { card, .. } if *card == dvx)),
        "dinvxuanshuang must not appear in Phase::Play legal_actions"
    );

    // 7b. `tanhuadong` countdown ticks at `phaseEnd` (`Phase::End`), even when `Phase::Play` is skipped!
    let thd = move_named_to_hand(&mut e, 0, "tanhuadong");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: thd, targets: vec![0] });
    drain(&mut e);
    let def_td = e.catalog().skill_by_name(engine_native::content::host::GJ_FOOD_TANHUADONG_KEY).unwrap();
    assert_eq!(e.state_core().skills.get(&(0, def_td)).unwrap().rounds, 3);

    // End Turn 0 cleanly (Play -> Discard -> End -> TurnEndWindow):
    // Right after Play -> Discard, drawn 1 AND rounds stays 3 until End -> TurnEndWindow!
    let h0_pre_play_end = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0_pre_play_end + 1, "tanhuadong_draw draws 1 at phaseUseEnd");
    assert_eq!(e.state_core().skills.get(&(0, def_td)).unwrap().rounds, 3, "tanhuadong does NOT tick at phaseUseEnd");
    advance_to_phase(&mut e, 0, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow
    drain(&mut e);
    assert_eq!(e.state_core().skills.get(&(0, def_td)).unwrap().rounds, 2, "tanhuadong ticks 3->2 at phaseEnd");

    // Seat 1 puts `Le` on seat 0, and Heart/non-Heart top-deck fails Le -> SKIPS seat 0's `Phase::Play`!
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = vec![dvx, thd];
    let le1 = move_kind_suit_to_hand(&mut e, 1, CardKind::Le, Suit::Spade, &mut used);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: le1, targets: vec![0] });
    pass_nullify_window(&mut e);
    let spade_top = *e.state_core().deck.iter().find(|&&c| e.catalog().cards[c as usize].suit == Suit::Spade).unwrap();
    e.state_core_mut().deck.retain(|&c| c != spade_top);
    e.state_core_mut().deck.push(spade_top);
    advance_to_phase(&mut e, 0, Phase::Judge);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw (Le Spade fails -> skip_play_phase = true!)
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> skips Play -> lands directly in Discard!
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard, "Le skipped Play phase");
    // Before phaseEnd, rounds still 2; after End -> TurnEndWindow, rounds decrements 2 -> 1!
    assert_eq!(e.state_core().skills.get(&(0, def_td)).unwrap().rounds, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow
    drain(&mut e);
    assert_eq!(e.state_core().skills.get(&(0, def_td)).unwrap().rounds, 1, "tanhuadong still ticks 2->1 at phaseEnd when Phase::Play skipped");
    assert_zone_conservation_all(&e);
}

#[test]
fn round3_defect8_mingkong_and_duhun() {
    let (_, mut e) = gujian_verified_and_engine(2026_9311, 4);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e.assign_general_named(1, "gjqt_fanglansheng").unwrap();
    e.assign_general_named(2, "gjqt_fengqingxue").unwrap(); // qinglan on DamageBefore!
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();

    // 8a. Cancelled damage at `DamageBefore` (qinglan cancels FireSha) runs BEFORE `DamageBegin` (`mingkong`):
    //     neither `mingkong` nor `mingkong2` draws!
    let fsha = find_unused_card_by(&e, &used, |d| d.kind == CardKind::FireSha);
    used.push(fsha);
    move_card_to_hand(&mut e, 0, fsha);
    let ql_card = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let h0_pre_cancel = e.debug_hand(0).len() - 1;
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_qinglan_card");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(ql_card)]);
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        h0_pre_cancel + 1,
        "cancelled FireSha draws 1 from qinglan ONLY, NOT from mingkong2"
    );
    assert!(!e.debug_has_skill(0, "mingkong2"));
    assert_eq!(flag(&e, 0, "gj_mingkong2_pending"), 0);

    // 8b. Uncancelled 1-dmg FireSha on empty-hand Fanglansheng:
    //     qinglan passes (`vec![]`), `mingkong` reduces 1->0 at `DamageBegin` -> `DamageFlow` emits `DamageZero` -> `mingkong2` triggers on `DamageZero` and source 0 draws 1!
    clear_all_hands(&mut e);
    let fsha2 = find_unused_card_by(&e, &used, |d| d.kind == CardKind::FireSha);
    used.push(fsha2);
    move_card_to_hand(&mut e, 0, fsha2);
    let ql_card2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Club, &mut used);
    let _ = ql_card2;
    let hp1_pre = e.debug_hp(1);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fsha2, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_qinglan_card");
    let _ = answer_choice(&mut e, vec![]); // pass qinglan
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_pre, "mingkong reduced 1 damage to 0");
    assert_eq!(e.debug_hand(0).len(), 1, "mingkong2 triggered on DamageZero and drew 1 for source");
    assert!(!e.debug_has_skill(0, "mingkong2"), "mingkong2 removed after firing");

    // 8c. 2-dmg (Jiu + Sha) on empty-hand Fanglansheng: `mingkong` reduces 2->1 -> deals 1 dmg -> `DamageEnd` triggers `mingkong2` and draws 1!
    clear_all_hands(&mut e);
    let jiu = move_kind_suit_to_hand(&mut e, 0, CardKind::Jiu, Suit::Spade, &mut used);
    let sha_2 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Heart, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_2, targets: vec![1] });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_pre - 1, "mingkong reduced 2 damage to 1");
    assert_eq!(e.debug_hand(0).len(), 1, "mingkong2 triggered on DamageEnd and drew 1 for source");
    assert!(!e.debug_has_skill(0, "mingkong2"), "mingkong2 removed after DamageEnd");

    // 8d. `duhun` clamps target HP cleanly to `target.max_hp`!
    let (_, mut e_dh) = gujian_verified_and_engine(2026_9312, 3);
    e_dh.assign_general_named(0, "gjqt_ouyangshaogong").unwrap();
    e_dh.assign_general_named(1, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e_dh, 1, Phase::Play);
    clear_all_hands(&mut e_dh);
    e_dh.debug_set_hp(0, 1);
    e_dh.debug_set_hp(1, 2);
    let mut u_dh = Vec::new();
    let high0 = find_unused_card_by(&e_dh, &u_dh, |d| d.rank >= 11 && d.kind != CardKind::Shan);
    u_dh.push(high0);
    move_card_to_hand(&mut e_dh, 0, high0);
    let low1 = find_unused_card_by(&e_dh, &u_dh, |d| d.rank <= 3);
    u_dh.push(low1);
    move_card_to_hand(&mut e_dh, 1, low1);
    let ksha1 = move_kind_suit_to_hand(&mut e_dh, 1, CardKind::Sha, Suit::Spade, &mut u_dh);
    let _ = submit(&mut e_dh, 1, GameAction::PlayCard { card: ksha1, targets: vec![0] });
    let duhun_def = e_dh.catalog().skill_by_name("duhun").unwrap();
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Skill(duhun_def)]);
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Player(1)]);
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Card(high0)]);
    let _ = answer_choice(&mut e_dh, vec![ChoiceOption::Card(low1)]);
    drain(&mut e_dh);
    assert!(e_dh.debug_hp(1) <= e_dh.state_core().player(1).max_hp as i32);
    assert!(e_dh.debug_hp(0) <= e_dh.state_core().player(0).max_hp as i32);
    assert_zone_conservation_all(&e_dh);

    // 8e. Public-info AI check (`BoundedPlanner`) on `gj_mingkong` / `gj_yange_target`:
    let planner = BoundedPlanner {
        node_budget: 8,
        deadline_ms: Some(100),
        cancel: Arc::new(AtomicBool::new(false)),
        lookahead_depth: 1,
        policy_variant: 1,
        content_tuning: None,
        l01_options: None,
    };
    let (_, mut e_ai) = gujian_verified_and_engine(2026_9313, 3);
    e_ai.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e_ai.assign_general_named(1, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e_ai, 2, Phase::End);
    let _ = submit(&mut e_ai, 2, GameAction::NextPhase);
    drain_until_prompt(&mut e_ai, "gj_yange_target");
    let ai_plan = e_ai.plan_action(0, &planner);
    assert!(matches!(ai_plan.action, GameAction::Answer { .. }), "AI handles gj_yange_target");
}
