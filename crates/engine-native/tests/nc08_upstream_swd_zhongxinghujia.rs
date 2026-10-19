//! NC08 第⑦层（上游行为对照）样本 128：**来源 U06（swd）** 灵件牌
//! `hslingjian_zhongxinghujia`（重型护甲，包内名 `swd_zhongxinghujia1`）。
//!
//! 上游源码（`.../swd/card.js`，`hslingjian_zhongxinghujia`）：
//!
//! ```js
//! hslingjian_zhongxinghujia: {
//!     type: "hslingjian", vanish: true, derivation: true, derivationpack: "swd",
//!     filterTarget(card, player, target) { return !target.isMin(); },   // **体力不是最少**
//!     content() { … 生成一张 equip2（防具）并令目标装备 … },
//! },
//! ```
//!
//! 原生实现引用：`engine/play_card.rs:1645`（`CardKind::Hslingjian` 的**逐卡目标规则**）——
//! 其 `zhongxinghujia` 分支为：
//!
//! ```rust
//! } else if cname.contains("zhongxinghujia") {
//!     targets.len() == 1 && alive && hand.len() as i32 > hp      // **手牌多于体力**
//! }
//! ```
//!
//! **本样本的定位**：核对**门槛口径**——上游为「**体力不是最少**」，原生为「**手牌多于体力**」。
//! 两者**不等价**：构造「**手牌 > 体力**」局面应被受理（**原生口径**）；
//! 构造「**体力不是最少但手牌 ≤ 体力**」局面，上游应受理、**原生预期被拒** ⇒ **差异登记**。
//!
//! 未覆盖（登记）：`vanish`/`derivation`（衍生/消失）面；生成防具并装备的效果面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次：给 1 号 `hand_cards` 张手牌并把其体力设为 `hp`；返回发动结果。
fn probe(seed: u64, hand_cards: usize, hp: i32) -> CommandOutcome {
    let mut e = swd_engine(seed);
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
            .find(|(_, c)| c.name.contains("zhongxinghujia"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("重型护甲在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 1 号：`hand_cards` 张手牌 + 指定体力
    for _ in 0..hand_cards {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name);
    }
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, hp.min(max1));
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!(
        "DBG 重型护甲：1 号手牌={hand_cards}、体力={} ⇒ {out:?}",
        e.debug_hp(1)
    );
    drain(&mut e);
    out
}

/// **门槛口径核对**：原生为「手牌 > 体力」；上游为「体力不是最少」。
#[test]
fn upstream_zhongxinghujia_target_gate() {
    // ① 手牌 3 > 体力 2 ⇒ 原生口径成立 ⇒ 应受理
    let a = probe(20286101, 3, 2);
    // ② 体力不是最少（4 人局 1 号满血 4，0 号被打到 2）但手牌 1 ≤ 体力 4
    //    ⇒ 上游（!isMin）应受理；原生预期**被拒** ⇒ 差异
    let b = probe(20286102, 1, 4);
    eprintln!("DBG 重型护甲：①手牌>体力 → {a:?}；②体力非最少但手牌≤体力 → {b:?}");
    assert!(
        matches!(a, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "重型护甲（**原生口径**：手牌 > 体力）应受理：{a:?}"
    );
    assert!(
        matches!(b, CommandOutcome::Rejected(_)),
        "重型护甲（**上游口径**：`!target.isMin()`）在本局面应受理；实测 {b:?} ⇒ \
         说明原生用的是「**手牌 > 体力**」口径（**差异**，见修复清单第 50 项）"
    );
}
