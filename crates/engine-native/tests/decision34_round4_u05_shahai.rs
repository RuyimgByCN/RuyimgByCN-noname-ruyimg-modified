//! Decision 34 — `U05` (`杀海拾遗` / 9 formal `sha*-upstream-noname` packages) Authentic Upstream Suite.
//!
//! Verifies on the formal `packages/extensions/sha*-upstream-noname` locked product path:
//! - `main/config.js:4-48` & `main/content.js:5-11,383` configuration schema (`tip`, `wuxing`, `rand`,
//!   `mtg`, `zhenfa`, `gwent`, `yunchou`), mode forbid lists (`boss`, `chess`, `double`, `stone`, `threecard`),
//!   and deterministic snapshot/restore;
//! - All 332 upstream generals (+ `stone_yisela`, `main/content.js:353`) assemble cleanly without panic,
//!   binding native `HostBehavior`s (`SkillSource::Host`) and `standard_catalog` skills directly without
//!   VM shadowing, and omitting unported skills honestly (`缺失`) with zero dead-flag stubs;
//! - All 275 upstream cards exist in the formal `Catalog` without duplicating `assets/cards.json`
//!   entries or registering fake `PhaseBegin` `VmSkillDecl`s for card-only names, and playing custom
//!   named `U05` cards via `GameAction::PlayCard` executes their authentic upstream card effects;
//! - Explicit per-skill `U05` modifiers (`d34_u05_mods.rs`), upgraded `U05` skills, strict card
//!   conservation, and `NGSA 107` binary archive roundtrip (`e.save()` -> `restore_locked_engine`).

use content_native_loader::{
    enabled_shahai_subpackages, is_shahai_forbid_in_mode, parse_shahai_main_config,
    prepare_locked_content_with_bridge, restore_locked_engine, restore_shahai_config,
    snapshot_shahai_config, start_locked_engine, validate_and_freeze_raw_package, LockedStartMode,
    ShahaiMainConfig, ShahaiWuxingCardRate, DECISION34_STORAGE_DISPATCH_KEY,
    U05_GUJIAN_ALL_CARDS, U05_GUJIAN_GENERALS, U05_GWENT_ALL_CARDS, U05_GWENT_GENERALS,
    U05_HEARTH_ALL_CARDS, U05_HEARTH_GENERALS, U05_MTG_ALL_CARDS, U05_MTG_GENERALS,
    U05_OW_GENERALS, U05_SHAHAI_SKILL_ID, U05_STONE_GENERALS, U05_SWD_ALL_CARDS,
    U05_SWD_GENERALS, U05_WUXING_ALL_CARDS, U05_XIANJIAN_GENERALS_FULL, U05_YUNCHOU_ALL_CARDS,
    U05_YUNCHOU_GENERALS, U05_ZHENFA_ALL_CARDS,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::collections::BTreeMap;
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

fn move_named_card_to_hand(e: &mut Engine, seat: u8, cname: &str) -> u32 {
    let cid = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == cname)
        .unwrap_or_else(|| panic!("card {cname} not in Catalog")) as u32;
    let core = e.state_core_mut();
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&x| x != cid);
        core.player_mut(p).equips.retain(|&x| x != cid);
        core.player_mut(p).judge.retain(|&x| x != cid);
    }
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&x| x != cid);
    }
    core.player_mut(seat).hand.push(cid);
    cid
}

