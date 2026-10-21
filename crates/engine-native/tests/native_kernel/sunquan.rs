use super::*;

// ---------------------------------------------------------------------------
// N11 主动技参数化选择：孙权制衡（ADR 0747）
// ---------------------------------------------------------------------------

/// 制衡：弃任意张手牌摸等量——弃 2 张摸 2 张；空选择弃权不摸。
#[test]
fn sunquan_zhiheng_discard_n_draw_n() {
    let mut e = Engine::new(557, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "制衡");
    // 准备 3 张已知手牌
    let c1 = e.debug_place(0, "shan");
    let c2 = e.debug_place(0, "shan");
    let _c3 = e.debug_place(0, "shan");
    // 发动制衡
    let skill_def = (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == "制衡")
        .expect("目录应有制衡");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应挂起于选择: {out:?}"
    );
    // 选择弃 2 张（c1、c2）
    let c = e.open_choice().expect("zhiheng_select 应挂起");
    assert_eq!(c.spec.prompt, "zhiheng_select");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 弃 2 摸 2：手牌 3-2+2=3
    assert_eq!(e.debug_hand(0).len(), 3);
    assert_eq!(discard_count(&e, c1), 1, "制衡弃置的牌入弃牌堆");
    assert_eq!(discard_count(&e, c2), 1);
    // 每回合限一次：再次发动 → SkillAlreadyUsed
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ));
    assert_zone_conservation(&e);
}
