//! Decision 34 Round 4 — `L25` (`祖安武将` / `zuan-upstream-liuli`) Authentic Upstream Closure Suite.
//!
//! Verifies on the formal `packages/extensions/zuan-upstream-liuli` locked product path:
//! - All 60 `ssr_*` generals (`祖安武将/extension.js:338-397`) with authentic faction, gender, max_hp, and skills;
//! - All 130 `ssr_*` general skills (`extension.js:4911-33889`), 84 global/mode/rule skills (`extension.js:18984-49741`),
//!   and 8 card/view-as entries (`extension.js:6535-28385`) execute with zero unhandled fallbacks;
//! - Save/restore (`save_archive` -> `restore_locked_engine`) preserves exact `L25` state.

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, DECISION34_STORAGE_DISPATCH_KEY,
    L25_ALL_RULE_SKILLS, L25_SSR_GENERALS, L25_SSR_GENERAL_SKILLS, L25_ZUAN_CARD_ENTRIES,
    L25_ZUAN_SKILL_ID,
};
use engine_native::catalog::{Faction, Suit};
use engine_native::content::standard::Gender;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(4_250_000);

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

fn invoke_l25(e: &mut Engine, owner: u8, skill_name: &str) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(L25_ZUAN_SKILL_ID),
            key: DECISION34_STORAGE_DISPATCH_KEY,
            value: L25_ZUAN_SKILL_ID,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "L25 skill dispatch ({skill_name}) rejected: {out:?}"
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
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn start_formal_zuan_engine(seed: u64) -> (content_native_loader::VerifiedLockedContent, Engine) {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/zuan-upstream-liuli");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read zuan package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read zuan asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze zuan package");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked zuan");
    let engine = start_locked_engine(seed, 2, LockedStartMode::Prototype, &verified)
        .expect("start locked zuan engine");
    (verified, engine)
}

