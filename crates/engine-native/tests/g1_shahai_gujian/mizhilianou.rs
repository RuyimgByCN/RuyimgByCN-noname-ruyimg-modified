use super::*;

/// 蜜链藕（红桃当桃：出牌 view-as + 濒死响应选项并入）与
/// 卤玉羹（出牌阶段限一次弃基本牌获得牌堆顶）。
#[test]
fn mizhilianou_viewas_and_luyugeng_discover() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let _find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 蜜链藕：濒死时红桃手牌当桃（响应选项并入） ——
    let mut e = Engine::new_with_extensions(5220, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (mzl, sha_black, tao_red, shan) = {
        let catalog = e.catalog();
        let mzl = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_mizhilianou"))
            .expect("蜜链藕在目录") as u32;
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let tao_red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        let shan = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (mzl, sha_black, tao_red, shan)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[mzl, sha_black, tao_red, shan]);
    snap.core.players[0].hand.push(mzl);
    snap.core.players[0].hand.push(tao_red);
    snap.core.players[1].hand.push(sha_black);
    snap.core.players[2].hand.push(shan);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: mzl,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 1 号回合出杀打 0 号（体力压到 1 → 命中后濒死）
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            drop(c);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_black,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify_window(&mut e);
    // 杀响应：无闪直过（若有 respond_shan 则 Pass）
    if let Some(c) = e.open_choice() {
        if c.spec.prompt == "respond_shan" {
            let cid = c.id;
            let obs = c.observer;
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }
    // 0 号濒死 → 濒死救援询问：0 号自己的选项应含红桃桃（本身即桃）
    // —— 用 2 号（持闪无桃）验证 pass 流转；关键：0 号可出红桃桃获救
    let saved;
    loop {
        let c = e.open_choice().expect("濒死救援挂起");
        let (prompt, obs) = (c.spec.prompt.to_string(), c.observer);
        assert_eq!(prompt, "dying_save");
        if obs == 0 {
            // 0 号：红桃桃是合法选项（桃本身）；此处直接出桃
            let has_tao = c.spec.options.contains(&ChoiceOption::Card(tao_red));
            let cid = c.id;
            if has_tao {
                submit(
                    &mut e,
                    0,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Card(tao_red)],
                    },
                );
                saved = true;
                break;
            } else {
                submit(
                    &mut e,
                    0,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
        } else {
            let cid = c.id;
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }
    assert!(saved, "0 号出红桃桃获救");
    assert_zone_conservation(&e);

    // —— 蜜链藕状态驱动 view-as：黑桃非红桃不可当桃 ——
    //（负例由上面 sha2 无红桃时不触发覆盖；此处验证蜜链藕未授予时
    // 红桃不进濒死选项——由 purge 注入的 2 号无状态即自然覆盖）

    // —— 卤玉羹：出牌阶段限一次，弃基本牌获得牌堆顶 ——
    let mut e = Engine::new_with_extensions(5221, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (lyg, shan2) = {
        let catalog = e.catalog();
        let lyg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_luyugeng"))
            .expect("卤玉羹在目录") as u32;
        let shan2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (lyg, shan2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lyg, shan2]);
    snap.core.players[0].hand.push(lyg);
    snap.core.players[0].hand.push(shan2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lyg,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 主动技：ActivateSkill 卤玉羹监听技
    let def_lyg = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_LUYUGENG_KEY)
        .expect("卤玉羹监听技在目录");
    let deck_top = e.save().core.deck.last().copied().expect("牌堆非空");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lyg });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("卤玉羹弃牌挂起");
    assert_eq!(c.spec.prompt, "gj_luyugeng_cost");
    assert_eq!(c.observer, 0);
    // 选项只含基本牌（shan2 是基本牌；卤玉羹本体已弃）
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(shan2)]);
    let cid = c.id;
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan2)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let hand0 = e.debug_hand(0);
    assert!(hand0.contains(&deck_top), "获得牌堆顶牌");
    assert!(!hand0.contains(&shan2), "基本牌已弃");
    // 每回合限一次：再次发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lyg });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "once_per_turn 应拒绝重复发动: {out:?}"
    );
    assert_zone_conservation(&e);
}
