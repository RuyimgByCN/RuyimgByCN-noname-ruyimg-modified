use super::*;

/// P2/N08 曹仁「据守」（真实翻面来源）：装配曹仁 → 结束阶段确认门
/// Confirm → 武将牌翻面 + 摸 1；翻面者回合开始被跳过并翻回（规则 v127
/// 翻面跳回合基建），故第二次据守为再翻至背面；装备【漩涡面具】后
/// 据守的翻面被取消（登记差异：摸牌仍执行）。
#[test]
fn caoren_jushou_flips_and_mianju_cancels_flip() {
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

    let mut e = Engine::new_with_extensions(3207, 3, &[&pack]).expect("建局");
    // 装配曹仁（P2/N08 装配授予原语）：据守随装配真实授予。
    e.debug_assemble_general(0, "曹仁");
    assert!(e.debug_has_skill(0, "据守"), "装配曹仁即授予据守");

    // 门感知推进：写轮眼/据守确认门留待调用方断言（返回 (observer, id)），
    // 其余门应答首选项；无门时推进阶段。
    fn step_until_gate_or(
        e: &mut Engine,
        target_player: u8,
        target_phase: Phase,
    ) -> Option<(u8, u64)> {
        // 返回 (observer, choice_id)
        for _ in 0..96 {
            if let Some(g) = e.open_choice() {
                if g.spec.prompt == "trigger_confirm" {
                    return Some((g.observer, g.id));
                }
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
                continue;
            }
            if e.turn().current == target_player && e.turn().phase == target_phase {
                return None;
            }
            let actor = e.turn().current;
            match submit(e, actor, GameAction::NextPhase) {
                CommandOutcome::Accepted { .. } => {}
                CommandOutcome::Pending { .. } => {} // 新开的门由下一轮处理
                other => panic!("advance: {other:?}"),
            }
        }
        panic!("96 步内未收敛");
    }

    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // ① 第一次据守：确认门 → Confirm → 翻面 + 摸 1。
    let gate = step_until_gate_or(&mut e, 0, Phase::End).expect("据守确认门应挂起");
    assert_eq!(gate.0, 0, "据守门 observer=0");
    let (gid, gobs) = (gate.1, gate.0);
    submit(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(e.debug_face_down(0), "据守翻面：武将牌背面朝上");

    // ② 第二次据守：先推过当前 End（①的门已消费）——0 号下个回合开
    // 始被跳过并翻回（v127 跳回合基建），再等其下个 End 的确认门
    // （此时正面 → 确认后翻至背面）+ 摸 1。
    let actor = e.turn().current;
    match submit(&mut e, actor, GameAction::NextPhase) {
        CommandOutcome::Accepted { .. } => {}
        other => panic!("推过当前 End: {other:?}"),
    }
    let gate = step_until_gate_or(&mut e, 0, Phase::End).expect("第二次据守确认门应挂起");
    assert_eq!(gate.0, 0);
    // 基线在确认前取：据守摸 1（推进途中 0 号 Draw 摸的 4 张不影响差值）。
    let hand_before2 = e.debug_hand(0).len();
    let (gid, gobs) = (gate.1, gate.0);
    submit(
        &mut e,
        gobs,
        GameAction::Answer {
            choice_id: gid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        e.debug_face_down(0),
        "第二次据守再翻至背面（回合开始已被跳过翻回）"
    );
    assert_eq!(e.debug_hand(0).len(), hand_before2 + 1, "第二次据守摸 1");

    // ③ 组合：装备面具后据守——翻面被取消（TurnOverBefore 强制取消）
    //    但摸牌仍执行（登记差异：取消翻面不影响据守摸牌）。
    // 先排空挂起门（v127 跳回合基建后循环变长，1/2 号可能进入弃牌门），
    // 再推进回 0 号出牌阶段（装备动作需在 Play 阶段），并重置朝向为
    // 正面（隔离②的翻转残留，保证断言只覆盖面具取消语义）。
    while let Some(g) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
            g.spec.options.iter().take(g.spec.min).copied().collect()
        } else {
            vec![g.spec.options[0]]
        };
        let (id, obs) = (g.id, g.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        );
    }
    advance_to_play(&mut e, 0);
    e.debug_set_face_down(0, false);
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
        "面具技能实例授予"
    );

    while let Some(g) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
            g.spec.options.iter().take(g.spec.min).copied().collect()
        } else {
            vec![g.spec.options[0]]
        };
        let (id, obs) = (g.id, g.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        );
    }
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    // 面具取消时无确认门（强制技不询问）——直接推进并断言：
    // face_down 不变（翻面被跳过）+ 手牌 +1（摸牌仍执行）。
    let hand_before3 = e.debug_hand(0).len();
    let face_before3 = e.debug_face_down(0);
    // 推进：据守确认门 → Confirm（发动据守）→ TurnOverBefore 子帧被
    // 面具强制取消（翻面不发生、摸牌执行）→ 阶段推进正常交棒。
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break; // 已离开 0 号回合（翻面被取消后正常交棒）
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Confirm],
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
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    assert_eq!(e.debug_hand(0).len(), hand_before3 + 1, "据守摸牌仍执行");
    assert_eq!(e.debug_face_down(0), face_before3, "翻面被面具取消");
    assert!(!e.debug_face_down(0), "0 号武将牌正面（翻面被面具取消）");
}
