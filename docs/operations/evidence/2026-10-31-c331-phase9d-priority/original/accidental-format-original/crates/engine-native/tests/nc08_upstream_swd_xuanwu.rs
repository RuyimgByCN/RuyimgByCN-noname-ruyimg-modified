//! NC08 第⑦层（上游行为对照）样本 155：**来源 U06（swd）** 法器
//! `xuanwuzhihuang`（玄武之璜）——**`global` 卡窗口的第 4 张验证**（同族法器，复用样本 154 骨架）。
//!
//! 上游源码（`skill.js` 的 **`g_xuanwuzhihuang`**，**逐字**）：
//!
//! ```js
//! xuanwuzhihuang: { type: "jiqi", global: "g_xuanwuzhihuang" },   // 卡本体无 content
//! g_xuanwuzhihuang: {
//!     trigger: { source: "damageEnd" },                            // **本人造成伤害后**
//!     forced: true,
//!     filter(event, player) {
//!         return player.countCards("h", "xuanwuzhihuang") > 0       // 须持有该卡
//!             && event.num > 0 && player.hp < player.maxHp;         // 且**已受伤**
//!     },
//!     content() { … player.recover(trigger.num); },                 // **回复伤害量**
//! },
//! ```
//!
//! **本样本的定位**：**已受伤的持有者造成伤害后，回复等量体力**（`forced`）。
//!
//! 未覆盖（登记）：`event.num > 0` 的**非正伤害**分支；`showCards` 表现面。

mod nc08_common;
use engine_native::{Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_xuanwuzhihuang`（**forced**）：已受伤的持有者**造成伤害后**回复**伤害量**体力。
#[test]
fn upstream_xuanwuzhihuang_heals_after_damage() {
    let mut e = swd_engine(20288701);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持玄武之璜，并**先受伤**（满足 hp < maxHp）
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("xuanwuzhihuang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("玄武之璜在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 2); // 受伤 2 点 ⇒ 回复空间充足
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    // 0 号对 1 号造成 1 点伤害 ⇒ damageEnd(source=0)
    let dmg = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    eprintln!("DBG 玄武之璜：0 号对 1 号造成 1 点伤害 = {dmg:?}（0 号 hp={hp0}）");
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<engine_native::ChoiceOption> = if c
            .spec
            .options
            .contains(&engine_native::ChoiceOption::Confirm)
        {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    let d0 = e.debug_hp(0) - hp0;
    let d1 = hp1 - e.debug_hp(1);
    eprintln!("DBG 玄武之璜：0 号 hp 变化 {d0:+}（应 +1）；1 号受伤 {d1}（应 1）");
    assert_eq!(d1, 1, "1 号应受到 1 点伤害（原始伤害）");
    assert_eq!(
        d0, 1,
        "玄武之璜（**forced**）：已受伤的持有者造成伤害后应**回复伤害量**（实测 {d0:+}）"
    );
}
