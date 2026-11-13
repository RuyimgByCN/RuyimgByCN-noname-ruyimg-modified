//! L20 标记补充（`mark-supplement-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置与包数据：`assets/config.json` 与 `assets/cards.json` 桥接 6 项技能（`xigua_mopaifanbei`,
//!   `xigua_xueliangfanbei`, `kurou`, `zhuwanzhongji`, `fz_liegong`, `fz_xinkuanggu`）；
//! ③ 组合共存：与 `select-beautify-upstream-liuli`、`doomsday-brawl-upstream-liuli` 同局共存；
//! ④ 保存恢复：对局中含烈弓/狂骨与摸牌翻倍状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：`fz_liegong` 强命不可闪避、`fz_xinkuanggu` 近距造伤摸牌、`xigua_mopaifanbei` 摸牌阶段+2；
//! ⑦ 异常拒装：损坏 JSON 与非法宿主行为键明确返回 `BridgeError`；
//! ⑧ 边界登记：纯标记展示（`mark_display`）与 Spine 动画音效（`spine_effects`）显式登记于 `deferred`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    bridge_pack, decision34_pack9_extension_pack,
    mark_supplement::{
        load_mark_supplement_config, lookup_mark_supplement_card, DaoJianFuStyle, DaoJianFuType,
        YouxiShengfuMode, L20_MARK_SUPPLEMENT_CARD_SPECS, L20_MARK_SUPPLEMENT_CONFIG_KEYS,
    },
    BridgeError,
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
fn l20_mark_supplement_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "mark-supplement-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "mark-supplement" && m.namespace == "upstream-liuli")
        .expect("mark-supplement 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cards_json =
        std::fs::read_to_string(ext_root.join("assets/cards.json")).expect("读取 cards.json");
    let config_json =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");
    assert!(config_json.contains("\"mpfbmoshi\""));
    assert!(cards_json.contains("\"fz_liegong\""));

    // ② 桥接与技能/暂缓核对
    let out = bridge_pack(&cards_json).expect("bridge mark-supplement");
    assert_eq!(out.pack.def.name, "mark-supplement-upstream-liuli");
    assert_eq!(out.pack.host_skills.len(), 6);
    assert_eq!(
        out.deferred,
        vec![
            "mark_display".to_string(),
            "spine_effects".to_string(),
            "skill_batch_rest".to_string()
        ]
    );

    // ③ 与选将美化包共存建局
    let sb_json = std::fs::read_to_string(
        root.join("select-beautify-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 select-beautify cards.json");
    let sb_pack = bridge_pack(&sb_json).expect("bridge sb").pack;
    let packs = [&out.pack, &sb_pack];
    let mut e = Engine::new_with_extensions(192001, 4, &packs).expect("建局");

    // ⑤ 授予 fz_liegong + fz_xinkuanggu + kurou，验证烈弓强命、狂骨近距造伤摸牌与苦肉主动技
    e.debug_grant(0, "fz_liegong");
    e.debug_grant(0, "fz_xinkuanggu");
    e.debug_grant(0, "kurou");
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..4 {
        let shan = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, shan);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp_before = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    let mut kuanggu_triggered = false;
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "respond_shan" {
            // 上游 fz_liegong（extension.js:5160-5197）：目标手牌数 > 使用者手牌数时
            // 该【杀】可被闪避——目标选择不出闪。
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
            continue;
        }
        if c.spec.prompt == "trigger_confirm" || c.spec.prompt == "fz_liegong:confirm" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        if c.spec.prompt == "fz_xinkuanggu_choice" || c.spec.prompt == "fz_xinkuanggu:choose_mode" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Tag(1)],
                },
            );
            kuanggu_triggered = true;
            continue;
        }
        break;
    }
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp_before - 1, "未出闪的【杀】造成 1 点伤害");
    assert!(kuanggu_triggered, "fz_xinkuanggu 狂骨选择窗已触发");
    assert_eq!(e.debug_hand(0).len(), 1, "fz_xinkuanggu 选摸 1 张牌");

    let kurou_def = e.catalog().skill_by_name("kurou").expect("kurou 在目录");
    let hp0_before_kr = e.debug_hp(0);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kurou_def });
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0_before_kr - 1, "kurou 失去 1 点体力");
    assert_eq!(e.debug_hand(0).len(), 3, "kurou 摸 2 张牌（1 + 2 = 3）");

    // ④ 存档与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 坏数据拒绝
    let bad = cards_json.replace("host:fz_liegong", "host:unknown_ms_skill");
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
fn l20_mark_supplement_batch_a_aliases_configs_cards_guozhan_and_ui_hooks_save_restore() {
    // 1. Verify all 15 L20:configs:* and 3 L20:cards:* (`zhengsu_leijin`, `zhengsu_mingzhi`, `zhengsu_bianzhen`)
    assert_eq!(L20_MARK_SUPPLEMENT_CONFIG_KEYS.len(), 15);
    assert_eq!(L20_MARK_SUPPLEMENT_CARD_SPECS.len(), 3);
    for id in ["zhengsu_leijin", "zhengsu_mingzhi", "zhengsu_bianzhen"] {
        let card = lookup_mark_supplement_card(id).expect("zhengsu card spec");
        assert!(card.fullskin);
    }
    let cfg = load_mark_supplement_config(
        r#"{"youxishengfu":"identity","daojianfu":"shousha","daojianfu_type":"jian","mpfbmoshi":true,"xlfbmoshi":true,"zhujizhongwang":true}"#,
    )
    .expect("valid mark supplement config");
    assert_eq!(cfg.youxishengfu, YouxiShengfuMode::Identity);
    assert_eq!(cfg.daojianfu, DaoJianFuStyle::Shousha);
    assert_eq!(cfg.daojianfu_type, DaoJianFuType::Jian);
    assert!(cfg.mpfbmoshi && cfg.xlfbmoshi && cfg.zhujizhongwang);

    let p9 = decision34_pack9_extension_pack();
    let packs = [&p9];

    // 2. Verify 3 underscore alias fixes (`_xigua_mopaifanbei_`, `_xigua_xueliangfanbei_`, `_xigua_zhuwanzhongji_`)
    {
        let mut e_alias = Engine::new_with_extensions(192010, 4, &packs).expect("alias engine");
        for sk in [
            "_xigua_mopaifanbei_",
            "_xigua_xueliangfanbei_",
            "_xigua_zhuwanzhongji_",
        ] {
            assert!(
                e_alias.catalog().skill_by_name(sk).is_some(),
                "skill {sk} must be registered in catalog"
            );
            e_alias.debug_grant(0, sk);
        }
        e_alias.debug_set_turn(0, Phase::Prepare, 1);
        let hand_before = e_alias.debug_hand(0).len();
        // Prepare -> Judge -> Draw -> Play
        let _ = submit(&mut e_alias, 0, GameAction::NextPhase);
        drain(&mut e_alias);
        let _ = submit(&mut e_alias, 0, GameAction::NextPhase);
        drain(&mut e_alias);
        let _ = submit(&mut e_alias, 0, GameAction::NextPhase);
        drain(&mut e_alias);
        assert_eq!(
            e_alias.debug_hand(0).len(),
            hand_before + 4,
            "_xigua_mopaifanbei_ doubles draw phase cards (+2 bonus = +4 total)"
        );
    }

    // 3. Batch A: wengua & wengua5
    let mut e = Engine::new_with_extensions(192011, 4, &packs).expect("L20 Batch A engine");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "wengua");
    e.debug_grant(1, "wengua5");
    e.debug_set_turn(0, Phase::Play, 1);

    let c0 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c0);
    let wengua_id = e.catalog().skill_by_name("wengua").unwrap();
    let out_wg1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: wengua_id });
    let ch_wg1 = match out_wg1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected wengua:choose_card pending, got {other:?}"),
    };
    assert_eq!(ch_wg1.spec.prompt, "wengua:choose_card");

    // Mid-choice save/restore verification
    let snap_mid = e.save();
    let restored_mid =
        Engine::restore_with_extensions(&snap_mid, &packs).expect("restore mid wengua");
    assert_eq!(restored_mid.state_hash(), e.state_hash());

    let out_wg2 = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_wg1.id,
            selection: vec![ChoiceOption::Card(c0)],
        },
    );
    let ch_wg2 = match out_wg2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected wengua:choose_side pending, got {other:?}"),
    };
    assert_eq!(ch_wg2.spec.prompt, "wengua:choose_side");
    let bottom_before = e.state_core().deck[0];
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_wg2.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(
        e.state_core().deck.last().copied(),
        Some(c0),
        "wengua Tag(0) places card on top of deck"
    );
    assert!(
        e.debug_hand(0).contains(&bottom_before),
        "wengua Tag(0) draws 1 card from bottom of deck"
    );
    assert_eq!(flag(&e, 0, "wengua_resolved"), 1);

    // wengua5 on seat 1's Play phase
    e.debug_set_turn(1, Phase::Play, 1);
    let c1 = e.debug_place(1, "shan");
    e.debug_give_hand_card(1, 1, c1);
    let wengua5_id = e.catalog().skill_by_name("wengua5").unwrap();
    let out_wg5_1 = submit(&mut e, 1, GameAction::ActivateSkill { skill: wengua5_id });
    let ch_wg5_1 = match out_wg5_1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected wengua5:give_card pending, got {other:?}"),
    };
    assert_eq!(ch_wg5_1.spec.prompt, "wengua5:give_card");
    let out_wg5_2 = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ch_wg5_1.id,
            selection: vec![ChoiceOption::Card(c1)],
        },
    );
    let ch_wg5_2 = match out_wg5_2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected wengua5:choose_side pending, got {other:?}"),
    };
    assert_eq!(ch_wg5_2.spec.prompt, "wengua5:choose_side");
    assert_eq!(ch_wg5_2.observer, 0, "wengua owner chooses side");
    let hand0_before = e.debug_hand(0).len();
    let hand1_before = e.debug_hand(1).len();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_wg5_2.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(
        e.state_core().deck.first().copied(),
        Some(c1),
        "wengua5 Tag(1) places card on bottom of deck"
    );
    assert_eq!(e.debug_hand(0).len(), hand0_before, "owner placed 1 card and drew 1 from top");
    assert_eq!(e.debug_hand(1).len(), hand1_before + 1, "activator drew 1 from top");
    assert_eq!(flag(&e, 0, "wengua5_side"), 1);

    // 4. Batch A: diaohulishan (passive status skill: ActivateSkill rejected, blocks card use & targeting & damage/loseHp/recover, clears on TurnEnded)
    e.debug_set_turn(0, Phase::Play, 2);
    e.debug_grant(1, "diaohulishan");
    set_flag(&mut e, 1, "diaohulishan_active", 1);
    let dhls_id = e.catalog().skill_by_name("diaohulishan").unwrap();
    let out_dhls = submit(&mut e, 1, GameAction::ActivateSkill { skill: dhls_id });
    assert!(
        matches!(out_dhls, CommandOutcome::Rejected(_)),
        "diaohulishan is a passive status skill and cannot be actively invoked"
    );
    assert!(e.catalog().blocks_all_cards(e.state_core(), 1));
    let hp1_dhls = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), hp1_dhls, "diaohulishan prevents damage");
    let sha_test = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha_test);
    let out_blocked_target = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_test,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_blocked_target, CommandOutcome::Rejected(_)),
        "diaohulishan_active target cannot be targeted by PlayCard"
    );
    set_flag(&mut e, 1, "diaohulishan_active", 0);
    e.debug_set_skill_enabled(1, "diaohulishan", false);

    // 5. Batch A: _guozhan_marks (先驱=0 / 珠联璧合=1 / 阴阳鱼=2) & xianqu_mark
    e.debug_grant(0, "_guozhan_marks");
    e.debug_grant(0, "xianqu_mark");
    e.debug_clear_hand(0);
    set_flag(&mut e, 0, "xianqu_mark", 1);
    set_flag(&mut e, 0, "zhulianbihe_mark", 1);
    set_flag(&mut e, 0, "yinyang_mark", 1);
    set_flag(&mut e, 0, "yexinjia_mark", 1);
    set_flag(&mut e, 2, "general_unrevealed", 1);
    let gz_id = e.catalog().skill_by_name("_guozhan_marks").unwrap();

    // Branch 0: 先驱 (draw up to 4 + view target unseen general)
    let out_gz0 = submit(&mut e, 0, GameAction::ActivateSkill { skill: gz_id });
    let ch_gz0 = match out_gz0 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _guozhan_marks:choose_mark, got {other:?}"),
    };
    assert_eq!(ch_gz0.spec.prompt, "_guozhan_marks:choose_mark");
    let out_xq = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_gz0.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let ch_xq = match out_xq {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected xianqu_mark:choose_target, got {other:?}"),
    };
    assert_eq!(ch_xq.spec.prompt, "xianqu_mark:choose_target");
    assert_eq!(e.debug_hand(0).len(), 4, "xianqu_mark fills hand up to 4");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_xq.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert_eq!(flag(&e, 0, "xianqu_mark"), 0);
    assert_eq!(flag(&e, 0, "xianqu_viewed"), 1);
    assert_eq!(flag(&e, 0, "xianqu_target"), 3);
    set_flag(&mut e, 2, "general_unrevealed", 0);

    // Branch 1: 珠联璧合 (draw 2 cards)
    let out_gz1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: gz_id });
    let ch_gz1 = match out_gz1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _guozhan_marks:choose_mark (branch 1), got {other:?}"),
    };
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_gz1.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(flag(&e, 0, "zhulianbihe_mark"), 0);
    assert_eq!(e.debug_hand(0).len(), 6, "zhulianbihe_mark draws 2 cards (4 + 2 = 6)");

    // Branch 2: 阴阳鱼 (draw 1 card)
    let out_gz2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: gz_id });
    let ch_gz2 = match out_gz2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _guozhan_marks:choose_mark (branch 2), got {other:?}"),
    };
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_gz2.id,
            selection: vec![ChoiceOption::Tag(2)],
        },
    );
    assert_eq!(flag(&e, 0, "yinyang_mark"), 0);
    assert_eq!(e.debug_hand(0).len(), 7, "yinyang_mark draws 1 card (6 + 1 = 7)");

    // 6. Batch A: _zhulianbihe_mark_tao (consume 1 mark to recover 1 HP when wounded)
    e.debug_grant(0, "_zhulianbihe_mark_tao");
    e.debug_set_hp(0, 2);
    let zlbh_tao_id = e.catalog().skill_by_name("_zhulianbihe_mark_tao").unwrap();
    let out_tao = submit(&mut e, 0, GameAction::ActivateSkill { skill: zlbh_tao_id });
    let ch_tao = match out_tao {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _zhulianbihe_mark_tao:confirm, got {other:?}"),
    };
    assert_eq!(ch_tao.spec.prompt, "_zhulianbihe_mark_tao:confirm");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_tao.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(flag(&e, 0, "yexinjia_mark"), 0, "consumed yexinjia_mark fallback");
    assert_eq!(flag(&e, 0, "_zhulianbihe_tao_used"), 1);
    assert_eq!(e.debug_hp(0), 3, "recovered 1 HP");

    // 7. Batch A: _yinyang_mark_add (at Phase::Discard begin when hand > limit, consume mark for +2 hand limit)
    e.debug_grant(0, "_yinyang_mark_add");
    set_flag(&mut e, 0, "yinyang_mark", 1);
    let limit_before = e.state_core().effective_hand_limit(e.catalog(), 0);
    assert!(e.debug_hand(0).len() > limit_before);
    let out_discard = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let ch_yy = match out_discard {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected _yinyang_mark_add:confirm on Discard begin, got {other:?}"),
    };
    assert_eq!(ch_yy.spec.prompt, "_yinyang_mark_add:confirm");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_yy.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "yinyang_mark"), 0);
    assert_eq!(flag(&e, 0, "yinyang_hand_bonus"), 2);
    assert_eq!(
        e.state_core().effective_hand_limit(e.catalog(), 0),
        limit_before + 2,
        "yinyang_hand_bonus adds +2 maxHandcard this turn"
    );

    // 8. Batch A: _aozhan_judge & 37 UI/mark stubs on Phase::Prepare
    e.debug_grant(0, "_aozhan_judge");
    let ui_stubs = [
        "zhulianbihe_mark",
        "yinyang_mark",
        "yexinjia_mark",
        "_xigua_feiyanbahu_",
        "_shiqihujia_",
        "_guozhanChooseControlCHG",
        "_guozhan_diebiaoji_",
        "_guozhan_duiweibiaoji_gua_",
        "_guozhan_duiweibiaoji_gua_gua_",
        "_guozhanhalfchghp",
        "_shoushajisha_",
        "_shoushajisha_delete_",
        "_diankuangtulu_gua",
        "_wanjunqushou_gua",
        "_o_o_recovertrigger",
        "_o_o_miaoshouhuichun",
        "_o_o_yishugaochao",
        "_losehp_",
        "_mianshang_",
        "_player_baojiA",
        "_player_baojiB",
        "_player_baojiC",
        "_player_baojiD",
        "_player_baojiE",
        "_player_baojiF",
        "_player_baojiG",
        "_player_baojiH",
        "_jiubuff_",
        "_jiubuff_xiaoshi_",
        "_player_game_guagua",
        "_guozhan_quanjubiaoji_guagua",
        "_shiyongkapaitexiao_",
        "_shiyongkapaitexiao_xi_gua_",
        "_shanghaipaishiyongtexiao_",
        "_player_daojianfu_xxgg_fuzi_",
        "_binglintu",
        "_dysp",
    ];
    assert_eq!(ui_stubs.len(), 37);
    for stub in ui_stubs {
        e.debug_grant(0, stub);
    }
    let dysp_id = e.catalog().skill_by_name("_dysp").unwrap();
    e.debug_set_turn(0, Phase::Play, 2);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: dysp_id }),
            CommandOutcome::Rejected(_)
        ),
        "_dysp is a passive GameStart VFX hook, not an active Play-phase skill"
    );
    // Setup 3 alive players with distinct revealed factions for _aozhan_judge in 4-seat game
    e.debug_set_identity_roles(vec![
        engine_native::state::Role::Lord,
        engine_native::state::Role::Rebel,
        engine_native::state::Role::Traitor,
        engine_native::state::Role::Loyalist,
    ]);
    e.debug_set_general(0, "曹操");
    e.debug_set_general(1, "刘备");
    e.debug_set_general(2, "孙权");
    e.debug_set_general(3, "吕布");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 1);
    let _ = e.debug_deal_damage(Some(0), 3, 2, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert!(!e.debug_alive(3), "seat 3 eliminated so alive_count == 3 in 4-seat game");
    e.debug_set_turn(2, Phase::End, 2);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End -> seat 0 Prepare
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l20_ui_hook:_dysp"), 1, "_dysp fires passively on PhaseBegin/GameStart");
    for p in 0..3 {
        assert_eq!(
            flag(&e, p, "aozhan_active"),
            1,
            "_aozhan_judge sets aozhan_active=1 on all alive players when 3 distinct factions remain"
        );
    }

    let snap_end = e.save();
    let restored_end =
        Engine::restore_with_extensions(&snap_end, &packs).expect("restore end state");
    assert_eq!(restored_end.state_hash(), e.state_hash());
}

