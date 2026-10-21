use super::*;

/// 桥接后的包：3 张实体牌 + 2 条宿主技能声明，行为标签合法。
#[test]
fn huanle_bridge_builds_pack() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "huanle-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    assert!(
        report
            .installed
            .iter()
            .any(|m| m.package_id == "huanle-cards"),
        "欢乐卡牌安装"
    );
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .unwrap();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    // 猴子/写轮眼（E03）：伴随技能由引擎表登记，装载器自动并入。
    assert_eq!(
        pack.card_count(),
        7,
        "苦无 + 手里剑×2 + 猴子 + 写轮眼 + 面具 + 酒尾（2026-10-07 补齐）"
    );
    assert_eq!(pack.host_skills.len(), 5, "两条行为声明 + 三个伴随技能");
}

/// 酒尾（可选触发）：结束阶段确认 → 已受伤回血；扩展机器经
/// OptionalGate 包装后仍可恢复。
#[test]
fn huanle_jiuwei_optional_confirm_and_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3202, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "huanle_jiuwei");
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2); // 已受伤 → 结束阶段应回血
    for p in 0..3 {
        e.debug_clear_hand(p); // 避免弃牌阶段的 discard_to_limit 先挂起
    }
    // 推进到结束阶段（酒尾确认挂起即停）
    for _ in 0..4 {
        if e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    // 可选触发确认（OptionalGate）
    let c = e.open_choice().expect("酒尾确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    // 字节恢复（OptionalGate 内含宿主机器）
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("可选触发挂起恢复");
    let cid = restored.open_choice().unwrap().id;
    submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(restored.debug_hp(0), 3, "已受伤 → 回复 1 点");
}

/// E03 猴子偷桃（使用前取消窗口原语）：
/// 真实装备猴子（equip_skills 授予）→ 他人出桃 → OptionalGate 确认门挂起
/// （挂起态可恢复）→ 确认后桃的结算被取消（不回血、不入弃堆），桃收为
/// 猴子持有者手牌，猴子被弃且装备技离场撤销；Pass 时桃正常结算。
#[test]
fn huanle_monkey_cancels_peach_and_takes_it() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3202, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 真实穿戴：0 号装备猴子 → equip_skills 授予 huanle_monkey。
    let monkey = e.debug_place(0, "hnl_monkey_3");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: monkey,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备猴子: {out:?}"
    );
    let def_monkey = e
        .catalog()
        .skill_by_name("huanle:monkey")
        .expect("猴子技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_monkey)),
        "装备授予 huanle_monkey 实例"
    );

    // 1 号受伤并持桃出桃 → 猴子确认门挂起（observer=0）。
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 2);
    e.debug_place(1, "tao");
    let tao = {
        let hand = e.debug_hand(1);
        let cat = e.catalog();
        *hand
            .iter()
            .find(|c| cat.card_kind(**c) == CardKind::Tao)
            .expect("桃在手") as u32
    };
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "猴子确认门应挂起: {out:?}"
    );
    let gate = e.open_choice().expect("确认门");
    assert_eq!(gate.spec.prompt, "huanle_monkey_confirm");
    assert_eq!(gate.observer, 0, "猴子持有者被询问");

    // 挂起态恢复：同集合恢复后确认门仍开。
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("确认门挂起恢复");
    assert_eq!(
        e.open_choice().expect("恢复后确认门仍挂起").spec.prompt,
        "huanle_monkey_confirm",
        "挂起语义已锚定"
    );

    // 确认：桃的结算被取消——1 号不回血、桃收为 0 号手牌、猴子入弃堆
    // 且装备技离场撤销（无 CardUsed 帧，使用不发生）。
    let (gate_id, gate_observer) = {
        let gate = e.open_choice().expect("恢复后确认门");
        (gate.id, gate.observer)
    };
    let out = submit(
        &mut e,
        gate_observer,
        GameAction::Answer {
            choice_id: gate_id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 2, "桃的结算被取消：不回血");
    assert!(e.debug_hand(0).contains(&tao), "桃收为猴子持有者手牌");
    assert!(
        !e.debug_equips(0).iter().any(|c| *c == monkey),
        "猴子应离开装备区"
    );
    assert!(
        !e.save().core.skills.contains_key(&(0, def_monkey)),
        "猴子离场撤销装备技"
    );
}

