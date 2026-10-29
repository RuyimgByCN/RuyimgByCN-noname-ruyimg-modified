//! NC08 第⑦层（上游行为对照）样本 101：**来源 U05（古剑）** 机关牌
//! `liutouge`（六骰格）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`liutouge`）：
//!
//! ```js
//! liutouge: {
//!     type: "jiguan", enable: true, filterTarget: true, wuxieable: true,   // 任意角色
//!     content() {
//!         if (player.getEnemies().includes(target)) target.getDebuff();     // 敌方 → **减益**
//!         else target.getBuff();                                           // 否则 → **增益**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果**：按「**是否敌方**」施加 `getDebuff()`（**减益**）或 `getBuff()`（**增益**）——
//!    上游为**效果池**（含随机性），具体条目由 `getBuff`/`getDebuff` 决定。
//!
//! 原生实现引用：`content/host/step/jiguan.rs:361`（`HostBehavior::JiguanLiutougeCard`），
//! 其代码注释即 **「自己 → 摸 1（增益简化）；其他角色 → 弃首张手牌（减益简化；
//! 登记差异：上游随机效果池）」**。
//!
//! **登记差异（本文件固定）**：**随机效果池 → 确定性简化** —— 上游 `getBuff`/`getDebuff` 为
//! **效果池**（含随机条目）；原生为**确定性两条**（自己摸 1 / 他人弃首张手牌）⇒ 属
//! 「**随机→确定性**」族（本项为**第 3 例**）。
//!
//! 未覆盖（登记）：`wuxieable` 无懈面；上游效果池的**具体条目清单**（需读 `getBuff` 实现）。

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

/// 上游点 2（**行为面 · 两支**）：对自己 → **摸 1**；对其他角色 → **弃首张手牌**。
#[test]
fn upstream_liutouge_two_branches() {
    // ① 对自己：摸 1
    let self_gain = {
        let mut e = gujian_engine(20273101);
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
                .find(|(_, c)| c.name.contains("liutouge"))
                .map(|(i, _)| i as engine_native::CardId)
                .expect("六骰格在目录");
            nc08_common::put_hand(&mut e, 0, id);
            id
        };
        let h0_before = e.debug_hand(0).len();
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            // `filterTarget: true` → 目标须**显式**给出（对自己即 vec![0]）
            action: GameAction::PlayCard { card, targets: vec![0] },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "六骰格（对自己）应可发动：{out:?}"
        );
        drain(&mut e);
        // 打出本体 −1、增益摸 1 → 手牌数不变
        e.debug_hand(0).len() as i64 - h0_before as i64
    };
    // ② 对其他角色：其弃首张手牌
    let other = {
        let mut e = gujian_engine(20273102);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let _ = e.debug_place(1, name);
        let card = {
            let id = e
                .catalog()
                .cards
                .iter()
                .enumerate()
                .find(|(_, c)| c.name.contains("liutouge"))
                .map(|(i, _)| i as engine_native::CardId)
                .expect("六骰格在目录");
            nc08_common::put_hand(&mut e, 0, id);
            id
        };
        let h1_before = e.debug_hand(1).len();
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![1] },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "六骰格（对其他角色）应可发动：{out:?}"
        );
        drain(&mut e);
        e.debug_hand(1).len() as i64 - h1_before as i64
    };
    eprintln!(
        "DBG 六骰格：对自己（增益）手牌净变化={self_gain}（打出本体 −1、摸 1 ⇒ 0）；\
         对其他角色（减益）手牌净变化={other}（应 −1）"
    );
    assert_eq!(self_gain, 0, "六骰格（增益简化）：对自己应**摸 1**（打出本体 −1 ⇒ 净 0）");
    assert_eq!(other, -1, "六骰格（减益简化）：其他角色应**弃一张手牌**（净 −1）");
}

/// **登记差异**（源码事实固定）：随机效果池 → 确定性简化。
#[test]
fn upstream_liutouge_random_pool_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/jiguan.rs"
    ))
    .expect("读 jiguan");
    let idx = src.find("JiguanLiutougeCard =>").expect("六骰格分支");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        seg.contains("增益简化") && seg.contains("减益简化") && seg.contains("上游随机效果池"),
        "原生注释应自述简化与差异：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js",
        ),
    )
    .expect("读上游 gujian/card.js");
    let idx = up.find("\tliutouge: {").expect("上游六骰格");
    let seg: String = up[idx..].chars().take(400).collect();
    assert!(
        seg.contains("getDebuff()") && seg.contains("getBuff()"),
        "上游依据：按敌我施加 getDebuff/getBuff（效果池）：{seg}"
    );
}
