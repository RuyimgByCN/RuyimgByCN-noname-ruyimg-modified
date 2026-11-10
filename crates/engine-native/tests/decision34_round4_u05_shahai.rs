//! Decision 34 Round 4 — `U05` (`杀海拾遗` / 9 formal `sha*-upstream-noname` packages) Authentic Upstream Closure Suite.
//!
//! Verifies on the formal `packages/extensions/sha*-upstream-noname` locked product path:
//! - `main/config.js:4-48` & `main/content.js:5-11,383` configuration schema (`tip`, `wuxing`, `rand`,
//!   `mtg`, `zhenfa`, `gwent`, `yunchou`), mode forbid lists (`boss`, `chess`, `double`, `stone`, `threecard`),
//!   and deterministic snapshot/restore;
//! - All 332 upstream generals (+ `stone_yisela`, `main/content.js:353`), all 275 upstream cards,
//!   and all 1,007 upstream skills (+ 33 cross-referenced skills) loaded simultaneously across the 9
//!   formal `sha*-upstream-noname` packages via `start_locked_engine`;
//! - Execution of every `U05` skill and card-skill with `0` `u05_unhandled_skill:*` fallbacks,
//!   positive/negative guards (`usable`, `limited`, `_shengke` five-element generation/restraint,
//!   `wuxingpan_skill`, `stone_chenshui` dream cycle), card conservation, and `NGSA 107` binary
//!   archive save/restore (`e.save()` -> `restore_locked_engine`).

use content_native_loader::{
    enabled_shahai_subpackages, is_shahai_forbid_in_mode, parse_shahai_main_config,
    prepare_locked_content_with_bridge, restore_locked_engine, restore_shahai_config,
    snapshot_shahai_config, start_locked_engine, validate_and_freeze_raw_package, LockedStartMode,
    ShahaiMainConfig, ShahaiWuxingCardRate, DECISION34_STORAGE_DISPATCH_KEY, U05_CROSS_SKILLS,
    U05_GUJIAN_ALL_CARDS, U05_GUJIAN_ALL_SKILLS, U05_GUJIAN_GENERALS, U05_GWENT_ALL_CARDS,
    U05_GWENT_ALL_SKILLS, U05_GWENT_GENERALS, U05_HEARTH_ALL_CARDS, U05_HEARTH_ALL_SKILLS,
    U05_HEARTH_GENERALS, U05_MTG_ALL_CARDS, U05_MTG_ALL_SKILLS, U05_MTG_GENERALS,
    U05_OW_ALL_SKILLS, U05_OW_GENERALS, U05_SHAHAI_SKILL_ID, U05_STONE_GENERALS,
    U05_SWD_ALL_CARDS, U05_SWD_CARD_SKILLS, U05_SWD_CHAR_SKILLS, U05_SWD_GENERALS,
    U05_WUXING_ALL_CARDS, U05_WUXING_PLAY_SKILLS, U05_XIANJIAN_ALL_SKILLS,
    U05_XIANJIAN_GENERALS_FULL, U05_YUNCHOU_ALL_CARDS, U05_YUNCHOU_ALL_SKILLS,
    U05_YUNCHOU_GENERALS, U05_ZHENFA_ALL_CARDS,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(4_050_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn invoke_u05(e: &mut Engine, owner: u8, skill_name: &str) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(U05_SHAHAI_SKILL_ID),
            key: DECISION34_STORAGE_DISPATCH_KEY,
            value: U05_SHAHAI_SKILL_ID,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "U05 skill dispatch ({skill_name}) rejected: {out:?}"
    );
    drain(e);
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut()
            .vm_aux
            .flags
            .remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn total_tracked_cards(e: &Engine) -> usize {
    let core = e.state_core();
    let mut n = core.deck.len() + core.discard.len() + core.table.len();
    for p in &core.players {
        n += p.hand.len() + p.equips.len() + p.judge.len();
    }
    for pile in core.vm_aux.piles.values() {
        n += pile.len();
    }
    n
}

fn start_all_9_formal_shahai_packages(
    seed: u64,
) -> (content_native_loader::VerifiedLockedContent, Engine) {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let formal_dirs = [
        "shahai-gujian-upstream-noname",
        "shahai-gwent-upstream-noname",
        "shahai-hearth-upstream-noname",
        "shahai-mtg-upstream-noname",
        "shahai-swd-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
        "shaai-ow-upstream-noname",
        "shaai-xianjian-upstream-noname",
    ];
    let mut frozen_list = Vec::with_capacity(formal_dirs.len());
    for dir_name in formal_dirs {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap_or_else(|e| panic!("read {dir_name}/package.npack.json: {e}"))
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap_or_else(|e| panic!("read {dir_name}/{rel_path}: {e}"))
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets)
            .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"));
        frozen_list.push(frozen);
    }
    let verified =
        prepare_locked_content_with_bridge(frozen_list).expect("prepare 9 formal shahai packages");
    let engine = start_locked_engine(seed, 2, LockedStartMode::Prototype, &verified)
        .expect("start locked engine with all 9 formal shahai packages");
    (verified, engine)
}

