//! L19 末日乱斗（`doomsday-brawl-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置与包数据：`assets/cards.json` 桥接 3 项技能（`shishi`, `xunmeng`, `zaibian`）+ `doomsday` 模式辅助函数；
//! ③ 组合共存：与 `mark-supplement-upstream-liuli`、`select-beautify-upstream-liuli` 同局共存；
//! ④ 保存恢复：对局中含 `zaibian`/`shishi` 技能状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：灾变（`zaibian`）出牌阶段开始摸牌、无尽模式人类位加成计算（`WujinMoshiConfig`）与身份转化链（`identity_chain`）；
//! ⑦ 异常拒装：损坏 JSON 与未知宿主行为键返回明确 `BridgeError`；
//! ⑧ 边界登记：`wujinmoshi`、`brawl_mode` 与 `skill_batch_rest` 在 `cards.json` 显式登记暂缓原因。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    bridge_pack, decision34_pack9_extension_pack, identity_chain, parse_zuan_brawl,
    validate_zuan_brawl, zaibian_draw, BridgeError, WujinMoshiConfig, ZuanBrawlMode,
    DOOMSDAY_ZOMBIE_GENERALS, IDENTITY_NAMES, JIANGSHIFZ_SKILLS, JIANGSHINJ_SKILLS,
};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
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

#[test]
fn l19_doomsday_brawl_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "doomsday-brawl-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "doomsday-brawl" && m.namespace == "upstream-liuli")
        .expect("doomsday-brawl 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cards_json =
        std::fs::read_to_string(ext_root.join("assets/cards.json")).expect("读取 cards.json");
    let config_json =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");
    assert!(config_json.contains("\"shishi\""));

    // ② 包数据桥接与模式辅助校验
    let out = bridge_pack(&cards_json).expect("bridge doomsday-brawl");
    assert_eq!(out.pack.def.name, "doomsday-brawl-upstream-liuli");
    assert_eq!(out.pack.host_skills.len(), 3);
    assert_eq!(
        out.deferred,
        vec![
            "wujinmoshi".to_string(),
            "brawl_mode".to_string(),
            "skill_batch_rest".to_string()
        ]
    );

    // doomsday 模式纯函数校验
    assert_eq!(IDENTITY_NAMES.len(), 15);
    assert_eq!(identity_chain(4), vec!["er", "san"]);
    assert_eq!(zaibian_draw(2, 1, 0), 3);
    assert_eq!(zaibian_draw(0, 3, 1), 0);
    let wj = WujinMoshiConfig {
        draw: 2,
        damage: 1,
        damage_taken: 1,
        recover: 1,
        max_hand_card: 3,
        damage_divisor: 30,
        human_seats: vec![0],
    };
    assert_eq!(wj.draw_bonus(0), 2);
    assert_eq!(wj.draw_bonus(1), 0);

    // ③ 与标记补充包共存建局
    let ms_json = std::fs::read_to_string(
        root.join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 mark-supplement cards.json");
    let ms_pack = bridge_pack(&ms_json).expect("bridge ms").pack;
    let packs = [&out.pack, &ms_pack];
    let mut e = Engine::new_with_extensions(191901, 4, &packs).expect("建局");

    // ⑤ 授予 zaibian 并验证出牌阶段开始额外摸牌
    e.debug_grant(0, "zaibian");
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    let hand_after_draw = e.debug_hand(0).len();
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase); // Draw -> Play (zaibian 触发)
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_after_draw + 2, "灾变额外摸 2 张");

    // ④ 存档与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 坏数据拒绝
    let bad = cards_json.replace("host:zaibian", "host:unknown_doomsday_skill");
    assert!(matches!(
        bridge_pack(&bad),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));
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

