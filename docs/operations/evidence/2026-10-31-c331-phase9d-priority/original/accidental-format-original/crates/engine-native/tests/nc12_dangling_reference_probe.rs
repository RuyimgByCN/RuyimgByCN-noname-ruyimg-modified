//! NC12 长程故障最小复现探针：在**测试侧**（允许输出）驱动多局策略对局，
//! 一旦命中「模拟提交/候选探测触发内核不变量 panic」的故障计数，就打印现场状态并失败。
//!
//! 背景：长程负载（`nc12_bench long`）在 rev=10 的早期局面出现
//! `machine.rs:408 card 93 not in hand of 0`；现场切片显示遮蔽时已正确跳过身份遮蔽，
//! 且该牌在遮蔽前就不在持有者手牌 → 属**内核自身悬空引用**
//! （见 docs/operations/evidence/2026-10-18-nc12-plan-crash/README.md §7）。
//!
//! 本探针的目标：把「哪个 seed/哪一步」固定下来，供后续最小化与修复使用。
//! 说明：探针**允许失败**（命中即失败并打印现场）；未命中则通过并打印统计。

use engine_native::ai::BoundedPlanner;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn fault_counts() -> (u64, u64) {
    (engine_native::engine::legal_actions::probe_faults(), 0)
}

/// 驱动一局；返回 (步数, 是否终局, 本局新增的探测故障数)。
fn swd_engine(seed: u64) -> Engine {
    // 长程负载用的是 `tier=swd`（含 swd 转换包）——标准目录无法复现，必须同档
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drive(seed: u64, max_steps: usize, planner: &BoundedPlanner) -> (usize, bool, u64) {
    let mut e = swd_engine(seed);
    let before = fault_counts().0;
    let mut steps = 0usize;
    while e.outcome().is_none() && steps < max_steps {
        let seat = match e.open_choice() {
            Some(open) => open.observer,
            None => e.turn().current,
        };
        let plan = e.plan_action(seat, planner);
        let sim_faults = plan.sim_faults as u64;
        if sim_faults > 0 {
            eprintln!(
                "PROBE sim_fault seed={seed} step={steps} seat={seat} action={:?} rev={} phase={:?} table={:?}",
                plan.action,
                e.revision(),
                e.turn().phase,
                e.state_core().table,
            );
            for (i, p) in e.state_core().players.iter().enumerate() {
                eprintln!(
                    "PROBE   p{i}: hp={}/{} hand={:?} equips={:?} judge={:?} off={:?}",
                    p.hp, p.max_hp, p.hand, p.equips, p.judge, p.off_court
                );
            }
            return (steps, e.outcome().is_some(), sim_faults);
        }
        let action: GameAction = plan.action;
        let out = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action,
        });
        if let CommandOutcome::Rejected(_) = out {
            // 拒绝也计入步数（避免死循环）
        }
        steps += 1;
    }
    let after = fault_counts().0;
    (steps, e.outcome().is_some(), after - before)
}

/// 多 seed 扫描：命中即打印现场并失败（供最小化）。
#[test]
#[ignore = "登记缺陷守护：命中内核悬空引用故障即失败（当前 seed 44170 命中）；修复后应转为通过"]
fn nc12_probe_scans_seeds_for_dangling_reference() {
    // 与长程负载同参数（budget=8、lookahead=1、variant=0），否则候选探索面不同
    let planner = BoundedPlanner::new(8)
        .with_deadline_ms(50)
        .with_lookahead(1)
        .with_policy_variant(0);
    // 覆盖长程负载（`nc12_bench long`，seed=42007）实际使用的 seed 序列
    // （长程实测在 game≈2163 处命中故障 → seed = 42007 + g）。
    let seeds: Vec<u64> = (44000..45200).collect();
    let mut scanned = 0usize;
    for seed in seeds {
        scanned += 1;
        let (steps, finished, faults) = drive(seed, 900, &planner);
        if faults > 0 {
            panic!(
                "seed={seed} 命中故障（steps={steps} finished={finished}）——现场已打印，请据此最小化"
            );
        }
    }
    eprintln!("PROBE 扫描完成：{scanned} 个 seed 无故障");
}
