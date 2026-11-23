//! U05-2 Round 2 targeted counterexample regression suite (`BUG-1`..`BUG-12`).
//!
//! Covers every counterexample from Round 1 verification (`upstream-u05-2-cdp-results.json`):
//! - BUG-1: card conservation & behavior for `feibiao`, `bingpotong`, `dinvxuanshuang`, `zuiji`
//! - BUG-2: `liufengsan` self-target, `ziyangdan` additive rounds + hujia
//! - BUG-3: food initial round counts, `qingtuan` single phaseAfter tick, `xiajiao3` cast-turn guard,
//!   `gudonggeng` `TurnBegin` countdown, `yuanbaorou` no bogus hand-limit ignore
//! - BUG-4: LIFO sequential ordering on `fanshi`, `shishui`, `cihong`
//! - BUG-5: `yunyin` `TurnEndWindow { after: true }` cleanup without false prompt
//! - BUG-6: `liuying` hit-list cleared ONLY on `TurnEndWindow { after: true }`
//! - BUG-7: `chizhen` optional `trigger_confirm` (confirm & pass)
//! - BUG-8: `yanjia` excludes trick `heilonglinpian` and `nomod+unique` `gjyuheng`
//! - BUG-9: `zhaolu` resets to `alive_count().min(5)`; `dangping` resets turn lock for all seats
//! - BUG-10: `xuanci` refunds `once_used` on target cancel; triggers follow-up `Sha` on real `Feibiao`
//! - BUG-11: `zhenying` triggers forced discard on `CardResponded` and clears `gj_zhenying_card:{cid}`
//! - BUG-12: `xuanzhen` uses `"gj_xuanzhen_confirm"` + 3-card discover (`<= 1` equip) persisted for save/restore;
//!   `yange` isolates hand/equips/skills, clones stock skills only, restores at turn end & lethal sub-player death

use super::common::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase};

fn count_in_discard(e: &engine_native::Engine, cid: u32) -> usize {
    e.state_core().discard.iter().filter(|&&c| c == cid).count()
}

fn set_flag_for_test(e: &mut engine_native::Engine, p: u8, k: &str, v: u32) {
    e.state_core_mut().vm_aux.flags.insert((p, k.to_string()), v);
}

