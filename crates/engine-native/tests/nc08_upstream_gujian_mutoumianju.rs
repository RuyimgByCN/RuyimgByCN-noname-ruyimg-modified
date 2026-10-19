//! NC08 第⑦层（上游行为对照）样本 103：**来源 U05（古剑）** 装备牌
//! `mutoumianju`（木头面具）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`mutoumianju`）：
//!
//! ```js
//! mutoumianju: {
//!     fullskin: true, type: "equip", subtype: "equip2",     // **防具位**
//!     skills: ["mutoumianju_skill"],                        // **装备即授予技能**
//!     ai: { equipValue: 4 },
//!     onLose() { … delete player.getStat().skill.mutoumianju_skill; },   // **失去时清理**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **类型**：`equip` / **`subtype: "equip2"`（防具位）**；
//! 2. **效果**：装备后**授予 `mutoumianju_skill`**；**失去装备**时**清理**（`onLose`）。
//!
//! 原生实现引用：`content/host/constants.rs:112`（`GJ_MUTOU_SKILL_KEY = "gj_mutoumianju_skill"`）；
//! 规则基准 golden 表第 53 行记：「木头面具（**view-as 装备技**——`equip_skills` + `once_used`，
//! `EquipSlot::Armor` 注册）」。
//!
//! **本样本的定位**：验证「**装备后技能确实授予**」（技能名取自**原生常量**，不猜名）。
//!
//! 未覆盖（登记）：`onLose`（失去装备时的**技能移除**）；`mutoumianju_skill` 的**效果面**。

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

/// 上游点 2（**行为面**）：装备后**授予技能**（技能名取自原生常量 `GJ_MUTOU_SKILL_KEY`）。
#[test]
fn upstream_mutoumianju_grants_skill_on_equip() {
    let mut e = gujian_engine(20273301);
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
            .find(|(_, c)| c.name.contains("mutoumianju"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("木头面具在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let equips_before = e.debug_equips(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 木头面具：装备={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "木头面具应可装备：{out:?}"
    );
    drain(&mut e);
    let equips_after = e.debug_equips(0).len();
    // 技能名取自**原生常量**（`gj_mutoumianju_skill`），不按上游键猜名
    let has_skill = e.debug_has_skill(0, "gj_mutoumianju_skill");
    eprintln!(
        "DBG 木头面具：装备区 {equips_before} → {equips_after}；是否获得技能 gj_mutoumianju_skill={has_skill}"
    );
    assert!(
        equips_after > equips_before,
        "木头面具：应进入装备区（{equips_before} → {equips_after}）"
    );
    assert!(
        has_skill,
        "木头面具：装备后应**授予技能**（上游 `skills: [\"mutoumianju_skill\"]`；\
         原生常量 `gj_mutoumianju_skill`）"
    );
}
