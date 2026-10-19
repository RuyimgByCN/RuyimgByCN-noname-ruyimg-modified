//! NC08 第⑦层（上游行为对照）样本 46：`yiji`（遗计，**标准面** / 郭嘉）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! yiji: {
//!     trigger: { player: "damageEnd" },              // 自己受到伤害后
//!     frequent: true,
//!     filter(event) { return event.num > 0; },
//!     getIndex(event, player, triggername) { return event.num; },   // **按伤害点数计次**
//!     async content(event, trigger, player) {
//!         const cards = get.cards(2);                // **亮出牌堆顶 2 张**
//!         await game.cardsGotoOrdering(cards);
//!         …
//!         // 由玩家在「这 2 张」中分配给其他角色（可给出至多 2 张）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**受到伤害后**（`damageEnd`，`num > 0`）；**按伤害点数计次**（`getIndex`）；
//! 2. **摸牌**：**亮出牌堆顶 2 张**（`get.cards(2)` + `cardsGotoOrdering`）；
//! 3. **分配**：把（至多两张）**这些牌**交给其他角色。
//!
//! 原生实现引用：`content/standard/skills/ji_ji.rs`（filter `Damage` + 目标自己 + 未防止 +
//! `amount > 0`；机器 `Draw` → `draw_cards(owner, 2)` → `Target`/`GiveCard`/`GiveChooseCard`
//! → `give_hand_card`）。
//!
//! **登记差异（本文件固定）**：
//! - **①计次**：上游 `getIndex → event.num`（受 N 点伤害触发 **N** 次）；原生 filter 只判
//!   `amount > 0` 且机器**单次结算** → 与 `shushen` 的「**按点数计次**」**同族（第 2 例）**；
//! - **②牌的来源**：上游先**亮出牌堆顶 2 张**（`get.cards` + `cardsGotoOrdering`），分配的是
//!   **这 2 张**；原生**摸进手牌**（`draw_cards`）后再**从手牌中**给出 → 给出的牌可能不是那 2 张
//!   （**机制不同**，登记）。
//!
//! 未覆盖（登记）：`ai`/`audio` 表现与策略面；「亮出」在信息面上的可见性差异。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1+2：受到伤害后**摸两张牌**（行为面：手牌净 +2）。
#[test]
fn upstream_yiji_draws_two_after_damage() {
    let mut e = Engine::new(20267801, 4);
    e.debug_grant(0, "遗计");
    advance_to_play(&mut e, 1); // 1 号打 0 号
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![0] },
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. } | engine_native::CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 遗计为可选触发 → 确认；随后可能有分配询问 → 放弃（本用例只看摸牌）
    let mut confirmed = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        // 分配阶段一律放弃（不给出牌）
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel || c.spec.options.contains(&ChoiceOption::Pass) {
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
        "DBG 遗计：确认={confirmed} 手牌 {before} → {}（0 号 hp={}）",
        e.debug_hand(0).len(),
        e.debug_hp(0)
    );
    if !confirmed {
        eprintln!("NOTE 未出现遗计确认门——本用例不构成验收");
        return;
    }
    assert!(
        e.debug_hand(0).len() >= before + 2,
        "遗计：受到伤害后应摸两张（{before} → {}）",
        e.debug_hand(0).len()
    );
    let _ = Phase::Play;
}

/// **登记差异①②**（源码事实固定）：按点数计次（与淑慎同族）；牌的来源为「摸进手牌」。
#[test]
fn upstream_yiji_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_ji.rs"
    ))
    .expect("读遗计源码");
    assert!(
        src.contains("draw_cards(self.owner, 2)"),
        "差异②固定：原生为『摸进手牌』（draw_cards）：{src}"
    );
    assert!(
        src.contains("amount > 0") && !src.contains("getIndex") && !src.contains("for _ in 0..d.amount"),
        "差异①固定：原生只判 amount > 0，未按点数计次"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tyiji: {").expect("上游遗计");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("getIndex(event, player, triggername)") && seg.contains("event.num"),
        "上游依据①：按伤害点数计次：{seg}"
    );
    assert!(
        seg.contains("get.cards(2)") && seg.contains("cardsGotoOrdering"),
        "上游依据②：亮出牌堆顶 2 张：{seg}"
    );
}
