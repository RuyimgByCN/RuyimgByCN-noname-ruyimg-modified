//! Phase 3 Track P3-A Batch 1 真实对局闭包与等价修复回归测试：
//! - `U05:configs` (`gwent`/`mtg`/`yunchou`/`zhenfa` 真加载过滤与存档恢复过滤)
//! - `U05:generals:diy_caiwenji`（经典版 `ycn_beige` 与 `olbeige` 严格分离）
//! - `U05:cards:qixingzhen`（弃手牌 + 真实 `use_virtual_card(Sha)` 不计入出杀次数）

use super::*;
use content_native_loader::restore_locked_engine_with_shahai_config;
use engine_native::ai::BoundedPlanner;
use engine_native::RejectReason;

fn has_general(e: &Engine, name: &str) -> bool {
    (0..e.catalog().general_count() as u16)
        .any(|g| e.catalog().general_meta(g).is_some_and(|m| m.0 == name))
}

#[test]
fn u05_p3a_configs_filter_gwent_mtg_yunchou_zhenfa_start_and_restore() {
    let pkg_dirs = [
        "shahai-gwent-upstream-noname",
        "shahai-gujian-upstream-noname",
        "shahai-mtg-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
        "shahai-swd-upstream-noname",
        "shahai-hearth-upstream-noname",
    ];
    let verified = load_u05_b1_verified(&pkg_dirs);

    let default_cfg = ShahaiMainConfig::default();
    let e_all = start_locked_engine_with_shahai_config(
        11_001,
        2,
        LockedStartMode::Prototype,
        &verified,
        &default_cfg,
    )
    .expect("default config starts all subpackages");
    assert!(has_general(&e_all, "gw_huoge"));
    assert!(has_general(&e_all, "gjqt_bailitusu"));
    assert!(has_general(&e_all, "mtg_jiding"));
    assert!(has_general(&e_all, "diy_caiwenji"));
    assert!(has_general(&e_all, "swd_anka"));
    assert!(has_general(&e_all, "hs_jaina"));
    assert!(e_all.catalog().cards.iter().any(|c| c.name == "qixingzhen"));

    // 1) gwent = false -> upstream `precontent.js:28-59`: disables gwent, gujian, swd, hearth
    let mut cfg_no_gwent = ShahaiMainConfig::default();
    cfg_no_gwent.gwent = false;
    let e_ng = start_locked_engine_with_shahai_config(
        11_002,
        2,
        LockedStartMode::Prototype,
        &verified,
        &cfg_no_gwent,
    )
    .expect("gwent=false starts without shahai-gwent and dependents");
    assert!(!has_general(&e_ng, "gw_huoge"), "gwent=false excludes gw_huoge");
    assert!(!has_general(&e_ng, "gjqt_bailitusu"), "gwent=false excludes gujian");
    assert!(!has_general(&e_ng, "swd_anka"), "gwent=false excludes swd");
    assert!(!has_general(&e_ng, "hs_jaina"), "gwent=false excludes hearth");
    assert!(!e_ng.catalog().cards.iter().any(|c| c.name == "shunyu_gwent" || c.name.starts_with("gw_")));
    assert!(has_general(&e_ng, "mtg_jiding"));
    assert!(has_general(&e_ng, "diy_caiwenji"));
    assert!(e_ng.catalog().cards.iter().any(|c| c.name == "qixingzhen"));

    let snap_ng = decode_archive(&encode_archive(&e_ng.save())).expect("decode NGSA 107");
    let e_ng_restored = restore_locked_engine_with_shahai_config(&snap_ng, &verified, &cfg_no_gwent)
        .expect("restore_locked_engine_with_shahai_config honors gwent=false");
    assert_eq!(e_ng_restored.state_hash(), e_ng.state_hash());
    assert!(!has_general(&e_ng_restored, "gw_huoge"));
    assert!(!has_general(&e_ng_restored, "swd_anka"));

    // 2) mtg = false
    let mut cfg_no_mtg = ShahaiMainConfig::default();
    cfg_no_mtg.mtg = false;
    let e_nm = start_locked_engine_with_shahai_config(
        11_003,
        2,
        LockedStartMode::Prototype,
        &verified,
        &cfg_no_mtg,
    )
    .expect("mtg=false starts without shahai-mtg");
    assert!(!has_general(&e_nm, "mtg_jiding"), "mtg=false excludes mtg_jiding");
    assert!(!e_nm.catalog().cards.iter().any(|c| c.name.starts_with("mtg_")));
    assert!(has_general(&e_nm, "gw_huoge"));
    assert!(has_general(&e_nm, "diy_caiwenji"));
    assert!(has_general(&e_nm, "swd_anka"));

    // 3) yunchou = false -> upstream `precontent.js:28-59`: disables yunchou, swd, hearth (keeps gwent & gujian)
    let mut cfg_no_yc = ShahaiMainConfig::default();
    cfg_no_yc.yunchou = false;
    let e_ny = start_locked_engine_with_shahai_config(
        11_004,
        2,
        LockedStartMode::Prototype,
        &verified,
        &cfg_no_yc,
    )
    .expect("yunchou=false starts without shahai-yunchou and dependents");
    assert!(!has_general(&e_ny, "diy_caiwenji"), "yunchou=false excludes diy_caiwenji");
    assert!(!has_general(&e_ny, "swd_anka"), "yunchou=false excludes swd");
    assert!(!has_general(&e_ny, "hs_jaina"), "yunchou=false excludes hearth");
    assert!(!e_ny.catalog().cards.iter().any(|c| c.name == "suolianjia" || c.name == "toulianghuanzhu"));
    assert!(has_general(&e_ny, "gw_huoge"), "yunchou=false keeps gwent");
    assert!(has_general(&e_ny, "gjqt_bailitusu"), "yunchou=false keeps gujian");
    assert!(has_general(&e_ny, "mtg_jiding"));

    // 4) zhenfa = false
    let mut cfg_no_zf = ShahaiMainConfig::default();
    cfg_no_zf.zhenfa = false;
    let e_nz = start_locked_engine_with_shahai_config(
        11_005,
        2,
        LockedStartMode::Prototype,
        &verified,
        &cfg_no_zf,
    )
    .expect("zhenfa=false starts without shahai-zhenfa");
    assert!(!e_nz.catalog().cards.iter().any(|c| c.name == "qixingzhen" || c.name == "tianfuzhen"));
    assert!(has_general(&e_nz, "diy_caiwenji"));
}

