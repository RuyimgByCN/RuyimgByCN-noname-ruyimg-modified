//! Reproducible benchmark runner (architecture plan §38).
//!
//! Runs the fixed scenarios from §38.2 and reports the metrics from §38.3.
//! Results are printed as stable key=value pairs for CI parsing. Per §38.5,
//! each run records CPU/build context so regressions are attributable.
//!
//! Usage:
//!   benchmark-runner                    # run all scenarios
//!   benchmark-runner start-hash         # single scenario
//!   benchmark-runner identity-match     # single scenario
//!   benchmark-runner trigger-dispatch   # 100k trigger fan-outs
//!   benchmark-runner distance-query     # 100k distance queries
//!   benchmark-runner tier-report [tier] # §38.1 device tier budget report

mod device_tiers;

use device_tiers::{render_tier_report, BudgetCheck, DeviceTier, TierBudget, TierReport};
use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId};
use engine_core::{project_observer, query, run_identity_match, Engine, MatchConfig, QueryContext};
use rule_api::QueryRequest;
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// O3：计数分配器——统计分配次数与字节数（仅 benchmark 进程内）。
struct CountingAlloc;

static ALLOC_OPS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_OPS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_OPS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// 分配快照（op, bytes）。
fn alloc_snapshot() -> (u64, u64) {
    (
        ALLOC_OPS.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

#[derive(Debug, Serialize)]
struct BenchReport {
    scenario: &'static str,
    samples: u64,
    total_us: u64,
    p50_ns: u64,
    p99_ns: u64,
    cpu: String,
    build: String,
}

/// 已排序样本的分位数（q ∈ (0,1]，最近邻取值）。
fn percentile(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (((sorted.len() as f64) * q) as usize).min(sorted.len() - 1);
    sorted[idx]
}

fn main() {
    let mut args = env::args().skip(1);
    let scenario = args.next().unwrap_or_else(|| "all".into());
    match scenario.as_str() {
        "all" => {
            bench_start_hash(4, 42, 50);
            bench_start_hash(8, 99, 50);
            bench_identity_match(4, 42, 2000);
            bench_trigger_dispatch(50_000);
            bench_distance_query(50_000);
            bench_query_cache(100_000);
            bench_snapshot_restore(100);
            bench_observer_projection(10_000);
            bench_ai_nodes(2_000);
        }
        "start-hash" => bench_start_hash(4, 42, 200),
        "identity-match" => bench_identity_match(4, 42, 5000),
        "trigger-dispatch" => bench_trigger_dispatch(100_000),
        "distance-query" => bench_distance_query(100_000),
        "snapshot" => bench_snapshot_restore(500),
        "query-cache" => bench_query_cache(100_000),
        "ai-nodes" => bench_ai_nodes(5_000),
        "projection" => bench_observer_projection(50_000),
        "tier-report" => {
            let tier_label = args.next().unwrap_or_else(|| "desktop-native".into());
            let tier = DeviceTier::from_label(&tier_label).unwrap_or_else(|| {
                eprintln!("unknown tier: {tier_label}");
                eprintln!("tiers: android-low-webview | android-mainstream | iphone-ipad | desktop-browser | desktop-native | server-baseline");
                std::process::exit(1);
            });
            run_tier_report(tier);
        }
        "tiers" => {
            // List all tiers.
            println!("=== §38.1 Device Tiers ===");
            for t in DeviceTier::all() {
                let b = TierBudget::for_tier(*t);
                println!("{}: {}", t.label(), t.display_name());
                println!("  start_hash_p50={}us  command_p95={}us  distance_q_p95={}us  frame={}ms  mem={}MB",
                    b.start_hash_p50_us, b.command_p95_us, b.distance_query_p95_us, b.frame_budget_ms, b.memory_budget_mb);
            }
        }
        other => {
            eprintln!("unknown scenario: {other}");
            eprintln!("scenarios: all | start-hash | identity-match | trigger-dispatch | distance-query | query-cache | snapshot | projection | ai-nodes | tier-report [tier] | tiers");
            std::process::exit(1);
        }
    }
}

/// Run benchmarks and check results against a tier's budget (§38.1).
fn run_tier_report(tier: DeviceTier) {
    let budget = TierBudget::for_tier(tier);
    let mut checks = Vec::new();

    // StartGame P50.
    let mut times_ns: Vec<u64> = Vec::with_capacity(50);
    for _ in 0..50 {
        let mut e = Engine::new();
        let s = Instant::now();
        e.submit(start_cmd(42, 4));
        times_ns.push(s.elapsed().as_nanos() as u64);
    }
    times_ns.sort_unstable();
    let p50_us = times_ns[times_ns.len() / 2] / 1000;
    checks.push(BudgetCheck::check(
        "start-hash-p50",
        p50_us,
        budget.start_hash_p50_us,
        "us",
    ));

    // Distance query P95.
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let mut ctx = QueryContext::new();
    for _ in 0..100 {
        let _ = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx,
        );
    }
    let mut q_times: Vec<u64> = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let s = Instant::now();
        let _ = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx,
        );
        q_times.push(s.elapsed().as_nanos() as u64);
    }
    q_times.sort_unstable();
    let p95_idx = (q_times.len() as f64 * 0.95) as usize;
    let q_p95_us = q_times[p95_idx] / 1000;
    checks.push(BudgetCheck::check(
        "distance-query-p95",
        q_p95_us,
        budget.distance_query_p95_us,
        "us",
    ));

    // Snapshot restore.
    let snap = eng.snapshot();
    let mut snap_times: Vec<u64> = Vec::with_capacity(100);
    for _ in 0..100 {
        let s = Instant::now();
        let mut r = Engine::new();
        r.restore_snapshot(&snap).unwrap();
        snap_times.push(s.elapsed().as_nanos() as u64);
    }
    snap_times.sort_unstable();
    let snap_us = snap_times[snap_times.len() / 2] / 1000;
    checks.push(BudgetCheck::check(
        "snapshot-restore",
        snap_us,
        budget.snapshot_restore_us,
        "us",
    ));

    // Observer projection.
    let mut proj_times: Vec<u64> = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let s = Instant::now();
        let _ = project_observer(eng.state(), ObserverKind::Player(p1));
        proj_times.push(s.elapsed().as_nanos() as u64);
    }
    proj_times.sort_unstable();
    let proj_us = proj_times[proj_times.len() / 2] / 1000;
    checks.push(BudgetCheck::check(
        "observer-projection",
        proj_us,
        budget.observer_projection_us,
        "us",
    ));

    let all_passed = checks.iter().all(|c| c.passed);
    let report = TierReport {
        tier: tier.label().into(),
        display_name: tier.display_name().into(),
        checks,
        all_passed,
    };
    print!("{}", render_tier_report(&report));
    if !all_passed {
        std::process::exit(1);
    }
}

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

