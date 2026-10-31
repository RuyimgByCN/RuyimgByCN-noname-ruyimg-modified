//! NC08 第⑦层（上游行为对照）样本 76：**来源 U05（杀海拾遗·运筹）** 武将技
//! `jieyan`（隙焰）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`jieyan`）：
//!
//! ```js
//! jieyan: {
//!     enable: "phaseUse",
//!     filter(event, player) { return player.hasCard(card => get.color(card) === "red", "h"); },
//!     filterCard(card, player) { return get.color(card) === "red"; },     // **弃一张红手牌**
//!     filterTarget: true,                        // **任何人**（含使用者）
//!     selectTarget: -1, multitarget: true, multiline: true,
//!     async content(event, trigger, player) {
//!         const targets = event.targets.sortBySeat();
//!         for (const target of targets) {
//!             await target.damage("fire", "nocard");      // **各受 1 点火焰伤害（无来源牌）**
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/费用**：出牌阶段；**弃一张红色手牌**；
//! 2. **目标**：`filterTarget: true` + `selectTarget: -1` → **全部角色（按 noname 语义含使用者）**；
//! 3. **效果**：**按座位序**各受 **1 点火焰伤害**（`nocard` → 无来源牌）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:191`（找一张**红色**手牌弃置 → `seats` =
//! **除使用者外**的全部存活角色 → 逐座位 `Damage(Fire, 1, card=None)`）。
//!
//! **登记差异（本文件固定）**：**使用者是否被波及** —— 上游 `filterTarget: true` +
//! `selectTarget: -1`（**含使用者**）↔ 原生 `seats` 过滤 `i != self.owner`（**排除使用者**）
//! → **规则损失/更宽松需按上游语义裁定**（本样本只固定该差异，不裁定方向）。
//!
//! 未覆盖（登记）：`multitarget`/`multiline` 的 UI 表现面；`sortBySeat` 与原生座位序的**起始点**
//! （原生从 0 号起，上游从当前回合角色起——单次发动下均为「座位序」，未逐点比对起始点）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+3（**行为面**）：弃一张红手牌 → **其余角色各受 1 点火伤**（原生排除使用者）。
#[test]
fn upstream_jieyan_discards_red_and_damages_others() {
    let mut e = yunchou_engine(20270801);
    e.debug_grant(0, "ycnjieyan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：一张红牌 + 一张黑牌（黑牌应保留）
    let red = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                )
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("红牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let black = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                )
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黑牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    let skill = e.catalog().skill_by_name("ycnjieyan").expect("隙焰在目录");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    eprintln!("DBG 隙焰：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "隙焰应可发动：{out:?}"
    );
    drain(&mut e);
    let hp_after: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    let hand = e.debug_hand(0);
    eprintln!("DBG 隙焰：hp {hp:?} → {hp_after:?}；自己手牌={hand:?}（红={red} 黑={black}）");
    assert!(!hand.contains(&red), "隙焰：应弃置一张**红色**手牌");
    assert!(hand.contains(&black), "隙焰：黑牌应保留");
    assert_eq!(
        hp_after[0], hp[0],
        "**差异登记**：原生**排除使用者**（0 号 hp 不变：{} → {}）",
        hp[0], hp_after[0]
    );
    for p in 1..4 {
        assert_eq!(
            hp_after[p],
            hp[p] - 1,
            "隙焰：其余角色（{p} 号）应各受 1 点火伤（{} → {}）",
            hp[p],
            hp_after[p]
        );
    }
}

/// **登记差异**（源码事实固定）：使用者是否被波及。
#[test]
fn upstream_jieyan_owner_exclusion_is_registered() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouJieyan =>").expect("隙焰分支");
    // 分支较长（含弃牌 + 座位枚举）→ 取到该分支结束附近（2000 字符）
    let seg: String = step[idx..].chars().take(2000).collect();
    assert!(
        seg.contains("i != self.owner"),
        "差异固定：原生排除使用者（i != self.owner）：{seg}"
    );
    assert!(
        seg.contains("DamageNature::Fire"),
        "原生依据：火焰伤害：{seg}"
    );
    let up = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
    ))
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tjieyan: {").expect("上游隙焰");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("filterTarget: true")
            && seg.contains("selectTarget: -1")
            && seg.contains("target.damage(\"fire\", \"nocard\")"),
        "上游依据：filterTarget true + selectTarget -1（含使用者）+ 火焰伤害：{seg}"
    );
}