#[test]
fn u05_p3a_diy_caiwenji_classic_beige_parity_and_olbeige_separation() {
    let cfg = ShahaiMainConfig::default();
    let (verified, mut e) = start_u05_b1_engine(
        11_010,
        2,
        LockedStartMode::Prototype,
        &cfg,
        &["shahai-yunchou-upstream-noname"],
    );
    e.assign_general_named(0, "diy_caiwenji").expect("assign diy_caiwenji");
    let before_total = total_tracked_cards(&e);

    // 1) Sourceless damage (`shandian` / lose_hp / etc.) does NOT trigger ycn_beige:
    // Verify via Shandian hitting spade 2..=9 (Wait: Shandian emits Damage(None, 0, 3)).
    let sd = find_card(&e, |c| c.kind == CardKind::ShanDian);
    let spade5 = find_card(&e, |c| c.suit == Suit::Spade && (2..=9).contains(&c.rank));
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, sd);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sd, targets: vec![] });
    drain(&mut e);
    advance_to_phase(&mut e, 1, Phase::Play);
    // Advance to seat 1 Play first so 1's Draw doesn't pop spade5; place spade5 right before 0's Judge:
    e.state_core_mut().player_mut(0).hp = 4;
    put_deck_top(&mut e, spade5);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(e.state_core().player(0).hp, 1, "Shandian (sourceless) hit diy_caiwenji without beige");

    // 2) Cancel BEFORE judge: seat 1 deals Sha damage -> asks u05_ycn_beige_discard (allow_cancel: true)
    // Cancelling must NOT draw a judge card or mutate hp/hand!
    advance_to_phase(&mut e, 1, Phase::Play);
    let cwj_drop1 = find_card(&e, |c| c.kind == CardKind::Sha);
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, cwj_drop1);
    e.state_core_mut().player_mut(0).hp = 3;
    let sha1 = find_card_excl(&e, &[cwj_drop1], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 1, sha1);
    let deck_top_before = e.state_core().deck.last().copied();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    let ch = e.open_choice().cloned().expect("u05_ycn_beige_discard open");
    assert_eq!(ch.observer, 0);
    assert!(ch.spec.allow_cancel);
    assert_eq!(ch.spec.min, 0);
    e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_ycn_beige_discard", vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().deck.last().copied(), deck_top_before, "cancel before judge must NOT draw judge card");
    assert_eq!(e.state_core().player(0).hp, 2);
    assert_eq!(e.state_core().player(0).hand, vec![cwj_drop1]);
    assert_eq!(flag(&e, 0, "beige_gains"), 0);

    // 3) Heart branch -> recover +1 (NO beige_gains)
    let cwj_h = find_card_excl(&e, &[cwj_drop1], |c| c.kind == CardKind::Shan);
    let sha2 = find_card_excl(&e, &[cwj_drop1, cwj_h], |c| c.kind == CardKind::Sha);
    let judge_heart = find_card_excl(&e, &[cwj_drop1, cwj_h, sha2], |c| c.suit == Suit::Heart);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, cwj_h);
    e.state_core_mut().player_mut(0).hp = 3;
    move_card_to_hand(&mut e, 1, sha2);
    put_deck_top(&mut e, judge_heart);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    answer(&mut e, "u05_ycn_beige_discard", vec![ChoiceOption::Card(cwj_h)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "Heart judge recovers +1 back to 3");
    assert_eq!(flag(&e, 0, "beige_gains"), 0, "classic beige never sets beige_gains");

    // 4) Diamond branch -> draw 2
    let cwj_d = find_card_excl(&e, &[sha2], |c| c.kind == CardKind::Shan);
    let sha3 = find_card_excl(&e, &[cwj_d], |c| c.kind == CardKind::Sha);
    let judge_diamond = find_card_excl(&e, &[cwj_d, sha3], |c| c.suit == Suit::Diamond);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, cwj_d);
    move_card_to_hand(&mut e, 1, sha3);
    put_deck_top(&mut e, judge_diamond);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha3, targets: vec![0] });
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    answer(&mut e, "u05_ycn_beige_discard", vec![ChoiceOption::Card(cwj_d)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 2, "Diamond judge draws 2");
    assert_eq!(flag(&e, 0, "beige_gains"), 0);

    // 5) Club branch -> attacker discards 2 (`u05_beige_club_discard`)
    let cwj_c = find_card_excl(&e, &[], |c| c.kind == CardKind::Shan);
    let sha4 = find_card_excl(&e, &[cwj_c], |c| c.kind == CardKind::Sha);
    let att_drop1 = find_card_excl(&e, &[cwj_c, sha4], |c| c.kind == CardKind::Tao);
    let att_drop2 = find_card_excl(&e, &[cwj_c, sha4, att_drop1], |c| c.kind == CardKind::Tao);
    let judge_club = find_card_excl(&e, &[cwj_c, sha4, att_drop1, att_drop2], |c| c.suit == Suit::Club);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, cwj_c);
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 1, sha4);
    move_card_to_hand(&mut e, 1, att_drop1);
    move_card_to_hand(&mut e, 1, att_drop2);
    put_deck_top(&mut e, judge_club);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha4, targets: vec![0] });
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    answer(&mut e, "u05_ycn_beige_discard", vec![ChoiceOption::Card(cwj_c)]);
    assert_eq!(
        e.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "u05_beige_club_discard"))
    );
    answer(&mut e, "u05_beige_club_discard", vec![ChoiceOption::Card(att_drop1), ChoiceOption::Card(att_drop2)]);
    drain(&mut e);
    assert!(e.state_core().player(1).hand.is_empty(), "Club branch forces attacker to discard 2");

    // 6) Spade branch -> attacker turns over
    let cwj_s = find_card_excl(&e, &[], |c| c.kind == CardKind::Shan);
    let sha5 = find_card_excl(&e, &[cwj_s], |c| c.kind == CardKind::Sha);
    let judge_spade = find_card_excl(&e, &[cwj_s, sha5], |c| c.suit == Suit::Spade);
    e.state_core_mut().players[0].hp = 3;
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, cwj_s);
    move_card_to_hand(&mut e, 1, sha5);
    put_deck_top(&mut e, judge_spade);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha5, targets: vec![0] });
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    answer(&mut e, "u05_ycn_beige_discard", vec![ChoiceOption::Card(cwj_s)]);
    drain(&mut e);
    assert!(e.state_core().player(1).face_down, "Spade branch turns attacker over");
    assert_eq!(flag(&e, 0, "beige_gains"), 0);

    // 7) Converted Sha (Wusheng red Tao -> Sha) ALSO triggers ycn_beige!
    e.state_core_mut().players[1].face_down = false;
    e.state_core_mut().players[0].hp = 3;
    e.debug_grant(1, "武圣");
    let red_tao = find_card_excl(&e, &[], |c| c.kind == CardKind::Tao && matches!(c.suit, Suit::Heart | Suit::Diamond));
    let cwj_ws = find_card_excl(&e, &[red_tao], |c| c.kind == CardKind::Shan);
    let judge_ws = find_card_excl(&e, &[red_tao, cwj_ws], |c| c.suit == Suit::Diamond);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 0, cwj_ws);
    move_card_to_hand(&mut e, 1, red_tao);
    put_deck_top(&mut e, judge_ws);
    let out_ws = submit(&mut e, 1, GameAction::PlayCard { card: red_tao, targets: vec![0] });
    assert!(!matches!(out_ws, CommandOutcome::Rejected(_)), "Wusheng Sha rejected: {out_ws:?}");
    drain_until_prompt(&mut e, "u05_ycn_beige_discard");
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("u05_ycn_beige_discard"));
    answer(&mut e, "u05_ycn_beige_discard", vec![ChoiceOption::Card(cwj_ws)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "Wusheng converted Sha triggered ycn_beige Diamond draw 2 for victim"
    );
    let _ = before_total;
}

