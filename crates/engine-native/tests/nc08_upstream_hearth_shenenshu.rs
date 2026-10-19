//! NC08 第⑦层（上游行为对照）样本 122：**来源 U07（hearth）** 锦囊
//! `shenenshu`（神恩术）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! shenenshu: {
//!     type: "trick", enable: true,
//!     selectTarget: -1, filterTarget(card, player, target) { return target === player; },
//!     modTarget: true,
//!     content() {
//!         "step 0"; var cards = target.getCards("h");
//!                   if (cards.length) target.lose(cards)._triggered = null;   // **失去全部手牌**
//!                   event.num = 1 + cards.length;                             // **数量 = 1 + 原手牌数**
//!         "step 1"; var list = get.typeCard("basic"); list.remove("du");      // 基本牌池（排除毒）
//!                   for (i < event.num) cards.push(game.createCard(list.randomGet()));   // **随机生成**
//!                   target.directgain(cards);                                 // **获得之**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**；
//! 2. **效果**：**失去全部手牌** → 获得 **`1 + 原手牌数`** 张**基本牌**（上游为**随机**抽取基本牌名）。
//!
//! 原生实现引用：宿主行为（hearth 锦囊机）。
//!
//! **本样本的定位**：验证「**数量 = 1 + 原手牌数**」与「**全为基本牌**」两点；
//! 并以 `询问=[]` **行为面确认**「随机 → 确定性」这一族内口径。
//!
//! **登记（源码自述「暂缓」）**：`content/standard.rs:52-53` 记「v40：…灵魂之火/照明弹/急耗卡
//! （CardKind 三种 + 宿主行为；**`shenenshu` 等复杂牌暂缓**）」⇒ 本卡**效果未实现**（已登记）。
//!
//! 未覆盖（登记）：生成牌的**具体牌名分布**（上游随机）；「排除毒」的边界。

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

/// 上游点 2（**行为面**）：**失去全部手牌 → 获得 (1 + 原手牌数) 张基本牌**。
///
/// **本轮未验收（登记，源码自述「暂缓」）**：实测——发动 `Accepted`，但**原手牌未失去**、
/// **未获得新牌**（手牌仍为原 2 张、其中基本牌 1 张）⇒ **效果为空（未实现）**。
/// **源码印证（决定性）**：`content/standard.rs:52-53` 的版本注释原文——
/// 「v40：G1 杀海拾遗·炉石子包切片——灵魂之火/照明弹/急耗卡（CardKind 三种 + 宿主行为；
/// **`shenenshu` 等复杂牌暂缓**）」⇒ **原生自己登记为「暂缓」**（非未记录的缺口）。
/// 用例以 `#[ignore]` 作为实现后的验收条件。
#[test]
#[ignore = "神恩术：效果未实现（原生注释自述『shenenshu 等复杂牌暂缓』）——见第 44 项"]
fn upstream_shenenshu_replaces_hand_with_basics() {
    let mut e = hearth_engine(20275201);
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
            .find(|(_, c)| c.name.contains("shenenshu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("神恩术在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 另给两张**非基本**手牌（便于验证「全部失去」）
    let mut old = Vec::new();
    for _ in 0..2 {
        // 取一张**非基本**牌（按 kind 判定，避免依赖具体牌名）
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(i, _)| {
                !matches!(
                    e.catalog().card_kind(*i as engine_native::CardId),
                    engine_native::CardKind::Sha
                        | engine_native::CardKind::Shan
                        | engine_native::CardKind::Tao
                        | engine_native::CardKind::Jiu
                )
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("非基本牌");
        nc08_common::put_hand(&mut e, 0, id);
        old.push(id);
    }
    let n_old = old.len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 神恩术：发动={out:?}（原非基本手牌 {n_old} 张）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神恩术应可发动（仅自己）：{out:?}"
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
    let old_gone = !old.iter().any(|c| hand.contains(c));
    let basics = hand
        .iter()
        .filter(|c| {
            matches!(
                e.catalog().card_kind(**c),
                engine_native::CardKind::Sha
                    | engine_native::CardKind::Shan
                    | engine_native::CardKind::Tao
                    | engine_native::CardKind::Jiu
            )
        })
        .count();
    eprintln!(
        "DBG 神恩术：询问={prompts:?}；手牌 {} 张（其中基本牌 {basics} 张）；原非基本手牌已全部失去={old_gone}",
        hand.len()
    );
    assert!(old_gone, "神恩术：应**失去全部原手牌**（上游 `target.lose(cards)`）");
    assert_eq!(
        hand.len(),
        1 + n_old,
        "神恩术：应获得 **1 + 原手牌数 = {}** 张（实测 {}）",
        1 + n_old,
        hand.len()
    );
    assert_eq!(
        basics,
        hand.len(),
        "神恩术：获得的应**全为基本牌**（上游 `get.typeCard(\"basic\")`，排除毒）"
    );
    eprintln!("NOTE 神恩术：询问={prompts:?}（上游为 `list.randomGet()` 随机生成 ⇒ 原生确定性属族内口径）");
}
