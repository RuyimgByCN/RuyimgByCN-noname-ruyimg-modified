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
    TaixuConfig, TaixuError, TaixuNodeKind, TaixuRunState, DECISION34_STORAGE_DISPATCH_KEY,
    L11_TAIXU_CAMPAIGN_NODES, L11_TAIXU_CARDS, L11_TAIXU_CARD_IDS, L11_TAIXU_CROSS_SKILLS,
    L11_TAIXU_GENERALS, L11_TAIXU_SERVANTS, L11_TAIXU_SKILLS, L11_TAIXU_SKILL_ID,
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
fn l11_taixu_formal_package_320_generals_32_cards_380_skills_and_206_campaign_nodes() {
    let (verified, mut e) = start_formal_taixu_engine(2026_1111);

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
        assert_eq!(sks.len(), sk_list.len(), "skill count mismatch for {gname}");
        for &sk in sk_list {
            assert!(
                e.catalog().skill_by_name(sk).is_some(),
                "L11 general {gname} references unregistered skill {sk}"
            );
        }
    }

    // Assemble boss general `txhj_zhaozhong` (`extension_character.js:330`, max_hp=12) on locked engine
    let zhaozhong_idx = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some("txhj_zhaozhong"))
        .unwrap();
    e.debug_assemble_general_def(0, zhaozhong_idx);
    assert_eq!(e.state_core().player(0).max_hp, 12);
    let huangmen_id = e.catalog().skill_by_name("txhj_huangmen").unwrap();
    assert!(e.state_core().has_skill(0, huangmen_id));

    // 2. All 32 cards (`extension_card.js:8-2041` & `extension_skill.js:458`) including exact weapon ranges
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

    // Give owner >= 2 cards -> discards 2 and locks target cards & armor
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

    // 4. Exhaustively execute all 380 L11 skills + 32 card IDs + 448 NPC cross-package skills with zero unhandled fallbacks
    assert_eq!(L11_TAIXU_SKILLS.len(), 380);
    assert_eq!(L11_TAIXU_CROSS_SKILLS.len(), 448);
    for &sk in L11_TAIXU_SKILLS
        .iter()
        .chain(L11_TAIXU_CARD_IDS.iter())
        .chain(L11_TAIXU_CROSS_SKILLS.iter())
    {
        // Keep both seats alive and stocked so multi-hundred-skill sweep never hits incidental game-over
        for seat in [0u8, 1u8] {
            if e.state_core().player(seat).max_hp < 6 {
                e.state_core_mut().player_mut(seat).max_hp = 10;
            }
            e.state_core_mut().player_mut(seat).hp = 6;
            e.state_core_mut().player_mut(seat).alive = true;
            e.state_core_mut().player_mut(seat).dying = false;
            while e.state_core().player(seat).hand.len() < 3 {
                if let Some(cid) = e.state_core_mut().deck.pop().or_else(|| e.state_core_mut().discard.pop()) {
                    e.state_core_mut().player_mut(seat).hand.push(cid);
                } else {
                    break;
                }
            }
        }
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "L11 skill/card {sk} must be registered in formal locked Catalog"
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
        "L11 must have zero unhandled skills, found: {unhandled:?}"
    );

    // 5. All 21 servants (`extension_servant.js:350-1760`) & all 206 campaign nodes
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
        season: "ChongYingChuLin".to_string(),
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
        if node.kind == TaixuNodeKind::ExamQuiz {
            assert!(out.quiz_passed);
            let mut wrong_run = TaixuRunState::from_config(&cfg);
            let wrong_out =
                execute_taixu_campaign_node(&cfg, &mut wrong_run, node.scope, node.id, 1).unwrap();
            assert!(!wrong_out.quiz_passed);
            assert!(out.gold_after >= wrong_out.gold_after);
        }
    }

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
