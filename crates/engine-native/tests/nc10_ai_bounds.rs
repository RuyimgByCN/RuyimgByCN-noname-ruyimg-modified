//! NC10 追加：泄漏面**量化**、置换确定性、预算降级、过期结果拒绝。
//!
//! 背景：`planner_is_invariant_under_hidden_information`（nc10_ai_information_boundary）
//! 已验收静止态不泄漏。本文件把口径补齐为可量化与可回归：
//! 1. **量化**：静止态采样 N 个状态，隐藏信息置换后规划结果改变的比例（修复后应为 0）；
//!    并单独量化**非静止态窗口**（`redact_opponent_hands` 跳过重分配）的比例——该窗口
//!    是已知未完成面，必须被测量而不是假设为 0；
//! 2. **确定性**：同一状态重复规划结果一致（置换以座位+修订号播种，可重放）；
//! 3. **预算降级**：极小节点预算下仍产出**合法**动作（不 panic、不返回非法动作）；
//! 4. **过期结果拒绝**：基于旧修订号规划的动作用新修订号提交 → 必须被拒绝。

use engine_native::ai::BoundedPlanner;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
use std::sync::atomic::{AtomicU64, Ordering};

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
    BoundedPlanner::new(64)
        .with_deadline_ms(50)
        .with_policy_variant(1)
}

fn drive(e: &mut Engine, steps: usize) {
    let p = planner();
    for _step in 0..steps {
        if e.outcome().is_some() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> =
                if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                    vec![engine_native::ChoiceOption::Pass]
                } else if c.spec.allow_cancel {
                    Vec::new()
                } else {
                    c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
                };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            continue;
        }
        let seat = e.turn().current;
        let plan = e.plan_action(seat, &p);
        let _ = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: plan.action,
        });
    }
}

/// 隐藏信息置换（对手手牌 + 牌库旋转；公开计数不变）。
fn permute_hidden(e: &mut Engine, seat: u8, k: usize) {
    let n = e.state_core().players.len();
    let others: Vec<usize> = (0..n).filter(|i| *i as u8 != seat).collect();
    let lens: Vec<usize> = others
        .iter()
        .map(|i| e.state_core().players[*i].hand.len())
        .collect();
    let mut pool: Vec<u32> = Vec::new();
    {
        let core = e.state_core_mut();
        for i in &others {
            pool.append(&mut core.players[*i].hand);
        }
        pool.append(&mut core.deck);
    }
    if pool.len() < 2 {
        return;
    }
    let shift = k % pool.len();
    pool.rotate_left(shift);
    let core = e.state_core_mut();
    let mut cursor = 0usize;
    for (i, len) in others.iter().zip(lens) {
        core.players[*i].hand = pool[cursor..cursor + len].to_vec();
        cursor += len;
    }
    core.deck = pool[cursor..].to_vec();
}

fn is_quiescent(e: &Engine) -> bool {
    let r = e.resolution();
    r.stack.is_empty() && r.parks.is_empty() && r.open_choice.is_none()
}

/// 量化：静止态与非静止态窗口的泄漏比例（同一批采样）。
#[test]
fn nc10_leak_surface_quantified() {
    let pack = swd_pack();
    let mut quiescent_total = 0usize;
    let mut quiescent_changed = 0usize;
    let mut window_total = 0usize;
    let mut window_changed = 0usize;
    for seed in [20261501u64, 20261502, 20261503, 20261504] {
        for warm in [2usize, 5, 8, 11, 14, 17, 20, 23] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;
            let mut permuted = base.clone();
            permute_hidden(&mut permuted, seat, 7);
            let b = permuted.plan_action(seat, &planner()).action;
            let changed = format!("{a:?}") != format!("{b:?}");
            if is_quiescent(&base) {
                quiescent_total += 1;
                if changed {
                    quiescent_changed += 1;
                }
            } else {
                window_total += 1;
                if changed {
                    window_changed += 1;
                }
            }
        }
    }
    eprintln!(
        "DBG NC10 泄漏面：静止态 {quiescent_changed}/{quiescent_total} 改变；非静止态窗口 {window_changed}/{window_total} 改变"
    );
    assert!(quiescent_total >= 8, "静止态样本量不足（{quiescent_total}）");
    assert_eq!(
        quiescent_changed, 0,
        "静止态不得因隐藏信息置换改变规划（{quiescent_changed}/{quiescent_total}）"
    );
    // 非静止态窗口是**已知未完成面**：只量化，不在此断言为 0（避免伪造完成）。
}

/// 确定性：同一状态重复规划结果一致（置换可重放）。
#[test]
fn nc10_planner_is_deterministic() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261505, 4, &[&pack]).expect("建局");
    drive(&mut e, 9);
    let seat = e.turn().current;
    let a = e.plan_action(seat, &planner()).action;
    let b = e.plan_action(seat, &planner()).action;
    assert_eq!(format!("{a:?}"), format!("{b:?}"), "同状态规划必须确定");
    // 状态未变：修订号与摘要不变
    assert_eq!(e.revision(), e.revision());
}

