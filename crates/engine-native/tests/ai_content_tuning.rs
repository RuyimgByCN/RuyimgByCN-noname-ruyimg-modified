//! L01 AI优化 消费面：内容调参（席位权重）参与原生规划器评分。
//!
//! 装载面（`content-native-loader::ai_tuning`）提供参数与口径；本测试验证
//! **原生 AI 消费**：同信息集 → 同决策（确定性），且调参改变决策方向
//! （高权重敌方优先）。

use engine_native::ai::{BoundedPlanner, ContentTuning};
use engine_native::{Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
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
    }
}

/// 构造：0 号持【杀】，1/3 号空手且相邻（距离 1，攻击范围内），轮到 0 号出牌阶段。
fn attack_setup(seed: u64) -> Engine {
    let mut e = Engine::new(seed, 4);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        assert!(
            matches!(
                out,
                engine_native::CommandOutcome::Accepted { .. }
                    | engine_native::CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    e
}

fn planned_target(weights: &[(u8, f64)]) -> Option<Vec<u8>> {
    let e = attack_setup(910001);
    let tuning = ContentTuning {
        seat_weights: weights.to_vec(),
        skill_threat: vec![("mohua2".into(), 2.0)],
        qz_cf: true,
    };
    let planner = BoundedPlanner::new(64)
        .with_policy_variant(2)
        .with_content_tuning(tuning);
    let plan = e.plan_action(0, &planner);
    match plan.action {
        GameAction::PlayCard { targets, .. } => Some(targets),
        _ => None,
    }
}

/// 同信息集 → 同决策（确定性）。
#[test]
fn content_tuning_is_deterministic() {
    let a = planned_target(&[(1, 1.0), (3, 1.0)]);
    let b = planned_target(&[(1, 1.0), (3, 1.0)]);
    assert_eq!(a, b, "同调参同局面：决策一致");
    assert!(a.is_some(), "应产出出牌计划：{a:?}");
}

/// 调参改变决策方向：高权重敌方优先。
#[test]
fn content_tuning_prefers_high_weight_enemy() {
    // 0 号与 1/3 号相邻（距离 1，均在攻击范围内）
    let heavy_three = planned_target(&[(1, 1.0), (3, 5.0)]);
    let heavy_one = planned_target(&[(1, 5.0), (3, 1.0)]);
    assert!(heavy_three.is_some() && heavy_one.is_some());
    assert!(
        heavy_three.as_ref().unwrap().contains(&3),
        "3 号权重高 → 目标含 3：{heavy_three:?}"
    );
    assert!(
        heavy_one.as_ref().unwrap().contains(&1),
        "1 号权重高 → 目标含 1：{heavy_one:?}"
    );
    assert_ne!(heavy_three, heavy_one, "调参应改变决策");
}

/// 调参缺省（无权重）→ 权重 1.0；`threat_of_skills` 取最大值；`benefit` 两态。
#[test]
fn content_tuning_defaults_and_helpers() {
    let t = ContentTuning {
        seat_weights: vec![(1, 2.0)],
        skill_threat: vec![("a".into(), 0.5), ("b".into(), 1.9)],
        qz_cf: true,
    };
    assert_eq!(t.weight_of(1), 2.0);
    assert_eq!(t.weight_of(2), 1.0);
    assert_eq!(t.threat_of_skills(["a", "b", "c"]), 1.9);
    assert_eq!(t.threat_of_skills(["c"]), 1.0);
    assert!((t.benefit(2.0, 3.0) - 6.0).abs() < 1e-9);
    let mut off = t.clone();
    off.qz_cf = false;
    assert!((off.benefit(2.0, 3.0) - 3.0).abs() < 1e-9);
    // 越界钳制
    let clamped = ContentTuning {
        seat_weights: vec![(3, 99.0)],
        ..Default::default()
    };
    assert_eq!(clamped.weight_of(3), 10.0);
}