/// §38.2: StartGame latency (Core metric: Resolution P50/P95).
fn bench_start_hash(players: u8, seed: u64, samples: u64) {
    let mut times_ns: Vec<u64> = Vec::with_capacity(samples as usize);
    // Warmup
    for _ in 0..3 {
        let mut e = Engine::new();
        e.submit(start_cmd(seed, players));
    }
    let t0 = Instant::now();
    for _ in 0..samples {
        let mut e = Engine::new();
        let s = Instant::now();
        e.submit(start_cmd(seed, players));
        times_ns.push(s.elapsed().as_nanos() as u64);
    }
    let total = t0.elapsed().as_micros() as u64;
    times_ns.sort_unstable();
    let p50 = times_ns[times_ns.len() / 2];
    let p99 = percentile(&times_ns, 0.99);
    emit(BenchReport {
        scenario: "start-hash",
        samples,
        total_us: total,
        p50_ns: p50,
        p99_ns: p99,
        cpu: cpu_info(),
        build: engine_api::ENGINE_BUILD_ID.into(),
    });
    println!(
        "scenario=start-hash players={players} seed={seed} samples={samples} \
         p50_ns={p50} p99_ns={p99} total_us={total}"
    );
}

/// §38.2: Identity match end-to-end (Core + Server metrics).
fn bench_identity_match(players: u8, seed: u64, max_steps: u32) {
    let samples = 10u64;
    let mut times_ms: Vec<u128> = Vec::new();
    for _ in 0..samples {
        let s = Instant::now();
        let report = run_identity_match(MatchConfig {
            player_count: players,
            rng_seed: seed,
            max_steps,
            ruleset: "identity-standard".into(),
        })
        .expect("match must complete");
        let elapsed = s.elapsed().as_millis();
        times_ms.push(elapsed);
        assert!(!report.capped, "match must not be capped");
    }
    times_ms.sort_unstable();
    let p50 = times_ms[times_ms.len() / 2];
    println!(
        "scenario=identity-match players={players} seed={seed} max_steps={max_steps} samples={samples} p50_ms={p50}"
    );
}

