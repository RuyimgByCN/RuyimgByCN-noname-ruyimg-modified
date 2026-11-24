//! Phase 3 Track P3-A Batch 1 真实武将对局与等价修复回归测试：
//! - `U05:skills:stone_chenshui` (`stone_yisela` 实配对局)
//! - `U05:generals:{hs_jaina, hs_kcthun, hs_xsylvanas}` (跨包 `huopu`/`luanji`/`busi` + 原生技)
//! - `U05:generals:{hs_alleria, hs_tgolem, hs_wuther}` (`xinliegong`/`xinwuyan`/`rejieming` 真实宿主实作)

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::Phase;
use engine_native::{CommandOutcome, RejectReason};

#[test]
fn u05_p3a_stone_yisela_chenshui_real_match() {
    let mut e = fresh_proto(11_101, &["stone_yisela", "hs_lrexxar"]);
    let before_total = total_tracked_cards(&e);
    assert!(has_skill(&e, 0, "stone_chenshui"));
    clear_hand(&mut e, 0);
    advance_to_phase(&mut e, 0, Phase::End);
    drain_auto(&mut e);
    assert!(
        e.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name.starts_with("hsmengjing_")),
        "stone_yisela real match Phase::End draws dream card via stone_chenshui"
    );
    assert_eq!(total_tracked_cards(&e), before_total);
}

#[test]
fn u05_p3a_hearth_cross_subpack_jaina_kcthun_xsylvanas() {
    // 1) `hs_jaina`: `shahai-hearth__ref_huopu` + `aoshu` + `bingjia`
    let mut ej = fresh_proto(11_102, &["hs_jaina", "hs_lrexxar"]);
    assert!(has_skill(&ej, 0, "aoshu"));
    assert!(has_skill(&ej, 0, "bingjia"));
    assert!(has_skill(&ej, 0, "shahai-hearth__ref_huopu"));
    clear_hand(&mut ej, 0);
    clear_hand(&mut ej, 1);
    advance_to_phase(&mut ej, 0, Phase::Play);
    // Natural Wuzhong (even if Heart) must play cleanly without huopu hijack:
    let wz = give_free_card_kind_suit(&mut ej, 0, CardKind::Wuzhong, Some(Suit::Heart));
    assert!(!matches!(play_card(&mut ej, 0, wz, vec![]), CommandOutcome::Rejected(_)));
    drain_auto(&mut ej);
    // Heart Tao targeted at seat 1 converts via `shahai-hearth__ref_huopu` -> Liuxinghuoyu (once per phase):
    let heart_tao1 = give_free_card_kind_suit(&mut ej, 0, CardKind::Tao, Some(Suit::Heart));
    let heart_tao2 = give_free_card_kind_suit(&mut ej, 0, CardKind::Tao, Some(Suit::Heart));
    let hp1 = ej.state_core().player(1).hp;
    let out_hp1 = play_card(&mut ej, 0, heart_tao1, vec![1]);
    assert!(!matches!(out_hp1, CommandOutcome::Rejected(_)), "ref_huopu first Heart->Liuxinghuoyu accepted");
    drain_auto(&mut ej);
    assert_eq!(ej.state_core().player(1).hp, hp1 - 1, "Liuxinghuoyu dealt 1 fire dmg");
    let out_hp2 = play_card(&mut ej, 0, heart_tao2, vec![1]);
    assert!(matches!(out_hp2, CommandOutcome::Rejected(_)), "ref_huopu usable:1 blocks 2nd conversion");

    // 2) `hs_kcthun`: `shahai-hearth__ref_luanji` unlimited uses in same Play phase (`shenhua/skill.js:7318`) + `xianji`
    let mut ek = fresh_proto(11_103, &["hs_kcthun", "hs_lrexxar"]);
    assert!(has_skill(&ek, 0, "xianji"));
    assert!(has_skill(&ek, 0, "shahai-hearth__ref_luanji"));
    advance_to_phase(&mut ek, 0, Phase::Play);
    clear_hand(&mut ek, 0);
    clear_hand(&mut ek, 1);
    for round in 0..2 {
        let s1 = give_free_card_kind_suit(&mut ek, 0, CardKind::Sha, Some(Suit::Spade));
        let s2 = give_free_card_kind_suit(&mut ek, 0, CardKind::Dismantle, Some(Suit::Spade));
        let out_lj = activate(&mut ek, 0, "shahai-hearth__ref_luanji");
        assert!(!matches!(out_lj, CommandOutcome::Rejected(_)), "ref_luanji round {round} accepted");
        assert_eq!(ek.open_choice().map(|c| c.spec.prompt), Some("luanji_discard"));
        answer(&mut ek, 0, vec![ChoiceOption::Card(s1), ChoiceOption::Card(s2)]);
        drain_auto(&mut ek);
    }
    assert_eq!(ek.state_core().player(1).hp, 2, "two luanji ArcheryAttacks in same Play phase dealt 2 dmg");

    // 3) `hs_xsylvanas`: `shahai-hearth__ref_busi` (`unique: true`) persists across two dyings + `xmojian`
    let mut ex = fresh_proto(11_104, &["hs_xsylvanas", "hs_lrexxar"]);
    assert!(has_skill(&ex, 0, "xshixin"));
    assert!(has_skill(&ex, 0, "xmojian"));
    assert!(has_skill(&ex, 0, "shahai-hearth__ref_busi"));
    advance_to_phase(&mut ex, 1, Phase::Play);
    clear_hand(&mut ex, 0);
    clear_hand(&mut ex, 1);
    for _dying_round in 0..2 {
        ex.state_core_mut().player_mut(0).hp = 1;
        ex.state_core_mut().turn.sha_used_play_phase = false;
        let heart_j = give_free_card_kind_suit(&mut ex, 0, CardKind::Tao, Some(Suit::Heart));
        put_card_on_deck_top(&mut ex, heart_j);
        clear_hand(&mut ex, 0);
        let sha = give_free_card_kind_suit(&mut ex, 1, CardKind::Sha, None);
        let _ = play_card(&mut ex, 1, sha, vec![0]);
        drain_auto(&mut ex);
        assert!(ex.state_core().player(0).alive, "ref_busi saved hs_xsylvanas");
        assert_eq!(ex.state_core().player(0).hp, 1);
        assert!(has_skill(&ex, 0, "shahai-hearth__ref_busi"), "ref_busi must persist (unique:true)");
    }
}