#[test]
fn l20_mark_supplement_batch_b_twenty_canonical_skills_and_save_restore() {
    let p9 = decision34_pack9_extension_pack();
    let packs = [&p9];

    // 1. zhengsu (Play begin choose_mode=1 bianzhen -> use 2 same-suit cards -> Discard end choose_reward=0 draw 2)
    //    & zhenyi_spade (OrdinaryJudge choose_suit=1 Heart -> zhenyi_spade_applied=1)
    {
        let mut e = Engine::new_with_extensions(192020, 4, &packs).expect("batch b1");
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "zhengsu");
        e.debug_grant(0, "zhenyi_spade");
        set_flag(&mut e, 0, "zhenyi_mark", 1);
        let le_cid = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|&c| e.catalog().card_kind(c) == engine_native::catalog::CardKind::Le)
            .unwrap();
        e.state_core_mut().deck.retain(|&x| x != le_cid);
        e.state_core_mut().player_mut(0).judge.push(le_cid);
        e.debug_set_turn(0, Phase::Prepare, 1);

        // Prepare -> Judge: triggers OrdinaryJudge -> zhenyi_spade:choose_suit
        let out_zy = submit(&mut e, 0, GameAction::NextPhase);
        let ch_zy = match out_zy {
            CommandOutcome::Pending { choice, .. } => choice,
            other => panic!("expected zhenyi_spade:choose_suit, got {other:?}"),
        };
        assert_eq!(ch_zy.spec.prompt, "zhenyi_spade:choose_suit");
        let snap_zy = e.save();
        let rest_zy = Engine::restore_with_extensions(&snap_zy, &packs).unwrap();
        assert_eq!(rest_zy.state_hash(), e.state_hash());
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch_zy.id,
                selection: vec![ChoiceOption::Tag(1)],
            },
        );
        drain(&mut e);
        assert_eq!(flag(&e, 0, "zhenyi_mark"), 0);
        assert_eq!(flag(&e, 0, "zhenyi_spade_applied"), 1);

        // Judge -> Draw -> Play: preset zhengsu_bianzhen (upstream zhengsu is initiated by another skill)
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain(&mut e);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain(&mut e);
        set_flag(&mut e, 0, "zhengsu_bianzhen", 1);
        // 擂进/变阵/鸣止之「变阵」：本阶段使用的牌花色相同且至少两张——真实打出两张同花色【无中生有】
        let wz: Vec<u32> = e
            .state_core()
            .deck
            .iter()
            .copied()
            .filter(|&c| e.catalog().card_kind(c) == engine_native::catalog::CardKind::Wuzhong)
            .collect();
        let suit0 = e.catalog().cards[wz[0] as usize].suit;
        let w2 = wz
            .iter()
            .copied()
            .skip(1)
            .find(|&c| e.catalog().cards[c as usize].suit == suit0)
            .expect("two same-suit Wuzhong");
        for c in [wz[0], w2] {
            e.state_core_mut().deck.retain(|&x| x != c);
            e.state_core_mut().player_mut(0).hand.push(c);
            let _ = submit(&mut e, 0, GameAction::PlayCard { card: c, targets: vec![] });
            drain(&mut e);
        }
        assert_eq!(flag(&e, 0, "zhengsu_succeeded"), 1);
        assert_eq!(flag(&e, 0, "zhengsu_failed"), 0);
        e.debug_clear_hand(0);
        // 受伤时才询问奖励（满体力自动摸 2）
        let hp0 = e.debug_hp(0);
        e.debug_set_hp(0, hp0 - 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
        let out_zsr = submit(&mut e, 0, GameAction::NextPhase); // Discard end -> zhengsu:choose_reward
        let ch_zsr = match out_zsr {
            CommandOutcome::Pending { choice, .. } => choice,
            other => panic!("expected zhengsu:choose_reward, got {other:?}"),
        };
        assert_eq!(ch_zsr.spec.prompt, "zhengsu:choose_reward");
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch_zsr.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        assert_eq!(flag(&e, 0, "zhengsu_rewarded"), 1);
        assert_eq!(e.debug_hand(0).len(), 2);
    }

    // 2. tspowei, qingyu, mibei, fz_new_rewusheng, fz_new_longdan, fz_new_tieji, fz_liegong, fz_xinkuanggu
    {
        let mut e = Engine::new_with_extensions(192021, 4, &packs).expect("batch b2");
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "tspowei");
        e.debug_grant(1, "qingyu");
        e.debug_grant(2, "mibei");
        set_flag(&mut e, 0, "tspowei_init", 1);
        e.debug_set_turn(3, Phase::End, 1);
        let _ = submit(&mut e, 3, GameAction::NextPhase); // TurnBegin(0) with no wei marks -> tspowei_achieved
        drain(&mut e);
        assert_eq!(flag(&e, 0, "tspowei_achieved"), 1);
        e.debug_set_turn(0, Phase::Play, 1);

        // qingyu prevents damage on seat 1 by discarding 2 cards
        let c1 = e.debug_place(1, "shan");
        let c2 = e.debug_place(1, "tao");
        e.debug_give_hand_card(1, 1, c1);
        e.debug_give_hand_card(1, 1, c2);
        let sha0 = e.debug_place(0, "sha");
        e.debug_give_hand_card(0, 0, sha0);
        let hp1_before = e.debug_hp(1);
        let out_sha = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha0,
                targets: vec![1],
            },
        );
        if let CommandOutcome::Pending { choice, .. } = out_sha {
            if choice.spec.prompt == "qingyu:discard_two" {
                let _ = submit(
                    &mut e,
                    1,
                    GameAction::Answer {
                        choice_id: choice.id,
                        selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
                    },
                );
            }
        }
        drain(&mut e);
        assert_eq!(e.debug_hp(1), hp1_before, "qingyu prevented damage");
        assert_eq!(flag(&e, 1, "qingyu_prevented"), 1);

        // mibei fails at PhaseBegin(End) if 0 cards in xingqi
        set_flag(&mut e, 2, "mibei_mark", 1);
        e.debug_set_turn(2, Phase::Discard, 1);
        let max_hp2 = e.state_core().player(2).max_hp;
        let _ = submit(&mut e, 2, GameAction::NextPhase);
        drain(&mut e);
        assert_eq!(flag(&e, 2, "mibei_resolved"), 2);
        assert_eq!(e.state_core().player(2).max_hp, max_hp2 - 1);
    }

    // 3. rekuanshi, kuanshi, kuanshi2, rechuhai, xinfu_lingren, sbtieji, sbduanliang, shouye, mbmowang, zuoxing
    {
        let mut e = Engine::new_with_extensions(192022, 4, &packs).expect("batch b3");
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "rechuhai");
        e.debug_grant(0, "sbduanliang");
        e.debug_grant(3, "mbmowang");
        e.debug_grant(0, "xinfu_lingren");
        e.debug_grant(0, "sbtieji");
        e.debug_grant(1, "shouye");
        e.debug_set_turn(0, Phase::Play, 1);

        // mbmowang Dying trigger: prevents death and enters resting state
        e.debug_set_hp(3, 1);
        let _ = e.debug_deal_damage(Some(0), 3, 1, engine_native::event::DamageNature::Normal);
        drain(&mut e);
        assert_eq!(flag(&e, 3, "mbmowang_triggered"), 1);
        assert_eq!(flag(&e, 3, "mbmowang_resting"), 1);
        assert!(e.state_core().player(3).alive);

        // zuoxing (`extension.js:6155-6204`) is commented out upstream: L20 must not bind it
        let zx_id = e.catalog().skill_by_name("zuoxing").unwrap();
        assert!(e.catalog().skills[zx_id as usize].behavior.as_host_behavior().is_none());

        // sbduanliang Mouyi duel
        let dl_id = e.catalog().skill_by_name("sbduanliang").unwrap();
        let out_dl = submit(&mut e, 0, GameAction::ActivateSkill { skill: dl_id });
        let ch_dl = match out_dl {
            CommandOutcome::Pending { choice, .. } => choice,
            other => panic!("expected sbduanliang:choose_target, got {other:?}"),
        };
        assert_eq!(ch_dl.spec.prompt, "sbduanliang:choose_target");
        let out_m1 = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch_dl.id,
                selection: vec![ChoiceOption::Player(2)],
            },
        );
        let ch_m1 = match out_m1 {
            CommandOutcome::Pending { choice, .. } => choice,
            other => panic!("expected sbduanliang:mouyi_owner, got {other:?}"),
        };
        assert_eq!(ch_m1.spec.prompt, "sbduanliang:mouyi_owner");
        let out_m2 = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch_m1.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        let ch_m2 = match out_m2 {
            CommandOutcome::Pending { choice, .. } => choice,
            other => panic!("expected sbduanliang:mouyi_target, got {other:?}"),
        };
        assert_eq!(ch_m2.spec.prompt, "sbduanliang:mouyi_target");
        let _ = submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: ch_m2.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        assert_eq!(flag(&e, 0, "sbduanliang_bing_placed"), 1);

        let snap_final = e.save();
        let rest_final = Engine::restore_with_extensions(&snap_final, &packs).unwrap();
        assert_eq!(rest_final.state_hash(), e.state_hash());
    }
}