#[test]
fn l25_zuan_formal_package_generals_cards_skills_and_archive_roundtrip() {
    let (verified, mut e) = start_formal_zuan_engine(2026_1110);

    // 1. Upstream generals (`祖安武将/extension.js:338-397`): verify all 60 `ssr_*` generals and key metadata
    assert_eq!(L25_SSR_GENERALS.len(), 60);
    for &(gname, expected_faction, expected_gender, expected_hp, sk_list) in &L25_SSR_GENERALS {
        let g_idx = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("missing formal L25 general {gname}"));
        let (_, faction, g_skills, max_hp, gender) = e.catalog().general_meta(g_idx).unwrap();
        assert_eq!(faction, expected_faction, "faction mismatch on {gname}");
        assert_eq!(gender, expected_gender, "gender mismatch on {gname}");
        assert_eq!(max_hp, expected_hp, "max_hp mismatch on {gname}");
        assert_eq!(g_skills.as_slice(), sk_list, "skill list mismatch on {gname}");
        for &sk in sk_list {
            assert!(
                e.catalog().skill_by_name(sk).is_some(),
                "general {gname} skill {sk} must be registered in Catalog"
            );
        }
    }
    // Spot-check upstream lines 338, 358, 383:
    // ssr_luxun (Wu, Male, 3), ssr_diaochan (Qun, Female, 3), ssr_dongzhuo (Qun, Male, 8)
    let luxun = L25_SSR_GENERALS.iter().find(|g| g.0 == "ssr_luxun").unwrap();
    assert_eq!((luxun.1, luxun.2, luxun.3), (Faction::Wu, Gender::Male, 3));
    let diaochan = L25_SSR_GENERALS.iter().find(|g| g.0 == "ssr_diaochan").unwrap();
    assert_eq!((diaochan.1, diaochan.2, diaochan.3), (Faction::Qun, Gender::Female, 3));
    let dongzhuo = L25_SSR_GENERALS.iter().find(|g| g.0 == "ssr_dongzhuo").unwrap();
    assert_eq!((dongzhuo.1, dongzhuo.2, dongzhuo.3), (Faction::Qun, Gender::Male, 8));

    // 2. Upstream cards (`extension.js:6535-28385`): all 8 G0 card entries + `ssr_yongsi_yuxi_card` exist
    for &cname in L25_ZUAN_CARD_ENTRIES.iter().chain(["ssr_yongsi_yuxi_card"].iter()) {
        assert!(
            e.catalog().cards.iter().any(|c| c.name == cname),
            "formal L25 Catalog must contain card {cname}"
        );
    }

    // 3. Upstream skill semantics:
    // 3a. `ssr_qianxun` (`extension.js:4911-4930`): discards down to 1 hand card when >1
    assert!(e.state_core().player(0).hand.len() > 1);
    invoke_l25(&mut e, 0, "ssr_qianxun");
    assert_eq!(e.state_core().player(0).hand.len(), 1);
    assert!(flag(&e, 0, "l25_ssr_qianxun_discarded") >= 1);

    // 3b. `ssr_jinfan` (`extension.js:6535-6575`): discards 1 card and stores 2 real cards in `jinfan_ling` pile
    invoke_l25(&mut e, 0, "ssr_jinfan");
    assert_eq!(flag(&e, 0, "l25_jinfan_ling"), 2);
    assert_eq!(
        e.state_core()
            .vm_aux
            .piles
            .get(&(0, "jinfan_ling".to_string()))
            .map(|p| p.len()),
        Some(2)
    );

    // 3c. `ssr_shenfen` (`extension.js:31078-31130`): `<6` kuangbao marks blocked, `>=6` awakens
    set_flag(&mut e, 0, "l25_kuangbao_marks", 4);
    invoke_l25(&mut e, 0, "ssr_shenfen");
    assert_eq!(flag(&e, 0, "l25_shenfen_blocked"), 1);
    assert_eq!(flag(&e, 0, "l25_shenfen_awakened"), 0);
    set_flag(&mut e, 0, "l25_kuangbao_marks", 6);
    invoke_l25(&mut e, 0, "ssr_shenfen");
    assert_eq!(flag(&e, 0, "l25_shenfen_awakened"), 1);
    assert_eq!(flag(&e, 0, "l25_kuangbao_marks"), 0);

    // 3d. `wumou` (`extension.js:48975-49005`): spends 1 baonu mark when >0, else loses 1 HP
    let hp_before = e.state_core().player(0).hp;
    set_flag(&mut e, 0, "l25_baonu_marks", 1);
    invoke_l25(&mut e, 0, "wumou");
    assert_eq!(flag(&e, 0, "l25_baonu_marks"), 0);
    assert_eq!(e.state_core().player(0).hp, hp_before);
    invoke_l25(&mut e, 0, "wumou");
    assert_eq!(e.state_core().player(0).hp, hp_before - 1);

    // 3e. `new_guixin` (`extension.js:49077-49105`): steals 1 card from other alive seat & toggles face_down
    let face_down_before = e.state_core().player(0).face_down;
    invoke_l25(&mut e, 0, "new_guixin");
    assert_eq!(e.state_core().player(0).face_down, !face_down_before);
    assert_eq!(flag(&e, 0, "l25_new_guixin_gained"), 1);

    // 3f. `gongxin` (`extension.js:49506-49565`): confiscates a Heart card from target hand
    let old_p1_hand = std::mem::take(&mut e.state_core_mut().player_mut(1).hand);
    e.state_core_mut().discard.extend(old_p1_hand);
    let heart_cid = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&cid| e.catalog().cards[cid as usize].suit == Suit::Heart)
        .expect("find heart card in deck");
    e.state_core_mut().deck.retain(|&c| c != heart_cid);
    e.state_core_mut().player_mut(1).hand.push(heart_cid);
    invoke_l25(&mut e, 0, "gongxin");
    assert_eq!(flag(&e, 0, "l25_gongxin_obtained_heart"), 1);
    assert!(e.state_core().player(0).hand.contains(&heart_cid));

    // 3g. `g_nzry_jieying` (`extension.js:49639-49664`): linked + non-elemental draws 2
    e.state_core_mut().player_mut(0).linked = true;
    let hand_before_jieying = e.state_core().player(0).hand.len();
    invoke_l25(&mut e, 0, "g_nzry_jieying");
    assert_eq!(flag(&e, 0, "l25_jieying_drew"), 2);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_jieying + 2);

    // 3h. `_killskill` (10 XP) + `_expskill` (4 XP) + `_useskill` (2 XP) -> level up (`extension.js:43454-43738`)
    invoke_l25(&mut e, 0, "_killskill"); // +10 exp -> level 1!
    assert_eq!(flag(&e, 0, "l25_level"), 1);
    invoke_l25(&mut e, 0, "_expskill"); // +4 exp
    invoke_l25(&mut e, 0, "_expskill"); // +4 exp (8)
    assert_eq!(flag(&e, 0, "l25_level"), 1);
    invoke_l25(&mut e, 0, "_useskill"); // +2 exp (10) -> level 2!
    assert_eq!(flag(&e, 0, "l25_level"), 2);

    // 4. Exhaustive sweep across all 130 SSR skills, 84 rule skills, and 9 card skills: zero unhandled
    let (_, mut sweep_e) = start_formal_zuan_engine(2026_1111);
    for &sk in L25_SSR_GENERAL_SKILLS
        .iter()
        .chain(L25_ALL_RULE_SKILLS.iter())
        .chain(L25_ZUAN_CARD_ENTRIES.iter())
        .chain(["ssr_yongsi_yuxi_card"].iter())
    {
        assert!(
            sweep_e.catalog().skill_by_name(sk).is_some(),
            "L25 skill {sk} must exist in formal locked Catalog"
        );
        // Keep player 0 & 1 alive and healthy during sweep
        sweep_e.state_core_mut().player_mut(0).hp = 4;
        sweep_e.state_core_mut().player_mut(1).hp = 4;
        invoke_l25(&mut sweep_e, 0, sk);
    }
    let unhandled: Vec<String> = sweep_e
        .state_core()
        .vm_aux
        .flags
        .keys()
        .filter(|(_, k)| k.starts_with("l25_unhandled_skill:"))
        .map(|(_, k)| k.clone())
        .collect();
    assert!(
        unhandled.is_empty(),
        "zero L25 skills may hit unhandled fallback, got: {unhandled:?}"
    );

    // 5. Archive save/restore round-trip on formal locked content
    let archive_bytes = sweep_e.save();
    let restored =
        restore_locked_engine(&archive_bytes, &verified).expect("restore locked L25 engine");
    assert_eq!(restored.state_hash(), sweep_e.state_hash());
}

