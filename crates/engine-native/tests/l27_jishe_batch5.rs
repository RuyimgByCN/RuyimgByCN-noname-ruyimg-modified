//! L27 选将美化批 5：集舍（`jishe`）——出牌阶段摸 1 + 累计标记。
//!
//! 上游 `extension/选将美化/extension.js`：`enable:'phaseUse'`，`filter`：
//! `getHandcardLimit()>0`；`content`：`player.draw()` + 累计 `jishe2` 标记
//! （`jishe2` 在本文件中**被注释**，其手牌上限削减面不生效 → 登记差异）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn sb_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("select-beautify-upstream-liuli/assets/cards.json"),
    )
    .expect("读选将美化包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
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
    for _ in 0..16 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 集舍：发动 → 摸 1 且标记 +1（可重复发动，标记累加）。
#[test]
fn jishe_draws_and_marks() {
    let mut e = sb_engine(154001);
    e.debug_grant(0, "jishe");
    assert!(e.catalog().skill_by_name("jishe").is_some());
    advance_to_play(&mut e, 0);
    let def = e.catalog().skill_by_name("jishe").expect("jishe");
    let mark = |e: &Engine| {
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, u32::from(def)))
            .copied()
            .unwrap_or(0)
    };
    let before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 1, "摸 1");
    assert_eq!(mark(&e), 1, "标记 +1");
    // 再次发动 → 标记累加
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out2:?}"
    );
    drain(&mut e);
    assert_eq!(mark(&e), 2, "标记累加");
}

/// 集舍（对照）：未持技者不可发动。
#[test]
fn jishe_absent_rejected() {
    let mut e = sb_engine(154002);
    advance_to_play(&mut e, 0);
    let def = e.catalog().skill_by_name("jishe").expect("jishe");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
}
