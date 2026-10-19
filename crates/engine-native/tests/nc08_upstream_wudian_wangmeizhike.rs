//! NC08 第⑦层（上游行为对照）样本 130：**来源 U10（玩点论杀）** 锦囊
//! `wangmeizhike`（望梅止渴）——**该来源的首个行为面样本**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js`）：
//!
//! ```js
//! wangmeizhike: {
//!     type: "trick", enable: true, filterTarget: true,
//!     async content(event, trigger, player) {
//!         const target = event.targets[0];
//!         let rec = false;
//!         if (target.isMinHp() && target.isDamaged()) { await target.recover(); rec = true; }  // 体力最少且受伤 → 回复 1
//!         if (target.isMinHandcard()) { await target.draw(rec ? 1 : 2); }                      // 手牌最少 → 摸 1 或 2
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **分支**：**体力最少且已受伤** → **回复 1**；**手牌最少** → **摸 1**（若已回复）**或 2**。
//!
//! 原生实现引用：宿主行为（玩点论杀锦囊机）。
//!
//! **本样本的定位**：验证**两支**——①体力最少+受伤+手牌最少 → 回复 1 + 摸 1；②仅手牌最少 → 摸 2。
//!
//! 未覆盖（登记）：`isMinHp`/`isMinHandcard` 的**并列最少**语义；`ai` 面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn wudian_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wudian-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读玩点论杀 cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次：`wounded` 决定 1 号是否受伤（并成为唯一体力最少者）。返回 `(hp 变化, 手牌变化)`。
fn run(seed: u64, wounded: bool) -> (i32, i64) {
    let mut e = wudian_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("wangmeizhike"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("望梅止渴在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 0/2/3 号各 2 张；1 号 1 张（**手牌最少**）
    for p in [0usize, 2, 3] {
        for _ in 0..2 {
            let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
            e.debug_place(p as engine_native::PlayerId, name);
        }
    }
    let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
    e.debug_place(1, name);
    if wounded {
        let max1 = e.state_core().player(1).max_hp as i32;
        e.debug_set_hp(1, max1 - 1); // 受伤 ⇒ 唯一体力最少
    }
    let hp1 = e.debug_hp(1);
    let h1 = e.debug_hand(1).len() as i64;
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
        "望梅止渴应可发动：{out:?}"
    );
    drain(&mut e);
    let d_hp = e.debug_hp(1) - hp1;
    let d_h = e.debug_hand(1).len() as i64 - h1;
    eprintln!("DBG 望梅止渴：wounded={wounded} → 1 号 hp{d_hp:+}、手牌{d_h:+}");
    (d_hp, d_h)
}

/// 上游点 2（**行为面 · 两支**）：受伤且最少 → 回复 1 + 摸 1；仅手牌最少 → 摸 2。
#[test]
fn upstream_wangmeizhike_two_branches() {
    let (hp_w, h_w) = run(20286301, true);
    let (hp_n, h_n) = run(20286302, false);
    eprintln!("DBG 望梅止渴：受伤支 → hp{hp_w:+}/手牌{h_w:+}；未受伤支 → hp{hp_n:+}/手牌{h_n:+}");
    assert_eq!(hp_w, 1, "望梅止渴（**体力最少且受伤**）：应**回复 1**");
    assert_eq!(h_w, 1, "望梅止渴（已回复）：**手牌最少**应**摸 1**（而非 2）");
    assert_eq!(hp_n, 0, "望梅止渴（未受伤）：**不回复**");
    assert_eq!(h_n, 2, "望梅止渴（未回复）：**手牌最少**应**摸 2**");
}