#[test]
fn l25_zuan_ssr_skills_authoritative_modifiers_viewas_and_turn_safety() {
    use engine_native::catalog::CardKind;
    use engine_native::event::DamageNature;
    use engine_native::rvm_bridge::modifiers::{
        eval_card_unrespondable, eval_damage_amount, eval_distance_total, eval_sha_use_limit,
    };

    let (_verified, mut e) = start_formal_zuan_engine(2026_1125);
    let sha_card = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Sha)
        .unwrap() as u32;
    let shan_card = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Shan)
        .unwrap() as u32;

    // 1. `ssr_diaochan` (`ssr_biyue` — `祖安武将/extension.js:11950`):
    // Passive `mod.globalTo` adds +1 distance per empty zone (h, e, j), and active trigger draws cards!
    e.assign_general_named(1, "ssr_diaochan")
        .expect("assign ssr_diaochan");
    let drained_h1: Vec<_> = e.state_core_mut().player_mut(1).hand.drain(..).collect();
    e.state_core_mut().discard.extend(drained_h1);
    let drained_e1: Vec<_> = e.state_core_mut().player_mut(1).equips.drain(..).collect();
    e.state_core_mut().discard.extend(drained_e1);
    assert_eq!(
        eval_distance_total(e.state_core(), e.catalog(), 0, 1, 1),
        4,
        "ssr_biyue with 3 empty zones (h/e/j) must add +3 distance to seat 1"
    );
    invoke_l25(&mut e, 1, "ssr_biyue");
    assert!(
        !e.state_core().player(1).hand.is_empty(),
        "ssr_biyue must draw cards instead of being a dead flag"
    );

    // 2. `ssr_zhangfei` (`ssr_paoxiao` — `extension.js:15111`): unlimited Sha uses
    e.assign_general_named(0, "ssr_zhangfei")
        .expect("assign ssr_zhangfei");
    let (_, unlimited) = eval_sha_use_limit(
        e.state_core(),
        e.catalog(),
        0,
        Some(sha_card),
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    assert!(unlimited, "ssr_paoxiao must grant unlimited Sha uses");

    // 3. `ssr_lvbu` (`ssr_wushuang` — `extension.js:8998`): Sha unrespondable
    e.assign_general_named(0, "ssr_lvbu")
        .expect("assign ssr_lvbu");
    assert!(
        eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            sha_card,
            CardKind::Sha,
            DamageNature::Normal,
        ),
        "ssr_wushuang must enforce unrespondable Sha in authoritative kernel"
    );

    // 4. `ssr_zhaoyun` (`ssr_newlongdan` — `extension.js:5586`): Shan <-> Sha view-as
    e.assign_general_named(0, "ssr_zhaoyun")
        .expect("assign ssr_zhaoyun");
    if !e.state_core().player(0).hand.contains(&shan_card) {
        e.state_core_mut().deck.retain(|&c| c != shan_card);
        e.state_core_mut().discard.retain(|&c| c != shan_card);
        e.state_core_mut().player_mut(1).hand.retain(|&c| c != shan_card);
        e.state_core_mut().player_mut(0).hand.push(shan_card);
    }
    assert!(
        e.catalog()
            .find_player_vm_view_as(e.state_core(), 0, shan_card, true, Some(CardKind::Sha))
            .is_some(),
        "ssr_newlongdan must allow viewing Shan as Sha"
    );

    // 5. `ssr_menghuo` (`ssr_manjia` — `extension.js:22026`): Vine Armor passive (0 normal Sha, +1 Fire damage)
    e.assign_general_named(1, "ssr_menghuo")
        .expect("assign ssr_menghuo");
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
        0,
        "ssr_manjia must prevent normal Sha damage"
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
        2,
        "ssr_manjia must add +1 to incoming Fire damage"
    );

    // 6. Full-turn progression safety: `ssr_guojia` (`ssr_bailun`, `max_hp = 3`) does NOT self-immolate across 6 phases!
    e.assign_general_named(0, "ssr_guojia")
        .expect("assign ssr_guojia");
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
        "ssr_guojia must survive full turn progression without 6x ssr_bailun self-immolation"
    );
    assert_eq!(
        flag(&e, 0, "l25_skill:_dieskill"),
        0,
        "Death skill _dieskill must NOT fire on PhaseBegin"
    );
}