#[test]
fn u05_p3a_hearth_hs_alleria_xinliegong() {
    let mut e = fresh_proto(11_105, &["hs_alleria", "hs_jaina", "hs_lrexxar", "hs_wuther"]);
    assert!(has_skill(&e, 0, "fengxing"));
    assert!(has_skill(&e, 0, "xinliegong"));
    let xlg_def = e.catalog().skill_by_name("xinliegong").unwrap();
    assert!(e.catalog().skills[xlg_def as usize].behavior.package_skill_id().is_none());

    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);

    // Distance 0 -> 2 in 4p ring is 2. Base attack range is 1.
    // Find rank-1 Sha (rejected OutOfRange) and rank>=2 Sha (accepted via xinliegong distance <= rank):
    let sha_rank1 = {
        let cat = e.catalog();
        (0..cat.cards.len() as u32)
            .find(|&id| cat.cards[id as usize].kind.is_sha_like() && cat.cards[id as usize].rank == 1)
            .unwrap_or_else(|| panic!("no rank-1 Sha in catalog"))
    };
    let sha_rank_hi = {
        let cat = e.catalog();
        (0..cat.cards.len() as u32)
            .find(|&id| cat.cards[id as usize].kind.is_sha_like() && cat.cards[id as usize].rank >= 3)
            .unwrap()
    };
    remove_cid_everywhere(&mut e, sha_rank1);
    e.state_core_mut().player_mut(0).hand.push(sha_rank1);
    assert!(matches!(
        play_card(&mut e, 0, sha_rank1, vec![2]),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "rank-1 Sha cannot reach distance 2");

    // Now give seat 0 sha_rank_hi + 1 extra card (so after using sha_rank_hi, hand == 1 >= target hand == 1),
    // and give target 2 a Shan + equal HP (>= 0.hp) -> both directHit AND +1 damage trigger!
    clear_hand(&mut e, 0);
    remove_cid_everywhere(&mut e, sha_rank_hi);
    e.state_core_mut().player_mut(0).hand.push(sha_rank_hi);
    let _extra0 = give_free_card_kind_suit(&mut e, 0, CardKind::Tao, None);
    let _shan2 = give_free_card_kind_suit(&mut e, 2, CardKind::Shan, None);
    e.state_core_mut().player_mut(0).hp = 3;
    e.state_core_mut().player_mut(2).hp = 4;

    let out = play_card(&mut e, 0, sha_rank_hi, vec![2]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "rank>=2 Sha reaches dist 2 via xinliegong");
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("hs_xinliegong_confirm"));
    assert_eq!(ai_choose_current(&e), vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e);
    assert_eq!(
        e.state_core().player(2).hp,
        2,
        "xinliegong made Sha unrespondable (ignored Shan2) AND +1 damage (4 -> 2)"
    );

    // When tieji-sealed, xinliegong rank-based attack range bypass must NOT apply:
    clear_hand(&mut e, 0);
    remove_cid_everywhere(&mut e, sha_rank_hi);
    e.state_core_mut().player_mut(0).hand.push(sha_rank_hi);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "sbtieji_target_sealed".to_string()), 1);
    assert!(matches!(
        play_card(&mut e, 0, sha_rank_hi, vec![2]),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ), "tieji-sealed xinliegong cannot bypass attack range");
}

