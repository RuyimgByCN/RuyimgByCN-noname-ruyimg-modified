//! swd deferred 消解批 130：天香（`tianxiang`）+ 天香·受（`tianxiang2`）。
//!
//! 上游 `character/shenhua.js:6501`：`damageBegin3` 可选——弃一张红桃手牌，
//! 把该伤害转移给一名其他角色（`trigger.player = target`）；该角色获得
//! `tianxiang2`（`damageAfter` forced）：按本次失去的体力摸等量牌后移除。

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
    for _ in 0..48 {
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
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
                selection: fallback,
            },
        });
    }
    false
}

/// 天香：受到 2 点伤害前弃红桃转移给 1 号 → 1 号受 2 伤并按失去体力摸 2；
/// 自己不受伤害、红桃牌离手。
#[test]
fn tianxiang_redirects_damage_and_target_draws() {
    let mut e = swd_engine(107001);
    e.debug_grant(0, "tianxiang");
    assert!(e.catalog().skill_by_name("tianxiang").is_some());
    assert!(
        e.catalog().skill_by_name("tianxiang2").is_some(),
        "天香·受在目录（包级声明）"
    );
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let heart = e.debug_place_suit(0, engine_native::catalog::Suit::Heart);
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(2), 0, 2, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "天香确认门"
    );
    assert!(
        answer(&mut e, "tianxiang_heart", vec![ChoiceOption::Card(heart)]),
        "天香选红桃窗"
    );
    assert!(
        answer(&mut e, "tianxiang_target", vec![ChoiceOption::Player(1)]),
        "天香选目标窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0, "伤害已转移：自己不受伤害");
    assert_eq!(e.debug_hp(1), hp1 - 2, "1 号承受 2 点伤害");
    assert_eq!(e.debug_hand(1).len(), 2, "1 号按失去体力摸 2");
    assert!(!e.debug_hand(0).contains(&heart), "红桃牌已弃");
    let t2 = e.catalog().skill_by_name("tianxiang2").expect("天香·受");
    assert!(
        !e.state_core().skills.contains_key(&(1, t2)),
        "天香·受用后移除"
    );
}

/// 天香（对照）：手中无红桃 → 不开窗（伤害照常由自己承受）。
#[test]
fn tianxiang_no_window_without_heart() {
    let mut e = swd_engine(107002);
    e.debug_grant(0, "tianxiang");
    e.debug_clear_hand(0);
    let spade = e.debug_place_suit(0, engine_native::catalog::Suit::Spade);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "无红桃不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "伤害照常");
    assert!(e.debug_hand(0).contains(&spade), "黑桃未弃");
}

/// 天香（对照）：放弃选择 → 不转移（伤害照常由自己承受）。
#[test]
fn tianxiang_decline_keeps_damage() {
    let mut e = swd_engine(107003);
    e.debug_grant(0, "tianxiang");
    e.debug_clear_hand(0);
    let heart = e.debug_place_suit(0, engine_native::catalog::Suit::Heart);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "天香确认门"
    );
    assert!(
        answer(&mut e, "tianxiang_heart", Vec::new()),
        "取消选红桃窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "取消 → 伤害不转移");
    assert!(e.debug_hand(0).contains(&heart), "红桃未弃");
}

/// 天香（对照）：未持技者受伤不开窗；天香·受由运行时授予（存档往返保持）。
#[test]
fn tianxiang_absent_and_grant_roundtrip() {
    let mut e = swd_engine(107004);
    e.debug_clear_hand(0);
    let _heart = e.debug_place_suit(0, engine_native::catalog::Suit::Heart);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1);
    // 运行时授予天香·受 → 存档往返保持实例
    let t2 = e.catalog().skill_by_name("tianxiang2").expect("天香·受");
    e.state_core_mut().skills.insert(
        (1, t2),
        engine_native::state::SkillInstance {
            owner: 1,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    let snap = e.save();
    assert!(snap.core.skills.contains_key(&(1, t2)));
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().skills.contains_key(&(1, t2)));
}
