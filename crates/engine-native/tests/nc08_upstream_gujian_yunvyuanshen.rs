//! NC08 第⑦层（上游行为对照）样本 99：**来源 U05（古剑）** 基本牌
//! `yunvyuanshen`（玉女元神）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`yunvyuanshen`）：
//!
//! ```js
//! yunvyuanshen: {
//!     type: "basic", enable: true, logv: false,
//!     filterTarget(card, player, target) { return !target.hasSkill("yunvyuanshen_skill"); },
//!     async content(event, trigger, player) {
//!         target.storage.yunvyuanshen_skill = game.createCard("yunvyuanshen");   // 造一张牌存入
//!         target.addSkill("yunvyuanshen_skill");                                 // **授予子技**
//!         …（本体归属的 UI 处理）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**未持** `yunvyuanshen_skill` 的角色；
//! 2. **效果**：目标**获得子技 `yunvyuanshen_skill`**（并为其存入一张生成的牌）。
//!
//! 原生实现引用：宿主行为（古剑基本牌机）—— 本样本以**行为面**观测「子技是否授予」。
//!
//! **本样本的方法要点**：子技查询用**多候选名**（避免「名称不匹配」被误判为缺失——
//! 见样本 97/第 40 项的教训）。

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

/// 候选目录名（上游键 / 原生可能的前缀形式）。
const CANDIDATES: &[&str] = &[
    "gj_yunvyuanshen",
    "gj_yunvyuanshen_listener",
    "yunvyuanshen_skill",
    "gj_yunvyuanshen_skill",
    "gj:yunvyuanshen_skill",
    "host:yunvyuanshen_skill",
    "yunvyuanshen",
];

/// 上游点 2（**行为面**）：目标**获得子技**。
#[test]
fn upstream_yunvyuanshen_grants_skill() {
    let mut e = gujian_engine(20272901);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("yunvyuanshen"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("玉女元神在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 玉女元神：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玉女元神应可发动：{out:?}"
    );
    drain(&mut e);
    let hits: Vec<&str> = CANDIDATES
        .iter()
        .copied()
        .filter(|n| e.debug_has_skill(1, n))
        .collect();
    eprintln!("DBG 玉女元神：1 号子技候选命中={hits:?}");
    assert!(
        !hits.is_empty(),
        "玉女元神：目标应获得子技（上游 `target.addSkill(\"yunvyuanshen_skill\")`）——\
         候选名 {CANDIDATES:?} 均未命中"
    );
}