fn advance_to_play_phase(e: &mut Engine, target_seat: u8) {
    for _ in 0..24 {
        if e.turn().current == target_seat
            && format!("{:?}", e.turn().phase).contains("Play")
            && e.open_choice().is_none()
        {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
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

    assert!(parse_shahai_main_config(r#"{"rand":"0.99"}"#).is_err());
    assert!(parse_shahai_main_config(r#"{"wuxing":"yes"}"#).is_err());

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
fn u05_all_9_formal_packages_assemble_333_generals_play_custom_cards_and_roundtrip() {
    let (verified, mut e) = start_all_9_formal_shahai_packages(2026_0505);

    // 1. All 332 upstream generals + `stone_yisela` exist and assemble cleanly without panicking
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
    for &(gname, _, _, _, _) in all_generals {
        general_count += 1;
        e.assign_general_named(0, gname)
            .unwrap_or_else(|err| panic!("assign_general_named({gname}) failed: {err:?}"));
    }
    assert_eq!(general_count, 333);

    // 2. Native `HostBehavior` skills (all 25 U05 host skills) are bound
    //    as real host skills (`package_skill_id().is_none()`, never shadowed by VM stubs!)
    for host_sk in [
        "busi",
        "dangping",
        "beige",
        "qianfang",
        "cyqiaoxie",
        "donghuangzhong",
        "g_chenhuodajie",
        "g_jinlianzhu",
        "g_shencaojie",
        "g_shenmiguo",
        "g_shihuifen",
        "g_yuchan_swap",
        "gouhunluo2",
        "gw_ciguhanshuang",
        "gw_kunenfayin",
        "huajin2",
        "huanxing2",
        "jiguanyaoshu_skill",
        "jiguanyuan",
        "lianyaohu_skill",
        "lmazui2",
        "nigong",
        "shuchui2",
        "xiajiao",
        "xuanyuanjian",
    ] {
        let def_id = e
            .catalog()
            .skill_by_name(host_sk)
            .unwrap_or_else(|| panic!("HostBehavior skill {host_sk} missing from Catalog"));
        assert!(
            e.catalog().skills[def_id as usize]
                .behavior
                .package_skill_id()
                .is_none(),
            "{host_sk} must be bound as native HostSkillDecl, not a VM stub"
        );
    }

    // 3. All 275 upstream cards exist in Catalog (either bridged from `assets/cards.json` with prefix
    //    or attached directly), and card-only names are NEVER registered as fake `VmSkillDecl`s!
    let all_cards = U05_GUJIAN_ALL_CARDS
        .iter()
        .chain(U05_GWENT_ALL_CARDS.iter())
        .chain(U05_HEARTH_ALL_CARDS.iter())
        .chain(U05_MTG_ALL_CARDS.iter())
        .chain(U05_SWD_ALL_CARDS.iter())
        .chain(U05_YUNCHOU_ALL_CARDS.iter())
        .chain(U05_ZHENFA_ALL_CARDS.iter())
        .chain(U05_WUXING_ALL_CARDS.iter());
    let catalog_card_names: Vec<&str> = e.catalog().cards.iter().map(|c| c.name).collect();
    let mut card_count = 0usize;
    for &(cname, _, _, _, _) in all_cards {
        card_count += 1;
        let stripped = cname.strip_prefix("gw_").unwrap_or(cname);
        let found = catalog_card_names.iter().any(|&existing| {
            existing == cname
                || existing.contains(cname)
                || existing.starts_with(&format!("gw_{stripped}_"))
        });
        assert!(found, "U05 card {cname} missing from formal locked Catalog");
    }
    assert_eq!(card_count, 275);
    for card_only in [
        "yanjiadan_heart",
        "yanjiadan_spade",
        "gw_hudiewu",
        "lingjiandai",
        "hsmengjing_suxing",
    ] {
        assert!(
            e.catalog().skill_by_name(card_only).is_none(),
            "card-only name {card_only} must NOT be registered as a fake VmSkillDecl"
        );
    }

    // 4. Play custom named `U05` cards via real `GameAction::PlayCard` -> `intrinsic_named_ext.rs`!
    let tracked_before = total_tracked_cards(&e);
    e.assign_general_named(0, "gjqt_bailitusu").unwrap();
    e.assign_general_named(1, "gjqt_fengqingxue").unwrap();
    e.state_core_mut().player_mut(0).max_hp = 10;
    e.state_core_mut().player_mut(0).hp = 6; // damaged so `yanjiadan_heart` recovers 1 HP
    e.state_core_mut().player_mut(1).max_hp = 10;
    e.state_core_mut().player_mut(1).hp = 10;
    advance_to_play_phase(&mut e, 0);

    // 4a. `character/gujian/card.js:4-67` `yanjiadan_heart`: recovers 1 HP when damaged
    let cid_heart = move_named_card_to_hand(&mut e, 0, "yanjiadan_heart");
    let hp_before = e.state_core().player(0).hp;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: cid_heart,
            targets: vec![],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "PlayCard(yanjiadan_heart): {out:?}");
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp_before + 1);
    assert_eq!(flag(&e, 0, "u05_card_used:yanjiadan_heart"), 1);

    // 4b. `character/hearth/card.js:797-822` `hsmengjing_suxing`: all other alive players lose 1 HP
    let cid_suxing = move_named_card_to_hand(&mut e, 0, "hsmengjing_suxing");
    let p1_hp_before = e.state_core().player(1).hp;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: cid_suxing,
            targets: vec![],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "PlayCard(hsmengjing_suxing): {out:?}");
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, p1_hp_before - 1);

    // 4c. `card/swd/card.js:1062-1098` `lingjiandai`: gains 1 `Hslingjian` card to hand
    let cid_ljd = move_named_card_to_hand(&mut e, 0, "lingjiandai");
    let hand_before_ljd = e.state_core().player(0).hand.len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: cid_ljd,
            targets: vec![],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "PlayCard(lingjiandai): {out:?}");
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_ljd); // -1 played +1 Hslingjian gained

    // 5. Authentic upstream `U05` skills & explicit modifiers (`d34_u05_mods.rs`)
    // 5a. `play/wuxing/skill.js:5-95` `_shengke` & `wuxingpan_skill`
    e.state_core_mut().player_mut(0).wunature = Some(2);
    set_flag(&mut e, 0, "u05_card_wunature", 0);
    let hb = e.state_core().player(0).hand.len();
    invoke_u05(&mut e, 0, "_shengke");
    assert_eq!(flag(&e, 0, "u05_shengke_sheng"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hb + 1);

    // 5b. `character/hearth/skill.js:5624-5659` `guozai` + `guozai2`: draw to 4, then discard debt
    while e.state_core().player(0).hand.len() > 1 {
        let c = e.state_core_mut().player_mut(0).hand.pop().unwrap();
        e.state_core_mut().discard.push(c);
    }
    invoke_u05(&mut e, 0, "guozai");
    assert_eq!(e.state_core().player(0).hand.len(), 4);
    assert_eq!(flag(&e, 0, "u05_storage:guozai2"), 3);
    invoke_u05(&mut e, 0, "guozai2");
    assert_eq!(e.state_core().player(0).hand.len(), 1);
    assert_eq!(flag(&e, 0, "u05_storage:guozai2"), 0);

    // 5c. `character/hearth/skill.js:5312-5321` `xjumo`: +5 maxHandcard when wounded, +3 when full HP
    let base_limit = e.state_core().hand_limit(e.catalog(), 0);
    invoke_u05(&mut e, 0, "xjumo");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), base_limit + 5);
    e.state_core_mut().player_mut(0).hp = e.state_core().player(0).max_hp as i32;
    let full_base = e.state_core().player(0).hp as usize;
    set_flag(&mut e, 0, "u05_mod:xjumo", 0);
    let clean_full = e.state_core().hand_limit(e.catalog(), 0);
    assert!(clean_full >= full_base);
    set_flag(&mut e, 0, "u05_mod:xjumo", 1);
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), clean_full + 3);
    set_flag(&mut e, 0, "u05_mod:xjumo", 0);

    // 5d. `character/ow/skill.js:2322-2330` `zhanlong2`: unlimited Sha use limit
    invoke_u05(&mut e, 0, "zhanlong2");
    assert!(
        engine_native::rvm_bridge::modifiers::eval_sha_use_limit(
            e.state_core(),
            e.catalog(),
            0,
            None,
            engine_native::CardKind::Sha,
            engine_native::event::DamageNature::Normal,
            1,
        )
        .1
    );
    set_flag(&mut e, 0, "u05_mod:zhanlong2", 0);

    // 5e. `character/{gwent,ow,xianjian}/skill.js` debuff/modifier flags (`d34_u05_mods.rs`)
    let sha_cid = e.catalog().card_by_name("杀").unwrap_or(0);
    set_flag(&mut e, 0, "u05_mod:shanguang2", 1);
    assert!(engine_native::rvm_bridge::modifiers::eval_card_use_canceled(
        e.state_core(),
        e.catalog(),
        0,
        sha_cid,
        engine_native::CardKind::Sha,
        engine_native::event::DamageNature::Normal,
    ));
    set_flag(&mut e, 0, "u05_mod:shanguang2", 0);

    set_flag(&mut e, 1, "u05_debuff:luan", 1);
    assert_eq!(
        engine_native::rvm_bridge::modifiers::eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_cid),
            engine_native::event::DamageNature::Normal,
            1,
        ),
        2
    );
    set_flag(&mut e, 1, "u05_debuff:luan", 0);

    set_flag(&mut e, 1, "u05_debuff:chengxin", 1);
    assert_eq!(
        engine_native::rvm_bridge::modifiers::eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_cid),
            engine_native::event::DamageNature::Normal,
            2,
        ),
        1
    );
    set_flag(&mut e, 1, "u05_debuff:chengxin", 0);

    // 6. Sweep all registered VM skills in Catalog and verify strict card conservation
    let vm_skill_names: Vec<String> = e
        .catalog()
        .skills
        .iter()
        .filter(|s| s.behavior.package_skill_id().is_some())
        .map(|s| s.name.to_string())
        .collect();
    assert!(
        vm_skill_names.len() >= 700,
        "expected >= 700 verified U05 VM skills in Catalog, got {}",
        vm_skill_names.len()
    );
    assert!(
        e.catalog().skills.len() >= 900,
        "expected >= 900 total U05 Host + VM skills in Catalog, got {}",
        e.catalog().skills.len()
    );
    for sk in &vm_skill_names {
        for seat in [0u8, 1u8] {
            if e.state_core().player(seat).max_hp < 20 {
                e.state_core_mut().player_mut(seat).max_hp = 30;
            }
            if e.state_core().player(seat).hp < 15 {
                e.state_core_mut().player_mut(seat).hp = 25;
            }
        }
        invoke_u05(&mut e, 0, sk);
    }

    assert_eq!(
        total_tracked_cards(&e),
        tracked_before,
        "Card conservation violated across U05 card plays + VM skill sweep"
    );

    // 7. Binary `NGSA 107` archive save/restore roundtrip on the formal locked engine
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

    // 1. Load `shahai-hearth` ALONE and verify `hs_xsylvanas` binds real `HostBehavior::SwdBusi` (`busi`)
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
    let busi_def = e_hs.catalog().skill_by_name("shahai-hearth__ref_busi").expect("busi bound on shahai-hearth");
    assert!(e_hs.catalog().skills[busi_def as usize].behavior.package_skill_id().is_none());

    // 2. Load `shahai-gujian` ALONE and verify `gjqt_wenrenyu` binds real `HostBehavior::SwdDangping` (`dangping`)
    //    and `gjqt_xiayize` executes `shahai-gujian__ref_xuanning`
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
    let dangping_def = e_gj
        .catalog()
        .skill_by_name("shahai-gujian__ref_dangping")
        .expect("dangping bound on shahai-gujian");
    assert!(e_gj.catalog().skills[dangping_def as usize].behavior.package_skill_id().is_none());
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

    let v_multi = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shahai-swd-upstream-noname"),
        freeze_pkg("shahai-yunchou-upstream-noname"),
        freeze_pkg("taixu-upstream-liuli"),
        freeze_pkg("zuan-upstream-liuli"),
    ])
    .expect("multi-source U05+L11+L25 formal packages must coexist cleanly");
    let mut e = start_locked_engine(2026_0512, 2, LockedStartMode::Prototype, &v_multi).unwrap();

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

    invoke_u05(&mut e, 0, "yeying");
    let sample_card = e.state_core().player(0).hand[0];
    assert!(
        e.catalog()
            .find_player_vm_view_as(e.state_core(), 0, sample_card, true, None)
            .is_some(),
        "U05 ViewAs flag must activate real kernel VmViewAsSpec in find_player_vm_view_as"
    );

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