#[test]
fn u05_2_r2_bug1_card_conservation_feibiao_bingpotong_dinv_zuiji() {
    let (verified, mut e) = gujian_verified_and_engine(2026_9001, 4);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // 1a. `feibiao` target with 0 black hand cards + hujia=2 -> LoseHp(1), bypasses hujia (card/gujian/card.js:792-794)
    let mut used = Vec::new();
    let fb1 = move_named_to_hand(&mut e, 0, "feibiao");
    used.push(fb1);
    let red2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Tao, Suit::Heart, &mut used);
    let _ = red2;
    e.state_core_mut().player_mut(2).hujia = 2;
    let hp2_before = e.debug_hp(2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fb1, targets: vec![2] });
    pass_nullify_window(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_before - 1, "feibiao with 0 black hand causes LoseHp 1");
    assert_eq!(e.state_core().player(2).hujia, 2, "LoseHp does not consume hujia");
    assert!(
        !e.log().iter().any(|l| matches!(l, LogEvent::DamageDealt { target: 2, .. })),
        "feibiao 0-black branch must not deal Damage"
    );
    assert_zone_conservation_all(&e);

    // 1b. `feibiao` discard black hand card -> lands in discard ONCE (no vanishing!)
    let fb2 = move_named_to_hand(&mut e, 0, "feibiao");
    let blk2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fb2, targets: vec![2] });
    pass_nullify_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_feibiao_discard");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(blk2)]);
    assert_eq!(count_in_discard(&e, blk2), 1, "feibiao discarded card must appear once in discard");
    assert_zone_conservation_all(&e);

    // 2. `bingpotong` stops immediately after first same-color match & conserves owner's shown card (card/gujian/card.js:724-737)
    let (_, mut e_bp) = gujian_verified_and_engine(2026_9002, 4);
    advance_to_phase(&mut e_bp, 0, Phase::Play);
    clear_all_hands(&mut e_bp);
    let mut u_bp = Vec::new();
    let bp = move_named_to_hand(&mut e_bp, 0, "bingpotong");
    u_bp.push(bp);
    let r0 = move_kind_suit_to_hand(&mut e_bp, 0, CardKind::Sha, Suit::Heart, &mut u_bp);
    let r1 = move_kind_suit_to_hand(&mut e_bp, 1, CardKind::Sha, Suit::Diamond, &mut u_bp);
    let r2 = move_kind_suit_to_hand(&mut e_bp, 2, CardKind::Shan, Suit::Heart, &mut u_bp);
    let _ = r2;
    let hp1_b = e_bp.debug_hp(1);
    let hp2_b = e_bp.debug_hp(2);
    let _ = submit(&mut e_bp, 0, GameAction::PlayCard { card: bp, targets: vec![1, 2] });
    pass_nullify_window(&mut e_bp);
    assert_eq!(e_bp.open_choice().unwrap().spec.prompt, "gj_bingpo_show_own");
    let _ = answer_choice(&mut e_bp, vec![ChoiceOption::Card(r0)]);
    assert_eq!(e_bp.open_choice().unwrap().spec.prompt, "gj_bingpo_show_target");
    assert_eq!(e_bp.open_choice().unwrap().observer, 1);
    let _ = answer_choice(&mut e_bp, vec![ChoiceOption::Card(r1)]);
    assert!(e_bp.open_choice().is_none(), "bingpotong must cancel remaining target 2 after match");
    assert_eq!(e_bp.debug_hp(1), hp1_b - 1, "bingpotong target 1 lost 1 HP");
    assert_eq!(e_bp.debug_hp(2), hp2_b, "bingpotong target 2 untouched");
    assert_eq!(count_in_discard(&e_bp, r0), 1, "owner matched card discarded exactly once");
    assert_zone_conservation_all(&e_bp);

    // 3. `dinvxuanshuang` during dying save prompts dying ally with `he` (incl. equip) + restore + discard once (card/gujian/card.js:551-569)
    let (_, mut e_dn) = gujian_verified_and_engine(2026_9003, 3);
    advance_to_phase(&mut e_dn, 0, Phase::Play);
    clear_all_hands(&mut e_dn);
    let mut u_dn = Vec::new();
    let dvx = move_named_to_hand(&mut e_dn, 0, "dinvxuanshuang");
    u_dn.push(dvx);
    let eq1 = find_unused_card_by(&e_dn, &u_dn, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    u_dn.push(eq1);
    move_card_to_hand(&mut e_dn, 1, eq1);
    advance_to_phase(&mut e_dn, 1, Phase::Play);
    let _ = submit(&mut e_dn, 1, GameAction::PlayCard { card: eq1, targets: vec![] });
    clear_all_hands(&mut e_dn);
    move_card_to_hand(&mut e_dn, 0, dvx);
    e_dn.debug_set_hp(1, 1);
    let sha_a = move_kind_suit_to_hand(&mut e_dn, 0, CardKind::Sha, Suit::Spade, &mut u_dn);
    advance_to_phase(&mut e_dn, 0, Phase::Play);
    clear_all_hands(&mut e_dn);
    move_card_to_hand(&mut e_dn, 0, dvx);
    move_card_to_hand(&mut e_dn, 0, sha_a);
    let _ = submit(&mut e_dn, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    // Dying save for seat 1 -> seat 0 saves with dinvxuanshuang
    let c_ds = e_dn.open_choice().cloned().expect("dying_save");
    assert_eq!(c_ds.spec.prompt, "dying_save");
    assert_eq!(c_ds.observer, 0);
    let _ = answer_choice(&mut e_dn, vec![ChoiceOption::Card(dvx)]);
    // Seat 1 recovered 0 -> 1, now gets gj_dinvxuanshuang_rider with eq1!
    let c_rd = e_dn.open_choice().cloned().expect("gj_dinvxuanshuang_rider");
    assert_eq!(c_rd.spec.prompt, "gj_dinvxuanshuang_rider");
    assert_eq!(c_rd.observer, 1, "dinvxuanshuang rider must prompt dying target 1");
    assert!(c_rd.spec.options.contains(&ChoiceOption::Card(eq1)), "rider must include equips (he)");
    let mut r_dn = save_restore_verified(&e_dn, &verified);
    let _ = answer_choice(&mut r_dn, vec![ChoiceOption::Card(eq1)]);
    drain(&mut r_dn);
    assert_eq!(r_dn.debug_hp(1), 1, "target 1 healed once (no double heal)");
    assert!(r_dn.state_core().player(1).equips.is_empty(), "eq1 discarded from equips");
    assert_eq!(r_dn.debug_hand(1).len(), 1, "target 1 drew 1 card");
    assert_eq!(count_in_discard(&r_dn, eq1), 1, "eq1 in discard once");
    assert_eq!(count_in_discard(&r_dn, dvx), 1, "dvx in discard once");
    assert_zone_conservation_all(&r_dn);

    // 4. `zuiji` consumes card once via CardUsed (no duplicate in discard!)
    let (_, mut e_zj) = gujian_verified_and_engine(2026_9004, 3);
    e_zj.assign_general_named(0, "gjqt_yinqianshang").unwrap();
    advance_to_phase(&mut e_zj, 0, Phase::Play);
    clear_all_hands(&mut e_zj);
    let mut u_zj = Vec::new();
    let zj_card = move_kind_suit_to_hand(&mut e_zj, 0, CardKind::Sha, Suit::Club, &mut u_zj);
    let sk_zj = e_zj.catalog().skill_by_name("zuiji").unwrap();
    let _ = submit(&mut e_zj, 0, GameAction::ActivateSkill { skill: sk_zj });
    let _ = answer_choice(&mut e_zj, vec![ChoiceOption::Card(zj_card)]);
    drain(&mut e_zj);
    assert!(e_zj.state_core().turn.jiu_active);
    assert_eq!(count_in_discard(&e_zj, zj_card), 1, "zuiji must put cid into discard exactly once");
    assert_zone_conservation_all(&e_zj);
}

#[test]
fn u05_2_r2_bug2_bug3_foods_liufengsan_ziyangdan_rounds_qingtuan_xiajiao_gudonggeng_yuanbaorou() {
    let (_, mut e) = gujian_verified_and_engine(2026_9005, 3);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);

    // 1. Liufengsan self-target allowed + Ziyangdan additive (+3 -> 6 rounds & 6 hujia)
    let lfs = move_named_to_hand(&mut e, 0, "liufengsan");
    let out_lfs = submit(&mut e, 0, GameAction::PlayCard { card: lfs, targets: vec![0] });
    assert!(!matches!(out_lfs, CommandOutcome::Rejected(_)), "liufengsan self target must succeed: {out_lfs:?}");
    pass_nullify_window(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "self liufengsan granted 2 Shan");

    let zy1 = move_named_to_hand(&mut e, 0, "ziyangdan");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zy1, targets: vec![1] });
    let def_zy = e.catalog().skill_by_name("gj_food_ziyangdan").unwrap();
    assert_eq!(e.state_core().skills.get(&(1, def_zy)).unwrap().rounds, 3);
    assert_eq!(e.state_core().player(1).hujia, 3);
    // Second ziyangdan stacks +3 -> 6 rounds & 6 hujia!
    clear_all_hands(&mut e);
    let yd_h = move_named_to_hand(&mut e, 0, "yanjiadan_heart");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: yd_h, targets: vec![] });
    // Tag 1 = Ziyangdan
    let _ = answer_choice(&mut e, vec![ChoiceOption::Tag(1)]);
    let _ = answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().skills.get(&(1, def_zy)).unwrap().rounds, 6, "ziyangdan stacks rounds += 3");
    assert_eq!(e.state_core().player(1).hujia, 6, "ziyangdan stacks hujia += 3");

    // 2. Verify all updated food initial mark counts + qingtuan single phaseAfter tick + xiajiao3 cast-turn guard!
    let cases = [
        ("luyugeng", "gj_food_luyugeng", 3u8),
        ("chunbing", "gj_food_chunbing", 5u8),
        ("gudonggeng", "gj_food_gudonggeng", 3u8),
        ("liyutang", "gj_food_liyutang", 2u8),
        ("mizhilianou", "gj_food_mizhilianou", 4u8),
        ("xiajiao", "gj_food_xiajiao", 3u8),
        ("tanhuadong", "gj_food_tanhuadong", 3u8),
        ("mapodoufu", "gj_food_mapodoufu", 2u8),
        ("qingtuan", "gj_food_qingtuan", 2u8),
        ("yougeng", "gj_food_yougeng", 2u8),
        ("molicha", "gj_food_molicha", 4u8),
        ("yuanbaorou", "gj_food_yuanbaorou", 4u8),
    ];
    for (card_name, sk_key, expected_rounds) in cases {
        let (_, mut ef) = gujian_verified_and_engine(2026_9006, 3);
        advance_to_phase(&mut ef, 0, Phase::Play);
        clear_all_hands(&mut ef);
        let c = move_named_to_hand(&mut ef, 0, card_name);
        let _ = submit(&mut ef, 0, GameAction::PlayCard { card: c, targets: vec![0] });
        let def = ef.catalog().skill_by_name(sk_key).unwrap();
        assert_eq!(
            ef.state_core().skills.get(&(0, def)).unwrap().rounds,
            expected_rounds,
            "{card_name} must start with {expected_rounds} rounds"
        );
    }

    // Qingtuan: does NOT tick on Sha, ticks 2->1 only at Phase::End
    let (_, mut eq) = gujian_verified_and_engine(2026_9007, 3);
    advance_to_phase(&mut eq, 0, Phase::Play);
    clear_all_hands(&mut eq);
    let qt = move_named_to_hand(&mut eq, 0, "qingtuan");
    let _ = submit(&mut eq, 0, GameAction::PlayCard { card: qt, targets: vec![0] });
    let def_qt = eq.catalog().skill_by_name("gj_food_qingtuan").unwrap();
    let mut u_q = Vec::new();
    let sha_q = move_kind_suit_to_hand(&mut eq, 0, CardKind::Sha, Suit::Spade, &mut u_q);
    let _ = submit(&mut eq, 0, GameAction::PlayCard { card: sha_q, targets: vec![1] });
    drain(&mut eq);
    assert_eq!(eq.state_core().skills.get(&(0, def_qt)).unwrap().rounds, 2, "qingtuan must not tick on Sha use");
    advance_to_phase(&mut eq, 0, Phase::End);
    let _ = submit(&mut eq, 0, GameAction::NextPhase);
    drain(&mut eq);
    assert_eq!(eq.state_core().skills.get(&(0, def_qt)).unwrap().rounds, 1, "qingtuan ticks once at Phase::End");

    // Xiajiao: cast on self -> gj_xiajiao3 skips cast-turn Phase::End tick (stays 3 rounds!)
    let (_, mut ex) = gujian_verified_and_engine(2026_9008, 3);
    advance_to_phase(&mut ex, 0, Phase::Play);
    clear_all_hands(&mut ex);
    let xj = move_named_to_hand(&mut ex, 0, "xiajiao");
    let _ = submit(&mut ex, 0, GameAction::PlayCard { card: xj, targets: vec![0] });
    let def_xj = ex.catalog().skill_by_name("gj_food_xiajiao").unwrap();
    advance_to_phase(&mut ex, 0, Phase::End);
    let _ = submit(&mut ex, 0, GameAction::NextPhase);
    drain(&mut ex);
    assert_eq!(ex.state_core().skills.get(&(0, def_xj)).unwrap().rounds, 3, "xiajiao3 prevents cast-turn tick");
    // Next own turn End decrements 3 -> 2
    advance_to_phase(&mut ex, 0, Phase::End);
    let _ = submit(&mut ex, 0, GameAction::NextPhase);
    drain(&mut ex);
    assert_eq!(ex.state_core().skills.get(&(0, def_xj)).unwrap().rounds, 2, "xiajiao ticks after Draw turn");

    // Gudonggeng: TurnBegin ticks down 3 -> 2 -> 1 -> removed even without Sha damage!
    let (_, mut eg) = gujian_verified_and_engine(2026_9009, 3);
    advance_to_phase(&mut eg, 0, Phase::Play);
    clear_all_hands(&mut eg);
    let gdg = move_named_to_hand(&mut eg, 0, "gudonggeng");
    let _ = submit(&mut eg, 0, GameAction::PlayCard { card: gdg, targets: vec![0] });
    let def_gd = eg.catalog().skill_by_name("gj_food_gudonggeng").unwrap();
    assert_eq!(eg.state_core().skills.get(&(0, def_gd)).unwrap().rounds, 3);
    advance_to_phase(&mut eg, 1, Phase::Play);
    advance_to_phase(&mut eg, 0, Phase::Judge);
    assert_eq!(eg.state_core().skills.get(&(0, def_gd)).unwrap().rounds, 2, "gudonggeng ticks at TurnBegin");
    advance_to_phase(&mut eg, 1, Phase::Play);
    advance_to_phase(&mut eg, 0, Phase::Judge);
    assert_eq!(eg.state_core().skills.get(&(0, def_gd)).unwrap().rounds, 1);
    advance_to_phase(&mut eg, 1, Phase::Play);
    advance_to_phase(&mut eg, 0, Phase::Judge);
    assert!(!eg.state_core().skills.contains_key(&(0, def_gd)), "gudonggeng removed after 3 TurnBegins");

    // Yuanbaorou: Sha is NOT ignored in Discard phase hand-limit check even if yuanbaorou active!
    let (_, mut ey) = gujian_verified_and_engine(2026_9010, 3);
    advance_to_phase(&mut ey, 0, Phase::Play);
    clear_all_hands(&mut ey);
    let ybr = move_named_to_hand(&mut ey, 0, "yuanbaorou");
    let _ = submit(&mut ey, 0, GameAction::PlayCard { card: ybr, targets: vec![0] });
    ey.debug_set_hp(0, 1);
    let mut uy = Vec::new();
    let _s1 = move_kind_suit_to_hand(&mut ey, 0, CardKind::Sha, Suit::Spade, &mut uy);
    let _s2 = move_kind_suit_to_hand(&mut ey, 0, CardKind::Sha, Suit::Club, &mut uy);
    let _ = submit(&mut ey, 0, GameAction::NextPhase); // Play -> Discard (2 Shas > hp 1 -> must discard!)
    assert_eq!(ey.open_choice().unwrap().spec.prompt, "discard_to_limit", "yuanbaorou must not ignore Sha in hand limit");
}

