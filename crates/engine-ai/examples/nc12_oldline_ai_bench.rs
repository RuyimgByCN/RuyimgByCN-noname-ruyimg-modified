//! NC12 旧线 AI（`engine-ai`）对照基准：信息集构建 / 合法动作 / 三种搜索的墙钟时间。
//!
//! 口径（方案 §NC12）：旧线与新线**分开统计**；本基准只测旧线 AI 自身耗时，与
//! `nc12_bench ai`（engine-native）并列记录，**不跨线计算收益**。
//!
//! 用法：`cargo run --offline --release -p engine-ai --example nc12_oldline_ai_bench -- 20 32`

use engine_ai::{
    build_information_set, legal_actions, AiRuntime, BeamSearch, HeuristicSearch, MctsSearch,
    SearchBudget,
};
use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;
use std::time::Instant;

fn start(seed: u64) -> Engine {
    let mut eng = Engine::new();
    let _ = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    eng
}

fn pct(samples: &[u128], p: f64) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    let mut v = samples.to_vec();
    v.sort_unstable();
    v[((v.len() as f64 - 1.0) * p).round() as usize]
}

fn report(metric: &str, samples: &[u128]) {
    let mean = samples.iter().sum::<u128>() / samples.len().max(1) as u128;
    println!(
        "RESULT scenario=oldline_ai metric={metric} n={} p50_ns={} p95_ns={} max_ns={} mean_ns={}",
        samples.len(),
        pct(samples, 0.5),
        pct(samples, 0.95),
        samples.iter().max().copied().unwrap_or(0),
        mean
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let iters: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(20);
    let nodes: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(32);

    let eng = start(20263200);
    let actor = eng.state().turn.current;

    let mut info = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        let _ = build_information_set(&eng, actor);
        info.push(t.elapsed().as_nanos());
    }
    report("information_set", &info);

    let mut legal = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        let _ = legal_actions(&eng, actor);
        legal.push(t.elapsed().as_nanos());
    }
    report("legal_actions", &legal);

    let budget = SearchBudget {
        max_nodes: nodes,
        max_depth: 3,
    };
    let rt = AiRuntime::new(20263200);
    let mut heuristic = HeuristicSearch;
    let mut beam = BeamSearch::default();
    let mut mcts = MctsSearch::default();
    for name in ["heuristic", "beam", "mcts"] {
        let mut samples = Vec::with_capacity(iters);
        for _ in 0..iters {
            let t = Instant::now();
            let _ = match name {
                "heuristic" => rt.decide(&eng, actor, &mut heuristic, budget),
                "beam" => rt.decide(&eng, actor, &mut beam, budget),
                _ => rt.decide(&eng, actor, &mut mcts, budget),
            };
            samples.push(t.elapsed().as_nanos());
        }
        report(name, &samples);
    }

    println!("NOTE 旧线 AI（engine-ai）独立口径；与新线 nc12_bench ai 并列记录，不跨线计算收益。");
}
