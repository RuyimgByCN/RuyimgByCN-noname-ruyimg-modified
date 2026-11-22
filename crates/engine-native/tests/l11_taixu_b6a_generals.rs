//! Batch L11-6a: All 320 `L11` (`太虚幻境`) generals static-field, skill-binding, instantiation,
//! Hujia/starting-HP, `"double"`-gender, and unshadowed host cross-skill parity tests.
//!
//! Upstream source: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_character.js:6-337`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    bridge_pack, decision34_pack9_for_coexistence, lookup_l11_general_profile,
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent, L11_TAIXU_GENERALS,
    L11_TAIXU_GENERAL_PROFILES, L11_TAIXU_SKILL_ID, L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::content::extension::content_lock_id;
use engine_native::content::standard::Gender;
use engine_native::state::{general_starting_hp, general_starting_hujia, GameMode};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

#[path = "l11_taixu_b6a/round3_fixes.rs"]
mod round3_fixes;

static TAIXU_B6A_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static B6A_DEDUP: AtomicU64 = AtomicU64::new(61_000_000);

pub(super) fn next_dedup() -> u64 {
    B6A_DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub(super) fn load_frozen_pkg(dir_name: &str) -> content_native_loader::FrozenPackage {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir_name);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .unwrap_or_else(|e| panic!("read {dir_name} package.npack.json: {e}"))
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap_or_else(|e| panic!("read {dir_name} asset {rel_path}: {e}"))
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    validate_and_freeze_raw_package(raw_json, assets)
        .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"))
}

pub(super) fn load_frozen_taixu() -> content_native_loader::FrozenPackage {
    load_frozen_pkg("taixu-upstream-liuli")
}

pub(super) fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_B6A_VERIFIED
        .get_or_init(|| prepare_locked_content_with_bridge(vec![load_frozen_taixu()]).unwrap())
}

pub(super) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub(super) fn answer_choice(e: &mut Engine, actor: u8, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let cid = e.open_choice().expect("expected open choice").id;
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    )
}

pub(super) fn drain_to_pass(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel && c.spec.min == 0 {
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

pub(super) fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if e.open_choice().is_some() {
            drain_to_pass(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, seat);
    assert_eq!(e.turn().phase, Phase::Play);
    assert!(e.open_choice().is_none());
}

pub(super) fn take_card_kind(e: &mut Engine, kind: CardKind) -> u32 {
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| e.catalog().card_kind(c) == kind)
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| e.catalog().card_kind(c) == kind)
    {
        return e.state_core_mut().discard.remove(pos);
    }
    for seat in 0..e.state_core().players.len() {
        if let Some(pos) = e
            .state_core()
            .player(seat as u8)
            .hand
            .iter()
            .position(|&c| e.catalog().card_kind(c) == kind)
        {
            return e.state_core_mut().player_mut(seat as u8).hand.remove(pos);
        }
    }
    panic!("missing card kind {kind:?}");
}

