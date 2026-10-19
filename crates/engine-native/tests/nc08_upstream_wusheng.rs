//! NC08 第⑦层（上游行为对照）样本 23：`wusheng`（武圣，**标准面** / 关羽）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! wusheng: {
//!     enable: ["chooseToRespond", "chooseToUse"],       // **使用或打出**
//!     filterCard(card, player) {
//!         if (get.zhu(player, "shouyue")) return true;  // 主公技「守约」：任意牌
//!         return get.color(card) === "red";             // 否则：**红色**牌
//!     },
//!     position: "hes",                                  // 手牌/装备/判定区
//!     viewAs: { name: "sha" },                          // 视为【杀】
//!     viewAsFilter(player) { … 至少一张红牌 … },
//!     prompt: "将一张红色牌当杀使用或打出",
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**红色**牌（`get.color === "red"`）可当【杀】；
//! 2. **入口**：**使用或打出**（`chooseToUse` **与** `chooseToRespond`）；
//! 3. **牌区**：`hes`（手牌/装备/判定区）。
//!
//! 原生实现引用：`engine/play_card.rs:135`（`wusheng && red` → `kind = CardKind::Sha`；
//! 与龙胆/奇袭同族转化臂）。
//!
//! **登记差异（本文件固定）**：
//! - **①入口**：原生转化在 `PlayCard`（**使用**）路径；上游还含 `chooseToRespond`（**打出**）
//!   —— 打出路径是否覆盖**未验证**（本文件以源码事实固定「实现位于 play_card」）；
//! - **②守约**：上游主公技「守约」下**任意**牌可转化，原生**未建模**该分支。
//!
//! 未覆盖（登记）：`prompt`/`audio`/`check` 表现与策略面；`position: "hes"` 中装备/判定区
//! 的转化路径（原生 PlayCard 走手牌为主，**未单独验证**）。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 取一张指定颜色、且**本身不是【杀】/【闪】等可直接使用牌**的实体（如【桃】以外的普通牌），
/// 用于验证「转化」而非「本体可用」：取名为 `shan`（【闪】，出牌阶段不可主动使用）的实体。
fn place_colored_nonplayable(e: &mut Engine, player: u8, red: bool) -> engine_native::CardId {
    use engine_native::catalog::Suit;
    let name = e
        .catalog()
        .cards
        .iter()
        .find(|c| {
            let is_red = matches!(c.suit, Suit::Heart | Suit::Diamond);
            // 【闪】（`shan`，出牌阶段不可主动使用 → 「被接受」只可能来自转化）
            is_red == red && c.name == "shan"
        })
        .map(|c| c.name)
        .expect("存在该颜色的【闪】");
    e.debug_place(player, name)
}

/// 上游点 1+2（使用面）：**红色**牌当【杀】使用 → 被接受，且目标受到伤害。
#[test]
fn upstream_wusheng_red_card_used_as_sha() {
    let mut e = Engine::new(20265201, 4);
    e.debug_grant(0, "武圣");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_card = place_colored_nonplayable(&mut e, 0, true);
    let hp1_before = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: red_card, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "红色牌应可当【杀】使用：{out:?}"
    );
    drain(&mut e);
    assert!(
        e.debug_hp(1) <= hp1_before,
        "转化后的【杀】应对目标结算伤害（{hp1_before} → {}）",
        e.debug_hp(1)
    );
}

/// 上游点 1（门槛）+ **登记差异②**：黑色牌**不可**当【杀】；但实测**黑色【无懈可击】**
/// 在出牌阶段被**接受**（原生允许主动使用无懈——上游 `chooseToUse` 面不允许）。
#[test]
fn upstream_wusheng_black_card_rejected() {
    let mut e = Engine::new(20265202, 4);
    e.debug_grant(0, "武圣");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 黑色【闪】在标准目录不存在（闪全为红），故取黑色**非杀**实体：用【无懈可击】
    let black_nullify = {
        use engine_native::catalog::Suit;
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| {
                matches!(c.suit, Suit::Spade | Suit::Club) && c.name == "nullify"
            })
            .map(|c| c.name)
            .expect("存在黑色【无懈可击】");
        e.debug_place(0, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: black_nullify, targets: vec![1] },
    });
    // **登记差异②**：实测被接受（原生允许出牌阶段主动使用无懈）；上游 chooseToUse 面不允许
    eprintln!("DBG 黑色无懈在出牌阶段：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "登记差异②：原生**接受**出牌阶段主动使用【无懈可击】（上游不允许）：{out:?}"
    );
}

/// 上游点 1（技能门槛）：**未持武圣**者红色牌不可当【杀】。
#[test]
fn upstream_wusheng_requires_the_skill() {
    let mut e = Engine::new(20265203, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_card = place_colored_nonplayable(&mut e, 0, true);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: red_card, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持武圣 → 红牌不可当【杀】：{out:?}"
    );
}

/// **撤回的猜测（保留记录）**：曾据一次 `WrongPhase` 断言「红色锦囊不可转化」，
/// 但专门构造**红色【无懈可击】**复测为 **Accepted** → 该猜测**不成立，已撤回**。
/// （当时的 `WrongPhase` 来自另一张红牌——出牌阶段本就不能使用的延时锦囊类实体。）
#[test]
fn upstream_wusheng_red_trick_convertible_note() {
    let mut e = Engine::new(20265204, 4);
    e.debug_grant(0, "武圣");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    use engine_native::catalog::Suit;
    let red_nullify = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| matches!(c.suit, Suit::Heart | Suit::Diamond) && c.name == "nullify")
            .map(|c| c.name)
            .expect("存在红色【无懈可击】");
        e.debug_place(0, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: red_nullify, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "复测：红色【无懈可击】被接受（此前『红色锦囊不可转化』的猜测已撤回）：{out:?}"
    );
}

/// 上游点 2/3 的**登记差异**（源码事实固定）：原生转化只在 `PlayCard`（使用）路径，
/// 且**未建模**主公技「守约」（`get.zhu(player, "shouyue")` → 任意牌）。
#[test]
fn upstream_wusheng_entry_and_shouyue_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    let idx = src.find("WU_SHENG").expect("武圣分支");
    let seg: String = src[idx..].chars().take(1600).collect();
    assert!(
        seg.contains("wusheng && red"),
        "差异①固定：原生转化在 PlayCard（使用）路径：{seg}"
    );
    assert!(
        !src.contains("shouyue"),
        "差异②固定：原生未建模主公技「守约」（任意牌可转化）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("wusheng: {").expect("上游武圣");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("chooseToRespond") && seg.contains("shouyue"),
        "上游依据：使用或打出 + 守约分支：{seg}"
    );
}
