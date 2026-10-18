//! L27 选将美化批 1：撷芳（`xiefang`）——距离 -存活女性数 + 女性数标记。
//!
//! 上游 `extension/选将美化/extension.js`：`mod.globalFrom` 距离
//! `- countPlayer(hasSex('female'))`；`trigger {global:['dieEnd','gameDrawEnd']}`
//! 锁定刷新 `xiefang_mark`（存活女性数；0 时移除）。

use engine_native::content::standard::Gender;
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

/// 撷芳：距离按存活女性数减少；标记随女性数刷新。
#[test]
fn xiefang_reduces_distance_by_alive_females() {
    let mut e = sb_engine(150001);
    e.debug_grant(0, "xiefang");
    assert!(e.catalog().skill_by_name("xiefang").is_some());
    // 基线距离（无女性武将时 -0）
    let base_dist = |e: &Engine, a: u8, b: u8| {
        e.state_core().computed_distance(e.catalog(), a, b)
    };
    let before = base_dist(&e, 0, 2);
    // 把 2 号设为女性（主将槽换成女性武将）
    let female = (0..e.catalog().general_count() as u16)
        .find(|g| {
            e.catalog()
                .general_meta(*g)
                .map(|(_, _, _, _, gender)| gender == Gender::Female)
                .unwrap_or(false)
        })
        .expect("女性武将");
    e.debug_set_general(2, &e.catalog().general_meta(female).unwrap().0);
    assert_eq!(e.state_core().effective_gender(2), Gender::Female);
    // 触发一次阶段结束（推进摸牌阶段结束）刷新标记
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e); // Draw → Play（摸牌阶段结束触发撷芳）
    drain(&mut e);
    let after = base_dist(&e, 0, 2);
    assert!(
        after < before,
        "存活女性数 1 → 距离减少（{before} → {after}）"
    );
    let def = e.catalog().skill_by_name("xiefang").expect("xiefang");
    let mark = e
        .state_core()
        .vm_aux
        .marks
        .get(&(0, u32::from(def)))
        .copied();
    assert_eq!(mark, Some(1), "标记=存活女性数");
}

/// 撷芳（对照）：未持技者距离不减、无标记。
#[test]
fn xiefang_absent_no_effect() {
    let mut e = sb_engine(150002);
    let female = (0..e.catalog().general_count() as u16)
        .find(|g| {
            e.catalog()
                .general_meta(*g)
                .map(|(_, _, _, _, gender)| gender == Gender::Female)
                .unwrap_or(false)
        })
        .expect("女性武将");
    let name = e.catalog().general_meta(female).unwrap().0;
    e.debug_set_general(2, &name);
    let before = e.state_core().computed_distance(e.catalog(), 0, 2);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    drain(&mut e);
    let after = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(before, after, "未持技：距离不变");
    let def = e.catalog().skill_by_name("xiefang").expect("xiefang");
    assert!(
        e.state_core().vm_aux.marks.get(&(0, u32::from(def))).is_none(),
        "未持技：无标记"
    );
}
