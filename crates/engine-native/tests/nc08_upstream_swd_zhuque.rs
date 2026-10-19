//! NC08 第⑦层（上游行为对照）样本 154：**来源 U06（swd）** 法器
//! `zhuquezhizhang`（朱雀之璋）——**`global` 卡窗口的第 3 张定位**（承接映射表与样本 150/153 的方法）。
//!
//! 上游源码（`card.js` + `skill.js` 的 **`g_zhuquezhizhang`**，**逐字**）：
//!
//! ```js
//! zhuquezhizhang: { type: "jiqi", global: "g_zhuquezhizhang" },   // 卡本体无 content
//! g_zhuquezhizhang: {
//!     trigger: { player: "damageEnd" },                            // **本人受到伤害后**
//!     forced: true,
//!     filter(event, player) {
//!         return event.source && event.source !== player && event.source.isAlive()
//!             && player.countCards("h", "zhuquezhizhang") > 0;      // **须持有该卡**
//!     },
//!     content() { … trigger.source.damage("fire"); … },             // **对来源造成 1 点火伤**
//! },
//! ```
//!
//! **本样本的定位**：**持有者受伤后，对伤害来源造成 1 点火伤**（`forced` ⇒ 无需确认）。
//!
//! 未覆盖（登记）：`check` 的 AI 面；`showCards` 的表现面。

mod nc08_common;
use engine_native::{Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_zhuquezhizhang`（**forced**）：持有者受伤后对**来源**造成 1 点火伤。
#[test]
fn upstream_zhuquezhizhang_retaliates_on_source() {
    let mut e = swd_engine(20288601);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持朱雀之璋；1 号对 0 号造成 1 点伤害
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("zhuquezhizhang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("朱雀之璋在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    let dmg = e.debug_deal_damage(
        Some(1),
        0,
        1,
        engine_native::event::DamageNature::Normal,
    );
    eprintln!("DBG 朱雀之璋：1 号对 0 号造成 1 点伤害 = {dmg:?}");
    // 应答可能出现的询问（forced 通常无询问）
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Confirm) {
                vec![engine_native::ChoiceOption::Confirm]
            } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    let d0 = hp0 - e.debug_hp(0);
    let d1 = hp1 - e.debug_hp(1);
    eprintln!("DBG 朱雀之璋：0 号受伤 {d0} 点（应 1）；1 号受伤 {d1} 点（应 1 = 反伤）");
    assert_eq!(d0, 1, "0 号应受到 1 点伤害（原始伤害）");
    assert_eq!(
        d1, 1,
        "朱雀之璋（**forced**）：持有者受伤后应对**来源**造成 1 点火伤（实测 1 号受伤 {d1} 点）"
    );
}
