//! swd deferred 消解批 126：集控（`jikong`）——阶段开始/失去手牌后虚拟雷杀。
//!
//! 上游 `character/swd.js`：`trigger {player:['loseEnd','phaseBegin']}`，
//! `direct:true`，`usable:1`——自己阶段开始，或自己失去手牌致手牌为空时，
//! 可对攻击距离内一名其他角色使用虚拟【雷杀】；取消不消耗次数。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn swd_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 swd 包")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn next_phase(e: &mut Engine) {
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
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
}

/// 应答指定 prompt（断言被接受）。
fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: fallback,
            },
        });
    }
    false
}

/// 集控（阶段开始面）：自己阶段开始 → 确认 → 选攻击距离内目标 → 虚拟雷杀
/// 造成 1 点雷伤（无闪）；每回合限一次（第二次阶段开始不再开窗）。
#[test]
fn jikong_phase_face_virtual_thunder_sha_once_per_turn() {
    let mut e = swd_engine(103001);
    e.debug_grant(0, "jikong");
    assert!(e.catalog().skill_by_name("jikong").is_some());
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 推进到 0 号某个阶段开始（入梦等无——直接 NextPhase 触发 PhaseBegin）
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    let hp1 = e.debug_hp(1);
    next_phase(&mut e); // Draw → Play begin（PhaseBegin 触发集控）
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "集控确认窗"
    );
    assert!(
        answer(&mut e, "jikong_target", vec![ChoiceOption::Player(1)]),
        "集控目标窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "虚拟雷杀 1 点雷伤");
    // 每回合限一次：本回合内再次阶段开始不再开窗
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "同回合不再开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 集控（失去手牌面）：自己失去手牌致手牌为空 → 开窗；有手牌时不开窗。
#[test]
fn jikong_lose_face_requires_empty_hand() {
    let mut e = swd_engine(103002);
    e.debug_grant(0, "jikong");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 有手牌时失去一张：不开窗
    let a = e.debug_place(0, "sha");
    let b = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, a);
    e.debug_give_hand_card(0, 0, b);
    e.debug_discard_card(0, a);
    drain(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "仍有手牌不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    // 失去最后一张：开窗 → 确认 → 选目标 → 雷杀
    let hp1 = e.debug_hp(1);
    e.debug_discard_card(0, b);
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "集控失去手牌确认窗"
    );
    assert!(
        answer(&mut e, "jikong_target", vec![ChoiceOption::Player(1)]),
        "集控失去手牌目标窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "失去手牌面虚拟雷杀命中");
}

/// 集控（对照）：取消目标窗 → 不消耗每回合次数（随后阶段开始仍可发动）。
#[test]
fn jikong_cancel_refunds_use() {
    let mut e = swd_engine(103003);
    e.debug_grant(0, "jikong");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "首次确认窗"
    );
    assert!(answer(&mut e, "jikong_target", Vec::new()), "取消目标窗");
    drain(&mut e);
    // 取消后同回合内下一个阶段开始仍可开窗（未消耗次数）
    next_phase(&mut e);
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "取消后仍可发动"
    );
    assert!(
        answer(&mut e, "jikong_target", vec![ChoiceOption::Player(1)]),
        "目标窗"
    );
    drain(&mut e);
}

/// 集控（对照）：未持技者阶段开始不开窗。
#[test]
fn jikong_absent_no_window() {
    let mut e = swd_engine(103004);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}