#[test]
fn l11_all_320_generals_static_fields_bindings_and_v107_lock_parity() {
    assert_eq!(L11_TAIXU_GENERALS.len(), 320);
    assert_eq!(L11_TAIXU_GENERAL_PROFILES.len(), 320);

    let mut unique_ids = BTreeSet::new();
    let mut shen_count = 0usize;
    let mut double_gender_count = 0usize;
    let mut reduced_hp_count = 0usize;
    let mut hujia_count = 0usize;
    let mut unseen_count = 0usize;
    let mut dualside_count = 0usize;

    for (idx, (&(id, faction, gender, max_hp, skills), profile)) in L11_TAIXU_GENERALS
        .iter()
        .zip(L11_TAIXU_GENERAL_PROFILES.iter())
        .enumerate()
    {
        assert!(unique_ids.insert(id), "duplicate L11 general id {id} at {idx}");
        assert_eq!(profile.id, id);
        assert_eq!(profile.faction, faction);
        assert_eq!(profile.gender, gender);
        assert_eq!(profile.max_hp, max_hp);
        assert_eq!(profile.skills, skills);
        assert!(
            profile.tags.contains(&"forbidai"),
            "{id} must preserve upstream forbidai tag"
        );
        assert_eq!(
            general_starting_hp(id, max_hp),
            profile.starting_hp as i32,
            "general_starting_hp mismatch for {id}"
        );
        assert_eq!(
            general_starting_hujia(id),
            profile.starting_hujia,
            "general_starting_hujia mismatch for {id}"
        );
        if profile.upstream_kingdom == "shen" {
            assert_eq!(faction, Faction::Qun);
            shen_count += 1;
        }
        if profile.upstream_sex == "double" {
            assert_eq!(id, "txhj_niufudongxie");
            double_gender_count += 1;
        }
        if profile.starting_hp < profile.max_hp {
            reduced_hp_count += 1;
        }
        if profile.starting_hujia > 0 {
            hujia_count += 1;
        }
        if profile.tags.contains(&"unseen") {
            unseen_count += 1;
        }
        if profile.tags.iter().any(|t| t.starts_with("dualside:")) {
            dualside_count += 1;
        }
    }
    assert_eq!(unique_ids.len(), 320);
    assert_eq!(shen_count, 12);
    assert_eq!(double_gender_count, 1);
    assert_eq!(reduced_hp_count, 28);
    assert_eq!(hujia_count, 6);
    assert_eq!(unseen_count, 2);
    assert_eq!(dualside_count, 2);

    // Check formal Catalog registration + skill resolution for every one of the 320 generals:
    let verified = taixu_verified();
    let e = start_locked_engine(2026_601, 2, LockedStartMode::Prototype, verified)
        .expect("start locked taixu");
    for profile in &L11_TAIXU_GENERAL_PROFILES {
        let g_idx = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some(profile.id))
            .unwrap_or_else(|| panic!("L11 general {} missing from Catalog", profile.id));
        let (_, fac, attached, mhp, gen) = e.catalog().general_meta(g_idx).unwrap();
        assert_eq!(fac, profile.faction);
        assert_eq!(mhp, profile.max_hp);
        assert_eq!(gen, profile.gender);
        let expected_defined: Vec<&str> = profile
            .skills
            .iter()
            .copied()
            .filter(|s| !L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS.contains(s))
            .collect();
        assert_eq!(
            attached.len(),
            expected_defined.len(),
            "defined skill count mismatch on {}",
            profile.id
        );
        for sk in &attached {
            let sid = e
                .catalog()
                .skill_by_name(sk)
                .unwrap_or_else(|| panic!("skill {sk} missing on {}", profile.id));
            assert_ne!(
                e.catalog().skills[sid as usize].name, "",
                "skill name empty for {sk}"
            );
        }
    }

    // Native HostSkillDecl must NOT be shadowed by `taixu-upstream-liuli/package.npack.json` VM stubs:
    for native_cross in ["reyingzi", "taoluan", "xinfu_xionghuo", "xinhongyan"] {
        let sid = e
            .catalog()
            .skill_by_name(native_cross)
            .unwrap_or_else(|| panic!("expected {native_cross} in Catalog"));
        assert!(
            e.catalog().skills[sid as usize]
                .behavior
                .package_skill_id()
                .is_none(),
            "{native_cross} must bind to native HostBehavior rather than npack VM stub"
        );
        assert_ne!(Some(L11_TAIXU_SKILL_ID), e.catalog().skills[sid as usize].behavior.package_skill_id());
    }

    // Preserve `decision34_pack9_for_coexistence` + `u01-dddanzhi-pending-v107.ngsa` bit-identically:
    let d3_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let d3_src = std::fs::read_to_string(d3_path)
        .expect("read d3 character.json")
        .replace("\r\n", "\n");
    let d3_pack = bridge_pack(&d3_src).expect("bridge d3").pack;
    let coex_pack = decision34_pack9_for_coexistence(&[&d3_pack]);
    assert_eq!(content_lock_id(&[&coex_pack]), 9_097_112_637_251_845_257);
    let ngsa_bytes = include_bytes!("fixtures/u01-dddanzhi-pending-v107.ngsa");
    let snap = decode_archive(ngsa_bytes).expect("decode v107 fixture");
    assert_eq!(encode_archive(&snap), ngsa_bytes);
    let restored_v107 = Engine::restore_with_extensions(&snap, &[&d3_pack, &coex_pack])
        .expect("restore v107 fixture with coexistence pack");
    assert_eq!(restored_v107.open_choice().unwrap().spec.prompt, "dddanzhi_target");
}

#[test]
fn l11_all_320_generals_assign_general_sets_exact_hp_max_hp_hujia_and_skills() {
    let verified = taixu_verified();
    let mut e = start_locked_engine(2026_602, 2, LockedStartMode::Prototype, verified)
        .expect("start locked taixu");

    for profile in &L11_TAIXU_GENERAL_PROFILES {
        e.assign_general_named(0, profile.id)
            .unwrap_or_else(|err| panic!("assign_general_named({}) failed: {err:?}", profile.id));
        let p0 = e.state_core().player(0);
        assert_eq!(p0.max_hp, profile.max_hp, "max_hp mismatch for {}", profile.id);
        assert_eq!(
            p0.hp, profile.starting_hp as i32,
            "starting_hp mismatch for {}",
            profile.id
        );
        assert_eq!(
            p0.hujia, profile.starting_hujia,
            "starting_hujia mismatch for {}",
            profile.id
        );
        assert!(p0.alive && !p0.dying, "alive/dying invalid for {}", profile.id);

        let g_idx = p0.general_slots[0].expect("slot 0 set");
        let (_, _, attached, _, _) = e.catalog().general_meta(g_idx).unwrap();
        for sk in attached {
            let sid = e.catalog().skill_by_name(sk).unwrap();
            assert!(
                e.state_core().has_skill(0, sid),
                "seat 0 missing active skill {sk} after assign_general_named({})",
                profile.id
            );
        }
    }

    // Identity Mode Lord (`+1 max_hp`, `+1 starting_hp`, preserved `hujia`) verification:
    let mut id_e = start_locked_engine(2026_603, 5, LockedStartMode::Identity, verified)
        .expect("start identity taixu");
    let lord_seat = id_e
        .state_core()
        .identity_roles()
        .and_then(|r| r.iter().position(|role| *role == engine_native::state::Role::Lord))
        .expect("lord seat") as u8;
    let non_lord_seat = if lord_seat == 0 { 1 } else { 0 };
    id_e.assign_general_named(lord_seat, "txhj_molijue")
        .expect("assign lord molijue");
    id_e.assign_general_named(non_lord_seat, "txhj_digongzhangbaox")
        .expect("assign non-lord zhangbaox");
    assert_eq!(id_e.state_core().player(lord_seat).max_hp, 22);
    assert_eq!(id_e.state_core().player(lord_seat).hp, 18);
    assert_eq!(id_e.state_core().player(lord_seat).hujia, 2);
    assert_eq!(id_e.state_core().player(non_lord_seat).max_hp, 25);
    assert_eq!(id_e.state_core().player(non_lord_seat).hp, 25);
    assert_eq!(id_e.state_core().player(non_lord_seat).hujia, 7);
}

