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
/// **上游 `step 1` 的 `chooseToDiscard("he", true)` 未在行为面复现**。
///
/// **重要更正**：原判「实现缺失」**已降级**——宿主实现**确实声明了两步**：
/// ```rust
/// // content/host/step/swd_equipment.rs:298-320
/// HostBehavior::SwdQinglongzhiguiHeld => match self.stage {
///     HostStage::Act => {
///         ctx.draw_cards(self.owner, 2);                       // 摸 2
///         let options = hand + equips → ChoiceOption::Card;
///         if options.is_empty() { return Flow::Done; }
///         self.stage = HostStage::SelectCard;
///         Flow::WaitChoice { prompt: "qinglongzhigui_discard", min: 1, … }
/// ```
/// ⇒ **实现存在**；问题在**「该 `WaitChoice` 为何不在 `open_choice()` 中可见」**。
///
/// 已排除：①持有条件 ②询问形态（覆盖 SelectCards/Confirm/Pass）③阶段未开始（摸 2 已证）
/// ④提示被 `advance_to_phase` 的 `drain` 吞掉（已改**手动推进、不 drain**，仍不可见）。
/// **剩余候选**：①`WaitChoice` 在 `NextPhase` 的**返回值**里而非 `open_choice()`；
/// ②机器在 `Act` 后**未进入 `SelectCard`**（如 `options` 为空——但手牌 5 张，不成立）；
/// ③`Flow::WaitChoice` 的登记路径与 `open_choice()` 读取路径不一致。
#[test]
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
    // **手动推进**（`advance_to_phase` 会 `drain`，把阶段开始时的弃牌询问一起吞掉 ⇒ 不可用）
    // ⚠ **第 55 项的更正经纬**：`Phase { begin: true }` 只在**回合起点的判定阶段**发射
    // （`kernel/events.rs:753/789`），建局时第 1 轮已发过 ⇒ 必须推进到**下一轮**（`round >= 2`）。
    for _ in 0..260 {
        if e.turn().round >= 2 {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        // **不吃询问**：仅在**无**待答询问时继续推进
        if e.open_choice().is_some() {
            break;
        }
    }
    assert_eq!(e.turn().current, 0, "应轮到 0 号");
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