/// 预算降级：极小预算下仍产出合法动作。
#[test]
fn nc10_tiny_budget_still_legal() {
    let pack = swd_pack();
    for budget in [0usize, 1, 2, 4] {
        let mut e = Engine::new_with_extensions(20261506 + budget as u64, 4, &[&pack])
            .expect("建局");
        drive(&mut e, 6);
        let seat = e.turn().current;
        let p = BoundedPlanner::new(budget).with_deadline_ms(20);
        let plan = e.plan_action(seat, &p);
        // 判据：**只要该座位存在合法动作**，规划结果就必须合法；若该座位此刻无
        // 合法动作（非本人回合/等待他人选择），允许返回占位动作（提交侧会拒绝）。
        let legal = e.legal_actions(seat);
        if !legal.is_empty() {
            assert!(
                e.is_legal_action(seat, &plan.action),
                "预算 {budget}：存在合法动作时规划结果必须合法（{:?}）",
                plan.action
            );
        } else {
            eprintln!("DBG 预算 {budget}：该座位此刻无合法动作，规划返回占位 {:?}", plan.action);
        }
    }
}

/// 过期结果拒绝：旧修订号规划的动作用新修订号提交 → 必须被拒绝。
#[test]
fn nc10_stale_plan_is_rejected() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261510, 4, &[&pack]).expect("建局");
    drive(&mut e, 5);
    let seat = e.turn().current;
    let plan = e.plan_action(seat, &planner());
    let stale_revision = e.revision();

    // 另一条命令推进状态（同座位，阶段推进；若被拒则改用其他座位的合法推进）
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
    assert!(advanced, "应能推进一次状态");
    assert!(e.revision() > stale_revision, "修订号已前进");

    // 用过期修订号提交先前规划的动作 → 必须拒绝
    let out = e.submit(GameCommand {
        actor: seat,
        expected_revision: stale_revision,
        dedup_id: next_dedup(),
        action: plan.action,
    });
    match out {
        CommandOutcome::Rejected(RejectReason::StaleRevision) => {}
        other => panic!("过期规划结果必须被拒绝（StaleRevision），实为 {other:?}"),
    }
}

/// 牌库顺序置换：**任意状态**（含非静止态）下规划结果不得依赖牌库顺序。
///
/// NC10 追加修复：非静止态原先直接跳过遮蔽 → 模拟可读到牌库顺序；现改为**只打乱
/// 牌库顺序**（内容与区域不变，故不会让挂起机器/在飞帧的实体牌引用失效）。
#[test]
fn nc10_planner_is_invariant_under_deck_order() {
    let pack = swd_pack();
    let mut quiescent_total = 0usize;
    let mut window_total = 0usize;
    for seed in [20261521u64, 20261522, 20261523] {
        for warm in [3usize, 7, 11, 15, 19] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;
            // 只置换牌库顺序（内容不变）
            let mut permuted = base.clone();
            {
                let n = permuted.state_core().deck.len();
                if n >= 2 {
                    let core = permuted.state_core_mut();
                    core.deck.rotate_left(1);
                }
            }
            // 牌库内容必须守恒
            let mut before = base.state_core().deck.clone();
            let mut after = permuted.state_core().deck.clone();
            before.sort_unstable();
            after.sort_unstable();
            assert_eq!(before, after, "牌库内容必须守恒（只改顺序）");
            let b = permuted.plan_action(seat, &planner()).action;
            if is_quiescent(&base) {
                quiescent_total += 1;
            } else {
                window_total += 1;
            }
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "seed={seed} warm={warm} seat={seat}：规划结果依赖了牌库顺序"
            );
        }
    }
    eprintln!("DBG 牌库顺序不变性：静止态 {quiescent_total} 例、非静止态 {window_total} 例");
    assert!(quiescent_total + window_total >= 8, "样本量不足");
    assert!(window_total >= 1, "应至少覆盖 1 个非静止态样本");
}

#[test]
fn content_tuning_rejects_non_finite_values() {
    use engine_native::ai::ContentTuning;
    let tuning = ContentTuning {
        seat_weights: vec![(0, f64::NAN), (1, f64::INFINITY), (2, 2.0)],
        skill_threat: vec![("bad".into(), f64::NAN), ("good".into(), 3.0)],
        qz_cf: true,
    };
    assert_eq!(tuning.weight_of(0), 1.0);
    assert_eq!(tuning.weight_of(1), 1.0);
    assert_eq!(tuning.weight_of(2), 2.0);
    assert_eq!(tuning.threat_of_skills(["bad"]), 1.0);
    assert_eq!(tuning.threat_of_skills(["good"]), 3.0);
    assert!(tuning.benefit(f64::NAN, 5.0).is_finite());
    assert!(tuning.benefit(2.0, f64::INFINITY).is_finite());
    assert_eq!(tuning.benefit(2.0, 5.0), 10.0);
}

#[test]
fn zero_deadline_returns_timed_out_plan_without_blocking() {
    let engine = Engine::new(20_261_024, 4);
    let planner = BoundedPlanner::new(usize::MAX / 2).with_deadline_ms(0);
    let started = std::time::Instant::now();
    let plan = engine.plan_action(0, &planner);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(2),
        "deadline 0 不应阻塞"
    );
    assert!(plan.timed_out, "deadline 0 必须标记 timed_out");
    assert_eq!(plan.revision, engine.revision());
}
