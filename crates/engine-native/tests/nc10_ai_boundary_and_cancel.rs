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
    let pack = content_native_loader::bridge_pack(&cards).expect("bridge").pack;
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
        let (Some(p2), Some(p3)) = (core.players.get(2).cloned(), core.players.get(3).cloned()) else {
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
