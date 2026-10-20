//! NC08 第⑦层（上游行为对照）样本 160：**来源 U06（swd）** 特殊牌
//! `shenmiguo`（神秘果）——**`global` 卡窗口的第 8 张验证**。
//!
//! 上游源码（`skill.js` 的 `g_shenmiguo`，**逐字摘录**）：
//! ```js
//! shenmiguo: { type: "special" },                       // 卡本体无 content
//! g_shenmiguo: {
//!     trigger: { player: "useCardAfter" },               // 本人使用牌后
//!     direct: true,
//!     filter(event, player) {
//!         if (event.parent.name === "g_shenmiguo") return false;   // 防递归
//!         if (_status.currentPhase !== player) return false;        // 自己的阶段
//!         if (event.parent.parent.name !== "phaseUse") return false; // 出牌阶段直接使用
//!         if (!event.targets || !event.card) return false;
//!         if (event.card.name === "shenmiguo") return false;
//!         if (event.card.name === … /* 五谷/机关/乐/兵/闪电 */) return false;
//!         …
//!     },
//!     content() { /* 用掉一颗神秘果，复制该牌的结算 */ },
//! }
//! ```
//!
//! **原生实现**（`filtering/swd.rs:476-500` + `step/swd_equipment.rs:497-540`）：
//! 过滤「自己在出牌阶段使用**基本牌或非装备锦囊**、且手持有神秘果」；
//! 生效方式为 **`turn.shenmiguo_used = true` + 追加一次 `CardUsed(神秘果)`**（**无询问**）。
//!
//! **本样本的判据**：使用【杀】后 **`turn.shenmiguo_used` 变为 true**
//! （该字段只由本窗口置位 ⇒ 无假阳性）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup, put_hand};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_shenmiguo`：自己在出牌阶段使用基本牌后，**自动用掉一颗神秘果**。
#[test]
fn upstream_shenmiguo_copies_card_after_use() {
    let mut e = swd_engine(20289004);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let fruit = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("shenmiguo"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("神秘果在目录");
    put_hand(&mut e, 0, fruit);
    // **按牌种**取【杀】（名字含 "sha" 的还有 swd_shandian 等，不可按名字）
    let sha = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, _)| {
            e.catalog().card_kind(*i as engine_native::CardId) == engine_native::CardKind::Sha
        })
        .map(|(i, _)| i as engine_native::CardId)
        .expect("有【杀】");
    assert_ne!(sha, fruit);
    put_hand(&mut e, 0, sha);
    assert!(!e.state_core().turn.shenmiguo_used, "初始应为未使用");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "【杀】应可使用：{out:?}"
    );
    drain(&mut e);
    let used = e.state_core().turn.shenmiguo_used;
    eprintln!("DBG 神秘果：使用杀后 shenmiguo_used={used}（应 true）");
    assert!(
        used,
        "神秘果（**forced**）：出牌阶段使用基本牌后应自动用掉一颗神秘果"
    );
}
