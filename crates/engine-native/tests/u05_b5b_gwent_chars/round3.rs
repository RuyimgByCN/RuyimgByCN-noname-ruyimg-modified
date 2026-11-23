//! U05-5b Round 3 缺陷回归（对应 #105）：
//! 1) `junchi_playable_cards` 對 `Sha`/`gw_wuyao` 須走完整 `is_single_target_legal_no_dist`（`blocks_all_cards`、`eval_card_use_canceled`、`eval_card_target_canceled`、`gj_food_molicha`、`热闷`）；
//! 2) `b5b_target_canceled` 中 `BOSS_REMEN`（热闷）仅当无护甲且牌种为无属性平杀（`CardKind::Sha`, `Normal`）时生效，`FireSha`/`LeiSha` 与已装备护甲目标豁免；补充 `has_zhuge_clean` 校验 `gw_dieyi_card` 假标连弩；
//! 3) `hupeng2` 对 `gw_dudayuanshuai2` 不可弃置锁补全至 `nuhou`、`shewu`、`gwzhanjiang`、`fayin`、`gwhuanbi`、`gwyewu`（含 `randomDiscard`），并连同 `shuijian`/`yinzhang`/`gwxiaoshou`/`hupeng` 一并覆盖。

use std::collections::BTreeMap;

use super::common::*;
use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
    LockedStartMode, VerifiedLockedContent,
};
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

fn load_pkg(ext_root: &std::path::Path, dir: &str) -> content_native_loader::FrozenPackage {
    let pkg_dir = ext_root.join(dir);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read pkg asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    validate_and_freeze_raw_package(raw_json, assets).expect("freeze pkg")
}

fn setup_gwent_plus_gujian_locked(seed: u64) -> (VerifiedLockedContent, engine_native::Engine) {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let verified = prepare_locked_content_with_bridge(vec![
        load_pkg(&ext_root, "shahai-gwent-upstream-noname"),
        load_pkg(&ext_root, "shahai-gujian-upstream-noname"),
    ])
    .expect("prepare gwent + gujian locked content");
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Prototype, &verified)
        .expect("start locked gwent+gujian engine");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        let h = std::mem::take(&mut p.hand);
        snap.core.discard.extend(h);
        let eq = std::mem::take(&mut p.equips);
        snap.core.discard.extend(eq);
        let jg = std::mem::take(&mut p.judge);
        snap.core.discard.extend(jg);
        let oc = std::mem::take(&mut p.off_court);
        snap.core.discard.extend(oc);
        p.hp = 4;
        p.max_hp = 4;
        p.face_down = false;
        p.linked = false;
        p.hujia = 0;
        p.boss_mad = false;
        p.yinshen_active = false;
    }
    snap.core.skills.clear();
    snap.core.vm_aux.flags.clear();
    snap.core.vm_aux.marks.clear();
    snap.core.vm_aux.piles.clear();
    snap.core.turn.extra_phases.clear();
    snap.core.turn.round = 1;
    let e = content_native_loader::restore_locked_engine(&snap, &verified).unwrap();
    (verified, e)
}

