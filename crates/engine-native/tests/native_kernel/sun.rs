use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：孙尚香（枭姬）/ 华佗（青囊）（ADR 0749）
// ---------------------------------------------------------------------------

/// 枭姬：失去装备区里的一张牌（同栏替换）时摸两张牌。
#[test]
fn sun_xiangxiang_jixiang_draws_on_equip_loss() {
    let mut e = Engine::new(577, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "枭姬");
    // 第一件武器（无枭姬触发——技能授予后才有）；按名精确取（v64 后
    // debug_place 按种检索在同种多牌时不确定）。
    let w1 = e.debug_move_named_card_to_hand(0, "weapon2");
    let out = play(&mut e, w1, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let before = e.debug_hand(0).len();
    // 替换武器：枭姬触发确认（可选）→ 确认后摸二
    let w2b = e.debug_move_named_card_to_hand(0, "weapon2b");
    let out = play(&mut e, w2b, vec![]);
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "枭姬闸门应挂起: {out:?}"
    );
    let c = e.open_choice().expect("枭姬闸门");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(
        e.debug_hand(0).len(),
        before + 2,
        "失去装备区里的一张牌应摸两张牌"
    );
    assert!(log_has(
        &e,
        |l| matches!(l, LE::CardsDrawn { player: 0, cards } if cards.len() == 2)
    ));
    assert_eq!(discard_count(&e, w1), 1, "被替换的武器弃置恰一份");
    assert_zone_conservation(&e);
}
