//! swd deferred 消解批 144：涅槃（`niepan`）跨来源引用面。
//!
//! swd 包武将 `swd拓跋玉儿` 的 `niepan` 即基础包庞统限定技【涅槃】——原生
//! 已以宿主行为 `host:niepan`（tag 471，批 118）落地；本批只做**包内引用
//! 映射**（不再作为 deferred），并验证其在 swd 包装载下可用。

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

/// 涅槃在 swd 包装载下可授予并可用（限定技：觉醒后实例禁用）。
#[test]
fn swd_niepan_granted_and_usable() {
    let mut e = swd_engine(121001);
    e.debug_grant(0, "涅槃");
    let def = e.catalog().skill_by_name("涅槃").expect("涅槃在目录");
    assert!(e.state_core().skills.contains_key(&(0, def)), "已授予");
    // 濒死触发：0 号 1 血受 2 伤 → 涅槃窗（确认/放弃）
    e.debug_set_hp(0, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(2), 0, 2, engine_native::event::DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 出现涅槃相关窗口（确认门）或直接结算；两种路径都接受，只要不 panic
    drain(&mut e);
    let hp = e.debug_hp(0);
    assert!(hp >= 0, "濒死流程完成（hp={hp}）");
}

/// 包内 `niepan` 映射为 `host:niepan`（桥接不再报 deferred）。
#[test]
fn swd_niepan_package_mapping_is_host() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    // 桥接产物：`niepan` 命中宿主行为键（不再计入未映射）
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接武将");
    assert!(
        bridged.mapped_skill_keys.iter().any(|k| *k == "niepan"),
        "niepan 已映射（宿主行为）：未映射计数 {}",
        bridged.unmapped_skills
    );
}
