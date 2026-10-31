//! swd 批 150：疾风（`jifeng`）——单目标牌多目标 + 多目标时概率失误。
//!
//! 上游 `character/swd.js`：`mod.selectTarget` 让【杀】与单目标锦囊目标数
//! 无限；`trigger {player:'useCardToBefore'}` 锁定——多目标使用时按概率
//! `1-(1.3+n/5)/n` **整牌取消**（失误）。
//!
//! 登记差异：原生**仅【杀】族**支持多目标（单目标锦囊的机器为单目标结算，
//! 其多目标扩展未落地）。

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

/// 疾风：持技者可用一张【杀】指定两名目标（未持技则被拒）。
#[test]
fn jifeng_allows_multi_target_sha() {
    let mut e = swd_engine(126001);
    e.debug_grant(0, "jifeng");
    assert!(e.catalog().skill_by_name("jifeng").is_some());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1, 3],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "疾风：多目标可接受 {out:?}"
    );
    drain(&mut e);
    // 对照：未持技者同牌被拒
    let mut e2 = swd_engine(126002);
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let sha2 = e2.debug_place(0, "sha");
    e2.debug_give_hand_card(0, 0, sha2);
    let out2 = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha2,
            targets: vec![1, 3],
        },
    });
    assert!(
        matches!(out2, CommandOutcome::Rejected(_)),
        "未持技：多目标被拒 {out2:?}"
    );
}

/// 疾风：多目标使用时按概率失误——多次尝试必出现整牌取消（无伤害）。
#[test]
fn jifeng_multi_target_may_fizzle() {
    let mut saw_fizzle = false;
    let mut saw_hit = false;
    for seed in 126100..126140u64 {
        let mut e = swd_engine(seed);
        e.debug_grant(0, "jifeng");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sha = e.debug_place(0, "sha");
        e.debug_give_hand_card(0, 0, sha);
        let hp1 = e.debug_hp(1);
        let hp3 = e.debug_hp(3);
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![1, 3],
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
        let d1 = hp1 - e.debug_hp(1);
        let d3 = hp3 - e.debug_hp(3);
        if d1 == 0 && d3 == 0 {
            saw_fizzle = true;
        } else if d1 == 1 && d3 == 1 {
            saw_hit = true;
        }
        if saw_fizzle && saw_hit {
            break;
        }
    }
    assert!(saw_hit, "多目标命中样本出现");
    assert!(saw_fizzle, "多目标失误（整牌取消）样本出现");
}

/// 疾风：单目标使用不触发失误判定（目标数 ≤1）。
#[test]
fn jifeng_single_target_never_fizzles() {
    for seed in 126200..126210u64 {
        let mut e = swd_engine(seed);
        e.debug_grant(0, "jifeng");
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
        assert_eq!(e.debug_hp(1), hp1 - 1, "单目标必命中（seed {seed}）");
    }
}