/// E03 写轮眼（技能窃取原语）：真实装备写轮眼 → 回合开始确认 → 选目标
/// → 选其技能（PickTag 下标）→ 临时获得（expires_end_of_turn）→ 回合
/// 结束后消失。
#[test]
fn huanle_xuelunyang_steals_target_skill_until_turn_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3203, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 真实穿戴：0 号装备写轮眼。
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xl,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备写轮眼: {out:?}"
    );
    let def_xl = e
        .catalog()
        .skill_by_name("huanle:xuelunyang")
        .expect("写轮眼技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_xl)),
        "装备授予实例"
    );

    // 1 号获得一个标准技能（窃取目标）。
    e.debug_grant(1, "马术");
    let def_mashu = e.catalog().skill_by_name("马术").expect("马术在目录");
    let stolen_name = e.catalog().skills[def_mashu as usize].name;

    // 写轮眼在装备者的**下一个**回合开始触发：先推进到 1 号回合，
    // 再推进到 0 号回合开始（Judge begin）→ 确认门挂起。
    advance_to_play(&mut e, 1);
    // 推进到 0 号回合开始（Judge begin）→ OptionalGate 确认门挂起
    // （NextPhase 的 Pending 返回即新开的门，由 open_choice 分支应答）。
    for _ in 0..64 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let gate = e.open_choice().expect("写轮眼确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    assert_eq!(gate.observer, 0);

    // 确认（OptionalGate 通过）→ 选目标（1 号）→ 选技能（PickTag 下标
    // 0 = 目标技能目录序首项 = 马术）。
    let (cid, obs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let target_pick = e.open_choice().expect("选目标挂起");
    assert_eq!(target_pick.spec.prompt, "huanle_xuelunyang_target");
    assert!(
        target_pick.spec.options.contains(&ChoiceOption::Player(1)),
        "目标选项应含 1 号"
    );
    let (tid, tobs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        tobs,
        GameAction::Answer {
            choice_id: tid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let pick = e.open_choice().expect("技能选择挂起");
    assert_eq!(pick.spec.prompt, "huanle_xuelunyang_skill");
    assert_eq!(pick.spec.kind, ChoiceKind::PickTag);
    let pick_id = {
        let g = e.open_choice().unwrap();
        g.id
    };
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: pick_id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );

    // 临时获得：0 号拥有「马术」实例（expires_end_of_turn）。
    let snap = e.save();
    let def_stolen = e
        .catalog()
        .skill_by_name(&stolen_name)
        .expect("被窃技能在目录");
    let inst = snap
        .core
        .skills
        .get(&(0, def_stolen))
        .expect("窃得技能实例");
    assert!(inst.expires_end_of_turn, "临时技能标记到回合结束");

    // 推进过 0 号回合结束 → 实例消失。
    for _ in 0..64 {
        if e.turn().current != 0 && e.save().core.skills.get(&(0, def_stolen)).is_none() {
            break;
        }
        if e.outcome().is_some() {
            break;
        }
        if e.open_choice().is_some() {
            let g = e.open_choice().unwrap();
            // 写轮眼确认门：Pass 放弃（不再窃取，保证到期断言时序干净）。
            if g.spec.prompt == "trigger_confirm" {
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
                continue;
            }
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            CommandOutcome::Rejected(_) => {}
            other => panic!("advance: {other:?}"),
        }
    }
    // 到期移除 = 实例禁用（enabled=false，条目保留）。
    let stolen_enabled = e
        .save()
        .core
        .skills
        .get(&(0, def_stolen))
        .map(|i| i.enabled)
        .unwrap_or(false);
    assert!(!stolen_enabled, "回合结束后窃得的技能应失效");
}

/// E03 写轮眼 Pass 路径：回合开始确认门选择放弃 → 不窃取，随后他人出桃
/// 正常结算（无取消窗口——猴子未装备、窃取未发生）。
#[test]
fn huanle_xuelunyang_pass_declines_theft() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3204, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xl,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼");
    e.debug_grant(1, "马术");

    // 推进到 1 号的出牌阶段；途中写轮眼确认门（每个 0 号回合开始触发
    // 一次）一律 Pass——放弃窃取。
    let mut passed_gates = 0usize;
    let mut reached = false;
    for _ in 0..96 {
        // 写轮眼在装备者的下个回合开始（Judge begin）触发——推到该点。
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            reached = true;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                passed_gates += 1;
                // OptionalGate 门 options=[Confirm]、allow_cancel=true：
                // 放弃 = 空选择取消（Pass 不在选项集，会被拒绝）。
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![],
                    },
                );
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    assert!(reached, "96 步内未到达 0 号回合开始（写轮眼触发点）");
    assert!(
        passed_gates >= 1,
        "写轮眼确认门应至少触发一次（已全部 Pass）"
    );
    // Pass 后无窃取实例（0 号只有写轮眼自身实例，无马术）。
    let def_mashu = e.catalog().skill_by_name("马术").unwrap();
    assert!(
        !e.save().core.skills.contains_key(&(0, def_mashu)),
        "放弃后不得窃得马术"
    );

    // 推进到 1 号出牌阶段（途中 0 号下回合开始的写轮眼门继续 Pass）。
    for gi2 in 0..96 {
        eprintln!(
            "DBG pass2 gi={gi2} cur={} phase={:?} open={:?}",
            e.turn().current,
            e.turn().phase,
            e.open_choice().map(|g| (g.observer, g.spec.prompt, g.id))
        );
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                // OptionalGate 门放弃 = 空选择取消。
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![],
                    },
                );
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }

    // 1 号出桃：无取消窗口（猴子未装备），正常结算回血。
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 2);
    e.debug_place(1, "tao");
    let tao = {
        let hand = e.debug_hand(1);
        let cat = e.catalog();
        *hand
            .iter()
            .find(|c| cat.card_kind(**c) == CardKind::Tao)
            .expect("桃在手") as u32
    };
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "无窗口时桃直接结算: {out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "桃正常回血");
}

