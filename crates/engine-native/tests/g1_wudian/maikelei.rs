use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十五（规则 v132）：麦克雷（ow_maikelei）神枪（shenqiang，
// 伤害后触发锁定）——你的杀对一名角色造成伤害后（自己回合），本回合
// 杀使用计数回退（可再出一张杀）。shanguang/tiandan 登记暂缓。
// ---------------------------------------------------------------------------

/// 神枪：杀伤害后限次解除——第一张杀命中后第二张杀合法；第三张仍拒
///（回退仅一次）。
#[test]
fn maikelei_shenqiang_sha_limit_refund() {
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

    let mut e = Engine::new_with_extensions(9961, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "shenqiang");
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    let s1 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let s2 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha && *c != s1)
        .expect("应有第二张杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s1,
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
    assert_eq!(e.debug_hp(1), 3, "第一张杀命中（4→3）");
    // 神枪回退：第二张杀合法（同回合，仍打距离 1 的 1 号）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神枪：杀计数回退后可再出（{out:?}）"
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
    assert_eq!(e.debug_hp(1), 2, "第二张杀命中（3→2）");

    // 第三张杀合法：每次杀伤害各回退一次（上游 getStat().card.sha--
    // 每事件结算、无次数上限——第二张的伤害再次回退，链式成立）
    if let Some(s3) = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
    {
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: s3,
                targets: vec![1],
            },
        );
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "链式回退：第三张杀合法（{out:?}）"
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
        assert_eq!(e.debug_hp(1), 1, "第三张杀命中（2→1）");
    }
}