#[test]
fn u05_b5b_r3_sharp105_junchi_sha_checks_legal_use_and_target_cancel() {
    // 1a) With shahai-gujian loaded, target seat 1 holds `gj_food_molicha`: junchi excludes black Sha + gw_wuyao, keeps red Sha
    let (vg, eg0) = setup_gwent_plus_gujian_locked(5660);
    let g_spade_sha = card_of_kind_and_suit(&eg0, CardKind::Sha, Some(Suit::Spade));
    let g_diamond_sha = card_of_kind_and_suit(&eg0, CardKind::Sha, Some(Suit::Diamond));
    let g_heart_sha = card_of_kind_and_suit(&eg0, CardKind::Sha, Some(Suit::Heart));
    let g_wuyao = find_card(&eg0, "gw_wuyao");
    let mut eg = grant_skill(&eg0, &vg, "junchi", 0);
    advance_to_seat_play(&mut eg, 2);
    // Grant after advancing past seat 1's Phase::End so `on_phase_end_gujian`'s `food_tick`
    // does not expire a `rounds=0` fixture grant before seat 2 acts.
    eg = grant_skill(&eg, &vg, "gj_food_molicha", 1);
    eg = clear_hand(&eg, &vg, 0);
    eg = clear_hand(&eg, &vg, 1);
    eg = clear_hand(&eg, &vg, 2);
    eg = inject_hand(&eg, &vg, &[g_spade_sha, g_wuyao, g_diamond_sha], 0);
    eg = inject_hand(&eg, &vg, &[g_heart_sha], 2);
    let p_out = submit(
        &mut eg,
        2,
        GameAction::PlayCard {
            card: g_heart_sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(p_out, CommandOutcome::Rejected(_)), "g_heart_sha accepted: {p_out:?}");
    pass_window(&mut eg);
    let jc = eg.open_choice().cloned().expect("junchi_use vs molicha");
    assert_eq!(jc.spec.prompt, "junchi_use");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(g_spade_sha)),
        "junchi must exclude black Sha against gj_food_molicha target"
    );
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(g_wuyao)),
        "junchi must exclude black gw_wuyao against gj_food_molicha target"
    );
    assert!(
        jc.spec.options.contains(&ChoiceOption::Card(g_diamond_sha)),
        "junchi keeps red Sha against gj_food_molicha target"
    );
    answer_cancel(&mut eg, 0);
    drain_default(&mut eg);

    let (v, e0) = setup_gwent_locked(5661);
    let spade_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let diamond_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Diamond));
    let heart_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Heart));
    let wuyao = find_card(&e0, "gw_wuyao"); // Suit::Club -> black

    // 1b) Target seat 1 has `u05_mod:molicha` (eval_card_target_canceled) AND owner seat 0 has `olsbguqu_restricted_suit` = Spade (eval_card_use_canceled):
    // Spade Sha excluded via eval_card_use_canceled; Club gw_wuyao excluded via eval_card_target_canceled; Diamond Sha stays legal!
    let mut e = grant_skill(&e0, &v, "junchi", 0);
    advance_to_seat_play(&mut e, 2);
    e = clear_hand(&e, &v, 0);
    e = clear_hand(&e, &v, 1);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[spade_sha, wuyao, diamond_sha], 0);
    e = inject_hand(&e, &v, &[heart_sha], 2);
    let mut snap = e.save();
    snap.core
        .vm_aux
        .flags
        .insert((0, "olsbguqu_restricted_suit".to_string()), 1); // Spade blocked by eval_card_use_canceled
    snap.core
        .vm_aux
        .flags
        .insert((1, "u05_mod:molicha".to_string()), 1); // black cards blocked by eval_card_target_canceled
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    let p_out = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: heart_sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(p_out, CommandOutcome::Rejected(_)), "heart_sha accepted: {p_out:?}");
    pass_window(&mut e);
    let jc = e.open_choice().cloned().expect("junchi_use with use/target cancel flags");
    assert_eq!(jc.spec.prompt, "junchi_use");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(spade_sha)),
        "junchi must respect eval_card_use_canceled on Sha"
    );
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(wuyao)),
        "junchi must respect eval_card_target_canceled on black gw_wuyao"
    );
    assert!(
        jc.spec.options.contains(&ChoiceOption::Card(diamond_sha)),
        "junchi retains uncanceled Diamond Sha"
    );
    answer_cancel(&mut e, 0);
    drain_default(&mut e);

    // 1c) Owner seat 0 under `blocks_all_cards` (`chixueqingfeng2_count > 0`): junchi must NOT trigger or offer Sha
    let mut e = grant_skill(&e0, &v, "junchi", 0);
    advance_to_seat_play(&mut e, 2);
    e = clear_hand(&e, &v, 0);
    e = clear_hand(&e, &v, 1);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[diamond_sha], 0);
    e = inject_hand(&e, &v, &[heart_sha], 2);
    let mut snap = e.save();
    snap.core
        .vm_aux
        .flags
        .insert((0, "chixueqingfeng2_count".to_string()), 1);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    let p_out = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: heart_sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(p_out, CommandOutcome::Rejected(_)), "heart_sha accepted: {p_out:?}");
    pass_window(&mut e);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "junchi_use"),
        "junchi must not offer Sha when owner is blocked by blocks_all_cards"
    );
    drain_default(&mut e);
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r3_sharp105_remen_only_blocks_plain_sha_without_armor() {
    let (v, e0) = setup_gwent_locked(5662);
    let plain_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Diamond));
    let fire_sha = card_of_kind_and_suit(&e0, CardKind::FireSha, None);
    let thunder_sha = card_of_kind_and_suit(&e0, CardKind::LeiSha, None);
    let wuyao = find_card(&e0, "gw_wuyao");
    let armor = find_cards_by(&e0, |d| d.kind.equip_slot() == Some(EquipSlot::Armor), 1)[0];
    let zhuge = card_of_kind_and_suit(&e0, CardKind::Zhuge, None);
    let shans = find_cards_by(&e0, |d| d.kind == CardKind::Shan, 4);

    let make_3p = |eng: engine_native::Engine| -> engine_native::Engine {
        let mut snap = eng.save();
        snap.core.players[3].hp = 0;
        snap.core.players[3].alive = false;
        content_native_loader::restore_locked_engine(&snap, &v).unwrap()
    };

    // 2a) Unarmed BOSS_REMEN (`热闷`) seat 1 blocks plain Sha/gw_wuyao, allows FireSha/LeiSha (lanquan + sqlongwu)
    let mut e = make_3p(grant_skill(&e0, &v, "lanquan", 0));
    e = grant_skill(&e, &v, "热闷", 1);
    e = set_deck_top(
        &e,
        &v,
        &[plain_sha, wuyao, fire_sha, thunder_sha, shans[0], shans[1]],
    );
    activate(&mut e, 0, "lanquan");
    let lqp = e.open_choice().cloned().expect("lanquan_pick vs unarmed remen");
    assert!(
        lqp.spec.options.contains(&ChoiceOption::Card(fire_sha)),
        "lanquan must allow FireSha when target has 热闷 without armor"
    );
    assert!(
        lqp.spec.options.contains(&ChoiceOption::Card(thunder_sha)),
        "lanquan must allow LeiSha when target has 热闷 without armor"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: lqp.id,
            selection: vec![ChoiceOption::Card(plain_sha)],
        },
    );
    let lqt = e.open_choice().cloned().expect("lanquan_target plain Sha vs unarmed remen");
    assert!(
        !lqt.spec.options.contains(&ChoiceOption::Player(1)),
        "unarmed 热闷 seat 1 blocks plain Sha"
    );
    assert!(lqt.spec.options.contains(&ChoiceOption::Player(2)));
    answer_cancel(&mut e, 0);

    // Unarmed remen allows FireSha targeting seat 1 in lanquan
    let mut e = make_3p(grant_skill(&e0, &v, "lanquan", 0));
    e = grant_skill(&e, &v, "热闷", 1);
    e = set_deck_top(
        &e,
        &v,
        &[fire_sha, thunder_sha, shans[0], shans[1], shans[2], shans[3]],
    );
    activate(&mut e, 0, "lanquan");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(fire_sha));
    let lqt = e.open_choice().cloned().expect("lanquan_target FireSha vs unarmed remen");
    assert!(
        lqt.spec.options.contains(&ChoiceOption::Player(1)),
        "lanquan FireSha must allow unarmed 热闷 seat 1"
    );
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
    assert_eq!(e.debug_hp(1), 3, "FireSha dealt 1 fire damage through 热闷");

    // 2b) Armed BOSS_REMEN (`热闷` + Armor equipped) allows plain Sha (checked in sqlongwu + jielue + junchi)
    let mut e = make_3p(grant_skill(&e0, &v, "sqlongwu", 0));
    e = grant_skill(&e, &v, "热闷", 1);
    e = inject_equip(&e, &v, armor, 1);
    e = inject_hand(&e, &v, &[plain_sha], 0);
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "sqlongwu_use_card") {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    let sw = e.open_choice().cloned().expect("sqlongwu_use_card vs armed remen");
    assert!(
        sw.spec.options.contains(&ChoiceOption::Card(plain_sha)),
        "armed 热闷 allows plain Sha"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: sw.id,
            selection: vec![ChoiceOption::Card(plain_sha)],
        },
    );
    let swt = e.open_choice().cloned().expect("sqlongwu_use_target vs armed remen");
    assert!(
        swt.spec.options.contains(&ChoiceOption::Player(1)),
        "armed 热闷 seat 1 is legal plain Sha target"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: swt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "plain Sha via sqlongwu dealt 1 damage to armed 热闷 seat 1");

    // Unarmed remen on junchi target: blocks plain Sha & gw_wuyao, allows FireSha/LeiSha
    let mut e = make_3p(grant_skill(&e0, &v, "junchi", 0));
    e = grant_skill(&e, &v, "热闷", 1);
    advance_to_seat_play(&mut e, 2);
    e = clear_hand(&e, &v, 0);
    e = clear_hand(&e, &v, 1);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[plain_sha, wuyao, thunder_sha], 0);
    e = inject_hand(&e, &v, &[fire_sha], 2);
    let p_out = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: fire_sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(p_out, CommandOutcome::Rejected(_)), "fire_sha accepted: {p_out:?}");
    pass_window(&mut e);
    let jc = e.open_choice().cloned().expect("junchi_use vs unarmed remen");
    assert!(
        !jc.spec.options.contains(&ChoiceOption::Card(plain_sha))
            && !jc.spec.options.contains(&ChoiceOption::Card(wuyao)),
        "junchi excludes plain Sha and gw_wuyao against unarmed 热闷 seat 1"
    );
    assert!(
        jc.spec.options.contains(&ChoiceOption::Card(thunder_sha)),
        "junchi keeps LeiSha against unarmed 热闷 seat 1"
    );
    answer_cancel(&mut e, 0);
    drain_default(&mut e);

    // 2c) Bonus: gw_dieyi-masked Zhuge does NOT grant unlimited Sha quota in has_zhuge_clean
    let red_duel = card_of_kind_and_suit(&e0, CardKind::Duel, Some(Suit::Diamond));
    let mut e = make_3p(grant_skill(&e0, &v, "lanquan", 0));
    e = inject_equip(&e, &v, zhuge, 0);
    e = inject_hand(&e, &v, &[plain_sha], 0);
    let mut snap = e.save();
    snap.core
        .vm_aux
        .flags
        .insert((0, format!("gw_dieyi_card:{zhuge}")), 1);
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: plain_sha,
            targets: vec![1],
        },
    );
    pass_window(&mut e);
    e = set_deck_top(
        &e,
        &v,
        &[fire_sha, wuyao, red_duel, shans[0], shans[1], shans[2]],
    );
    activate(&mut e, 0, "lanquan");
    let lqp = e.open_choice().cloned().expect("lanquan_pick with masked Zhuge");
    assert!(
        lqp.spec.options.contains(&ChoiceOption::Card(red_duel)),
        "lanquan still offers non-Sha Duel"
    );
    assert!(
        !lqp.spec.options.contains(&ChoiceOption::Card(fire_sha))
            && !lqp.spec.options.contains(&ChoiceOption::Card(wuyao)),
        "gw_dieyi-masked Zhuge must NOT grant unlimited Sha quota"
    );
    answer_cancel(&mut e, 0);
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_r3_sharp105_hupeng2_locks_all_remaining_costs_and_random_discard() {
    let (v, e0) = setup_gwent_locked(5663);
    let d2 = find_card(&e0, "gw_dudayuanshuai2");
    let spade_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(Suit::Spade));
    let shans = find_cards_by(&e0, |d| d.kind == CardKind::Shan, 4);
    let du = card_of_kind_and_suit(&e0, CardKind::Duel, Some(Suit::Diamond));

    // a) ASK_NUHOU_COST & filter_group_d "nuhou" + step_nuhou randomDiscard
    // Only d2 in hand -> taking damage does NOT trigger nuhou
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "nuhou", 0);
    advance_to_seat_play(&mut e, 1);
    e = clear_hand(&e, &v, 0);
    e = clear_hand(&e, &v, 1);
    e = inject_hand(&e, &v, &[d2], 0);
    e = inject_hand(&e, &v, &[spade_sha], 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: spade_sha,
            targets: vec![0],
        },
    );
    pass_window(&mut e);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "nuhou_cost"),
        "nuhou must not trigger when owner only holds locked gw_dudayuanshuai2"
    );
    drain_default(&mut e);
    assert!(e.debug_hand(0).contains(&d2));

    // Owner has [shans[0], d2], enemy seat 1 has [shans[1], d2] -> nuhou_cost excludes d2, randomDiscard spares seat 1's d2
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "nuhou", 0);
    // Mark seat 3 dead so nuhou picks seat 1
    let mut snap = e.save();
    snap.core.players[3].hp = 0;
    snap.core.players[3].alive = false;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_play(&mut e, 1);
    e = clear_hand(&e, &v, 0);
    e = clear_hand(&e, &v, 1);
    e = inject_hand(&e, &v, &[shans[0], d2], 0);
    e = inject_hand(&e, &v, &[spade_sha, shans[1], d2], 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: spade_sha,
            targets: vec![0],
        },
    );
    pass_window(&mut e);
    let nc = e.open_choice().cloned().expect("nuhou_cost");
    assert_eq!(nc.spec.prompt, "nuhou_cost");
    assert!(
        !nc.spec.options.contains(&ChoiceOption::Card(d2)),
        "nuhou_cost must exclude gw_dudayuanshuai2"
    );
    assert_eq!(nc.spec.options, vec![ChoiceOption::Card(shans[0])]);
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: nc.id,
            selection: vec![ChoiceOption::Card(shans[0])],
        },
    );
    drain_default(&mut e);
    assert_eq!(
        e.debug_hand(1),
        vec![d2],
        "nuhou randomDiscard must discard shans[1] and never discard gw_dudayuanshuai2"
    );

    // b) ASK_SHEWU_COST & activate_group_d "shewu"
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "shewu", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    assert!(
        matches!(activate(&mut e, 0, "shewu"), CommandOutcome::Rejected(_)),
        "shewu cannot activate with only locked gw_dudayuanshuai2"
    );
    let mut e = inject_hand(&e, &v, &[shans[0], d2], 0);
    activate(&mut e, 0, "shewu");
    let sc = e.open_choice().cloned().expect("shewu_cost");
    assert_eq!(sc.spec.prompt, "shewu_cost");
    assert!(
        !sc.spec.options.contains(&ChoiceOption::Card(d2)),
        "shewu_cost must exclude gw_dudayuanshuai2"
    );
    answer_cancel(&mut e, 0);

    // c) ASK_GWZHANJIANG_COST & filter_group_d "gwzhanjiang"
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwzhanjiang", 0);
    advance_to_seat_play(&mut e, 2);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[d2], 0);
    advance_to_seat_prepare(&mut e, 3);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "gwzhanjiang_cost"),
        "gwzhanjiang must not trigger when owner only holds gw_dudayuanshuai2"
    );
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwzhanjiang", 0);
    advance_to_seat_play(&mut e, 2);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[shans[0], d2], 0);
    advance_to_seat_prepare(&mut e, 3);
    let zc = e.open_choice().cloned().expect("gwzhanjiang_cost");
    assert_eq!(zc.spec.prompt, "gwzhanjiang_cost");
    assert!(
        !zc.spec.options.contains(&ChoiceOption::Card(d2)),
        "gwzhanjiang_cost must exclude gw_dudayuanshuai2"
    );
    answer_cancel(&mut e, 0);

    // d) ASK_FAYIN_COST & try_spawn_fayin_sha_begin + step_fayin randomDiscard(2)
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "fayin", 0);
    e = inject_hand(&e, &v, &[spade_sha, d2], 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: spade_sha,
            targets: vec![1],
        },
    );
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "fayin_cost"),
        "fayin must not ask fayin_cost when only locked gw_dudayuanshuai2 remains"
    );
    pass_window(&mut e);

    // With [shans[0], d2] in attacker hand and [shans[1], du, d2] in victim hand, find seed rolling fayin case 0 (randomDiscard)
    let mut saw_fayin_discard = false;
    for seed in 5700u64..5740u64 {
        let (_, be0) = setup_gwent_locked(seed);
        let mut fe = grant_skill(&be0, &v, "hupeng", 2);
        fe = grant_skill(&fe, &v, "fayin", 0);
        fe = inject_hand(&fe, &v, &[spade_sha, shans[0], d2], 0);
        fe = inject_hand(&fe, &v, &[shans[1], du, d2], 1);
        submit(
            &mut fe,
            0,
            GameAction::PlayCard {
                card: spade_sha,
                targets: vec![1],
            },
        );
        let fc = fe.open_choice().cloned().expect("fayin_cost");
        assert_eq!(fc.spec.prompt, "fayin_cost");
        assert!(
            !fc.spec.options.contains(&ChoiceOption::Card(d2)),
            "fayin_cost must exclude gw_dudayuanshuai2"
        );
        submit(
            &mut fe,
            0,
            GameAction::Answer {
                choice_id: fc.id,
                selection: vec![ChoiceOption::Card(shans[0])],
            },
        );
        pass_window(&mut fe);
        drain_default(&mut fe);
        assert!(
            fe.debug_hand(1).contains(&d2),
            "fayin randomDiscard(2) must never discard gw_dudayuanshuai2 (seed {seed})"
        );
        if !fe.debug_hand(1).contains(&shans[1]) && !fe.debug_hand(1).contains(&du) {
            saw_fayin_discard = true;
            assert_eq!(
                fe.debug_hand(1),
                vec![d2],
                "fayin case 0 discarded both non-locked cards and spared d2"
            );
            break;
        }
    }
    assert!(saw_fayin_discard, "expected fayin randomDiscard branch across seeds");

    // e) ASK_GWHUANBI_COST & activate_group_a "gwhuanbi"
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwhuanbi", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    assert!(
        matches!(activate(&mut e, 0, "gwhuanbi"), CommandOutcome::Rejected(_)),
        "gwhuanbi cannot activate with only locked gw_dudayuanshuai2"
    );
    let mut e = inject_hand(&e, &v, &[shans[0], d2], 0);
    activate(&mut e, 0, "gwhuanbi");
    let hbc = e.open_choice().cloned().expect("gwhuanbi_cost");
    assert_eq!(hbc.spec.prompt, "gwhuanbi_cost");
    assert!(
        !hbc.spec.options.contains(&ChoiceOption::Card(d2)),
        "gwhuanbi_cost must exclude gw_dudayuanshuai2"
    );
    answer_cancel(&mut e, 0);

    // f) ASK_GWYEWU_COST & activate_group_a "gwyewu" + step_gwyewu randomDiscard
    // Case 1: owner only holds d2 -> rejected
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwyewu", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    e = inject_hand(&e, &v, &[shans[0]], 1);
    assert!(
        matches!(activate(&mut e, 0, "gwyewu"), CommandOutcome::Rejected(_)),
        "gwyewu cannot activate when owner only holds gw_dudayuanshuai2"
    );
    // Case 2: enemies only hold d2 -> rejected
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwyewu", 0);
    e = inject_hand(&e, &v, &[shans[0]], 0);
    e = inject_hand(&e, &v, &[d2], 1);
    assert!(
        matches!(activate(&mut e, 0, "gwyewu"), CommandOutcome::Rejected(_)),
        "gwyewu cannot activate when enemies only hold locked gw_dudayuanshuai2"
    );
    // Case 3: owner picks shans[0] (red, opposite color of spade/d2 or matching same color), enemy seat 1 holds [shans[1] (red), d2]
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "gwyewu", 0);
    e = inject_hand(&e, &v, &[shans[0], d2], 0);
    e = inject_hand(&e, &v, &[shans[1], d2], 1);
    // Seat 3 has empty hand; only seat 1 has discardable enemy cards
    activate(&mut e, 0, "gwyewu");
    let ywc = e.open_choice().cloned().expect("gwyewu_cost");
    assert_eq!(ywc.spec.prompt, "gwyewu_cost");
    assert!(
        !ywc.spec.options.contains(&ChoiceOption::Card(d2)),
        "gwyewu_cost must exclude gw_dudayuanshuai2"
    );
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ywc.id,
            selection: vec![ChoiceOption::Card(shans[0])],
        },
    );
    drain_default(&mut e);
    assert_eq!(
        e.debug_hand(1),
        vec![d2],
        "gwyewu randomDiscard must discard shans[1] and spare enemy gw_dudayuanshuai2"
    );

    // g) Additional self-discard cost guards in group_c (`shuijian`, `yinzhang`, `gwxiaoshou`)
    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "yinzhang", 0);
    e = grant_skill(&e, &v, "gwxiaoshou", 0);
    e = inject_hand(&e, &v, &[d2], 0);
    assert!(
        matches!(activate(&mut e, 0, "yinzhang"), CommandOutcome::Rejected(_)),
        "yinzhang cannot activate with only gw_dudayuanshuai2"
    );
    assert!(
        matches!(activate(&mut e, 0, "gwxiaoshou"), CommandOutcome::Rejected(_)),
        "gwxiaoshou cannot activate with only gw_dudayuanshuai2"
    );
    let mut e = inject_hand(&e, &v, &[shans[0], d2], 0);
    activate(&mut e, 0, "yinzhang");
    let yzc = e.open_choice().cloned().expect("yinzhang_cost");
    assert!(!yzc.spec.options.contains(&ChoiceOption::Card(d2)));
    answer_cancel(&mut e, 0);
    activate(&mut e, 0, "gwxiaoshou");
    let xsc = e.open_choice().cloned().expect("gwxiaoshou_cost");
    assert!(!xsc.spec.options.contains(&ChoiceOption::Card(d2)));
    answer_cancel(&mut e, 0);

    let mut e = grant_skill(&e0, &v, "hupeng", 2);
    e = grant_skill(&e, &v, "shuijian", 0);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[d2], 0);
    advance_to_seat_prepare(&mut e, 0);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "shuijian_cost"),
        "shuijian must not ask when owner only holds gw_dudayuanshuai2"
    );
    assert_zone_conservation(&e);
}