#[test]
fn l19_doomsday_batch_a_modes_1_to_3_generals_config_and_save_restore() {
    // Verify 2 zombie generals (`jiangshifz`, `jiangshinj`) & 1 config (`zuan_brawl`)
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[0].id, "jiangshifz");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[0].gender, "male");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[0].group, "ye");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[0].max_hp, 5);
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[0].skills, &JIANGSHIFZ_SKILLS);
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[1].id, "jiangshinj");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[1].gender, "female");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[1].group, "ye");
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[1].max_hp, 3);
    assert_eq!(DOOMSDAY_ZOMBIE_GENERALS[1].skills, &JIANGSHINJ_SKILLS);

    for mode_str in ["off", "standard", "random", "all", "daily"] {
        assert!(validate_zuan_brawl(mode_str));
        assert_eq!(parse_zuan_brawl(Some(mode_str)).unwrap().as_str(), mode_str);
    }
    assert_eq!(parse_zuan_brawl(None), Ok(ZuanBrawlMode::All));
    assert!(!validate_zuan_brawl("invalid_mode"));
    assert!(parse_zuan_brawl(Some("invalid_mode")).is_err());

    let p9 = decision34_pack9_extension_pack();
    let packs = [&p9];
    let mut e = Engine::new_with_extensions(191902, 4, &packs).expect("L19 Batch A engine");

    // Mode 1: baozou (unlimited sha) + ganran (equip as Tiesuo) + ganran2 ("ganran2:choose_equip")
    e.debug_grant(0, "baozou");
    e.debug_grant(0, "ganran");
    e.debug_grant(0, "ganran2");
    e.debug_grant(0, "_jisuangailv");
    let equip_cid = e.debug_place(0, "zhuge");
    e.debug_give_hand_card(0, 0, equip_cid);
    e.debug_set_turn(0, Phase::Play, 1);

    let ganran2_id = e.catalog().skill_by_name("ganran2").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: ganran2_id });
    let choice = match out {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected ganran2:choose_equip pending, got {other:?}"),
    };
    assert_eq!(choice.spec.prompt, "ganran2:choose_equip");
    let snap_mid = e.save();
    let restored_mid =
        Engine::restore_with_extensions(&snap_mid, &packs).expect("restore mid ganran2");
    assert_eq!(restored_mid.state_hash(), e.state_hash());
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Card(equip_cid)],
        },
    );
    assert_eq!(flag(&e, 0, "ganran2_recast"), 1);
    assert_eq!(flag(&e, 0, "ganran_active"), 1);

    // Mode 1: _huzhu ("_huzhu:discard_tao" -> "_huzhu:choose_target")
    e.debug_grant(0, "_huzhu");
    let tao_cid = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, tao_cid);
    e.state_core_mut().player_mut(1).hp = 2;
    let huzhu_id = e.catalog().skill_by_name("_huzhu").unwrap();
    let out_hz1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: huzhu_id });
    let c_hz1 = match out_hz1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _huzhu:discard_tao, got {other:?}"),
    };
    assert_eq!(c_hz1.spec.prompt, "_huzhu:discard_tao");
    let out_hz2 = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_hz1.id,
            selection: vec![ChoiceOption::Card(tao_cid)],
        },
    );
    let c_hz2 = match out_hz2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _huzhu:choose_target, got {other:?}"),
    };
    assert_eq!(c_hz2.spec.prompt, "_huzhu:choose_target");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_hz2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(flag(&e, 0, "_huzhu_healed"), 1);
    assert_eq!(e.state_core().player(1).hp, 3);

    // Mode 1: _tuizhi + _tuizhi2 + _jisuangailv
    e.debug_grant(0, "_tuizhi");
    e.debug_grant(0, "_tuizhi2");
    set_flag(&mut e, 0, "_tuizhi", 7);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "_tuizhi"), 8);
    assert_eq!(flag(&e, 0, "_tuizhi2_human_win"), 1);
    assert_eq!(flag(&e, 0, "_jisuangailv_init"), 1);

    // Mode 2 & Mode 3: _gainSkill + _wujinmoshiRE + wujinmoshiRE1 / wujinmoshiRE2 via combat
    e.debug_set_turn(0, Phase::Play, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "_gainSkill");
    e.debug_grant(1, "_wujinmoshiRE");
    e.debug_grant(1, "wujinmoshiRE1");
    e.debug_set_hp(1, 1);
    let sha1 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 1, "_wujinmoshiRE_used"), 1);
    assert_eq!(
        e.state_core().player(1).hp,
        e.state_core().player(1).max_hp as i32
    );

    e.debug_grant(3, "英姿");
    set_flag(&mut e, 0, "drlt_duorui_sealed_slots_mask", 0b111);
    e.debug_set_hp(3, 1);
    let sha2 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![3],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "_gainSkill_count"), 1);
    assert_eq!(flag(&e, 0, "drlt_duorui_sealed_slots_mask"), 0);
}

