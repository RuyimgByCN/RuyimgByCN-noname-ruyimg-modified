//! swd deferred 消解批 146：轩辕剑（卡 + 三子技）+ 轩辕（取剑）。
//!
//! 上游 `card/swd.js`：`xuanyuanjian`（独特武器，攻击范围 +2，装备时 +1
//! 护甲；体力 ≤2 且无「轩辕」则弃置）+ `xuanyuanjian2`（造成伤害 +1 且转
//! 雷属性）+ `xuanyuanjian3`（加伤后无「轩辕」则失去 1 体力）；`swd.js`
//! `xuanyuan`（结束阶段限定技：弃黑桃牌，从弃牌堆取【轩辕剑】装备）。

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

/// 轩辕剑（对照）：持「轩辕」时无反噬。
#[test]
fn xuanyuanjian_no_backlash_with_xuanyuan() {
    let mut e = swd_engine(123002);
    e.debug_grant(0, "xuanyuanjian2");
    e.debug_grant(0, "xuanyuanjian3");
    e.debug_grant(0, "xuanyuan");
    let hp2 = e.debug_hp(2);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2 - 2, "加伤 1");
    assert_eq!(e.debug_hp(0), hp0, "持「轩辕」不反噬");
}

/// 轩辕剑：`xuanyuanjian_marked` 字段存档往返。
#[test]
fn xuanyuanjian_state_roundtrip() {
    let mut e = swd_engine(123004);
    e.state_core_mut().turn.xuanyuanjian_marked = true;
    let snap = e.save();
    assert!(snap.core.turn.xuanyuanjian_marked);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().turn.xuanyuanjian_marked);
}
