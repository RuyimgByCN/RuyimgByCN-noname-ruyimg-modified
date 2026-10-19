//! NC08 第⑦层（上游行为对照）样本 109：**来源 U05（古剑）** 食物牌
//! `molicha`（茉莉茶）。
//!
//! 上游源码（`.../gujian/skill.js` 的子技 `molicha`）：
//!
//! ```js
//! molicha: {
//!     mark: "card", trigger: { player: "phaseAfter" }, forced: true,
//!     intro: { content: (storage, player) =>
//!         "你不能成为其他角色的黑色牌的目标（剩余" + player.storage.molicha_markcount + "回合）" },
//!     mod: {
//!         targetEnabled(card, player, target) {
//!             if (player !== target && get.color(card) === "black") return false;   // **黑色牌不可指定**
//!         },
//!     },
//!     content() { player.storage.molicha_markcount--; … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **效果**：持有者**不能成为其他角色【黑色】牌的目标**（`mod.targetEnabled` 返回 false）；
//! 2. **范围**：仅**其他角色**（`player !== target`）的**黑色**牌；**红色牌不受限**；
//! 3. **倒计时**：`phaseAfter` 递减 `markcount`，归零移除。
//!
//! 原生实现引用：`content/host/step/gujian.rs:331`（`HostBehavior::GujianFoodMolicha => Flow::Done`）；
//! 键 `content/host.rs:346`（`HostSkillDecl`）、`constants.rs`（`GJ_FOOD_MOLICHA_KEY`）。
//!
//! **本样本的定位**：验证「**黑色牌不可指定 / 红色牌可指定**」这一**限制面**。
//! 若限制未生效 → **如实登记**（不写成通过）。
//!
//! 未覆盖（登记）：倒计时（`markcount`）递减与归零移除；`mark: "card"` 的标记面。

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

/// 跑一次：0 号喝茉莉茶后，1 号用**黑/红**【杀】打 0 号，返回该次出牌的结果。
fn probe(seed: u64, black: bool) -> CommandOutcome {
    let mut e = gujian_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达 0 号出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tea = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("molicha"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("茉莉茶在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: tea, targets: vec![0] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "茉莉茶应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_molicha");
    // 轮到 1 号，用指定颜色的【杀】打 0 号
    assert!(advance_to_phase(&mut e, 1, Phase::Play), "应到达 1 号出牌阶段");
    let sha = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                c.name == "sha"
                    && if black {
                        matches!(
                            c.suit,
                            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                        )
                    } else {
                        matches!(
                            c.suit,
                            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                        )
                    }
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("对应颜色的杀");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let res = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![0] },
    });
    eprintln!(
        "DBG 茉莉茶：子技={has}；1 号用**{}**【杀】打 0 号 → {res:?}",
        if black { "黑" } else { "红" }
    );
    drain(&mut e);
    res
}

/// 上游点 1+2（**行为面**）：**黑色**牌不可指定（Rejected）；**红色**牌可指定（Accepted）。
#[test]
fn upstream_molicha_blocks_black_cards() {
    let black = probe(20273901, true);
    let red = probe(20273902, false);
    eprintln!("DBG 茉莉茶：黑牌结果={black:?}；红牌结果={red:?}");
    assert!(
        matches!(black, CommandOutcome::Rejected(_)),
        "茉莉茶：持有者**不能成为其他角色黑色牌的目标**（上游 mod.targetEnabled 返回 false）：{black:?}"
    );
    assert!(
        matches!(
            red,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "茉莉茶：**红色**牌不受限（应可指定）：{red:?}"
    );
}