/// §38.2: Trigger dispatch throughput (10万次 trigger fan-out in plan).
/// Approximated by running EndPhase (which fans through trigger windows).
///
/// 验收口径修正（ADR 0703 "先修正验收口径"）：旧实现以固定
/// `ExpectedRevision(0)`/`IdempotencyKey(0)` 重复提交并**忽略 submit 结果**，
/// revision 前进后几乎全部落入 `StaleRevision` 拒绝路径，却被照常计入
/// "触发吞吐"。实测第二层拒绝源是选择屏障（`illegal_action:choice pending`）。
/// 现改为：跟踪 revision 与幂等键、actor 取当前回合玩家/选择观察者，
/// 遇挂起选择以确定性中性策略自动应答（取前 min_count 个选项 token），
/// 使 EndPhase 真正扇出到触发窗口；吞吐只统计**实际接受**的命令
/// （分 EndPhase / ResolveChoice 两类，P50/P95 覆盖全部接受命令），
/// 拒绝单独按原因计数并记录事件数；接受数为 0 时场景无效、非零码退出。
fn bench_trigger_dispatch(iterations: u64) {
    use std::collections::BTreeMap;

    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    let mut revision = eng.state().revision;
    let mut idempotency: u64 = 0;
    let mut accepted_end_phase: u64 = 0;
    let mut accepted_choice: u64 = 0;
    let mut rejected: u64 = 0;
    let mut events_total: u64 = 0;
    let mut rejected_by: BTreeMap<String, u64> = BTreeMap::new();
    let mut accepted_ns: Vec<u64> = Vec::new();
    let (a0, b0) = alloc_snapshot();
    let t0 = Instant::now();
    for _ in 0..iterations {
        if eng.state().outcome.is_finished() {
            break;
        }
        let (kind, actor) = match eng.last_choice() {
            Some(choice) => {
                let a = choice.observer;
                (auto_answer(choice), a)
            }
            None => {
                let a = eng.state().turn.current;
                if a == PlayerId::NONE {
                    break;
                }
                (CommandKind::EndPhase, a)
            }
        };
        idempotency += 1;
        let is_end_phase = matches!(kind, CommandKind::EndPhase);
        let s = Instant::now();
        let result = eng.submit(Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(idempotency),
            kind,
        });
        let dt = s.elapsed().as_nanos() as u64;
        match result {
            CommandResult::Accepted { new_revision, .. } => {
                if is_end_phase {
                    accepted_end_phase += 1;
                } else {
                    accepted_choice += 1;
                }
                revision = new_revision;
                events_total += eng.last_events().len() as u64;
                accepted_ns.push(dt);
            }
            CommandResult::Rejected { reason } => {
                rejected += 1;
                *rejected_by.entry(reject_key(&reason)).or_insert(0) += 1;
            }
        }
    }
    let elapsed = t0.elapsed();
    let accepted = accepted_end_phase + accepted_choice;
    if accepted == 0 {
        eprintln!(
            "scenario=trigger-dispatch INVALID: accepted=0 rejected={rejected} \
             rejected_by={rejected_by:?} — throughput not measurable"
        );
        std::process::exit(1);
    }
    accepted_ns.sort_unstable();
    let p50 = percentile(&accepted_ns, 0.50);
    let p95 = percentile(&accepted_ns, 0.95);
    let p99 = percentile(&accepted_ns, 0.99);
    let per_op_ns = elapsed.as_nanos() / (accepted + rejected).max(1) as u128;
    let (a1, b1) = alloc_snapshot();
    let a_ops = (a1 - a0) / (accepted + rejected).max(1);
    let a_bytes = (b1 - b0) / (accepted + rejected).max(1);
    // 终局条件：驱动结束时对局是否终局、结果与回合数（§3 验收口径）。
    let outcome = eng.state().outcome.clone();
    let finished = outcome.is_finished();
    println!(
        "scenario=trigger-dispatch iterations={iterations} accepted={accepted} \
         accepted_end_phase={accepted_end_phase} accepted_choice={accepted_choice} \
         rejected={rejected} events={events_total} p50_ns={p50} p95_ns={p95} p99_ns={p99} \
         per_op_ns={per_op_ns} total_ms={} finished={finished} outcome={outcome:?} \
         rounds={} rejected_by={rejected_by:?} allocs_delta={} bytes_delta={} allocs_per_op={} bytes_per_op={}",
        elapsed.as_millis(),
        eng.state().turn.round,
        a1 - a0,
        b1 - b0,
        a_ops,
        a_bytes
    );
}

