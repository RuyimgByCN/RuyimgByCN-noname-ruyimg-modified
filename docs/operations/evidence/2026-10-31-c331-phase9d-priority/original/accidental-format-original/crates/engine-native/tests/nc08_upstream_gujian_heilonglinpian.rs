//! NC08 第⑦层（上游行为对照）样本 97：**来源 U05（古剑）** 锦囊
//! `heilonglinpian`（黑龙鳞片）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`heilonglinpian`）：
//!
//! ```js
//! heilonglinpian: {
//!     type: "trick", enable: true, toself: true,
//!     filterTarget(card, player, target) { return target === player; },   // **仅自己**
//!     selectTarget: -1, modTarget: true,
//!     content() {
//!         target.changeHujia();                                  // **+1 护甲**
//!         target.addTempSkill("heilonglinpian", { player: "phaseBegin" });   // 临时技至**自己回合开始**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself`）；
//! 2. **效果**：**+1 护甲**；并获得**临时技**（持续到自己**回合开始**时移除）。
//!
//! 原生实现引用：宿主行为（古剑锦囊机）—— 本样本以**行为面**观测「护甲 +1」这一即时效果。
//!
//! 未覆盖（登记）：临时技的**到期移除**（需跨回合驱动）；`modTarget` 的额外影响面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**+1 护甲**。
#[test]
fn upstream_heilonglinpian_grants_hujia() {
    let mut e = gujian_engine(20272701);
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
            .find(|(_, c)| c.name.contains("heilonglinpian"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黑龙鳞片在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hujia_before = e.state_core().player(0).hujia;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 黑龙鳞片：发动={out:?}（使用者护甲前值={hujia_before}）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "黑龙鳞片应可发动（仅自己）：{out:?}"
    );
    drain(&mut e);
    let p0 = e.state_core().player(0);
    // 临时技在目录中的名字待核对（上游键为 `heilonglinpian`；原生可能用 `gj_...` 前缀）
    // 逐个候选目录名查询临时技（避免「名称不匹配」被误判为缺失）
    let candidates = [
        "heilonglinpian",
        "gj_heilonglinpian",
        "gj:heilonglinpian",
        "host:heilonglinpian",
        "heilonglinpian_skill",
    ];
    let hits: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|n| e.debug_has_skill(0, n))
        .collect();
    eprintln!(
        "DBG 黑龙鳞片：使用者护甲 {hujia_before} → {}；临时技候选命中={hits:?}",
        p0.hujia
    );
    assert_eq!(
        p0.hujia,
        hujia_before + 1,
        "黑龙鳞片：使用者应 **+1 护甲**（上游 `target.changeHujia()`；{hujia_before} → {}）",
        p0.hujia
    );
}
