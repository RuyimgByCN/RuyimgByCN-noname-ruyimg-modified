use super::*;

// ---------------------------------------------------------------------------
// 拒绝与回滚：非法应答、错误时机、预算耗尽，状态逐字节不变
// ---------------------------------------------------------------------------

#[test]
fn rejections_leave_state_unchanged() {
    let mut e = Engine::new(71, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    let (choice_id, _) = open_choice_snapshot(&e);
    let hash_before = e.state_hash();

    // 非选项里的牌 → InvalidChoice；状态不变、选择仍开放
    match e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Card(9999)],
        },
    }) {
        CommandOutcome::Rejected(RejectReason::InvalidChoice) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before);
    assert_eq!(open_choice_snapshot(&e), (choice_id, "trigger_confirm"));

    // 错误 choice_id → NoSuchChoice
    match e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: choice_id + 100,
            selection: vec![ChoiceOption::Confirm],
        },
    }) {
        CommandOutcome::Rejected(RejectReason::NoSuchChoice) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before);

    // 选择挂起时不能出牌
    let any_card = e.debug_hand(1)[0];
    match e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard {
            card: any_card,
            targets: vec![2],
        },
    }) {
        CommandOutcome::Rejected(RejectReason::ChoicePending) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before);

    // 错误阶段使用闪——但选择挂起的门槛先拦截
    // （debug_place 本身改状态，因此以其之后为回滚基准）
    let shan = e.debug_place(1, "shan");
    let hash_after_place = e.state_hash();
    match e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard {
            card: shan,
            targets: vec![],
        },
    }) {
        CommandOutcome::Rejected(RejectReason::ChoicePending) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.state_hash(), hash_after_place);

    // 正常应答仍可继续
    assert!(matches!(
        answer(&mut e, ChoiceOption::Confirm),
        CommandOutcome::Pending { .. }
    ));
}