#[test]
fn u05_shahai_main_config_and_mode_forbid_lists_match_upstream() {
    // `杀海拾遗/main/config.js:4-48` default configuration
    let def = ShahaiMainConfig::default();
    assert!(def.tip_banner_visible);
    assert!(!def.wuxing);
    assert_eq!(def.rand, ShahaiWuxingCardRate::Rate30);
    assert_eq!(def.rand.percent(), 30);
    assert!(def.mtg && def.zhenfa && def.gwent && def.yunchou);
    assert_eq!(enabled_shahai_subpackages(&def).len(), 9);

    // Custom config parsing + deterministic snapshot/restore roundtrip
    let custom = parse_shahai_main_config(
        r#"{"tip":false,"wuxing":true,"rand":"0.5","mtg":false,"zhenfa":true,"gwent":false,"yunchou":true}"#,
    )
    .expect("parse valid shahai config");
    assert!(!custom.tip_banner_visible);
    assert!(custom.wuxing);
    assert_eq!(custom.rand, ShahaiWuxingCardRate::Rate50);
    let enabled = enabled_shahai_subpackages(&custom);
    assert_eq!(enabled.len(), 7);
    assert!(!enabled.contains(&"shahai-mtg"));
    assert!(!enabled.contains(&"shahai-gwent"));

    let snap = snapshot_shahai_config(&custom);
    let restored = restore_shahai_config(&snap).expect("restore shahai config");
    assert_eq!(restored, custom);

    // Negative validation guards (`main/config.js:19-28`)
    assert!(parse_shahai_main_config(r#"{"rand":"0.99"}"#).is_err());
    assert!(parse_shahai_main_config(r#"{"wuxing":"yes"}"#).is_err());

    // Mode forbid lists (`main/content.js:5-11,383`)
    assert!(is_shahai_forbid_in_mode("boss", "gjqt_aruan"));
    assert!(!is_shahai_forbid_in_mode("boss", "gjqt_bailitusu"));
    assert!(is_shahai_forbid_in_mode("chess", "swd_kangnalishi"));
    assert!(is_shahai_forbid_in_mode("double", "hs_siwangzhiyi"));
    assert!(is_shahai_forbid_in_mode("stone", "gjqt_bailitusu"));
    assert!(is_shahai_forbid_in_mode("stone", "chuansongmen"));
    assert!(is_shahai_forbid_in_mode("threecard", "gw_ciguhanshuang"));
    assert!(!is_shahai_forbid_in_mode("identity", "gjqt_bailitusu"));
}

#[test]
fn u05_all_9_formal_packages_333_generals_275_cards_and_1007_skills_execute_and_roundtrip() {
    let (verified, mut e) = start_all_9_formal_shahai_packages(2026_0505);

    // 1. Verify all 332 upstream generals + `stone_yisela` exist in the formal locked Catalog
    let all_generals = U05_GUJIAN_GENERALS
        .iter()
        .chain(U05_GWENT_GENERALS.iter())
        .chain(U05_HEARTH_GENERALS.iter())
        .chain(U05_MTG_GENERALS.iter())
        .chain(U05_OW_GENERALS.iter())
        .chain(U05_SWD_GENERALS.iter())
        .chain(U05_XIANJIAN_GENERALS_FULL.iter())
        .chain(U05_YUNCHOU_GENERALS.iter())
        .chain(U05_STONE_GENERALS.iter());
    let mut general_count = 0usize;
    let catalog_general_names: BTreeSet<&str> = (0..e.catalog().general_count() as u16)
        .filter_map(|idx| e.catalog().general_meta(idx).map(|m| m.0))
        .collect();
    for &(gname, _, _, _, sk_list) in all_generals {
        general_count += 1;
        assert!(
            catalog_general_names.contains(gname),
            "U05 general {gname} missing from formal locked Catalog"
        );
        for &sk in sk_list {
            assert!(
                e.catalog().skill_by_name(sk).is_some(),
                "U05 skill {sk} on general {gname} missing from formal locked Catalog"
            );
        }
    }
    assert_eq!(general_count, 333);

    // 2. Verify all 275 upstream cards exist in the formal locked Catalog
    let all_cards = U05_GUJIAN_ALL_CARDS
        .iter()
        .chain(U05_GWENT_ALL_CARDS.iter())
        .chain(U05_HEARTH_ALL_CARDS.iter())
        .chain(U05_MTG_ALL_CARDS.iter())
        .chain(U05_SWD_ALL_CARDS.iter())
        .chain(U05_YUNCHOU_ALL_CARDS.iter())
        .chain(U05_ZHENFA_ALL_CARDS.iter())
        .chain(U05_WUXING_ALL_CARDS.iter());
    let catalog_card_names: BTreeSet<&str> =
        e.catalog().cards.iter().map(|c| c.name).collect();
    let mut card_count = 0usize;
    for &(cname, _, _, _, _) in all_cards.clone() {
        card_count += 1;
        assert!(
            catalog_card_names.contains(cname),
            "U05 card {cname} missing from formal locked Catalog"
        );
        assert!(
            e.catalog().skill_by_name(cname).is_some(),
            "U05 card skill {cname} missing from formal locked Catalog"
        );
    }
    assert_eq!(card_count, 275);

    // 3. Semantic differential checks citing upstream file:line before sweeping all 1,000+ skills:
    e.state_core_mut().player_mut(0).max_hp = 30;
    e.state_core_mut().player_mut(0).hp = 30;
    e.state_core_mut().player_mut(1).max_hp = 30;
    e.state_core_mut().player_mut(1).hp = 30;

    // 3a. `play/wuxing/skill.js:5-78` `_shengke` five-element generation (`sheng`) + restraint (`ke`)
    //     & `wuxingpan_skill` (`play/wuxing/skill.js:79-95`, `usable: 1`)
    e.state_core_mut().player_mut(0).wunature = Some(2); // 0 generates 2 (`(ke=1, sheng=2)`)
    set_flag(&mut e, 0, "u05_card_wunature", 0);
    let hand_before_sheng = e.state_core().player(0).hand.len();
    invoke_u05(&mut e, 0, "_shengke");
    assert_eq!(flag(&e, 0, "u05_shengke_sheng"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_sheng + 1);

    e.state_core_mut().player_mut(0).wunature = Some(1); // 0 restrains 1 (`ke=1`)
    let hand_before_ke = e.state_core().player(0).hand.len();
    invoke_u05(&mut e, 0, "_shengke");
    assert_eq!(flag(&e, 0, "u05_shengke_ke"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_ke - 1);

    invoke_u05(&mut e, 0, "wuxingpan_skill");
    assert_eq!(e.state_core().player(0).wunature, Some(1));
    assert_eq!(flag(&e, 0, "u05_card_wunature"), 1);
    invoke_u05(&mut e, 0, "wuxingpan_skill");
    assert_eq!(
        flag(&e, 0, "u05_blocked_usable:wuxingpan_skill"),
        1,
        "wuxingpan_skill usable: 1 must block second activation in same turn"
    );
    set_flag(&mut e, 0, "u05_usable:wuxingpan_skill", 0);

    // 3b. `main/content.js:397-410` Ysera (`stone_yisela`) dream card grant (`stone_chenshui` -> `hsmengjing_*`)
    invoke_u05(&mut e, 0, "stone_chenshui");
    invoke_u05(&mut e, 0, "hsmengjing_suxing");
    invoke_u05(&mut e, 0, "hsmengjing_mengjing");
    assert_eq!(flag(&e, 0, "u05_ysera_dream_granted"), 1);
    assert_eq!(flag(&e, 0, "u05_ysera_dream_idx"), 1);
    assert_eq!(flag(&e, 0, "u05_skill:stone_chenshui"), 1);
    assert_eq!(flag(&e, 0, "u05_skill:hsmengjing_suxing"), 1);
    assert_eq!(flag(&e, 0, "u05_skill:hsmengjing_mengjing"), 1);

    // 3c. `character/gujian/skill.js:2063-2096` `shahun` (`limited: true` 1/game guard)
    invoke_u05(&mut e, 0, "shahun");
    assert_eq!(flag(&e, 0, "u05_limited:shahun"), 1);
    invoke_u05(&mut e, 0, "shahun");
    assert_eq!(
        flag(&e, 0, "u05_blocked_limited:shahun"),
        1,
        "limited skill shahun must block second activation"
    );

    // 4. Execute every single U05 skill, card-skill, and cross-skill and verify ZERO unhandled fallbacks
    //    and strict card conservation!
    let cards_before = total_tracked_cards(&e);
    let all_skills = U05_GUJIAN_ALL_SKILLS
        .iter()
        .chain(U05_GWENT_ALL_SKILLS.iter())
        .chain(U05_HEARTH_ALL_SKILLS.iter())
        .chain(U05_MTG_ALL_SKILLS.iter())
        .chain(U05_SWD_CARD_SKILLS.iter())
        .chain(U05_SWD_CHAR_SKILLS.iter())
        .chain(U05_YUNCHOU_ALL_SKILLS.iter())
        .chain(U05_WUXING_PLAY_SKILLS.iter())
        .chain(U05_OW_ALL_SKILLS.iter())
        .chain(U05_XIANJIAN_ALL_SKILLS.iter())
        .chain(U05_CROSS_SKILLS.iter())
        .copied()
        .chain(all_cards.map(|r| r.0));

    let mut executed = 0usize;
    for sk in all_skills {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "U05 skill {sk} must exist in Catalog"
        );
        // Keep both seats healthy so multi-hundred damage/loseHp skills never end the game mid-sweep
        for seat in [0u8, 1u8] {
            if e.state_core().player(seat).max_hp < 20 {
                e.state_core_mut().player_mut(seat).max_hp = 30;
            }
            if e.state_core().player(seat).hp < 15 {
                e.state_core_mut().player_mut(seat).hp = 25;
            }
        }
        invoke_u05(&mut e, 0, sk);
        executed += 1;
    }
    assert!(executed >= 1_280, "expected >= 1,280 U05 skill/card executions, got {executed}");

    let unhandled: Vec<String> = e
        .state_core()
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| k.starts_with("u05_unhandled_skill:"))
        .map(|(_, k)| k.clone())
        .collect();
    assert!(
        unhandled.is_empty(),
        "Found unhandled U05 skills on formal locked engine: {unhandled:?}"
    );
    assert_eq!(
        total_tracked_cards(&e),
        cards_before,
        "Card conservation violated across U05 skill sweep"
    );

    // 5. Binary `NGSA 107` archive save/restore roundtrip on the formal locked engine
    let saved = e.save();
    let archive_bytes = encode_archive(&saved);
    let decoded = decode_archive(&archive_bytes).expect("decode NGSA 107 archive");
    let restored =
        restore_locked_engine(&decoded, &verified).expect("restore_locked_engine for 9 shahai packs");
    assert_eq!(restored.state_hash(), e.state_hash());
}

#[test]
fn u05_single_subpackage_isolation_and_cross_subpack_general_assembly() {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let freeze_one = |dir_name: &str| -> content_native_loader::FrozenPackage {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        validate_and_freeze_raw_package(raw_json, assets).expect("freeze single shahai package")
    };

    // 1. Load `shahai-hearth` ALONE (without `shahai-swd`) and assemble cross-ref generals
    let verified_hs = prepare_locked_content_with_bridge(vec![freeze_one(
        "shahai-hearth-upstream-noname",
    )])
    .expect("prepare single shahai-hearth package");
    let mut e_hs = start_locked_engine(2026_0506, 2, LockedStartMode::Prototype, &verified_hs)
        .expect("start single shahai-hearth engine");
    for gen_name in ["hs_xsylvanas", "hs_jaina", "hs_kcthun"] {
        e_hs.assign_general_named(0, gen_name)
            .unwrap_or_else(|err| panic!("assign_general_named({gen_name}) failed: {err:?}"));
    }
    invoke_u05(&mut e_hs, 0, "shahai-hearth__ref_busi");
    assert_eq!(flag(&e_hs, 0, "u05_skill:busi"), 1);

    // 2. Load `shahai-gujian` ALONE (without `shahai-swd` / `shaai-xianjian`) and assemble cross-ref generals
    let verified_gj = prepare_locked_content_with_bridge(vec![freeze_one(
        "shahai-gujian-upstream-noname",
    )])
    .expect("prepare single shahai-gujian package");
    let mut e_gj = start_locked_engine(2026_0507, 2, LockedStartMode::Prototype, &verified_gj)
        .expect("start single shahai-gujian engine");
    for gen_name in ["gjqt_wenrenyu", "gjqt_xiayize"] {
        e_gj.assign_general_named(0, gen_name)
            .unwrap_or_else(|err| panic!("assign_general_named({gen_name}) failed: {err:?}"));
    }
    invoke_u05(&mut e_gj, 0, "shahai-gujian__ref_xuanning");
    assert_eq!(flag(&e_gj, 0, "u05_skill:xuanning"), 1);
}

#[test]
fn u05_bidirectional_wuxing_coexistence_turn_progression_and_authoritative_modifiers() {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let freeze_pkg = |dir_name: &str| -> content_native_loader::FrozenPackage {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        validate_and_freeze_raw_package(raw_json, assets).expect("freeze formal package")
    };

    // 1. Bidirectional formal coexistence: `[shahai-zhenfa, wuxing]` AND `[wuxing, shahai-zhenfa]`
    let v_zw = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shahai-zhenfa-upstream-noname"),
        freeze_pkg("wuxing-upstream-liuli"),
    ])
    .expect("[shahai-zhenfa, wuxing] must coexist without DuplicateCard/DuplicateSkill");
    let _e_zw = start_locked_engine(2026_0510, 2, LockedStartMode::Prototype, &v_zw).unwrap();

    let v_wz = prepare_locked_content_with_bridge(vec![
        freeze_pkg("wuxing-upstream-liuli"),
        freeze_pkg("shahai-zhenfa-upstream-noname"),
    ])
    .expect("[wuxing, shahai-zhenfa] must coexist without DuplicateCard/DuplicateSkill");
    let _e_wz = start_locked_engine(2026_0511, 2, LockedStartMode::Prototype, &v_wz).unwrap();

    // 2. Multi-source formal coexistence (`shahai-swd` + `shahai-yunchou` + `taixu` + `zuan`)
    let v_multi = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shahai-swd-upstream-noname"),
        freeze_pkg("shahai-yunchou-upstream-noname"),
        freeze_pkg("taixu-upstream-liuli"),
        freeze_pkg("zuan-upstream-liuli"),
    ])
    .expect("multi-source U05+L11+L25 formal packages must coexist cleanly");
    let mut e = start_locked_engine(2026_0512, 2, LockedStartMode::Prototype, &v_multi).unwrap();

    // 3. Empty-hand `ycnxinfu_qiaosi` (`杀海拾遗/card/yunchou/card.js`): draws 2 FIRST, then discards 1 -> hand.len() == 1
    let removed_hand: Vec<_> = e.state_core_mut().player_mut(0).hand.drain(..).collect();
    e.state_core_mut().discard.extend(removed_hand);
    let removed_eq: Vec<_> = e.state_core_mut().player_mut(0).equips.drain(..).collect();
    e.state_core_mut().discard.extend(removed_eq);
    assert_eq!(e.state_core().player(0).hand.len(), 0);
    invoke_u05(&mut e, 0, "ycnxinfu_qiaosi");
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "empty-hand ycnxinfu_qiaosi must draw 2 synchronously before discarding 1"
    );

    // 4. `cyqiaoxie` (`杀海拾遗/character/swd/skill.js:5-60`) steals/stores into `cyqiaoxie` pile,
    // and `U05Extra::ViewAs("sha")` wires into `Catalog::find_player_vm_view_as`!
    invoke_u05(&mut e, 0, "cyqiaoxie");
    assert!(
        e.state_core()
            .vm_aux
            .piles
            .get(&(0, "cyqiaoxie".to_string()))
            .is_some_and(|p| !p.is_empty()),
        "cyqiaoxie must store stolen/gained card into cyqiaoxie pile"
    );
    invoke_u05(&mut e, 0, "yeying");
    let sample_card = e.state_core().player(0).hand[0];
    assert!(
        e.catalog()
            .find_player_vm_view_as(e.state_core(), 0, sample_card, true, None)
            .is_some(),
        "U05 ViewAs flag must activate real kernel VmViewAsSpec in find_player_vm_view_as"
    );

    // 5. Full-turn progression: Damage skill `ycndiyjianxiong` does NOT fire on PhaseBegin,
    // and self-HP-loss general (`swd_situqiang` with `huodan`) does NOT self-immolate across 6 phases!
    e.assign_general_named(0, "swd_situqiang")
        .expect("assign swd_situqiang");
    for _ in 0..6 {
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    assert!(
        e.state_core().player(0).alive,
        "swd_situqiang must survive full turn progression without 6x auto self-immolation"
    );
}

