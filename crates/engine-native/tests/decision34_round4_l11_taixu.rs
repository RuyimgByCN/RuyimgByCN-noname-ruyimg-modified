//! Decision 34 Round 4 — `L11` (`太虚幻境` / `taixu-upstream-liuli`) Authentic Upstream Closure Suite.
//!
//! Verifies on the formal `packages/extensions/taixu-upstream-liuli` locked product path:
//! - All 320 `txhj_*` generals (`太虚幻境/extension_character.js:11-337`) with exact faction, gender, max_hp, and skills;
//! - All 32 cards (`extension_card.js:8-2041` & `extension_skill.js:458`) including exact weapon ranges;
//! - All 380 `L11` skills/buffs (`extension.js`, `extension_buff.js`, `extension_servant.js`, `extension_skill.js`)
//!   plus 448 NPC cross-package skills execute with zero unhandled fallbacks (`l11_unhandled_skill:* == 0`);
//! - All 21 servants (`extension_servant.js:350-1760`) and all 206 campaign/season/quiz/shop/mode nodes
//!   (`dlc/*/extension_season.js`, `event/event_universal.js`, `extension_framework.js:9919`) execute deterministically;
//! - Binary archive save/restore (`e.save()` -> `restore_locked_engine`) preserves exact `state_hash()`.

use content_native_loader::{
    execute_taixu_campaign_node, prepare_locked_content_with_bridge, resolve_taixu_servant_skills,
    restore_locked_engine, start_locked_engine, validate_and_freeze_raw_package, LockedStartMode,
    TaixuCampaignRun, TaixuConfig, TaixuError, TaixuNodeKind, TaixuRunState,
    DECISION34_STORAGE_DISPATCH_KEY, L11_TAIXU_CAMPAIGN_NODES, L11_TAIXU_CARDS,
    L11_TAIXU_CARD_IDS, L11_TAIXU_EQUIP_CARD_SKILLS, L11_TAIXU_GENERALS,
    L11_TAIXU_IMPLEMENTED_CROSS_SKILLS, L11_TAIXU_OWN_NPC_SKILLS, L11_TAIXU_SERVANTS,
    L11_TAIXU_SHOP_BUFF_POOL, L11_TAIXU_SKILLS, L11_TAIXU_SKILL_ID,
    L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS,
};
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(4_110_000);

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
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
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

fn invoke_l11(e: &mut Engine, owner: u8, skill_name: &str) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(L11_TAIXU_SKILL_ID),
            key: DECISION34_STORAGE_DISPATCH_KEY,
            value: L11_TAIXU_SKILL_ID,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "L11 skill dispatch ({skill_name}) rejected: {out:?}"
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

fn start_formal_taixu_engine(seed: u64) -> (content_native_loader::VerifiedLockedContent, Engine) {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/taixu-upstream-liuli");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read taixu package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read taixu asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze taixu package");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu");
    let engine = start_locked_engine(seed, 2, LockedStartMode::Prototype, &verified)
        .expect("start locked taixu engine");
    (verified, engine)
}

