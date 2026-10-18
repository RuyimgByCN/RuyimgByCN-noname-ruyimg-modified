//! NC08 第⑦层（上游行为对照）样本 10：`gw_yanziyaoshui`（燕子药水，U05 gwent 卡牌子包）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js`）：
//!
//! ```js
//! gw_yanziyaoshui: {
//!     fullborder: "bronze",
//!     type: "spell", subtype: "spell_bronze",
//!     enable: true,
//!     filterTarget: true,                 // 任意合法目标（含自己）
//!     content() {
//!         if (target.isMinHandcard()) {   // 目标是**手牌最少者**（之一）
//!             target.draw(2);
//!         } else {
//!             target.draw();
//!         }
//!     },
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **恰 1 名合法目标**（`filterTarget: true`，无 `selectTarget` 限制 → 单目标；含自己）；
//! 2. **目标是手牌最少者（之一）→ 摸 2**；
//! 3. **否则 → 摸 1**。
//!
//! 原生实现引用：`engine/play_card.rs:1508`（目标校验：恰 1 名存活）、
//! `kernel/intrinsic.rs:363`（无懈窗口包装 + `GwentYanzhiCard`）、
//! `content/host/step/gwent.rs:6`（效果：`alive_players().all(hand.len() >= t.hand.len())` → 2/1）。
//!
//! 未覆盖（登记）：`fullborder`/`subtype`/`ai` 属表现与策略面；无懈窗口本身由既有窗口测试覆盖。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 按**前缀**取目录牌名（gwent 卡名带花色点数后缀，如 `gw_yanziyaoshui#heart8`），
/// 再用 `debug_place`（精确名匹配）置入指定玩家手牌。
fn place_card_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.starts_with(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

fn gwent_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gwent-upstream-noname/assets/cards.json"))
            .expect("读 gwent 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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

/// 上游点 2：目标为**手牌最少者**（此处手牌最少）→ 摸 2 张。
#[test]
fn upstream_yanziyaoshui_draws_two_for_min_hand() {
    let mut e = gwent_engine(20263501);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标 1 手牌 1 张（最少）；其他人 3 张
    let c1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, c1);
    for p in [0u8, 2, 3] {
        for _ in 0..3 {
            let c = e.debug_place(p, "shan");
            e.debug_give_hand_card(p, p, c);
        }
    }
    let yanzi = place_card_by_prefix(&mut e, "gw_yanziyaoshui", 0);
    let before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: yanzi, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.debug_hand(1).len(),
        before + 2,
        "上游 isMinHandcard() → draw(2)"
    );
}

/// 上游点 3：目标**不是**手牌最少者 → 只摸 1 张。
#[test]
fn upstream_yanziyaoshui_draws_one_otherwise() {
    let mut e = gwent_engine(20263502);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标 1 手牌 3 张；2 号只有 1 张 → 1 号不是最少者
    for _ in 0..3 {
        let c = e.debug_place(1, "sha");
        e.debug_give_hand_card(1, 1, c);
    }
    let c2 = e.debug_place(2, "shan");
    e.debug_give_hand_card(2, 2, c2);
    let yanzi = place_card_by_prefix(&mut e, "gw_yanziyaoshui", 0);
    let before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: yanzi, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), before + 1, "非最少者 → draw(1)");
}

/// 上游点 1：目标面为**恰 1 名**——空目标表与多目标均被拒。
#[test]
fn upstream_yanziyaoshui_requires_exactly_one_target() {
    let mut e = gwent_engine(20263503);
    advance_to_play(&mut e, 0);
    let yanzi = place_card_by_prefix(&mut e, "gw_yanziyaoshui", 0);
    let none = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: yanzi, targets: vec![] },
    });
    assert!(
        matches!(none, CommandOutcome::Rejected(_)),
        "空目标表必须被拒（上游为单目标法术）：{none:?}"
    );
    let two = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: yanzi, targets: vec![1, 2] },
    });
    assert!(
        matches!(two, CommandOutcome::Rejected(_)),
        "多目标必须被拒（上游为单目标法术）：{two:?}"
    );
}
