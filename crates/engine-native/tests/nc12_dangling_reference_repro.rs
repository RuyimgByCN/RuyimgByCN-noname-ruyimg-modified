//! NC12 内核悬空引用：**确定性最小复现**（seed 44170，第 10 步）。
//!
//! 由 `nc12_dangling_reference_probe` 扫描得到：swd 档、4 座、策略 variant 0 / budget 8 /
//! lookahead 1，seed **44170** 在第 **10** 步（seat 1 回答 choice 2）时，模拟提交触发
//! `machine.rs:408 card 93 not in hand of 0`。
//!
//! 本文件把该局面固定下来：先按同参数驱动到第 9 步（不触发故障），再打印**挂起机器**与
//! 玩家区域，供定位「哪台机器引用了已不在手牌的牌」。
//!
//! 说明：本用例**允许失败**（命中故障即失败并打印现场）；修复后应转为通过并断言不再触发。

use engine_native::ai::BoundedPlanner;
use engine_native::{CommandOutcome, Engine, GameCommand};
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
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn dump(e: &Engine, tag: &str) {
    let res = e.resolution();
    eprintln!(
        "[{tag}] rev={} phase={:?} turn={} stack={} parks={} choice={:?} table={:?}",
        e.revision(),
        e.turn().phase,
        e.turn().current,
        res.stack.len(),
        res.parks.len(),
        e.open_choice().map(|c| (c.id, c.spec.prompt)),
        e.state_core().table,
    );
    for (i, p) in e.state_core().players.iter().enumerate() {
        eprintln!(
            "[{tag}]   p{i}: hp={}/{} hand={:?} equips={:?} judge={:?} off={:?}",
            p.hp, p.max_hp, p.hand, p.equips, p.judge, p.off_court
        );
    }
    // 事件栈：定位**发出用牌事件**的帧（panic 来自 commit_card_used，需知道谁在提交）
    for (i, frame) in res.stack.iter().enumerate() {
        eprintln!(
            "[{tag}]   frame[{i}] id={:?} parent={:?} kind={:?} stage={:?} data={:?}",
            frame.id, frame.parent, frame.kind, frame.stage, frame.data
        );
    }
    for (i, park) in res.parks.iter().enumerate() {
        // `Park` 无 Debug：打印可读字段 + 机器快照（若可序列化）
        eprintln!(
            "[{tag}]   park[{i}] resume_depth={} slot={:?} snapshot={:?}",
            park.resume_depth,
            park.slot,
            park.machine.snapshot()
        );
    }
}

#[test]
#[ignore = "登记缺陷守护：seed 44170 第 10 步复现内核悬空引用；修复后应转为通过"]
fn nc12_repro_seed_44170_step_10() {
    // 打印故障上下文日志（非打印式记录，测试可读）——用于定位**发出失效用牌的路径**
    engine_native::ai::clear_sim_fault_log();
    engine_native::engine::legal_actions::clear_probe_fault_log();
    let planner = BoundedPlanner::new(8)
        .with_deadline_ms(50)
        .with_lookahead(1)
        .with_policy_variant(0);
    let mut e = swd_engine(44170);
    let mut faults = 0u64;
    for step in 0..12usize {
        let seat = match e.open_choice() {
            Some(open) => open.observer,
            None => e.turn().current,
        };
        let plan = e.plan_action(seat, &planner);
        if plan.sim_faults > 0 {
            faults += plan.sim_faults as u64;
            eprintln!("[repro] step={step} seat={seat} 命中 sim_faults={}", plan.sim_faults);
            dump(&e, "fault");
            break;
        }
        dump(&e, &format!("step{step}"));
        let action = plan.action;
        let out = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action,
        });
        if let CommandOutcome::Rejected(reason) = &out {
            eprintln!("[repro] step={step} 被拒：{reason:?}");
        }
    }
    if faults > 0 {
        for (i, line) in engine_native::ai::sim_fault_log().iter().enumerate() {
            eprintln!("[sim-fault {i}] {line}");
        }
        for (i, line) in engine_native::engine::legal_actions::probe_fault_log().iter().enumerate() {
            eprintln!("[probe-fault {i}] {line}");
        }
    }
    assert_eq!(
        faults, 0,
        "内核悬空引用复现（seed 44170）——现场已打印；修复后本断言应保持 0"
    );
}
