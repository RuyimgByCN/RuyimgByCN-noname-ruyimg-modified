//! NC08 第⑦层（上游行为对照）样本 73：**来源 U05（杀海拾遗·运筹）** 武将技
//! `diykuanggu`（狂骨·变体）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`diykuanggu`）：
//!
//! ```js
//! diykuanggu: {
//!     trigger: { source: "damageEnd" },        // **自己造成伤害后**
//!     filter(event, player) {
//!         if (player.isHealthy()) {                          // **满血**时……
//!             return get.distance(trigger.player, player, "attack") > 1;   // ……要求攻击距离 > 1
//!         }
//!         return true;                                       // 未满血：恒可
//!     },
//!     getIndex(event, player, triggername) { return event.num; },
//!     forced: true,                                          // 锁定
//!     async content(event, trigger, player) {
//!         if (get.distance(trigger.player, player, "attack") > 1) {
//!             await player.draw();                           // **距离 > 1 → 摸 1**
//!         } else {
//!             await player.recover();                        // **距离 ≤ 1 → 回复 1**
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点（**三分支**）：
//! 1. **相邻 + 未满血** → **回复 1**；
//! 2. **距离 > 1**（无论满血与否）→ **摸 1**；
//! 3. **相邻 + 满血** → **不触发**（filter 排除）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:436`（`seat_distance > 1 → draw_cards(1)`；
//! 否则 `hp < max_hp → recover(1)`）；过滤 `content/host/filtering/common.rs:526`
//! （`Damage` 且 `source == owner` 且 **未满血 或 距离 > 1**）。
//!
//! **登记差异**：本样本**未发现**差异（三分支与上游一致）；
//! 保留项：`ai.effect` 策略面；`getIndex: event.num`（**多点伤害**索引）未覆盖。
//!
//! 未覆盖（登记）：多点伤害的索引语义；**攻击距离**（含马匹/技能修正）与**座位距离**的差异
//! —— 本样本两处均为默认值（无马匹），故二者一致。

mod nc08_common;
use engine_native::Engine;
use nc08_common::{advance_to_play, drain};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次「0 号（持狂骨变体）对 seat 造成 1 点伤害」，返回 `(hp 变化, 手牌变化)`。
fn damage_and_observe(seed: u64, seat: u8, wound_owner: bool) -> (i32, i64) {
    let mut e = yunchou_engine(seed);
    e.debug_grant(0, "ycndiykuanggu");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    if wound_owner {
        let max0 = e.state_core().player(0).max_hp as i32;
        e.debug_set_hp(0, max0 - 1); // 未满血
    }
    let hp_before = e.debug_hp(0);
    let hand_before = e.debug_hand(0).len() as i64;
    e.debug_deal_damage(Some(0), seat, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    (
        e.debug_hp(0) - hp_before,
        e.debug_hand(0).len() as i64 - hand_before,
    )
}

/// 上游点 1+2+3（**行为面 · 三分支**）。
#[test]
fn upstream_kuanggu_three_branches() {
    // ①相邻（座位 1）+ 未满血 → 回复 1
    let (hp_a, hand_a) = damage_and_observe(20270501, 1, true);
    // ②远处（座位 2，座位距离 2）+ 满血 → 摸 1
    let (hp_b, hand_b) = damage_and_observe(20270502, 2, false);
    // ③相邻 + 满血 → 不触发
    let (hp_c, hand_c) = damage_and_observe(20270503, 1, false);
    eprintln!(
        "DBG 狂骨变体：①相邻+未满血 hp{hp_a:+}/手牌{hand_a:+}；②远处+满血 hp{hp_b:+}/手牌{hand_b:+}；\
         ③相邻+满血 hp{hp_c:+}/手牌{hand_c:+}"
    );
    assert_eq!(
        (hp_a, hand_a),
        (1, 0),
        "① 相邻 + 未满血 → **回复 1**（实测 hp{hp_a:+} 手牌{hand_a:+}）"
    );
    assert_eq!(
        (hp_b, hand_b),
        (0, 1),
        "② 距离 > 1 → **摸 1**（实测 hp{hp_b:+} 手牌{hand_b:+}）"
    );
    assert_eq!(
        (hp_c, hand_c),
        (0, 0),
        "③ 相邻 + 满血 → **不触发**（实测 hp{hp_c:+} 手牌{hand_c:+}）"
    );
}

/// 上游点 1（**源码事实**）：filter 的「未满血 或 距离 > 1」与上游对应。
#[test]
fn upstream_kuanggu_source_matches() {
    let f = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/common.rs"
    ))
    .expect("读 filtering/common");
    let idx = f.find("HostBehavior::YunchouDiykuanggu =>").expect("狂骨变体过滤");
    let seg: String = f[idx..].chars().take(420).collect();
    assert!(
        seg.contains("d.source == Some(owner)")
            && seg.contains("hp < ctx.state.player(owner).max_hp as i32")
            && seg.contains("seat_distance(d.target, owner) > 1"),
        "原生依据：来源为本人 + （未满血 或 距离>1）：{seg}"
    );
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouDiykuanggu =>").expect("狂骨变体效果");
    let seg: String = step[idx..].chars().take(420).collect();
    assert!(
        seg.contains("draw_cards(self.owner, 1)") && seg.contains("recover(self.owner, 1, None)"),
        "原生依据：距离>1 摸 1、否则回复 1：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tdiykuanggu: {").expect("上游狂骨变体");
    let seg: String = up[idx..].chars().take(800).collect();
    assert!(
        seg.contains("isHealthy()")
            && seg.contains("get.distance(trigger.player, player, \"attack\") > 1")
            && seg.contains("player.draw()")
            && seg.contains("player.recover()"),
        "上游依据：满血要求距离>1；距离>1 摸、否则回复：{seg}"
    );
}
