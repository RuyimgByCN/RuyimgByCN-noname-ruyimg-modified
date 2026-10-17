//! swd deferred 消解批 124：守印（`shouyin`）——濒死全员回复限定技。
//!
//! 上游 `character/swd.js`：`enable: chooseToUse` + `save:true`，`filter`：
//! 濒死窗、未发动、未翻面；效果：`awakenSkill` + `turnOver()`（自翻面）+
//! 按座位序将全部已受伤角色回复至满体力（`recover(maxHp - hp)`）。

use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
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
    for _ in 0..48 {
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

/// 守印：任意角色濒死时发动 → 自翻面 + 全员回复至满；限一次。
#[test]
fn shouyin_flips_self_and_heals_everyone_once() {
    let mut e = swd_engine(101001);
    e.debug_grant(2, "shouyin");
    assert!(e.catalog().skill_by_name("shouyin").is_some());
    // 布阵：0 号 1 血（将被打到濒死）、1 号 2/4、2 号（持技）3/4、3 号满血
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 2);
    e.debug_set_max_hp(2, 4);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(0, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("濒死窗");
    assert_eq!(c.spec.prompt, "trigger_confirm", "守印可选触发确认门");
    assert_eq!(c.observer, 2, "观察者=持技者");
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 4, "0 号回复至满（maxHp 4）");
    assert_eq!(e.debug_hp(1), 4, "1 号回复至满 4");
    assert_eq!(e.debug_hp(2), 4, "2 号回复至满 4");
    assert!(e.state_core().player(2).face_down, "自翻面");
    assert!(e.state_core().player(2).shouyin_used, "限定技已发动");
    assert!(e.debug_alive(0), "濒死被救回");
    // 限一次：再次濒死不再开窗
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.observer != 2 || c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "限定技不再开窗：{:?}",
        c.map(|c| (c.observer, c.spec.prompt))
    );
    drain(&mut e);
}

/// 守印（对照）：持技者已翻面 → 不开窗。
#[test]
fn shouyin_no_window_when_flipped() {
    let mut e = swd_engine(101002);
    e.debug_grant(2, "shouyin");
    e.debug_set_face_down(2, true);
    e.debug_set_hp(0, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| !(c.observer == 2 && c.spec.prompt == "trigger_confirm"))
            .unwrap_or(true),
        "已翻面不开窗：{:?}",
        c.map(|c| (c.observer, c.spec.prompt))
    );
    drain(&mut e);
    assert!(!e.state_core().player(2).shouyin_used, "未发动");
}

/// 守印（对照）：未持技者濒死不触发（无窗）。
#[test]
fn shouyin_absent_no_window() {
    let mut e = swd_engine(101003);
    assert!(e.catalog().skill_by_name("shouyin").is_some());
    e.debug_set_hp(0, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "无守印不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 守印：存档往返保持限定位与翻面。
#[test]
fn shouyin_state_roundtrip() {
    let mut e = swd_engine(101004);
    {
        let p = e.state_core_mut().player_mut(2);
        p.shouyin_used = true;
        p.face_down = true;
    }
    let snap = e.save();
    assert!(snap.core.players[2].shouyin_used);
    assert!(snap.core.players[2].face_down);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().player(2).shouyin_used);
    assert!(restored.state_core().player(2).face_down);
}
