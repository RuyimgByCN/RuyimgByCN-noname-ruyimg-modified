//! swd deferred 消解批 140：移花（`yihua`）——单目标牌反弹（对调使用/目标）。
//!
//! 上游 `character/swd.js`：`trigger {target:'useCardToBefore'}`，`direct:true`
//! ——成为**单目标**牌的目标时可选：弃 2 张手牌，把该牌的「使用者/目标」对调
//! （反弹给原使用者，且视为自己使用）。

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
    let pack = content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
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
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
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
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// 移花：1 号被 0 号的杀指定为目标 → 弃 2 张手牌 → 反弹（0 号受伤）。
#[test]
fn yihua_bounces_single_target_card() {
    let mut e = swd_engine(117001);
    e.debug_grant(1, "yihua");
    assert!(e.catalog().skill_by_name("yihua").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let c1 = e.debug_place(1, "tao");
    let c2 = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, c1);
    e.debug_give_hand_card(0, 1, c2);
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "移花确认门"
    );
    assert!(
        answer(
            &mut e,
            "yihua_discard",
            vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]
        ),
        "移花弃 2 张窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "弃 2 张手牌");
    assert_eq!(e.debug_hp(1), hp1, "1 号未受伤（已反弹）");
    assert_eq!(e.debug_hp(0), hp0 - 1, "0 号（原使用者）受伤");
}

/// 移花（对照）：放弃 → 照常结算（1 号无闪受伤）。
#[test]
fn yihua_decline_resolves_normally() {
    let mut e = swd_engine(117002);
    e.debug_grant(1, "yihua");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let c1 = e.debug_place(1, "tao");
    let c2 = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, c1);
    e.debug_give_hand_card(0, 1, c2);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", Vec::new()),
        "移花确认门（放弃）"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "放弃：1 号照常受伤");
    assert_eq!(e.debug_hand(1).len(), 2, "手牌未弃");
}

/// 移花（对照）：手牌不足 2 张 → 不开窗。
#[test]
fn yihua_no_window_with_one_card() {
    let mut e = swd_engine(117003);
    e.debug_grant(1, "yihua");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let c1 = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, c1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "手牌不足不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 移花（对照）：未持技者被杀不开窗。
#[test]
fn yihua_absent_no_window() {
    let mut e = swd_engine(117004);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let c1 = e.debug_place(1, "tao");
    let c2 = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, c1);
    e.debug_give_hand_card(0, 1, c2);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}