/// E03 漩涡面具（翻面原语）：真实装备面具 → debug_flip 翻面被取消
/// （TurnOverBefore 强制取消，face_down 不变）→ 摘除面具后翻面正常。
///
/// E03 漩涡面具（翻面原语）：真实装备面具 → debug_flip 翻面被取消
/// （TurnOverBefore 强制取消，face_down 不变）→ 摘除面具后翻面正常。
#[test]
fn huanle_mianju_blocks_flip_while_equipped() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3206, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 摘除状态基线：未装备面具时翻面正常发生。
    let flipped = e.debug_flip(1);
    assert!(flipped, "无面具时翻面应发生");
    assert!(e.debug_face_down(1), "1 号武将牌背面朝上");

    // 真实穿戴：0 号装备面具。
    let mianju = e.debug_place(0, "hnl_mianju_5");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mianju,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备面具: {out:?}"
    );
    let def_mianju = e
        .catalog()
        .skill_by_name("huanle:mianju")
        .expect("面具技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_mianju)),
        "装备授予实例"
    );

    // 翻面被取消：0 号 flip → TurnOverBefore 强制取消 → face_down 不变。
    let flipped = e.debug_flip(0);
    assert!(!flipped, "面具应取消 0 号的翻面");
    assert!(!e.debug_face_down(0), "0 号武将牌朝向不变");

    // 摘除面具（离场撤销）后翻面正常。
    e.debug_remove_equip(0, mianju);
    assert!(
        !e.save().core.skills.contains_key(&(0, def_mianju)),
        "离场撤销实例"
    );
    let flipped2 = e.debug_flip(0);
    assert!(flipped2, "摘除面具后翻面应发生");
    assert!(e.debug_face_down(0), "0 号武将牌背面朝上");
}

