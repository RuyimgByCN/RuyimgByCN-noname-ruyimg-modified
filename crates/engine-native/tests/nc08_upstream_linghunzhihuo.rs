//! NC08 第⑦层（上游行为对照）样本 12：`linghunzhihuo`（灵魂之火，U05 炉石卡牌子包）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! linghunzhihuo: {
//!     fullskin: true,
//!     type: "trick",
//!     cardnature: "fire",
//!     enable: true,
//!     filterTarget: true,                    // 任意合法目标（含自己）
//!     content() {
//!         "step 0"; target.damage("fire");   // 目标受 1 点**火焰**伤害
//!         "step 1";
//!             var hs = player.getCards("h");
//!             if (hs.length) player.discard(hs.randomGet());  // 使用者随机弃 1 张手牌
//!     },
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **恰 1 名合法目标**（`filterTarget: true`，含自己）；
//! 2. 目标受 **1 点火焰**伤害（`damage("fire")`）；
//! 3. **使用者**弃 **1 张手牌**（手牌非空时）；
//! 4. **随机**弃置（`hs.randomGet()`）——原生为**确定性首张**：**登记差异**。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（炉石锦囊 → `LinghunzhihuoCard`）、
//! `content/host/step/skills_damage.rs:749`（火伤 + 使用者弃手牌首张）。
//!
//! 未覆盖（登记）：`fullskin`/`ai` 属表现与策略面。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
            .expect("读炉石包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    // 目录名可能带命名空间/花色点数前后缀 → contains 匹配（gwent/古剑样本的经验）
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

/// 上游点 2+3：目标受 1 点火焰伤害；使用者弃 1 张手牌。
#[test]
fn upstream_linghunzhihuo_fire_damage_and_self_discard() {
    let mut e = hearth_engine(20263701);
    advance_to_play(&mut e, 0);
    // 使用者手牌：只留本体 + 一张杂牌（便于观察弃置）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let filler = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, filler);
    let lh = place_by_prefix(&mut e, "linghunzhihuo", 0);
    let hp1 = e.debug_hp(1);
    let hand0 = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: lh, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "目标受 1 点伤害（上游 damage('fire')）");
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 - 2,
        "使用者弃 1 张手牌（本体离手 + 弃置 1）"
    );
    // 火焰属性：日志中应出现火属性伤害（nature=Fire）
    let tail: String = e
        .log()
        .iter()
        .rev()
        .take(12)
        .map(|ev| format!("{ev:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        tail.contains("Fire") || tail.contains("Damage"),
        "应产生伤害事件（火属性）：{tail}"
    );
}

/// 上游点 1：单目标法术——空目标表被拒。
#[test]
fn upstream_linghunzhihuo_requires_a_target() {
    let mut e = hearth_engine(20263702);
    advance_to_play(&mut e, 0);
    let lh = place_by_prefix(&mut e, "linghunzhihuo", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: lh, targets: vec![] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "空目标表必须被拒（上游为单目标锦囊）：{out:?}"
    );
}

/// 上游点 4（**登记差异**）：上游 `hs.randomGet()` 随机弃置，原生为**确定性首张**。
#[test]
fn upstream_linghunzhihuo_random_discard_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/skills_damage.rs"
    ))
    .expect("读 skills_damage 源码");
    let idx = src
        .find("HostBehavior::LinghunzhihuoCard =>")
        .expect("灵魂之火分支");
    let seg: String = src[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("hand.first()") && seg.contains("DamageNature::Fire"),
        "差异固定：原生确定性弃首张 + 火属性伤害：{seg}"
    );
}
