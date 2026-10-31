//! L27 选将美化批 2：止息（`new_zhixi`）——用牌计数 → 封锁出牌/响应。
//!
//! 上游 `extension/选将美化/extension.js`：`mod.cardEnabled/cardUsable/
//! cardRespondable` 在「`storage.new_zhixi2` 或 计数标记 ≥ 体力」时全部返回
//! false；`trigger {player:'useCard'}` 锁定：计数 +1，若为**锦囊**则置
//! `new_zhixi2`。

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
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
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

fn mark_of(e: &Engine, seat: u8, name: &str) -> i32 {
    let d = e.catalog().skill_by_name(name).expect(name);
    e.state_core()
        .vm_aux
        .marks
        .get(&(seat, u32::from(d)))
        .copied()
        .unwrap_or(0)
}

/// 止息：使用锦囊后立刻封锁出牌（计数标记与锦囊记录同时置位）。
#[test]
fn new_zhixi_locks_after_trick() {
    let mut e = sb_engine(151001);
    // 只授予主技实例；`new_zhixi_mark2` 仅作为**标记键定义**存在（无需实例）
    e.debug_grant(0, "new_zhixi");
    assert!(e.catalog().skill_by_name("new_zhixi").is_some());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 先用一张锦囊（无中生有）
    let trick = e.debug_place(0, "wuzhong");
    e.debug_give_hand_card(0, 0, trick);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: trick,
            targets: vec![],
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
    assert_eq!(mark_of(&e, 0, "new_zhixi"), 1, "计数 +1");
    assert_eq!(mark_of(&e, 0, "new_zhixi_mark2"), 1, "锦囊记录置位");
    // 再用一张【杀】→ 应被封锁
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(out2, CommandOutcome::Rejected(_)),
        "止息封锁出牌：{out2:?}"
    );
}

/// 止息（对照）：未持技者使用锦囊后仍可继续出牌。
#[test]
fn new_zhixi_absent_no_lock() {
    let mut e = sb_engine(151002);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let trick = e.debug_place(0, "wuzhong");
    e.debug_give_hand_card(0, 0, trick);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: trick,
            targets: vec![],
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
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out2 = e.submit(GameCommand {
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
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "未持技不封锁：{out2:?}"
    );
}