#[test]
fn l19_doomsday_batch_b_modes_4_to_7_rpg_yuanqi_jingcheng_zhengba() {
    let p9 = decision34_pack9_extension_pack();
    let packs = [&p9];
    let mut e = Engine::new_with_extensions(191903, 4, &packs).expect("L19 Batch B engine 1");

    // 1. Shared RPG progression: _levelskill, _expskill, _exp2skill, _useskill, _killskill, _relifeskill
    e.debug_grant(0, "_levelskill");
    e.debug_grant(0, "_expskill");
    e.debug_grant(0, "_exp2skill");
    e.debug_grant(0, "_useskill");
    e.debug_grant(0, "_killskill");
    e.debug_grant(1, "_relifeskill");
    set_flag(&mut e, 1, "zuan_relife", 1);

    e.debug_set_turn(0, Phase::Play, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    set_flag(&mut e, 0, "zuan_exp", 75);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain(&mut e);
    // CardUsed (+5 exp, +5 score) + Damage 1 (+20 exp, +10 score) -> 100 exp -> Lv2 (+1 maxHp, +1 recover, draw 2, grant skill)
    assert_eq!(flag(&e, 0, "zuan_level"), 2);
    assert_eq!(flag(&e, 0, "zuan_score"), 15);
    assert_eq!(flag(&e, 0, "_levelskill_granted"), 1);
    // Seat 1 triggered _relifeskill on Dying: zuan_relife -> 0, hp -> 4, hand -> 4
    assert_eq!(flag(&e, 1, "zuan_relife"), 0);
    assert_eq!(flag(&e, 1, "_relifeskill_revived"), 1);
    assert_eq!(e.state_core().player(1).hp, 4);

    // _exp2skill at Phase::End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End (_exp2skill: +10 exp, +5 score)
    drain(&mut e);
    assert_eq!(flag(&e, 0, "zuan_exp"), 10);
    assert_eq!(flag(&e, 0, "zuan_score"), 20);

    // 2. _yuanqiskill: Mode 0 (spend 100 exp -> choose skill) & Mode 1 (remove skill -> draw 2 + recover 1)
    e.debug_set_turn(0, Phase::Play, 2);
    e.debug_grant(0, "_yuanqiskill");
    set_flag(&mut e, 0, "zuan_exp", 100);
    let yq_id = e.catalog().skill_by_name("_yuanqiskill").unwrap();
    let out_yq1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yq_id });
    let c_yq1 = match out_yq1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _yuanqiskill:choose_mode, got {other:?}"),
    };
    assert_eq!(c_yq1.spec.prompt, "_yuanqiskill:choose_mode");
    let snap_yq = e.save();
    let restored_yq =
        Engine::restore_with_extensions(&snap_yq, &packs).expect("restore mid _yuanqiskill");
    assert_eq!(restored_yq.state_hash(), e.state_hash());

    let out_yq2 = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yq1.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let c_yq2 = match out_yq2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _yuanqiskill:choose_skill, got {other:?}"),
    };
    assert_eq!(c_yq2.spec.prompt, "_yuanqiskill:choose_skill");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yq2.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(flag(&e, 0, "_yuanqiskill_gained"), 1);
    assert_eq!(flag(&e, 0, "zuan_exp"), 0);

    // Mode 1 on next turn
    e.debug_grant(0, "_yuanqiskill");
    e.debug_set_turn(0, Phase::Play, 3);
    e.debug_set_hp(0, 2);
    let out_yq3 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yq_id });
    let c_yq3 = match out_yq3 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _yuanqiskill:choose_mode for mode 1, got {other:?}"),
    };
    let out_yq4 = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yq3.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    let c_yq4 = match out_yq4 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _yuanqiskill:remove_skill, got {other:?}"),
    };
    assert_eq!(c_yq4.spec.prompt, "_yuanqiskill:remove_skill");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yq4.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(flag(&e, 0, "_yuanqiskill_removed"), 1);

    // 3. zuan_jingcheng & 4. zuan_zhengba
    let mut e2 = Engine::new_with_extensions(191904, 4, &packs).expect("L19 Batch B engine 2");
    e2.debug_grant(0, "_zhudieskill");
    e2.debug_grant(1, "_zhongzhubuff");
    e2.debug_grant(1, "baozou");
    e2.debug_grant(2, "_tamodeskill");
    e2.debug_grant(3, "_qunxiongzhengbaskill");
    e2.state_core_mut().player_mut(0).max_hp = 5;
    e2.state_core_mut().player_mut(0).hp = 5;
    set_flag(&mut e2, 0, "is_zhu", 1);

    // _zhongzhubuff saves seat 1 using seat 0's max_hp/hp
    e2.debug_set_turn(0, Phase::Play, 1);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_set_hp(1, 1);
    let sha_jc = e2.debug_place(0, "sha");
    e2.debug_give_hand_card(0, 0, sha_jc);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha_jc,
            targets: vec![1],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 1, "_zhongzhubuff_triggered"), 1);
    assert_eq!(e2.state_core().player(0).max_hp, 4);
    assert_eq!(e2.state_core().player(1).hp, 2);

    // _zhudieskill saves seat 0 using tower nei (seat 2)
    e2.debug_set_turn(1, Phase::Play, 1);
    e2.debug_set_hp(0, 1);
    let sha_zd = e2.debug_place(1, "sha");
    e2.debug_give_hand_card(1, 1, sha_zd);
    let _ = submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: sha_zd,
            targets: vec![0],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 0, "_zhudieskill_saved"), 1);
    assert!(e2.state_core().player(0).hp >= 1);

    // _qunxiongzhengbaskill respawns seat 3 (seat 0 is distance 1 from seat 3)
    e2.debug_set_turn(0, Phase::Play, 2);
    e2.state_core_mut().turn.sha_used_play_phase = false;
    e2.debug_set_hp(3, 1);
    let sha_zb = e2.debug_place(0, "sha");
    e2.debug_give_hand_card(0, 0, sha_zb);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha_zb,
            targets: vec![3],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 3, "_qunxiongzhengbaskill_respawned"), 1);
    assert_eq!(e2.state_core().player(3).hp, 4);
}

