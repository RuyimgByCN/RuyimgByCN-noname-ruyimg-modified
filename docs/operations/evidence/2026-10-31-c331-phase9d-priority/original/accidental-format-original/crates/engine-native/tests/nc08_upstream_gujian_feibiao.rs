//! NC08 第⑦层（上游行为对照）样本 91：**来源 U05（古剑）** 机关牌
//! `feibiao`（飞镖）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`feibiao`）：
//!
//! ```js
//! feibiao: {
//!     type: "jiguan", enable: true, wuxieable: true,
//!     outrange: { globalFrom: 2 },                  // **无视距离**（攻击范围 +2）
//!     filterTarget(card, player, target) { return target !== player; },
//!     content() {
//!         "step 0";
//!         if (!target.countCards("h", { color: "black" })) { target.loseHp(); event.finish(); }
//!         else target.chooseToDiscard({ color: "black" }, "弃置一张黑色手牌或受失去1点体力")…;
//!         "step 1";
//!         if (!result.bool) target.loseHp();       // **未弃 → 失去 1 点体力（体力流失，非伤害）**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**（**无视距离**：`outrange.globalFrom = 2`）；
//! 2. **分支**：目标**有黑色手牌** → **弃一张黑色手牌** 或 **失去 1 点体力**；
//!    目标**无黑色手牌** → **直接失去 1 点体力**；
//! 3. **语义**：**`loseHp`（体力流失）而非 `damage`** —— 不吃伤害修饰/护甲。
//!
//! 原生实现引用：`content/host/step/gujian.rs:90`（`GujianFeibiaoCard` 阶段机）。
//!
//! **本样本的定位**：验证两个分支的**净效果**（弃牌免流失 / 无黑牌则流失）。
//!
//! 未覆盖（登记）：**`loseHp` 与 `damage` 的语义差异**（需护甲/伤害修饰载体才能区分）；
//! `outrange`（无视距离）的射程面；`wuxieable` 无懈面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn feibiao_card(e: &mut Engine) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("feibiao"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("飞镖在目录");
    nc08_common::put_hand(e, 0, id);
    id
}

/// 跑一次飞镖：`target_has_black` 决定目标是否持黑色手牌；返回 `(hp 变化, 询问序列)`。
fn run(seed: u64, target_has_black: bool, discard: bool) -> (i32, Vec<String>) {
    let mut e = gujian_engine(seed);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black = if target_has_black {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                )
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黑色牌");
        nc08_common::put_hand(&mut e, 1, id);
        Some(id)
    } else {
        None
    };
    let card = feibiao_card(&mut e);
    let hp_before = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "飞镖应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 目标是否弃牌
        if discard {
            if let Some(b) = black {
                if c.spec.options.contains(&ChoiceOption::Card(b)) {
                    let _ = e.submit(GameCommand {
                        actor: c.observer,
                        expected_revision: e.revision(),
                        dedup_id: next_dedup(),
                        action: GameAction::Answer {
                            choice_id: c.id,
                            selection: vec![ChoiceOption::Card(b)],
                        },
                    });
                    continue;
                }
            }
        }
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
    drain(&mut e);
    (e.debug_hp(1) - hp_before, prompts)
}

/// 上游点 2（**行为面 · 两分支**）：有黑牌弃之免流失；无黑牌（或未弃）则**失去 1 点体力**。
#[test]
fn upstream_feibiao_two_branches() {
    let (d_has, p_has) = run(20272101, true, true); // 有黑牌且弃之
    let (d_none, p_none) = run(20272102, false, false); // 无黑牌
    let (d_decline, _) = run(20272103, true, false); // 有黑牌但不弃
    eprintln!(
        "DBG 飞镖：有黑牌且弃 → hp{d_has:+}（询问{p_has:?}）；无黑牌 → hp{d_none:+}（询问{p_none:?}）；有黑牌不弃 → hp{d_decline:+}"
    );
    assert_eq!(d_has, 0, "飞镖：目标弃黑色手牌 → **不失去体力**（hp 不变）");
    assert_eq!(d_none, -1, "飞镖：目标无黑色手牌 → **直接失去 1 点体力**");
    assert_eq!(d_decline, -1, "飞镖：有黑牌但**未弃** → **失去 1 点体力**");
}
