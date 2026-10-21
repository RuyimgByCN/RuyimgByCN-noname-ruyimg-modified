use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十一（规则 v138）：查莉娅（ow_zhaliya）屏障（pingzhang，
// global 伤害前置窗口 direct）+应用（owliyong，摸牌阶段开始触发锁定）
// 双技全闭环——自己受伤弃红桃/他人受伤弃黑桃（各每回合一次）伤害 -1；
// 摸牌阶段额外摸上回合屏障次数张。空壳 pingzhang2/3=两 bool 标志。
// ---------------------------------------------------------------------------

/// 屏障：自己受伤弃红桃-1 伤（他人受伤需黑桃）；各每回合一次；应用：
/// 下回合摸牌阶段额外摸上回合次数张。
#[test]
fn zhaliya_pingzhang_barrier_and_owliyong_draw() {
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

    let mut e = Engine::new_with_extensions(9994, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "pingzhang");
    e.debug_grant(0, "owliyong");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    let heart = e.debug_place(0, "tao"); // 红桃素材（赤焰? tao=红桃系）
    let _ = heart;

    // ① 0 号杀 1 号（1 号掉血前 1 号杀 0 号——先做 1 号回合打 0 号：
    //    自己受伤弃红桃-1 伤）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw（1 摸 2）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    let hand0_before = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: atk,
            targets: vec![0],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 屏障询问 observer=0（自己受伤→红桃 tao）
    let c = e.open_choice().cloned().expect("屏障询问");
    assert_eq!(c.observer, 0);
    drop(c);
    // 弃红桃（0 号手牌中的 tao/红桃）
    let red = e
        .debug_hand(0)
        .into_iter()
        .find(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Heart
            )
        })
        .expect("0 号应有红桃");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(red)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), 4, "屏障：伤害 1-1=0（体力不变）");
    assert!(e.state_core().player(0).pingzhang_self_used, "自己项置位");
    let _ = hand0_before;

    // ② 0 号回合：判定开始标志转计数+摸牌阶段应用（+1）
    //    （驱动：1 收束→0 Judge 转计数→0 Draw 摸 2+应用 1）
    let hand0_before_draw = e.debug_hand(0).len();
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 收束→2 Judge
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
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
    // 0 号摸牌阶段摸 2（基础）+1（应用）——出牌阶段手牌=净增 3-上回合消耗
    assert!(!e.state_core().player(0).pingzhang_self_used, "标志已清");
    // 计数保持到下回合判定阶段（上游 storage 直到 phaseBegin 重置同口径）
    assert_eq!(e.state_core().player(0).pingzhang_count, 1, "计数保留");
    let gained = e.debug_hand(0).len() as i64 - hand0_before_draw as i64;
    assert_eq!(gained, 3, "摸牌阶段=基础 2+应用 1（上回合屏障 1 次）");
}
