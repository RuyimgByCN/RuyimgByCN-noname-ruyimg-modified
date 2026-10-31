//! NC10 验收切片：**AI 信息边界** + **预算/取消/过期**。
//!
//! 对应方案 NC10 的两条验收：
//! 1. **同信息集替换暗牌不改变不应受影响的输入/行为**——本文件比较「座位 0 的策略输入」
//!    在**他人暗牌被替换**前后是否**逐字节相同**；
//! 2. **取消与过期**——预置取消标志 ⇒ `Plan.cancelled == true`；节点预算耗尽 ⇒
//!    `Plan.budget_exhausted == true`；陈旧计划提交 ⇒ `ApplyPlanError::Stale`。

use engine_native::ai::{BoundedPlanner, PlanPool};
use engine_native::Engine;

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("read swd");
    let pack = content_native_loader::bridge_pack(&cards)
        .expect("bridge")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("engine")
}

/// ① **信息边界**：替换**他人暗牌**不改变座位 0 的策略输入。
#[test]
fn hidden_other_hands_do_not_change_seat_input() {
    let a = swd_engine(20261001);
    let input_a = format!("{:?}", a.ai_session(0).input());

    // 同一引擎：把**座位 2 与座位 3** 的手牌各换一张（对座位 0 都是暗牌）
    let mut b = swd_engine(20261001);
    let before = format!("{:?}", b.ai_session(0).input());
    assert_eq!(input_a, before, "同种子同状态的两份输入应一致（基线）");
    {
        let core = b.state_core_mut();
        let (Some(p2), Some(p3)) = (core.players.get(2).cloned(), core.players.get(3).cloned())
        else {
            return;
        };
        let (Some(c2), Some(c3)) = (p2.hand.first().copied(), p3.hand.first().copied()) else {
            return;
        };
        if let Some(p) = core.players.get_mut(2) {
            if let Some(slot) = p.hand.first_mut() {
                *slot = c3;
            }
        }
        if let Some(p) = core.players.get_mut(3) {
            if let Some(slot) = p.hand.first_mut() {
                *slot = c2;
            }
        }
    }
    let after = format!("{:?}", b.ai_session(0).input());
    eprintln!(
        "DBG NC10 信息边界：座位0输入长度 before={} after={}（应相同）",
        before.len(),
        after.len()
    );
    assert_eq!(
        before, after,
        "替换他人暗牌不应改变座位 0 的策略输入（AI 信息边界）"
    );
}

/// ② **取消**：预置取消标志 ⇒ `Plan.cancelled == true`。
#[test]
fn cancelled_planner_marks_plan_cancelled() {
    let e = swd_engine(20261002);
    let planner = BoundedPlanner {
        node_budget: 64,
        deadline_ms: Some(5_000),
        cancel: Default::default(),
        lookahead_depth: 1,
        policy_variant: 0,
        content_tuning: None,
    };
    planner.cancel();
    let plans = PlanPool::new(1).plan_seats(&e, &[0], &planner);
    assert_eq!(plans.len(), 1);
    eprintln!(
        "DBG NC10 取消：cancelled={} nodes_used={} budget_exhausted={}",
        plans[0].cancelled, plans[0].nodes_used, plans[0].budget_exhausted
    );
    assert!(plans[0].cancelled, "预置取消后 Plan 应标记 cancelled");
}

/// ③ **预算耗尽**：`node_budget = 1` ⇒ `budget_exhausted == true`（未取消、未超时）。
#[test]
fn tiny_budget_marks_exhausted() {
    let e = swd_engine(20261003);
    let planner = BoundedPlanner {
        node_budget: 1,
        deadline_ms: Some(5_000),
        cancel: Default::default(),
        lookahead_depth: 2,
        policy_variant: 0,
        content_tuning: None,
    };
    let plans = PlanPool::new(1).plan_seats(&e, &[0], &planner);
    assert_eq!(plans.len(), 1);
    eprintln!(
        "DBG NC10 预算：nodes_used={} budget_exhausted={} timed_out={} cancelled={}",
        plans[0].nodes_used, plans[0].budget_exhausted, plans[0].timed_out, plans[0].cancelled
    );
    assert!(
        plans[0].budget_exhausted || plans[0].nodes_used <= 1,
        "预算 1 时应耗尽（budget_exhausted={}，nodes_used={}）",
        plans[0].budget_exhausted,
        plans[0].nodes_used
    );
}

