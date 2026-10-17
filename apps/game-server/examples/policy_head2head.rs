//! T13 更强候选头对头评估（2026-10-17）：座位 0 用两层搜索（512 节点/
//! 50ms），其余座位用单层启发式（64 节点/50ms）；对照为全部座位单层。
//! 同一组 seed 下比较座位 0 的胜率与平均成本——两层是否带来质量提升。
//!
//! 运行：cargo run --release --offline -p game-server --example policy_head2head -- \
//!         [seeds=24] [max_steps=1200]

#[path = "../src/native.rs"]
mod native;

use engine_native::ai::BoundedPlanner;
use engine_native::Engine;

fn drive(seed: u64, max_steps: u64, two_ply_seat0: bool) -> (bool, u64, i32) {
    let mut e = Engine::new(seed, 4);
    let deep = BoundedPlanner::new(512)
        .with_deadline_ms(50)
        .with_lookahead(2);
    let shallow = BoundedPlanner::new(64).with_deadline_ms(50);
    let mut steps = 0u64;
    let mut since_progress = 0u64;
    while e.outcome().is_none() && steps < max_steps {
        let seat = match e.open_choice() {
            Some(open) => open.observer,
            None => e.turn().current,
        };
        let planner = if two_ply_seat0 && seat == 0 {
            &deep
        } else {
            &shallow
        };
        let plan = e.plan_action(seat, planner);
        match e.apply_plan(&plan) {
            Ok(engine_native::CommandOutcome::Accepted { .. })
            | Ok(engine_native::CommandOutcome::Pending { .. })
            | Ok(engine_native::CommandOutcome::Suspended) => since_progress = 0,
            Ok(engine_native::CommandOutcome::Rejected(_)) => since_progress += 1,
            Err(engine_native::ai::ApplyPlanError::Stale { .. }) => since_progress += 1,
            Err(engine_native::ai::ApplyPlanError::Cancelled) => break,
        }
        steps += 1;
        if since_progress >= 64 {
            break;
        }
    }
    let seat0_win = e
        .outcome()
        .map(|o| matches!(o.winner, engine_native::Winner::Player(0)))
        .unwrap_or(false);
    let hp0 = e.debug_hp(0);
    (seat0_win, steps, hp0)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let seeds: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(24);
    let max_steps: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1200);

    for (label, two_ply) in [("control-all-shallow", false), ("seat0-two-ply", true)] {
        let mut wins = 0u64;
        let mut total_steps = 0u64;
        let mut total_hp = 0i64;
        for seed in 0..seeds {
            let (win, steps, hp0) = drive(8200 + seed, max_steps, two_ply);
            if win {
                wins += 1;
            }
            total_steps += steps;
            total_hp += hp0 as i64;
        }
        println!(
            "arm={label} seeds={seeds} seat0_wins={wins} seat0_win_rate_pct={} avg_steps={} avg_seat0_hp={}",
            wins * 100 / seeds.max(1),
            total_steps / seeds.max(1),
            total_hp / seeds.max(1) as i64,
        );
    }
}