#[test]
fn l19_doomsday_batch_b_modes_8_to_10_taofa_guixin_huanhai_ansha() {
    let p9 = decision34_pack9_extension_pack();
    let packs = [&p9];
    let mut e = Engine::new_with_extensions(191905, 4, &packs).expect("L19 Batch B engine 3");

    // 5. zuan_taofa: _guanmodeskill & _guandieskill (Phase 1 -> Phase 2 -> Phase 3 -> fan win)
    e.debug_grant(0, "_guanmodeskill");
    e.debug_grant(0, "_guandieskill");
    e.debug_grant(1, "baozou");
    e.debug_set_turn(1, Phase::Play, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for expected_phase in [2u32, 3u32] {
        e.debug_set_hp(0, 1);
        let s = e.debug_place(1, "sha");
        e.debug_give_hand_card(1, 1, s);
        let _ = submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: s,
                targets: vec![0],
            },
        );
        drain(&mut e);
        assert_eq!(flag(&e, 0, "guan_phase"), expected_phase);
    }
    e.debug_set_hp(0, 1);
    let s3 = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, s3);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: s3,
            targets: vec![0],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "taofa_fan_win"), 1);

    // 6. zuan_guixin (_guixinmodeskill) & 7. zuan_huanhai (_huanhaiskill + _seadreammodeskill) & 8. zuan_ansha (_dietime + _anshaskill)
    let mut e2 = Engine::new_with_extensions(191906, 4, &packs).expect("L19 Batch B engine 4");
    e2.debug_grant(0, "_guixinmodeskill");
    e2.debug_grant(1, "_seadreammodeskill");
    e2.debug_grant(2, "_anshaskill");
    e2.debug_grant(3, "_dietime");
    set_flag(&mut e2, 0, "guixin_camp", 1);
    set_flag(&mut e2, 2, "ansha_is_zhu", 1);
    set_flag(&mut e2, 2, "ansha_lives", 1);

    e2.debug_set_turn(0, Phase::Play, 1);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    // 0 hits 1 -> _guixinmodeskill converts 1 to camp 1 & _seadreammodeskill advances stage -> 2
    e2.debug_set_hp(1, 1);
    let sha1 = e2.debug_place(0, "sha");
    e2.debug_give_hand_card(0, 0, sha1);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 0, "_guixinmodeskill_converted"), 1);
    assert_eq!(flag(&e2, 1, "guixin_camp"), 1);

    // 1 hits 2 -> _anshaskill swaps zhu to 1 and respawns 2 as fan
    e2.debug_set_turn(1, Phase::Play, 1);
    e2.state_core_mut().turn.sha_used_play_phase = false;
    e2.debug_set_hp(2, 1);
    let sha2 = e2.debug_place(1, "sha");
    e2.debug_give_hand_card(1, 1, sha2);
    let _ = submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![2],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 2, "_anshaskill_zhu_swapped"), 1);
    assert_eq!(flag(&e2, 1, "ansha_is_zhu"), 1);

    // 2 hits 1 -> _seadreammodeskill advances stage -> 2
    e2.debug_set_turn(2, Phase::Play, 1);
    e2.state_core_mut().turn.sha_used_play_phase = false;
    e2.debug_set_hp(1, 1);
    let sha3 = e2.debug_place(2, "sha");
    e2.debug_give_hand_card(2, 2, sha3);
    let _ = submit(
        &mut e2,
        2,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    drain(&mut e2);
    assert_eq!(flag(&e2, 1, "_seadreammodeskill_stage"), 2);

    // _dietime expires at 5
    set_flag(&mut e2, 3, "dietime", 4);
    e2.debug_set_turn(3, Phase::Discard, 1);
    let _ = submit(&mut e2, 3, GameAction::NextPhase); // Discard -> End (_dietime -> 5 -> dies)
    drain(&mut e2);
    assert_eq!(flag(&e2, 3, "_dietime_expired"), 1);
    assert!(!e2.state_core().player(3).alive);
}