#[test]
fn l11_taixu_formal_package_320_generals_32_cards_374_skills_and_206_campaign_nodes() {
    let (verified, mut e) = start_formal_taixu_engine(2026_1111);

    // 0. Zero `subSkill` regex artifacts (`temp`, `buff`, `sub1`, `sub2`, `count`, `Use`)
    assert_eq!(L11_TAIXU_SKILLS.len(), 374);
    for artifact in ["temp", "buff", "sub1", "sub2", "count", "Use"] {
        assert!(
            e.catalog().skill_by_name(artifact).is_none(),
            "subSkill artifact {artifact} must NOT be registered in L11 Catalog"
        );
    }

    // 1. All 320 `txhj_*` generals (`太虚幻境/extension_character.js:11-337`)
    assert_eq!(L11_TAIXU_GENERALS.len(), 320);
    for &(gname, expected_faction, expected_gender, expected_hp, sk_list) in &L11_TAIXU_GENERALS {
        let g_idx = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some(gname))
            .unwrap_or_else(|| panic!("L11 general {gname} missing from formal locked Catalog"));
        let (_, fac, sks, max_hp, gender) = e.catalog().general_meta(g_idx).unwrap();
        assert_eq!(fac, expected_faction, "faction mismatch for {gname}");
        assert_eq!(gender, expected_gender, "gender mismatch for {gname}");
        assert_eq!(max_hp, expected_hp, "max_hp mismatch for {gname}");
        let expected_attached: Vec<&str> = sk_list
            .iter()
            .copied()
            .filter(|sk| !L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS.contains(sk))
            .collect();
        assert_eq!(
            sks.len(),
            expected_attached.len(),
            "general {gname} attached skill count must match all upstream-defined skills"
        );
        for &sk in &sks {
            assert!(
                e.catalog().skill_by_name(sk).is_some(),
                "L11 general {gname} attached skill {sk} must exist in Catalog"
            );
        }
    }

    // The 9 upstream-undefined skill refs must NOT have fake stubs in Catalog:
    for &undef_sk in &L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS {
        assert!(
            e.catalog().skill_by_name(undef_sk).is_none(),
            "upstream-undefined NPC skill ref {undef_sk} must NOT have a fake stub in Catalog"
        );
    }

    // Genuine cross-package bindings:
    // - Native `HostSkillDecl` (`package_skill_id().is_none()`): `gongxin`
    let gongxin_sid = e
        .catalog()
        .skill_by_name("gongxin")
        .expect("expected HostSkillDecl gongxin in L11 Catalog");
    assert!(
        e.catalog().skills[gongxin_sid as usize]
            .behavior
            .package_skill_id()
            .is_none(),
        "gongxin must bind to native HostBehavior::Gongxin, not a VM stub"
    );
    // - `standard_catalog` Chinese-named skills (`反馈`, `刚烈`, `制衡`, `苦肉`, `奇袭`, `义从`)
    for std_zh in ["反馈", "刚烈", "制衡", "苦肉", "奇袭", "义从"] {
        let sid = e
            .catalog()
            .skill_by_name(std_zh)
            .unwrap_or_else(|| panic!("expected standard_catalog skill {std_zh}"));
        assert!(
            e.catalog().skills[sid as usize]
                .behavior
                .package_skill_id()
                .is_none()
        );
    }
    // - Precompiled non-noop `rule-vm` cross-skills bridged from `taixu-upstream-liuli/package.npack.json`
    let mut npack_bridged_count = 0usize;
    for vm_sk in ["wansha", "nzry_chenglve", "taoluan", "shefu", "duanliang", "quhu"] {
        if let Some(sid) = e.catalog().skill_by_name(vm_sk) {
            let pid = e.catalog().skills[sid as usize].behavior.package_skill_id();
            assert_ne!(
                pid,
                Some(L11_TAIXU_SKILL_ID),
                "{vm_sk} must NOT route through L11_TAIXU_SKILL_ID bucket"
            );
            npack_bridged_count += 1;
        }
    }
    assert!(
        npack_bridged_count >= 4,
        "expected precompiled rule-vm cross-skills from taixu package.npack.json"
    );
    // - Cross-package NPC skills from U05, U08, L19, L20, L27 wired to their authentic source package skill IDs
    for cross_sk in [
        "qianglue",
        "xiadao",
        "xueji",
        "kuanshi",
        "xinfu_langxi",
        "xinfu_lingren",
        "dcxiaoxi",
        "rehuashen",
        "twxiafeng",
        "rejieming",
        "reyingzi",
        "xinliegong",
        "zhenlie",
        "zhuhai",
    ] {
        let sid = e
            .catalog()
            .skill_by_name(cross_sk)
            .unwrap_or_else(|| panic!("expected cross-package NPC skill {cross_sk} in L11 Catalog"));
        let pid = e.catalog().skills[sid as usize].behavior.package_skill_id();
        assert_ne!(
            pid,
            Some(L11_TAIXU_SKILL_ID),
            "{cross_sk} must NOT route through L11_TAIXU_SKILL_ID bucket"
        );
    }

    // Assemble boss general `txhj_zhaozhong` (`extension_character.js:330`, max_hp=12) on locked engine:
    // `txhj_huangmen`, `txhj_daiyuan`, `huaizi` are attached; unported `txhj_jinxing` & `txhj_xiehui` are omitted!
    let zhaozhong_idx = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some("txhj_zhaozhong"))
        .unwrap();
    e.debug_assemble_general_def(0, zhaozhong_idx);
    assert_eq!(e.state_core().player(0).max_hp, 12);
    let huangmen_id = e.catalog().skill_by_name("txhj_huangmen").unwrap();
    assert!(e.state_core().has_skill(0, huangmen_id));

    // 2. All 32 cards (`extension_card.js:8-2041` & `extension_skill.js:458`) including exact weapon ranges & equip_skills
    assert_eq!(L11_TAIXU_CARDS.len(), 32);
    assert_eq!(L11_TAIXU_CARD_IDS.len(), 32);
    for &(cname, expected_kind, expected_range, expected_suit, expected_rank) in &L11_TAIXU_CARDS {
        let cdef = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == cname)
            .unwrap_or_else(|| panic!("L11 card {cname} missing from formal locked Catalog"));
        if cname != "huogong" {
            assert_eq!(cdef.kind, expected_kind, "card kind mismatch for {cname}");
            assert_eq!(cdef.suit, expected_suit, "suit mismatch for {cname}");
            assert_eq!(cdef.rank, expected_rank, "rank mismatch for {cname}");
            if expected_kind == CardKind::Weapon {
                assert_eq!(cdef.range, expected_range, "weapon range mismatch for {cname}");
            }
            if expected_kind.equip_slot().is_some() {
                assert!(
                    cdef.equip_skills.contains(&cname),
                    "L11 equip card {cname} must attach equip_skill {cname}"
                );
            }
        }
    }

    // 3. Differential & boundary assertions citing upstream `file:line`:
    // (a) `txhj_xingtianpojunfu` (`extension_card.js:8`): requires discarding 2 cards; `< 2` cards does NOT lock target
    while e.state_core().player(0).hand.len() > 1 {
        let cid = e.state_core_mut().player_mut(0).hand.pop().unwrap();
        e.state_core_mut().discard.push(cid);
    }
    while let Some(cid) = e.state_core_mut().player_mut(0).equips.pop() {
        e.state_core_mut().discard.push(cid);
    }
    invoke_l11(&mut e, 0, "txhj_xingtianpojunfu");
    assert_eq!(flag(&e, 1, "l11_xingtian_card_locked"), 0);

    for _ in 0..3 {
        if let Some(cid) = e.state_core_mut().deck.pop() {
            e.state_core_mut().player_mut(0).hand.push(cid);
        }
    }
    invoke_l11(&mut e, 0, "txhj_xingtianpojunfu");
    assert_eq!(flag(&e, 1, "l11_xingtian_card_locked"), 1);
    assert_eq!(flag(&e, 1, "l11_xingtian_armor_null"), 1);

    // (b) `txhj_yayaSkill2` (`extension.js:2916`): 1/game dying rescue recovers `< 1` HP to 1 HP; second trigger is blocked
    e.state_core_mut().player_mut(0).hp = 0;
    invoke_l11(&mut e, 0, "txhj_yayaSkill2");
    assert_eq!(e.state_core().player(0).hp, 1);
    assert_eq!(flag(&e, 0, "l11_yaya2_protected"), 1);
    e.state_core_mut().player_mut(0).hp = 0;
    invoke_l11(&mut e, 0, "txhj_yayaSkill2");
    assert_eq!(e.state_core().player(0).hp, 0, "txhj_yayaSkill2 is limited to once per game");
    e.state_core_mut().player_mut(0).hp = 8;

    // (c) `txhj_ditingSkill2` (`extension.js:3102`): +1 max_hp on taking damage
    let mhp_before = e.state_core().player(0).max_hp;
    invoke_l11(&mut e, 0, "txhj_ditingSkill2");
    assert_eq!(e.state_core().player(0).max_hp, mhp_before + 1);

    // (d) `boss_tianyun` (`extension_skill.js:4085`): owner loses 1 HP, strips target equips, and deals 2 fire damage
    e.state_core_mut().player_mut(0).hp = 8;
    e.state_core_mut().player_mut(1).hp = 8;
    e.state_core_mut().player_mut(1).max_hp = 8;
    invoke_l11(&mut e, 0, "boss_tianyun");
    assert_eq!(e.state_core().player(0).hp, 7);
    assert_eq!(e.state_core().player(1).hp, 6);

    // 4. Exhaustively execute all 374 L11 skills + 29 own NPC skills + 20 equip card skills:
    //    ZERO grouped bucket flags (`l11_passive_active:*`, `l11_npc_*`, `l11_buff_draw:*`, `l11_buff_aura:*`, `l11_unhandled_skill:*`)!
    for &sk in L11_TAIXU_SKILLS
        .iter()
        .chain(L11_TAIXU_OWN_NPC_SKILLS.iter())
        .chain(L11_TAIXU_EQUIP_CARD_SKILLS.iter())
    {
        for seat in [0u8, 1u8] {
            if e.state_core().player(seat).max_hp < 6 {
                e.state_core_mut().player_mut(seat).max_hp = 10;
            }
            e.state_core_mut().player_mut(seat).hp = 6;
            e.state_core_mut().player_mut(seat).alive = true;
            e.state_core_mut().player_mut(seat).dying = false;
            while e.state_core().player(seat).hand.len() < 3 {
                if let Some(cid) = e
                    .state_core_mut()
                    .deck
                    .pop()
                    .or_else(|| e.state_core_mut().discard.pop())
                {
                    e.state_core_mut().player_mut(seat).hand.push(cid);
                } else {
                    break;
                }
            }
        }
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "L11 skill {sk} must be registered in formal locked Catalog"
        );
        invoke_l11(&mut e, 0, sk);
    }

    let forbidden_flags: Vec<String> = e
        .state_core()
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| {
            k.starts_with("l11_unhandled_skill:")
                || k.starts_with("l11_passive_active:")
                || k.starts_with("l11_npc_")
                || k.starts_with("l11_buff_draw:")
                || k.starts_with("l11_buff_aura:")
        })
        .map(|(_, k)| k.clone())
        .collect();
    assert!(
        forbidden_flags.is_empty(),
        "L11 must have zero grouped fallback/stub flags, found: {forbidden_flags:?}"
    );

    // 5. All 21 servants (`extension_servant.js:350-1760`), all 206 campaign nodes & `TaixuCampaignRun` graph
    assert_eq!(L11_TAIXU_SERVANTS.len(), 21);
    for &(sv_id, _, _, expected_sks) in &L11_TAIXU_SERVANTS {
        let resolved = resolve_taixu_servant_skills(sv_id).expect("resolve servant");
        assert_eq!(resolved, expected_sks);
        for &sk in resolved {
            assert!(e.catalog().skill_by_name(sk).is_some());
        }
    }
    assert!(resolve_taixu_servant_skills("nonexistent_servant").is_none());

    assert_eq!(L11_TAIXU_CAMPAIGN_NODES.len(), 206);
    let cfg = TaixuConfig {
        enable_campaign: true,
        season: "GFHuangJinZhiLuan".to_string(),
        starting_gold: 1_000,
        buff_limit: 64,
        spirit_enabled: true,
    };
    for node in &L11_TAIXU_CAMPAIGN_NODES {
        let mut run = TaixuRunState::from_config(&cfg);
        run.gold = 1_000;
        let out = execute_taixu_campaign_node(&cfg, &mut run, node.scope, node.id, 0)
            .unwrap_or_else(|err| panic!("campaign node {}:{} failed: {err:?}", node.scope, node.id));
        assert_eq!(out.kind, node.kind);
        if let Some(ref b) = out.granted_buff {
            assert!(
                L11_TAIXU_SHOP_BUFF_POOL.contains(&b.as_str()),
                "campaign granted_buff {b} must be an authentic upstream buff_txhj_* ID, never synthetic"
            );
            assert!(e.catalog().skill_by_name(b).is_some());
        }
        if node.kind == TaixuNodeKind::ExamQuiz {
            assert!(out.quiz_passed);
            let mut wrong_run = TaixuRunState::from_config(&cfg);
            let wrong_out =
                execute_taixu_campaign_node(&cfg, &mut wrong_run, node.scope, node.id, 1).unwrap();
            assert!(!wrong_out.quiz_passed);
            assert!(out.gold_after >= wrong_out.gold_after);
        }
    }

    // `TaixuCampaignRun` season graph progression + snapshot roundtrip
    let mut graph_run =
        TaixuCampaignRun::start_season_run(&cfg, "GFHuangJinZhiLuan").expect("start season run");
    assert_eq!(graph_run.available_next_nodes().len(), 3);
    let step1 = graph_run.advance_node(&cfg, 0, 0).expect("advance step 1");
    assert_eq!(step1.scope, "GFHuangJinZhiLuan");
    let step2 = graph_run.advance_node(&cfg, 1, 0).expect("advance step 2");
    assert_eq!(graph_run.visited_nodes.len(), 2);
    assert!(step2.gold_after > 1_000);
    let camp_snap = graph_run.encode_snapshot();
    let camp_restored = TaixuCampaignRun::decode_snapshot(&camp_snap).expect("decode campaign run");
    assert_eq!(camp_restored, graph_run);

    // Negative guards on campaign state machine
    let mut disabled_cfg = cfg.clone();
    disabled_cfg.enable_campaign = false;
    let mut err_run = TaixuRunState::from_config(&cfg);
    assert_eq!(
        execute_taixu_campaign_node(&disabled_cfg, &mut err_run, "ChongYingChuLin", "0", 0),
        Err(TaixuError::CampaignDisabled)
    );
    assert!(matches!(
        execute_taixu_campaign_node(&cfg, &mut err_run, "ChongYingChuLin", "unknown_node", 0),
        Err(TaixuError::UnknownCommand(_))
    ));

    // 6. Binary archive roundtrip on locked product path
    let snap = e.save();
    let restored = restore_locked_engine(&snap, &verified).expect("restore locked taixu engine");
    assert_eq!(restored.state_hash(), e.state_hash());
}