/// 确定性中性自动应答：取前 min_count 个选项 token；无选项时按
/// allow_cancel 取消或提交空选择（constraint-only）。
fn auto_answer(choice: &engine_api::ChoiceRequest) -> CommandKind {
    if choice.options.is_empty() {
        return CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![],
            cancelled: choice.allow_cancel,
        };
    }
    let n = choice.min_count.max(1) as usize;
    let selections = choice.options.iter().take(n).map(|o| o.token).collect();
    CommandKind::ResolveChoice {
        choice_id: choice.choice_id,
        selections,
        cancelled: false,
    }
}

/// Short stable key for reject-reason histogram (never leaks payloads).
fn reject_key(reason: &engine_api::RejectReason) -> String {
    use engine_api::RejectReason as R;
    match reason {
        R::InvalidActor => "invalid_actor".into(),
        R::StaleRevision { .. } => "stale_revision".into(),
        R::DuplicateIdempotency => "duplicate_idempotency".into(),
        R::IllegalAction { code } => format!("illegal_action:{code}"),
        R::InvalidChoice => "invalid_choice".into(),
        R::MatchOver => "match_over".into(),
        R::NotStarted => "not_started".into(),
        R::Fault { kind, .. } => format!("fault:{kind:?}"),
    }
}

/// §38.2: Distance query throughput (10万次 distance Query).
fn bench_distance_query(iterations: u64) {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let mut ctx = QueryContext::new();
    // Warmup
    for _ in 0..100 {
        let _ = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx,
        );
    }
    let (a0, b0) = alloc_snapshot();
    let t0 = Instant::now();
    for _ in 0..iterations {
        let _ = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx,
        );
    }
    let (a1, b1) = alloc_snapshot();
    let a_ops = (a1 - a0) / iterations;
    let a_bytes = (b1 - b0) / iterations;
    let elapsed = t0.elapsed();
    let per_q_ns = elapsed.as_nanos() / iterations as u128;
    println!(
        "scenario=distance-query iterations={iterations} per_query_ns={per_q_ns} total_ms={} allocs_delta={} bytes_delta={} allocs_per_op={} bytes_per_op={}",
        elapsed.as_millis(),
        a1 - a0,
        b1 - b0,
        a_ops,
        a_bytes
    );
}

/// O3：query 缓存命中率（同键重复查询——无提交边界时键不失效）。
fn bench_query_cache(iterations: u64) {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let req = QueryRequest::Distance { from: p1, to: p2 };
    let mut ctx = QueryContext::with_cache(true);
    // 冷段：首查写缓存
    let t0 = Instant::now();
    for _ in 0..100 {
        let _ = query(eng.state(), &req, &mut ctx);
    }
    let cold_ns = t0.elapsed().as_nanos() / 100;
    let hits_before = ctx.cache_hits();
    // 热段：同键查询应全部命中
    let t0 = Instant::now();
    for _ in 0..iterations {
        let _ = query(eng.state(), &req, &mut ctx);
    }
    let warm_ns = t0.elapsed().as_nanos() / iterations as u128;
    let hits = ctx.cache_hits() - hits_before;
    let misses = iterations.saturating_sub(hits);
    let rate_pct = hits * 100 / iterations.max(1);
    println!(
        "scenario=query-cache iterations={iterations} cold_ns={cold_ns} warm_ns={warm_ns} hits={hits} misses={misses} hit_rate_pct={rate_pct}"
    );
}

