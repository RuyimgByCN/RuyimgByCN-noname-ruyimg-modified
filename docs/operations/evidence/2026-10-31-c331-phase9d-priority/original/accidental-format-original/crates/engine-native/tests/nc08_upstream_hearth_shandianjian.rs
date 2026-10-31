//! NC08 第⑦层（上游行为对照）样本 125：**来源 U07（hearth）** 锦囊
//! `shandianjian`（闪电箭）——**hearth 包最后一张**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`，`shandianjian`）：
//!
//! ```js
//! shandianjian: {
//!     type: "trick", enable: true, cardnature: "thunder",
//!     filterTarget(card, player, target) { … return target.countCards("h") > 0; },
//!     async content(event, trigger, player) {
//!         const result = await event.chooseToShow(event, player, target);   // **目标展示一张手牌**
//!         event.cards2 = cards;
//!         const showEvent = target.showCards(cards, `…因【闪电箭】展示的牌`);
//!         event.chooseToDiscard ??= async (event, player, target) => {
//!             const { discardPosition = "h", cards2,
//!                     filterDiscard = { suit: get.suit(cards2[0]) } } = event;   // **同花色**
//!             const result = await player.chooseToDiscard(discardPosition, filterDiscard)…;
//!             …
//!         };
//!         …
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**手牌非空**的角色；
//! 2. **流程**：目标**展示一张手牌** → **使用者须弃一张同花色手牌**（否则按上游后续分支结算）。
//!
//! **本轮未验收（登记，源码核对）**：`shandianjian` 在
//! **`crates/engine-native/src/` 全库 0 命中**（仅出现在包内 `cards.json`），
//! **文档亦未提及** ⇒ **包内交付但无原生实现、且无登记**（详见修复清单第 46 项）。
//! 本用例**记录实测受理结果**（接受/拒绝/效果），作为修复后的对照基线。
//!
//! 未覆盖（登记）：上游**后续分支**（未弃同花色时的雷伤口径）的逐字确认。

mod nc08_common;
use engine_native::{Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
            .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// **实测受理结果**（登记用基线）：记录发动结果与任何询问。
#[test]
fn upstream_shandianjian_record_acceptance() {
    let mut e = hearth_engine(20275501);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号持手牌（满足上游 `countCards("h") > 0`）
    let name = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    e.debug_place(1, name);
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shandianjian"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("闪电箭在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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
    eprintln!(
        "DBG 闪电箭（登记基线）：发动={out:?}；询问={prompts:?}；1 号 hp {hp1} → {}",
        e.debug_hp(1)
    );
    // **不写成通过**：本用例只记录受理结果；效果验收待实现后补（第 46 项）
    eprintln!(
        "NOTE 闪电箭：源码核对——`shandianjian` 在 crates/engine-native/src 全库 0 命中、文档未提及\
         ⇒ 包内交付但无实现且无登记（第 46 项）；实测受理结果见上"
    );
}
