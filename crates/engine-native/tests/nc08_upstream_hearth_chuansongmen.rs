//! NC08 第⑦层（上游行为对照）样本 119：**来源 U07（hearth）** 锦囊
//! `chuansongmen`（传送门）——**hearth 包最后一张**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! chuansongmen: {
//!     type: "trick", enable: true, discard: false, toself: true,
//!     selectTarget: -1, filterTarget(card, player, target) { return target === player; },
//!     modTarget: true,
//!     content() {
//!         "step 0";
//!         var gained = get.cards()[0];                 // **牌堆顶一张**
//!         target.gain(gained, "gain2");                // **获得之**
//!         if (… lib.filter.filterCard(gained, target, …)) {
//!             var next = target.chooseToUse();         // **立即询问是否使用该牌**
//!             next.filterCard = card => card === gained;
//!             next.prompt = "是否使用" + get.translation(gained) + "？";
//!         } else { … }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself`；且 `discard: false` ⇒ **本体不入弃堆**）；
//! 2. **效果**：**获得牌堆顶一张**；随后**立即询问是否使用该牌**（「立即使用链」）。
//!
//! 原生实现引用：`kernel/intrinsic.rs:444/476`（`CardKind::Chuansongmen`）。
//! **登记差异**（基准 golden 表第 55 行）：「传送门（获得牌堆顶；**登记差异：立即使用链暂缓**）」。
//!
//! **本样本的定位**：验证「**获得牌堆顶**」；并**行为面确认**「立即使用链暂缓」（**无该询问**）。
//!
//! 未覆盖（登记）：`modTarget` 影响面；`discard: false`（本体去向）的完整核对。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
        .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**获得牌堆顶一张**；**立即使用链**未实现（登记差异）。
#[test]
fn upstream_chuansongmen_gains_deck_top() {
    let mut e = hearth_engine(20274901);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("chuansongmen"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("传送门在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let top_before = e.state_core().deck.last().copied();
    let h_before = e.debug_hand(0).len();
    let deck_before = e.state_core().deck.len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 传送门：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "传送门应可发动（仅自己）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
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
    drain(&mut e);
    let hand = e.debug_hand(0);
    let deck_after = e.state_core().deck.len();
    eprintln!(
        "DBG 传送门：询问={prompts:?}；手牌 {h_before} → {}；牌堆 {deck_before} → {deck_after}；\
         原牌堆顶（{top_before:?}）现在手中={}",
        hand.len(),
        top_before.map(|c| hand.contains(&c)).unwrap_or(false)
    );
    assert!(
        top_before.map(|c| hand.contains(&c)).unwrap_or(false),
        "传送门：应**获得牌堆顶一张**（上游 `get.cards()[0]` + `target.gain`）"
    );
    assert!(
        prompts.is_empty(),
        "传送门（**登记差异：立即使用链暂缓**）：不应出现「是否使用该牌」的询问（实测 {prompts:?}）"
    );
}
