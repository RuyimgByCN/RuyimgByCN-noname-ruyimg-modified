use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十三（规则 v130）：法老之鹰（ow_falaozhiying）飞弹
//（feidan，杀伤害后触发）——可弃 1 张牌：受伤者距离 1 内的其他角色各
// 受 1 点无牌伤害（按座位序；可放弃=不弃不弹）；杀无视距离。
// huoyu（虚拟 useCard）/feiying（共享技能文件）登记暂缓。
// ---------------------------------------------------------------------------

/// 飞弹：杀伤害后弃牌弹射（受伤者距 1 内的 0/2 号各受 1，含自己）；
/// 放弃（空选择）不弹射；杀无视距离（距离 2 可达）；决斗伤害不触发。
#[test]
fn falaozhiying_feidan_splash_cancel_and_range() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9941, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feidan");
    // 连弩解除限次；牌堆多放杀+一张弃牌素材
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    let d1 = e.debug_place(0, "tao");
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 杀 1 号（距离 1）→ 飞弹弃牌询问 → 弃 tao 弹射：受伤者 1 号距
    //    1 内的其他角色=0、2 号各受 1（含自己）
    let sha1 = pick_suit(&e, engine_native::catalog::Suit::Spade)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Club))
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Heart))
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond))
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
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
    let out = answer_open(&mut e, vec![ChoiceOption::Card(d1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "杀伤害（4→3）");
    assert_eq!(e.debug_hp(0), 3, "弹射含自己（4→3）");
    assert_eq!(e.debug_hp(2), 3, "弹射距离 1 邻座（4→3）");
    assert!(e.state_core().discard.contains(&d1), "飞弹弃牌入弃牌堆");

    // ② 杀 2 号（距离 2——飞弹杀无视距离）→ 飞弹放弃（空选择）→ 不弹
    let sha2 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应仍有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "距离 2 可达：{out:?}"
    );
    let out = answer_open(&mut e, Vec::new());
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(2), 2, "仅杀伤害（3→2）");
    assert_eq!(e.debug_hp(1), 3, "放弃不弹射（3 不变）");
    assert_eq!(e.debug_hp(3), 4, "放弃不弹射（4 不变）");

    // ③ 决斗伤害（非杀）不触发飞弹
    e.debug_set_hp(1, 4);
    let duel = e.debug_place(0, "duel");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
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
    assert_eq!(e.debug_hp(1), 3, "决斗伤害（4→3）");
    assert!(e.open_choice().is_none(), "决斗不触发飞弹");
}
