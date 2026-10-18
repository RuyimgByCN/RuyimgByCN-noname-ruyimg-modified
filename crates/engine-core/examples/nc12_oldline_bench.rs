//! NC12 旧线（`engine-core`）对照基准：建局 / 提交命令 / 快照 / 恢复的墙钟时间。
//!
//! 口径（方案 §NC12）：**旧线与新线分开统计**，同设备同工具链；本基准只测旧线自身的
//! 绝对耗时（p50/p95/max），与 `nc12_bench`（engine-native）并列记录，**不计算跨线收益**。
//!
//! 用法：`cargo run --offline --release -p engine-core --example nc12_oldline_bench -- 30 200`

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;
use std::time::Instant;

fn start_cmd(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
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
        "RESULT scenario=oldline metric={metric} n={} p50_ns={} p95_ns={} max_ns={} mean_ns={}",
        samples.len(),
        pct(samples, 0.5),
        pct(samples, 0.95),
        samples.iter().max().copied().unwrap_or(0),
        mean
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let iters: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(30);
    let steps: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);

    // ① 建局
    let mut build = Vec::with_capacity(iters);
    for i in 0..iters {
        let t = Instant::now();
        let mut e = Engine::new();
        let _ = e.submit(start_cmd(20263100 + i as u64, 4));
        build.push(t.elapsed().as_nanos());
    }
    report("build", &build);

    // ② 提交命令（推进阶段；被拒也计入——旧线语义）
    let mut e = Engine::new();
    let _ = e.submit(start_cmd(20263199, 4));
    let mut submit = Vec::with_capacity(steps);
    for i in 0..steps {
        let cmd = Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(i as u64 + 1),
            kind: CommandKind::EndPhase,
        };
        let t = Instant::now();
        let _ = e.submit(cmd);
        submit.push(t.elapsed().as_nanos());
    }
    report("submit", &submit);

    // ③ 快照 / ④ 恢复
    let mut snap = Vec::with_capacity(iters);
    let mut restore = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        let s = e.snapshot();
        snap.push(t.elapsed().as_nanos());
        let t = Instant::now();
        let _ = s.restore();
        restore.push(t.elapsed().as_nanos());
    }
    report("snapshot", &snap);
    report("restore", &restore);

    println!("NOTE 旧线（engine-core）独立口径；与新线 nc12_bench 并列记录，不跨线计算收益。");
}
