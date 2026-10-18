//! NC03 AI 一致性：规划器产出的动作必须通过内核权威校验。
//!
//! 现状（登记差异，见证据）：AI 的候选枚举仍是视图镜像（`ai.rs::
//! enumerate_candidates_variant`），未改用 `legal_actions`——原因是非变异校验
//! 探针按候选克隆引擎（实测 **0.54 ms/候选**，一次查询 1.62 ms），节点预算
//! 下代价过高（NC12 基线 + NC03 待办：非克隆校验入口）。
//!
//! 本文件验收「AI 不会提交非法动作」这一可观测契约：不同种子/预算/变体下，
//! `plan_action` 产出的动作都能通过 `is_legal_action`（与正式提交同源校验）。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
        let _ = e.submit(engine_native::GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 规划器在多种子/预算/变体下的动作均通过权威校验。
#[test]
fn planner_actions_pass_authoritative_validation() {
    let mut checked = 0usize;
    for seed in [20261031u64, 20261032, 20261033] {
        for (nodes, variant) in [(16usize, 0u8), (64, 1)] {
            let mut e = Engine::new(seed, 4);
            for _ in 0..30 {
                if e.turn().current == 0 && e.turn().phase == Phase::Play {
                    break;
                }
                let actor = e.turn().current;
                let _ = e.submit(engine_native::GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: engine_native::GameAction::NextPhase,
                });
                drain(&mut e);
            }
            let seat = e.turn().current;
            let planner = BoundedPlanner::new(nodes).with_policy_variant(variant);
            let plan = e.plan_action(seat, &planner);
            assert!(
                e.is_legal_action(seat, &plan.action),
                "seed={seed} nodes={nodes} variant={variant}：规划动作须通过内核校验（{:?}）",
                plan.action
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 6, "六组配置均已检查");
}

/// 反向探针：伪造的非法动作会被权威校验拒绝（证明校验不是恒真）。
#[test]
fn forged_action_is_rejected_by_authoritative_validation() {
    let mut e = Engine::new(20261034, 4);
    for _ in 0..30 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: engine_native::GameAction::NextPhase,
        });
        drain(&mut e);
    }
    // 不存在的牌 id + 不存在的目标座位
    let forged = engine_native::GameAction::PlayCard {
        card: 9999,
        targets: vec![9],
    };
    assert!(
        !e.is_legal_action(0, &forged),
        "伪造动作必须被权威校验拒绝"
    );
}
