//! NC08 第⑦层（上游行为对照）样本 48：`qianxun`（谦逊，**标准面** / 陆逊）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qianxun: {
//!     mod: {
//!         targetEnabled(card, player, target, now) {
//!             if (card.name === "shunshou" || card.name === "lebu") {
//!                 return false;              // 不能成为【顺手牵羊】或【乐不思蜀】的目标
//!             }
//!         },
//!     },
//!     audio: 2,
//! },
//! ```
//!
//! 由 JS 推导的规则点：持技者**不能成为【顺手牵羊】或【乐不思蜀】的目标**（`mod.targetEnabled`
//! → 统一拦截所有目标校验路径）。
//!
//! 原生实现引用（**2 处显式校验**）：
//! - `engine/play_card.rs:688`（顺手牵羊分支：`qianxun → IllegalTargets`）；
//! - `engine/play_card.rs:791`（`kind == CardKind::Le` 分支：`qianxun → IllegalTargets`）。
//!
//! **登记差异（本文件固定）**：**覆盖方式**——上游 `mod.targetEnabled`（统一拦截）；原生
//! **2 处显式校验**（顺手牵羊 + 乐不思蜀，与上游两张牌一一对应）→ 语义等价、**维护性风险**
//! 同 `kongcheng`（新增此类目标路径时须同步）。
//!
//! 未覆盖（登记）：`audio` 表现面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点（**顺手牵羊**）：持谦逊者不能被【顺手牵羊】指定。
#[test]
fn upstream_qianxun_blocks_snatch() {
    let mut e = Engine::new(20268001, 4);
    e.debug_grant(1, "谦逊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号持一张牌（否则顺手牵羊另有「目标无牌」限制，无法区分原因）
    let name1 = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    let _ = e.debug_place(1, name1);
    let snatch = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "snatch")
        .expect("顺手牵羊在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, snatch);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: snatch,
            targets: vec![1],
        },
    });
    eprintln!("DBG 谦逊（顺手牵羊）：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "谦逊：不能成为【顺手牵羊】目标，实测 {out:?}"
    );
    let _ = Phase::Play;
}

/// 上游点（**乐不思蜀**）：持谦逊者不能被【乐不思蜀】指定。
#[test]
fn upstream_qianxun_blocks_lebu() {
    let mut e = Engine::new(20268002, 4);
    e.debug_grant(1, "谦逊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let le = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "le")
        .expect("乐不思蜀在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, le);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: le,
            targets: vec![1],
        },
    });
    eprintln!("DBG 谦逊（乐不思蜀）：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "谦逊：不能成为【乐不思蜀】目标，实测 {out:?}"
    );
    drain(&mut e);
}

/// 上游点（**反例**）：持谦逊者仍可被**其它**锦囊指定（【过河拆桥】）。
#[test]
fn upstream_qianxun_allows_other_tricks() {
    let mut e = Engine::new(20268003, 4);
    e.debug_grant(1, "谦逊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let name1 = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    let _ = e.debug_place(1, name1);
    let guohe = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "dismantle")
        .expect("过河拆桥在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, guohe);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: guohe,
            targets: vec![1],
        },
    });
    eprintln!("DBG 谦逊（过河拆桥，反例）：{out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "谦逊只拦顺手牵羊/乐不思蜀，其它锦囊应可指定，实测 {out:?}"
    );
    drain(&mut e);
}

/// **登记差异**（源码事实固定）：`mod` 统一拦截 ↔ 2 处显式校验（与两张牌一一对应）。
#[test]
fn upstream_qianxun_coverage_difference_is_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tqianxun: {").expect("上游谦逊");
    let seg: String = up[idx..].chars().take(360).collect();
    assert!(
        seg.contains("targetEnabled") && seg.contains("\"shunshou\"") && seg.contains("\"lebu\""),
        "上游依据：统一拦截顺手牵羊/乐不思蜀：{seg}"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card");
    let hits = src.matches("QIAN_XUN").count();
    assert_eq!(
        hits, 2,
        "原生应为 2 处显式校验（顺手牵羊 + 乐不思蜀），实测 {hits} 处"
    );
    assert!(
        src.contains("kind == CardKind::Le"),
        "其中一处应位于 `kind == CardKind::Le`（乐不思蜀）分支"
    );
}