fn drain_until_prompt(e: &mut Engine, prompt: &str) {
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            return;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

#[test]
fn u05_p3a_qixingzhen_virtual_sha_no_limit_and_kongcheng_respect() {
    let cfg = ShahaiMainConfig::default();
    let (verified, mut e) = start_u05_b1_engine(
        11_020,
        4,
        LockedStartMode::Prototype,
        &cfg,
        &["shahai-zhenfa-upstream-noname", "shahai-swd-upstream-noname"],
    );
    let wei = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let shu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let wu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    e.debug_assemble_general(0, "孙权");
    e.debug_assemble_general(1, "刘备");
    e.debug_assemble_general(2, "曹操");
    e.debug_assemble_general(3, "刘备");
    let _ = (&wei, &shu, &wu);
    e.debug_grant(1, "空城");
    e.debug_grant(0, "anlianying");
    advance_to_phase(&mut e, 0, Phase::Play);
    let qxz = find_card(&e, |c| c.name == "qixingzhen");
    let eq1 = find_card(&e, |c| c.kind == CardKind::Armor);
    let drop3 = find_card_excl(&e, &[eq1], |c| c.kind == CardKind::Tao);
    let shan3 = find_card_excl(&e, &[eq1, drop3], |c| c.kind == CardKind::Shan);
    let extra_sha0 = find_card_excl(&e, &[eq1, drop3, shan3], |c| c.kind == CardKind::Sha);
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, qxz);
    move_card_to_hand(&mut e, 0, extra_sha0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    // Place armor equip on seat 1 with empty hand (tests full "he" discard + kongcheng!):
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != eq1);
        core.discard.retain(|x| *x != eq1);
        core.table.retain(|x| *x != eq1);
        for pl in &mut core.players {
            pl.hand.retain(|x| *x != eq1);
            pl.equips.retain(|x| *x != eq1);
        }
        core.players[1].equips.push(eq1);
    }
    move_card_to_hand(&mut e, 3, drop3);
    move_card_to_hand(&mut e, 3, shan3);
    let deck_before = e.state_core().deck.len();
    let log_start = e.log().len();
    let hp3_before = e.state_core().player(3).hp;
    let hp1_before = e.state_core().player(1).hp;
    let out = submit(&mut e, 0, GameAction::PlayCard { card: qxz, targets: vec![0] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "qixingzhen rejected: {out:?}");

    // Stage 1: besieger 1 loses equip from "he" (1 CardDiscarded log); besieger 3 loses hand card (1 log)!
    for obs_exp in [1u8, 3] {
        let ch = e.open_choice().cloned().expect("swd_qixingzhen_discard");
        assert_eq!(ch.observer, 0);
        assert_eq!(ch.spec.prompt, "swd_qixingzhen_discard");
        let pick = if obs_exp == 1 { eq1 } else { drop3 };
        assert!(ch.spec.options.contains(&ChoiceOption::Card(pick)), "must include 'he' card {pick}");
        answer(&mut e, "swd_qixingzhen_discard", vec![ChoiceOption::Card(pick)]);
    }
    // Stage 2: virtual Sha (card: None, no physical deck carrier) respects kongcheng on seat 1 and targets seat 3!
    let ch_shan = e.open_choice().cloned().expect("respond_shan for seat 3 only");
    assert_eq!(ch_shan.observer, 3);
    assert_eq!(ch_shan.spec.prompt, "respond_shan");
    let _ = &verified;
    assert_eq!(
        e.plan_action(3, &BoundedPlanner::new(2)).action,
        GameAction::Answer {
            choice_id: ch_shan.id,
            selection: vec![ChoiceOption::Card(shan3)],
        }
    );
    answer(&mut e, "respond_shan", vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert!(e.state_core().player(1).equips.is_empty(), "seat 1 equip discarded via 'he'");
    assert_eq!(e.state_core().player(3).hp, hp3_before - 1);
    assert_eq!(e.state_core().player(1).hp, hp1_before, "kongcheng immune to qixingzhen virtual Sha");
    assert_eq!(
        e.state_core().deck.len(),
        deck_before,
        "virtual Sha (card: None) must NOT consume a physical deck card"
    );
    assert_eq!(e.state_core().player(0).hand.len(), 1, "anlianying NOT falsely triggered by virtual Sha");
    for (seat, cid) in [(1u8, eq1), (3u8, drop3)] {
        let logs: Vec<_> = e.log()[log_start..]
            .iter()
            .filter(|ev| matches!(ev, engine_native::LogEvent::CardDiscarded { player, cards } if *player == seat && cards == &vec![cid]))
            .collect();
        assert_eq!(logs.len(), 1, "exactly 1 CardDiscarded log for seat {seat} card {cid}");
    }
    assert!(
        !e.state_core().turn.sha_used_play_phase,
        "qixingzhen virtual Sha MUST NOT consume sha_used_play_phase"
    );
    // Player 0 can still play a real Sha in the same Play phase!
    let out_sha = submit(&mut e, 0, GameAction::PlayCard { card: extra_sha0, targets: vec![3] });
    assert!(!matches!(out_sha, CommandOutcome::Rejected(RejectReason::ShaLimit)));
}
