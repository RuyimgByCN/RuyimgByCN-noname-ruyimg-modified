//! NC08 第⑦层（上游行为对照）样本 96：**来源 U05（古剑）** 基本牌
//! `dinvxuanshuang`（帝女玄霜）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`dinvxuanshuang`）：
//!
//! ```js
//! dinvxuanshuang: {
//!     type: "basic", savable: true, selectTarget: -1,     // **可作濒死救援牌**
//!     content() {
//!         "step 0"; target.recover();                     // **回复 1 点体力**
//!         "step 1";
//!         if (target.isIn()) target.chooseToDiscard([1, Infinity], "he", "弃置任意张牌并摸等量的牌");
//!         else event.finish();
//!         "step 2";
//!         if (result.bool) target.draw(result.cards.length);   // **弃 N 摸 N**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **性质**：`basic` + **`savable`**（可作**濒死救援**牌）；
//! 2. **效果**：目标**回复 1 点体力**；随后**可弃任意张 `he` 牌并摸等量**（可取消）。
//!
//! 原生实现引用：宿主行为（古剑基本牌机）—— 本样本以**行为面**观测「回复」这一即时效果。
//!
//! **本样本的定位**：验证**回复效果**是否执行（对照：同批食物牌「本体归属」缺失，
//! 但那是 `$gain2` 面；本卡的回复属另一机制）。
//!
//! 未覆盖（登记）：**`savable`（濒死救援）**的使用路径；弃 N 摸 N 的数值面；`ai` 策略面。

mod nc08_common;
use engine_native::{CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason};
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

/// 上游 `card/gujian/card.js:551` 无 `enable: true`（仅 `savable: true`）：
/// 出牌阶段不可主动使用（`WrongPhase`）；濒死救援窗口使用时自身回复 1 点体力至 1，非目标不回复。
#[test]
fn upstream_dinvxuanshuang_recovers() {
    let mut e = gujian_engine(20272601);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max1 - 1);
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dinvxuanshuang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("帝女玄霜在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out_play = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    assert!(
        matches!(out_play, CommandOutcome::Rejected(RejectReason::WrongPhase)),
        "帝女玄霜无 enable:true，出牌阶段应拒绝：{out_play:?}"
    );
    let sha = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == CardKind::Sha)
        .map(|(i, _)| i as engine_native::CardId)
        .expect("杀在目录");
    e.debug_set_hp(0, 1);
    let hp_before = e.debug_hp(1);
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);
    while !(e.turn().current == 1 && e.turn().phase == Phase::Play) {
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    nc08_common::put_hand(&mut e, 1, sha);
    let _ = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![0] },
    });
    let c_save = e.open_choice().cloned().expect("0 号应进入 dying_save");
    assert_eq!(c_save.spec.prompt, "dying_save");
    let out = e.submit(GameCommand {
        actor: c_save.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c_save.id,
            selection: vec![ChoiceOption::Card(card)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "帝女玄霜濒死救援应受理：{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 1, "帝女玄霜：濒死救援回复 1 点体力");
    assert_eq!(e.debug_hp(1), hp_before, "帝女玄霜：非目标 1 号不应回复");
}
