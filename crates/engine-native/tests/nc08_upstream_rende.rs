//! NC08 第⑦层（上游行为对照）样本 57：`rende`（仁德，**标准面** / 刘备）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! rende: {
//!     enable: "phaseUse",                        // 出牌阶段（**无次数限制**）
//!     filterCard: true, selectCard: [1, Infinity], allowChooseAll: true,
//!     discard: false, lose: false,               // **给出**（不是弃置）
//!     filterTarget(card, player, target) { return player != target; },
//!     async content(event, trigger, player) {
//!         const num = player.iterHistory("lose", evt => …)   // **本回合已给出的张数**
//!             .map(evt => evt.cards.length).reduce((a, b) => a + b, 0);
//!         await player.give(event.cards, event.target);
//!         if (num < 2 && num + event.cards.length > 1) {     // **跨过 2 张时回复 1**
//!             await player.recover();
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/次数**：出牌阶段，**可多次发动**（上游**无 `usable`**）；
//! 2. **费用**：把**任意张手牌给出**（`discard/lose: false` → **交给**目标）；
//! 3. **收益**：**本回合累计**给出**跨过 2 张**时（`num < 2 && num + n > 1`）**回复 1 点**——
//!    即 2 张一次给 或 1+1 分两次给 **都**触发，且**只在跨过时触发一次**。
//!
//! 原生实现引用：`content/standard/skills/rende.rs`（`recover(owner, 1, None)`）；
//! 注册面 `standard_skills.rs:193`（注释「出牌阶段**可多次发动**（上游 enable: phaseUse 无次数限制）；
//! **累计给牌数入回合状态（跨过 2 张回复 1 点）**」，`limits: SkillLimits::default()`）。
//!
//! **登记（文档不一致，非行为缺陷）**：`rende.rs` **文件头注释**写「出牌阶段**限一次**的主动技」，
//! 与注册面（可多次 + 累计）**不一致** → 注释滞后（行为以注册面为准，与上游一致）。
//!
//! 未覆盖（登记）：`check`（AI 给牌评分）与 `audio` 表现/策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

fn skill_id(e: &Engine, name: &str) -> u16 {
    e.catalog().skill_by_name(name).expect("技能在目录")
}

/// 跑一次「给出 n 张」：返回是否被接受。
fn give_cards(e: &mut Engine, n: usize) -> bool {
    let skill = skill_id(e, "仁德");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    if !matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
        eprintln!("DBG 仁德发动被拒：{out:?}");
        return false;
    }
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Player(1)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            continue;
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .take(n)
            .copied()
            .collect();
        if cards.len() == n {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: cards },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
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
    drain(e);
    true
}

/// 上游点 2+3（**一次给 2 张**）：给出两张 → **回复 1**。
#[test]
fn upstream_rende_two_cards_once_recovers() {
    let mut e = Engine::new(20268901, 4);
    e.debug_grant(0, "仁德");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let _ = e.debug_place(0, name);
    }
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1); // 受伤，便于观察回复
    let hp0 = e.debug_hp(0);
    let ok = give_cards(&mut e, 2);
    eprintln!(
        "DBG 仁德（一次 2 张）：发动={ok} 自己 hp {hp0} → {} 目标手牌={:?}",
        e.debug_hp(0),
        e.debug_hand(1)
    );
    assert!(ok, "仁德应可发动");
    assert_eq!(
        e.debug_hp(0),
        hp0 + 1,
        "仁德：一次给出 2 张 → 回复 1（{hp0} → {}）",
        e.debug_hp(0)
    );
    assert_eq!(e.debug_hand(1).len(), 2, "目标应收到 2 张牌");
    let _ = Phase::Play;
}

/// 上游点 3（**累计语义**）：**1 + 1 分两次给** → **跨过 2 张时**回复 1（且只回一次）。
#[test]
fn upstream_rende_cumulative_recovers_once() {
    let mut e = Engine::new(20268902, 4);
    e.debug_grant(0, "仁德");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..3 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let _ = e.debug_place(0, name);
    }
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 2); // 受伤 2 点，便于观察「只回一次」
    let hp0 = e.debug_hp(0);
    // 第 1 次：给 1 张（累计 1 < 2 → 不回复）
    let ok1 = give_cards(&mut e, 1);
    let hp_after_1 = e.debug_hp(0);
    // 第 2 次：再给 1 张（累计 2 → 跨过 → 回复 1）
    let ok2 = give_cards(&mut e, 1);
    let hp_after_2 = e.debug_hp(0);
    // 第 3 次：再给 1 张（累计 3 → 已跨过 → **不**再回复）
    let ok3 = give_cards(&mut e, 1);
    let hp_after_3 = e.debug_hp(0);
    eprintln!(
        "DBG 仁德（1+1+1）：发动={ok1}/{ok2}/{ok3} hp {hp0} → {hp_after_1} → {hp_after_2} → {hp_after_3}"
    );
    assert!(ok1 && ok2 && ok3, "仁德应可多次发动（上游无 usable）");
    assert_eq!(hp_after_1, hp0, "累计 1 张：不回复");
    assert_eq!(hp_after_2, hp0 + 1, "累计跨过 2 张：回复 1");
    assert_eq!(hp_after_3, hp0 + 1, "已跨过：不再回复（只触发一次）");
}

/// **登记（文档不一致）**：`rende.rs` 头注释「限一次」与注册面「可多次 + 累计」不一致。
#[test]
fn upstream_rende_doc_inconsistency_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/rende.rs"
    ))
    .expect("读仁德源码");
    let head: String = src.chars().take(300).collect();
    assert!(
        head.contains("出牌阶段限一次"),
        "登记：文件头注释仍写『限一次』（滞后）：{head}"
    );
    let def = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/standard_skills.rs"
    ))
    .expect("读标准技能表");
    let idx = def.find("name: REN_DE").expect("仁德注册");
    let seg: String = def[idx..].chars().take(320).collect();
    assert!(
        seg.contains("可多次发动") && seg.contains("跨过 2 张回复 1 点"),
        "注册面应写『可多次发动 + 累计跨过 2 张回复 1 点』：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\trende: {").expect("上游仁德");
    let seg: String = up[idx..].chars().take(2200).collect();
    assert!(
        seg.contains("num < 2 && num + event.cards.length > 1"),
        "上游依据：累计跨过 2 张时回复：{seg}"
    );
    assert!(
        !seg.contains("usable:"),
        "上游依据：**无 usable**（可多次发动）"
    );
}
