//! T13 策略质量候选评估（2026-10-14）：同一组 seed 上对比「BoundedPlanner
//! 启发式（现状，nodes=64）」与「NextPhase 基线（nodes=0——候选预算即
//! 耗尽，动作退化为兜底 NextPhase）」的驱动结果：终局率/步数分布/耗时/
//! 胜方分布。seed 固定可重复；key=value 输出供证据留存。
//!
//! 运行：cargo run --release --offline -p game-server --example policy_eval -- \
//!         [seeds=10] [max_steps=1200]

#[path = "../src/native.rs"]
mod native;

use native::{NativeContent, NativeGameRequest};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
enum Policy {
    /// 现状：BoundedPlanner 启发式规划（64 节点/50ms）。
    Planner,
    /// 更强候选：两层搜索（512 节点/50ms）——每个候选后再模拟对手最优应对。
    TwoPly,
    /// 更强候选：策略变体 1（64 节点/50ms）——全手牌候选 + 牌值/存活评分。
    StrongV2,
    /// 基线：nodes=0——不做任何规划，动作退化为 NextPhase 兜底。
    NextPhaseOnly,
}

impl Policy {
    fn label(self) -> &'static str {
        match self {
            Policy::Planner => "planner-heuristic",
            Policy::TwoPly => "two-ply-512",
            Policy::StrongV2 => "strong-v2-64",
            Policy::NextPhaseOnly => "nextphase-baseline",
        }
    }

    fn planner_nodes(self) -> usize {
        match self {
            Policy::Planner | Policy::StrongV2 => 64,
            Policy::TwoPly => 512,
            Policy::NextPhaseOnly => 0,
        }
    }

    fn lookahead(self) -> u8 {
        match self {
            Policy::TwoPly => 2,
            _ => 1,
        }
    }

    fn variant(self) -> u8 {
        match self {
            Policy::StrongV2 => 1,
            _ => 0,
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let seeds: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(10);
    let max_steps: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1200);

    let packages_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let store = std::env::temp_dir().join("t13-policy-eval-store");
    let content = NativeContent::open(packages_root, store).expect("open content");
    let _ = content.import();

    for policy in [
        Policy::Planner,
        Policy::TwoPly,
        Policy::StrongV2,
        Policy::NextPhaseOnly,
    ] {
        let mut finished = 0u64;
        let mut total_steps = 0u64;
        let mut total_ms = 0u128;
        let mut stalled_runs = 0u64;
        let mut winners: Vec<String> = Vec::new();
        for seed in 0..seeds {
            let seed = 8100 + seed;
            let t0 = std::time::Instant::now();
            let req = NativeGameRequest {
                packages: vec![],
                seed,
                seats: 4,
                max_steps,
                planner_nodes: policy.planner_nodes(),
                planner_lookahead: policy.lookahead(),
                planner_variant: policy.variant(),
            };
            let summary = native::create_native_game(&content, &req).expect("驱动");
            let ms = t0.elapsed().as_millis();
            total_ms += ms;
            total_steps += summary.steps;
            if summary.finished {
                finished += 1;
            }
            if summary.stalled {
                stalled_runs += 1;
            }
            if let Some(w) = &summary.winner {
                winners.push(w.clone());
            }
            println!(
                "policy={} seed={seed} finished={} steps={} elapsed_ms={ms} winner={:?} stalled={}",
                policy.label(),
                summary.finished,
                summary.steps,
                summary.winner,
                summary.stalled
            );
        }
        let mut hist: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
        for w in &winners {
            *hist.entry(w.clone()).or_insert(0) += 1;
        }
        println!(
            "policy={} SUMMARY runs={seeds} finished={finished} finish_rate_pct={} avg_steps={} avg_ms={} stalled={stalled_runs} winner_hist={hist:?}",
            policy.label(),
            finished * 100 / seeds.max(1) as u64,
            total_steps / seeds.max(1) as u64,
            total_ms / seeds.max(1) as u128,
        );
    }
}
