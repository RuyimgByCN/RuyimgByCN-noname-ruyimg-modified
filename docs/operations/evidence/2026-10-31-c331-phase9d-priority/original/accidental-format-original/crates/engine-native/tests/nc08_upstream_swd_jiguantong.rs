//! NC08 第⑦层（上游行为对照）样本 68：**来源 U05（杀海拾遗·swd）** 机关牌
//! `jiguantong`（机关火筒）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`，`jiguantong`）：
//!
//! ```js
//! jiguantong: {
//!     type: "jiguan", cardnature: "fire", enable: true, wuxieable: true,
//!     selectTarget: -1, reverseOrder: true,                 // **AOE + 逆序**
//!     filterTarget(card, player, target) { return target !== player; },   // **其他角色**
//!     content() {
//!         "step 0";
//!         if (target.countCards("h")) {
//!             target.chooseToDiscard("机关火筒：弃置一张手牌或受到1点火焰伤害");   // **弃牌免伤**
//!         } else {
//!             target.damage("fire");                                        // **无手牌直接火伤**
//!             event.parent.preResult = true;
//!         }
//!         // …contentAfter：本次结算**无实伤**时使用者摸 1
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**（AOE，逆序结算）；
//! 2. **每目标**：**有手牌 → 弃一张手牌或受 1 点火焰伤害**（可取消＝受伤害）；**无手牌 → 直接火伤**；
//! 3. **contentAfter**：本次结算**未造成实际伤害**时，**使用者摸 1**。
//!
//! 原生实现引用：`content/host/step/swd_card_effects.rs:645`（`Act` 逐目标：无手牌→
//! `Damage(Fire,1)`；有手牌→`SWD_JIGUANTONG_DISCARD`（min1/max1/**allow_cancel**）；取消=火伤；
//! targets 空后按日志实伤判定 `!damaged → draw_cards(owner,1)` 并弃本体）。
//!
//! **登记差异（本文件固定）**：
//! - **①结算顺序**：上游 `reverseOrder: true`（**逆序**结算）↔ 原生按**座位序**——
//!   `engine/play_card.rs:1526` 注释自述「结算序=座位序**登记差异**」；
//! - 保留项：`wuxieable` 无懈面；`cardnature: fire` 的属性面。
//!
//! **门槛（已核对）**：该卡为 AOE，`play_card.rs:1526` 要求 `targets` **恰为**
//! 「自当前回合角色起的座位序中全部其他存活角色」→ 夹具须传**全部其他角色**。
//!
//! 未覆盖（登记）：多目标逆序结算顺序；无懈可击交互。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn jiguantong(e: &mut Engine) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguantong"))
        .expect("机关火筒在目录") as engine_native::CardId;
    nc08_common::put_hand(e, 0, id);
    id
}

/// 上游点 2（**分支 A**）：目标**有手牌** → 弃一张 → **免伤**；且**使用者摸 1**（contentAfter）。
#[test]
fn upstream_jiguantong_discard_avoids_damage() {
    let mut e = swd_engine(20270001);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = jiguantong(&mut e);
    // **AOE：全部其他角色**都须有手牌，才能「全员弃牌免伤」→ 本次无实伤 → 使用者摸 1
    let target_cards: Vec<engine_native::CardId> = (1..4)
        .map(|p| {
            let name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.name == "shan")
                .map(|c| c.name)
                .unwrap();
            e.debug_place(p, name)
        })
        .collect();
    let hp1 = e.debug_hp(1);
    let owner_hand = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1, 2, 3],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "机关火筒应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 每个被询问的目标各弃自己那张手牌
        if let Some(card) = target_cards
            .iter()
            .find(|x| c.spec.options.contains(&ChoiceOption::Card(**x)))
        {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(*card)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
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
    drain(&mut e);
    eprintln!(
        "DBG 机关火筒（弃牌分支）：询问={prompts:?} 目标 hp {hp1} → {} 使用者手牌 {owner_hand} → {}",
        e.debug_hp(1),
        e.debug_hand(0).len()
    );
    assert!(
        prompts.iter().any(|p| p.contains("jiguantong")),
        "机关火筒：有手牌目标应出现弃牌询问，实测 {prompts:?}"
    );
    assert_eq!(e.debug_hp(1), hp1, "机关火筒：弃牌后**免伤**（hp 不变）");
    assert_eq!(
        e.debug_hand(0).len(),
        owner_hand,
        "机关火筒：**全员弃牌免伤** → 本次无实伤 → 使用者摸 1（打出本体 −1、摸牌 +1 → 手牌数不变）"
    );
}

/// 上游点 2（**分支 B**）：目标**无手牌** → **直接受 1 点火焰伤害**；使用者**不摸牌**。
#[test]
fn upstream_jiguantong_empty_hand_takes_fire_damage() {
    let mut e = swd_engine(20270002);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = jiguantong(&mut e);
    let hp1 = e.debug_hp(1);
    let owner_hand = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1, 2, 3],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "机关火筒应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    drain(&mut e);
    eprintln!(
        "DBG 机关火筒（无手牌分支）：询问={prompts:?} 目标 hp {hp1} → {} 使用者手牌 {owner_hand} → {}",
        e.debug_hp(1),
        e.debug_hand(0).len()
    );
    assert!(
        !prompts.iter().any(|p| p.contains("jiguantong")),
        "机关火筒：无手牌目标**不应**出现弃牌询问（直接火伤），实测 {prompts:?}"
    );
    assert!(
        e.debug_hp(1) < hp1,
        "机关火筒：无手牌目标应受 1 点火伤（{hp1} → {}）",
        e.debug_hp(1)
    );
    assert_eq!(
        e.debug_hand(0).len(),
        owner_hand - 1,
        "机关火筒：本次**有实伤** → 使用者**不**摸牌（手牌应恰为打出本体后 −1）"
    );
}
