//! Batch 26 (Decision 34 Complete 37-Source In-Repo Closure): end-to-end integration,
//! differential, negative/boundary guard, and save/restore verification for the
//! remaining 9 sources (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`)
//! plus `U07` (`jianbi`, `choudu`).

use content_native_loader::{
    bridge_pack, decision34_closure_extension_pack, decision34_pack9_extension_pack,
    decision34_pack9_for_coexistence, resolve_l24_skin_transform,
    resolve_l27_choose_character_two, resolve_u01_3d_gallery, validate_and_apply_l25_diy,
    DDD_SKILL_DISPATCH_ID, DECISION34_STORAGE_DISPATCH_KEY, L08_DECADE_SKILLS,
    L08_DECADE_SKILL_ID, L14_SHOUSHA_SKILLS, L14_SHOUSHA_SKILL_ID, L19_DOOMSDAY_CHARACTERS,
    L19_DOOMSDAY_SKILLS, L19_DOOMSDAY_SKILL_ID, L20_MARK_SUPPLEMENT_CARDS,
    L20_MARK_SUPPLEMENT_SKILLS, L20_SKILL_DISPATCH_ID, L24_SKIN_SKILL_ID, L24_SKIN_SWITCH_SKILLS,
    L25DiyEditorDraft, L25_ZUAN_CHARACTERS, L25_ZUAN_SKILLS, L25_ZUAN_SKILL_ID,
    L27DualGeneralPick, L27_SELECT_BEAUTIFY_SKILLS, L27_SELECT_SKILL_ID, MTG_LAND_CARD_ID,
    OW_SKILL_DISPATCH_ID, SWD_DUIJUE_ID, SWD_TANLIN_ID, U01_DDD_GENERALS, U01_DDD_SKILLS,
    U05_MTG_LAND_CARDS, U05_OW_SKILLS, U05_SWD_REMAINING_SKILLS, U05_XIANJIAN_GENERALS,
    U05_XIANJIAN_SKILLS, U05_YUNCHOU_REMAINING_SKILLS, U05_ZHENFA_CARDS,
    U07_WUDIAN_REMAINING_SKILLS, WUDIAN_CHOUDU_ID, WUDIAN_JIANBI_ID, XIANJIAN_SKILL_DISPATCH_ID,
    YUNCHOU_SKILL_ID, ZHENFA_POZHENJUE_ID,
};
use engine_native::event::{EventKind, Phase};
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

