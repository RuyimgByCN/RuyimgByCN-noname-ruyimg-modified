use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：曹操（奸雄）/ 郭嘉（遗计）（ADR 0738）
// ---------------------------------------------------------------------------

/// 奸雄：受到伤害后可获得造成此伤害的牌（牌在弃牌堆）。
/// 1 号持奸雄；0 号出杀打 1（无闪受伤）→ 触发确认 → 确认后获得该杀。
#[test]
fn caocao_jianxiong_takes_damaging_card() {
    let mut e = Engine::new(463, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "奸雄");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]); // 1 无闪 → 受伤 → 奸雄闸门询问
    let c = e.open_choice().expect("奸雄触发确认应挂起");
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
    assert!(e.debug_hand(1).contains(&sha), "奸雄应获得造成伤害的杀");
    assert_eq!(discard_count(&e, sha), 0, "获得的牌应离开弃牌堆");
    assert_zone_conservation(&e);
}
