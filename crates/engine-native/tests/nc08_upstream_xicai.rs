//! NC08 第⑦层（上游行为对照）样本 9：`xicai`（戏才，U05 运筹帷幄 / 蔡文姬·DIY 面）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`）：
//!
//! ```js
//! xicai: {
//!     preHidden: true,
//!     trigger: { player: "damageEnd" },                 // 受到伤害后
//!     filter(event, player) {
//!         return get.itemtype(event.cards) === "cards"   // 伤害带牌
//!             && get.position(event.cards[0], true) === "o";  // 牌在**处理区**
//!     },
//!     async content(event, trigger, player) {
//!         await player.gain(trigger.cards, "gain2");     // 获得这些伤害牌
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **触发时机 = 受到伤害后**（`damageEnd`）；
//! 2. **门槛 = 伤害带牌且牌在处理区**（`position === "o"`，即尚未入弃牌堆）；
//! 3. **效果 = 获得这些伤害牌**（`gain`）。
//!
//! 原生实现引用：`content/host/filtering/common.rs:521`（过滤）、
//! `content/host/step/yunchou.rs:413`（效果：table 优先、弃堆兜底，取回入手并记 `CardsGiven`）。
//!
//! **登记差异**：原生不校验「处理区」位置，改为 **table 优先、弃牌堆兜底** —— 因此当伤害牌
//! 已经入弃牌堆时，上游（`position === "o"` 为假）**不触发**，原生仍会取回。本文件用
//! **源码事实固定**记录该差异，不写成对齐。
//!
//! 未覆盖（登记）：`preHidden`/`ai` 属表现与策略面。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn yunchou_engine(seed: u64) -> Engine {
    // 运筹帷幄的**武将面**在 `character.json`（cards.json 只有卡牌段）：双产物桥接
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹帷幄 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
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

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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

/// 上游点 1+3：受到**带牌**伤害后，伤害牌被戏才持有者获得（不入弃牌堆）。
///
/// 走**真实路径**：0 号对 1 号（戏才持有者）出【杀】→ 伤害牌在**处理区**（原生 `table`）→
/// 戏才取回。此前用 `debug_deal_damage_with_card` 失败的原因：该助手只把 `card` 写入
/// `DamageData`，**不把牌放入处理区**，而原生效果按「table 优先、弃堆兜底」取牌 → 取不到。
#[test]
fn upstream_xicai_gains_damage_card() {
    let mut e = yunchou_engine(20263401);
    e.debug_grant(1, "ycnxicai"); // 持有者为**受伤者**
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hand1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 戏才为可选触发：确认发动
    let mut confirmed = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    drain(&mut e);
    assert!(confirmed, "戏才应经触发确认门（可选技）");
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before + 1,
        "上游 gain(trigger.cards)：持有者获得伤害牌（【杀】）"
    );
    assert!(
        e.debug_hand(1).contains(&sha),
        "获得的应是**该伤害牌**（{sha}）：{:?}",
        e.debug_hand(1)
    );
    assert!(
        !e.state_core().discard.contains(&sha),
        "伤害牌不应留在弃牌堆（已被获得）"
    );
}

/// 上游点 2 的**登记差异**：原生不校验「处理区」位置（table 优先、弃堆兜底）。
#[test]
fn upstream_xicai_position_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou 源码");
    let idx = src.find("HostBehavior::YunchouXicai =>").expect("戏才分支");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        seg.contains("ctx.core.discard.iter().position") && seg.contains("table.iter().position"),
        "差异固定：原生为 table 优先 + 弃堆兜底（上游要求 position === 'o' 处理区）：{seg}"
    );
    // 行为面：未受伤/无带牌伤害时不应触发（过滤臂要求 Damage 事件）
    let mut e = yunchou_engine(20263402);
    e.debug_grant(0, "ycnxicai");
    let hand_before = e.debug_hand(0).len();
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before,
        "无带牌伤害（无牌可获）→ 手牌不变"
    );
}
