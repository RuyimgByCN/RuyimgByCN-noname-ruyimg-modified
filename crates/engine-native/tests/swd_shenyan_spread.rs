//! swd deferred 消解批 121：神焰（`shenyan`）——火伤 +1 + 邻位蔓延。
//!
//! 上游 `character/swd.js`（`shenyan` 51 行 + `shenyan2` 17 行）：限定技，
//! 自己造成的**火属性**伤害 +1（`trigger.num++`），并记录原目标距离 ≤1 的
//! 其他角色（座位序、不含来源与原目标）；主伤害结算后（`global damageAfter`）
//! 逐个对其造成 1 点**无来源**火伤，随后移除。

use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
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
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
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

/// 火伤 +1 且邻位蔓延：0 号（神焰）火伤 1 号 → 1 号受 2；1 号的邻位
/// （座位序：2 号距离 1；3 号距离 1——环形 4 人）各受 1 无来源火伤；
/// 0 号（来源）不受。
#[test]
fn shenyan_boosts_fire_and_spreads_to_neighbors() {
    let mut e = swd_engine(98001);
    e.debug_grant(0, "shenyan");
    assert!(
        e.catalog().skill_by_name("shenyan").is_some(),
        "神焰在目录（宿主行为）"
    );
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let hp3 = e.debug_hp(3);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 2, "火伤 1 → 神焰 +1 = 2");
    // 4 人环：1 号距离 1 的是 0 号与 2 号；排除来源（0）与原目标（1）→ 仅 2 号
    assert_eq!(e.debug_hp(2), hp2 - 1, "邻位 2 号蔓延 1 点火伤");
    assert_eq!(e.debug_hp(3), hp3, "非邻位 3 号不受影响");
    assert_eq!(e.debug_hp(0), hp0, "来源不受自己的蔓延");
    assert!(e.state_core().player(0).shenyan_used, "限定位已置");
    assert!(
        e.state_core().player(0).shenyan_pending.is_empty(),
        "蔓延队列已清"
    );
}

/// 对照：非火属性伤害不触发（不加成、不蔓延、限定位不置）。
#[test]
fn shenyan_ignores_non_fire_damage() {
    let mut e = swd_engine(98002);
    e.debug_grant(0, "shenyan");
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "普通伤害不加成");
    assert_eq!(e.debug_hp(2), hp2, "不蔓延");
    assert!(!e.state_core().player(0).shenyan_used, "限定位未置");
}

/// 限定技：第二次火伤不再加成、不蔓延。
#[test]
fn shenyan_limited_once() {
    let mut e = swd_engine(98003);
    e.debug_grant(0, "shenyan");
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "第二次火伤不加成");
    assert_eq!(e.debug_hp(2), hp2, "第二次不蔓延");
}

/// 对照：他人造成的火伤不加成（来源非持有者）。
#[test]
fn shenyan_only_own_fire_damage() {
    let mut e = swd_engine(98004);
    e.debug_grant(0, "shenyan");
    let hp1 = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(2), 1, 1, DamageNature::Fire);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "他人火伤不加成");
    assert!(!e.state_core().player(0).shenyan_used);
}

/// 存档往返：限定位与待蔓延队列保持。
#[test]
fn shenyan_state_roundtrip() {
    let mut e = swd_engine(98005);
    e.debug_grant(0, "shenyan");
    {
        let p = e.state_core_mut().player_mut(0);
        p.shenyan_used = true;
        p.shenyan_pending = vec![2, 3];
    }
    let snap = e.save();
    assert!(snap.core.players[0].shenyan_used);
    assert_eq!(snap.core.players[0].shenyan_pending, vec![2, 3]);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().player(0).shenyan_used);
    assert_eq!(restored.state_core().player(0).shenyan_pending, vec![2, 3]);
}