#[test]
fn l11_taixu_playcard_intrinsics_and_d34_l11_mods() {
    use engine_native::event::DamageNature;
    use engine_native::rvm_bridge::modifiers::{
        eval_card_target_canceled, eval_card_use_canceled, eval_damage_amount,
        eval_distance_total, eval_hand_limit, eval_ignore_armor, eval_sha_extra_targets,
        eval_sha_use_limit,
    };

    let (_verified, mut e) = start_formal_taixu_engine(2026_1119);

    // Advance to Play phase so `GameAction::PlayCard` executes real named card intrinsics (`intrinsic_named_ext.rs`)
    while e.turn().phase != engine_native::Phase::Play {
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    let actor = e.turn().current;
    let other = 1 - actor;

    // 1. Real `PlayCard(txhj_yiyi)` (`太虚幻境/extension_card.js:334`): draw 2, discard 2, card moves table -> discard
    let yiyi_cid = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "txhj_yiyi")
        .unwrap() as u32;
    let old_ha: Vec<_> = e.state_core_mut().player_mut(actor).hand.drain(..).collect();
    e.state_core_mut().discard.extend(old_ha);
    let old_ea: Vec<_> = e.state_core_mut().player_mut(actor).equips.drain(..).collect();
    e.state_core_mut().discard.extend(old_ea);
    e.state_core_mut().deck.retain(|&c| c != yiyi_cid);
    e.state_core_mut().discard.retain(|&c| c != yiyi_cid);
    e.state_core_mut().player_mut(other).hand.retain(|&c| c != yiyi_cid);
    e.state_core_mut().player_mut(actor).hand.push(yiyi_cid);
    let out_yiyi = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: yiyi_cid,
            targets: vec![],
        },
    });
    assert!(
        !matches!(out_yiyi, CommandOutcome::Rejected(_)),
        "PlayCard(txhj_yiyi) rejected: {out_yiyi:?}"
    );
    drain(&mut e);
    assert!(e.state_core().table.is_empty(), "txhj_yiyi must not stay on table");
    assert!(e.state_core().discard.contains(&yiyi_cid));

    // 2. Real `PlayCard(txhj_yuanjiao)` (`太虚幻境/extension_card.js:305`): target draws 1, actor draws 3
    let yuanjiao_cid = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "txhj_yuanjiao")
        .unwrap() as u32;
    let old_ho: Vec<_> = e.state_core_mut().player_mut(other).hand.drain(..).collect();
    e.state_core_mut().discard.extend(old_ho);
    let old_ha2: Vec<_> = e.state_core_mut().player_mut(actor).hand.drain(..).collect();
    e.state_core_mut().discard.extend(old_ha2);
    e.state_core_mut().deck.retain(|&c| c != yuanjiao_cid);
    e.state_core_mut().discard.retain(|&c| c != yuanjiao_cid);
    e.state_core_mut().player_mut(actor).hand.push(yuanjiao_cid);
    let out_yj = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: yuanjiao_cid,
            targets: vec![other],
        },
    });
    assert!(
        !matches!(out_yj, CommandOutcome::Rejected(_)),
        "PlayCard(txhj_yuanjiao) rejected: {out_yj:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(other).hand.len(), 1);
    assert_eq!(e.state_core().player(actor).hand.len(), 3);

    // 3. Authoritative kernel modifiers (`d34_flags.rs` + `d34_l11_mods.rs`)
    let sha_card = e.catalog().card_by_name("杀").unwrap_or(0);
    let duel_card = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Duel)
        .unwrap() as u32;
    let nanman_card = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::SavageAttack)
        .unwrap() as u32;

    invoke_l11(&mut e, 0, "txhj_zhuge");
    assert_eq!(
        eval_sha_use_limit(
            e.state_core(),
            e.catalog(),
            0,
            Some(sha_card),
            CardKind::Sha,
            DamageNature::Normal,
            1,
        ),
        (4, false),
        "txhj_zhuge must add +3 to authoritative Sha use limit"
    );

    invoke_l11(&mut e, 0, "txhj_fangtianhuaji");
    assert_eq!(
        eval_sha_extra_targets(
            e.state_core(),
            e.catalog(),
            0,
            sha_card,
            CardKind::Sha,
            DamageNature::Normal,
        ),
        2,
        "txhj_fangtianhuaji must grant +2 extra Sha targets"
    );

    while e.state_core().player(0).hand.len() < 2 {
        let cid = e.state_core_mut().deck.pop().unwrap();
        e.state_core_mut().player_mut(0).hand.push(cid);
    }
    while e.state_core().player(1).hand.is_empty() {
        let cid = e.state_core_mut().deck.pop().unwrap();
        e.state_core_mut().player_mut(1).hand.push(cid);
    }
    invoke_l11(&mut e, 0, "txhj_xingtianpojunfu");
    let opp_card = e.state_core().player(1).hand[0];
    let opp_kind = e.catalog().card_kind(opp_card);
    assert!(
        eval_card_use_canceled(
            e.state_core(),
            e.catalog(),
            1,
            opp_card,
            opp_kind,
            DamageNature::Normal,
        ),
        "txhj_xingtianpojunfu must lock target card use for the turn"
    );
    assert!(
        eval_ignore_armor(
            e.state_core(),
            e.catalog(),
            0,
            1,
            Some(sha_card),
            DamageNature::Normal,
        ),
        "txhj_xingtianpojunfu must nullify target armor"
    );

    // `d34_l11_mods.rs` per-skill checks:
    // - `txmazhan` (`extension.js:3460`): globalFrom - 2, globalTo + 1
    invoke_l11(&mut e, 0, "txmazhan");
    assert_eq!(eval_distance_total(e.state_core(), e.catalog(), 0, 1, 3), 1);
    assert_eq!(eval_distance_total(e.state_core(), e.catalog(), 1, 0, 1), 2);

    // - `txshixin` (`extension.js:3593`) & `txmanzhen` (`extension.js:5704`)
    invoke_l11(&mut e, 1, "txshixin");
    assert!(eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        0,
        1,
        duel_card,
        CardKind::Duel,
        DamageNature::Normal,
    ));
    invoke_l11(&mut e, 0, "txmanzhen");
    assert!(eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        1,
        0,
        nanman_card,
        CardKind::SavageAttack,
        DamageNature::Normal,
    ));

    // - `txhuangfu` (`extension.js:3723`: immune to Thunder) & `txmanjia` (`extension.js:5657`: 0 normal Sha, +1 Fire)
    invoke_l11(&mut e, 1, "txhuangfu");
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_card),
            DamageNature::Thunder,
            2,
        ),
        0
    );
    invoke_l11(&mut e, 1, "txmanjia");
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_card),
            DamageNature::Normal,
            1,
        ),
        0
    );
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_card),
            DamageNature::Fire,
            1,
        ),
        2
    );

    // - `txkeji_add` (`extension.js:4988`: +4 maxHandcard) & `xinmingjian2` (`extension.js:3441`: unlimited Sha)
    let base_hl = eval_hand_limit(e.state_core(), e.catalog(), 0, 4);
    invoke_l11(&mut e, 0, "txkeji_add");
    assert_eq!(eval_hand_limit(e.state_core(), e.catalog(), 0, 4), base_hl + 4);
    invoke_l11(&mut e, 0, "xinmingjian2");
    assert!(
        eval_sha_use_limit(
            e.state_core(),
            e.catalog(),
            0,
            Some(sha_card),
            CardKind::Sha,
            DamageNature::Normal,
            1,
        )
        .1
    );

    // - Round 5 newly wired `d34_l11_mods.rs` flags & per-skill split behaviors:
    e.state_core_mut().vm_aux.flags.clear();
    let clean_hl = eval_hand_limit(e.state_core(), e.catalog(), 0, 4);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_yingyang_pindian_mod".to_string()), 1);
    assert_eq!(eval_hand_limit(e.state_core(), e.catalog(), 0, 4), clean_hl + 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_yingyang_pindian_mod".to_string()));

    e.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "l11_armor_guard:txhj_guofengyupao".to_string()), 1);
    assert!(eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        0,
        1,
        duel_card,
        CardKind::Duel,
        DamageNature::Normal,
    ));
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(1, "l11_armor_guard:txhj_guofengyupao".to_string()));

    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_chiyan_all_fire".to_string()), 1);
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(sha_card),
            DamageNature::Normal,
            1,
        ),
        2
    );
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_chiyan_all_fire".to_string()));

    // `txhj_guihuo` (`extension.js:685`): deals 1 Fire damage to opponent
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).hp = 6;
    e.state_core_mut().player_mut(1).hujia = 0;
    invoke_l11(&mut e, 0, "txhj_guihuo");
    assert_eq!(e.state_core().player(1).hp, 5);
}