#[test]
fn l11_hujia_double_gender_unshadowed_host_skills_and_save_restore_parity() {
    let verified = taixu_verified();

    // 1. Real combat with `txhj_molijue` (`17/21/2`) & Hujia 1:1 damage absorption + save/restore:
    let mut e = start_locked_engine(2026_604, 2, LockedStartMode::Prototype, verified)
        .expect("start taixu");
    e.assign_general_named(0, "txhj_reganning").unwrap();
    e.assign_general_named(1, "txhj_molijue").unwrap();
    assert_eq!(e.state_core().player(1).hp, 17);
    assert_eq!(e.state_core().player(1).max_hp, 21);
    assert_eq!(e.state_core().player(1).hujia, 2);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let sha1 = take_card_kind(&mut e, CardKind::Sha);
    let sha2 = take_card_kind(&mut e, CardKind::Sha);
    e.state_core_mut().player_mut(0).hand.push(sha1);
    e.state_core_mut().player_mut(0).hand.push(sha2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain_to_pass(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 1, "1st Sha absorbs 1 hujia");
    assert_eq!(e.state_core().player(1).hp, 17, "hp stays 17 while hujia > 0");
    let snap = e.save();
    let restored = restore_locked_engine(&snap, verified).expect("restore after hujia");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert_eq!(restored.state_core().player(1).hujia, 1);

    // 2. Upstream `"double"` gender on `txhj_niufudongxie` (`extension_character.js:175`):
    //    `matches_gender(Male) && matches_gender(Female)`, `different_gender_from` BOTH Male & Female,
    //    and real `Cixiong` sword duel check against Male (`txhj_guanyu`) and Female (`txhj_redaqiao`)!
    for (opp_name, expect_cixiong) in [
        ("txhj_guanyu", true),
        ("txhj_redaqiao", true),
    ] {
        let mut eg = start_locked_engine(2026_605, 2, LockedStartMode::Prototype, verified)
            .expect("start double gender match");
        eg.assign_general_named(0, "txhj_niufudongxie").unwrap();
        eg.assign_general_named(1, opp_name).unwrap();
        assert!(eg.state_core().has_double_gender(eg.catalog(), 0));
        assert!(eg.state_core().matches_gender(eg.catalog(), 0, Gender::Male));
        assert!(eg.state_core().matches_gender(eg.catalog(), 0, Gender::Female));
        assert!(eg.state_core().different_gender_from(eg.catalog(), 0, 1));
        assert!(eg.state_core().same_gender_as(eg.catalog(), 0, 1));

        let cixiong = take_card_kind(&mut eg, CardKind::Cixiong);
        let sha = take_card_kind(&mut eg, CardKind::Sha);
        let opp_hand = take_card_kind(&mut eg, CardKind::Tao);
        advance_to_play(&mut eg, 0);
        eg.debug_clear_hand(0);
        eg.debug_clear_hand(1);
        eg.state_core_mut().player_mut(1).hand.push(opp_hand);
        eg.state_core_mut().player_mut(0).hand.push(cixiong);
        let _ = submit(
            &mut eg,
            0,
            GameAction::PlayCard {
                card: cixiong,
                targets: vec![],
            },
        );
        drain_to_pass(&mut eg);
        eg.state_core_mut().player_mut(0).hand.push(sha);
        let out = submit(
            &mut eg,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        let opened_cixiong = match out {
            CommandOutcome::Pending { choice } => choice.spec.prompt == "cixiong_discard_choose",
            _ => false,
        };
        assert_eq!(
            opened_cixiong, expect_cixiong,
            "double-sex txhj_niufudongxie Cixiong vs {opp_name}"
        );
    }
    // Negative control: male `txhj_guanyu` vs male `txhj_xusheng` does NOT trigger Cixiong:
    let mut eg_neg = start_locked_engine(2026_606, 2, LockedStartMode::Prototype, verified).unwrap();
    eg_neg.assign_general_named(0, "txhj_guanyu").unwrap();
    eg_neg.assign_general_named(1, "txhj_xusheng").unwrap();
    assert!(!eg_neg.state_core().different_gender_from(eg_neg.catalog(), 0, 1));

    // 3. Unshadowed native HostSkillDecl execution (`taoluan` on `txhj_huanyanzhangrang` +
    //    `xinfu_xionghuo` on `txhj_xurongy` `2/5`) + pending save/restore -> Answer:
    let mut el = start_locked_engine(2026_607, 2, LockedStartMode::Prototype, verified).unwrap();
    el.assign_general_named(0, "txhj_huanyanzhangrang").unwrap();
    el.assign_general_named(1, "txhj_xurongy").unwrap();
    assert_eq!(el.state_core().player(1).hp, 2);
    assert_eq!(el.state_core().player(1).max_hp, 5);
    let prof_zr = lookup_l11_general_profile("txhj_huanyanzhangrang").unwrap();
    assert_eq!(prof_zr.source_line, 336);
    advance_to_play(&mut el, 0);
    let taoluan_id = el.catalog().skill_by_name("taoluan").unwrap();
    let extra = take_card_kind(&mut el, CardKind::Shan);
    el.state_core_mut().player_mut(0).hand.push(extra);
    let CommandOutcome::Pending { choice } =
        submit(&mut el, 0, GameAction::ActivateSkill { skill: taoluan_id })
    else {
        panic!("expected native HostBehavior taoluan active choice on txhj_huanyanzhangrang");
    };
    assert_eq!(choice.spec.prompt, "taoluan:choose_card");
    let pending_snap = el.save();
    let mut restored_pending = restore_locked_engine(&pending_snap, verified).unwrap();
    assert_eq!(restored_pending.state_hash(), el.state_hash());
    let _ = answer_choice(&mut restored_pending, choice.observer, Vec::new());

    // 4. Multi-faction + 0-skill general assembly smoke (`txhj_shenguanyu`, `txhj_mayunlu`,
    //    `txhj_redaqiao`, `txhj_simazhao`, `txhj_huanjingshouhutongzi`) + `xinfu_xionghuo` activation:
    for (gname, expected_fac, expected_hp, min_skills) in [
        ("txhj_shenguanyu", Faction::Qun, 4, 2usize),
        ("txhj_mayunlu", Faction::Shu, 4, 2),
        ("txhj_redaqiao", Faction::Wu, 3, 2),
        ("txhj_simazhao", Faction::Wei, 3, 2),
        ("txhj_huanjingshouhutongzi", Faction::Qun, 2, 0),
    ] {
        let mut em = start_locked_engine(2026_608, 2, LockedStartMode::Prototype, verified).unwrap();
        em.assign_general_named(0, gname).unwrap();
        assert_eq!(em.state_core().effective_faction(em.catalog(), 0), Some(expected_fac));
        assert_eq!(em.state_core().player(0).hp, expected_hp);
        assert_eq!(em.state_core().player(0).max_hp, expected_hp as u32);
        let g_idx = em.state_core().player(0).general_slots[0].unwrap();
        assert!(em.catalog().general_meta(g_idx).unwrap().2.len() >= min_skills);
    }

    let mut ex = start_locked_engine(2026_609, 2, LockedStartMode::Prototype, verified).unwrap();
    ex.assign_general_named(0, "txhj_xurongy").unwrap();
    ex.assign_general_named(1, "txhj_reganning").unwrap();
    advance_to_play(&mut ex, 0);
    let xh_id = ex.catalog().skill_by_name("xinfu_xionghuo").unwrap();
    let CommandOutcome::Pending { choice } =
        submit(&mut ex, 0, GameAction::ActivateSkill { skill: xh_id })
    else {
        panic!("expected native HostBehavior xinfu_xionghuo target prompt on txhj_xurongy");
    };
    let _ = answer_choice(&mut ex, choice.observer, vec![ChoiceOption::Player(1)]);
    assert_eq!(
        ex.state_core().vm_aux.flags.get(&(1, "xinfu_xionghuo_marked".to_string())).copied(),
        Some(1)
    );
    assert_eq!(
        ex.state_core().vm_aux.flags.get(&(0, "xinfu_xionghuo_marks".to_string())).copied(),
        Some(2)
    );
}

fn take_card_matching(
    e: &mut Engine,
    mut pred: impl FnMut(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&mut pred))
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&mut pred))
    {
        return e.state_core_mut().discard.remove(pos);
    }
    for seat in 0..e.state_core().players.len() {
        if let Some(pos) = e
            .state_core()
            .player(seat as u8)
            .hand
            .iter()
            .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&mut pred))
        {
            return e.state_core_mut().player_mut(seat as u8).hand.remove(pos);
        }
    }
    panic!("missing matching card in deck/discard/hand");
}

