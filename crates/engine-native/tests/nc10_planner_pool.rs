//! NC10 长驻有界规划工作池：并发受限、队列满降级、取消、过期拒绝。

use engine_native::ai::BoundedPlanner;
use engine_native::ai_pool::{PlannerPool, PoolFull};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

fn planner() -> BoundedPlanner {
    BoundedPlanner::new(32).with_deadline_ms(50)
}

/// 长驻池：多次调用复用同一批线程；结果与串行规划一致（动作合法）。
#[test]
fn nc10_pool_reuses_workers_and_returns_legal_plans() {
    let pack = swd_pack();
    let e = Engine::new_with_extensions(20262001, 4, &[&pack]).expect("建局");
    let pool = PlannerPool::new(2, 8);
    assert_eq!(pool.workers(), 2);
    assert_eq!(pool.queue_cap(), 8);

    for _ in 0..3 {
        let (plans, degraded) = pool.plan_all(&e, &[0, 1, 2, 3], &planner());
        assert_eq!(plans.len(), 4, "四个座位都有规划产物");
        assert!(!degraded, "队列充足时不应降级");
        for plan in &plans {
            assert!(
                e.is_legal_action(plan.seat, &plan.action) || e.legal_actions(plan.seat).is_empty(),
                "产物动作必须合法（座位 {}）",
                plan.seat
            );
        }
    }
    pool.shutdown();
}

/// 队列满：`try_submit` 明确返回 `PoolFull`；`plan_all` 降级为串行且**不丢结果**。
#[test]
fn nc10_pool_full_degrades_without_losing_plans() {
    let pack = swd_pack();
    let e = Engine::new_with_extensions(20262002, 4, &[&pack]).expect("建局");
    // 容量 1：占满队列后继续提交必然失败
    let pool = PlannerPool::new(1, 1);
    let blocker = std::sync::Arc::new(std::sync::Barrier::new(2));
    let b2 = blocker.clone();
    pool.try_submit(move || {
        b2.wait();
    })
    .expect("首个任务应入队");
    // 该任务已被工作线程取走或占满队列 → 第二次提交应失败（池满）
    let second = pool.try_submit(|| {});
    assert_eq!(second, Err(PoolFull), "队列满必须明确返回 PoolFull");
    blocker.wait();

    // plan_all 在池满/竞争下仍返回全部座位结果（降级为串行补齐）
    let (plans, degraded) = pool.plan_all(&e, &[0, 1, 2, 3], &planner());
    assert_eq!(plans.len(), 4, "降级也不得丢结果");
    let _ = degraded; // 是否降级取决于竞争，不作硬断言
    pool.shutdown();
}

/// 取消：`cancel` 置位后产物带 `cancelled`（节点边界检查），不阻塞规则主通道。
#[test]
fn nc10_pool_respects_cancellation() {
    let pack = swd_pack();
    let e = Engine::new_with_extensions(20262003, 4, &[&pack]).expect("建局");
    let pool = PlannerPool::new(1, 4);
    let cancel = std::sync::Arc::new(AtomicBool::new(true));
    let mut p = planner();
    p.cancel = cancel;
    let (plans, _) = pool.plan_all(&e, &[0], &p);
    assert_eq!(plans.len(), 1);
    assert!(plans[0].cancelled, "取消标志置位后产物应标记 cancelled");
    pool.shutdown();
}

/// 过期拒绝：基于旧修订号规划的动作用新修订号提交 → `StaleRevision`（池不改语义）。
#[test]
fn nc10_pool_plans_are_revision_guarded() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20262004, 4, &[&pack]).expect("建局");
    let pool = PlannerPool::new(1, 4);
    let (plans, _) = pool.plan_all(&e, &[e.turn().current], &planner());
    let plan = plans.into_iter().next().expect("产物");
    let stale = plan.revision;
    // 推进一次状态
    let mut advanced = false;
    for actor in 0..4u8 {
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        if matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
            advanced = true;
            break;
        }
    }
    assert!(advanced, "应能推进一次");
    let out = e.submit(GameCommand {
        actor: plan.seat,
        expected_revision: stale,
        dedup_id: next_dedup(),
        action: plan.action,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::StaleRevision)),
        "过期规划结果必须被拒绝: {out:?}"
    );
    pool.shutdown();
}
