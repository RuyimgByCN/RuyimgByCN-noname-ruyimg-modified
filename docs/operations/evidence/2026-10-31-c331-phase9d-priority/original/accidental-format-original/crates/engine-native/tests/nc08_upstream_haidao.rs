//! NC08 第⑦层（上游行为对照）样本 13：`haidao`（海岛，U05 mtg 卡牌子包 / 地）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! haidao: {
//!     type: "land",
//!     fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },  // 每回合限一张地
//!     notarget: true,                                                  // 无目标
//!     content() {
//!         "step 0";
//!             game.changeLand("mtg_haidao", player);                   // 换地
//!             if (player.isHealthy()) { player.changeHujia(); event.finish(); }  // 满血 → +1 护甲
//!             else { player.chooseToDiscard("he", "海岛") … }          // 否则：弃 1 张 / 取消
//!         "step 1";
//!             if (result.bool) player.recover(); else player.changeHujia();      // 回复 1 / +1 护甲
//!         "step 2"; delete player._temp_mtg_haidao;
//!     },
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **每回合限一张地**（`enable: !hasSkill("land_used")`）；
//! 2. **无目标**（`notarget: true` → 空目标表）；
//! 3. **满血 → 直接 +1 护甲**（`isHealthy()` 分支，不再询问）；
//! 4. **非满血 → 二选一**：弃 1 张（手牌或装备）→ **回复 1**；**取消** → **+1 护甲**。
//!
//! 原生实现引用：`content/host/step/mtg.rs:60`（Act：换地/`land_used`/满血 +1 甲/否则询问）、
//! `:100`（SelectCard：弃牌 → 回复，取消 → +1 甲）。
//!
//! 未覆盖（登记）：`fullborder`/`ai` 属表现与策略面；`_temp_mtg_haidao` 为上游内部标记。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn mtg_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-mtg-upstream-noname/assets/cards.json"))
        .expect("读 mtg 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 上游点 2+3：无目标 + **满血** → 直接 +1 护甲（不出现询问）。
#[test]
fn upstream_haidao_healthy_grants_hujia_without_prompt() {
    let mut e = mtg_engine(20264001);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 4); // 满血（上限 4）
    let land = place_by_prefix(&mut e, "haidao", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: land,
            targets: vec![],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "满血分支不应出现询问（上游 event.finish()）：{:?}",
        e.open_choice().map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert!(
        e.state_core().turn.land_used,
        "换地后本回合地已用（land_used）"
    );
}

/// 上游点 4（弃牌支）：非满血 → 弃 1 张 → **回复 1**。
#[test]
fn upstream_haidao_damaged_discard_recovers() {
    let mut e = mtg_engine(20264002);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let filler = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, filler);
    e.debug_set_hp(0, 2);
    let land = place_by_prefix(&mut e, "haidao", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: land,
            targets: vec![],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let c = e.open_choice().cloned().expect("非满血应出现弃牌询问");
    assert!(
        c.spec.allow_cancel,
        "询问应可取消（上游 chooseToDiscard 可取消）"
    );
    let hp_before = e.debug_hp(0);
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(filler)],
        },
    });
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_before + 1, "弃牌支 → 回复 1 点体力");
}

/// 上游点 4（取消支）：非满血 → **取消** → +1 护甲（体力不变）。
#[test]
fn upstream_haidao_damaged_cancel_grants_hujia() {
    let mut e = mtg_engine(20264003);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let filler = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, filler);
    e.debug_set_hp(0, 2);
    let land = place_by_prefix(&mut e, "haidao", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: land,
            targets: vec![],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let c = e.open_choice().cloned().expect("非满血应出现弃牌询问");
    let hp_before = e.debug_hp(0);
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: Vec::new(),
        },
    });
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_before, "取消支 → 体力不变");
    assert_eq!(e.debug_hand(0).len(), 1, "取消支 → 不弃牌");
}

/// 上游点 1：每回合限一张地——同回合第二张地被拒。
#[test]
fn upstream_haidao_is_once_per_turn() {
    let mut e = mtg_engine(20264004);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 4);
    let first = place_by_prefix(&mut e, "haidao", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: first,
            targets: vec![],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    // 换用**另一张地**（包内 haidao 只有一张）：规则是「每回合限一张地」，与具体地无关
    let second = place_by_prefix(&mut e, "yixialan", 0);
    let again = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: second,
            targets: vec![],
        },
    });
    assert!(
        matches!(again, CommandOutcome::Rejected(_)),
        "同回合第二张地应被拒（上游 enable !hasSkill('land_used')）：{again:?}"
    );
}
