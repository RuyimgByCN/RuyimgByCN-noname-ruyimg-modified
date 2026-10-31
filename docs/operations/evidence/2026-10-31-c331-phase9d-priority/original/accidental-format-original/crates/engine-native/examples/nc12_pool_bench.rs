//! NC12 前后对比：`PlanPool::plan_seats`（长驻有界池）vs `plan_seats_scoped`
//! （逐调用 `std::thread::scope`）。
//!
//! 口径（方案 §NC12）：**同设备、同工具链、同负载**；两路径在同一进程内交替采样，
//! 报告 p50/p95/max 与线程创建次数差（后者为结构性差异，非时间收益）。
//!
//! 用法（在 noname-next/ 根目录）：
//! ```bash
//! cargo run --offline --release -p engine-native --example nc12_pool_bench -- \
//!     tier=swd seats=4 iters=30 budget=32
//! ```

use engine_native::ai::{BoundedPlanner, PlanPool};
use engine_native::Engine;
use std::time::Instant;

fn arg(args: &[String], key: &str, default: &str) -> String {
    args.iter()
        .find_map(|a| a.strip_prefix(&format!("{key}=")).map(|v| v.to_string()))
        .unwrap_or_else(|| default.to_string())
}

fn tier_pack(tier: &str) -> Option<engine_native::content::extension::ExtensionPack> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = match tier {
        "none" => return None,
        "swd" => "shahai-swd-upstream-noname",
        "huanle" => "huanle-upstream-noname",
        other => panic!("未知 tier: {other}（none|swd|huanle）"),
    };
    let json = std::fs::read_to_string(root.join(dir).join("assets/cards.json")).expect("读包");
    Some(
        content_native_loader::bridge_pack(&json)
            .expect("桥接")
            .pack,
    )
}

fn pct(samples: &[u128], p: f64) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    let mut v = samples.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tier = arg(&args, "tier", "swd");
    let seats: u8 = arg(&args, "seats", "4").parse().expect("seats");
    let iters: usize = arg(&args, "iters", "30").parse().expect("iters");
    let budget: usize = arg(&args, "budget", "32").parse().expect("budget");

    let pack = tier_pack(&tier);
    let engine = match &pack {
        Some(p) => Engine::new_with_extensions(20262101, seats as usize, &[p]).expect("建局"),
        None => Engine::new(20262101, seats as usize),
    };
    let planner = BoundedPlanner::new(budget).with_deadline_ms(50);
    let pool = PlanPool::new(seats as usize);
    let all_seats: Vec<u8> = (0..seats).collect();
    // 跨调用复用的池（NC10 设计意图：线程只创建一次）
    let reused = engine_native::ai_pool::PlannerPool::new(seats as usize, seats as usize);

    // 预热（避免首轮页错误/线程创建污染）
    let _ = pool.plan_seats_pooled(&engine, &all_seats, &planner);
    let _ = pool.plan_seats_scoped(&engine, &all_seats, &planner);
    let _ = pool.plan_seats_with_pool(&reused, &engine, &all_seats, &planner);

    let mut pooled: Vec<u128> = Vec::with_capacity(iters);
    let mut scoped: Vec<u128> = Vec::with_capacity(iters);
    let mut reused_ns: Vec<u128> = Vec::with_capacity(iters);
    for i in 0..iters {
        // 交替顺序，避免系统性顺序偏差
        if i % 2 == 0 {
            let t = Instant::now();
            let _ = pool.plan_seats_pooled(&engine, &all_seats, &planner);
            pooled.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = pool.plan_seats_scoped(&engine, &all_seats, &planner);
            scoped.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = pool.plan_seats_with_pool(&reused, &engine, &all_seats, &planner);
            reused_ns.push(t.elapsed().as_nanos());
        } else {
            let t = Instant::now();
            let _ = pool.plan_seats_with_pool(&reused, &engine, &all_seats, &planner);
            reused_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = pool.plan_seats_scoped(&engine, &all_seats, &planner);
            scoped.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = pool.plan_seats_pooled(&engine, &all_seats, &planner);
            pooled.push(t.elapsed().as_nanos());
        }
    }

    println!(
        "RESULT scenario=pool_bench tier={tier} config=seats={seats},iters={iters},budget={budget}"
    );
    for (name, s) in [
        ("scoped", &scoped),
        ("pooled_per_call", &pooled),
        ("pooled_reused", &reused_ns),
    ] {
        println!(
            "RESULT scenario=pool_bench tier={tier} path={name} n={} p50_ns={} p95_ns={} max_ns={} mean_ns={}",
            s.len(),
            pct(s, 0.50),
            pct(s, 0.95),
            s.iter().max().copied().unwrap_or(0),
            s.iter().sum::<u128>() / s.len().max(1) as u128,
        );
    }
    println!(
        "RESULT scenario=pool_bench tier={tier} threads: scoped={seats}/call pooled_per_call={seats}/call pooled_reused=0/call（池只建一次）"
    );
    println!(
        "NOTE 同设备同负载前后对比；两路径在同一进程交替采样，p50 可比较，绝对值受并发负载影响。"
    );
}