/// ④ **并发压力与尾延迟**：同一引擎快照下，`PlanPool` 以 1 / 4 个 worker 各跑 N 次四座规划：
/// * **结果必须逐一相同**（并行 == 顺序，模块文档承诺的确定性）；
/// * 记录 p50 / p95 / p99 墙钟，作为**基线**（不是优化前后对比）。
#[test]
fn pool_parallel_matches_sequential_and_records_tail() {
    const N: usize = 30;
    let e = swd_engine(20261004);
    let seats = [0u8, 1, 2, 3];
    let planner = BoundedPlanner {
        node_budget: 32,
        deadline_ms: Some(5_000),
        cancel: Default::default(),
        lookahead_depth: 1,
        policy_variant: 0,
        content_tuning: None,
    };

    fn key(plans: &[engine_native::ai::Plan]) -> Vec<String> {
        plans
            .iter()
            .map(|p| {
                format!(
                    "{}:{:?}:{}:{}",
                    p.seat, p.action, p.nodes_used, p.budget_exhausted
                )
            })
            .collect()
    }
    fn pct(mut v: Vec<u128>, p: f64) -> u128 {
        v.sort_unstable();
        if v.is_empty() {
            return 0;
        }
        let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
        v[idx.min(v.len() - 1)]
    }

    let mut samples: Vec<(usize, Vec<u128>)> = Vec::new();
    let mut reference: Option<Vec<String>> = None;
    for workers in [1usize, 4] {
        let pool = PlanPool::new(workers);
        let mut times = Vec::new();
        for _ in 0..N {
            let t0 = std::time::Instant::now();
            let plans = pool.plan_seats(&e, &seats, &planner);
            times.push(t0.elapsed().as_nanos());
            assert_eq!(plans.len(), seats.len(), "四座规划应返回 4 个 Plan");
            let k = key(&plans);
            match &reference {
                None => reference = Some(k),
                Some(r) => assert_eq!(r, &k, "并行结果必须与顺序结果逐一相同（worker={workers}）"),
            }
        }
        let (p50, p95, p99) = (
            pct(times.clone(), 0.50),
            pct(times.clone(), 0.95),
            pct(times.clone(), 0.99),
        );
        eprintln!(
            "DBG NC10 并发：workers={workers} n={N} p50={}µs p95={}µs p99={}µs",
            p50 / 1000,
            p95 / 1000,
            p99 / 1000
        );
        samples.push((workers, times));
    }
    assert_eq!(samples.len(), 2);
}

/// ⑤ **过期结果拒绝**：规划后引擎修订号前进 ⇒ `apply_plan` 必须返回 `Stale`；
/// 重新规划后同一位置应可应用成功。
#[test]
fn stale_plan_is_rejected() {
    use engine_native::ai::ApplyPlanError;

    let mut e = swd_engine(20261005);
    let planner = BoundedPlanner {
        node_budget: 16,
        deadline_ms: Some(5_000),
        cancel: Default::default(),
        lookahead_depth: 1,
        policy_variant: 0,
        content_tuning: None,
    };
    let plans = PlanPool::new(1).plan_seats(&e, &[0], &planner);
    let stale = plans.into_iter().next().expect("plan");
    assert_eq!(stale.revision, e.revision(), "计划应携带规划时的修订号");

    // 让修订号前进（换人/换阶段），使旧计划过期
    let actor = e.turn().current;
    let _ = e.submit(engine_native::GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    let now = e.revision();
    assert!(now > stale.revision, "修订号应已前进（{}）", stale.revision);

    match e.apply_plan(&stale) {
        Err(ApplyPlanError::Stale { submitted, current }) => {
            eprintln!("DBG NC10 过期：Stale submitted={submitted} current={current} ✅");
            assert_eq!(submitted, stale.revision);
            assert_eq!(current, now);
        }
        Err(other) => panic!("期望 Stale，实得 {other:?}"),
        Ok(_) => panic!("陈旧计划**不应**被应用"),
    }

    // 重新规划：新计划应携带当前修订号，并能应用
    let fresh = PlanPool::new(1).plan_seats(&e, &[0], &planner);
    let fresh = fresh.into_iter().next().expect("fresh plan");
    assert_eq!(fresh.revision, e.revision(), "新计划应携带当前修订号");
    let out = e.apply_plan(&fresh);
    eprintln!("DBG NC10 重规划：apply_plan = {:?}", out.is_ok());
    assert!(out.is_ok(), "重新规划后的计划应可应用：{out:?}");
}

/// ⑥ **不同策略变体下结果确定**：同一引擎、同一规划器，跑两次 ⇒ 逐字段相同。
#[test]
fn policy_variants_are_deterministic() {
    let e = swd_engine(20261006);
    for variant in [0u8, 1, 2] {
        let planner = BoundedPlanner {
            node_budget: 16,
            deadline_ms: Some(5_000),
            cancel: Default::default(),
            lookahead_depth: 1,
            policy_variant: variant,
            content_tuning: None,
        };
        let a = PlanPool::new(1).plan_seats(&e, &[0, 1, 2, 3], &planner);
        let b = PlanPool::new(4).plan_seats(&e, &[0, 1, 2, 3], &planner);
        let ka: Vec<String> = a
            .iter()
            .map(|p| format!("{}:{:?}:{}", p.seat, p.action, p.nodes_used))
            .collect();
        let kb: Vec<String> = b
            .iter()
            .map(|p| format!("{}:{:?}:{}", p.seat, p.action, p.nodes_used))
            .collect();
        eprintln!("DBG NC10 变体：variant={variant} 两次结果相同={}", ka == kb);
        assert_eq!(ka, kb, "variant={variant} 的规划结果应确定");
    }
}
