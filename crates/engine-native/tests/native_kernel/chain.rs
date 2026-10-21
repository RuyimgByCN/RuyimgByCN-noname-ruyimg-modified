use super::*;

/// 铁索横置/重置：对已链玩家再出铁索会解除。
#[test]
fn chain_card_toggles_link() {
    let mut e = Engine::new(111, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    assert!(matches!(
        play(&mut e, chain, vec![1, 2]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(e.debug_linked(1) && e.debug_linked(2));
    assert!(!e.debug_linked(0) && !e.debug_linked(3));
    // 铁索已结算离手（不再在任何玩家手中）
    for p in 0..4 {
        assert!(!e.debug_hand(p).contains(&chain), "chain must leave hands");
    }
    // 再对 1 出铁索 → 重置
    let chain2 = e.debug_place(0, "chain");
    assert!(matches!(
        play(&mut e, chain2, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(!e.debug_linked(1));
    assert!(e.debug_linked(2));
    assert!(log_has(&e, |l| matches!(
        l,
        LE::Linked {
            player: 1,
            linked: false
        }
    )));
}

/// 铁索连环的**逐目标**无懈窗口（对齐上游 `useCardToBegin` 逐目标窗口，
/// 规则 v290）：【铁索连环】为 `complexTarget` 非 multitarget，每个目标各有
/// 独立无懈窗口，无懈以 `neutralize()` 只作用于该目标。本测试冻结第一方向：
/// 抵消目标 1 的窗口后 1 不横置、2 照常横置，且整张牌只入弃牌堆一次。
#[test]
fn chain_nullify_is_per_target_first_only() {
    let mut e = Engine::new(173, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    let nul = e.debug_place(1, "nullify");
    let out = play(&mut e, chain, vec![1, 2]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(
        e.open_choice().unwrap().observer,
        1,
        "目标 1 的窗口先询问 1"
    );
    let out = answer(&mut e, ChoiceOption::Card(nul));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_linked(1), "被抵消的目标不横置");
    assert!(
        e.debug_linked(2),
        "其余目标照常横置（旧整牌级窗口会一起跳过）"
    );
    for p in 0..4 {
        assert!(!e.debug_hand(p).contains(&chain), "铁索离手");
    }
    assert_eq!(
        e.state_core()
            .discard
            .iter()
            .filter(|c| **c == chain)
            .count(),
        1,
        "铁索整张只入弃牌堆一次"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
    // 只有目标 1 的窗口询问过（目标 2 窗口队列无人持无懈，不询问）
    assert_eq!(count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. })), 1);
}

/// 同上的另一方向：放弃目标 1 的窗口、对目标 2 出无懈——被抵消者只有
/// 目标 2，目标 1 已横置（证明两窗口不共享抵消状态与询问进度）。
#[test]
fn chain_nullify_is_per_target_second_only() {
    let mut e = Engine::new(174, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    let nul = e.debug_place(1, "nullify");
    let out = play(&mut e, chain, vec![1, 2]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    // 目标 1 的窗口放弃
    let out = answer(&mut e, ChoiceOption::Pass);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(
        e.open_choice().unwrap().observer,
        1,
        "目标 2 的窗口继续询问 1"
    );
    let out = answer(&mut e, ChoiceOption::Card(nul));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.debug_linked(1), "目标 1 的窗口未抵消 → 已横置");
    assert!(!e.debug_linked(2), "目标 2 的窗口被抵消 → 不横置");
    assert_eq!(
        e.state_core()
            .discard
            .iter()
            .filter(|c| **c == chain)
            .count(),
        1,
        "铁索整张只入弃牌堆一次"
    );
}

/// 两目标各自的窗口都被无懈抵消 → 无人横置，铁索仍只弃置一次。
/// 同一窗口内的无懈对无懈链（第二张无懈会反转抵消状态）不属于本测试：
/// 这里在目标 1 的窗口被抵消后放弃追加无懈，保留目标 2 的知识面。
#[test]
fn chain_double_nullify_leaves_nobody_linked() {
    let mut e = Engine::new(176, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    let nul1 = e.debug_place(1, "nullify");
    let nul2 = e.debug_place(2, "nullify");
    let out = play(&mut e, chain, vec![1, 2]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    // 目标 1 的窗口被抵消
    let out = answer(&mut e, ChoiceOption::Card(nul1));
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(
        e.open_choice().unwrap().observer,
        2,
        "同一窗口的无懈对抗询问 2"
    );
    // 放弃追加无懈：目标 1 的窗口关闭于「已抵消」
    let out = answer(&mut e, ChoiceOption::Pass);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(
        e.open_choice().unwrap().observer,
        2,
        "目标 2 的窗口询问持无懈的 2"
    );
    assert!(!e.debug_linked(1), "目标 1 曾被抵消且无追加无懈");
    let out = answer(&mut e, ChoiceOption::Card(nul2));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        !e.debug_linked(1) && !e.debug_linked(2),
        "两个窗口均被抵消，无人横置"
    );
    assert_eq!(
        e.state_core()
            .discard
            .iter()
            .filter(|c| **c == chain)
            .count(),
        1,
        "铁索整张只入弃牌堆一次"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
}

/// 传导伤害来源保留：火杀铁索传导后，伤害来源仍是原始来源。
#[test]
fn chain_propagation_preserves_source() {
    let mut e = Engine::new(307, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    let fsha = e.debug_place(0, "fire_sha");
    play(&mut e, fsha, vec![1]);
    // 传导给 2 的伤害来源仍为 0（原始来源保留）
    assert!(log_has(&e, |l| matches!(
        l,
        LE::DamageDealt {
            source: Some(0),
            target: 2,
            ..
        }
    )));
    // 被传导的目标 2 确实受伤
    assert_eq!(hp(&e, 2), 3);
    assert_zone_conservation(&e);
}
