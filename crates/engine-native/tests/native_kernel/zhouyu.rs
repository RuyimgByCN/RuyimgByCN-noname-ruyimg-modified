use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：周瑜（英姿）（ADR 0748）
// ---------------------------------------------------------------------------

/// 英姿：摸牌阶段摸牌时额外摸一张牌（PhaseBegin(Draw) 阶段触发）。
#[test]
fn zhouyu_yingzi_extra_draw_at_draw_phase() {
    let mut e = Engine::new(531, 4);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "英姿");
    // 推进到 1 号摸牌阶段：英姿触发确认
    loop {
        if e.open_choice().is_some() {
            break;
        }
        if e.turn().current == 1 && e.turn().phase == Phase::Draw {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    let c = e.open_choice().expect("英姿触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 1);
    let cid = c.id;
    let before = e.debug_hand(1).len();
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
    assert_eq!(e.debug_hand(1).len(), before + 1, "英姿应额外摸一张");
    assert!(log_has(
        &e,
        |l| matches!(l, LE::CardsDrawn { player: 1, cards } if cards.len() == 1)
    ));
    assert_zone_conservation(&e);
}
