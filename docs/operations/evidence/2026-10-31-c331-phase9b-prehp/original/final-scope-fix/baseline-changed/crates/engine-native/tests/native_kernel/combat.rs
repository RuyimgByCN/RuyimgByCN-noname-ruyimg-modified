use super::*;

/// 杀的完整链路：闪响应免伤、放弃则扣血；出牌阶段限一次杀；距离限制。
#[test]
fn sha_dodge_damage_limit_and_range() {
    // 场景一：目标出闪
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_clear_hand(1); // 目标在自己回合摸的牌清掉，只留闪
    let shan = e.debug_place(1, "shan");
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "respond_shan");
    assert!(matches!(
        answer(&mut e, ChoiceOption::Card(shan)),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 1), 4);

    // 场景二：目标无闪直接扣血；同阶段第二张杀被拒（限一次）
    let mut e = Engine::new(11, 4);
    let sha1 = e.debug_place(0, "sha");
    let _sha2 = e.debug_place(0, "sha");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1); // 推进后清空，避免摸牌干扰
    assert!(matches!(
        play(&mut e, sha1, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 1), 3);
    let sha2 = e
        .debug_hand(0)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha)
        .expect("仍持有一张杀");
    match play(&mut e, sha2, vec![1]) {
        CommandOutcome::Rejected(RejectReason::ShaLimit) => {}
        other => panic!("expected ShaLimit, got {other:?}"),
    }

    // 场景三：距离 2 的目标不合法（4 人座，0 -> 2）
    let mut e = Engine::new(11, 4);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    advance_to_play(&mut e, 0);
    match play(&mut e, sha, vec![2]) {
        CommandOutcome::Rejected(RejectReason::IllegalTargets) => {}
        other => panic!("expected IllegalTargets, got {other:?}"),
    }
}

/// 火杀经铁索传导：同属性同点数、传导后全员解链。
#[test]
fn fire_sha_propagates_through_chain() {
    let mut e = Engine::new(113, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    // 火杀打 1（1 无闪）
    let fsha = e.debug_place(0, "fire_sha");
    assert!(matches!(
        play(&mut e, fsha, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    // 目标 1 与传导目标 2 各 -1；3 号（未链）不受影响
    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 4);
    // 全员解链
    assert!(!e.debug_linked(1) && !e.debug_linked(2));
    // 两段同属性伤害都在日志
    assert_eq!(
        count_log(&e, |l| matches!(
            l,
            LE::DamageDealt { target: 1, .. } | LE::DamageDealt { target: 2, .. }
        )),
        2
    );
}

/// 普通伤害不传导、不解链。
#[test]
fn normal_damage_does_not_propagate() {
    let mut e = Engine::new(115, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    let sha = e.debug_place(0, "sha");
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 2), 4, "普通伤害不传导");
    assert!(e.debug_linked(1) && e.debug_linked(2), "普通伤害不解链");
}

/// 雷属性伤害传导（debug 入口 = 后续技能造成属性伤害的内核路径）。
#[test]
fn thunder_damage_propagates_and_unlinks_all() {
    let mut e = Engine::new(117, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain1 = e.debug_place(0, "chain");
    play(&mut e, chain1, vec![0, 1]);
    let chain2 = e.debug_place(0, "chain");
    play(&mut e, chain2, vec![2]);
    // 三人被链
    assert!(e.debug_linked(0) && e.debug_linked(1) && e.debug_linked(2));
    // 雷伤打 0：传导给 1、2（座位顺序），全员解链
    assert!(matches!(
        e.debug_deal_damage(None, 0, 1, DamageNature::Thunder),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 0), 3);
    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 2), 3);
    assert!(!e.debug_linked(0) && !e.debug_linked(1) && !e.debug_linked(2));
    // 已解链后再打 → 无传导
    assert!(matches!(
        e.debug_deal_damage(None, 1, 1, DamageNature::Thunder),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 1), 2);
    assert_eq!(hp(&e, 0), 3);
    assert_eq!(hp(&e, 2), 3);
}

/// 传导伤害是完全的伤害结算：可被移伤转移（火杀传导 → 移伤改写目标）。
#[test]
fn propagated_damage_is_full_resolution() {
    let mut e = Engine::new(119, 4);
    e.debug_grant(1, "移伤");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    // 牌堆构成变更（v62）后发牌位移：3 号起手可能摸到无懈——链子的
    // 无懈窗口会挂起（火杀提交将因此被拒），先排空（放弃）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    let fsha = e.debug_place(0, "fire_sha");
    play(&mut e, fsha, vec![1]);
    // 火杀的无懈窗口同理排空（放弃），随后等移伤确认。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    // 火杀先在 1 处结算 → 移伤确认 → 转移给 3
    assert_eq!(prompt_of(&e), "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(prompt_of(&e), "yishang_target");
    answer(&mut e, ChoiceOption::Player(3));
    // 转移后的 1 不掉血；传导以 3 为中心 → 2（仍链）掉血
    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 3), 3);
    assert_eq!(hp(&e, 2), 3);
}

/// 火杀与普通杀共享出牌阶段限一次。
#[test]
fn fire_sha_shares_sha_limit() {
    let mut e = Engine::new(121, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let fsha = e.debug_place(0, "fire_sha");
    assert!(matches!(
        play(&mut e, fsha, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 1), 3);
    let sha = e.debug_place(0, "sha");
    match play(&mut e, sha, vec![1]) {
        CommandOutcome::Rejected(RejectReason::ShaLimit) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(e.catalog().card_kind(0), CardKind::Sha);
}

// ---------------------------------------------------------------------------
// N08 前置：DamageData 携牌信息（ADR 0737）
// ---------------------------------------------------------------------------

/// 杀造成的伤害携带牌信息：DamageDealt 日志的 card = 该杀。
#[test]
fn sha_damage_dealt_carries_card() {
    let mut e = Engine::new(451, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]); // 1 无闪 → 伤害结算
    assert!(log_has(&e, |l| matches!(l,
        LE::DamageDealt { target: 1, amount: 1, card: Some(c), .. } if *c == sha)));
    assert_eq!(hp(&e, 1), 3);
}