fn freeze_b6b_pkg(dir_name: &str) -> content_native_loader::FrozenPackage {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir_name);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .unwrap_or_else(|e| panic!("read {dir_name}: {e}"))
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = std::collections::BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    content_native_loader::validate_and_freeze_raw_package(raw_json, assets).expect("freeze")
}

#[test]
fn u05_p3a_hearth_hs_tgolem_xinwuyan() {
    let mut e = fresh_proto(11_106, &["hs_tgolem", "hs_lrexxar"]);
    assert!(has_skill(&e, 0, "xinwuyan"));
    assert!(has_skill(&e, 0, "guozaix"));
    let xwy_def = e.catalog().skill_by_name("xinwuyan").unwrap();
    assert!(e.catalog().skills[xwy_def as usize].behavior.package_skill_id().is_none());

    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    // 1) Trick damage FROM hs_tgolem prevented:
    let aoe = give_free_card_kind_suit(&mut e, 0, CardKind::SavageAttack, None);
    let hp1_before = e.state_core().player(1).hp;
    play_card(&mut e, 0, aoe, vec![]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before, "xinwuyan prevents trick dmg dealt by owner");

    // 2) Sha damage FROM hs_tgolem NOT prevented:
    let sha = give_free_card_kind_suit(&mut e, 0, CardKind::Sha, None);
    play_card(&mut e, 0, sha, vec![1]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "xinwuyan does NOT prevent basic Sha damage");

    // 3) Trick damage TO hs_tgolem prevented:
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let hp0_before = e.state_core().player(0).hp;
    let aoe1 = give_free_card_kind_suit(&mut e, 1, CardKind::SavageAttack, None);
    play_card(&mut e, 1, aoe1, vec![]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_before, "xinwuyan prevents trick dmg taken by owner");

    // 4) Delay trick Shandian (`type == "delay"` -> `is_trick_kind()`) TO hs_tgolem IS prevented:
    let sd = find_card_matching(&e, "shandian");
    let spade5 = {
        let cat = e.catalog();
        (0..cat.cards.len() as u32)
            .find(|&id| cat.cards[id as usize].suit == Suit::Spade && (2..=9).contains(&cat.cards[id as usize].rank))
            .unwrap()
    };
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    remove_cid_everywhere(&mut e, sd);
    e.state_core_mut().player_mut(0).hand.push(sd);
    play_card(&mut e, 0, sd, vec![]);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 1, Phase::Play);
    put_card_on_deck_top(&mut e, spade5);
    let hp0_pre_sd = e.state_core().player(0).hp;
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.state_core().player(0).hp,
        hp0_pre_sd,
        "Shandian (delay/trick) MUST be prevented by xinwuyan"
    );

    // 5) Multi-pack check: Huoshan (delay -> prevented) vs Gwent spell (`gw_zhuoshao`) & Zhenfa (`niaoxiangzhen`) ARE NOT prevented!
    let verified_multi = content_native_loader::prepare_locked_content_with_bridge(vec![
        freeze_b6b_pkg("shahai-hearth-upstream-noname"),
        freeze_b6b_pkg("shahai-gwent-upstream-noname"),
        freeze_b6b_pkg("shahai-yunchou-upstream-noname"),
        freeze_b6b_pkg("shahai-zhenfa-upstream-noname"),
    ])
    .expect("prepare hearth+gwent+yunchou+zhenfa");
    let mut em = content_native_loader::start_locked_engine(
        11_116,
        4,
        content_native_loader::LockedStartMode::Prototype,
        &verified_multi,
    )
    .expect("start hearth+gwent+yunchou+zhenfa");
    em.assign_general_named(0, "hs_tgolem").unwrap();
    advance_to_phase(&mut em, 0, Phase::Play);
    for p in 0..4 {
        clear_hand(&mut em, p);
        em.state_core_mut().player_mut(p).hp = 4;
    }
    let hs = (0..em.catalog().cards.len() as u32)
        .find(|&id| {
            let n = em.catalog().cards[id as usize].name;
            n == "huoshan" || n.starts_with("yc_huoshan")
        })
        .expect("huoshan card in yunchou pack");
    let heart5 = {
        let cat = em.catalog();
        (0..cat.cards.len() as u32)
            .find(|&id| id != hs && cat.cards[id as usize].suit == Suit::Heart && (2..=9).contains(&cat.cards[id as usize].rank))
            .unwrap()
    };
    remove_cid_everywhere(&mut em, hs);
    em.state_core_mut().player_mut(0).hand.push(hs);
    play_card(&mut em, 0, hs, vec![]);
    drain_auto(&mut em);
    advance_to_phase(&mut em, 3, Phase::Play);
    put_card_on_deck_top(&mut em, heart5);
    let hp0_pre_hs = em.state_core().player(0).hp;
    let hp1_pre_hs = em.state_core().player(1).hp;
    advance_to_phase(&mut em, 0, Phase::Play);
    assert_eq!(
        em.state_core().player(0).hp,
        hp0_pre_hs,
        "Huoshan (delay/trick) damage to owner MUST be prevented by xinwuyan"
    );
    assert_eq!(
        em.state_core().player(1).hp,
        hp1_pre_hs - 1,
        "Huoshan actually triggered Heart 2-9 hit (dealt splash 1 to seat 1 without xinwuyan)"
    );
    advance_to_phase(&mut em, 1, Phase::Play);
    for p in 0..4 {
        clear_hand(&mut em, p);
        em.state_core_mut().player_mut(p).hp = if p == 0 { 4 } else { 3 };
    }
    let zhuoshao = find_card_matching(&em, "gw_zhuoshao");
    remove_cid_everywhere(&mut em, zhuoshao);
    em.state_core_mut().player_mut(1).hand.push(zhuoshao);
    let out_zs = play_card(&mut em, 1, zhuoshao, vec![0]);
    assert!(!matches!(out_zs, CommandOutcome::Rejected(_)), "gw_zhuoshao rejected: {out_zs:?}");
    drain_auto(&mut em);
    assert_eq!(
        em.state_core().player(0).hp,
        3,
        "gw_zhuoshao ('spell', not 'trick') MUST NOT be prevented by xinwuyan"
    );

    // Also test `gwmaoxian_haluo` (`type: "gwmaoxian"`, deals 1 dmg to min_hp players) while seat 0 (`hs_tgolem`) is still on seat 0:
    for p in 0..4 {
        em.state_core_mut().player_mut(p).hp = if p == 0 { 2 } else { 4 };
    }
    let haluo = find_card_matching(&em, "gwmaoxian_haluo");
    remove_cid_everywhere(&mut em, haluo);
    em.state_core_mut().player_mut(1).hand.push(haluo);
    let out_haluo = play_card(&mut em, 1, haluo, vec![]);
    assert!(!matches!(out_haluo, CommandOutcome::Rejected(_)), "gwmaoxian_haluo rejected: {out_haluo:?}");
    drain_auto(&mut em);
    assert_eq!(
        em.state_core().player(0).hp,
        1,
        "gwmaoxian_haluo ('gwmaoxian', not 'trick') MUST NOT be prevented by xinwuyan"
    );

    // Hearth derived off-deck (`_1`) non-trick damage cards MUST NOT be prevented by xinwuyan,
    // while real Hearth trick cards (`yuansuhuimie`, `linghunzhihuo`) MUST be prevented!
    advance_to_phase(&mut e, 1, Phase::Play);
    for p in 0..2 {
        clear_hand(&mut e, p);
        e.state_core_mut().player_mut(p).max_hp = 8;
        e.state_core_mut().player_mut(p).hp = 8;
    }
    // Positive controls: real Hearth tricks `yuansuhuimie` and `linghunzhihuo` MUST be prevented!
    let ys = find_card_matching(&e, "yuansuhuimie");
    remove_cid_everywhere(&mut e, ys);
    e.state_core_mut().player_mut(1).hand.push(ys);
    assert!(!matches!(play_card(&mut e, 1, ys, vec![]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, 8, "yuansuhuimie (trick) MUST be prevented by xinwuyan");

    let lh = find_card_matching(&e, "linghunzhihuo");
    remove_cid_everywhere(&mut e, lh);
    e.state_core_mut().player_mut(1).hand.push(lh);
    assert!(!matches!(play_card(&mut e, 1, lh, vec![0]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    clear_hand(&mut e, 1);
    assert_eq!(e.state_core().player(0).hp, 8, "linghunzhihuo (trick) MUST be prevented by xinwuyan");

    // Positive control sharing `CardKind::Wuzhong`: `shandianjian` (type: "trick") MUST still be prevented!
    let _s0_heart = give_free_card_kind_suit(&mut e, 0, CardKind::Tao, Some(Suit::Heart));
    let s1_heart = give_free_card_kind_suit(&mut e, 1, CardKind::Tao, Some(Suit::Heart));
    let sdj = find_card_matching(&e, "shandianjian");
    remove_cid_everywhere(&mut e, sdj);
    e.state_core_mut().player_mut(1).hand.push(sdj);
    assert!(!matches!(play_card(&mut e, 1, sdj, vec![0]), CommandOutcome::Rejected(_)));
    if let Some(ch) = e.open_choice().cloned() {
        let pick = if ch.spec.options.contains(&ChoiceOption::Card(s1_heart)) {
            vec![ChoiceOption::Card(s1_heart)]
        } else {
            ch.spec.options.iter().take(1).copied().collect()
        };
        answer(&mut e, ch.observer, pick);
    }
    drain_auto(&mut e);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    assert_eq!(e.state_core().player(0).hp, 8, "shandianjian (`CardKind::Wuzhong`, type='trick') MUST be prevented by xinwuyan");

    // Non-trick Hearth off-deck `_1` damage cards MUST NOT be prevented by xinwuyan:
    for (cname, targets, expected_hp) in [
        ("hsjixie_zhadan_1", vec![], 7),
        ("hsqizhou_huo_1", vec![0], 6),
        ("hsqingyu_shandian_1", vec![0], 5),
        ("hsdusu_huoyanhua_1", vec![0], 4),
        ("hsmengjing_feicuiyoulong_1", vec![0], 3),
        ("hsshenqi_kongbusangzhong_1", vec![], 2),
    ] {
        let cid = find_card_matching(&e, cname);
        remove_cid_everywhere(&mut e, cid);
        e.state_core_mut().player_mut(1).hand.push(cid);
        let out = play_card(&mut e, 1, cid, targets);
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{cname} rejected: {out:?}");
        drain_auto(&mut e);
        assert_eq!(
            e.state_core().player(0).hp,
            expected_hp,
            "{cname} (non-trick derived card) MUST NOT be prevented by xinwuyan"
        );
    }

    // Zhenfa `niaoxiangzhen` (`type: "zhenfa"`): seat 0/1 Wei, seat 2/3 Shu + xinwuyan on seat 2
    for p in 0..4 {
        em.debug_remove_general(p);
        em.state_core_mut().player_mut(p).hp = 4;
    }
    em.debug_assemble_general(0, "曹操");
    em.debug_assemble_general(1, "曹操");
    em.debug_assemble_general(2, "刘备");
    em.debug_assemble_general(3, "刘备");
    em.debug_grant(2, "xinwuyan");
    advance_to_phase(&mut em, 0, Phase::Play);
    for p in 0..4 {
        clear_hand(&mut em, p);
    }
    let nx = find_card_matching(&em, "niaoxiangzhen");
    remove_cid_everywhere(&mut em, nx);
    em.state_core_mut().player_mut(0).hand.push(nx);
    let out_nx = play_card(&mut em, 0, nx, vec![2, 3]);
    assert!(!matches!(out_nx, CommandOutcome::Rejected(_)), "niaoxiangzhen rejected: {out_nx:?}");
    drain_auto(&mut em);
    assert_eq!(
        em.state_core().player(2).hp,
        3,
        "niaoxiangzhen ('zhenfa', not 'trick') MUST NOT be prevented by xinwuyan"
    );
}

#[test]
fn u05_p3a_hearth_hs_wuther_rejieming() {
    let mut e = fresh_proto(11_107, &["hs_wuther", "hs_lrexxar"]);
    assert!(has_skill(&e, 0, "fengxian"));
    assert!(has_skill(&e, 0, "rejieming"));
    let rjm_def = e.catalog().skill_by_name("rejieming").unwrap();
    assert!(e.catalog().skills[rjm_def as usize].behavior.package_skill_id().is_none());

    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    // Give 1 Jiu + Sha to deal 2 dmg so rejieming triggers twice:
    let jiu = give_free_card_kind_suit(&mut e, 1, CardKind::Jiu, None);
    let sha = give_free_card_kind_suit(&mut e, 1, CardKind::Sha, None);
    // Pad seat 1 hand to 4 (= max_hp) so picking 1 draws 2 without granting owner +1 bonus:
    for _ in 0..2 {
        let _ = give_free_card_kind_suit(&mut e, 1, CardKind::Tao, None);
    }
    let out_jiu = play_card(&mut e, 1, jiu, vec![]);
    assert!(!matches!(out_jiu, CommandOutcome::Rejected(_)), "jiu rejected: {out_jiu:?}");
    drain_auto(&mut e);
    let out_sha = play_card(&mut e, 1, sha, vec![0]);
    assert!(!matches!(out_sha, CommandOutcome::Rejected(_)), "sha rejected: {out_sha:?}");

    // Trigger 1 of 2: AI picks self (0, friendly & hand < max_hp 4), save/restore round-trip works:
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_rejieming_target"),
        "expected hs_rejieming_target; hp0={}, hp1={}, jiu_active={}",
        e.state_core().player(0).hp,
        e.state_core().player(1).hp,
        e.state_core().turn.jiu_active,
    );
    assert_eq!(ai_choose_current(&e), vec![ChoiceOption::Player(0)]);
    save_restore_and_answer(&mut e, "hs_rejieming_target", vec![ChoiceOption::Player(0)]);
    // Seat 0 had 0 cards -> draws 2 -> still 2 < max_hp(4) -> owner draws +1 (= 3 total)!
    assert_eq!(e.state_core().player(0).hand.len(), 3, "0 draws 2 + 1 bonus when hand < max_hp");

    // Trigger 2 of 2: Pass rejected on SelectPlayers (no Pass option), vec![] cancels cleanly, or pick seat 1:
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("hs_rejieming_target"));
    let out_bad_pass = answer(&mut e, 0, vec![ChoiceOption::Pass]);
    assert!(
        matches!(out_bad_pass, CommandOutcome::Rejected(RejectReason::InvalidChoice)),
        "SelectPlayers without ChoiceOption::Pass must reject Pass: {out_bad_pass:?}"
    );
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hand.len(), 4, "1 drew 2");
    assert_eq!(e.state_core().player(0).hand.len(), 3, "no owner bonus when target hand >= max_hp");

    // Verify vec![] cancel is accepted on hs_rejieming_target too (advance into next round for seat 1):
    advance_to_phase(&mut e, 0, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha2 = give_free_card_kind_suit(&mut e, 1, CardKind::Sha, None);
    let out_sha2 = play_card(&mut e, 1, sha2, vec![0]);
    assert!(!matches!(out_sha2, CommandOutcome::Rejected(_)), "sha2 rejected: {out_sha2:?}");
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("hs_rejieming_target"));
    let out_cancel = answer(&mut e, 0, vec![]);
    assert!(!matches!(out_cancel, CommandOutcome::Rejected(_)), "vec![] cancel accepted: {out_cancel:?}");
    drain_auto(&mut e);
}