/// E03 载荷验收（2026-10-07）：原生欢乐包 7 张实体牌与上游
/// `card/list.js` 逐张对照（名称序/花色/点数）+ 装备栏与伴随装备技。
/// 上游源（archive/upstream/noname/.../欢乐卡牌/card/list.js，
/// commit 5c02a745）：kuwu ♠6 / shoulijian ♦4 / shoulijian ♠4 /
/// monkey ♦5 / xuelunyang ♥2 / mianju ♣3 / jiuwei ♥9。
#[test]
fn huanle_pack_payload_matches_upstream_list() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    // 逐张对照（card_prefix hnl_ → 目录名 hnl_<name>_<i>）。
    let expect: [(&str, &str, u8, Option<CardKind>); 7] = [
        ("hnl_kuwu_0", "spade", 6, Some(CardKind::Weapon)),
        ("hnl_shoulijian_1", "diamond", 4, Some(CardKind::ShouLiJian)),
        ("hnl_shoulijian_2", "spade", 4, Some(CardKind::ShouLiJian)),
        ("hnl_monkey_3", "diamond", 5, Some(CardKind::Monkey)),
        ("hnl_xuelunyang_4", "heart", 2, Some(CardKind::Xuelunyang)),
        ("hnl_mianju_5", "club", 3, Some(CardKind::Mianju)),
        ("hnl_jiuwei_6", "heart", 9, Some(CardKind::Jiuwei)),
    ];
    assert_eq!(
        pack.cards.len(),
        7,
        "上游 7 张实体牌（酒尾 2026-10-07 补齐）"
    );
    for (card, (name, suit, rank, kind)) in pack.cards.iter().zip(expect) {
        assert_eq!(card.name, name, "牌名/序");
        let suit = match suit {
            "spade" => engine_native::catalog::Suit::Spade,
            "heart" => engine_native::catalog::Suit::Heart,
            "club" => engine_native::catalog::Suit::Club,
            _ => engine_native::catalog::Suit::Diamond,
        };
        assert_eq!(card.suit, suit, "{name} 花色与上游不符");
        assert_eq!(card.rank, rank, "{name} 点数与上游不符");
        assert_eq!(Some(card.kind), kind, "{name} 牌种");
    }

    // 装备栏对照（上游 subtype）：kuwu 武器、mianju 防具（equip2，
    // 2026-10-07 修正）、monkey/xuelunyang/jiuwei 宝物（equip5）。
    assert_eq!(
        CardKind::Weapon.equip_slot(),
        Some(engine_native::catalog::EquipSlot::Weapon)
    );
    assert_eq!(
        CardKind::Mianju.equip_slot(),
        Some(engine_native::catalog::EquipSlot::Armor)
    );
    assert_eq!(
        CardKind::Monkey.equip_slot(),
        Some(engine_native::catalog::EquipSlot::Treasure)
    );
    assert_eq!(
        CardKind::Xuelunyang.equip_slot(),
        Some(engine_native::catalog::EquipSlot::Treasure)
    );
    assert_eq!(
        CardKind::Jiuwei.equip_slot(),
        Some(engine_native::catalog::EquipSlot::Treasure)
    );
    assert_eq!(CardKind::ShouLiJian.equip_slot(), None, "手里剑为基础牌");

    // 真实建局：酒尾装备授予 huanle_jiuwei（包声明映射
    // host:phase_end_recover_or_draw）。
    let mut e = Engine::new_with_extensions(3210, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let jw = e.debug_place(0, "hnl_jiuwei_6");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jw,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备酒尾: {out:?}"
    );
    let def = e
        .catalog()
        .skill_by_name("huanle_jiuwei")
        .expect("酒尾技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def)),
        "装备酒尾授予实例"
    );
}

