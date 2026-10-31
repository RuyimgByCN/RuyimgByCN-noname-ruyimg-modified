//! L27 选将美化批 4：诈降（`sbzhaxiang`）——用牌数 < 已损失体力 → 无限制 + 直伤。
//!
//! 上游 `extension/选将美化/extension.js`：`mod.targetInRange`/`cardUsable`
//! 在 `countUsed() < getDamagedHp()` 时无视距离/次数；`trigger {player:'useCard'}`
//! 锁定：`countUsed() <= getDamagedHp()` 时对全场 directHit；`group` 的
//! `sbzhaxiang_draw` 在摸牌阶段 `num += getDamagedHp()`。

use engine_native::catalog::Suit;
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

/// 诈降：已损失体力 2 → 摸牌阶段多摸 2 张。
#[test]
fn sbzhaxiang_extra_draw() {
    let mut e = sb_engine(153001);
    e.debug_grant(0, "sbzhaxiang");
    assert!(e.catalog().skill_by_name("sbzhaxiang").is_some());
    e.debug_set_hp(0, 2); // 上限 4 → 已损失 2
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge → Draw（摸牌）
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Draw);
    assert_eq!(e.debug_hand(0).len(), before + 4, "正常 2 + 已损失 2");
}

/// 诈降（对照）：未持技者按正常摸 2。
#[test]
fn sbzhaxiang_absent_normal_draw() {
    let mut e = sb_engine(153002);
    e.debug_set_hp(0, 2);
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 2, "未持技：正常摸 2");
}

/// 诈降：已损失体力 2 → 出杀不限次数（第 2 张杀仍可出）。
#[test]
fn sbzhaxiang_unlimited_sha() {
    let mut e = sb_engine(153003);
    e.debug_grant(0, "sbzhaxiang");
    e.debug_set_hp(0, 2);
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 两张杀 + 两次出杀
    for _ in 0..2 {
        let sha = e.debug_place(0, "sha");
        e.debug_give_hand_card(0, 0, sha);
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
            "诈降：出杀不被次数限制 {out:?}"
        );
        drain(&mut e);
    }
}

/// 诈降（对照）：未持技者第二张杀被次数限制拒绝。
#[test]
fn sbzhaxiang_absent_sha_limited() {
    let mut e = sb_engine(153004);
    e.debug_set_hp(0, 2);
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha1 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha1,
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
    let sha2 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha2);
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    });
    assert!(
        matches!(out2, CommandOutcome::Rejected(_)),
        "未持技：第二张杀被拒 {out2:?}"
    );
    let _ = Suit::Heart;
}
