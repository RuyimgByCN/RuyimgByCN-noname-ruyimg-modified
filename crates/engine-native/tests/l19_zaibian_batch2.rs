//! L19 末日乱斗批 2：灾变（`zaibian`）——出牌阶段开始按身份数摸牌。
//!
//! 上游 `extension/末日乱斗/extension.js`：`trigger {player:'phaseUseBegin'}`
//! 锁定——`get.population('zhong') - get.population('fan') - get.population('nei') + 2`
//! > 0 时摸该数量张牌。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn db_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("doomsday-brawl-upstream-liuli/assets/cards.json"),
    )
    .expect("读末日乱斗包");
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

/// 灾变：身份局（主+忠+反+内）下 0 号出牌阶段开始摸「忠+2-反-内」张。
#[test]
fn zaibian_draws_by_identity_counts() {
    let mut e = db_engine(156001);
    e.debug_grant(0, "zaibian");
    assert!(e.catalog().skill_by_name("zaibian").is_some());
    // 记录身份分布（Prototype 模式下无身份 → 摸 2 张 = 0+2-0-0）
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Judge);
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge → Draw（正常摸 2）
    drain(&mut e);
    let after_draw = e.debug_hand(0).len();
    assert_eq!(after_draw, before + 2, "正常摸牌 2");
    next_phase(&mut e); // Draw → Play（灾变触发）
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Play);
    // Prototype 模式无身份 → 灾变摸 2（0+2-0-0）
    assert_eq!(e.debug_hand(0).len(), after_draw + 2, "灾变：忠+2-反-内 = 2");
}

/// 灾变（对照）：未持技者出牌阶段开始不摸牌。
#[test]
fn zaibian_absent_no_draw() {
    let mut e = db_engine(156002);
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e); // → Draw
    drain(&mut e);
    let after_draw = e.debug_hand(0).len();
    next_phase(&mut e); // → Play
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), after_draw, "未持技：出牌阶段开始不摸牌");
}
