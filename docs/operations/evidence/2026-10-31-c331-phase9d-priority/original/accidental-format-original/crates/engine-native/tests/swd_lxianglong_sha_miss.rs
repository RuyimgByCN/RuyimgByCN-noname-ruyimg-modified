//! swd deferred 消解批 141：李香龙（`lxianglong`）——杀被闪避后反击。
//!
//! 上游 `character/swd.js`：`trigger {target:'shaMiss'}`，`priority:5`，
//! `direct:true`——自己的【杀】被闪避后可选：对攻击范围内一名角色使用一张
//! 虚拟【杀】。

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
    for _ in 0..64 {
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

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..12 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

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

/// 李香龙：0 号的杀被 1 号闪避 → 确认 → 选射程内 3 号 → 虚拟杀命中（3 号受伤）。
#[test]
fn lxianglong_counterattacks_after_sha_miss() {
    let mut e = swd_engine(118001);
    e.debug_grant(0, "lxianglong");
    assert!(e.catalog().skill_by_name("lxianglong").is_some());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(0, 1, shan);
    let hp3 = e.debug_hp(3);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 1 号出闪
    assert!(
        answer(&mut e, "respond_shan", vec![ChoiceOption::Card(shan)]),
        "闪响应窗"
    );
    // 杀被闪避 → 李香龙窗（确认门）
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "李香龙确认门"
    );
    assert!(
        answer(&mut e, "lxianglong_target", vec![ChoiceOption::Player(3)]),
        "李香龙目标窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(3), hp3 - 1, "虚拟杀命中 3 号");
}

/// 李香龙（对照）：杀命中（未闪避）→ 不开窗。
#[test]
fn lxianglong_no_window_when_sha_hits() {
    let mut e = swd_engine(118002);
    e.debug_grant(0, "lxianglong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "杀命中");
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "lxianglong_target")
            .unwrap_or(true),
        "未闪避不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
}

/// 李香龙（对照）：放弃发动 → 不再追加杀。
#[test]
fn lxianglong_decline_no_extra_sha() {
    let mut e = swd_engine(118003);
    e.debug_grant(0, "lxianglong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(0, 1, shan);
    let hp3 = e.debug_hp(3);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "respond_shan", vec![ChoiceOption::Card(shan)]),
        "闪响应窗"
    );
    assert!(
        answer(&mut e, "trigger_confirm", Vec::new()),
        "李香龙确认门（放弃）"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(3), hp3, "放弃：无追加杀");
}

/// 李香龙（对照）：未持技者杀被闪避不开窗。
#[test]
fn lxianglong_absent_no_window() {
    let mut e = swd_engine(118004);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(0, 1, shan);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "respond_shan", vec![ChoiceOption::Card(shan)]),
        "闪响应窗"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "lxianglong_target")
            .unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}
