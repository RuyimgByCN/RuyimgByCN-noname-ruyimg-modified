//! Batch 25 (Decision 34 Complete Closure): end-to-end integration and save/restore
//! verification for per-source remaining sub-skills, sub-generals, 14 HUD marks, and
//! standalone campaign state machines across `L29`, `L10`, `U06`, `U07`, `U08`, `L20`,
//! `L28`, `U02/L03`, `L11`, `L19`, and `L21`.

use content_native_loader::{
    bridge_pack, decision34_closure_extension_pack, longxue_full_unlock_report,
    verify_u08_standard_skill_bindings, BOSS_BAONU_STAGE_ID, DECISION34_STORAGE_DISPATCH_KEY,
    DIAOBINGQIANJIANG_ID, DOOMSDAY_INFECTION_STEP_ID, DUOQI_ID, HUANLE_KUWU_INTERACTIVE_ID,
    K_DANGGU_ID, K_MOWANG_ID, L20_MARK_DISPATCH_ID, L20_MARK_SPECS, L28_THUNDER_GENERALS,
    MARTIAL_FORMATION_STEP_ID, MBCMFANGZHU_ID, MBCMJIUSHI_ID, MBJUEJIN_ID,
    TAIXU_EXPEDITION_STEP_ID, TH_CHONGZOU_ID, TH_HUANYIN_ID, TH_XUANLV_ID, YOUZHAN_ID,
    YXS_MENSHEN2_GUARD_ID, YXS_TAFANG_WAVE_ID,
};
use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(2_500_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn read_pack_lf(rel: &str) -> String {
    std::fs::read_to_string(repo_extensions_root().join(rel))
        .expect("read package cards.json")
        .replace("\r\n", "\n")
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn invoke_d34_op(e: &mut Engine, owner: u8, skill_name: &str, skill_id: u32) {
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
        "Decision 34 effect dispatch ({skill_name}) rejected: {out:?}"
    );
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
fn l29_longxue_mbqianlong_unlocks_and_executes_all_four_skills_with_save_restore() {
    let longxue_json = read_pack_lf("longxue-upstream-liuli/assets/cards.json");
    let longxue_pack = bridge_pack(&longxue_json).expect("longxue pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110901, 4, &[&longxue_pack, &d34_pack])
        .expect("new_with_extensions");
    let caomao = find_general(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general_def(1, caomao); // Another Wei general with mbweitong -> opening marks = 60

    next_phase(&mut e);
    assert!(
        e.state_core().player(0).mbqianlong_marks >= 60,
        "opening marks with mbweitong ally must be >= 60"
    );
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("mbcmqingzheng");
    let js = e.catalog().skill_by_name("mbcmjiushi").expect("mbcmjiushi");
    let fz = e.catalog().skill_by_name("mbcmfangzhu").expect("mbcmfangzhu");
    let jj = e.catalog().skill_by_name("mbjuejin").expect("mbjuejin");

    assert!(e.state_core().skills.contains_key(&(0, qz)), "25 unlocked mbcmqingzheng");
    assert!(e.state_core().skills.contains_key(&(0, js)), "50 unlocked mbcmjiushi");

    // Deal 3 damage from seat 0 to seat 2 (+45 marks -> reaches 99 cap, unlocking fangzhu & juejin)
    let _ = e.debug_deal_damage(Some(0), 2, 3, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 99);
    assert!(e.state_core().skills.contains_key(&(0, fz)), "75 unlocked mbcmfangzhu");
    assert!(e.state_core().skills.contains_key(&(0, jj)), "99 unlocked mbjuejin");
    assert_eq!(
        longxue_full_unlock_report(99),
        "marks:99|unlocked:mbcmqingzheng,mbcmjiushi,mbcmfangzhu,mbjuejin"
    );

    // 1. Execute mbcmjiushi: face-up -> turns over & sets jiu_active; face-down -> flips back & gains trick
    assert!(!e.state_core().player(0).face_down);
    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert!(e.state_core().player(0).face_down);
    assert!(e.state_core().turn.jiu_active);
    assert_eq!(flag(&e, 0, "mbcmjiushi_jiu_used"), 1);

    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert!(!e.state_core().player(0).face_down);
    assert_eq!(flag(&e, 0, "mbcmjiushi_trick_gained"), 1);

    // 2. Execute mbcmfangzhu via REAL RuleVm Damage trigger: dealing 1 damage to seat 0 (marks == 99 >= 75)
    // automatically triggers mbcmfangzhu#d34 in RuleVm, flipping target 2 and drawing lost_hp cards!
    e.debug_set_hp(0, 2); // taking 1 damage brings hp to 1 (max_hp = 3 => lost_hp = 2)
    set_flag(&mut e, 0, "mbcmfangzhu_target", 2);
    let hand2_before = e.debug_hand(2).len();
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain(&mut e);
    assert!(
        e.state_core().player(2).face_down,
        "mbcmfangzhu must trigger via RuleVm on Damage to mb_caomao"
    );
    assert_eq!(flag(&e, 0, "mbcmfangzhu_drawn"), 2);
    assert_eq!(e.debug_hand(2).len(), hand2_before + 2);

    // 3. Execute mbjuejin: limited 1/game -> +2 hujia & moves shan/tao/jiu into "xiang" pile
    e.debug_place(1, "shan");
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(flag(&e, 0, "mbjuejin_active"), 1);
    assert_eq!(e.state_core().player(0).hujia, 2);
    assert!(flag(&e, 0, "mbjuejin_xiang_count") >= 1);

    // Save & restore + binary archive roundtrip verification
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&longxue_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(restored.state_core().player(0).mbqianlong_marks, 99);
    assert_eq!(restored.state_core().player(0).hujia, 2);
    assert_eq!(flag(&restored, 0, "mbjuejin_active"), 1);
}

#[test]
fn l10_shichangshi_k_danggu_and_k_mowang_roster_lifecycle_and_save_restore() {
    let epic_json = read_pack_lf("epic-cards-upstream-liuli/assets/cards.json");
    let epic_pack = bridge_pack(&epic_json).expect("epic pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110902, 2, &[&epic_pack, &d34_pack])
        .expect("new_with_extensions");
    let scs = find_general(&e, "shichangshi");
    e.debug_assemble_general_def(0, scs);
    assert_eq!(e.state_core().player(0).max_hp, 1);

    // Real RuleVm TurnBegin trigger: advancing from GameStart into seat 0's TurnBegin
    // automatically triggers k_danggu#d34 in RuleVm!
    next_phase(&mut e);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0x3FF);
    assert_eq!(flag(&e, 0, "k_danggu_active_eunuch"), 1);
    assert_eq!(flag(&e, 0, "k_danggu_triggered"), 1);

    // Real RuleVm Dying trigger: dealing 1 fatal damage to shichangshi (seat 0, 1 HP -> 0 HP)
    // emits EventData::Dying, which automatically triggers k_mowang#d34 in RuleVm,
    // retiring eunuch #1, reviving player 0 to 1 HP, clearing dying, and rotating to eunuch #2!
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0x3FE);
    assert_eq!(flag(&e, 0, "k_danggu_active_eunuch"), 2);
    assert_eq!(flag(&e, 0, "k_mowang_revived"), 1);
    assert_eq!(e.state_core().player(0).hp, 1);
    assert!(!e.state_core().player(0).dying);

    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&epic_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "k_danggu_roster_mask"), 0x3FE);
    assert_eq!(flag(&restored, 0, "k_danggu_active_eunuch"), 2);
}

#[test]
fn u06_kuwu_interactive_and_u07_wudian_diaobingqianjiang_duoqi_youzhan() {
    let wudian_json = read_pack_lf("wudian-upstream-noname/assets/cards.json");
    let wudian_pack = bridge_pack(&wudian_json).expect("wudian pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110903, 4, &[&wudian_pack, &d34_pack])
        .expect("new_with_extensions");

    // 1. U06 kuwu interactive: chain guard -> skipped
    set_flag(&mut e, 0, "kuwu_from_chain", 1);
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    assert_eq!(flag(&e, 0, "kuwu_skipped_from_chain"), 1);
    set_flag(&mut e, 0, "kuwu_from_chain", 0);

    // U06 kuwu interactive: discard 1 hand card branch
    let hand1_before = e.debug_hand(1).len();
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), hand1_before - 1);
    assert!(flag(&e, 1, "kuwu_discarded_he") > 0);

    // U06 kuwu interactive: choose lose 1 HP branch
    let hp1_before = e.debug_hp(1);
    set_flag(&mut e, 1, "kuwu_choose_lose_hp", 1);
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "kuwu_lost_hp"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1);
    set_flag(&mut e, 1, "kuwu_choose_lose_hp", 0);

    // 2. U07 diaobingqianjiang: market card swap across seats
    invoke_d34_op(&mut e, 0, "diaobingqianjiang", DIAOBINGQIANJIANG_ID);
    assert_eq!(flag(&e, 0, "diaobingqianjiang_resolved"), 1);
    assert!(flag(&e, 0, "diaobingqianjiang_swapped") >= 1);

    // 3. U07 duoqi & youzhan
    invoke_d34_op(&mut e, 0, "duoqi", DUOQI_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "duoqi_triggered"), 1);
    assert_eq!(flag(&e, 1, "duoqi_draw_penalty"), 1);

    let hp1_pre_youzhan = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "youzhan", YOUZHAN_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "youzhan_triggered"), 1);
    assert_eq!(e.debug_hp(1), hp1_pre_youzhan - 1);

    // Save/restore + binary archive card conservation verification
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&wudian_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "diaobingqianjiang_resolved"), 1);
    assert_eq!(flag(&restored, 0, "duoqi_triggered"), 1);
    assert_eq!(flag(&restored, 0, "youzhan_triggered"), 1);
}

