//! L20 标记补充批 1：摸牌翻倍（`_xigua_mopaifanbei_`）+ 血量翻倍（`_xigua_xueliangfanbei_`）。
//!
//! 上游 `extension/标记补充/extension.js`（配置门控模式修正）：
//! - `_xigua_mopaifanbei_`：`trigger {player:'phaseDrawBegin2'}` → `trigger.num += 2`；
//! - `_xigua_xueliangfanbei_`：`trigger {global:'gameStart'}` → `gainMaxHp(hp)` + `recover(hp)`。
//! 原生以**技能实例启用**承载上游 `lib.config` 门控（宿主按配置注入，登记差异）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn ms_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读标记补充包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
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

/// 摸牌翻倍：持技者摸牌阶段额外摸 2 张。
#[test]
fn mopai_doubles_draw() {
    let mut e = ms_engine(159001);
    e.debug_grant(0, "xigua_mopaifanbei");
    assert!(e.catalog().skill_by_name("xigua_mopaifanbei").is_some());
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge → Draw
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 4, "正常 2 + 额外 2");
}

/// 血量翻倍：持技者首次阶段开始时上限与体力翻倍。
#[test]
fn xueliang_doubles_hp() {
    let mut e = ms_engine(159002);
    e.debug_grant(0, "xigua_xueliangfanbei");
    e.debug_set_hp(0, 3);
    let max_before = e.state_core().player(0).max_hp;
    // 推进到任意阶段开始（首次触发）
    next_phase(&mut e);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).max_hp,
        max_before + 3,
        "上限 + 当前体力(3)"
    );
    assert_eq!(e.debug_hp(0), 6, "回复 3（3+3）");
    // 二次阶段开始不再翻倍
    next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).max_hp, max_before + 3, "仅一次");
}

/// 对照：未持技者正常摸 2、体力不翻倍。
#[test]
fn absent_no_mods() {
    let mut e = ms_engine(159003);
    e.debug_set_hp(0, 3);
    let max_before = e.state_core().player(0).max_hp;
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 2, "未持技：正常摸 2");
    assert_eq!(
        e.state_core().player(0).max_hp,
        max_before,
        "未持技：上限不变"
    );
}
