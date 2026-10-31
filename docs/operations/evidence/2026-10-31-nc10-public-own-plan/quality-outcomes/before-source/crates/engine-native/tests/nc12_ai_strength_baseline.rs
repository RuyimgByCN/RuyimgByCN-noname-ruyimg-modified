//! NC12/N10 AI 强度基线：策略对局长度、终局率与故障率的**统计口径**。
//!
//! 背景：2026-10-19 遮蔽填充池收紧（公开区/在飞区不再进入填充池）后，
//! `ai_planner::policy_v2_drives_all_seats_without_answer_routing_panic`（seed 8211）
//! 步数由 **179 → 484**。本用例给出**同参数下的统计基线**（多 seed 的步数分布与终局率），
//! 供后续判断该变化是否可接受、以及填充策略是否需要重新设计。
//!
//! 说明：本用例**只统计不断言步数**（步数分布是策略面观测，不是规则正确性）；若命中
//! 内核悬空引用故障（`Plan.sim_faults` / `legal_actions::probe_faults()`）则**失败**并打印。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

#[test]
fn nc12_ai_strength_baseline_statistics() {
    let planner = BoundedPlanner::new(8)
        .with_deadline_ms(50)
        .with_lookahead(1)
        .with_policy_variant(0);
    let seeds: Vec<u64> = (45000..45200).collect();
    let mut steps_all: Vec<usize> = Vec::new();
    let mut finished = 0usize;
    let mut faults = 0u64;
    for seed in seeds {
        let mut e = swd_engine(seed);
        let probe_before = engine_native::engine::legal_actions::probe_faults();
        let mut steps = 0usize;
        while e.outcome().is_none() && steps < 1200 {
            let seat = match e.open_choice() {
                Some(open) => open.observer,
                None => e.turn().current,
            };
            let plan = e.plan_action(seat, &planner);
            faults += plan.sim_faults as u64;
            if plan.sim_faults > 0 {
                break; // 命中故障：本局终止统计（故障由断言报告）
            }
            let action = plan.action;
            let _ = e.submit(GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action,
            });
            steps += 1;
        }
        faults += engine_native::engine::legal_actions::probe_faults() - probe_before;
        steps_all.push(steps);
        if e.outcome().is_some() {
            finished += 1;
        }
    }
    steps_all.sort_unstable();
    let n = steps_all.len();
    let pct = |p: f64| steps_all[((n as f64 - 1.0) * p).round() as usize];
    eprintln!(
        "AI_BASELINE n={n} finished={finished} steps_p50={} steps_p90={} steps_max={} faults={faults}",
        pct(0.5),
        pct(0.9),
        steps_all[n - 1]
    );
    assert_eq!(
        faults, 0,
        "统计期间命中内核悬空引用故障（详见 nc12_dangling_reference_repro）"
    );
    assert!(
        finished * 2 >= n,
        "终局率过低：{finished}/{n}（策略面观测，低于半数视为异常）"
    );
}

/// Reproducible strategy comparison; run explicitly with --ignored --nocapture --test-threads=1.
/// Latency is diagnostic unless the host is otherwise idle. Percentiles use nearest rank.
#[test]
#[ignore = "manual fixed-seed quality and latency matrix"]
fn nc10_quality_matrix() {
    quality_matrix(8);
}

#[test]
#[ignore = "manual production-budget quality and latency matrix"]
fn nc10_quality_matrix_production() {
    quality_matrix(64);
}

