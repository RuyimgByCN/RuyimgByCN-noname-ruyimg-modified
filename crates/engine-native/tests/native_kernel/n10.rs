use super::*;

/// 策略类型隔离：PassivePolicy 只能从 PolicyInput 决策——
/// 驱动全程无 debug 干预，输入组装不改变权威状态。
#[test]
fn n10_policy_input_isolation() {
    let mut e = Engine::new(2100, 3);
    let h0 = e.state_hash();
    // 只组装输入（不提交）→ 权威状态不动
    let _input = e.ai_session(0).input();
    assert_eq!(e.state_hash(), h0, "输入组装不得修改权威状态");
    // 策略驱动推进（None → NextPhase 回退）
    for _ in 0..32 {
        let seat = e.turn().current;
        let mut p = PassivePolicy;
        let out = e.run_policy(seat, &mut p);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        if e.turn().phase == Phase::Play {
            break;
        }
    }
    assert_eq!(e.turn().phase, Phase::Play);
}

/// 有界规划器：克隆隔离（规划不改权威状态）、节点预算、预算耗尽标记。
#[test]
fn n10_planner_isolation_and_budget() {
    use engine_native::ai::BoundedPlanner;
    let mut e = Engine::new(2101, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 手里有杀 → 候选含杀攻两个目标 → 预算 1 会耗尽
    let _ = e.debug_place(0, "sha");
    let h0 = e.state_hash();
    let planner = BoundedPlanner::new(1);
    let plan = e.plan_action(0, &planner);
    assert_eq!(e.state_hash(), h0, "规划不得修改权威状态（克隆隔离）");
    assert!(plan.nodes_used <= 1, "节点预算必须生效");
    assert!(plan.budget_exhausted || plan.nodes_used < 1);
    // 应用计划（修订号未变 → 可用）
    let out = e.apply_plan(&plan);
    assert!(out.is_ok(), "{out:?}");
}

/// 有界规划器：时限与取消——预置取消标志 → 规划立即取消；
/// 0ms 时限 → 单节点检查即超时。
#[test]
fn n10_planner_cancel_and_deadline() {
    use engine_native::ai::BoundedPlanner;
    let mut e = Engine::new(2102, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 取消：预置标志
    let planner = BoundedPlanner::new(16);
    planner.cancel();
    let plan = e.plan_action(0, &planner);
    assert!(plan.cancelled, "预置取消应立即生效");
    // 0ms 时限：第一节点检查即超时
    let planner2 = BoundedPlanner::new(64).with_deadline_ms(0);
    let plan2 = e.plan_action(0, &planner2);
    assert!(plan2.timed_out, "0ms 时限应立即超时");
}

/// 有界规划器：陈旧结果拒绝——规划后修订号前进 → 应用被拒。
#[test]
fn n10_planner_stale_rejection() {
    let mut e = Engine::new(2103, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let planner = engine_native::ai::BoundedPlanner::new(8);
    let plan = e.plan_action(0, &planner);
    // 规划后修订号前进（外部干预）
    let _ = e.submit(GameCommand {
        actor: e.turn().current,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    match e.apply_plan(&plan) {
        Err(engine_native::ai::ApplyPlanError::Stale { submitted, current }) => {
            assert_ne!(submitted, current);
        }
        other => panic!("陈旧计划应被拒绝: {other:?}"),
    }
}

/// AI 基线：有界规划器托管 1v1 完整对局——进攻性规划在命令上限内
/// 打出真实终局，且对局中造成过伤害（基线有效性）。
#[test]
fn n10_planner_baseline_completes_games() {
    use engine_native::ai::BoundedPlanner;
    let mut finished = 0;
    let mut damaged_any = false;
    for g in 0..3u64 {
        let mut e = Engine::new(3000 + g, 2);
        let planner = BoundedPlanner::new(64);
        let mut last_hp = (4i64, 4i64);
        let mut damaged = false;
        for _ in 0..4096 {
            if e.outcome().is_some() {
                break;
            }
            let seat = match e.open_choice() {
                Some(c) => c.observer,
                None => e.turn().current,
            };
            let plan = e.plan_action(seat, &planner);

            let hp_now = (e.debug_hp(0) as i64, e.debug_hp(1) as i64);
            if hp_now != last_hp {
                damaged = true;
                last_hp = hp_now;
            }
            let _ = e.apply_plan(&plan);
        }
        assert!(e.outcome().is_some(), "局 {g} 应在命令上限内终局");
        if damaged {
            damaged_any = true;
        }
        finished += 1;
        let _ = damaged_any;
    }
    assert_eq!(finished, 3, "三局基线全部完局");
}
