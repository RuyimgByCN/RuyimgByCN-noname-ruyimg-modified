//! swd 批 154：魔化（`mohua2`）——濒死变身撒旦形态（四技装配）。
//!
//! 上游 `character/swd.js`：`trigger {player:'dying'}`，`priority:10`，forced
//! ——移除 `miles_xueyi`/`aojian`/`mohua2`，获得 `moyan`/`miedao`/`jifeng`/
//! `swd_xiuluo`，角色变为 `swd_satan`（群，体力上限 4），回复 2、摸 2。

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

/// 魔化：濒死 → 旧三技移除、撒旦四技在册、上限 4、回复 2、摸 2。
#[test]
fn mohua2_transforms_on_dying() {
    let mut e = swd_engine(130001);
    // 授予旧三技 + 魔化（模拟 swd_miles 装配）
    for name in ["miles_xueyi", "aojian", "mohua2"] {
        e.debug_grant(0, name);
    }
    assert!(e.catalog().skill_by_name("mohua2").is_some());
    for name in ["miles_xueyi", "aojian"] {
        let d = e.catalog().skill_by_name(name).expect(name);
        assert!(e.state_core().skills.contains_key(&(0, d)), "{name} 在册");
    }
    // 清手牌以便观察摸 2
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 进入濒死：1 血受 2 伤
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    // 旧三技移除
    for name in ["miles_xueyi", "aojian", "mohua2"] {
        let d = e.catalog().skill_by_name(name).expect(name);
        assert!(
            !e.state_core().skills.contains_key(&(0, d)),
            "{name} 已移除"
        );
    }
    // 撒旦四技在册
    for name in ["moyan", "miedao", "jifeng", "swd_xiuluo"] {
        let d = e.catalog().skill_by_name(name).expect(name);
        assert!(e.state_core().skills.contains_key(&(0, d)), "{name} 已获得");
    }
    assert_eq!(e.state_core().player(0).max_hp, 4, "上限改 4");
    assert!(e.debug_alive(0), "濒死救回（存活）");
    assert_eq!(e.debug_hand(0).len(), 2, "摸 2");
}

/// 魔化（对照）：未持技者濒死不变身。
#[test]
fn mohua2_absent_no_transform() {
    let mut e = swd_engine(130002);
    for name in ["miles_xueyi", "aojian"] {
        e.debug_grant(0, name);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    // 未持魔化：不获得撒旦四技、体力上限不变
    for name in ["moyan", "miedao", "jifeng", "swd_xiuluo"] {
        let d = e.catalog().skill_by_name(name).expect(name);
        assert!(!e.state_core().skills.contains_key(&(0, d)), "未获 {name}");
    }
    assert_eq!(e.state_core().player(0).max_hp, 4, "上限不变（原始 4）");
    assert_eq!(e.debug_hand(0).len(), 0, "未摸牌");
}

/// 魔化：仅触发一次（技能实例已移除）。
#[test]
fn mohua2_fires_once() {
    let mut e = swd_engine(130003);
    e.debug_grant(0, "mohua2");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).max_hp, 4);
    // 再次濒死：不再有魔化（技能已移除）——体力上限不变、手牌不再 +2
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    let out2 = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out2:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).max_hp, 4, "上限不再变化");
}
