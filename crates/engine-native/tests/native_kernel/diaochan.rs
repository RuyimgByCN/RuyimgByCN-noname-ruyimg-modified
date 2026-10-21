use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：貂蝉（闭月）/ 甄姬（洛神）（ADR 0739）
// ---------------------------------------------------------------------------

/// 闭月：结束阶段开始时摸一张牌（PhaseBegin(End) 触发）。
#[test]
fn diaochan_biyue_draws_at_end_phase() {
    let mut e = Engine::new(479, 4);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "闭月");
    // 记录 1 号 End 阶段开始前手牌数，推进至 End 阶段开始
    let before = e.debug_hand(1).len();
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            break;
        }
        assert!(e.open_choice().is_none(), "End 前不应有悬挂选择");
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    // 闭月为可选触发：闸门确认后才摸牌
    let c = e.open_choice().expect("闭月触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 1);
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(
        e.debug_hand(1).len(),
        before + 1,
        "闭月应在结束阶段开始时摸一张"
    );
    assert!(log_has(
        &e,
        |l| matches!(l, LE::CardsDrawn { player: 1, cards } if cards.len() == 1)
    ));
    assert_zone_conservation(&e);
}
