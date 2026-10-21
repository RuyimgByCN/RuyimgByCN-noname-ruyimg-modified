use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十二（规则 v139）：黑影（ow_heiying）隐匿（yinshen，
// 弃牌触发锁定）——失去手牌基本牌且手牌无基本牌→潜行（不能成为其他
// 角色的卡牌目标；自己回合判定阶段开始解除）。qinru/maichong 登记暂缓。
// ---------------------------------------------------------------------------

/// 隐匿：弃置基本牌后手牌无基本牌→潜行（他人杀不可指定）；自己判定
/// 阶段开始解除。
#[test]
fn heiying_yinshen_untargetable_until_own_judge() {
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

    let mut e = Engine::new_with_extensions(9995, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yinshen");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1
    let _ = e.debug_place(0, "wuzhong"); // 非基本牌留存
    let sha = e.debug_place(0, "sha"); // 基本牌弃置素材

    // 弃牌阶段：弃 sha→触发隐匿（潜行置位）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（弃牌门挂起）
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().player(0).yinshen_active, "潜行置位");
    assert!(
        !e.debug_hand(0).iter().any(|c| {
            matches!(
                e.catalog().cards[*c as usize].kind,
                engine_native::catalog::CardKind::Sha
                    | engine_native::catalog::CardKind::Shan
                    | engine_native::catalog::CardKind::Tao
                    | engine_native::catalog::CardKind::Jiu
            )
        }),
        "手牌无基本牌"
    );

    // 1 号回合杀 0 号：不可指定（潜行）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: atk,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)
        ),
        "潜行不可被指定：{out:?}"
    );

    // 解除：驱动到 0 号下回合判定阶段
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    assert!(!e.state_core().player(0).yinshen_active, "自己判定阶段解除");
}
