use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第三十三批切片（规则 v175）：苏媚（pal_sumei，shu 3 女）
// 射影（sheying，source Damage after 每回合 1 次）+花好（huahu，phaseUse
// limited 多目标舍身）——伤害牌入弃堆判定黑则回收；失 1 体力上限+
// clearSkills 自己令任意名其他角色各回复 1+获 1 护甲+摸 1。dujiang 毒
// 牌种不在目录登记暂缓。
// ---------------------------------------------------------------------------

/// 射影：实体伤害牌入弃堆判定黑→回收；判定非黑不回收。
#[test]
fn sumei_sheying_black_judge_reclaim() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10046, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "sheying");
    // 伤害牌（sha 实体入弃堆）；判定堆顶压黑（spade）确保判定黑
    let sha = e.debug_place(0, "sha");
    let _ = e.debug_set_deck_top_by(|d| {
        matches!(
            d.suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    });

    // ① 0 号杀 1 号（实体牌入弃堆）→ 判定黑→回收
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 1 号无闪直伤
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 判定牌花色随 seed——分断言：红（未回收，本回合封）或黑（已回收）
    let judged_black = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::LogEvent::Judged { card, .. } => Some(*card),
            _ => None,
        })
        .next()
        .map(|c| {
            matches!(
                e.catalog().cards[c as usize].suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            )
        })
        .expect("射影判定已发生");
    if judged_black {
        assert!(e.state_core().player(0).hand.contains(&sha), "判定黑：回收");
        assert!(e.state_core().turn.sheying_used, "置位");
    } else {
        assert!(
            !e.state_core().player(0).hand.contains(&sha),
            "判定红：不回收"
        );
        assert!(
            e.state_core().turn.sheying_used,
            "红也置位（direct 触发即耗次数）"
        );
    }
    assert!(e.state_core().turn.sheying_used, "每回合 1 次置位");
}

/// 花好：多目标舍身——失 1 体力上限+clearSkills 自己，目标们各回复 1+
/// 获 1 护甲+摸 1；一次性。
#[test]
fn sumei_huahu_sacrifice_multi_target() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10047, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 3); // 受伤：回复可见
    }
    e.debug_grant(0, "huahu");

    let def = e.catalog().skill_by_name("huahu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("花好目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_HUAHU_TARGETS
    );
    assert_eq!(c.spec.min, 1);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
    );
    assert_eq!(e.state_core().player(0).max_hp, 3, "失 1 体力上限");
    assert!(e.state_core().player(0).huahu_used, "一次性置位");
    for p in 1..3 {
        assert_eq!(e.debug_hp(p), 4, "{p} 号回复（3→4）");
        assert_eq!(e.state_core().player(p).hujia, 1, "{p} 号获 1 护甲");
        assert_eq!(e.debug_hand(p).len(), 1, "{p} 号摸 1");
    }
    let huahu_def = e.catalog().skill_by_name("huahu").expect("def");
    assert!(
        !e.state_core().skills.contains_key(&(0, huahu_def)),
        "clearSkills 自己"
    );
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