#[test]
fn l11_round2_std_catalog_xueji_feiying_jieyin_and_double_gender_regressions() {
    let verified = taixu_verified();

    // 1. Standard-Catalog skills (`liuli`, `longdan`, `mashu`, `wushuang`, `yicong`) must bind directly
    //    to `standard_catalog` (`流离`, `龙胆`, `马术`, `无双`, `义从`) rather than U05 VM stubs,
    //    plus live distance/skill checks (`太虚幻境/extension_character.js`).
    {
        let mut es = start_locked_engine(2026_610, 4, LockedStartMode::Prototype, verified)
            .expect("start locked taixu for std skill parity");
        for (gname, expected_std_zh) in [
            ("txhj_redaqiao", "流离"),
            ("txhj_zhaoyuny", "龙胆"),
            ("txhj_machao", "马术"),
            ("txhj_relvbu", "无双"),
            ("txhj_baimaxiaoqi", "义从"),
        ] {
            es.assign_general_named(0, gname).unwrap();
            let g_idx = es.state_core().player(0).general_slots[0].unwrap();
            let attached = es.catalog().general_meta(g_idx).unwrap().2;
            assert!(
                attached.contains(&expected_std_zh),
                "{gname} must bind standard skill {expected_std_zh:?}, got {attached:?}"
            );
            let sid = es.catalog().skill_by_name(expected_std_zh).unwrap();
            assert!(
                es.catalog().skills[sid as usize].behavior.package_skill_id().is_none(),
                "{expected_std_zh} must remain native standard behavior"
            );
            assert!(es.state_core().has_skill(0, sid));
        }

        // Live `马术`: `txhj_huanxiangbingnv` at seat 0 reduces distance to opposite seat 2 (2 -> 1).
        es.assign_general_named(0, "txhj_huanxiangbingnv").unwrap();
        assert_eq!(es.state_core().computed_distance(es.catalog(), 0, 2), 1);

        // Live `义从`: `txhj_baimaxiaoqi` (3/4 HP -> globalFrom -1, hp<=2 => globalTo +1).
        es.assign_general_named(0, "txhj_baimaxiaoqi").unwrap();
        assert_eq!(es.state_core().computed_distance(es.catalog(), 0, 2), 1);
        assert_eq!(es.state_core().computed_distance(es.catalog(), 2, 0), 2);
        es.debug_set_hp(0, 2);
        assert_eq!(es.state_core().computed_distance(es.catalog(), 0, 2), 1);
        assert_eq!(es.state_core().computed_distance(es.catalog(), 2, 0), 3);

        // Live `无双`: `txhj_relvbu` playing Sha requires 2 Shan responses.
        es.assign_general_named(0, "txhj_relvbu").unwrap();
        es.assign_general_named(1, "txhj_zhaoyuny").unwrap();
        advance_to_play(&mut es, 0);
        es.debug_clear_hand(0);
        es.debug_clear_hand(1);
        let sha = take_card_kind(&mut es, CardKind::Sha);
        let shan1 = take_card_kind(&mut es, CardKind::Shan);
        let longdan_sha = take_card_kind(&mut es, CardKind::Sha);
        es.state_core_mut().player_mut(0).hand.push(sha);
        es.state_core_mut().player_mut(1).hand.push(shan1);
        es.state_core_mut().player_mut(1).hand.push(longdan_sha);
        let CommandOutcome::Pending { choice } = submit(
            &mut es,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        ) else {
            panic!("expected Shan response prompt against wushuang Sha");
        };
        // Native `龙胆` lets `txhj_zhaoyuny` use `Sha` alongside `Shan` as response!
        assert!(
            choice.spec.options.contains(&ChoiceOption::Card(shan1))
                && choice.spec.options.contains(&ChoiceOption::Card(longdan_sha)),
            "txhj_zhaoyuny standard 龙胆 must allow Sha as Shan response"
        );
        let _ = answer_choice(&mut es, 1, vec![ChoiceOption::Card(shan1)]);
        // Still needs 2nd Shan due to native `无双`!
        assert!(
            es.open_choice().is_some(),
            "txhj_relvbu standard 无双 must require 2nd Shan response"
        );
        let _ = answer_choice(&mut es, 1, vec![ChoiceOption::Card(longdan_sha)]);
        drain_to_pass(&mut es);
        assert_eq!(es.state_core().player(1).hp, 6, "dodged wushuang Sha with 龙胆");
    }

    // 2. Authentic Guan Yinping `xueji` (`sp.js:21742-21777`) on `txhj_guanyinping` (`extension_character.js:219`):
    //    negative without red card, cancel refund, equip red card + multi-target link + Fire chain + save/restore + AI.
    {
        let mut eg = start_locked_engine(2026_611, 4, LockedStartMode::Identity, verified)
            .expect("start locked taixu for xueji");
        eg.assign_general_named(0, "txhj_guanyinping").unwrap();
        eg.assign_general_named(1, "txhj_machao").unwrap();
        eg.assign_general_named(2, "txhj_madai").unwrap();
        eg.assign_general_named(3, "txhj_guanyu").unwrap();
        let xueji_sid = eg.catalog().skill_by_name("xueji").unwrap();
        assert_ne!(
            eg.catalog().skills[xueji_sid as usize].behavior.package_skill_id(),
            Some(L11_TAIXU_SKILL_ID)
        );
        advance_to_play(&mut eg, 0);
        eg.debug_clear_hand(0);
        let black_sha = take_card_matching(&mut eg, |d| {
            d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        eg.state_core_mut().player_mut(0).hand.push(black_sha);
        assert!(
            matches!(
                submit(&mut eg, 0, GameAction::ActivateSkill { skill: xueji_sid }),
                CommandOutcome::Rejected(_)
            ),
            "xueji requires at least 1 red card in hand/equips"
        );

        let red_card = take_card_matching(&mut eg, |d| {
            !d.kind.is_equip() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        eg.state_core_mut().player_mut(0).hand.push(red_card);
        // Injured by 2 HP (`max(1, getDamagedHp())` == 2 targets max)
        let max0 = eg.state_core().player(0).max_hp as i32;
        eg.debug_set_hp(0, max0 - 2);
        let hp1_before = eg.state_core().player(1).hp;
        let hp2_before = eg.state_core().player(2).hp;

        // Cancel on `xueji:card` must keep red card & refund `once_used`:
        let CommandOutcome::Pending { choice } =
            submit(&mut eg, 0, GameAction::ActivateSkill { skill: xueji_sid })
        else {
            panic!("expected xueji:card");
        };
        assert_eq!(choice.spec.prompt, "xueji:card");
        assert!(choice.spec.allow_cancel);
        assert!(!choice.spec.options.contains(&ChoiceOption::Card(black_sha)));
        assert!(choice.spec.options.contains(&ChoiceOption::Card(red_card)));
        let _ = answer_choice(&mut eg, 0, Vec::new());
        assert!(eg.state_core().player(0).hand.contains(&red_card));

        // Cancel on `xueji:target` must ALSO keep red card & refund `once_used`:
        let _ = submit(&mut eg, 0, GameAction::ActivateSkill { skill: xueji_sid });
        let _ = answer_choice(&mut eg, 0, vec![ChoiceOption::Card(red_card)]);
        let t_ask = eg.open_choice().expect("xueji:target").clone();
        assert_eq!(t_ask.spec.prompt, "xueji:target");
        assert_eq!(t_ask.spec.min, 1);
        assert_eq!(t_ask.spec.max, 2);
        assert!(t_ask.spec.allow_cancel);
        let _ = answer_choice(&mut eg, 0, Vec::new());
        assert!(eg.state_core().player(0).hand.contains(&red_card));

        // Re-activate, pick red card, save/restore mid-`xueji:target`, select [seat 1, seat 2]:
        let _ = submit(&mut eg, 0, GameAction::ActivateSkill { skill: xueji_sid });
        let _ = answer_choice(&mut eg, 0, vec![ChoiceOption::Card(red_card)]);
        let mid_snap = eg.save();
        let mut eg_restored = restore_locked_engine(&mid_snap, verified).unwrap();
        assert_eq!(eg_restored.state_hash(), eg.state_hash());
        assert_eq!(
            eg_restored.open_choice().map(|c| c.spec.clone()),
            eg.open_choice().map(|c| c.spec.clone())
        );
        let _ = answer_choice(&mut eg, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
        drain_to_pass(&mut eg);
        let _ = answer_choice(
            &mut eg_restored,
            0,
            vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        );
        drain_to_pass(&mut eg_restored);
        assert_eq!(eg.state_hash(), eg_restored.state_hash());
        assert!(!eg.state_core().player(0).hand.contains(&red_card));
        assert!(eg.state_core().discard.contains(&red_card));
        assert_eq!(
            eg.state_core().player(1).hp,
            hp1_before - 1,
            "targets[0] must take 1 Fire damage"
        );
        assert_eq!(
            eg.state_core().player(2).hp,
            hp2_before - 1,
            "linked targets[1] must take chained 1 Fire damage"
        );
        assert!(
            matches!(
                submit(&mut eg, 0, GameAction::ActivateSkill { skill: xueji_sid }),
                CommandOutcome::Rejected(_)
            ),
            "xueji usable:1 rejects 2nd activation in same Play phase"
        );

        // AI (`BoundedPlanner`) on `xueji:card` and `xueji:target` (uses public Versus team attitude):
        let mut e_ai = start_locked_engine(2026_612, 4, LockedStartMode::Prototype, verified).unwrap();
        e_ai.assign_general_named(0, "txhj_guanyinping").unwrap();
        e_ai.assign_general_named(1, "txhj_machao").unwrap();
        e_ai.assign_general_named(2, "txhj_madai").unwrap();
        e_ai.assign_general_named(3, "txhj_guanyu").unwrap();
        e_ai.state_core_mut().mode = GameMode::Versus {
            teams: vec![vec![0, 1], vec![2, 3]],
            substitutes: vec![vec![], vec![]],
        };
        e_ai.debug_set_hp(0, 3);
        advance_to_play(&mut e_ai, 0);
        e_ai.debug_clear_hand(0);
        let red_sha = take_card_matching(&mut e_ai, |d| {
            d.kind == CardKind::Sha && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let red_tao = take_card_matching(&mut e_ai, |d| {
            d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        e_ai.state_core_mut().player_mut(0).hand.extend([red_sha, red_tao]);
        let _ = submit(&mut e_ai, 0, GameAction::ActivateSkill { skill: xueji_sid });
        let ai_card = e_ai.plan_action(0, &BoundedPlanner::new(8)).action;
        assert_eq!(
            ai_card,
            GameAction::Answer {
                choice_id: e_ai.open_choice().unwrap().id,
                selection: vec![ChoiceOption::Card(red_sha)],
            },
            "AI must pick lower-value red card (Sha over Tao)"
        );
        let _ = submit(&mut e_ai, 0, ai_card);
        let ai_tgt = e_ai.plan_action(0, &BoundedPlanner::new(8)).action;
        assert_eq!(
            ai_tgt,
            GameAction::Answer {
                choice_id: e_ai.open_choice().unwrap().id,
                selection: vec![ChoiceOption::Player(2)],
            },
            "AI must pick lowest-HP enemy seat 2 (max=1) and avoid ally seat 1"
        );
        // Cancel when no enemies remain on board:
        e_ai.state_core_mut().mode = GameMode::Versus {
            teams: vec![vec![0, 1, 2, 3], vec![]],
            substitutes: vec![vec![], vec![]],
        };
        assert_eq!(
            e_ai.plan_action(0, &BoundedPlanner::new(8)).action,
            GameAction::Answer {
                choice_id: e_ai.open_choice().unwrap().id,
                selection: Vec::new(),
            },
            "AI must cancel xueji:target when no enemies exist"
        );
    }

    // 3. Turn-1 `feiying` (`txhj_xuelan`, `txhj_huangfusongy`, `txhj_huangfusongx`) BEFORE owner PhaseBegin,
    //    plus stopping immediately when removed/disabled or sealed by `tieji`:
    {
        let mut ef = start_locked_engine(2026_613, 4, LockedStartMode::Prototype, verified)
            .expect("start locked taixu for Turn-1 feiying");
        ef.assign_general_named(0, "txhj_machao").unwrap();
        ef.assign_general_named(1, "txhj_xuelan").unwrap();
        ef.assign_general_named(2, "txhj_huangfusongy").unwrap();
        ef.assign_general_named(3, "txhj_huangfusongx").unwrap();
        // `txhj_machao` at seat 0 has `马术` (-1 from 0); check seat 2 -> seat 1 (no `马术`, base 1 + feiying 1 = 2)
        // and seat 0 -> seat 1 (base 1 + feiying 1 - mashu 1 = 1), BEFORE any PhaseBegin runs!
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 2, 1), 2);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 1), 1);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 1, 2), 2);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 3), 1);
        ef.assign_general_named(0, "txhj_guanyu").unwrap();
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 1), 2);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 3), 2);
        // Disable `feiying` on seat 1 -> immediately drops back to 1
        ef.debug_set_skill_enabled(1, "feiying", false);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 1), 1);
        // Re-enable and seal via `fz_new_tieji_target_sealed` -> sealed `feiying` stops granting +1
        ef.debug_set_skill_enabled(1, "feiying", true);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 1), 2);
        ef.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "fz_new_tieji_target_sealed".to_string()), 1);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 0, 1), 1);
    }

    // 4. `JieYinBehavior::activate` + `save -> restore` on `L11` male (`txhj_handiliuxie`) & `"double"`-sex
    //    `txhj_niufudongxie`, negative female (`txhj_redaqiao`), `effective_gender_override == 0` vs `1`/`2`,
    //    and `U08` `fengliu` counting alive `"double"`-sex `txhj_niufudongxie`:
    {
        for target_gen in ["txhj_handiliuxie", "txhj_niufudongxie"] {
            let mut ej = start_locked_engine(2026_614, 3, LockedStartMode::Prototype, verified).unwrap();
            let sx = (0..ej.catalog().general_count() as u16)
                .find(|&g| ej.catalog().general_meta(g).map(|m| m.0) == Some("孙尚香"))
                .expect("孙尚香");
            ej.assign_general(0, sx).unwrap();
            ej.assign_general_named(1, target_gen).unwrap();
            ej.assign_general_named(2, "txhj_redaqiao").unwrap();
            ej.debug_set_hp(0, 2);
            ej.debug_set_hp(1, 2);
            ej.debug_set_hp(2, 2); // injured female must NOT appear in `jieyin_target`
            advance_to_play(&mut ej, 0);
            ej.debug_clear_hand(0);
            let c1 = take_card_kind(&mut ej, CardKind::Sha);
            let c2 = take_card_kind(&mut ej, CardKind::Shan);
            ej.state_core_mut().player_mut(0).hand.extend([c1, c2]);
            let jy_sid = ej.catalog().skill_by_name("结姻").unwrap();
            let CommandOutcome::Pending { choice } =
                submit(&mut ej, 0, GameAction::ActivateSkill { skill: jy_sid })
            else {
                panic!("JieYin activate must succeed for {target_gen}");
            };
            assert_eq!(choice.spec.prompt, "jieyin_cards");
            let _ = answer_choice(&mut ej, 0, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
            let snap_jy = ej.save();
            let mut ej_restored = restore_locked_engine(&snap_jy, verified).unwrap();
            assert_eq!(
                ej_restored.open_choice().map(|c| c.spec.clone()),
                ej.open_choice().map(|c| c.spec.clone()),
                "jieyin_target snapshot must include {target_gen} and omit female seat 2"
            );
            assert_eq!(
                ej.open_choice().unwrap().spec.options,
                vec![ChoiceOption::Player(1)]
            );
            let _ = answer_choice(&mut ej, 0, vec![ChoiceOption::Player(1)]);
            let _ = answer_choice(&mut ej_restored, 0, vec![ChoiceOption::Player(1)]);
            assert_eq!(ej.state_hash(), ej_restored.state_hash());
            assert_eq!(ej.state_core().player(0).hp, 3);
            assert_eq!(ej.state_core().player(1).hp, 3);
        }

        // Negative: only injured other player is female (`txhj_redaqiao`) -> JieYin Rejected!
        let mut ej_neg = start_locked_engine(2026_615, 2, LockedStartMode::Prototype, verified).unwrap();
        let sx = (0..ej_neg.catalog().general_count() as u16)
            .find(|&g| ej_neg.catalog().general_meta(g).map(|m| m.0) == Some("孙尚香"))
            .unwrap();
        ej_neg.assign_general(0, sx).unwrap();
        ej_neg.assign_general_named(1, "txhj_redaqiao").unwrap();
        ej_neg.debug_set_hp(0, 2);
        ej_neg.debug_set_hp(1, 2);
        advance_to_play(&mut ej_neg, 0);
        let jy_sid = ej_neg.catalog().skill_by_name("结姻").unwrap();
        assert!(matches!(
            submit(&mut ej_neg, 0, GameAction::ActivateSkill { skill: jy_sid }),
            CommandOutcome::Rejected(_)
        ));

        // `effective_gender_override == 0` must preserve `double` gender, whereas `1`/`2` overrides it:
        ej_neg.assign_general_named(1, "txhj_niufudongxie").unwrap();
        ej_neg.state_core_mut().vm_aux.flags.insert((1, "effective_gender_override".to_string()), 0);
        assert!(ej_neg.state_core().has_double_gender(ej_neg.catalog(), 1));
        assert!(ej_neg.state_core().matches_gender(ej_neg.catalog(), 1, Gender::Male));
        assert!(ej_neg.state_core().matches_gender(ej_neg.catalog(), 1, Gender::Female));
        ej_neg.state_core_mut().vm_aux.flags.insert((1, "effective_gender_override".to_string()), 1);
        assert!(!ej_neg.state_core().has_double_gender(ej_neg.catalog(), 1));
        assert!(ej_neg.state_core().matches_gender(ej_neg.catalog(), 1, Gender::Male));
        assert!(!ej_neg.state_core().matches_gender(ej_neg.catalog(), 1, Gender::Female));

        // `U08` `fengliu` (`count_alive_females`) counts `"double"`-sex `txhj_niufudongxie` + Female `txhj_redaqiao`:
        let verified_u08 = prepare_locked_content_with_bridge(vec![
            load_frozen_pkg("taixu-upstream-liuli"),
            load_frozen_pkg("hero-kill-upstream-noname"),
        ])
        .unwrap();
        let mut efeng = start_locked_engine(2026_616, 3, LockedStartMode::Prototype, &verified_u08).unwrap();
        efeng.assign_general_named(0, "yxs_lvzhi").unwrap();
        efeng.assign_general_named(1, "txhj_niufudongxie").unwrap();
        efeng.assign_general_named(2, "txhj_machao").unwrap();
        efeng.debug_grant(0, "fengliu");
        efeng.debug_clear_hand(0);
        efeng.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut efeng, 0, GameAction::NextPhase); // Judge -> Draw (draws 2 + fengliu alive_females)
        drain_to_pass(&mut efeng);
        // `yxs_lvzhi` (Female/double? Female) + `txhj_niufudongxie` (double -> matches Female) = 2 alive females -> 2 base + 2 fengliu = 4
        let _ = submit(&mut efeng, 0, GameAction::NextPhase); // Draw -> Play
        assert_eq!(
            efeng.state_core().player(0).hand.len(),
            4,
            "fengliu must count double-gender txhj_niufudongxie as alive female (2 base + 2 = 4)"
        );
    }
}
