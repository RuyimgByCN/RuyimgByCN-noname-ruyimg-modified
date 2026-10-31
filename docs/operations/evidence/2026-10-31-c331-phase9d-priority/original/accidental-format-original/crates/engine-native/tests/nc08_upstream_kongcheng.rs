//! NC08 第⑦层（上游行为对照）样本 47：`kongcheng`（空城，**标准面** / 诸葛亮）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! kongcheng: {
//!     mod: {
//!         targetEnabled(card, player, target, now) {
//!             if (!target.hasCards("h")) {                  // 目标**无手牌**
//!                 if (card.name === "sha" || card.name === "juedou") {
//!                     return false;                          // 不能成为【杀】/【决斗】目标
//!                 }
//!             }
//!         },
//!     },
//!     group: "kongcheng1",
//!     ai: { noh: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：持技者**手牌为空**时**不能成为【杀】或【决斗】的目标**（`mod.targetEnabled`
//! → 对所有目标校验路径生效）。
//!
//! 原生实现引用（**3 处显式校验**）：
//! - `engine/play_card.rs:643`（决斗目标）；
//! - `engine/play_sha.rs:306`（【杀】目标）；
//! - `snap/asks.rs:187`（借刀类目标选择）。
//! 三处判据一致：`kongcheng && core.player(target).hand.is_empty() → IllegalTargets`。
//!
//! **登记差异（本文件固定）**：**覆盖方式**——上游是 `mod.targetEnabled`（**统一拦截所有
//! 目标校验路径**）；原生是**3 处显式校验**（已覆盖【杀】/【决斗】/借刀类）→ 语义等价但
//! **新增目标路径时需同步**（登记，非缺陷）。
//!
//! 未覆盖（登记）：`kongcheng1`（`loseEnd` 组技，上游表现/AI 面）、`ai.noh` 策略面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点（**主判据**）：持空城且**无手牌** → 不能被【杀】指定。
#[test]
fn upstream_kongcheng_blocks_sha_when_hand_empty() {
    let mut e = Engine::new(20267901, 4);
    e.debug_grant(1, "空城");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号持空城且**手牌为空**；0 号持【杀】
    let sha = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    eprintln!("DBG 空城（无手牌）：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "空城：无手牌者不能成为【杀】目标，实测 {out:?}"
    );
    let _ = Phase::Play;
}

/// 上游点（**反例**）：持空城但**有手牌** → 可被【杀】指定。
#[test]
fn upstream_kongcheng_allows_sha_when_hand_not_empty() {
    let mut e = Engine::new(20267902, 4);
    e.debug_grant(1, "空城");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    // 1 号持一张手牌 → 空城不生效
    let name1 = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    let _ = e.debug_place(1, name1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    eprintln!("DBG 空城（有手牌）：{out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "持空城但**有手牌**时应可被指定，实测 {out:?}"
    );
    drain(&mut e);
}

/// **登记差异**（源码事实固定）：上游 `mod` 统一拦截 ↔ 原生 3 处显式校验。
#[test]
fn upstream_kongcheng_coverage_difference_is_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tkongcheng: {").expect("上游空城");
    let seg: String = up[idx..].chars().take(600).collect();
    assert!(
        seg.contains("targetEnabled")
            && seg.contains("card.name === \"sha\"")
            && seg.contains("juedou"),
        "上游依据：mod.targetEnabled 拦截【杀】/【决斗】：{seg}"
    );
    let sites = [
        ("src/engine/play_card.rs", "决斗目标"),
        ("src/engine/play_sha.rs", "【杀】目标"),
        ("../engine-native-kernel/src/snap/asks.rs", "借刀类目标选择"),
    ];
    for (rel, label) in sites {
        let src =
            std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel))
                .unwrap_or_else(|_| panic!("读 {rel}"));
        assert!(
            src.contains("KONG_CHENG"),
            "原生 {label} 应有空城校验（{rel}）"
        );
    }
    let def = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/standard_skills.rs"
    ))
    .expect("读标准技能表");
    assert!(
        def.contains("权限型能力：出牌校验经 skill_by_name 查询持有者"),
        "原生注释应说明空城为权限型能力（无触发时机）"
    );
}
