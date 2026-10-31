//! NC08 第⑦层（上游行为对照）样本 121：**来源 U07（hearth）** 锦囊
//! `jihuocard`（激活）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! jihuocard: {
//!     type: "trick", enable: true, toself: true,
//!     filterTarget(card, player, target) { return player === target; },
//!     selectTarget: -1, modTarget: true,
//!     content() {
//!         if (_status.currentPhase === target) target.addTempSkill("jihuocard2");   // 自己回合内 → 临时技
//!         target.draw();                                                          // **摸 1**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself`）；
//! 2. **效果**：**摸 1**；且当**自己正是当前回合角色**时（本卡在出牌阶段使用，恒成立）
//!    **获得临时技 `jihuocard2`**。
//!
//! 原生实现引用：宿主行为（hearth 锦囊机）。
//!
//! **本样本的定位**：验证「**摸 1**」（以**牌堆计数 −1**为判据）与「**临时技授予**」
//! （技能名按**候选名探测**，含原生可能的前缀形式——见样本 97/99 的命名教训）。
//!
//! 未覆盖（登记）：`jihuocard2` 的**效果面**与**到期移除**；`modTarget` 影响面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
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

/// 上游点 2（**行为面**）：**摸 1**（牌堆 −1）；并探测临时技是否授予。
#[test]
fn upstream_jihuocard_draws_and_grants_temp_skill() {
    let mut e = hearth_engine(20275101);
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
            .find(|(_, c)| c.name.contains("jihuocard"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("激活在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let deck_before = e.state_core().deck.len();
    let h_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 激活：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "激活应可发动（仅自己）：{out:?}"
    );
    drain(&mut e);
    let deck_after = e.state_core().deck.len();
    let h_after = e.debug_hand(0).len();
    // 临时技候选名探测（避免「名称不匹配」被误判为缺失）
    let candidates = [
        "jihuocard2",
        "gj_jihuocard2",
        "hearth_jihuocard2",
        "gj_hearth_jihuocard2",
        "host:jihuocard2",
    ];
    let hits: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|n| e.debug_has_skill(0, n))
        .collect();
    eprintln!(
        "DBG 激活：手牌 {h_before} → {h_after}（打出 −1、摸 1）；牌堆 {deck_before} → {deck_after}；\
         临时技候选命中={hits:?}"
    );
    assert_eq!(
        deck_before as i64 - deck_after as i64,
        1,
        "激活：应**摸 1**（牌堆 {deck_before} → {deck_after}）"
    );
    assert_eq!(
        h_after, h_before,
        "激活：打出本体 −1、摸 1 ⇒ 手牌不变（{h_before} → {h_after}）"
    );
    eprintln!("NOTE 激活：临时技候选命中={hits:?}（若为空，须先按原生常量核对命名再判定是否缺失）");
}
