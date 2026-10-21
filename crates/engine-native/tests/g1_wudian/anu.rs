use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十四切片（规则 v155）：阿奴（pal_anu）蛊舞（anwugu，伤
// 害后确认）+蛊舞·缚（anwugu2，三钩）——你伤害其他未中蛊角色后令其获
// 蛊 3 阶段：每回合最多使用 3 张牌+手牌上限 -1，自己每阶段结束 -1。
// lingdi 登记暂缓。
// ---------------------------------------------------------------------------

/// 蛊舞：伤害→获蛊；蛊舞生效中出牌受限（3 张后拒绝）+手牌上限 -1；阶
/// 段结束递减至解除。
#[test]
fn anu_anwugu_afflict_and_limits() {
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

    let mut e = Engine::new_with_extensions(10012, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "anwugu");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_place(0, "sha");

    // ① 杀 1 号→蛊舞确认门→Confirm→1 号获蛊 3 阶段
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
    assert_eq!(e.debug_hp(1), 3, "杀命中（4→3）");
    assert_eq!(e.state_core().player(1).anwugu_count, 3, "获蛊 3 阶段");

    // 0 号 Play→Discard→End（0 手牌 0 张无弃牌门）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End

    // ② 蛊舞生效：1 号回合出第 3 张牌被拒（countUsed>=3）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    for _ in 0..4 {
        e.debug_place(1, "wuzhong");
    }
    let mut used = 0;
    while used < 4 {
        let wz = e
            .debug_hand(1)
            .into_iter()
            .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Wuzhong)
            .expect("应有无中生有");
        let out = submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: wz,
                targets: vec![],
            },
        );
        if used < 3 {
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "前 3 张可用：{out:?}"
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
        } else {
            assert!(
                matches!(
                    &out,
                    CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)
                ),
                "蛊舞：第 4 张牌拒绝（{out:?}）"
            );
            break;
        }
        used += 1;
    }
    assert_eq!(used, 3, "蛊舞=每回合 3 张（第 4 张拒绝）");
    // 阶段结束递减：0 的 Discard/End 阶段结束已递 3→1（蛊舞在 0 回合中
    // 挂上，0 的后续阶段结束同样递减——上游 phaseAfter 每阶段同口径）
    assert_eq!(e.state_core().player(1).anwugu_count, 1, "蛊舞阶段结束递减");

    // ③ 阶段结束递减：1 号收束→蛊 2
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→收束→2 Judge
                                                      // 0 的 Discard/End 阶段结束已递 2 次（3→1），1 的 Judge/Draw 阶段
                                                      // 结束再递 2 次（1→0，归零解除）
    assert_eq!(e.state_core().player(1).anwugu_count, 0, "蛊舞递减归零解除");
}
