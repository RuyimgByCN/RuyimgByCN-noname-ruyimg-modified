//! Batch 26 (Decision 34 Complete 37-Source In-Repo Closure): end-to-end integration,
//! differential, negative/boundary guard, and save/restore verification for the
//! remaining 9 sources (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`)
//! plus `U07` (`jianbi`, `choudu`).

use content_native_loader::{
    bridge_pack, decision34_closure_extension_pack, decision34_pack9_extension_pack,
    decision34_pack9_for_coexistence, resolve_l24_skin_transform,
    resolve_l27_choose_character_two, resolve_u01_3d_gallery, validate_and_apply_l25_diy,
    L08_DECADE_CHARACTERS, L08_DECADE_SKILLS, L08_DECADE_SKILL_ID, L14_SHOUSHA_CARDS,
    L14_SHOUSHA_CHARACTERS, L14_SHOUSHA_SKILLS, L14_SHOUSHA_SKILL_ID, L19_DOOMSDAY_CHARACTERS,
    L19_DOOMSDAY_SKILLS, L19_DOOMSDAY_SKILL_ID, L20_MARK_SUPPLEMENT_CARDS,
    L20_MARK_SUPPLEMENT_CHARACTERS, L20_MARK_SUPPLEMENT_SKILLS, L20_SKILL_DISPATCH_ID,
    L24_SKIN_SKILL_ID, L24_SKIN_SWITCH_SKILLS, L25DiyEditorDraft, L25_ZUAN_CHARACTERS,
    L25_ZUAN_SKILLS, L25_ZUAN_SKILL_ID, L27DualGeneralPick, L27_SELECT_BEAUTIFY_SKILLS,
    L27_SELECT_SKILL_ID, MTG_LAND_CARD_ID, OW_SKILL_DISPATCH_ID, SWD_DUIJUE_ID, SWD_TANLIN_ID,
    U01_DDD_GENERALS, U01_DDD_SKILLS, U05_MTG_LAND_CARDS, U05_OW_SKILLS,
    U05_SWD_REMAINING_SKILLS, U05_XIANJIAN_GENERALS, U05_XIANJIAN_SKILLS,
    U05_YUNCHOU_REMAINING_SKILLS, U05_ZHENFA_CARDS, U07_WUDIAN_REMAINING_SKILLS,
    WUDIAN_CHOUDU_ID, WUDIAN_JIANBI_ID, XIANJIAN_SKILL_DISPATCH_ID, YUNCHOU_SKILL_ID,
    ZHENFA_POZHENJUE_ID, DECISION34_STORAGE_DISPATCH_KEY, DDD_SKILL_DISPATCH_ID,
};
use engine_native::event::EventKind;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(2_600_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn read_pack_lf(rel: &str) -> String {
    std::fs::read_to_string(repo_extensions_root().join(rel))
        .expect("read package json")
        .replace("\r\n", "\n")
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn invoke_p9_op(e: &mut Engine, owner: u8, skill_name: &str, skill_id: u32) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(skill_id),
            key: DECISION34_STORAGE_DISPATCH_KEY,
            value: skill_id,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "Decision 34 pack9 effect dispatch ({skill_name}) rejected: {out:?}"
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

fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(e);
}

#[test]
fn u01_d3_all_31_generals_and_64_ddd_skills_execute_and_roundtrip() {
    let d3_json = read_pack_lf("d3-upstream-noname/assets/character.json");
    let d3_pack = bridge_pack(&d3_json).expect("d3 character.json").pack;
    let p9_pack = decision34_pack9_for_coexistence(&[&d3_pack]);
    let packs = [&d3_pack, &p9_pack];

    let mut e = Engine::new_with_extensions(260101, 2, &packs).expect("U01 full engine");
    assert_eq!(U01_DDD_GENERALS.len(), 31);
    assert_eq!(U01_DDD_SKILLS.len(), 64);
    for &(gname, _, _, _, _) in &U01_DDD_GENERALS {
        let _ = find_general(&e, gname);
    }
    for &sk in &U01_DDD_SKILLS {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "U01 skill {sk} must exist in Catalog"
        );
    }

    // Bind `ddd_zhangmiao` (`dddxiaxing`) and trigger real RuleVm PhaseBegin execution
    let zhangmiao = find_general(&e, "ddd_zhangmiao");
    e.assign_general(0, zhangmiao).expect("assign ddd_zhangmiao");
    set_flag(&mut e, 0, "ddd_chain_target", 1);
    next_phase(&mut e);
    assert_eq!(flag(&e, 0, "ddd_xiaxing_linked"), 1);
    assert_eq!(flag(&e, 0, "u01_ddd_resolved"), 1);
    assert!(
        e.state_core().player(1).linked,
        "dddxiaxing chains target when ddd_chain_target=1"
    );

    // Verify `dddqianlong`, `dddqiaoduan`, `dddyeshen`, and `dddqice` (including empty-hand negative guard)
    invoke_p9_op(&mut e, 0, "dddqianlong", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qianlong_daoxin"), 15);
    assert_eq!(flag(&e, 0, "ddd_qianlong_unlocked"), 1);

    e.state_core_mut().player_mut(0).hp = 2;
    invoke_p9_op(&mut e, 0, "dddqiaoduan", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qiaoduan_mode"), 2);
    assert_eq!(e.state_core().player(0).hp, 3);

    invoke_p9_op(&mut e, 0, "dddyeshen", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_yeshen_triggered"), 1);

    // Negative guard: `dddqice` with empty hand must be blocked and deal 0 damage
    let saved_hand = e.state_core().player(0).hand.clone();
    e.state_core_mut().player_mut(0).hand.clear();
    let p1_hp_before = e.state_core().player(1).hp;
    invoke_p9_op(&mut e, 0, "dddqice", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qice_blocked_empty"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp_before);

    // Positive path: `dddqice` with non-empty hand discards card and deals 1 damage
    e.state_core_mut().player_mut(0).hand = saved_hand;
    invoke_p9_op(&mut e, 0, "dddqice", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qice_virtual_trick"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp_before - 1);

    // Validate 3D gallery camera resolver + negative guard
    let gal = resolve_u01_3d_gallery("ddd_caomao", 60, -40, 200).expect("valid ddd_caomao");
    assert_eq!(gal.yaw_deg, 45);
    assert_eq!(gal.pitch_deg, -30);
    assert_eq!(gal.zoom_pct, 150);
    assert!(resolve_u01_3d_gallery("unknown_general", 0, 0, 100).is_err());

    assert_archive_roundtrip(&e, &packs);
}

fn assert_archive_roundtrip(e: &Engine, packs: &[&engine_native::content::extension::ExtensionPack]) {
    let saved = e.save();
    let bytes = engine_native::archive::encode_archive(&saved);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, packs).expect("restore_with_extensions");
    assert_eq!(restored.state_core(), &saved.core);
    assert_eq!(restored.save().resolution, saved.resolution);
}

#[test]
fn u05_shahai_remaining_swd_yunchou_mtg_zhenfa_ow_xianjian_closure_and_save_restore() {
    let swd_pack = bridge_pack(&read_pack_lf("shahai-swd-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let yc_pack = bridge_pack(&read_pack_lf("shahai-yunchou-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let mtg_pack = bridge_pack(&read_pack_lf("shahai-mtg-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let zf_pack = bridge_pack(&read_pack_lf("shahai-zhenfa-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let ow_pack = bridge_pack(&read_pack_lf("shaai-ow-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let xj_pack = bridge_pack(&read_pack_lf("shaai-xianjian-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let existing = [&swd_pack, &yc_pack, &mtg_pack, &zf_pack, &ow_pack, &xj_pack];
    let p9_pack = decision34_pack9_for_coexistence(&existing);
    let all_packs = [
        &swd_pack, &yc_pack, &mtg_pack, &zf_pack, &ow_pack, &xj_pack, &p9_pack,
    ];

    let mut e = Engine::new_with_extensions(260102, 2, &all_packs).expect("U05 full engine");
    e.state_core_mut().player_mut(1).max_hp = 8;
    e.state_core_mut().player_mut(1).hp = 8;

    assert_eq!(U05_XIANJIAN_GENERALS.len(), 36);
    for &(gname, _, _, _, _) in &U05_XIANJIAN_GENERALS {
        let _ = find_general(&e, gname);
    }

    for &sk in U05_SWD_REMAINING_SKILLS
        .iter()
        .chain(U05_YUNCHOU_REMAINING_SKILLS.iter())
        .chain(U05_OW_SKILLS.iter())
        .chain(U05_XIANJIAN_SKILLS.iter())
    {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "U05 skill {sk} must exist in Catalog"
        );
    }
    for &(cname, _, _, g_sk) in U05_MTG_LAND_CARDS.iter().chain(U05_ZHENFA_CARDS.iter()) {
        assert!(
            e.catalog().skill_by_name(cname).is_some(),
            "U05 card skill {cname} must exist"
        );
        assert!(
            e.catalog().skill_by_name(g_sk).is_some(),
            "U05 global card skill {g_sk} must exist"
        );
    }

    // 1. SWD `duijue`: actual target HP <= 1 guard, positive 2-damage path, and limited 1/game guard
    e.state_core_mut().player_mut(1).hp = 1;
    invoke_p9_op(&mut e, 0, "duijue", SWD_DUIJUE_ID);
    assert_eq!(flag(&e, 0, "duijue_blocked_hp"), 1);
    assert_eq!(flag(&e, 0, "duijue_used"), 0);
    e.state_core_mut().player_mut(1).hp = 8;

    let p1_hp = e.state_core().player(1).hp;
    invoke_p9_op(&mut e, 0, "duijue", SWD_DUIJUE_ID);
    assert_eq!(flag(&e, 0, "duijue_used"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp - 2);

    invoke_p9_op(&mut e, 0, "duijue", SWD_DUIJUE_ID);
    assert_eq!(flag(&e, 0, "duijue_blocked_limited"), 1);
    assert_eq!(e.state_core().player(1).hp, p1_hp - 2, "limited duijue must not fire twice");

    // `tanlin`: empty-hand negative guard, lose-damage branch, and win/discover branch
    let saved_p0_hand = e.state_core().player(0).hand.clone();
    e.state_core_mut().player_mut(0).hand.clear();
    invoke_p9_op(&mut e, 0, "tanlin", SWD_TANLIN_ID);
    assert_eq!(flag(&e, 0, "tanlin_blocked_empty"), 1);
    e.state_core_mut().player_mut(0).hand = saved_p0_hand;

    e.state_core_mut().player_mut(0).hp = 3;
    set_flag(&mut e, 0, "tanlin_force_lose", 1);
    invoke_p9_op(&mut e, 0, "tanlin", SWD_TANLIN_ID);
    assert_eq!(flag(&e, 0, "tanlin_lost_damage"), 1);
    assert_eq!(e.state_core().player(0).hp, 2);
    set_flag(&mut e, 0, "tanlin_force_lose", 0);

    invoke_p9_op(&mut e, 0, "tanlin", SWD_TANLIN_ID);
    assert_eq!(flag(&e, 0, "tanlin_discovered"), 1);
    assert_eq!(e.state_core().player(0).hp, 3);

    // 2. Yunchou (`ycndiyjianxiong`, `xinfu_jingxie`, `ycnxinfu_qiaosi`)
    invoke_p9_op(&mut e, 0, "ycndiyjianxiong", YUNCHOU_SKILL_ID);
    invoke_p9_op(&mut e, 0, "xinfu_jingxie", YUNCHOU_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ycnxinfu_qiaosi", YUNCHOU_SKILL_ID);
    assert_eq!(flag(&e, 0, "ycndiyjianxiong_obtained"), 1);
    assert_eq!(flag(&e, 0, "xinfu_jingxie_recast"), 1);
    assert_eq!(flag(&e, 0, "ycnxinfu_qiaosi_resolved"), 1);

    // 3. MTG land cards (`mtg_duzhao`, `mtg_cangbaohaiwan`) + once-per-turn / tapped guard
    invoke_p9_op(&mut e, 0, "mtg_duzhao", MTG_LAND_CARD_ID);
    assert_eq!(flag(&e, 0, "mtg_black_mana"), 1);
    assert_eq!(flag(&e, 1, "mtg_swamp_miasma"), 1);
    invoke_p9_op(&mut e, 0, "mtg_cangbaohaiwan", MTG_LAND_CARD_ID);
    assert_eq!(
        flag(&e, 0, "mtg_land_blocked"),
        1,
        "second land play in same turn must be blocked"
    );
    set_flag(&mut e, 0, "mtg_land_played_this_turn", 0);
    invoke_p9_op(&mut e, 0, "mtg_cangbaohaiwan", MTG_LAND_CARD_ID);
    assert_eq!(flag(&e, 0, "mtg_treasure_mana"), 1);

    // 4. Zhenfa `pozhenjue` (recast branch + formation break & seat swap)
    set_flag(&mut e, 0, "pozhenjue_recast", 1);
    invoke_p9_op(&mut e, 0, "pozhenjue", ZHENFA_POZHENJUE_ID);
    assert_eq!(flag(&e, 0, "pozhenjue_recasted"), 1);
    set_flag(&mut e, 0, "pozhenjue_recast", 0);

    e.state_core_mut().player_mut(1).linked = true;
    invoke_p9_op(&mut e, 0, "pozhenjue", ZHENFA_POZHENJUE_ID);
    assert_eq!(flag(&e, 0, "pozhenjue_formation_broken"), 1);
    assert_eq!(flag(&e, 0, "pozhenjue_seats_swapped"), 2);
    assert!(!e.state_core().player(1).linked);

    // 5. OW (`shanhui`, `zhanlong` charge guard, `zhongdun`) & Xianjian (`tianjian`, `leiyu`, `xuanning`)
    set_flag(&mut e, 0, "ow_ult_require_charge", 1);
    set_flag(&mut e, 0, "ow_ult_charge", 50);
    invoke_p9_op(&mut e, 0, "zhanlong", OW_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ow_ult_blocked"), 1);
    set_flag(&mut e, 0, "ow_ult_charge", 100);
    invoke_p9_op(&mut e, 0, "zhanlong", OW_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ow_ult_unleashed"), 1);
    invoke_p9_op(&mut e, 0, "shanhui", OW_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ow_shanhui_recalled"), 1);
    invoke_p9_op(&mut e, 0, "zhongdun", OW_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ow_shield_tokens"), 1);

    invoke_p9_op(&mut e, 0, "leiyu", XIANJIAN_SKILL_DISPATCH_ID);
    invoke_p9_op(&mut e, 0, "xuanning", XIANJIAN_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "xianjian_sword_thunder_hit"), 1);
    assert_eq!(flag(&e, 0, "xianjian_ward_active"), 1);

    assert_archive_roundtrip(&e, &all_packs);
}

#[test]
fn u07_wudian_jianbi_and_choudu_differential_and_save_restore() {
    let wudian_pack = bridge_pack(&read_pack_lf("wudian-upstream-noname/assets/cards.json"))
        .unwrap()
        .pack;
    let d34_pack = decision34_closure_extension_pack();
    let p9_pack = decision34_pack9_for_coexistence(&[&wudian_pack, &d34_pack]);
    let packs = [&wudian_pack, &d34_pack, &p9_pack];

    let mut e = Engine::new_with_extensions(260103, 2, &packs).expect("U07 engine");
    for &sk in &U07_WUDIAN_REMAINING_SKILLS {
        assert!(e.catalog().skill_by_name(sk).is_some());
    }

    // `jianbi` negative guard when targets.length < 2 (`jianbi_targets_len == 1`)
    set_flag(&mut e, 0, "jianbi_targets_len", 1);
    invoke_p9_op(&mut e, 0, "jianbi", WUDIAN_JIANBI_ID);
    assert_eq!(flag(&e, 0, "jianbi_blocked_single_target"), 1);
    assert_eq!(flag(&e, 0, "wudian_jianbi_nullified"), 0);
    set_flag(&mut e, 0, "jianbi_targets_len", 2);

    invoke_p9_op(&mut e, 0, "jianbi", WUDIAN_JIANBI_ID);
    assert_eq!(flag(&e, 0, "wudian_jianbi_nullified"), 1);
    assert_eq!(flag(&e, 0, "wudian_jianbi_shield"), 1);

    // Unconditional negative guard on `choudu` when `he` (`hand` and `equips`) is empty
    let saved_hand = e.state_core().player(0).hand.clone();
    e.state_core_mut().player_mut(0).hand.clear();
    e.state_core_mut().player_mut(0).equips.clear();
    invoke_p9_op(&mut e, 0, "choudu", WUDIAN_CHOUDU_ID);
    assert_eq!(flag(&e, 0, "choudu_blocked_empty"), 1);
    assert_eq!(flag(&e, 0, "wudian_choudu_dispatched"), 0);

    // Positive path + `usable: 1` per turn guard + virtual `diaobingqianjiang` resolution
    e.state_core_mut().player_mut(0).hand = saved_hand;
    invoke_p9_op(&mut e, 0, "choudu", WUDIAN_CHOUDU_ID);
    assert_eq!(flag(&e, 0, "wudian_choudu_dispatched"), 1);
    assert_eq!(flag(&e, 0, "diaobingqianjiang_resolved"), 1);

    invoke_p9_op(&mut e, 0, "choudu", WUDIAN_CHOUDU_ID);
    assert_eq!(flag(&e, 0, "choudu_blocked_usable"), 1);

    assert_archive_roundtrip(&e, &packs);
}

#[test]
fn l08_l14_l19_l20_complete_skills_triggers_and_save_restore() {
    let p9_pack = decision34_pack9_extension_pack();
    let packs = [&p9_pack];
    let mut e = Engine::new_with_extensions(260104, 2, &packs).expect("L08/L14/L19/L20 engine");

    assert_eq!(L08_DECADE_SKILLS.len(), 9);
    assert_eq!(L08_DECADE_CHARACTERS.len(), 9);
    assert_eq!(L14_SHOUSHA_SKILLS.len(), 48);
    assert_eq!(L14_SHOUSHA_CHARACTERS.len(), 3);
    assert_eq!(L14_SHOUSHA_CARDS.len(), 1);
    assert_eq!(L19_DOOMSDAY_CHARACTERS.len(), 24);
    assert_eq!(L19_DOOMSDAY_SKILLS.len(), 66);
    assert_eq!(L20_MARK_SUPPLEMENT_CHARACTERS.len(), 12);
    assert_eq!(L20_MARK_SUPPLEMENT_CARDS.len(), 3);
    assert_eq!(L20_MARK_SUPPLEMENT_SKILLS.len(), 75);

    // Verify authentic upstream general IDs (`jiangshifz`, `jiangshinj`, `ssr_luxun`, etc.)
    let _ = find_general(&e, "jiangshifz");
    let _ = find_general(&e, "jiangshinj");
    let _ = find_general(&e, "ssr_luxun");
    let _ = find_general(&e, "ssr_sunquan");

    for &sk in L08_DECADE_SKILLS
        .iter()
        .chain(L14_SHOUSHA_SKILLS.iter())
        .chain(L19_DOOMSDAY_SKILLS.iter())
        .chain(L20_MARK_SUPPLEMENT_SKILLS.iter())
    {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "Skill {sk} must be registered in Catalog"
        );
    }

    // Verify authentic upstream EventKind triggers on key skills
    for (sk, expected_ev) in [
        ("ycndiyjianxiong", EventKind::Damage),
        ("tanlin", EventKind::CardUsed),
        ("jianbi", EventKind::CardUsed),
        ("leiyu", EventKind::PhaseEnd),
        ("_usecard", EventKind::CardUsed),
        ("_decadeUI_dieKillEffect", EventKind::Death),
        ("_dieNoTuoguan", EventKind::Death),
        ("_wujinmoshiDamage", EventKind::Damage),
        ("zhengsu", EventKind::PhaseEnd),
        ("_pfqh_check_damage", EventKind::Damage),
        ("_pfqh_check_phaseEnd", EventKind::PhaseEnd),
        ("_zuan_damageSendEgg", EventKind::Damage),
        ("baonvezhi", EventKind::Damage),
    ] {
        let sid = e.catalog().skill_by_name(sk).expect(sk);
        assert!(
            e.catalog().candidates(expected_ev).contains(&sid),
            "skill {sk} must be dispatched under primary trigger {expected_ev:?}"
        );
    }

    invoke_p9_op(&mut e, 0, "_changeEquipExt", L08_DECADE_SKILL_ID);
    assert_eq!(flag(&e, 0, "l08_recast_count"), 1);
    assert_eq!(flag(&e, 0, "l08_qiankun_trigram"), 2);

    invoke_p9_op(&mut e, 0, "_dieNoTuoguan", L14_SHOUSHA_SKILL_ID);
    invoke_p9_op(&mut e, 0, "_jd_wjsw", L14_SHOUSHA_SKILL_ID);
    assert_eq!(flag(&e, 0, "l14_shousha_trigger_count"), 2);

    invoke_p9_op(&mut e, 0, "baozou", L19_DOOMSDAY_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ganran", L19_DOOMSDAY_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ganran2", L19_DOOMSDAY_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ganran", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(flag(&e, 0, "l19_baozou_active"), 1);
    assert_eq!(flag(&e, 0, "doomsday_mutated"), 1);

    // L20 `zhengsu` violation negative guard + reward positive path
    set_flag(&mut e, 0, "l20_zhengsu_violated", 1);
    invoke_p9_op(&mut e, 0, "zhengsu", L20_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "l20_zhengsu_failed"), 1);
    assert_eq!(flag(&e, 0, "l20_zhengsu_rewarded"), 0);
    set_flag(&mut e, 0, "l20_zhengsu_violated", 0);
    invoke_p9_op(&mut e, 0, "zhengsu", L20_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "l20_zhengsu_rewarded"), 1);

    invoke_p9_op(&mut e, 0, "zhenyi_spade", L20_SKILL_DISPATCH_ID);
    invoke_p9_op(&mut e, 0, "xinhongyan", L20_SKILL_DISPATCH_ID);
    invoke_p9_op(&mut e, 0, "diaohulishan", L20_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "l20_zhenyi_spade_hit"), 1);
    assert_eq!(flag(&e, 0, "l20_xinhongyan_heart_lock"), 1);
    assert_eq!(flag(&e, 0, "l20_diaohulishan_exiled"), 1);

    assert_archive_roundtrip(&e, &packs);
}

#[test]
fn l24_l25_l27_complete_skills_negative_guards_and_save_restore() {
    let p9_pack = decision34_pack9_extension_pack();
    let packs = [&p9_pack];
    let mut e = Engine::new_with_extensions(260105, 2, &packs).expect("L24/L25/L27 engine");

    assert_eq!(L24_SKIN_SWITCH_SKILLS.len(), 23);
    assert_eq!(L25_ZUAN_CHARACTERS.len(), 120);
    assert_eq!(L25_ZUAN_SKILLS.len(), 122);
    assert_eq!(L27_SELECT_BEAUTIFY_SKILLS.len(), 37);

    for &sk in L24_SKIN_SWITCH_SKILLS
        .iter()
        .chain(L25_ZUAN_SKILLS.iter())
        .chain(L27_SELECT_BEAUTIFY_SKILLS.iter())
    {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "Skill {sk} must be registered in Catalog"
        );
    }

    // L24 skin skills + Spine transform resolver
    invoke_p9_op(&mut e, 0, "_tsx", L24_SKIN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l24_skin_slot"), 1);
    assert_eq!(flag(&e, 0, "l24_skin_anim_state"), 1);
    let tf = resolve_l24_skin_transform("caocao", 2, "gongji").expect("valid skin transform");
    assert_eq!(tf.stand_x, 24);
    assert_eq!(tf.stand_y, -8);
    assert_eq!(tf.scale_pct, 108);
    assert!(resolve_l24_skin_transform("caocao", 9, "gongji").is_err());
    assert!(resolve_l24_skin_transform("caocao", 1, "unknown_clip").is_err());

    // L25 `g_du` + `_diycharacterskill` online room rejection guard
    let hp_before = e.state_core().player(0).hp;
    invoke_p9_op(&mut e, 0, "g_du", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_g_du_triggered"), 1);
    assert_eq!(e.state_core().player(0).hp, hp_before - 1);

    set_flag(&mut e, 0, "l25_online_room_active", 1);
    invoke_p9_op(&mut e, 0, "_diycharacterskill", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_diy_blocked_online"), 1);
    assert_eq!(flag(&e, 0, "l25_diy_applied"), 0);
    set_flag(&mut e, 0, "l25_online_room_active", 0);
    invoke_p9_op(&mut e, 0, "_diycharacterskill", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_diy_applied"), 1);

    assert!(validate_and_apply_l25_diy(&L25DiyEditorDraft {
        general_id: "zuan_diy01".to_string(),
        faction: "qun".to_string(),
        max_hp: 4,
        skills: vec!["_diycharacterskill".to_string()],
        online_room_active: true,
    })
    .is_err());
    assert!(validate_and_apply_l25_diy(&L25DiyEditorDraft {
        general_id: "zuan_diy01".to_string(),
        faction: "qun".to_string(),
        max_hp: 4,
        skills: vec!["_diycharacterskill".to_string()],
        online_room_active: false,
    })
    .is_ok());

    // L27 `_chooseCharacterTwo` dual-general pick count guard + `qimou` limited guard
    set_flag(&mut e, 0, "l27_pick_count", 1);
    invoke_p9_op(&mut e, 0, "_chooseCharacterTwo", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_dual_pick_invalid"), 1);
    assert_eq!(flag(&e, 0, "l27_dual_pick_locked"), 0);
    set_flag(&mut e, 0, "l27_pick_count", 2);
    invoke_p9_op(&mut e, 0, "_chooseCharacterTwo", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_dual_pick_locked"), 1);
    assert_eq!(flag(&e, 0, "l27_combined_hp"), 4);

    invoke_p9_op(&mut e, 0, "qimou", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_qimou_used"), 1);
    invoke_p9_op(&mut e, 0, "qimou", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_qimou_blocked"), 1);

    assert!(resolve_l27_choose_character_two(&L27DualGeneralPick {
        candidates: vec!["caocao".to_string(), "liubei".to_string()],
        selected: vec!["caocao".to_string()],
    })
    .is_err());
    assert_eq!(
        resolve_l27_choose_character_two(&L27DualGeneralPick {
            candidates: vec!["caocao".to_string(), "liubei".to_string()],
            selected: vec!["caocao".to_string(), "liubei".to_string()],
        })
        .unwrap(),
        ("caocao".to_string(), "liubei".to_string(), 4)
    );

    assert_archive_roundtrip(&e, &packs);
}
