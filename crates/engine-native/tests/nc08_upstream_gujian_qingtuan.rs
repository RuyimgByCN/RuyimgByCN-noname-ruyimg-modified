//! NC08 第⑦层（上游行为对照）样本 104：**来源 U05（古剑）** 食物牌
//! `qingtuan`（青团）——**食物族「非归属效果」抽样**。
//!
//! 上游源码（`.../gujian/card.js` 的 `qingtuan` + `.../gujian/skill.js:225` 的 `qingtuan`）：
//!
//! ```js
//! // card.js
//! qingtuan: {
//!     type: "food", enable: true,
//!     filterTarget(card, player, target) { return !target.hasSkill("qingtuan"); },
//!     content() {
//!         target.$gain2(cards);  target.storage.qingtuan = card;
//!         target.storage.qingtuan_markcount = 2;  target.addSkill("qingtuan");
//!     },
//! },
//! // skill.js:225（子技）
//! qingtuan: {
//!     … 提示「你在回合内使用首张杀时摸一张牌（剩余 N 回合）」…
//!     player.storage.qingtuan_markcount--;      // 每回合递减
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **获得方式**：食物牌把子技授予目标（**本体归属**缺失见第 37 项）；
//! 2. **子技效果**：**回合内使用首张【杀】时摸 1 张**（`once_per_turn` 门控）；
//! 3. **计数**：`markcount` 每回合递减（存活回合数）。
//!
//! 原生实现引用：`content/host/step/gujian.rs:247`（`GujianFoodQingtuan`：注释
//! 「**回合内使用首张杀（once_per_turn 门控）：摸 1**」+ `food_tick`）。
//!
//! **本样本的定位**：验证子技的**触发面**（首张杀摸 1）与**门控**（同回合第二张**不**再摸）——
//! 这是食物族「**非归属效果**」的抽样（归属面已由样本 94 普查）。
//!
//! 未覆盖（登记）：**`once_per_turn` 门控**（同回合第二张【杀】被 `ShaLimit` 拒绝，
//! 无法借此验证 —— 需额外出杀载体或跨回合驱动）；`markcount` 的**逐回合递减/到期**；
//! `filterTarget` 门槛。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2+3（**行为面**）：首张【杀】**摸 1**；同回合第二张**不再摸**（`once_per_turn` 门控）。
#[test]
fn upstream_qingtuan_first_sha_draws_once_per_turn() {
    let mut e = gujian_engine(20273401);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：青团 + 两张【杀】
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("qingtuan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("青团在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let shas: Vec<engine_native::CardId> = (0..2)
        .map(|_| {
            let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
            e.debug_place(0, name)
        })
        .collect();
    // ① 吃青团（对自己）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: food, targets: vec![0] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "青团应可发动（对自己）：{out:?}"
    );
    drain(&mut e);
    let has_skill = e.debug_has_skill(0, "gj_food_qingtuan");
    let h_after_food = e.debug_hand(0).len();
    // ② 第一张【杀】→ 应摸 1
    let out1 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shas[0], targets: vec![1] },
    });
    assert!(
        matches!(
            out1,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "第一张杀应可发动：{out1:?}"
    );
    drain(&mut e);
    let h_after_sha1 = e.debug_hand(0).len();
    // ③ 第二张【杀】→ 门控：**不再摸**
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shas[1], targets: vec![1] },
    });
    drain(&mut e);
    let h_after_sha2 = e.debug_hand(0).len();
    eprintln!(
        "DBG 青团：子技已授予={has_skill}；吃完后手牌={h_after_food}；首张杀后={h_after_sha1}；\
         第二张杀后={h_after_sha2}（第二张结果 {out2:?}）"
    );
    assert!(has_skill, "青团：目标应获得子技 gj_food_qingtuan");
    assert_eq!(
        h_after_sha1,
        h_after_food,
        "青团：**首张【杀】应摸 1**（打出杀 −1、摸 1 ⇒ 手牌不变；{h_after_food} → {h_after_sha1}）"
    );
    // **门控未验收（登记）**：同回合第二张【杀】被 `ShaLimit`（杀次数上限）拒绝，
    // 无法借此验证 `once_per_turn` 门控 —— 需**允许额外出杀**的载体（如本包「元宝肉」）
    // 或**跨回合**驱动（次回合首杀应再次摸 1）。此处**只**断言首杀摸 1 已生效。
    eprintln!(
        "NOTE 青团门控未验收：第二张杀被 {out2:?} 拒绝（ShaLimit）——需额外出杀载体或跨回合驱动"
    );
    let _ = (h_after_sha2, h_after_sha1);
}