fn quality_matrix(node_budget: usize) {
    use engine_native::{CommandOutcome, Role, Winner};
    use std::time::Instant;
    fn percentile(values: &mut [u128], percentile: usize) -> u128 {
        values.sort_unstable();
        values[(values.len() * percentile).div_ceil(100).saturating_sub(1)]
    }
    let mut matrix_errors = 0;
    for seats in [4, 8] {
        for identity in [false, true] {
            for skills in [false, true] {
                for (variant, depth) in [(0, 1), (1, 1), (1, 2)] {
                    let mut latencies = Vec::new();
                    let (mut games, mut finished, mut wins, mut rejected, mut faults) =
                        (0, 0, 0, 0, 0);
                    let (mut nodes, mut exhausted, mut leaves, mut steps) = (0, 0, 0, 0);
                    for seed in [45000, 45001] {
                        for challenger in 0..seats {
                            let mut e = if identity {
                                Engine::new_identity(seed, seats)
                            } else {
                                Engine::new(seed, seats)
                            };
                            if skills {
                                for seat in 0..seats {
                                    e.assign_general_named(
                                        seat as u8,
                                        ["曹操", "刘备", "孙权", "赵云"][seat % 4],
                                    )
                                    .unwrap();
                                }
                            }
                            let role = e.roles().map(|r| r[challenger]);
                            let probe_before = engine_native::engine::legal_actions::probe_faults();
                            let baseline = BoundedPlanner::new(node_budget).with_policy_variant(0);
                            let candidate = BoundedPlanner::new(node_budget)
                                .with_policy_variant(variant)
                                .with_lookahead(depth);
                            let mut game_steps = 0;
                            while e.outcome().is_none() && game_steps < 800 {
                                let actor = e
                                    .open_choice()
                                    .map(|c| c.observer)
                                    .unwrap_or(e.turn().current);
                                let planner = if actor as usize == challenger {
                                    &candidate
                                } else {
                                    &baseline
                                };
                                let started = Instant::now();
                                let plan = e.plan_action(actor, planner);
                                latencies.push(started.elapsed().as_nanos());
                                faults += plan.sim_faults as u64;
                                if plan.sim_faults > 0 {
                                    eprintln!("QUALITY_FAULT seed={seed} challenger={challenger} step={game_steps} context={:?}", engine_native::ai::sim_fault_log());
                                    engine_native::ai::clear_sim_fault_log();
                                }
                                nodes += plan.nodes_used;
                                exhausted += usize::from(plan.budget_exhausted);
                                leaves += usize::from(plan.nodes_used == 0);
                                let result = e.apply_plan(&plan).unwrap();
                                rejected +=
                                    usize::from(matches!(result, CommandOutcome::Rejected(_)));
                                game_steps += 1;
                                if matches!(result, CommandOutcome::Rejected(_)) {
                                    eprintln!("QUALITY_REJECT seats={seats} seed={seed} actor={actor} plan={plan:?} choice={:?} result={result:?}", e.open_choice());
                                    break;
                                }
                            }
                            faults +=
                                engine_native::engine::legal_actions::probe_faults() - probe_before;
                            games += 1;
                            steps += game_steps;
                            if let Some(outcome) = e.outcome() {
                                finished += 1;
                                wins += usize::from(match outcome.winner {
                                    Winner::Player(seat) => seat as usize == challenger,
                                    Winner::LordSide => {
                                        matches!(role, Some(Role::Lord | Role::Loyalist))
                                    }
                                    Winner::Rebels => role == Some(Role::Rebel),
                                    Winner::Traitor => role == Some(Role::Traitor),
                                    _ => false,
                                });
                            }
                            eprintln!("QUALITY_GAME budget={node_budget} seats={seats} identity={identity} skills={skills} variant={variant} depth={depth} seed={seed} challenger={challenger} role={role:?} steps={game_steps} winner={:?}", e.outcome());
                        }
                    }
                    let p50 = percentile(&mut latencies, 50);
                    let p95 = percentile(&mut latencies, 95);
                    let p99 = percentile(&mut latencies, 99);
                    eprintln!("QUALITY_SUMMARY budget={node_budget} seats={seats} identity={identity} skills={skills} variant={variant} depth={depth} games={games} finished={finished} challenger_wins={wins} steps={steps} samples={} rejected={rejected} faults={faults} nodes={nodes} exhausted={exhausted} zero_node={leaves} p50_ns={p50} p95_ns={p95} p99_ns={p99}", latencies.len());
                    matrix_errors += rejected + faults as usize;
                }
            }
        }
    }
    assert_eq!(
        matrix_errors, 0,
        "quality matrix rejected plans or simulation faults"
    );
}
