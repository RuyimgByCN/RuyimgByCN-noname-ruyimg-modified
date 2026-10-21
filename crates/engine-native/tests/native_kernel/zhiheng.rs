use super::*;

/// 制衡：空选择（无手牌或全弃权）不摸牌直接结束。
#[test]
fn zhiheng_decline_draws_nothing() {
    let mut e = Engine::new(559, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_grant(0, "制衡");
    // 无手牌：activate 返回 None → 引擎 InvalidChoice
    let skill_def = (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == "制衡")
        .expect("目录应有制衡");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));
}
