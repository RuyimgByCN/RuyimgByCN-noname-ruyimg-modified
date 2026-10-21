use super::*;

/// 苦肉：未授予技能的玩家发动 → NoSuchSkill。
#[test]
fn activate_skill_without_grant_rejected() {
    let mut e = Engine::new(547, 4);
    advance_to_play(&mut e, 0);
    let skill_def = (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == "苦肉")
        .expect("目录应有苦肉");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::NoSuchSkill)
    ));
}

// ---------------------------------------------------------------------------
// N08 主动技阶段门控：ActivateSkill 默认限出牌阶段（上游 enable:
// "phaseUse"——仁德曾在判定阶段被放行）；响应入口类技能（激将
// chooseToUse/chooseToRespond）经目录声明例外，不绑定阶段。
// ---------------------------------------------------------------------------

/// 仁德在判定阶段发动 → WrongPhase 拒绝且状态/事件不变；
/// 恢复件同样拒绝；推进到出牌阶段后正常挂起 rende_cards。
#[test]
fn activate_skill_rejected_outside_play_phase() {
    // 新开局当前回合玩家处于判定阶段
    let mut e = Engine::new(2701, 3);
    assert_eq!(e.turn().phase, Phase::Judge);
    let cur = e.turn().current;
    e.debug_grant(cur, "仁德");
    e.debug_place(cur, "shan");
    let skill = e.catalog().skill_by_name("仁德").unwrap();
    let before = e.save();
    let out = submit_cmd(&mut e, cur, GameAction::ActivateSkill { skill });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::WrongPhase)),
        "判定阶段发动仁德必须被拒: {out:?}"
    );
    assert_eq!(e.save(), before, "被拒命令不得改变任何状态");

    // 恢复件在相同局面同样拒绝（门控在命令边界，恢复后仍生效）
    let mut r = Engine::restore(&before).unwrap();
    let out2 = submit_cmd(&mut r, cur, GameAction::ActivateSkill { skill });
    assert!(matches!(
        out2,
        CommandOutcome::Rejected(RejectReason::WrongPhase)
    ));

    // 推进到出牌阶段后可正常发动
    advance_to_play(&mut e, cur);
    let out3 = submit_cmd(&mut e, cur, GameAction::ActivateSkill { skill });
    match out3 {
        CommandOutcome::Pending { choice } => {
            assert_eq!(
                choice.spec.prompt, "rende_cards",
                "出牌阶段应挂起 rende_cards"
            );
        }
        other => panic!("出牌阶段发动仁德应挂起 rende_cards: {other:?}"),
    }
}
