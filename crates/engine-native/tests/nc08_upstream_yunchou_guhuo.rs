//! NC08 第⑦层（上游行为对照）样本 78：**来源 U05（杀海拾遗·运筹）** 武将技
//! `diyguhuo`（蛊惑·变体）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`diyguhuo`）：
//!
//! ```js
//! diyguhuo: {
//!     trigger: { player: "phaseBegin" },        // **回合开始**
//!     forced: true,                              // 锁定
//!     filter(event, player) { return player.countCards("hej") > 0; },   // 手牌/装备/判定区有牌
//!     content() {
//!         "step 0"; player.draw(2);                                  // **摸 2**
//!         "step 1";
//!         var next = player.discardPlayerCard(player, "hej", 2, true);   // **弃自己 hej 区 2 张**
//!         next.filterButton = b => lib.filter.cardDiscardable(b.link, player);   // 不可弃者除外
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**回合开始**（锁定；`hej` 有牌才触发）；
//! 2. **效果**：**摸 2** → 再**弃自己手牌/装备/判定区的 2 张**（`discardPlayerCard(..., true)`）；
//! 3. **可选性**：由**玩家选择**要弃的 2 张（`filterButton` 排除不可弃牌，如延迟锦囊）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs`（`YunchouDiyguhuo`：`draw_cards(owner, 2)` →
//! 按 **判定区 → 装备区 → 手牌首张** 的**固定顺序**自动弃 2 张，装备走 `revoke`）。
//!
//! **登记差异（本文件固定）**：**弃牌的选择权** —— 上游为**玩家自选 2 张**（并排除不可弃牌）；
//! 原生为**固定顺序自动弃**（判定区→装备→手牌）→ 与「取牌选择权」族**同族**
//! （`liuli`/`tuxi`/`qicai` 等已有该族条目）。
//!
//! 未覆盖（登记）：`filterButton`（延迟锦囊不可弃）的对应行为；`ai.effect` 策略面。

mod nc08_common;
use engine_native::{Engine, GameAction, GameCommand, Phase};
use nc08_common::{drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：回合开始 → **摸 2** 且**弃 2**（原生按判定区→装备→手牌自动弃）。
#[test]
fn upstream_guhuo_draws_two_and_discards_two() {
    let mut e = yunchou_engine(20271001);
    e.debug_grant(0, "ycndiyguhuo");
    // 引擎初态即 0 号判定阶段（回合开始已过）→ 须等**下一轮**自己的回合开始
    assert!(
        nc08_common::advance_to_phase(&mut e, 0, Phase::Draw),
        "应到达摸牌阶段"
    );
    // 布置：判定区 1 张 + 装备区 1 件 + 手牌若干
    // 判定区放**任意一张**牌即可（本用例验证的是「弃牌顺序」，与牌的类别无关）
    let judge_card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(i, c)| {
                // 避开与装备/手牌同名的牌，确保是**独立的一张**
                *i as engine_native::CardId != 0 && !c.name.contains("armor") && c.name != "shan"
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("任意牌");
        nc08_common::put_judge(&mut e, 0, id);
        id
    };
    let equip = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("armor"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("防具");
        nc08_common::put_equips(&mut e, 0, id);
        id
    };
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let _ = e.debug_place(0, name);
    }
    let judge_before = e.debug_judge(0).len();
    let equips_before = e.debug_equips(0).len();
    let hand_before = e.debug_hand(0).len();
    // 驱动一整轮，等 0 号**下一次**回合开始
    for _ in 0..400 {
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
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    let judge_after = e.debug_judge(0).len();
    let equips_after = e.debug_equips(0).len();
    let hand_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 蛊惑·变体：判定 {judge_before} → {judge_after}（卡={judge_card}）；\
         装备 {equips_before} → {equips_after}（卡={equip}）；手牌 {hand_before} → {hand_after}"
    );
    // 原生口径：摸 2 后按「判定区 → 装备区 → 手牌」自动弃 2 张 → 判定区与装备区各 −1、手牌 +2
    assert_eq!(judge_after, judge_before - 1, "蛊惑·变体：应先弃**判定区**那张");
    assert_eq!(equips_after, equips_before - 1, "蛊惑·变体：再弃**装备区**那张");
    // 手牌数**不能**作为判据：驱动一整轮会经过该角色自己的**弃牌阶段**（`discard_to_limit`），
    // 手牌变化被其污染（实测 8 → 4）。故只用**判定区/装备区**两个干净信号。
    let _ = (hand_before, hand_after);
}

/// **登记差异**（源码事实固定）：弃牌的选择权（玩家自选 ↔ 固定顺序自动）。
#[test]
fn upstream_guhuo_choice_difference_is_registered() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouDiyguhuo =>").expect("蛊惑变体分支");
    let seg: String = step[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("judge.first()") && seg.contains("equips.first()"),
        "原生依据：固定顺序（判定区 → 装备区 → 手牌）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tdiyguhuo: {").expect("上游蛊惑变体");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("discardPlayerCard(player, \"hej\", 2, true)")
            && seg.contains("cardDiscardable"),
        "上游依据：玩家自选 hej 区 2 张 + 排除不可弃：{seg}"
    );
}