/// O3：AI 节点成本（旧线 engine-ai 束搜索——nodes_used 与 ns/node）。
fn bench_ai_nodes(iterations: u64) {
    use engine_ai::{BeamSearch, SearchBudget, SearchPolicy, StateFork};
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 5));
    let actor = eng.state().turn.current;
    let mut search = BeamSearch::default();
    let budget = SearchBudget {
        max_nodes: 10_000,
        max_depth: 8,
    };
    let mut total_nodes = 0u64;
    let t0 = Instant::now();
    for _ in 0..iterations {
        let mut fork = StateFork::from_authority(&eng, 99);
        let r = search.select(&mut fork, actor, budget);
        total_nodes += r.nodes_used;
    }
    let ns = t0.elapsed().as_nanos();
    let ns_per_node = if total_nodes > 0 {
        ns / total_nodes as u128
    } else {
        0
    };
    println!(
        "scenario=ai-nodes iterations={iterations} nodes_total={total_nodes} ns_per_node={ns_per_node} total_ms={}",
        ns / 1_000_000
    );
}

/// §38.2: Snapshot capture + restore latency (Replay metric).
fn bench_snapshot_restore(iterations: u64) {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 4));
    // 预热对局：同样跟踪 revision/幂等键/actor，保证 5 条 EndPhase 真正被接受。
    let mut revision = eng.state().revision;
    let mut idempotency: u64 = 0;
    for _ in 0..5 {
        let actor = eng.state().turn.current;
        if actor == PlayerId::NONE || eng.state().outcome.is_finished() {
            break;
        }
        idempotency += 1;
        if let CommandResult::Accepted { new_revision, .. } = eng.submit(Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(idempotency),
            kind: CommandKind::EndPhase,
        }) {
            revision = new_revision;
        }
    }
    let t0 = Instant::now();
    for _ in 0..iterations {
        let snap = eng.snapshot();
        let mut restored = Engine::new();
        restored.restore_snapshot(&snap).unwrap();
    }
    let elapsed = t0.elapsed();
    let per_op_ns = elapsed.as_nanos() / iterations as u128;
    println!(
        "scenario=snapshot-restore iterations={iterations} per_op_ns={per_op_ns} total_ms={}",
        elapsed.as_millis()
    );
}

/// §38.2: Observer projection throughput (Client metric: ObserverDelta size).
fn bench_observer_projection(iterations: u64) {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 8));
    let p1 = PlayerId::new(1);
    // 扇出：单次投影产出的座位+已知牌条目数（O3 project_observer 扇出面）。
    let sample = project_observer(eng.state(), ObserverKind::Player(p1));
    let fanout = sample.seats.len() + sample.known_cards.len();
    let (a0, b0) = alloc_snapshot();
    let t0 = Instant::now();
    for _ in 0..iterations {
        let _ = project_observer(eng.state(), ObserverKind::Player(p1));
    }
    let (a1, b1) = alloc_snapshot();
    let a_ops = (a1 - a0) / iterations;
    let a_bytes = (b1 - b0) / iterations;
    let elapsed = t0.elapsed();
    let per_op_ns = elapsed.as_nanos() / iterations as u128;
    println!(
        "scenario=observer-projection iterations={iterations} per_op_ns={per_op_ns} total_ms={} fanout_targets={fanout} allocs_delta={} bytes_delta={} allocs_per_op={} bytes_per_op={}",
        elapsed.as_millis(),
        a1 - a0,
        b1 - b0,
        a_ops,
        a_bytes
    );
}

fn emit(report: BenchReport) {
    // Stable JSON for CI artifact ingestion.
    if let Ok(json) = serde_json::to_string(&report) {
        eprintln!("{json}");
    }
}

fn cpu_info() -> String {
    // Best-effort; not security-sensitive.
    std::env::consts::ARCH.to_string()
}
