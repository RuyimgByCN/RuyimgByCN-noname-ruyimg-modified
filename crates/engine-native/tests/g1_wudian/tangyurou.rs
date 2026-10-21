use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十五切片（规则 v156）：唐雨柔（pal_tangyurou）甜香去
//（txianqu，伤害前置窗口确认每回合限一次）——防止此伤害+摸 2+回复 1。
// qiongguang（纯表现技 noop）登记不转。
// ---------------------------------------------------------------------------

/// 甜香去：自己出牌阶段造成伤害→确认→防止+摸 2+回复 1；第二次不触发。
#[test]
fn tangyurou_txianqu_prevent_and_replenish() {
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

    let mut e = Engine::new_with_extensions(10013, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "txianqu");
    e.debug_set_hp(0, 2); // 已受伤（回复可见）
    e.debug_set_hp(1, 4);
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    let hand_before = e.debug_hand(0).len();

    // ① 杀 1 号→甜香去确认门→Confirm→防止+摸 2+回复 1（体力不变 2）
    let sha = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        assert_eq!(c.spec.prompt, "trigger_confirm");
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Confirm],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 4, "甜香去：伤害防止（体力不变）");
    assert_eq!(e.debug_hp(0), 3, "甜香去：自己回复 1（2→3）");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "摸 2（-1 杀+2=净 +1）"
    );

    // ② 第二次：txianqu_used 置位不触发（杀照常 4→3）
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
    assert_eq!(e.debug_hp(1), 3, "第二次伤害照常（4→3）");
}