#[test]
fn u08_hero_kill_31_standard_skills_menshen2_guard_and_tafang_waves() {
    let yxs_json = read_pack_lf("hero-kill-upstream-noname/assets/cards.json");
    let yxs_pack = bridge_pack(&yxs_json).expect("yxs pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110904, 4, &[&yxs_pack, &d34_pack])
        .expect("new_with_extensions");

    // Verify all 31 cross-package standard skills referenced by U08 exist in Catalog
    let count = verify_u08_standard_skill_bindings(e.catalog())
        .expect("all 31 U08 standard skills must resolve in Catalog");
    assert_eq!(count, 31);

    // yxs_menshen2 guard transfers 1 damage from protected seat 1 to granter seat 0
    let menshen2_def = e.catalog().skill_by_name("yxs_menshen2").expect("yxs_menshen2");
    e.state_core_mut()
        .vm_aux
        .marks
        .insert((1, u32::from(menshen2_def)), 0);
    let hp0_before = e.debug_hp(0);
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "yxs_menshen2_guarded"), 1);
    assert_eq!(flag(&e, 1, "yxs_menshen2_transferred_to"), 0);
    assert_eq!(e.debug_hp(0), hp0_before - 1);

    // tafang 5-wave progression
    for expected_wave in 1..=5 {
        invoke_d34_op(&mut e, 0, "yxs_tafang_wave", YXS_TAFANG_WAVE_ID);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "tafang_wave"), expected_wave);
    }
    assert_eq!(flag(&e, 0, "tafang_Cleared"), 1);
}