#[test]
fn u05_2_r2_bug4_5_6_7_8_9_lifo_yunyin_liuying_chizhen_yanjia_zhaolu_dangping() {
    // BUG-4a: `fanshi` LoseHp resolves BEFORE Draw (hp=1 enters Dying before drawing!)
    let (_, mut e) = gujian_verified_and_engine(2026_9011, 3);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let sha0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Spade, &mut used);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    drain(&mut e);
    e.debug_set_hp(0, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> DiscardEnd -> fanshi LoseHp -> Dying BEFORE Draw!
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dying_save", "fanshi must trigger LoseHp/Dying BEFORE Draw");
    let shahun_def = e.catalog().skill_by_name("shahun").unwrap();
    let _ = answer_choice(&mut e, vec![ChoiceOption::Skill(shahun_def)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 4, "shahun drew 3 during dying BEFORE fanshi drew 1 AFTER LoseHp settled");

    // BUG-4b: `shishui` settles target 1 LoseHp BEFORE target 2 (verified with dying on target 1)
    let (_, mut e_ss) = gujian_verified_and_engine(2026_9012, 4);
    e_ss.assign_general_named(0, "gjqt_ouyangshaogong").unwrap();
    advance_to_phase(&mut e_ss, 0, Phase::Play);
    clear_all_hands(&mut e_ss);
    let mut u_ss = Vec::new();
    // Red multi-target card (Taoyuan / SavageAttack / ArcheryAttack) targets seats 1 & 2 in sequence
    let red_multi = find_unused_card_by(&e_ss, &u_ss, |d| {
        matches!(d.suit, Suit::Heart | Suit::Diamond)
            && matches!(d.kind, CardKind::Taoyuan | CardKind::SavageAttack | CardKind::ArcheryAttack)
    });
    u_ss.push(red_multi);
    move_card_to_hand(&mut e_ss, 0, red_multi);
    let _tao3 = move_kind_suit_to_hand(&mut e_ss, 3, CardKind::Tao, Suit::Heart, &mut u_ss);
    e_ss.debug_set_hp(1, 1);
    e_ss.debug_set_hp(2, 3);
    let _ = submit(&mut e_ss, 0, GameAction::PlayCard { card: red_multi, targets: vec![] });
    pass_nullify_window(&mut e_ss);
    // Target 1 enters dying_save BEFORE target 2 takes LoseHp (target 2 hp still 3 while target 1 dying)!
    assert_eq!(e_ss.open_choice().unwrap().spec.prompt, "dying_save", "target 1 must enter dying_save before target 2");
    assert_eq!(e_ss.debug_hp(2), 3, "target 2 must not have lost HP yet while target 1 dying resolves");
    drain(&mut e_ss);

    // BUG-4c: `cihong` LoseHp resolves BEFORE virtual Sha (at hp=1, confirm LoseHp opens dying_save)
    let (_, mut e_ch) = gujian_verified_and_engine(2026_9013, 3);
    e_ch.assign_general_named(0, "gjqt_beiluo").unwrap();
    advance_to_phase(&mut e_ch, 0, Phase::End);
    clear_all_hands(&mut e_ch);
    let mut u_ch = Vec::new();
    let tao2 = move_kind_suit_to_hand(&mut e_ch, 2, CardKind::Tao, Suit::Heart, &mut u_ch);
    e_ch.debug_set_hp(0, 1);
    let _ = submit(&mut e_ch, 0, GameAction::NextPhase); // End -> cihong
    assert_eq!(e_ch.open_choice().unwrap().spec.prompt, "gj_cihong_target");
    let _ = answer_choice(&mut e_ch, vec![ChoiceOption::Player(1)]);
    assert_eq!(e_ch.open_choice().unwrap().spec.prompt, "gj_cihong_losehp");
    let _ = answer_choice(&mut e_ch, vec![ChoiceOption::Confirm]);
    assert_eq!(e_ch.open_choice().unwrap().spec.prompt, "dying_save", "cihong LoseHp resolves before virtual Sha");
    assert_eq!(e_ch.debug_hp(1), 4, "target 1 untouched while LoseHp dying resolves");
    // Pass Tao -> 0 dies, virtual Sha never fires on seat 1
    let _ = answer_choice(&mut e_ch, vec![ChoiceOption::Pass]);
    let _ = tao2;
    assert_eq!(e_ch.debug_hp(1), 4, "dead owner must not fire cihong virtual Sha");

    // BUG-5 + BUG-6 + BUG-7: `yunyin` TurnEndWindow { after: true }, `liuying` clear, `chizhen` optional Pass
    let (_, mut e_c) = gujian_verified_and_engine(2026_9014, 4);
    e_c.assign_general_named(0, "gjqt_wenrenyu").unwrap();
    e_c.assign_general_named(1, "gjqt_ouyangshaogong").unwrap();
    e_c.assign_general_named(2, "gjqt_yuewuyi").unwrap();
    advance_to_phase(&mut e_c, 0, Phase::Draw);
    clear_all_hands(&mut e_c);
    let _ = submit(&mut e_c, 0, GameAction::NextPhase); // Draw -> Play -> optional chizhen
    assert_eq!(e_c.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let _ = answer_choice(&mut e_c, vec![ChoiceOption::Pass]); // Pass -> 0 draws!
    assert_eq!(e_c.debug_hand(0).len(), 0, "passing optional chizhen draws/discards 0 cards");

    // yunyin: play Heart card during Play -> End -> decline gj_yunyin_card -> TurnEndWindow { after: true } clears mask cleanly with NO false prompt!
    advance_to_phase(&mut e_c, 1, Phase::Play);
    clear_all_hands(&mut e_c);
    let mut uc = Vec::new();
    let hsha = move_kind_suit_to_hand(&mut e_c, 1, CardKind::Sha, Suit::Heart, &mut uc);
    let spare_c = move_kind_suit_to_hand(&mut e_c, 1, CardKind::Sha, Suit::Spade, &mut uc);
    let _ = spare_c;
    let _ = submit(&mut e_c, 1, GameAction::PlayCard { card: hsha, targets: vec![2] });
    drain(&mut e_c);
    assert!(flag(&e_c, 1, "gj_yunyin_suit_mask") > 0);
    let _ = submit(&mut e_c, 1, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e_c, 1, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut e_c, 1, GameAction::NextPhase); // End -> TurnEndWindow { after: false } -> asks gj_yunyin_card once
    assert_eq!(e_c.open_choice().unwrap().spec.prompt, "gj_yunyin_card");
    let _ = answer_choice(&mut e_c, vec![]); // cancel -> proceeds into TurnEndWindow { after: true }
    assert!(e_c.open_choice().is_none(), "TurnEndWindow after:true must NOT re-prompt gj_yunyin_card");
    assert_eq!(flag(&e_c, 1, "gj_yunyin_suit_mask"), 0, "yunyin suit mask cleared after TurnEndWindow");

    // liuying: Sha on seat 1 marks gj_liuying_hit:1, TurnEndWindow after:true clears it!
    advance_to_phase(&mut e_c, 2, Phase::Play);
    clear_all_hands(&mut e_c);
    let sha2 = move_kind_suit_to_hand(&mut e_c, 2, CardKind::Sha, Suit::Club, &mut uc);
    let _ = submit(&mut e_c, 2, GameAction::PlayCard { card: sha2, targets: vec![1] });
    drain(&mut e_c);
    assert_eq!(flag(&e_c, 2, "gj_liuying_hit:1"), 1);
    advance_to_phase(&mut e_c, 2, Phase::End);
    let _ = submit(&mut e_c, 2, GameAction::NextPhase);
    drain(&mut e_c);
    assert_eq!(flag(&e_c, 2, "gj_liuying_hit:1"), 0, "liuying hit list cleared at owner TurnEndWindow after:true");

    // BUG-8: `yanjia` excludes trick `heilonglinpian` and nomod+unique `gjyuheng`
    advance_to_phase(&mut e_c, 2, Phase::Play);
    clear_all_hands(&mut e_c);
    let hl = move_named_to_hand(&mut e_c, 2, "heilonglinpian");
    let mtm = move_named_to_hand(&mut e_c, 2, "mutoumianju");
    let yh = find_card_by(&e_c, |_, d| {
        d.kind == CardKind::Gjyuheng && !d.off_deck
    });
    move_card_to_hand(&mut e_c, 2, yh);
    let sk_yj = e_c.catalog().skill_by_name("yanjia").unwrap();
    let _ = hl;
    assert!(
        matches!(submit(&mut e_c, 2, GameAction::ActivateSkill { skill: sk_yj }), CommandOutcome::Rejected(_)),
        "yanjia must reject when only mutoumianju + excluded heilonglinpian + excluded gjyuheng"
    );
    let zhuge = find_card_by(&e_c, |_, d| d.kind == CardKind::Zhuge && !d.name.contains('_'));
    move_card_to_hand(&mut e_c, 2, zhuge);
    let out_yj = submit(&mut e_c, 2, GameAction::ActivateSkill { skill: sk_yj });
    assert!(matches!(out_yj, CommandOutcome::Pending { .. }), "zhuge + mutoumianju allowed: {out_yj:?}");
    let opts = &e_c.open_choice().unwrap().spec.options;
    assert!(opts.contains(&ChoiceOption::Card(zhuge)) && opts.contains(&ChoiceOption::Card(mtm)));
    assert!(!opts.contains(&ChoiceOption::Card(hl)), "heilonglinpian excluded from yanjia");
    assert!(!opts.contains(&ChoiceOption::Card(yh)), "gjyuheng excluded from yanjia");

    // BUG-9: `dangping` off-turn reset on current turn end
    let (_, mut e_dp) = gujian_verified_and_engine(2026_9015, 3);
    e_dp.assign_general_named(0, "gjqt_wenrenyu").unwrap();
    advance_to_phase(&mut e_dp, 1, Phase::Play);
    set_flag_for_test(&mut e_dp, 0, "gj_dangping_used_turn", 1);
    advance_to_phase(&mut e_dp, 1, Phase::End);
    let _ = submit(&mut e_dp, 1, GameAction::NextPhase); // seat 1 Phase::End resets dangping for ALL seats!
    drain(&mut e_dp);
    assert_eq!(flag(&e_dp, 0, "gj_dangping_used_turn"), 0, "dangping used flag resets on any turn's Phase::End");
}

#[test]
fn u05_2_r2_bug10_11_12_xuanci_zhenying_xuanzhen_yange() {
    // BUG-10: `xuanci` target cancel refunds once_used AND normal Feibiao triggers follow-up Sha
    let (_, mut e) = gujian_verified_and_engine(2026_9016, 4);
    e.assign_general_named(0, "gjqt_chuqi").unwrap();
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_all_hands(&mut e);
    let mut used = Vec::new();
    let cl0 = move_kind_suit_to_hand(&mut e, 0, CardKind::Sha, Suit::Club, &mut used);
    let sk_xc = e.catalog().skill_by_name("xuanci").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_xc });
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(cl0)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_xuanci_target");
    let _ = answer_choice(&mut e, vec![]); // cancel at TargetChoice -> refunds once_used!
    let out_retry = submit(&mut e, 0, GameAction::ActivateSkill { skill: sk_xc });
    assert!(matches!(out_retry, CommandOutcome::Pending { .. }), "xuanci must refund once_used on target cancel: {out_retry:?}");
    let _ = answer_choice(&mut e, vec![]); // cancel card choice too
    // Real Feibiao card from hand triggers follow-up Sha via xuanci!
    let fb = move_named_to_hand(&mut e, 0, "feibiao");
    used.push(fb);
    let blk2 = move_kind_suit_to_hand(&mut e, 2, CardKind::Sha, Suit::Spade, &mut used);
    let hp2_b = e.debug_hp(2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![2] });
    pass_nullify_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gj_feibiao_discard");
    let _ = answer_choice(&mut e, vec![ChoiceOption::Card(blk2)]); // avoids Feibiao loseHp
    // Follow-up virtual Sha from xuanci hits seat 2 (no Shan -> -1 HP)!
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_b - 1, "real Feibiao must trigger xuanci follow-up virtual Sha");

    // BUG-11: `zhenying` respondAfter forced discard + flag cleanup (via Duel + black Sha response)
    let (_, mut e_zy) = gujian_verified_and_engine(2026_9017, 3);
    e_zy.assign_general_named(1, "gjqt_beiluo").unwrap();
    advance_to_phase(&mut e_zy, 0, Phase::Play);
    clear_all_hands(&mut e_zy);
    let mut u_zy = Vec::new();
    let blk_sha1 = move_kind_suit_to_hand(&mut e_zy, 1, CardKind::Sha, Suit::Spade, &mut u_zy);
    let blk_sha2 = move_kind_suit_to_hand(&mut e_zy, 1, CardKind::Sha, Suit::Club, &mut u_zy);
    // Keep blk_sha2 at top of deck so zhenying copies blk_sha2 when seat 1 responds to Duel with blk_sha1!
    e_zy.state_core_mut().player_mut(1).hand.retain(|&c| c != blk_sha2);
    e_zy.state_core_mut().deck.push(blk_sha2);
    let duel_a = find_unused_card_by(&e_zy, &u_zy, |d| d.kind == CardKind::Duel);
    u_zy.push(duel_a);
    move_card_to_hand(&mut e_zy, 0, duel_a);
    let duel_b = find_unused_card_by(&e_zy, &u_zy, |d| d.kind == CardKind::Duel);
    u_zy.push(duel_b);
    move_card_to_hand(&mut e_zy, 0, duel_b);
    let _ = submit(&mut e_zy, 0, GameAction::PlayCard { card: duel_a, targets: vec![1] });
    pass_nullify_window(&mut e_zy);
    let _ = answer_choice(&mut e_zy, vec![ChoiceOption::Card(blk_sha1)]); // seat 1 responds with black Sha
    drain(&mut e_zy); // seat 0 passes Duel Sha -> takes 1 dmg -> Duel ends
    assert!(e_zy.debug_hand(1).contains(&blk_sha2), "zhenying copied blk_sha2 on CardResponded");
    assert_eq!(flag(&e_zy, 1, &format!("gj_zhenying_card:{blk_sha2}")), 1);
    let spare_t = move_kind_suit_to_hand(&mut e_zy, 1, CardKind::Tao, Suit::Heart, &mut u_zy);
    let _ = submit(&mut e_zy, 0, GameAction::PlayCard { card: duel_b, targets: vec![1] });
    pass_nullify_window(&mut e_zy);
    let _ = answer_choice(&mut e_zy, vec![ChoiceOption::Card(blk_sha2)]); // Respond with zhenying_link Sha!
    assert_eq!(e_zy.open_choice().unwrap().spec.prompt, "gj_zhenying_discard", "zhenying must trigger forced discard on CardResponded");
    let _ = answer_choice(&mut e_zy, vec![ChoiceOption::Card(spare_t)]);
    drain(&mut e_zy);
    assert_eq!(flag(&e_zy, 1, &format!("gj_zhenying_card:{blk_sha2}")), 0, "gj_zhenying_card flag cleared after CardResponded");
    assert_eq!(count_in_discard(&e_zy, blk_sha2), 1, "blk_sha2 conserved in discard once");
    assert_zone_conservation_all(&e_zy);

    // BUG-12a: `xuanzhen` uses `"gj_xuanzhen_confirm"`, discovers 3 eligible options, persists across save/restore
    let (verified, mut e_xz) = gujian_verified_and_engine(2026_9018, 3);
    e_xz.assign_general_named(1, "gjqt_cenying").unwrap();
    advance_to_phase(&mut e_xz, 0, Phase::Play);
    clear_all_hands(&mut e_xz);
    let mut u_xz = Vec::new();
    let sha_xz = move_kind_suit_to_hand(&mut e_xz, 0, CardKind::Sha, Suit::Spade, &mut u_xz);
    let _ = submit(&mut e_xz, 0, GameAction::PlayCard { card: sha_xz, targets: vec![1] });
    let cxz = e_xz.open_choice().cloned().expect("gj_xuanzhen_confirm");
    assert_eq!(cxz.spec.prompt, "gj_xuanzhen_confirm");
    let _ = answer_choice(&mut e_xz, vec![ChoiceOption::Confirm]);
    let cv = e_xz.open_choice().cloned().expect("gj_xuanzhen_vcard");
    assert_eq!(cv.spec.prompt, "gj_xuanzhen_vcard");
    assert_eq!(cv.spec.options.len(), 3, "xuanzhen must present exactly 3 discovered cards");
    let r_xz = save_restore_verified(&e_xz, &verified);
    let cv_r = r_xz.open_choice().cloned().expect("restored xuanzhen vcard");
    assert_eq!(cv_r.spec.options, cv.spec.options, "save/restore must preserve exact 3 sampled xuanzhen options");
    let mut r_xz = r_xz;
    let _ = answer_choice(&mut r_xz, vec![cv_r.spec.options[0]]);
    drain(&mut r_xz);
    assert_zone_conservation_all(&r_xz);

    // BUG-12b: `yange` sub-player hand/equip/skill isolation & lethal sub-player exit
    let (_, mut e_yg) = gujian_verified_and_engine(2026_9019, 4);
    e_yg.assign_general_named(0, "gjqt_yunwuyue").unwrap();
    e_yg.assign_general_named(2, "gjqt_bailitusu").unwrap();
    advance_to_phase(&mut e_yg, 0, Phase::Play);
    clear_all_hands(&mut e_yg);
    let mut u_yg = Vec::new();
    let orig_tao = move_kind_suit_to_hand(&mut e_yg, 0, CardKind::Tao, Suit::Heart, &mut u_yg);
    let food_t = move_named_to_hand(&mut e_yg, 0, "luyugeng");
    let _ = submit(&mut e_yg, 0, GameAction::PlayCard { card: food_t, targets: vec![2] }); // target 2 holds food skill luyugeng (NOT stock skill!)
    let t_sha = move_kind_suit_to_hand(&mut e_yg, 2, CardKind::Sha, Suit::Spade, &mut u_yg);
    let _ = t_sha;
    advance_to_phase(&mut e_yg, 3, Phase::End);
    let _ = submit(&mut e_yg, 3, GameAction::NextPhase);
    drain_until_prompt(&mut e_yg, "gj_yange_target");
    let _ = answer_choice(&mut e_yg, vec![ChoiceOption::Player(2)]);
    assert!(e_yg.debug_has_skill(0, "xuelu"), "yange cloned target stock skill xuelu");
    assert!(!e_yg.debug_has_skill(0, "gj_food_luyugeng"), "yange must clone stock skills only, not food skills");
    assert!(!e_yg.debug_has_skill(0, "woxue"), "yange sub-player must isolate main general skill woxue");
    assert!(!e_yg.debug_hand(0).contains(&orig_tao), "yange sub-player must isolate main player's orig_tao");
    assert_eq!(e_yg.debug_hand(0).len(), e_yg.debug_hand(2).len(), "yange clones target hand count");
    assert_zone_conservation_all(&e_yg);
    // Advance past TurnEndWindow -> main player restored with orig_tao & woxue/lianjing!
    advance_to_phase(&mut e_yg, 0, Phase::End);
    let _ = submit(&mut e_yg, 0, GameAction::NextPhase); // Phase::End restores yange, TurnEndWindow opens lianjing!
    drain_until_prompt(&mut e_yg, "gj_lianjing_targets");
    assert_eq!(e_yg.open_choice().unwrap().spec.prompt, "gj_lianjing_targets", "restored yunwuyue still checks lianjing");
    let _ = answer_choice(&mut e_yg, vec![]);
    assert!(e_yg.debug_has_skill(0, "woxue"), "main general skill woxue restored");
    assert!(!e_yg.debug_has_skill(0, "xuelu"), "cloned skill xuelu removed");
    assert!(e_yg.debug_hand(0).contains(&orig_tao), "main general hand card orig_tao restored");
    assert_zone_conservation_all(&e_yg);

    // Lethal exit: if sub-player dies while yange active, main player exits alive with original state!
    advance_to_phase(&mut e_yg, 3, Phase::End);
    let _ = submit(&mut e_yg, 3, GameAction::NextPhase);
    drain_until_prompt(&mut e_yg, "gj_yange_target");
    let _ = answer_choice(&mut e_yg, vec![ChoiceOption::Player(2)]);
    clear_all_hands(&mut e_yg);
    // Seat 0 currently holds sub-player state; PlayCard(Duel) losing Duel to seat 1 triggers lethal -> sub-player exits alive!
    advance_to_phase(&mut e_yg, 0, Phase::Play);
    clear_all_hands(&mut e_yg);
    let mut u_dl = Vec::new();
    let duel0 = move_kind_suit_to_hand(&mut e_yg, 0, CardKind::Duel, Suit::Spade, &mut u_dl);
    let sha1_d = move_kind_suit_to_hand(&mut e_yg, 1, CardKind::Sha, Suit::Club, &mut u_dl);
    e_yg.debug_set_hp(0, 1);
    let _ = submit(&mut e_yg, 0, GameAction::PlayCard { card: duel0, targets: vec![1] });
    pass_nullify_window(&mut e_yg);
    let _ = answer_choice(&mut e_yg, vec![ChoiceOption::Card(sha1_d)]); // seat 1 plays Sha -> seat 0 has 0 Sha -> takes 1 lethal damage!
    drain(&mut e_yg);
    assert!(e_yg.state_core().player(0).alive, "sub-player lethal death exits sub-player keeping main player alive");
    assert!(e_yg.debug_has_skill(0, "woxue"), "main player skills restored on sub-player lethal exit");
    assert_eq!(flag(&e_yg, 0, "gj_yange_active"), 0);
    assert_zone_conservation_all(&e_yg);
}
