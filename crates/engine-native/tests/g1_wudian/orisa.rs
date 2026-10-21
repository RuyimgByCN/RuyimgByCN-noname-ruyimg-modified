use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十九（规则 v136）：奥丽莎（ow_orisa）强固（qianggu，
// 出牌阶段限一次）+强固守卫（qianggu2，CardUsed 触发锁定）——弃 2 张
// he 获得 +2 护甲与强固：他人对持有者使用杀需弃一张基本牌否则杀无效
//（自己下回合判定阶段开始解除）。woliu 登记暂缓。
// ---------------------------------------------------------------------------

/// 强固：弃 2 张→甲 +2（0→2）；守卫：使用者无基本牌→杀无效（体力不
/// 变+牌不进弃牌堆）；解除：自己下回合判定阶段开始。
#[test]
fn orisa_qianggu_armor_and_guard_nullify() {
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

    let mut e = Engine::new_with_extensions(9992, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "qianggu");
    e.debug_grant(0, "qianggu2");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);

    // 强固：弃 2 张→甲 0→2+置位
    let def_qg = e.catalog().skill_by_name("qianggu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_qg });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.save().core.players[0].hujia, 2, "+2 护甲");
    assert!(e.state_core().player(0).qianggu_active, "强固生效");

    // 1 号回合杀 0 号（强固跨回合持续）：守卫询问使用者（无基本牌→
    // Pass→杀无效）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    let atk = e
        .debug_hand(1)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("1 号应有杀");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: atk,
            targets: vec![0],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("强固守卫询问");
    assert_eq!(c.observer, 1, "守卫询问 observer=使用者");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Pass]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), 4, "杀无效：体力不变");

    // 解除：驱动到 0 号下回合判定阶段开始
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
    assert!(
        !e.state_core().player(0).qianggu_active,
        "自己判定阶段开始解除"
    );
}