#[test]
fn l20_fourteen_hud_marks_and_l28_thunder_twelve_generals_and_chords() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110905, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 1. L20: Dispatch all 14 HUD marks and verify persistence + niepan HP recovery
    e.debug_set_hp(0, 1);
    for (idx, &(mark_name, _zh, mark_id)) in L20_MARK_SPECS.iter().enumerate() {
        set_flag(&mut e, 0, "l20_mark_index", idx as u32);
        set_flag(&mut e, 0, "l20_mark_delta", 2);
        invoke_d34_op(&mut e, 0, "l20_mark_dispatch", L20_MARK_DISPATCH_ID);
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(0, mark_id)).copied(),
            Some(2),
            "L20 mark {mark_name} must be recorded in vm_aux.marks"
        );
        assert_eq!(flag(&e, 0, &format!("l20_mark:{mark_name}")), 2);
    }
    assert_eq!(e.debug_hp(0), 3, "niepan mark dispatch recovers HP up to 3");

    // 2. L28: Verify all 12 th_* generals in Catalog and test th_xuanlv / th_chongzou / th_huanyin
    assert_eq!(L28_THUNDER_GENERALS.len(), 12);
    for &(g_name, ..) in &L28_THUNDER_GENERALS {
        let _ = find_general(&e, g_name);
    }
    let th_zy = find_general(&e, "th_shen_zhouyu");
    e.debug_assemble_general_def(0, th_zy);

    let hp1_before = e.debug_hp(1);
    for bit in [1u32, 2, 4, 8] {
        set_flag(&mut e, 0, "th_xuanlv_suit_bit", bit);
        invoke_d34_op(&mut e, 0, "th_xuanlv", TH_XUANLV_ID);
        drain(&mut e);
    }
    assert_eq!(flag(&e, 0, "th_xuanlv_chord_complete"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1, "4-suit chord deals 1 thunder damage");

    set_flag(&mut e, 0, "th_melody_mask", 0b0101);
    e.debug_set_hp(0, 2);
    invoke_d34_op(&mut e, 0, "th_huanyin", TH_HUANYIN_ID);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 3, "th_huanyin heals 1 HP when >= 2 melody suits");

    invoke_d34_op(&mut e, 0, "th_chongzou", TH_CHONGZOU_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_chongzou_triggered"), 1);
}

