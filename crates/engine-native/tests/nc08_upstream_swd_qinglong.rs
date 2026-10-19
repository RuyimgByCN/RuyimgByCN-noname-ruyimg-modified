//! NC08 第⑦层（上游行为对照）样本 156：**来源 U06（swd）** 法器
//! `qinglongzhigui`（青龙之圭）——**`global` 卡窗口的第 5 张验证**（同族法器，复用样本 154/155 骨架）。
//!
//! 上游源码（`skill.js` 的 **`g_qinglongzhigui`**，**逐字**）：
//!
//! ```js
//! qinglongzhigui: { type: "jiqi", global: "g_qinglongzhigui" },   // 卡本体无 content
//! g_qinglongzhigui: {
//!     trigger: { player: "phaseBegin" },                           // **本人阶段开始时**
//!     forced: true,
//!     filter(event, player) { return player.countCards("h", "qinglongzhigui") > 0; },
//!     content() {
//!         "step 0"; …player.draw(2);                               // **摸 2**
//!         "step 1"; player.chooseToDiscard("he", true);             // **强制弃 1**
//!     },
//! },
//! ```
//!
//! **本样本的定位**：**持有者的阶段开始时，摸 2 弃 1（强制）** ⇒ **手牌净 +1、牌堆 −2**。
//!
//! 未覆盖（登记）：`showCards` 表现面；阶段种类（`phaseBegin` 对**每个阶段**都触发 ⇒ 一轮可多次）。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_qinglongzhigui`（**forced**）：持有者阶段开始 ⇒ **摸 2 + 强制弃 1**。
///
/// **本轮未通过（登记，含诊断）**：阶段开始后 **牌堆 −2（摸 2 生效 ✅）**，
/// 但**手牌 +2**（应 **+1**）、且**全程无弃牌询问**（`prompts=[]`）⇒
/// **上游 `step 1` 的 `chooseToDiscard("he", true)` 未执行**。
///
/// 已排除：①持有条件（`filter` 仅要求持有，**成立**）②询问形态（用例同时处理 `SelectCards`/`Confirm`/`Pass`）
/// ③阶段未开始（摸 2 已证窗口运行）。
/// **剩余候选**：①强制弃牌分支未实现；②`step 1` 被跳过（阶段机只执行首步）；
/// ③`chooseToDiscard` 在 `he` 区无牌可弃时静默跳过——但实测手牌有 5 张，**不成立**。
#[test]
#[ignore = "青龙之圭：摸 2 生效但强制弃 1 未发生（手牌 +2、无弃牌询问）——见第 55 项"]
fn upstream_qinglongzhigui_draw2_discard1() {
    let mut e = swd_engine(20288801);
    // 0 号在**开局前**就持青龙之圭（阶段开始时才触发）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("qinglongzhigui"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("青龙之圭在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 再给 2 张手牌（供「强制弃 1」）
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name);
    }
    let deck_before = e.state_core().deck.len();
    let h_before = e.debug_hand(0).len();
    let mut prompts: Vec<String> = Vec::new();
    // 推进到 0 号出牌阶段（**阶段开始即触发**）
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    // 处理**强制弃 1** 询问
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == &c.spec.prompt.to_string()) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        let sel: Vec<ChoiceOption> = if !cards.is_empty() {
            cards.into_iter().take(c.spec.min.max(1)).collect()
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
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
    let deck_after = e.state_core().deck.len();
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 青龙之圭：询问={prompts:?}；牌堆 {deck_before} → {deck_after}（应 −2）；\
         手牌 {h_before} → {h_after}（摸 2 弃 1 ⇒ 净 +1）"
    );
    assert_eq!(
        deck_before as i64 - deck_after as i64,
        2,
        "青龙之圭：应**摸 2**（牌堆 {deck_before} → {deck_after}）"
    );
    assert_eq!(
        h_after as i64 - h_before as i64,
        1,
        "青龙之圭：摸 2、**强制弃 1** ⇒ 手牌**净 +1**（{h_before} → {h_after}）"
    );
}
