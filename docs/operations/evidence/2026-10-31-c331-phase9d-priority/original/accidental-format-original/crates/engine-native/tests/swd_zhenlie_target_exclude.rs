//! swd deferred 消解批 139：贞烈（`zhenlie`）——成为杀/锦囊目标时失体力并移出目标。
//!
//! 上游 `character/yijiang.js:11678`：`trigger {target:'useCardToTargeted'}`，
//! `direct:true`——成为其他角色【杀】或锦囊牌目标时可选：失去 1 点体力，
//! 将该牌从自己的目标中移除，并弃置使用者一张牌（手/装备）。

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

/// 贞烈：被杀指定为目标 → 确认 → 失 1 体力 + 移出目标（不受伤害）+ 弃置使用者一张牌。
#[test]
fn zhenlie_excludes_self_from_sha_and_discards_user_card() {
    let mut e = swd_engine(116001);
    e.debug_grant(1, "zhenlie");
    assert!(e.catalog().skill_by_name("zhenlie").is_some());
    advance_to_play(&mut e, 0);
    // 0 号用杀打 1 号；1 号无闪
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let extra = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, extra);
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
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "贞烈确认门"
    );
    assert!(
        answer(&mut e, "zhenlie_discard", vec![ChoiceOption::Card(extra)]),
        "贞烈弃牌窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "失去 1 点体力");
    assert!(!e.debug_hand(0).contains(&extra), "使用者被弃置一张牌");
    assert_eq!(e.debug_hand(1).len(), 0, "未被杀命中（已移出目标）");
}

/// 贞烈（对照）：放弃发动 → 照常结算（无闪则受伤）。
#[test]
fn zhenlie_decline_takes_damage() {
    let mut e = swd_engine(116002);
    e.debug_grant(1, "zhenlie");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
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
    assert!(
        answer(&mut e, "trigger_confirm", Vec::new()),
        "贞烈确认门（放弃=空选择）"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "放弃：照常受 1 伤");
}

/// 贞烈（对照）：未被指定为目标（他人对他人用杀）→ 不开窗。
#[test]
fn zhenlie_no_window_when_not_targeted() {
    let mut e = swd_engine(116003);
    e.debug_grant(1, "zhenlie");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![3],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "非目标不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 贞烈（对照）：未持技者被杀不开窗。
#[test]
fn zhenlie_absent_no_window() {
    let mut e = swd_engine(116004);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
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
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "照常受伤");
}
