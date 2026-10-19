//! NC08 第⑦层（上游行为对照）样本 105：**来源 U05（古剑）** 食物牌
//! `yuanbaorou`（元宝肉）——并**同时**验证青团（样本 104）遗留的 `once_per_turn` 门控。
//!
//! 上游源码（`.../gujian/card.js` 的 `yuanbaorou`；同族 `qingtuan` 见 `skill.js:225`）：
//!
//! ```js
//! yuanbaorou: { type: "food", enable: true,
//!     filterTarget(card, player, target) { return !target.hasSkill("yuanbaorou"); },
//!     content() { … storage.yuanbaorou_markcount = N; target.addSkill("yuanbaorou"); } }
//! ```
//!
//! 规则基准 golden 表第 45 行记：「**元宝肉（额外一张杀——杀次数校验钩子）**」；
//! 原生 `content/host/step/gujian.rs:254` 注释：「**出牌阶段可额外使用一张杀**（每回合一次）。
//! 上游 `cardUsable num+1` 与 mark 回合递减；效果在**杀次数校验**与**目标校验**，**宿主无机器步骤**」。
//!
//! **本样本的定位（双重目的）**：
//! 1. 验证元宝肉的「**额外出杀**」（同回合第二张杀**被允许**）；
//! 2. 借此**补验青团**的 `once_per_turn` 门控（首杀摸 1、**第二杀不再摸**）——
//!    样本 104 因 `ShaLimit` 无法验证，本样本用元宝肉**解开该阻塞**。
//!
//! 未覆盖（登记）：`markcount` 的逐回合递减；元宝肉与其它出杀次数来源的**叠加**。

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

fn find(e: &Engine, key: &str) -> engine_native::CardId {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains(key))
        .map(|(i, _)| i as engine_native::CardId)
        .unwrap_or_else(|| panic!("{key} 在目录"))
}

/// 上游点 1+2（**行为面**）：元宝肉令同回合**第二张杀被允许**；青团门控令**仅首杀摸 1**。
#[test]
fn upstream_yuanbaorou_extra_sha_and_qingtuan_gate() {
    let mut e = gujian_engine(20273501);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：青团 + 元宝肉 + 两张【杀】
    let food_q = find(&e, "qingtuan");
    nc08_common::put_hand(&mut e, 0, food_q);
    let food_y = find(&e, "yuanbaorou");
    nc08_common::put_hand(&mut e, 0, food_y);
    let shas: Vec<engine_native::CardId> = (0..2)
        .map(|_| {
            let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
            e.debug_place(0, name)
        })
        .collect();
    // ① 吃青团（对自己）→ ② 吃元宝肉（对自己）
    for food in [food_q, food_y] {
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
            "食物牌应可对自己发动：{out:?}"
        );
        drain(&mut e);
    }
    let has_q = e.debug_has_skill(0, "gj_food_qingtuan");
    let has_y = e.debug_has_skill(0, "gj_food_yuanbaorou");
    let h0 = e.debug_hand(0).len();
    // ③ 第一张【杀】
    let out1 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shas[0], targets: vec![1] },
    });
    drain(&mut e);
    let h1 = e.debug_hand(0).len();
    // ④ 第二张【杀】——元宝肉应令其**被允许**
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: shas[1], targets: vec![1] },
    });
    drain(&mut e);
    let h2 = e.debug_hand(0).len();
    eprintln!(
        "DBG 元宝肉+青团：子技 qingtuan={has_q} yuanbaorou={has_y}；吃后手牌={h0}；\
         首杀后={h1}（{out1:?}）；第二杀后={h2}（{out2:?}）"
    );
    assert!(has_q && has_y, "两种食物子技均应授予");
    assert_eq!(
        h1,
        h0,
        "青团：**首张杀应摸 1**（打出杀 −1、摸 1 ⇒ 手牌不变；{h0} → {h1}）"
    );
    assert!(
        matches!(
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "元宝肉：**同回合第二张杀应被允许**（额外一张杀）：{out2:?}"
    );
    assert_eq!(
        h2,
        h1 - 1,
        "青团（`once_per_turn` 门控，本样本**补验**）：**第二张杀不再摸**（应恰为 −1；{h1} → {h2}）"
    );
}
