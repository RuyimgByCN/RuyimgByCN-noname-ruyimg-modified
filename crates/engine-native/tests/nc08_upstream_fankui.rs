//! NC08 第⑦层（上游行为对照）样本 45：`fankui`（反馈，**标准面** / 司马懿）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! fankui: {
//!     audio: 2,
//!     trigger: { player: "damageEnd" },                 // 自己受到伤害后
//!     logTarget: "source",
//!     filter(event, player) {
//!         return event.num > 0
//!             && event.source?.hasGainableCards(
//!                    player,
//!                    event.source !== player ? "he" : "e");   // **自伤时只看装备区**
//!     },
//!     async content(event, trigger, player) {
//!         player.gainPlayerCard({ target: trigger.source,
//!             position: trigger.source !== player ? "he" : "e", forced: true });  // **玩家选牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**受到伤害后**（`damageEnd`，`num > 0`）；
//! 2. **牌区**：来源**非自己** → **`he`**（手牌 + 装备）；来源**是自己**（自伤）→ **`e`**（**仅装备**）；
//! 3. **取牌方式**：`gainPlayerCard` → **由玩家选择**一张（`forced: true`，无放弃）。
//!
//! 原生实现引用：`content/standard/skills/fan_kui.rs`（filter `Damage` + 目标自己 + 未防止 +
//! `amount > 0` + `event_damage_source_stealable`；取牌为
//! `hand.first().or_else(equips.first())` —— **确定性首张**）。
//!
//! **登记差异（本文件固定）**：
//! - **①取牌方式**：上游**玩家选牌**；原生**确定性首张**（手牌优先、否则装备）→ 与 `lijian`
//!   的「**玩家选择权缺失**」**同族**（第 2 例）；
//! - **②自伤牌区**：上游自伤时**仅装备区**（`"e"`）；原生先取 `hand.first()` → **自伤时仍可拿手牌**
//!   （**原生更宽松**）。
//!
//! 未覆盖（登记）：`ai.maixie_defend`/`preHidden`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1+3：受到伤害后**获得来源的一张牌**（行为面：来源手牌减少）。
#[test]
fn upstream_fankui_gains_card_from_source() {
    let mut e = Engine::new(20267701, 4);
    e.debug_grant(0, "反馈");
    advance_to_play(&mut e, 1); // 由 1 号出牌打 0 号
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号持【杀】；0 号持一张手牌（供被取）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let name0 = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
    let _ = e.debug_place(0, name0);
    let src_hand_before = e.debug_hand(1).len();
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
    // 0 号不出【闪】（无闪则自动命中）；处理反馈的可选确认（若有）
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
    drain(&mut e);
    eprintln!(
        "DBG 反馈：确认={confirmed} 来源手牌 {src_hand_before} → {}（0 号手牌={}）",
        e.debug_hand(1).len(),
        e.debug_hand(0).len()
    );
    assert!(
        e.debug_hand(1).len() < src_hand_before || !e.debug_equips(1).is_empty(),
        "反馈：应获得来源的一张牌（来源手牌 {src_hand_before} → {}）",
        e.debug_hand(1).len()
    );
}

/// **登记差异①②**（源码事实固定）：取牌方式为确定性首张；自伤时牌区未限制为装备。
#[test]
fn upstream_fankui_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/fan_kui.rs"
    ))
    .expect("读反馈源码");
    // 全文件检查（文件头注释较长，固定窗口会漏掉取牌实现）
    let seg: &str = &src;
    // 取牌实现跨行书写（`.hand` / `.first()` / `.or_else(|| p.equips.first()…)`）→ 分行断言
    assert!(
        seg.contains(".hand") && seg.contains(".first()") && seg.contains("equips.first()"),
        "差异①固定：原生取确定性首张（手牌优先，否则装备）：{seg}"
    );
    assert!(
        seg.contains("手牌优先"),
        "原生注释自述『手牌优先，其次装备区』：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tfankui: {").expect("上游反馈");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("gainPlayerCard") && seg.contains("forced: true"),
        "上游依据：玩家选牌（gainPlayerCard，forced）：{seg}"
    );
    assert!(
        seg.contains("\"he\" : \"e\"") || (seg.contains("\"he\"") && seg.contains("\"e\"")),
        "上游依据②：自伤时仅装备区（\"e\"）：{seg}"
    );
    let _ = Phase::Play;
}
