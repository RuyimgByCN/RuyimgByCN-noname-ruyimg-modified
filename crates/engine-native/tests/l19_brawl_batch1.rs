//! L19 末日乱斗批 1：誓师（`shishi`）+ 循梦（`xunmeng`）。
//!
//! 上游 `extension/末日乱斗/extension.js`：
//! - `shishi`：`trigger {source:'dieAfter'}` 锁定——`gainMaxHp(1)` + `recover()`；
//! - `xunmeng`：`trigger {source:'damageBegin'}` 锁定——伤害载体为【杀】且父事件
//!   非铁索（`_lianhuan*`）→ `trigger.num++`；若自己体力 >1 则 `loseHp()`。

use engine_native::event::DamageNature;
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

/// 誓师：击杀 3 号后体力上限 +1 且回复 1。
#[test]
fn shishi_gains_max_hp_and_recovers() {
    let mut e = db_engine(155001);
    e.debug_grant(0, "shishi");
    assert!(e.catalog().skill_by_name("shishi").is_some());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(3, 1);
    let max_before = e.state_core().player(0).max_hp;
    let hp_before = e.debug_hp(0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![3] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_alive(3), "3 号阵亡");
    assert_eq!(
        e.state_core().player(0).max_hp,
        max_before + 1,
        "誓师：体力上限 +1"
    );
    assert_eq!(e.debug_hp(0), hp_before + 1, "誓师：回复 1");
}

/// 循梦：以【杀】造成伤害 +1，且自己体力 >1 时失去 1 点体力。
#[test]
fn xunmeng_adds_damage_and_loses_hp() {
    let mut e = db_engine(155002);
    e.debug_grant(0, "xunmeng");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    e.debug_set_hp(0, 3);
    let hp0_before = e.debug_hp(0);
    let hp1_before = e.debug_hp(1);
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
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before - 2, "循梦：杀伤害 +1（共 2）");
    assert_eq!(e.debug_hp(0), hp0_before - 1, "循梦：自己失去 1 点体力");
}

/// 循梦（对照）：未持技者正常 1 点伤害、自己不失去体力。
#[test]
fn xunmeng_absent_normal() {
    let mut e = db_engine(155003);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    e.debug_set_hp(0, 3);
    let hp0_before = e.debug_hp(0);
    let hp1_before = e.debug_hp(1);
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
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before - 1, "未持技：1 点伤害");
    assert_eq!(e.debug_hp(0), hp0_before, "未持技：自己不失去体力");
    let _ = DamageNature::Normal;
}