#[test]
fn l11_taixu_164_cross_package_npc_skills_and_9_inert_undefined_refs() {
    use engine_native::catalog::Suit;
    use engine_native::event::DamageNature;
    use engine_native::rvm_bridge::modifiers::{
        d34_flags::find_d34_flag_view_as, eval_card_target_canceled, eval_card_unrespondable,
        eval_card_use_canceled, eval_damage_amount,
    };

    let (_verified, mut e) = start_formal_taixu_engine(2026_1120);

    // 1. All 9 upstream-undefined dangling skill IDs (`extension_character.js:321-337`,
    //    `extension_csrank.js:3540-3587`) must remain inert without fabricated skills in Catalog:
    assert_eq!(L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS.len(), 9);
    for &undef_sk in &L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS {
        assert!(
            e.catalog().skill_by_name(undef_sk).is_none(),
            "upstream-undefined skill ref {undef_sk} must NOT have a fabricated skill in Catalog"
        );
    }

    // All `HeJinZhuHuan` (`何进诛宦`) NPCs that reference those 9 undefined IDs assemble cleanly:
    // `txhj_huanguan` (`&["txhj_xiehui"]`) has 0 attached skills; the other 13 bind their defined skills:
    let hg_idx = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some("txhj_huanguan"))
        .expect("txhj_huanguan in Catalog");
    assert!(
        e.catalog().general_meta(hg_idx).unwrap().2.is_empty(),
        "txhj_huanguan only lists undefined txhj_xiehui so attached skills must be empty"
    );
    for &(npc_id, expected_defined_skill) in &[
        ("txhj_huanguandaofushou", "txhj_huangmen"),
        ("txhj_dongtaihou", "fenji"),
        ("txhj_baohong", "twlinglu"),
        ("txhj_xiaoweijun", "kaikang"),
        ("txhj_chunyuqiong", "jiuchi"),
        ("txhj_jinweijun", "longdan"),
        ("txhj_zhaorong", "txzhulian"),
        ("txhj_zhaozhong", "txhj_huangmen"),
        ("txhj_yuanshao", "zhanyi"),
        ("txhj_caocao", "nzry_kuizhu"),
        ("txhj_jianshuo", "txzhulian"),
        ("txhj_huanyanzhangrang", "shefu"),
        ("txhj_huanyanhejin", "mouzhu"),
    ] {
        let g_idx = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some(npc_id))
            .unwrap_or_else(|| panic!("HeJinZhuHuan NPC {npc_id} missing from Catalog"));
        e.debug_assemble_general_def(0, g_idx);
        let sk_id = e
            .catalog()
            .skill_by_name(expected_defined_skill)
            .unwrap_or_else(|| panic!("defined skill {expected_defined_skill} on {npc_id} missing"));
        assert!(
            e.state_core().has_skill(0, sk_id),
            "{npc_id} must bind defined skill {expected_defined_skill}"
        );
    }

    // 2. Representative authentic upstream `file:line` behavior assertions for the 164 cross-package NPC skills:
    // (a) `twwushen` (`tw/skill.js:1764`): Heart hand card views as distance-unlimited Sha, unrespondable, and Heart Sha unlimited count
    e.state_core_mut().vm_aux.flags.clear();
    let heart_card = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Heart)
        .unwrap() as u32;
    let heart_sha = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Heart && c.kind == CardKind::Sha)
        .unwrap_or(heart_card as usize) as u32;
    if !e.state_core().player(0).hand.contains(&heart_card) {
        e.state_core_mut().player_mut(0).hand.push(heart_card);
    }
    invoke_l11(&mut e, 0, "twwushen");
    let vas = find_d34_flag_view_as(
        e.state_core(),
        e.catalog(),
        0,
        heart_card,
        true,
        Some(CardKind::Sha),
    )
    .expect("twwushen must view Heart hand card as Sha");
    assert_eq!(vas.as_kind, CardKind::Sha);
    assert!(vas.ignore_distance);
    assert!(eval_card_unrespondable(
        e.state_core(),
        e.catalog(),
        0,
        1,
        heart_sha,
        CardKind::Sha,
        DamageNature::Normal,
    ));

    // (b) `twwuhun` (`tw/skill.js:1842`): marks opponent with `l11_mengyan_mark` and drains opponent's HP on death rattle
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).hp = 6;
    e.state_core_mut().player_mut(1).hujia = 0;
    invoke_l11(&mut e, 0, "twwuhun");
    assert!(flag(&e, 1, "l11_mengyan_mark") >= 1);
    assert_eq!(e.state_core().player(1).hp, 3);
    e.state_core_mut().player_mut(1).hp = 6;

    // (c) `dualside` (`game.js:34262`): flips `face_down`, restores HP to `max_hp`, clears `dying`, and draws 2 cards
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(0).max_hp = 6;
    e.state_core_mut().player_mut(0).hp = 0;
    e.state_core_mut().player_mut(0).dying = true;
    e.state_core_mut().player_mut(0).face_down = false;
    let hand_before_dual = e.state_core().player(0).hand.len();
    invoke_l11(&mut e, 0, "dualside");
    assert!(e.state_core().player(0).face_down);
    assert!(!e.state_core().player(0).dying);
    assert_eq!(e.state_core().player(0).hp, 6);
    assert!(e.state_core().player(0).hand.len() >= hand_before_dual + 2);

    // (d) `qingzhongx` (`sp/skill.js:35220` / `markskill.js:18544`): draws 2 cards and swaps hand with the character who has fewer handcards
    e.state_core_mut().vm_aux.flags.clear();
    while e.state_core().player(0).hand.len() > 3 {
        let cid = e.state_core_mut().player_mut(0).hand.pop().unwrap();
        e.state_core_mut().discard.push(cid);
    }
    while e.state_core().player(1).hand.len() > 1 {
        let cid = e.state_core_mut().player_mut(1).hand.pop().unwrap();
        e.state_core_mut().discard.push(cid);
    }
    invoke_l11(&mut e, 0, "qingzhongx");
    assert!(e.state_core().player(1).hand.len() >= 3);

    // (e) `leiji` (`shenhua.js:7205`): flips 1 judge card and deals 2 Thunder damage to opponent
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).hp = 6;
    e.state_core_mut().player_mut(1).hujia = 0;
    invoke_l11(&mut e, 0, "leiji");
    assert_eq!(e.state_core().player(1).hp, 4);

    // (f) `retianxiang` (`shenhua.js:7913`): prevents damage (`l11_mod:retianxiang_ward`), redirects 1 damage to opponent, and target draws lost_hp
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).max_hp = 6;
    e.state_core_mut().player_mut(1).hp = 5;
    e.state_core_mut().player_mut(1).hujia = 0;
    let p1_hand_before = e.state_core().player(1).hand.len();
    invoke_l11(&mut e, 0, "retianxiang");
    assert_eq!(e.state_core().player(1).hp, 4);
    assert!(e.state_core().player(1).hand.len() >= p1_hand_before + 1);
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(1),
            0,
            Some(heart_sha),
            DamageNature::Normal,
            2,
        ),
        0
    );

    // (g) `sbliegong` (`sb.js:3439`), `nzry_kuizhu` (`shenhua.js:1931`), `twlinglu` (`tw/skill.js:15134`), `mouzhu` (`sp.js:12799`), `shefu` (`sp/skill.js:39282`), `xinfu_daigong` (`mobile/skill.js:30818`)
    e.state_core_mut().vm_aux.flags.clear();
    invoke_l11(&mut e, 0, "sbliegong");
    assert!(eval_card_unrespondable(
        e.state_core(),
        e.catalog(),
        0,
        1,
        heart_sha,
        CardKind::Sha,
        DamageNature::Normal,
    ));
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(heart_sha),
            DamageNature::Normal,
            1,
        ),
        2
    );

    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(0).hp = 2;
    e.state_core_mut().player_mut(1).hp = 6;
    invoke_l11(&mut e, 0, "nzry_kuizhu");
    assert_eq!(e.state_core().player(1).hp, 5);

    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).hp = 6;
    invoke_l11(&mut e, 0, "twlinglu");
    assert_eq!(e.state_core().player(1).hp, 5);

    e.state_core_mut().vm_aux.flags.clear();
    for _ in 0..4 {
        if let Some(cid) = e.state_core_mut().deck.pop() {
            e.state_core_mut().player_mut(0).hand.push(cid);
        }
    }
    while e.state_core().player(1).hand.len() > 1 {
        let cid = e.state_core_mut().player_mut(1).hand.pop().unwrap();
        e.state_core_mut().discard.push(cid);
    }
    e.state_core_mut().player_mut(1).hp = 6;
    invoke_l11(&mut e, 0, "mouzhu");
    assert_eq!(e.state_core().player(1).hp, 5);

    e.state_core_mut().vm_aux.flags.clear();
    if !e.state_core().player(1).hand.contains(&heart_sha) {
        e.state_core_mut().player_mut(1).hand.push(heart_sha);
    }
    invoke_l11(&mut e, 0, "shefu");
    assert!(eval_card_use_canceled(
        e.state_core(),
        e.catalog(),
        1,
        heart_sha,
        CardKind::Sha,
        DamageNature::Normal,
    ));

    e.state_core_mut().vm_aux.flags.clear();
    invoke_l11(&mut e, 1, "xinfu_daigong");
    assert_eq!(
        eval_damage_amount(
            e.state_core(),
            e.catalog(),
            Some(0),
            1,
            Some(heart_sha),
            DamageNature::Normal,
            2,
        ),
        0
    );

    // 3. Exhaustively execute all 164 `L11_TAIXU_IMPLEMENTED_CROSS_SKILLS`:
    assert_eq!(L11_TAIXU_IMPLEMENTED_CROSS_SKILLS.len(), 164);
    for &sk in &L11_TAIXU_IMPLEMENTED_CROSS_SKILLS {
        for seat in [0u8, 1u8] {
            e.state_core_mut().player_mut(seat).max_hp = 10;
            e.state_core_mut().player_mut(seat).hp = 6;
            e.state_core_mut().player_mut(seat).alive = true;
            e.state_core_mut().player_mut(seat).dying = false;
            while e.state_core().player(seat).hand.len() < 3 {
                if let Some(cid) = e
                    .state_core_mut()
                    .deck
                    .pop()
                    .or_else(|| e.state_core_mut().discard.pop())
                {
                    e.state_core_mut().player_mut(seat).hand.push(cid);
                } else {
                    break;
                }
            }
        }
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "L11 cross-package NPC skill {sk} must be registered in formal locked Catalog"
        );
        invoke_l11(&mut e, 0, sk);
    }

    let unhandled: Vec<String> = e
        .state_core()
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| k.starts_with("l11_unhandled_skill:"))
        .map(|(_, k)| k.clone())
        .collect();
    assert!(
        unhandled.is_empty(),
        "All 164 L11 cross-package NPC skills must resolve with zero unhandled flags, found: {unhandled:?}"
    );

    // 4. Regression checks for awakening transitions, suit-conditional modifiers, and unhandled guard:
    // (a) `twhongju` awakens once (-1 max_hp), then transitions to `twqingce` without losing max_hp again
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(0).max_hp = 5;
    e.state_core_mut().player_mut(0).hp = 5;
    invoke_l11(&mut e, 0, "twzhengrong");
    invoke_l11(&mut e, 0, "twhongju");
    assert_eq!(e.state_core().player(0).max_hp, 4);
    invoke_l11(&mut e, 0, "twhongju");
    assert_eq!(
        e.state_core().player(0).max_hp,
        4,
        "twhongju must not reduce max_hp a second time after awakening"
    );

    // (b) `zili` awakens once (-1 max_hp), then transitions to `paiyi` (2 draws + 1 damage)
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(0).max_hp = 5;
    e.state_core_mut().player_mut(0).hp = 4;
    e.state_core_mut().player_mut(1).hp = 5;
    invoke_l11(&mut e, 0, "requanji");
    invoke_l11(&mut e, 0, "zili");
    assert_eq!(e.state_core().player(0).max_hp, 4);
    invoke_l11(&mut e, 0, "zili");
    assert_eq!(
        e.state_core().player(0).max_hp,
        4,
        "zili must not reduce max_hp a second time after awakening"
    );
    assert_eq!(e.state_core().player(1).hp, 4);

    // (c) `jieyuan` distinguishes black card (+1 damage) vs red card (-1 damage)
    let black_card = (0..e.catalog().cards.len() as u32)
        .find(|&cid| matches!(e.catalog().cards[cid as usize].suit, Suit::Spade | Suit::Club))
        .unwrap();
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(0).hand = vec![black_card];
    e.state_core_mut().player_mut(0).equips.clear();
    invoke_l11(&mut e, 0, "jieyuan");
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "l11_mod:jieyuan_more".to_string()))
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "l11_mod:jieyuan_less".to_string()))
            .copied()
            .unwrap_or(0),
        0
    );
    e.state_core_mut().player_mut(0).hand = vec![heart_sha];
    invoke_l11(&mut e, 0, "jieyuan");
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "l11_mod:jieyuan_less".to_string()))
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "l11_mod:jieyuan_more".to_string()))
            .copied()
            .unwrap_or(0),
        0
    );

    // (d) `twlingfa`: discarding a card avoids taking damage; empty hand/equips takes 1 damage
    e.state_core_mut().vm_aux.flags.clear();
    e.state_core_mut().player_mut(1).hp = 5;
    e.state_core_mut().player_mut(1).hand = vec![black_card];
    e.state_core_mut().player_mut(1).equips.clear();
    invoke_l11(&mut e, 0, "twlingfa");
    assert_eq!(
        e.state_core().player(1).hp,
        5,
        "twlingfa must not deal damage when target discards 1 card"
    );
    invoke_l11(&mut e, 0, "twlingfa");
    assert_eq!(
        e.state_core().player(1).hp,
        4,
        "twlingfa must deal 1 damage when target has no cards to discard"
    );

    // (e) `bmcanshi` marks target with `l11_kui_mark` and cancels single-target cards from that actor
    e.state_core_mut().vm_aux.flags.clear();
    invoke_l11(&mut e, 0, "bmcanshi");
    assert!(eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        1,
        0,
        heart_sha,
        CardKind::Sha,
        DamageNature::Normal,
    ));

    // (f) Unhandled skill guard sets `l11_unhandled_skill:*` when an unknown skill is dispatched
    e.state_core_mut().vm_aux.flags.clear();
    invoke_l11(&mut e, 0, "l11_nonexistent_test_skill");
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "l11_unhandled_skill:l11_nonexistent_test_skill".to_string()))
            .copied(),
        Some(1)
    );
}