#[allow(deprecated)]
fn invoke_p9_op(e: &mut Engine, owner: u8, skill_name: &str, skill_id: u32) {
    let out = e.debug_execute_effect(
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

    // 1. `dddxiaxing` 侠行 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59-147`):
    // Negative guard: when `he` (`hand` and `equips`) is empty, `dddxiaxing` does NOT trigger on `Phase::Prepare`.
    let zhangmiao = find_general(&e, "ddd_zhangmiao");
    e.assign_general(0, zhangmiao).expect("assign ddd_zhangmiao");
    let saved_hand = e.state_core().player(0).hand.clone();
    e.state_core_mut().player_mut(0).hand.clear();
    e.state_core_mut().player_mut(0).equips.clear();
    e.debug_set_turn(1, Phase::End, 1);
    let out_empty = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out_empty, CommandOutcome::Accepted { .. }),
        "dddxiaxing must not open choice when hand and equips are empty: {out_empty:?}"
    );
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 0);
    e.state_core_mut().player_mut(0).hand = saved_hand;

    // Positive interactive trigger on `Phase::Prepare` (`skill.js:59-109`):
    // Advance seat 1 from `Phase::End` into seat 0's `Phase::Prepare` -> opens `"dddxiaxing_card"`.
    let c_discard = e.state_core().player(0).hand[0];
    let shan_in_deck = e.debug_place(1, "shan");
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != shan_in_deck);
    e.state_core_mut().deck.push(shan_in_deck);
    e.debug_set_turn(1, Phase::End, 1);
    let out_prep = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let choice = match out_prep {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxiaxing_card choice, got {other:?}"),
    };
    assert_eq!(choice.spec.prompt, "dddxiaxing_card");
    assert!(choice.spec.allow_cancel);

    // Mid-choice save/restore roundtrip while `dddxiaxing_card` is open:
    let pending_snap = e.save();
    let mut restored_pending =
        Engine::restore_with_extensions(&pending_snap, &packs).expect("restore mid-choice dddxiaxing");
    assert_eq!(restored_pending.state_hash(), e.state_hash());

    let answer_cmd = GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![engine_native::ChoiceOption::Card(c_discard)],
        },
    };
    let _ = restored_pending.submit(answer_cmd.clone());
    let _ = e.submit(answer_cmd);
    assert_eq!(restored_pending.state_hash(), e.state_hash());
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 1);
    assert!(!e.state_core().player(0).hand.contains(&c_discard));
    assert_eq!(
        e.state_core().deck.last().copied(),
        Some(c_discard),
        "dddxiaxing must place the chosen card on top of the draw pile"
    );
    assert!(
        e.state_core().turn.jiu_active,
        "dddxiaxing_begin on Prepare must treat target as using virtual Jiu"
    );

    // Active view-as (`dddxiaxing_viewas` at `skill.js:110-147`):
    // In `Phase::Play`, consuming `dddxiaxing_xia` views as a virtual `Sha` without distance limit.
    let dddxiaxing_def = e.catalog().skill_by_name("dddxiaxing").expect("dddxiaxing def");
    let cur_round = e.turn().round;
    e.debug_set_turn(0, Phase::Play, cur_round);
    let out_viewas = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: dddxiaxing_def },
    });
    let sha_choice = match out_viewas {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending zhangba_sha_target choice, got {other:?}"),
    };
    assert_eq!(sha_choice.spec.prompt, "zhangba_sha_target");
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: sha_choice.id,
            selection: vec![engine_native::ChoiceOption::Player(1)],
        },
    });
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 0);

    // 2. `dddqianlong` 潜龙 (`3D精选/character/skill.js:2819`):
    // Lord skill on `ddd_caomao` — draws 3 cards upon killing a character (`player.draw(3)` at `skill.js:2842`), NO daoxin marks.
    let hand_before_qianlong = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "dddqianlong", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qianlong_kill_reward"), 1);
    assert_eq!(flag(&e, 0, "ddd_qianlong_daoxin"), 0, "U01 dddqianlong must not use L29 daoxin marks");
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_qianlong + 3);

    // 3. `dddqiaoduan` 巧断 (`3D精选/character/skill.js:747`):
    // When linked (`ddd_qiaoduan_from_link = 1`), puts X cards on bottom of deck and recovers 1 HP.
    e.state_core_mut().player_mut(0).hp = 2;
    set_flag(&mut e, 0, "ddd_qiaoduan_from_link", 1);
    invoke_p9_op(&mut e, 0, "dddqiaoduan", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qiaoduan_mode"), 2);
    assert_eq!(e.state_core().player(0).hp, 3);

    // 4. `dddyeshen` 夜审 (`3D精选/character/skill.js:656`):
    // Counts down from 3 (`2 -> 1 -> 0`), and on reaching 0 resets to 3 and deals 1 fire damage to owner.
    invoke_p9_op(&mut e, 0, "dddyeshen", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_yeshen_triggered"), 1);
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 2);
    invoke_p9_op(&mut e, 0, "dddyeshen", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 1);
    let hp_before_backfire = e.state_core().player(0).hp;
    invoke_p9_op(&mut e, 0, "dddyeshen", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "dddyeshen_count"), 3);
    assert_eq!(flag(&e, 0, "dddyeshen_fire_backfire"), 1);
    assert_eq!(e.state_core().player(0).hp, hp_before_backfire - 1);

    // 5. `dddqice` 奇策 (`3D精选/character/skill.js:3507`):
    // At PhaseZhunbei draws 2 cards (`player.draw(2)`) and disables `dddqice` (`ddd_qice_disabled = 1`)
    // until the player uses a trick card (`ddd_qice_trick_used = 1`).
    let hand_before_qice = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "dddqice", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qice_disabled"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_qice + 2);

    // Second invocation while disabled is blocked without drawing
    invoke_p9_op(&mut e, 0, "dddqice", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qice_blocked_disabled"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_qice + 2);

    // Using a trick card re-enables `dddqice`
    set_flag(&mut e, 0, "ddd_qice_trick_used", 1);
    invoke_p9_op(&mut e, 0, "dddqice", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_qice_disabled"), 0);

    // 6. `dddjungui` 隽轨 (`3D精选/character/skill.js:384`), `dddshilie` 示烈 (`skill.js:511`),
    // and `ddddiedang` 跌宕 (`skill.js:802`) authentic behavior:
    invoke_p9_op(&mut e, 0, "dddjungui", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_jungui_triggered"), 1);
    assert_eq!(flag(&e, 1, "ddd_jungui_shown_hand"), 1);
    assert_eq!(flag(&e, 0, "ddd_jungui_discarded"), 2);

    let hand_before_shilie = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "dddshilie", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_shilie_viewas_shashan"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_shilie);
    invoke_p9_op(&mut e, 0, "dddshilie", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "dddshilie_blocked_used"), 1);

    let hand_before_diedang = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "ddddiedang", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddddiedang_state"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_diedang + 2);
    invoke_p9_op(&mut e, 0, "ddddiedang", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddddiedang_state"), 0);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_diedang);

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
    assert_eq!(L14_SHOUSHA_SKILLS.len(), 48);
    assert_eq!(L19_DOOMSDAY_CHARACTERS.len(), 2);
    assert_eq!(L19_DOOMSDAY_SKILLS.len(), 66);
    assert_eq!(L20_MARK_SUPPLEMENT_CARDS.len(), 3);
    assert_eq!(L20_MARK_SUPPLEMENT_SKILLS.len(), 74);

    // Verify authentic upstream general IDs (`jiangshifz`, `jiangshinj` in `末日乱斗/extension.js:65-66`,
    // `ssr_luxun`, `ssr_sunquan` in `祖安扩展/extension.js:56051`)
    let _ = find_general(&e, "jiangshifz");
    let _ = find_general(&e, "jiangshinj");
    let _ = find_general(&e, "ssr_luxun");
    let _ = find_general(&e, "ssr_sunquan");

    // Verify fabricated generals and skills are absent from Catalog
    for fabricated_general in [
        "decade_caocao",
        "decade_liubei",
        "vir_zombie",
        "vir_tyrant",
        "ms_huangzhong",
        "ms_weiyan",
        "zuan_caocao",
        "zuan_diy01",
    ] {
        assert!(
            (0..e.catalog().general_count() as u16)
                .all(|idx| e.catalog().general_meta(idx).map(|m| m.0) != Some(fabricated_general)),
            "fabricated general {fabricated_general} must not exist in Catalog"
        );
    }
    for fabricated_skill in [
        "_decade_qiankunbagua",
        "shousha_qiankunbagua",
        "spshidi",
        "spcishi",
        "zhefu",
        "yidu",
        "diaohulishan_card",
        "huoshaolianying",
        "lulitongxin",
        "ms_longmark",
        "gx_Bozhi",
        "zuan_jianxiong",
    ] {
        assert!(
            e.catalog().skill_by_name(fabricated_skill).is_none(),
            "fabricated skill {fabricated_skill} must not exist in Catalog"
        );
    }

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

    // Verify authentic upstream EventKind triggers on key VM skills, plus real `HostBehavior::WudianJianbi` on `jianbi`
    assert!(
        p9_pack
            .host_skills
            .iter()
            .any(|hs| hs.name == "jianbi" && hs.behavior.aoe_exclude()),
        "jianbi must bind real HostBehavior::WudianJianbi (`host:wudian_jianbi`) rather than a shadowed VM stub"
    );
    for (sk, expected_ev) in [
        ("ycndiyjianxiong", EventKind::Damage),
        ("tanlin", EventKind::CardUsed),
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

    // L08 UI hook (`十周年UI/skill.js:84`) records hook trigger WITHOUT drawing cards
    let hand_before_l08 = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "_changeEquipExt", L08_DECADE_SKILL_ID);
    assert_eq!(flag(&e, 0, "l08_recast_count"), 1);
    assert_eq!(flag(&e, 0, "l08_qiankun_trigram"), 0);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_l08,
        "L08 UI hook _changeEquipExt must not draw cards"
    );

    // L14 UI hooks (`手杀ui/skill/main1.js:34`, `lbtn/main1.js:805`) record hook triggers WITHOUT drawing cards or dealing damage
    let hand_before_l14 = e.state_core().player(0).hand.len();
    let hp1_before_l14 = e.state_core().player(1).hp;
    invoke_p9_op(&mut e, 0, "_dieNoTuoguan", L14_SHOUSHA_SKILL_ID);
    invoke_p9_op(&mut e, 0, "_menuCheat", L14_SHOUSHA_SKILL_ID);
    assert_eq!(flag(&e, 0, "l14_shousha_trigger_count"), 2);
    assert_eq!(flag(&e, 0, "l14_skill:_dieNoTuoguan"), 1);
    assert_eq!(flag(&e, 0, "l14_skill:_menuCheat"), 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_l14,
        "L14 UI hook _dieNoTuoguan must not draw cards"
    );
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before_l14,
        "L14 UI hooks must not deal combat damage"
    );

    let hand_before_baozou = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "baozou", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(flag(&e, 0, "l19_baozou_active"), 1);
    assert_eq!(flag(&e, 0, "l19_baozou_sha_unlimited"), 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_baozou,
        "L19 baozou (`末日乱斗/extension.js:333-347`) is mod.cardUsable(sha)->Infinity and must not draw cards"
    );
    invoke_p9_op(&mut e, 0, "ganran", L19_DOOMSDAY_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ganran2", L19_DOOMSDAY_SKILL_ID);
    invoke_p9_op(&mut e, 0, "ganran", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(flag(&e, 0, "doomsday_mutated"), 1);

    // L20 `zhengsu` (`标记补充/extension.js:244`) violation negative guard + reward positive path
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
    assert_eq!(L25_ZUAN_CHARACTERS.len(), 60);
    assert_eq!(L25_ZUAN_SKILLS.len(), 122);
    assert_eq!(L27_SELECT_BEAUTIFY_SKILLS.len(), 43);

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

    // L24 skin skills + Spine transform resolver (`皮肤切换/extension.js:641`) — does NOT draw cards
    let hand_before_l24 = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "_tsx", L24_SKIN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l24_skin_slot"), 1);
    assert_eq!(flag(&e, 0, "l24_skin_anim_state"), 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_l24,
        "L24 skin hook _tsx must not draw cards"
    );
    let tf = resolve_l24_skin_transform("caocao", 2, "gongji").expect("valid skin transform");
    assert_eq!(tf.stand_x, 24);
    assert_eq!(tf.stand_y, -8);
    assert_eq!(tf.scale_pct, 108);
    assert!(resolve_l24_skin_transform("caocao", 9, "gongji").is_err());
    assert!(resolve_l24_skin_transform("caocao", 1, "unknown_clip").is_err());

    // L25 `g_du` (`祖安扩展/extension.js:64935`) + `_diycharacterskill` online room rejection guard
    let hp_before = e.state_core().player(0).hp;
    invoke_p9_op(&mut e, 0, "g_du", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_g_du_triggered"), 1);
    assert_eq!(e.state_core().player(0).hp, hp_before - 1);

    let hand_before_diy = e.state_core().player(0).hand.len();
    set_flag(&mut e, 0, "l25_online_room_active", 1);
    invoke_p9_op(&mut e, 0, "_diycharacterskill", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_diy_blocked_online"), 1);
    assert_eq!(flag(&e, 0, "l25_diy_applied"), 0);
    set_flag(&mut e, 0, "l25_online_room_active", 0);
    invoke_p9_op(&mut e, 0, "_diycharacterskill", L25_ZUAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "l25_diy_applied"), 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_diy,
        "L25 _diycharacterskill hook must not draw cards"
    );

    assert!(validate_and_apply_l25_diy(&L25DiyEditorDraft {
        general_id: "ssr_luxun".to_string(),
        faction: "wu".to_string(),
        max_hp: 3,
        skills: vec!["_diycharacterskill".to_string()],
        online_room_active: true,
    })
    .is_err());
    assert!(validate_and_apply_l25_diy(&L25DiyEditorDraft {
        general_id: "ssr_luxun".to_string(),
        faction: "wu".to_string(),
        max_hp: 3,
        skills: vec!["_diycharacterskill".to_string()],
        online_room_active: false,
    })
    .is_ok());

    // L27 `_chooseCharacterTwo` (`选将美化/extension.js:1165`) dual-general pick count guard + `qimou` (`extension.js:5902`) vs `reqimou` (`extension.js:6027`) + `baonvezhi` (`extension.js:184`)
    set_flag(&mut e, 0, "l27_pick_count", 1);
    invoke_p9_op(&mut e, 0, "_chooseCharacterTwo", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_dual_pick_invalid"), 1);
    assert_eq!(flag(&e, 0, "l27_dual_pick_locked"), 0);
    set_flag(&mut e, 0, "l27_pick_count", 2);
    invoke_p9_op(&mut e, 0, "_chooseCharacterTwo", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_dual_pick_locked"), 1);
    assert_eq!(flag(&e, 0, "l27_combined_hp"), 4);

    let hand_before_baonue = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "baonvezhi", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_baonvezhi_resolved"), 1);
    assert_eq!(flag(&e, 0, "l27_baonue_marks"), 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_baonue,
        "baonvezhi gains baonue mark without drawing cards"
    );

    let hand_before_qimou = e.state_core().player(0).hand.len();
    let hp_before_qimou = e.state_core().player(0).hp;
    invoke_p9_op(&mut e, 0, "qimou", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_qimou_used"), 1);
    assert_eq!(flag(&e, 0, "l27_qimou_range_bonus"), 1);
    assert_eq!(e.state_core().player(0).hp, hp_before_qimou - 1);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_qimou,
        "upstream qimou (`选将美化/extension.js:5902-6026`) loses X HP for distance/sha bonus without drawing cards"
    );
    invoke_p9_op(&mut e, 0, "qimou", L27_SELECT_SKILL_ID);
    assert_eq!(flag(&e, 0, "l27_qimou_blocked"), 1);

    let hand_before_reqimou = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "reqimou", L27_SELECT_SKILL_ID);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_reqimou + 1,
        "upstream reqimou (`选将美化/extension.js:6027-6090`) draws X cards"
    );

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

#[test]
fn formal_packages_extensions_load_and_execute_on_locked_product_path() {
    use content_native_loader::{
        prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
        validate_and_freeze_raw_package, LockedStartMode,
    };
    use std::collections::BTreeMap;

    let ext_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let formal_dirs = [
        "d3-upstream-noname",
        "thunder-upstream-liuli",
        "longxue-upstream-liuli",
        "epic-cards-upstream-liuli",
        "doomsday-brawl-upstream-liuli",
        "mark-supplement-upstream-liuli",
        "zuan-upstream-liuli",
        "select-beautify-upstream-liuli",
        "wudian-upstream-noname",
        "hero-kill-upstream-noname",
        "huanle-upstream-noname",
        "ai-tuning-upstream-liuli",
        "wuxing-upstream-liuli",
    ];

    let mut frozen_list = Vec::new();
    for dir_name in formal_dirs {
        let pkg_dir = ext_root.join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap_or_else(|e| panic!("read {dir_name}/package.npack.json: {e}"));
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap_or_else(|e| panic!("read {dir_name}/{rel_path}: {e}"));
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets)
            .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"));
        frozen_list.push(frozen);
    }

    let verified =
        prepare_locked_content_with_bridge(frozen_list).expect("prepare formal locked packages");
    let mut e = start_locked_engine(260199, 2, LockedStartMode::Prototype, &verified)
        .expect("start locked engine with formal packages");

    // Verify authentic upstream generals from formal packages are in the locked Catalog
    for expected_general in [
        "ddd_zhangmiao",
        "ddd_caomao",
        "th_guanning",
        "th_majun",
        "th_shen_zhouyu",
        "shichangshi",
        "jiangshifz",
        "jiangshinj",
        "ssr_luxun",
        "ssr_sunquan",
        "费诗",
        "韩龙",
        "留赞",
        "yxs_qinqiong",
        "yxs_wuzetian",
        "yxs_nandinggeer",
        "mb_caomao",
        "mb_caomao2",
    ] {
        assert!(
            (0..e.catalog().general_count() as u16)
                .any(|idx| e.catalog().general_meta(idx).map(|m| m.0) == Some(expected_general)),
            "formal locked Catalog must include upstream general {expected_general}"
        );
    }

    // Verify authentic upstream `dddxiaxing` executes via interactive choice on the locked engine
    // and survives `restore_locked_engine` mid-choice (`3D精选/character/skill.js:59-147`):
    let zhangmiao_locked = find_general(&e, "ddd_zhangmiao");
    e.assign_general(0, zhangmiao_locked).expect("assign ddd_zhangmiao on locked engine");
    let pick_card = e.state_core().player(0).hand[0];
    e.debug_set_turn(1, Phase::End, 1);
    let out_locked = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let locked_choice = match out_locked {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxiaxing_card on locked engine, got {other:?}"),
    };
    assert_eq!(locked_choice.spec.prompt, "dddxiaxing_card");

    let snap_pending = e.save();
    let mut restored = restore_locked_engine(&snap_pending, &verified).expect("restore locked engine mid-choice");
    assert_eq!(restored.state_hash(), e.state_hash());

    let ans = GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: locked_choice.id,
            selection: vec![engine_native::ChoiceOption::Card(pick_card)],
        },
    };
    let _ = e.submit(ans.clone());
    let _ = restored.submit(ans);
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 1);
    assert_eq!(restored.state_hash(), e.state_hash());

    // Verify extended upstream resolvers in `decision34_ext.rs` and `decision34_u08_l19_l20_l27.rs`:
    // 1. U01 remaining skills (`dddxianxi` `skill.js:1798`, `dddjuxian` `skill.js:275`, `dddfenji` `skill.js:3866`, `dddduanbing` `skill.js:5131`)
    invoke_p9_op(&mut e, 0, "dddxianxi", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_xianxi_target_added"), 1);
    invoke_p9_op(&mut e, 0, "dddjuxian", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_juxian_virtue"), 1);
    let hp_before_fenji = e.state_core().player(0).hp;
    invoke_p9_op(&mut e, 0, "dddfenji", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_fenji_burst"), 1);
    assert_eq!(e.state_core().player(0).hp, hp_before_fenji - 1);
    let hand_before_duanbing = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "dddduanbing", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "ddd_dddduanbing_range_mod"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_duanbing);

    // 2. L28 remaining `th_*` skills (`th_caiyi` `extension.js:4054`, `th_chongxu` `extension.js:4354`, `th_tiansuan` `extension.js:7633`, `th_geyuan` `extension.js:8141`,
    //    `th_bushi` `extension.js:3783`, `th_guili` `extension.js:4289`, `th_shengong` `extension.js:4812`, `th_pingcai` `extension.js:5181`,
    //    `th_zhengjing` `extension.js:5953`, `th_chengye` `extension.js:6155`, `th_buxu` `extension.js:6358`, `th_yufeng` `extension.js:6591`,
    //    `th_tianshu` `extension.js:6785`, `th_jieshu` `extension.js:8394`, `th_gusuan` `extension.js:8443`)
    invoke_p9_op(&mut e, 0, "th_caiyi", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_caiyi_yang"), 1);
    invoke_p9_op(&mut e, 0, "th_chongxu", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_chongxu_marks"), 2);
    invoke_p9_op(&mut e, 0, "th_tiansuan", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_tiansuan_lot"), 1);
    invoke_p9_op(&mut e, 0, "th_geyuan", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_geyuan_pi"), 1);
    invoke_p9_op(&mut e, 0, "th_bushi", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_bushi_suits_assigned"), 1);
    invoke_p9_op(&mut e, 0, "th_guili", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_guili_extra_turn"), 1);
    invoke_p9_op(&mut e, 0, "th_shengong", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_shengong_forged"), 1);
    invoke_p9_op(&mut e, 0, "th_pingcai", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_pingcai_relic_wiped"), 1);
    invoke_p9_op(&mut e, 0, "th_zhengjing", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_zhengjing_sorted"), 1);
    invoke_p9_op(&mut e, 0, "th_chengye", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_chengye_liujing_count"), 1);
    invoke_p9_op(&mut e, 0, "th_buxu", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_buxu_added_classic"), 1);
    invoke_p9_op(&mut e, 0, "th_yufeng", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_yufeng_flight_score"), 2);
    invoke_p9_op(&mut e, 0, "th_tianshu", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_tianshu_granted_yaoshu"), 1);
    invoke_p9_op(&mut e, 0, "th_jieshu", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_jieshu_drawn"), 1);
    invoke_p9_op(&mut e, 0, "th_gusuan", content_native_loader::TH_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "th_gusuan_awakened"), 1);

    // 3. L10 `k_danggu` non-conflicting pair (`史诗卡牌/extension.js:250-295`): eunuch 0 (`scstaoluan`) conflicts with eunuch 1 (`scschiyan`), so partner is eunuch 2 (`scszimou`)
    invoke_p9_op(&mut e, 0, "k_danggu", content_native_loader::K_DANGGU_ID);
    assert_eq!(flag(&e, 0, "l10_scs_skill:scstaoluan"), 1);
    assert_eq!(flag(&e, 0, "k_danggu_partner_eunuch"), 3);
    assert_eq!(flag(&e, 0, "l10_scs_skill:scszimou"), 1);

    // 4. L29 `mbcmfangzhu` baiban/jin options (`龙血玄黄/character/mobile.js:17343-17392`), `mbcmqingzheng` (`龙血玄黄/extension.js:180`), & `mbcmxscw` (`mobile.js:17410`)
    e.state_core_mut().player_mut(0).mbqianlong_marks = 80;
    set_flag(&mut e, 0, "mbcmfangzhu_option", 1);
    invoke_p9_op(&mut e, 0, "mbcmfangzhu", content_native_loader::MBCMFANGZHU_ID);
    assert_eq!(flag(&e, 1, "mbcmfangzhu_baiban"), 1);
    set_flag(&mut e, 0, "mbcmfangzhu_option", 2);
    invoke_p9_op(&mut e, 0, "mbcmfangzhu", content_native_loader::MBCMFANGZHU_ID);
    assert_eq!(flag(&e, 1, "mbcmfangzhu_jin"), 1);
    invoke_p9_op(&mut e, 0, "mbcmqingzheng", content_native_loader::L29_LONGXUE_SKILL_ID);
    assert_eq!(flag(&e, 0, "mbcmqingzheng_resolved"), 1);
    invoke_p9_op(&mut e, 0, "mbcmxscw", 0);
    assert_eq!(flag(&e, 0, "mbcmxscw_triggered"), 1);

    // 5. L01 `yiyi` (`AI优化/extension.js:9205-9260`), L06 `_shengke`/`wuxingpan_skill` (`五行生克/extension.js:18-167`),
    //    U06 `monkey`/`mianju`/`xuelunyang`/`jiuwei` (`欢乐卡牌/card/skill.js:4-255`),
    //    U07 `youdishenru`/`ciqiu_dying`/`kangyin2` (`玩点论杀/character/skill.js:5-720`),
    //    and U08 `U08_YXS_SKILLS` (`英雄杀/character/skill.js:5-3135`)
    invoke_p9_op(&mut e, 0, "yiyi", content_native_loader::L01_YIYI_ID);
    assert_eq!(flag(&e, 0, "l01_yiyi_resolved"), 1);
    set_flag(&mut e, 0, "wuxing_card_nature", 1);
    set_flag(&mut e, 0, "wuxing_player_nature", 3);
    invoke_p9_op(&mut e, 0, "_shengke", content_native_loader::L06_WUXING_ID);
    assert_eq!(flag(&e, 0, "wuxing_sheng_triggered"), 1);
    invoke_p9_op(&mut e, 0, "wuxingpan_skill", content_native_loader::L06_WUXING_ID);
    assert_eq!(flag(&e, 0, "wuxingpan_reassigned_nature"), 1);
    invoke_p9_op(&mut e, 0, "monkey", content_native_loader::U06_HUANLE_SKILL_ID);
    assert_eq!(flag(&e, 0, "monkey_stole_tao"), 1);
    invoke_p9_op(&mut e, 0, "mianju", content_native_loader::U06_HUANLE_SKILL_ID);
    assert_eq!(flag(&e, 0, "mianju_prevented_turnover"), 1);
    invoke_p9_op(&mut e, 0, "xuelunyang", content_native_loader::U06_HUANLE_SKILL_ID);
    assert_eq!(flag(&e, 0, "xuelunyang_copied_skill"), 1);
    invoke_p9_op(&mut e, 0, "jiuwei", content_native_loader::U06_HUANLE_SKILL_ID);
    assert_eq!(flag(&e, 0, "jiuwei_triggered"), 1);
    invoke_p9_op(&mut e, 0, "youdishenru", content_native_loader::U07_WUDIAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "youdishenru_counter_damage"), 1);
    invoke_p9_op(&mut e, 0, "ciqiu_dying", content_native_loader::U07_WUDIAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "ciqiu_dying_instant_kill"), 1);
    invoke_p9_op(&mut e, 0, "kangyin2", content_native_loader::U07_WUDIAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "kangyin2_bonus_applied"), 1);
    for &yxs_sk in &content_native_loader::U08_YXS_SKILLS {
        if e.state_core().player(1).hp < 3 {
            e.state_core_mut().player_mut(1).max_hp = 10;
            e.state_core_mut().player_mut(1).hp = 10;
            e.state_core_mut().player_mut(1).dying = false;
        }
        invoke_p9_op(&mut e, 0, yxs_sk, content_native_loader::U08_YXS_SKILL_ID);
        assert_eq!(flag(&e, 0, &format!("u08_skill:{yxs_sk}")), 1);
    }
    assert_eq!(flag(&e, 0, "yxs_fanji_counter"), 1);
    assert_eq!(flag(&e, 0, "yxs_yixin_resolved"), 1);
    assert_eq!(flag(&e, 0, "yxs_lshengong_copied_equip"), 1);

    // 6. U01 `dddjinggou` (`3D精选/character/skill.js:443-471`) & `dddmoyan` (`skill.js:472-509`, 0 draw on owner)
    invoke_p9_op(&mut e, 0, "dddjinggou", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "dddjinggou_weapon_snipe"), 1);
    let hand_before_moyan = e.state_core().player(0).hand.len();
    invoke_p9_op(&mut e, 0, "dddmoyan", DDD_SKILL_DISPATCH_ID);
    assert_eq!(flag(&e, 0, "dddmoyan_recorded_count"), 1);
    assert_eq!(e.state_core().player(0).hand.len(), hand_before_moyan);

    // 7. U06 `kuwu` (`欢乐卡牌/card/skill.js:41`) & U07 `g_youdishenru` (`玩点论杀/card/skill.js:16`) + deferred general skills on locked Catalog
    assert!(e.catalog().skill_by_name("kuwu").is_some());
    invoke_p9_op(&mut e, 0, "kuwu", content_native_loader::HUANLE_KUWU_INTERACTIVE_ID);
    assert!(flag(&e, 1, "kuwu_discarded_he") > 0 || flag(&e, 1, "kuwu_lost_hp") == 1);

    assert!(e.catalog().skill_by_name("g_youdishenru").is_some());
    invoke_p9_op(&mut e, 0, "g_youdishenru", content_native_loader::U07_WUDIAN_SKILL_ID);
    assert_eq!(flag(&e, 0, "youdishenru_nullified_sha"), 1);

    for (gen_name, expected_sk) in [
        ("刘馥", "duoqi"),
        ("田豫", "youzhan"),
        ("造之仁君", "jianbi"),
        ("杨仪", "choudu"),
    ] {
        let g_idx = find_general(&e, gen_name);
        let sk_id = e.catalog().skill_by_name(expected_sk).expect("U07 skill in Catalog");
        let mut probe = start_locked_engine(260200, 2, LockedStartMode::Prototype, &verified).unwrap();
        probe.debug_assemble_general_def(0, g_idx);
        assert!(
            probe.state_core().has_skill(0, sk_id),
            "locked general {gen_name} (`玩点论杀/character/character.js`) must carry executable skill {expected_sk}"
        );
    }

    // 8. L10 `k_danggu` (`史诗卡牌/extension.js:249-310`): grants real executable Ten Eunuchs `SkillInstance`s
    let scs_taoluan_id = e.catalog().skill_by_name("scstaoluan").expect("scstaoluan in Catalog");
    let scs_zimou_id = e.catalog().skill_by_name("scszimou").expect("scszimou in Catalog");
    assert!(e.state_core().has_skill(0, scs_taoluan_id));
    assert!(e.state_core().has_skill(0, scs_zimou_id));
    let mut l10_probe = start_locked_engine(260204, 2, LockedStartMode::Prototype, &verified).unwrap();
    let hp1_before_scs = l10_probe.state_core().player(1).hp;
    invoke_p9_op(&mut l10_probe, 0, "scsniqu", content_native_loader::L10_SCS_SKILL_ID);
    assert_eq!(flag(&l10_probe, 0, "l10_scs_resolved:scsniqu"), 1);
    assert_eq!(l10_probe.state_core().player(1).hp, hp1_before_scs - 1);

    // 9. L19 (`末日乱斗/extension.js:367-3384`): authentic zombie max_hp (`jiangshifz`=5, `jiangshinj`=3) + `zaibian`/`_tuizhi`/`_jiangshi`/`_huzhu`/`_guixinmodeskill`
    let fz_idx = find_general(&e, "jiangshifz");
    let nj_idx = find_general(&e, "jiangshinj");
    let mut l19_probe = start_locked_engine(260201, 2, LockedStartMode::Prototype, &verified).unwrap();
    l19_probe.debug_assemble_general_def(0, fz_idx);
    l19_probe.debug_assemble_general_def(1, nj_idx);
    assert_eq!(l19_probe.state_core().player(0).max_hp, 5);
    assert_eq!(l19_probe.state_core().player(1).max_hp, 3);

    set_flag(&mut l19_probe, 0, "l19_human_count", 3);
    set_flag(&mut l19_probe, 0, "l19_zombie_count", 1);
    let hand_before_zaibian = l19_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l19_probe, 0, "zaibian", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(l19_probe.state_core().player(0).hand.len(), hand_before_zaibian + 3);

    set_flag(&mut l19_probe, 0, "l19_tuizhi_turns", 7);
    invoke_p9_op(&mut l19_probe, 0, "_tuizhi", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(flag(&l19_probe, 0, "l19_tuizhi_victory"), 1);

    l19_probe.state_core_mut().player_mut(1).hp = 1;
    invoke_p9_op(&mut l19_probe, 0, "_huzhu", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(flag(&l19_probe, 0, "l19_huzhu_healed"), 1);
    assert_eq!(l19_probe.state_core().player(1).hp, 2);

    invoke_p9_op(&mut l19_probe, 1, "_jiangshi", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(l19_probe.state_core().player(1).max_hp, 5);
    assert_eq!(l19_probe.state_core().player(1).hp, 5);
    assert_eq!(l19_probe.state_core().player(1).hand.len(), 4);

    let hujia_before = l19_probe.state_core().player(0).hujia;
    invoke_p9_op(&mut l19_probe, 0, "_guixinmodeskill", L19_DOOMSDAY_SKILL_ID);
    assert_eq!(l19_probe.state_core().player(0).hujia, hujia_before + 2);

    // 10. L20 (`标记补充/extension.js:2261-6240`): `kurou` (-1 HP +2 draw), `qingyu` (<2 guard + >=2 discard 2 & prevent damage), `mibei` (+3 draw), and 0-draw passive/deck-swap skills
    let mut l20_probe = start_locked_engine(260202, 2, LockedStartMode::Prototype, &verified).unwrap();
    let hp_before_kurou = l20_probe.state_core().player(0).hp;
    let hand_before_kurou = l20_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l20_probe, 0, "kurou", L20_SKILL_DISPATCH_ID);
    assert_eq!(l20_probe.state_core().player(0).hp, hp_before_kurou - 1);
    assert_eq!(l20_probe.state_core().player(0).hand.len(), hand_before_kurou + 2);

    let hand_before_mibei = l20_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l20_probe, 0, "mibei", L20_SKILL_DISPATCH_ID);
    assert_eq!(l20_probe.state_core().player(0).hand.len(), hand_before_mibei + 3);
    assert_eq!(flag(&l20_probe, 0, "l20_granted_xinmouli"), 1);

    let hand_before_qingyu = l20_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l20_probe, 0, "qingyu", L20_SKILL_DISPATCH_ID);
    assert_eq!(flag(&l20_probe, 0, "l20_qingyu_prevented"), 1);
    assert_eq!(l20_probe.state_core().player(0).hand.len(), hand_before_qingyu - 2);

    for zero_draw_sk in ["fz_new_rewusheng", "fz_liegong", "wengua5", "zuoxing", "_chongzhu"] {
        let h0 = l20_probe.state_core().player(0).hand.len();
        invoke_p9_op(&mut l20_probe, 0, zero_draw_sk, L20_SKILL_DISPATCH_ID);
        assert_eq!(
            l20_probe.state_core().player(0).hand.len(),
            h0,
            "L20 skill {zero_draw_sk} (`标记补充/extension.js`) must not increase hand size via draw(1)"
        );
    }
    let max_hp_before_fanbei = l20_probe.state_core().player(0).max_hp;
    invoke_p9_op(&mut l20_probe, 0, "_xigua_xueliangfanbei_", L20_SKILL_DISPATCH_ID);
    assert_eq!(l20_probe.state_core().player(0).max_hp, max_hp_before_fanbei * 2);

    // 11. L27 (`选将美化/extension.js:1959-8930`): `twharvestinori` (draw 2*X) vs `twzhouhu` (heal X), `twxiafeng` (0 draw), `bolan` (0 draw), and 6 newly registered skills
    let mut l27_probe = start_locked_engine(260203, 2, LockedStartMode::Prototype, &verified).unwrap();
    for new_l27_sk in ["huamu", "jishe2", "mbcmqingzheng", "mbwuling", "potzhuangshi", "sbqingzheng"] {
        assert!(l27_probe.catalog().skill_by_name(new_l27_sk).is_some());
    }
    set_flag(&mut l27_probe, 0, "l27_mahou_x", 2);
    let h_inori = l27_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l27_probe, 0, "twharvestinori", L27_SELECT_SKILL_ID);
    assert_eq!(l27_probe.state_core().player(0).hand.len(), h_inori - 1 + 4);

    l27_probe.state_core_mut().player_mut(0).hp = 1;
    set_flag(&mut l27_probe, 0, "l27_mahou_x", 2);
    let h_zhouhu = l27_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l27_probe, 0, "twzhouhu", L27_SELECT_SKILL_ID);
    assert_eq!(l27_probe.state_core().player(0).hand.len(), h_zhouhu - 1);
    assert_eq!(l27_probe.state_core().player(0).hp, 3);

    set_flag(&mut l27_probe, 0, "l27_baonue_marks", 2);
    let h_xiafeng = l27_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l27_probe, 0, "twxiafeng", L27_SELECT_SKILL_ID);
    assert_eq!(l27_probe.state_core().player(0).hand.len(), h_xiafeng);
    assert_eq!(flag(&l27_probe, 0, "l27_twxiafeng_spent"), 2);

    let h_bolan = l27_probe.state_core().player(0).hand.len();
    invoke_p9_op(&mut l27_probe, 0, "bolan", L27_SELECT_SKILL_ID);
    assert_eq!(l27_probe.state_core().player(0).hand.len(), h_bolan);
    assert_eq!(flag(&l27_probe, 0, "l27_bolan_temp_skill"), 1);

    // 12. L28 (`雷霆万钧/extension.js:6003-8381`): non-skill keys & fabricated names absent; fortune-lot & `th_gushe2` subskills execute without spurious draw(1)
    for invalid_l28_key in [
        "th_caiyi_info",
        "th_qiaosi_map",
        "th_tiansuan_faq",
        "th_geyuan_info",
        "th_taipingyaoshu_info",
        "th_caiyi_yang",
        "th_caiyi_yin",
        "th_pingcai_Wolong",
        "th_jingxie1",
    ] {
        assert!(
            e.catalog().skill_by_name(invalid_l28_key).is_none(),
            "non-skill or fabricated key {invalid_l28_key} must not be registered in Catalog"
        );
    }
    for (lot_sk, expected_flag) in [
        ("th_tiansuan2_0", "th_tiansuan2_0_prevent_damage"),
        ("th_tiansuan2_1", "th_tiansuan2_1_cap_damage"),
        ("th_tiansuan2_2", "th_tiansuan2_2_fire"),
        ("th_tiansuan2_3", "th_tiansuan2_3_extra_damage"),
        ("th_tiansuan2_4", "th_tiansuan2_4_forbid_tao_jiu"),
        ("th_gushe2", "th_gushe2_marks"),
    ] {
        assert!(e.catalog().skill_by_name(lot_sk).is_some());
        let h_before = e.state_core().player(0).hand.len();
        invoke_p9_op(&mut e, 0, lot_sk, content_native_loader::TH_SKILL_DISPATCH_ID);
        assert_eq!(flag(&e, 0, expected_flag), 1);
        assert_eq!(
            e.state_core().player(0).hand.len(),
            h_before,
            "L28 subskill {lot_sk} (`雷霆万钧/extension.js:8035-8381`) must not draw cards"
        );
    }

    let final_snap = e.save();
    let final_restored = restore_locked_engine(&final_snap, &verified).expect("final locked restore");
    assert_eq!(final_restored.state_hash(), e.state_hash());
}



