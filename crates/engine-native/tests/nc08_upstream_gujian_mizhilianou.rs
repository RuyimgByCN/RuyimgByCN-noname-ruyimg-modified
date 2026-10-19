//! NC08 第⑦层（上游行为对照）样本 112：**来源 U05（古剑）** 食物牌
//! `mizhilianou`（蜜汁藕）——**食物族最后一张**。
//!
//! 上游源码（`.../gujian/skill.js` 的子技 `mizhilianou` + 其 `subSkill.use`）：
//!
//! ```js
//! mizhilianou: {
//!     mark: "card", trigger: { player: "phaseAfter" }, forced: true,
//!     intro: { content: (storage, player) =>
//!         "你可以将一张红桃牌当作桃使用（剩余" + player.storage.mizhilianou_markcount + "回合）" },
//!     group: "mizhilianou_use",
//!     subSkill: { use: { enable: "chooseToUse", filterCard: { suit: "heart" },
//!                        position: "he", viewAs: { name: "tao" } } },   // **红桃 → 桃**
//!     content() { player.storage.mizhilianou_markcount--; … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**红桃**牌（**`position: "he"`**：手牌 + 装备区）**当【桃】使用**；
//! 2. **倒计时**：`phaseAfter` 递减 `markcount`，归零移除子技。
//!
//! 原生实现引用：`content/host/step/gujian.rs:375`（`HostBehavior::GujianFoodMizhilianou => Flow::Done`）
//! —— **该空 step 是合理的**（效果是**转化**，应在**出牌路径**实现，同武圣/倾国）；
//! 故本样本**以行为面判定**（不因 step 为空推断缺失——见样本 109 茉莉茶的教训）。
//!
//! **本样本的定位**：验证「**红桃牌当桃使用**」是否生效。
//!
//! 未覆盖（登记）：`position: "he"` 的**装备区**入口（牌区族）；`markcount` 递减与归零移除。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1（**行为面**）：持有子技时**红桃牌可当【桃】使用**（回复 1）。
///
/// **本轮未验收（差异登记）**：**对照组**先验证「普通【桃】对自己」→ **`Accepted`**（hp 3 → 4）
/// ⇒ **目标形态正确、夹具无问题**；随后「**红桃非桃牌当【桃】**」→ **`Rejected(IllegalTargets)`** ✗
/// ⇒ **转化未实现**（上游 `subSkill.use`：`filterCard: { suit: "heart" }` + `viewAs: { name: "tao" }`）。
/// 详见修复清单第 41 项；用例以 `#[ignore]` 作为修复后的验收条件。
#[test]
#[ignore = "蜜汁藕：红桃牌应可当【桃】使用（上游 viewAs tao）——实测 Rejected，见第 41 项"]
fn upstream_mizhilianou_heart_as_tao() {
    let mut e = gujian_engine(20274201);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号受伤（便于观测回复）
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("mizhilianou"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("蜜汁藕在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: food, targets: vec![0] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "蜜汁藕应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_mizhilianou");
    // 放一张**红桃**牌（非【桃】）→ 应可当【桃】对自己使用
    let heart = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                matches!(c.suit, engine_native::catalog::Suit::Heart) && c.name != "tao"
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("红桃非桃牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp_before = e.debug_hp(0);
    // **对照组**：先看普通【桃】能否对自己使用（排除「桃的目标形态不对」这一夹具因素）
    let plain_tao = {
        let name = e.catalog().cards.iter().find(|c| c.name == "tao").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let ctrl = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // 【桃】对自己：引擎按**无显式目标**形态受理（对照实验已确认 vec![0] 会被拒）
        action: GameAction::PlayCard { card: plain_tao, targets: vec![] },
    });
    drain(&mut e);
    eprintln!("DBG 蜜汁藕（对照）：普通【桃】对自己 → {ctrl:?}");
    let hp_after_ctrl = e.debug_hp(0);
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: heart, targets: vec![] },
    });
    drain(&mut e);
    let hp_after = e.debug_hp(0);
    eprintln!(
        "DBG 蜜汁藕：子技={has}；对照后 hp={hp_after_ctrl}；红桃牌（非桃）当桃使用={out2:?}；         hp {hp_after_ctrl} → {hp_after}"
    );
    assert!(has, "蜜汁藕：应获得子技 gj_food_mizhilianou");
    assert!(
        matches!(
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "蜜汁藕：**红桃牌应可当【桃】使用**（上游 viewAs tao / filterCard heart）：{out2:?}"
    );
    assert_eq!(
        hp_after,
        hp_before + 1,
        "蜜汁藕：当作【桃】使用应**回复 1 点体力**（{hp_before} → {hp_after}）"
    );
}
