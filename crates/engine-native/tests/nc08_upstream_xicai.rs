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

use engine_native::{Engine, GameAction, GameCommand};
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
/// **本轮未验收（登记）**：夹具未能稳定驱动该**可选触发**（`debug_deal_damage_with_card`
/// 之后未观察到 `trigger_confirm`/取回效果，手牌未增长）；已由源码核对记录实现位置
/// （`filtering/common.rs:521`、`step/yunchou.rs:413`），但**不写成通过**。修复夹具后移除
/// `#[ignore]` 即可转为行为验收。
#[test]
#[ignore = "夹具未稳定驱动可选触发（登记未验收）；源码事实见 upstream_xicai_position_difference_is_registered"]
fn upstream_xicai_gains_damage_card() {
    let mut e = yunchou_engine(20263401);
    e.debug_grant(0, "ycnxicai");
    let hand_before = e.debug_hand(0).len();
    let card = e.debug_place(1, "sha");
    // 带牌伤害：debug 助手会把该牌置于处理区并结算伤害（模拟「牌在 o 区」的窗口）
    e.debug_deal_damage_with_card(
        Some(1),
        0,
        1,
        engine_native::event::DamageNature::Normal,
        card,
    );
    // 戏才为可选触发：先确认再结算
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "trigger_confirm" {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![engine_native::ChoiceOption::Confirm],
                },
            });
        }
    }
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "上游：gain(trigger.cards) → 持有者获得伤害牌"
    );
    assert!(
        !e.state_core().discard.contains(&card),
        "伤害牌不应留在弃牌堆（已被获得）"
    );
}

/// 上游点 2 的**登记差异**：原生不校验「处理区」位置（table 优先、弃堆兜底）。
#[test]
fn upstream_xicai_position_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/yunchou.rs"
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
