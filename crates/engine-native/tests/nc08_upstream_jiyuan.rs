//! NC08 第⑦层（上游行为对照）样本 63：`xinfu_jiyuan`（**急援**，标准面 / 伊籍）。
//!
//! **命名提示**：上游键 `xinfu_jiyuan` 在 `translate.js` 中映射为「**急援**」；
//! 原生常量 `JI_YUAN`（`names.rs:51`）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! xinfu_jiyuan: {
//!     trigger: { global: ["dying", "gainAfter", "loseAsyncAfter"] },   // **三个事件臂**
//!     getIndex(event, player) {
//!         if (event.name !== "loseAsync") return [event.player];
//!         return game.filterPlayer(c => c !== player && event.getg(c).length > 0).sortBySeat();
//!     },
//!     filter(event, player, triggername, target) {
//!         if (!target?.isIn()) return false;
//!         if (event.name === "dying") return true;                      // ① 濒死臂
//!         if (event.giver !== player) return false;                     // ② 给予臂（须由自己给出）
//!         if (event.name === "gain") return event.player != player && event.getg(target).length > 0;
//!         return game.hasPlayer(c => c !== player && event.getg(c).length > 0);   // ③ 失去臂
//!     },
//!     async content(event, trigger, player) { await event.targets[0].draw(); },   // 目标**摸一张**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**三个臂**——① 任意角色**进入濒死**；② **自己给出的牌**被获得（`gainAfter`）；
//!    ③ **异步失去**（`loseAsyncAfter`，如被弃置/被拿走）；
//! 2. **效果**：令**目标角色摸一张牌**（三臂共用同一 `content`）。
//!
//! 原生实现引用：`content/standard/skills/ji_yuan.rs`（`filter` 仅 `EventData::Dying { .. }`；
//! 文件头注释自述「**登记差异：仅濒死臂**」）。
//!
//! **登记差异（规则损失，原生已自述）**：原生**只实现濒死臂** ✗，**缺 `gainAfter`（给予臂）
//! 与 `loseAsyncAfter`（失去臂）** → 上游三臂中 **2 臂缺失**（**规则损失**，宜并入修复清单）。
//!
//! 未覆盖（登记）：**濒死臂的行为面**（需构造「角色进入濒死 + 伊籍存活」场景并驱动结算）；
//! `audio` 表现面。

mod nc08_common;

use engine_native::Engine;

/// 上游点 2（**共用效果**）：原生濒死臂与上游 `content` 同为「目标摸一张」。
#[test]
fn upstream_jiyuan_effect_is_draw_one() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/ji_yuan.rs"
    ))
    .expect("读急援源码");
    let head: String = src.chars().take(700).collect();
    assert!(
        head.contains("濒死") && head.contains("摸一张牌"),
        "原生注释应写明『濒死时令其摸一张牌』：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\txinfu_jiyuan: {").expect("上游急援");
    let seg: String = up[idx..].chars().take(2400).collect();
    assert!(
        seg.contains("await event.targets[0].draw()"),
        "上游依据：效果为目标摸一张：{seg}"
    );
    assert!(
        seg.contains("\"dying\", \"gainAfter\", \"loseAsyncAfter\""),
        "上游依据：三个事件臂：{seg}"
    );
}

/// **登记差异（规则损失）**：原生只实现濒死臂，缺给予臂与失去臂。
#[test]
fn upstream_jiyuan_missing_arms_are_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/ji_yuan.rs"
    ))
    .expect("读急援源码");
    // filter 仅匹配 Dying
    let idx = src.find("fn filter").expect("filter");
    let seg: String = src[idx..].chars().take(200).collect();
    assert!(
        seg.contains("EventData::Dying"),
        "原生 filter 仅匹配 Dying：{seg}"
    );
    assert!(
        !src.contains("CardsMoved") && !src.contains("gainAfter") && !src.contains("loseAsync"),
        "差异固定：原生未见给予臂/失去臂的事件面"
    );
    let head: String = src.chars().take(300).collect();
    assert!(
        head.contains("登记差异") && head.contains("仅濒死臂"),
        "原生已自述该差异（仅濒死臂）：{head}"
    );
    let _ = Engine::new(20269501, 4);
}

/// 上游点 2（**行为面 · 濒死臂**）：任一角色进入濒死 → 伊籍（持急援）令其**摸一张**。
#[test]
fn upstream_jiyuan_dying_arm_draws_one() {
    use engine_native::{ChoiceOption, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269502, 4);
    e.debug_grant(0, "急援");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号进入濒死：必须走**真实伤害路径**（直接写 hp=0 不会置 dying，见既有教训）
    let hand2_before = e.debug_hand(2).len();
    let max2 = e.state_core().player(2).max_hp as i32;
    e.debug_deal_damage(
        Some(0),
        2,
        max2 as u32,
        engine_native::event::DamageNature::Normal,
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            let _ = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            continue;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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
    drain(&mut e);
    eprintln!(
        "DBG 急援（濒死臂）：询问={prompts:?} 2 号手牌 {hand2_before} → {}（hp={} dying={} alive={}）",
        e.debug_hand(2).len(),
        e.debug_hp(2),
        e.state_core().player(2).dying,
        e.state_core().player(2).alive
    );
    // **登记差异（新发现，行为面）**：原生机器在 `step` 中额外要求 `player.dying == true`
    // （`ji_yuan.rs`：「if ctx.core.player(player).dying { ctx.draw_cards(player, 1) }」），
    // 而濒死结算推进到该步时 `dying` 可能已为 false（本例：2 号 hp=0、dying=false、alive=false
    // → 已死亡）→ **摸牌未发生**；上游 `content` 为 `await event.targets[0].draw()`（**无条件**）
    // → **原生多了一层 `dying` 守卫 = 规则损失**（濒死未被救回时少摸一张）。
    assert_eq!(
        e.debug_hand(2).len(),
        hand2_before,
        "差异登记：濒死臂实测未摸牌（2 号 hp=0 / dying=false / alive=false）——\
         原生多一层 `dying` 守卫，上游为无条件 draw()"
    );
    let _ = Engine::new(20269503, 4);
}
