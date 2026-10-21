use super::*;

// ---------------------------------------------------------------------------
// 嵌套选择链（散财）：确认 → 摸牌子事件 → 选目标 → 选牌
// ---------------------------------------------------------------------------

#[test]
fn sancai_nested_choice_chain_and_decline() {
    // 确认路径：三个选择依序出现，中途夹一张 CardsDrawn 子事件
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0); // 推进后清空目标手牌，保证无闪且计数确定
    let hand_before = e.debug_hand(0).len();
    assert!(matches!(
        play(&mut e, sha, vec![0]),
        CommandOutcome::Pending { .. }
    ));
    // 0 无闪 → 直接伤害 → 散财确认
    assert_eq!(prompt_of(&e), "trigger_confirm");
    assert!(matches!(
        answer(&mut e, ChoiceOption::Confirm),
        CommandOutcome::Pending { .. }
    ));
    // 确认后先结算摸牌（子事件），再问目标
    assert_eq!(prompt_of(&e), "sancai_target");
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::CardsDrawn { player: 0, .. }
    )));
    assert!(matches!(
        answer(&mut e, ChoiceOption::Player(2)),
        CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "sancai_card");
    let card_opt = e
        .open_choice()
        .unwrap()
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .unwrap();
    assert!(matches!(
        answer(&mut e, card_opt),
        CommandOutcome::Accepted { .. }
    ));
    let ChoiceOption::Card(given) = card_opt else {
        unreachable!()
    };
    // 0：+2 摸 -1 给；2：+1
    assert_eq!(e.debug_hand(0).len(), hand_before + 2 - 1);
    assert!(e.debug_hand(2).contains(&given));
    assert_eq!(hp(&e, 0), 3);
    assert_eq!(
        count_log(&e, |l| matches!(l, LogEvent::ChoiceOpened { .. })),
        4
    ); // 无闪跳过响应 +3

    // 放弃路径：空应答 = 不发动
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let hand_before = e.debug_hand(0).len();
    play(&mut e, sha, vec![0]);
    assert_eq!(prompt_of(&e), "trigger_confirm");
    assert!(matches!(
        answer_empty(&mut e),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(e.debug_hand(0).len(), hand_before);
    assert_eq!(hp(&e, 0), 3);
}
