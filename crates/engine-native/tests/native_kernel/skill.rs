use super::*;

/// N08 动态技能资格（启停）：技能授予但 disabled 时不进入改判资格队列，
/// 判定静默完成（无询问、恰一次 Finalized、无 Rewritten）。
#[test]
fn skill_disabled_gate_excludes_rewriter() {
    let (mut e, _le) = build_guidao_pending();
    e.debug_grant(2, "鬼道");
    let _black = place_black_sha(&mut e, 2);
    e.debug_set_skill_enabled(2, "鬼道", false);
    // 判定静默完成：无任何询问
    for _ in 0..24 {
        if e.log()
            .iter()
            .any(|l| matches!(l, LE::JudgementFinalized { .. }))
        {
            break;
        }
        assert!(e.open_choice().is_none(), "disabled 技能不应产生改判询问");
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(e.open_choice().is_none());
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementRewritten { .. })),
        0
    );
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1
    );
    // 重新启用后恢复资格：下一轮 0 回合对 1 再用一张乐
    e.debug_set_skill_enabled(2, "鬼道", true);
    advance_to_play(&mut e, 0);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le2 = e.debug_place(0, "le");
    let out = play(&mut e, le2, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    loop {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "gui_dao_rewrite" {
                break;
            }
            // 摸牌可能带来无懈：一律 Pass（不抵消乐），改判询问才会出现
            assert_eq!(c.spec.prompt, "nullify_window");
            let cid = c.id;
            let obs = c.observer;
            let out = e.submit(GameCommand {
                actor: obs,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            continue;
        }
        if e.log()
            .iter()
            .filter(|l| matches!(l, LE::JudgementFinalized { .. }))
            .count()
            >= 2
        {
            panic!("重新启用后应出现改判询问");
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
}