/// E03 载荷验收：酒尾（♥9 宝物，上游 phaseEnd frequent）——结束阶段
/// 可选：已受伤回复 1，满血摸 1；放弃则无事。确认门由 OptionalGate
/// 承担（TriggerMeta PhaseBegin + 过滤器 End begin:true = 结束阶段）。
#[test]
fn huanle_jiuwei_recovers_or_draws_at_turn_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3211, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jw = e.debug_place(0, "hnl_jiuwei_6");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jw,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备酒尾: {out:?}"
    );

    // 推进到 0 号结束阶段并应答确认门（confirm=true 时确认）。
    fn advance_to_end_gate(e: &mut Engine) {
        for _ in 0..96 {
            match e.open_choice() {
                Some(g) if g.spec.prompt == "trigger_confirm" && g.observer == 0 => return,
                Some(g) => {
                    let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                        g.spec.options.iter().take(g.spec.min).copied().collect()
                    } else {
                        vec![g.spec.options[0]]
                    };
                    let (id2, obs2) = (g.id, g.observer);
                    submit(
                        e,
                        obs2,
                        GameAction::Answer {
                            choice_id: id2,
                            selection: sel,
                        },
                    );
                }
                None => {
                    let actor = e.turn().current;
                    match submit(e, actor, GameAction::NextPhase) {
                        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                        other => panic!("advance: {other:?}"),
                    }
                }
            }
        }
        panic!("96 步内未到达结束阶段确认门");
    }

    // ① 受伤路径：hp 3/4 → End 确认 → 回复到 4。
    e.debug_set_hp(0, 3);
    advance_to_end_gate(&mut e);
    let (gid, gobs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hp(0), 4, "受伤时酒尾回复 1");

    // ② 满血路径：下回合 End 确认 → 摸 1。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("满血酒尾确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    let (gid2, gobs2) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        gobs2,
        GameAction::Answer {
            choice_id: gid2,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "满血时酒尾摸 1");
    assert_eq!(e.debug_hp(0), 4, "满血不回复");

    // ③ 放弃路径：再一回合 End 放弃 → 无事。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before3 = e.debug_hand(0).len();
    let gate3 = e.open_choice().expect("放弃路径确认门应挂起");
    let (gid3, gobs3) = (gate3.id, gate3.observer);
    submit(
        &mut e,
        gobs3,
        GameAction::Answer {
            choice_id: gid3,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(0).len(), hand_before3, "放弃不摸牌");
    assert_eq!(e.debug_hp(0), 4, "放弃不回复");
}

