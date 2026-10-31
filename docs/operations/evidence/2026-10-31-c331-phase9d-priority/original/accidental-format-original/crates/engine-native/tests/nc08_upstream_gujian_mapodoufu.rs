//! NC08 第⑦层（上游行为对照）样本 93：**来源 U05（古剑）** 食物牌
//! `mapodoufu`（麻婆豆腐）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`mapodoufu`）：
//!
//! ```js
//! mapodoufu: {
//!     type: "food", enable: true,
//!     filterTarget(card, player, target) { return !target.hasSkill("mapodoufu"); },
//!     content() {
//!         if (target === targets[0] && cards.length) target.$gain2(cards);   // **目标获得此牌**
//!         target.storage.mapodoufu = card;
//!         target.storage.mapodoufu_markcount = 2;                            // **2 个计数**
//!         target.addSkill("mapodoufu");                                      // 计时子技
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**未持** `mapodoufu` 的角色；
//! 2. **效果**：目标**获得此牌** + 挂 **2 个计数** + 获得计时子技。
//!
//! 原生实现引用：`content/host/step/gujian.rs:212`（食物机通用映射：
//! `CardKind::Mapodoufu => (GJ_FOOD_MAPODOUFU_KEY, GJ_FOOD_ROUNDS_DEFAULT)`）、
//! `:267`（`GujianFoodMapodoufu` 分支）。
//!
//! **本样本的定位**：与样本 89（鲤鱼汤）**同一检查点**——「**目标是否真的获得此牌**」——
//! 用于判断「用牌效果缺失」是**鲤鱼汤个案**还是**食物牌共性**。
//!
//! 未覆盖（登记）：计数的逐轮结算；门槛（`!hasSkill`）。

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

/// 上游点 2（**行为面**）：目标**获得此牌**（手牌 +1）。
///
/// **本轮未验收（登记，含诊断）**：实测发动 `Accepted`，但**目标手牌 0 → 0**、
/// **该牌进入弃牌堆**（`弃堆含此牌=true`）✗ —— 与样本 89（鲤鱼汤）**完全同一现象**
/// ⇒ 「用牌效果（`$gain2` + storage + 计数 + 授技）缺失」**至少是食物牌共性（2/2）**
/// （详见修复清单第 37 项）。用例以 `#[ignore]` 作为修复后的验收条件。
#[test]
#[ignore = "麻婆豆腐：目标应获得此牌（上游 $gain2 + storage + addSkill）——与鲤鱼汤同现象，见第 37 项"]
fn upstream_mapodoufu_target_gains_the_card() {
    let mut e = gujian_engine(20272301);
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
            .find(|(_, c)| c.name.contains("mapodoufu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("麻婆豆腐在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 麻婆豆腐：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "麻婆豆腐应可发动：{out:?}"
    );
    drain(&mut e);
    let hand = e.debug_hand(1);
    eprintln!(
        "DBG 麻婆豆腐：1 号是否获得子技 gj_food_mapodoufu={}；1 号手牌 {h1_before} → {}（含此牌={}）；弃堆含此牌={}",
        e.debug_has_skill(1, "gj_food_mapodoufu"),
        hand.len(),
        hand.contains(&card),
        e.state_core().discard.contains(&card)
    );
    assert!(
        hand.contains(&card),
        "麻婆豆腐：目标应**获得此牌**（上游 `target.$gain2(cards)`；手牌 {h1_before} → {}）",
        hand.len()
    );
}
