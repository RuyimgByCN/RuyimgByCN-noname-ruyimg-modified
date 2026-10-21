use super::*;

// ---------------------------------------------------------------------------
// N11 主动技能命令面：黄盖苦肉（ADR 0743）
// ---------------------------------------------------------------------------

/// 苦肉：出牌阶段主动技——失去 1 点体力摸两张牌（每回合限一次）。
#[test]
fn huanggai_kurou_lose_hp_draw_two() {
    let mut e = Engine::new(547, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "苦肉");
    // 找到苦肉技能定义号（SkillDefId = 目录下标）
    let skill_def = (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == "苦肉")
        .expect("目录应有苦肉");
    let before = e.debug_hand(0).len();
    let hp_before = e.debug_hp(0);
    // 发动苦肉
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), hp_before - 1, "苦肉应失去 1 点体力");
    assert_eq!(e.debug_hand(0).len(), before + 2, "苦肉应摸两张牌");
    // 每回合限一次：再次发动 → SkillAlreadyUsed
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
        ),
        "同回合重复发动应拒绝: {out:?}"
    );
    assert_zone_conservation(&e);
}