/// E03 跨牌组合矩阵（台账 U06，2026-10-07）①：写轮眼窃取据守——
/// 窃得的据守在窃得者回合结束阶段真实触发（翻面 + 摸 1），目标保留
/// 原技能（窃取为复制非转移）。
#[test]
fn huanle_combo_stolen_jushou_fires_for_thief() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3212, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xl,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备写轮眼: {out:?}"
    );
    // 目标 1 号装配曹仁（装配授予据守）。
    e.debug_assemble_general(1, "曹仁");
    assert!(e.debug_has_skill(1, "据守"), "1 号有据守");

    // 推进到 1 号回合（途中 1 号的据守确认门放弃），再到 0 号回合开始
    // 的写轮眼确认门。
    for _ in 0..128 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
            let sel: Vec<ChoiceOption> = if g.spec.prompt == "trigger_confirm" {
                vec![] // 其他玩家的可选触发一律放弃（保持 1 号朝向不变）
            } else if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let (cid, obs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let target_pick = e.open_choice().expect("选目标挂起");
    assert!(target_pick.spec.options.contains(&ChoiceOption::Player(1)));
    let (tid, tobs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        tobs,
        GameAction::Answer {
            choice_id: tid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let pick = e.open_choice().expect("技能选择挂起");
    let (pid, pobs) = (pick.id, pick.observer);
    submit(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );

    // 窃取完成：0 号拥有据守（expires），1 号保留原实例。
    let def_jushou = e.catalog().skill_by_name("据守").expect("据守在目录");
    let snap = e.save();
    assert!(snap.core.skills.contains_key(&(0, def_jushou)), "窃得据守");
    assert!(
        snap.core.skills.contains_key(&(1, def_jushou)),
        "目标保留据守（复制非转移）"
    );
    assert!(!e.debug_face_down(0), "窃得时 0 号尚未翻面");

    // 推进到 0 号结束阶段：窃得的据守触发 → 确认 → 翻面 + 摸 1。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("窃得据守确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    assert_eq!(gate.observer, 0);
    let (gid, gobs) = (gate.id, gate.observer);
    submit(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(e.debug_face_down(0), "窃得的据守翻面（对窃得者生效）");
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "窃得的据守摸 1");
}

/// E03 跨牌组合矩阵 ②：窃得据守 + 面具——窃得者的据守翻面被面具
/// 取消（TurnOverBefore 前置窗口对窃得技同样生效），摸牌仍执行
/// （登记差异复用）。
#[test]
fn huanle_combo_stolen_jushou_flip_cancelled_by_mask() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3213, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xl,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼");
    e.debug_assemble_general(1, "曹仁");

    for _ in 0..128 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
            let sel: Vec<ChoiceOption> = if g.spec.prompt == "trigger_confirm" {
                vec![]
            } else if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let (cid, obs) = {
        let g = e.open_choice().unwrap();
        (g.id, g.observer)
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let target_pick = e.open_choice().expect("选目标挂起");
    let (tid, tobs) = (target_pick.id, target_pick.observer);
    submit(
        &mut e,
        tobs,
        GameAction::Answer {
            choice_id: tid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let pick = e.open_choice().expect("技能选择挂起");
    let (pid, pobs) = (pick.id, pick.observer);
    submit(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let def_jushou = e.catalog().skill_by_name("据守").unwrap();
    assert!(
        e.save().core.skills.contains_key(&(0, def_jushou)),
        "窃得据守"
    );

    // 推进到 0 号出牌阶段，装备面具，再到结束阶段确认据守。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            submit(
                &mut e,
                obs2,
                GameAction::Answer {
                    choice_id: id2,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let mj = e.debug_place(0, "hnl_mianju_5");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mj,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备面具: {out:?}"
    );
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("面具下窃得据守确认门应挂起");
    let (gid, gobs) = (gate.id, gate.observer);
    submit(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(!e.debug_face_down(0), "面具取消窃得据守的翻面");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "摸牌仍执行（登记差异复用）"
    );
}

/// E03 跨牌组合矩阵 ③④：苦无的伤害来源联动——
/// ③ 手里剑造成的伤害**不是杀**：苦无不触发（两线语义一致的负例，
/// 目标弃权受伤后手牌不变）；
/// ④ 雷杀伤害触发苦无（原生 is_sha_like 含雷/火杀——登记差异的
/// 行为面固化：上游过滤精确杀名）。
#[test]
fn huanle_combo_kuwu_sha_like_and_non_sha_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3214, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 苦无牌种为 Weapon（与标准武器同种）：按目录名精确定位实体牌，
    // debug_place 的按种检索会误取标准武器。
    let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: kuwu,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备苦无");
    let def_kuwu = e
        .catalog()
        .skill_by_name("huanle_kuwu")
        .expect("苦无技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_kuwu)),
        "装备苦无授予实例"
    );

    // ③ 苦无持有者用手里剑：目标弃权受伤 → 苦无不触发（伤害牌非杀）。
    let sj = e.debug_place(0, "hnl_shoulijian_1");
    // 目标手牌两张均非闪：手里剑弃权受伤后不触发苦无；雷杀必命中。
    let t1_card = e.debug_place(1, "tao");
    let t1_card2 = e.debug_place(1, "jiu");
    let hp_before = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sj,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "手里剑目标询问挂起: {out:?}"
    );
    let c = e.open_choice().expect("手里剑嵌套选择");
    assert_eq!(c.observer, 1);
    let (cid, cobs) = (c.id, c.observer);
    // 弃权（Pass）= 受 1 点伤害。
    submit(
        &mut e,
        cobs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert_eq!(e.debug_hp(1), hp_before - 1, "弃权受 1 伤");
    assert!(
        e.debug_hand(1).contains(&t1_card) && e.debug_hand(1).contains(&t1_card2),
        "手里剑伤害不触发苦无（伤害牌不是杀，两线一致）"
    );

    // ④ 雷杀伤害：苦无触发（is_sha_like 含雷杀——登记差异行为面），
    // 目标弃一张手牌（确定性首张）。
    let leisha = e.debug_place(0, "leisha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: leisha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雷杀使用: {out:?}"
    );
    // 目标无闪：无强制出闪应答（若挂起则应答首选项）。
    if let Some(c) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        } else {
            vec![c.spec.options[0]]
        };
        let (id2, obs2) = (c.id, c.observer);
        submit(
            &mut e,
            obs2,
            GameAction::Answer {
                choice_id: id2,
                selection: sel,
            },
        );
    }
    assert_eq!(e.debug_hp(1), hp_before - 2, "雷杀命中");
    assert!(
        !e.debug_hand(1).contains(&t1_card),
        "雷杀伤害触发苦无：目标被弃一张手牌"
    );
    assert!(
        e.debug_hand(1).contains(&t1_card2),
        "苦无为确定性弃首张（登记差异：上游目标自选）"
    );
}

/// 整包验收首包（U06 升 G3 剩余项，2026-10-08）：真实宿主全生命周期——
/// 独立安装+激活 → 安装包→原生建局 → 对局中卸载（revoke_slot
/// deactivate）→ 在局续玩（lock_pinned 锁定副本）→ 重装守卫（Revoked）。
#[test]
fn huanle_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活（真实发现→安装→激活集）
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .expect("欢乐卡牌安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "安装后应处于激活集"
    );

    // ② 安装包 → 原生建局（bridge 自安装产物的 cards.json）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(3401, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 7);

    // 推进到对局中并落存档（在局状态）
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载：revoke_slot(deactivate=true)
    store
        .revoke_slot(&meta.namespace, "huanle-cards", "验收：对局中卸载", true)
        .expect("撤销");
    let revoke = store.read_revoke().unwrap();
    assert!(
        revoke.is_slot_revoked(&meta.namespace, "huanle-cards"),
        "槽位应已撤销"
    );
    assert!(
        !store.read_active().unwrap().packages.contains_key(&slot),
        "撤销后应移出激活集"
    );

    // ④ 在局续玩（lock_pinned）：引擎持有已装载副本，恢复+提交不受
    //    宿主撤销影响；包内容仍可用（目录含 7 张扩展牌）
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复（锁定副本）");
    assert_eq!(
        resumed.catalog().cards.len(),
        base_cards + 7,
        "锁定副本内容仍可用"
    );
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫：撤销后同包再安装被拒（Revoked）
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack 存在");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 新局不再能用已撤销包（宿主层安装产物已失效；引擎层由内容锁拒绝）：
    //    通过重新 import 验证守卫（import_extensions 内部 install 会撞撤销）。
    let report2 = import_extensions(&store, &root, &options());
    // 已安装+已撤销：install 阶段应报 Revoked（不再静默通过）
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

// ---------------------------------------------------------------------------
// T12/U06 G5 切片：同内容锁跨进程存档接力 + 错版/缺包拒绝。
// 父进程把「手里剑嵌套选择挂起态」经 NGSA 落盘，自启子进程（本测试
// 二进制 + HUANLE_G5_WORKER 环境变量识别）以同一扩展内容锁恢复、续跑
// 落账、回写；父进程校验回写。错版（包版本 +1）与缺扩展（纯标准）在
// 子进程内断言拒绝后仍完成正例接力。
// ---------------------------------------------------------------------------

#[test]
fn huanle_g5_cross_process_relay() {
    use engine_native::archive::{decode_archive, encode_archive};

    // 子进程分支：读档→错版/缺包拒绝断言→恢复→续跑→回写
    if let Ok(archive_path) = std::env::var("HUANLE_G5_ARCHIVE") {
        let reply_path = std::env::var("HUANLE_G5_REPLY").expect("reply path");
        let bytes = std::fs::read(&archive_path).expect("子进程读存档");
        let snap = decode_archive(&bytes).expect("子进程解码");

        let pack = huanle_pack_from_repo();
        // 错版拒绝：包版本 +1 → 内容锁不匹配
        let mut wrong_version = huanle_pack_from_repo();
        wrong_version.def.version += 1;
        assert!(
            Engine::restore_with_extensions(&snap, &[&wrong_version]).is_err(),
            "错版拒绝"
        );
        // 缺包拒绝：纯标准目录无法恢复扩展局
        assert!(Engine::restore(&snap).is_err(), "缺扩展包拒绝");
        // 正例：同一扩展内容锁恢复 + 续跑落账
        let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("子进程恢复");
        let c = e.open_choice().expect("子进程挂起询问在位");
        assert_eq!(c.spec.prompt, "host_target_discard_equip");
        let equip_in_options = c
            .spec
            .options
            .iter()
            .find_map(|o| match o {
                ChoiceOption::Card(id) => Some(*id),
                _ => None,
            })
            .expect("选项含目标装备");
        let (cid, obs) = (c.id, c.observer);
        // 子进程 dedup 高水位独立于父进程——用高段键避免与存档内已用键
        // 冲突（N10：重复键按重复命令干净拒绝）。
        let out = e.submit(GameCommand {
            actor: obs,
            expected_revision: e.revision(),
            dedup_id: 0xDEAD_BEEF_0000_0001,
            action: GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(equip_in_options)],
            },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "子进程应答被拒: {out:?}"
        );
        assert_eq!(
            e.debug_hp(1),
            e.debug_hp(2),
            "子进程续跑：弃装备免伤（体力不变；3 人局对照座位同值）"
        );
        assert!(
            !e.state_core().players[1].equips.contains(&equip_in_options),
            "子进程续跑：装备已弃"
        );
        std::fs::write(&reply_path, encode_archive(&e.save())).expect("子进程回写");
        return;
    }

    // 父进程：装配 → 手里剑挂起 → 落盘 → 自启子进程 → 校验回写
    let pack = huanle_pack_from_repo();
    let mut e = Engine::new_with_extensions(3301, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "huanle_kuwu");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (shoulijian_id,) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("扩展手里剑在目录") as u32;
        (sj,)
    };
    let mut snap = e.save();
    let purge = |snap: &mut engine_native::StateSnapshot, id: u32| {
        snap.core.deck.retain(|c| *c != id);
        snap.core.discard.retain(|c| *c != id);
        snap.core.table.retain(|c| *c != id);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != id);
            p.equips.retain(|c| *c != id);
            p.judge.retain(|c| *c != id);
        }
    };
    purge(&mut snap, shoulijian_id);
    snap.core.players[0].hand.push(shoulijian_id);
    let equip_id = {
        let cat = e.catalog();
        cat.card_by_name("horse_plus").expect("目录有马牌")
    };
    purge(&mut snap, equip_id);
    snap.core.players[1].equips.push(equip_id);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shoulijian_id,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "手里剑挂起");
    let c = e.open_choice().expect("嵌套选择挂起");
    assert_eq!(c.spec.prompt, "host_target_discard_equip");

    // 落盘 + 自启子进程
    let dir = tempfile::tempdir().expect("临时目录");
    let archive_path = dir.path().join("relay.ngsa");
    let reply_path = dir.path().join("relay.result.ngsa");
    std::fs::write(&archive_path, encode_archive(&e.save())).expect("父进程落盘");
    let exe = std::env::current_exe().expect("测试二进制");
    let status = std::process::Command::new(exe)
        .args([
            "--exact",
            "huanle::huanle_g5_cross_process_relay",
            "--nocapture",
        ])
        .env("HUANLE_G5_ARCHIVE", archive_path.display().to_string())
        .env("HUANLE_G5_REPLY", reply_path.display().to_string())
        .output()
        .expect("自启子进程");
    assert!(
        status.status.success(),
        "子进程接力失败: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    // 父进程校验回写存档
    let reply_bytes = std::fs::read(&reply_path).expect("读回写存档");
    let r =
        Engine::restore_with_extensions(&decode_archive(&reply_bytes).expect("解码回写"), &[&pack])
            .expect("父进程恢复回写");
    assert_eq!(
        r.debug_hp(1),
        r.debug_hp(2),
        "跨进程接力：免伤（体力不变）在回写存档中"
    );
    assert!(
        !r.state_core().players[1].equips.contains(&equip_id),
        "跨进程接力：装备已弃"
    );
}
