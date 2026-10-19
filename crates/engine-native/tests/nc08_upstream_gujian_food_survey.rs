//! NC08 第⑦层（上游行为对照）样本 94：**来源 U05（古剑）食物牌「本体归属」普查**
//! （12 张食物牌逐卡抽样）。
//!
//! 上游食物牌的统一模式（`gujian/card.js`，各卡 `content` 首段）：
//!
//! ```js
//! content() {
//!     if (target === targets[0] && cards.length) target.$gain2(cards);   // **本体归属：牌进目标手牌**
//!     target.storage.<key> = card;
//!     target.storage.<key>_markcount = N;
//!     target.addSkill("<key>");
//! },
//! ```
//!
//! 已登记的共性（修复清单第 37 项，样本 89/93 + 既有春饼样本）：
//! **`$gain2`（本体归属）缺失**（牌入弃堆而非目标手牌），而 `storage`/`markcount`（原生以 `rounds` 建模）
//! 与 `addSkill` **均正常**。
//!
//! **本样本的作用**：对**全部 12 张食物牌**逐卡执行「目标是否获得此牌」的**同一检查点**，
//! 给出**完整分布**（是全缺、还是有个案例外）。
//!
//! **判读约定**：本样本**不预设**结论——逐卡打印「是否获得本体」，并把**实际观测**写入断言：
//! 若全部 12 张均未获得 ⇒ 共性完整；若有个案获得 ⇒ 该卡为**例外**（须在证据中单列）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游 `type: "food"` 的 12 张卡（自 `gujian/card.js` 提取）。
const FOOD_CARDS: &[&str] = &[
    "luyugeng",
    "chunbing",
    "gudonggeng",
    "liyutang",
    "mizhilianou",
    "xiajiao",
    "tanhuadong",
    "mapodoufu",
    "qingtuan",
    "yougeng",
    "molicha",
    "yuanbaorou",
];

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 单卡观测：返回 `None`（牌不在目录/不可发动）或 `Some(目标是否获得本体)`。
fn probe(name: &str, seed: u64) -> Option<bool> {
    let mut e = gujian_engine(seed);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        // 目录中的牌名**包含**包内键（如 `gj_liyutang`），故用 contains 匹配
        .find(|(_, c)| c.name.contains(name))
        .map(|(i, _)| i as engine_native::CardId)?;
    nc08_common::put_hand(&mut e, 0, card);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    if !matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        return None;
    }
    drain(&mut e);
    Some(e.debug_hand(1).contains(&card))
}

/// **普查**：逐卡检查「目标是否获得本体」，并打印完整分布。
#[test]
fn food_cards_body_ownership_survey() {
    let mut gained: Vec<&str> = Vec::new();
    let mut not_gained: Vec<&str> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();
    for (i, name) in FOOD_CARDS.iter().enumerate() {
        match probe(name, 20272400 + i as u64) {
            Some(true) => gained.push(name),
            Some(false) => not_gained.push(name),
            None => skipped.push(name),
        }
    }
    eprintln!(
        "DBG 食物牌本体归属普查（共 {}）：获得={:?}；未获得={:?}；未发动/不在目录={:?}",
        FOOD_CARDS.len(),
        gained,
        not_gained,
        skipped
    );
    // 结论：已登记的共性为「本体归属缺失」→ 断言**未获得**的集合应覆盖所有可发动者
    assert!(
        gained.is_empty(),
        "登记（第 37 项）：食物牌**本体归属缺失**——但以下卡**却获得了本体**（须单列为例外）：{gained:?}"
    );
    assert!(
        !not_gained.is_empty(),
        "普查应有可发动且未获得本体的样本（实测 {not_gained:?}）"
    );
}