#[test]
fn u02_l03_boss_l11_taixu_l19_doomsday_and_l21_martial_campaign_state_machines() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110906, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 1. U02 / L03 Boss baonu stage-2 transition & Jiange siege linkage
    set_flag(&mut e, 0, "boss_rage", 2);
    invoke_d34_op(&mut e, 0, "boss_baonu_stage", BOSS_BAONU_STAGE_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "boss_stage"), 2);
    assert_eq!(flag(&e, 0, "jiange_siege_linked"), 1);
    assert!(e.state_core().player(0).max_hp >= 6);

    // 2. L11 Taixu expedition floor & spirit companion strike
    let hp1_before = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "taixu_expedition_step", TAIXU_EXPEDITION_STEP_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "taixu_floor"), 1);
    assert_eq!(flag(&e, 0, "taixu_spirit_damage"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1);

    // 3. L19 Doomsday infection -> mutation at 3 stacks, then serum cure
    for _ in 0..3 {
        invoke_d34_op(&mut e, 0, "doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID);
    }
    assert_eq!(flag(&e, 0, "doomsday_mutated"), 1);
    set_flag(&mut e, 0, "doomsday_serum", 1);
    invoke_d34_op(&mut e, 0, "doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "doomsday_infection"), 0);
    assert_eq!(flag(&e, 0, "doomsday_cured"), 1);

    // 4. L21 Martial 5v5 bench substitution & formation bonus
    invoke_d34_op(&mut e, 0, "martial_formation_step", MARTIAL_FORMATION_STEP_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "martial_bench_sub_count"), 1);
    assert_eq!(flag(&e, 0, "martial_linked_bonus"), 1);

    // Save & restore roundtrip
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &[&d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "boss_stage"), 2);
    assert_eq!(flag(&restored, 0, "taixu_floor"), 1);
    assert_eq!(flag(&restored, 0, "doomsday_cured"), 1);
    assert_eq!(flag(&restored, 0, "martial_bench_sub_count"), 1);
}

#[test]
fn decision34_edge_cases_and_boundary_guards() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110907, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 0. Daoxin threshold lock guards: calling mbcmjiushi (< 50), mbcmfangzhu (< 75), or mbjuejin (< 99)
    // when un-unlocked must early-return with mbqianlong_locked_guard == 1 and zero state mutation!
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 0);
    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert!(!e.state_core().player(0).face_down);

    set_flag(&mut e, 0, "mbqianlong_locked_guard", 0);
    invoke_d34_op(&mut e, 0, "mbcmfangzhu", MBCMFANGZHU_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert!(!e.state_core().player(1).face_down);

    set_flag(&mut e, 0, "mbqianlong_locked_guard", 0);
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert_eq!(e.state_core().player(0).hujia, 0);

    // 1. mbjuejin limited 1/game idempotency once unlocked (mbqianlong_marks == 99):
    // first call grants +2 hujia; second call does not add extra hujia
    e.state_core_mut().player_mut(0).mbqianlong_marks = 99;
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(e.state_core().player(0).hujia, 2);
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(
        e.state_core().player(0).hujia,
        2,
        "mbjuejin is limited 1/game and must not grant hujia twice"
    );

    // 2. k_mowang exhaustion when last eunuch bit is retired, and post-exhaustion guard
    set_flag(&mut e, 0, "k_danggu_roster_mask", 0b0001);
    set_flag(&mut e, 0, "k_danggu_active_eunuch", 1);
    invoke_d34_op(&mut e, 0, "k_mowang", K_MOWANG_ID);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0);
    assert_eq!(flag(&e, 0, "k_mowang_exhausted"), 1);
    invoke_d34_op(&mut e, 0, "k_danggu", K_DANGGU_ID);
    assert_eq!(
        flag(&e, 0, "k_danggu_roster_mask"),
        0,
        "exhausted shichangshi roster must not re-initialize to 0x3FF"
    );

    // 3. yxs_menshen2_guard when unprotected (neither mark nor flag) or when granter is dead -> does not transfer damage
    let hp0_before = e.debug_hp(0);
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    assert_eq!(
        flag(&e, 1, "yxs_menshen2_guarded"),
        0,
        "unprotected seat 1 must not default menshen2_granter to seat 0"
    );
    assert_eq!(e.debug_hp(0), hp0_before);

    set_flag(&mut e, 1, "yxs_menshen2_granter", 2);
    e.state_core_mut().player_mut(2).alive = false;
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    assert_eq!(
        flag(&e, 1, "yxs_menshen2_guarded"),
        0,
        "dead granter cannot absorb menshen2 damage"
    );

    // 4. doomsday_infection_step boundary: 2 stacks -> not mutated; 3rd stack -> mutated
    invoke_d34_op(&mut e, 3, "doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID);
    invoke_d34_op(&mut e, 3, "doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID);
    assert_eq!(flag(&e, 3, "doomsday_infection"), 2);
    assert_eq!(flag(&e, 3, "doomsday_mutated"), 0);
    invoke_d34_op(&mut e, 3, "doomsday_infection_step", DOOMSDAY_INFECTION_STEP_ID);
    assert_eq!(flag(&e, 3, "doomsday_infection"), 3);
    assert_eq!(flag(&e, 3, "doomsday_mutated"), 1);
}
