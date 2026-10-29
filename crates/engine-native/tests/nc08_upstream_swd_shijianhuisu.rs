//! NC08 第⑦层（上游行为对照）样本 64：**来源 U05（杀海拾遗·swd）** 卡牌
//! `hslingjian_shijianhuisu`（时间回溯 / 灵剑）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1392`）：
//!
//! ```js
//! hslingjian_shijianhuisu: {
//!     type: "hslingjian", vanish: true, enable: true, derivation: true,
//!     filterTarget(card, player, target) {
//!         return target !== player && target.countCards("e") > 0;   // **非自己 + 有装备**
//!     },
//!     content() {
//!         var es = target.getCards("e");
//!         target.gain(es);                                          // **装备区全部牌收回手牌**
//!         target.$gain2(es);
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**非自己**且**装备区有牌**；
//! 2. **效果**：目标**装备区的全部牌收回手牌**（`gain`）。
//!
//! 原生实现引用：`content/host/step/swd_card_effects.rs:1025`
//! （`HostBehavior::SwdShijianhuisuCard`：遍历目标 `equips` → `revoke_equip_skills` +
//! `hand.push`；本体入弃堆）。上游另有 `_equip1..4` 子技（装备后摸牌 / 距离修饰等），
//! 属该灵剑的**装备态**效果，**未纳入本样本**（登记）。
//!
//! **登记差异（本文件固定）**：**目标门槛未在原生侧校验**——上游 `filterTarget` 要求
//! 「非自己 + 目标装备区有牌」；原生 `SwdShijianhuisuCard` 只做效果（`if alive`），
//! **未见**对目标装备数为 0 的拒绝（与同类 host 卡「过滤器统一走 `filter_swd`」的实现方式有关，
//! 具体是否覆盖 `countCards("e") > 0` **未核对**，登记待办）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **swd 包**建局（与其它 U05 样本同法）。
fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 定位「时间回溯」卡（包内名为 `swd_shijianhuisu1`）。
fn shijianhuisu_card(e: &Engine) -> engine_native::CardId {
    e.catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("shijianhuisu"))
        .expect("时间回溯在目录") as engine_native::CardId
}

/// 上游点 2（**行为面**）：目标**装备区全部牌收回手牌**（装备技随之撤销）。
#[test]
fn upstream_shijianhuisu_recalls_equips() {
    let mut e = swd_engine(20269601);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号装备一件防具（装备区 1 张）
    let armor = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("armor"))
            .expect("防具") as engine_native::CardId;
        nc08_common::put_equips(&mut e, 1, id);
        id
    };
    let card = shijianhuisu_card(&e);
    nc08_common::put_hand(&mut e, 0, card);
    let equips_before = e.debug_equips(1).len();
    let hand_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 时间回溯：发动={out:?}（目标装备 {equips_before} 张）");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "时间回溯应可发动：{out:?}"
    );
    drain(&mut e);
    let equips_after = e.debug_equips(1).len();
    let hand_after = e.debug_hand(1).len();
    eprintln!("DBG 时间回溯：装备 {equips_before} → {equips_after}；手牌 {hand_before} → {hand_after}");
    assert_eq!(equips_after, 0, "时间回溯：目标装备区应清空");
    assert!(
        hand_after > hand_before,
        "时间回溯：装备区的牌应收回目标手牌（{hand_before} → {hand_after}）"
    );
    assert!(
        e.debug_hand(1).contains(&armor),
        "时间回溯：被收回的应是那张装备牌（{armor}）"
    );
    let _ = ChoiceOption::Pass;
}

/// 上游点 1（**门槛**）：上游要求目标**装备区有牌**；原生是否校验 → 源码事实登记。
#[test]
fn upstream_shijianhuisu_target_gate_note() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/swd_lingjian_cards.rs"
    ))
    .expect("读 swd_card_effects");
    let idx = src.find("SwdShijianhuisuCard =>").expect("时间回溯分支");
    let seg: String = src[idx..].chars().take(900).collect();
    assert!(
        seg.contains("revoke_equip_skills") && seg.contains("hand.push"),
        "原生依据：撤销装备技并把牌收回手牌：{seg}"
    );
    assert!(
        !seg.contains("countCards") && !seg.contains("equips.is_empty"),
        "登记：原生该分支**未**校验「目标装备区有牌」（上游 filterTarget 要求）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js",
        ),
    )
    .expect("读上游 swd/card.js");
    let idx = up.find("hslingjian_shijianhuisu: {").expect("上游时间回溯");
    let seg: String = up[idx..].chars().take(500).collect();
    assert!(
        seg.contains("countCards(\"e\") > 0") && seg.contains("target !== player"),
        "上游依据：非自己 + 装备区有牌：{seg}"
    );
}
