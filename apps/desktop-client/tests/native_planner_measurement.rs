//! Native-only measurements for PA09; these are not WebView or PA08 acceptance results.
use engine_native::ai::{BoundedPlanner, PlanPool};
use engine_native::ai_pool::PlannerPool;
use engine_native::{Engine, GameAction, GameCommand};
use std::time::Instant;

fn percentile(values: &mut [u128], percent: usize) -> u128 {
    values.sort_unstable();
    values[(values.len() * percent).div_ceil(100).saturating_sub(1)]
}

#[test]
#[ignore = "native wall-clock measurement; run explicitly with --ignored --nocapture"]
fn actual_single_seat_planning_compares_existing_pools_and_native_clock() {
    let mut engine = Engine::new_identity(42, 4);
    for _ in 0..2 {
        engine.submit(GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: GameAction::NextPhase,
        });
    }
    let planner = BoundedPlanner::new(64).with_deadline_ms(50);
    let pool = PlanPool::new(4);
    let persistent = PlannerPool::new(4, 8);
    let expected = engine.plan_action(0, &planner);
    let mut direct = Vec::new();
    let mut scoped = Vec::new();
    let mut pooled = Vec::new();
    for _ in 0..100 {
        let started = Instant::now();
        let plan = engine.plan_action(0, &planner);
        direct.push(started.elapsed().as_nanos());
        assert_eq!(plan.action, expected.action);
        let started = Instant::now();
        let plans = pool.plan_seats(&engine, &[0], &planner);
        scoped.push(started.elapsed().as_nanos());
        assert_eq!(plans[0].action, expected.action);
        let started = Instant::now();
        let plans = pool.plan_seats_with_pool(&persistent, &engine, &[0], &planner);
        pooled.push(started.elapsed().as_nanos());
        assert_eq!(plans[0].action, expected.action);
    }
    persistent.shutdown();
    println!(
        "{}",
        serde_json::json!({
            "workload": "4-seat identity seed42, first play phase, current seat only, 64 nodes / 50ms",
            "samples": 100, "direct_p50_ns": percentile(&mut direct, 50),
            "scoped_p50_ns": percentile(&mut scoped, 50), "persistent_p50_ns": percentile(&mut pooled, 50),
            "direct_p95_ns": percentile(&mut direct, 95), "scoped_p95_ns": percentile(&mut scoped, 95),
            "persistent_p95_ns": percentile(&mut pooled, 95),
        })
    );
    // Stress the same planner's native clock without changing its candidate policy.
    let mut snapshot = engine.save();
    let cards = std::mem::take(&mut snapshot.core.deck);
    snapshot.core.players[0].hand.extend(cards);
    engine = Engine::restore(&snapshot).unwrap();
    let hand_count = engine.view_for(0).players[0].hand_count;
    let clock = BoundedPlanner::new(usize::MAX).with_deadline_ms(1);
    let started = Instant::now();
    let bounded = engine.plan_action(0, &clock);
    let elapsed = started.elapsed();
    assert!(
        bounded.timed_out,
        "stress workload must actually reach the native deadline"
    );
    assert!(
        elapsed.as_millis() < 1000,
        "native planning did not return promptly"
    );
    println!(
        "{}",
        serde_json::json!({"workload": "native deadline stress: full deck in hand", "hand_count": hand_count, "deadline_ms": 1,
        "elapsed_us": elapsed.as_micros(), "timed_out": bounded.timed_out, "nodes_used": bounded.nodes_used})
    );
}
