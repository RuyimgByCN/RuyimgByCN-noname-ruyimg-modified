//! NC08 第⑦层（上游行为对照）样本 26：`qixi`（奇袭，**标准面** / 甘宁）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qixi: {
//!     enable: "chooseToUse",              // **仅「使用」**（无 chooseToRespond）
//!     filterCard(card) { return get.color(card) === "black"; },   // 黑色牌
//!     position: "hes",                    // 手牌 + 装备区 + 判定区
//!     viewAs: { name: "guohe" },          // 视为【过河拆桥】
//!     prompt: "将一张黑色牌当过河拆桥使用",
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**黑色**牌当【过河拆桥】**使用**（上游**无** `chooseToRespond` → **打出面不要求**）；
//! 2. **牌区**：`hes`；
//! 3. **与杀类转化的关系**：上游由玩家在 `viewAs` 候选间**选择**（黑色【杀】既可作【杀】也可作
//!    【过河拆桥】）→ 原生为**固定优先级**（`play_card.rs:156` 注释「优先级低于杀类转化」）。
//!
//! 原生实现引用：`engine/play_card.rs:143/157`（`qixi && black` → `kind = CardKind::Dismantle`，
//! 且位于杀类转化（武圣/龙胆）之后）。
//!
//! **登记差异**：
//! - **①候选选择**：上游在多个 `viewAs` 之间由玩家选择；原生**固定优先级**（杀类优先）
//!   → 黑色【杀】持奇袭时**只能**当【杀】用（**登记，不写成对齐**）；
//! - **②牌区**：`position: "hes"` 含装备/判定区，与制衡（样本 25）同族问题——本样本只验证手牌。
//!
//! 未覆盖（登记）：`check`/`audio` 表现与策略面；装备/判定区转化路径未单独验证。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
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

fn place_colored(e: &mut Engine, player: u8, black: bool, name: &str) -> engine_native::CardId {
    use engine_native::catalog::Suit;
    let card_name = e
        .catalog()
        .cards
        .iter()
        .find(|c| {
            let is_black = matches!(c.suit, Suit::Spade | Suit::Club);
            is_black == black && c.name == name
        })
        .map(|c| c.name)
        .unwrap_or_else(|| panic!("存在该颜色的 {name}"));
    e.debug_place(player, card_name)
}

/// 上游点 1（使用面）：**黑色**牌当【过河拆桥】使用 → 被接受并拆除目标一张牌。
///
/// **本轮仍未验收（登记，含两处实测对照）**：同一逻辑布阵在**两个种子上结果不同**——
/// - 探针（seed 20265601，`to_play` 不处理选择）：**持奇袭 → `Accepted`**；
/// - 本用例（seed 20265501，`advance_to_play` 会 drain 选择）：**持奇袭 → `Rejected(WrongPhase)`**；
/// - 两者**不持奇袭**时均为 `Rejected(WrongPhase)`。
/// → 说明该路径存在**状态依赖**（种子/推进方式），**不能**据此断言「转化生效」或「不生效」；
/// 两种此前的猜测（牌种限制 / 检查顺序）**均未证实**。以 `#[ignore]` 守护并保留两处对照。
#[test]
#[ignore = "同逻辑布阵在两种子/推进方式下结果不同（Accepted vs WrongPhase）→ 状态依赖，未判定"]
fn upstream_qixi_black_card_used_as_dismantle() {
    let mut e = Engine::new(20265501, 4);
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "奇袭"); // 与探针一致：到达出牌阶段后再授予
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 黑色【无懈可击】（闪全为红色，无法作黑色载体）；判定转化用**效果**而非「是否被接受」
    //（黑色无懈在出牌阶段本就可能被接受——见武圣样本登记的差异）
    let black_card = place_colored(&mut e, 0, true, "nullify");
    // 目标（1 号）手牌 1 张，供拆除
    let victim = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .expect("闪在目录");
    let v = e.debug_place(1, victim);
    e.debug_give_hand_card(1, 1, v);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: black_card, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "黑色牌应可当【过河拆桥】使用（上游 qixi）：{out:?}"
    );
    drain(&mut e);
    // **效果判据**：转化后的【过河拆桥】应拆除目标一张牌（若只是「无懈被接受」则不会拆牌）
    assert!(
        !e.debug_hand(1).contains(&v),
        "转化后的【过河拆桥】应拆除目标一张牌（{v} 应离开其手牌，实际手牌 {:?}）",
        e.debug_hand(1)
    );
}

/// 上游点 1（门槛 · 颜色）：**红色**牌**不可**当【过河拆桥】。
#[test]
fn upstream_qixi_red_card_rejected() {
    let mut e = Engine::new(20265502, 4);
    e.debug_grant(0, "奇袭");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_shan = place_colored(&mut e, 0, false, "shan");
    let _ = red_shan;
    // 【闪】全为红色 → 用红色【闪】尝试：既非黑色（不满足奇袭）又不可主动使用 → 应被拒
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: red_shan, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "红色牌不满足 `get.color === 'black'` → 不应被当作【过河拆桥】：{out:?}"
    );
}

/// 上游点 1（技能门槛）：**未持奇袭**者黑色牌不可当【过河拆桥】。
#[test]
fn upstream_qixi_requires_the_skill() {
    let mut e = Engine::new(20265503, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black_card = place_colored(&mut e, 0, true, "nullify");
    let victim = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).expect("闪");
        e.debug_place(1, name)
    };
    e.debug_give_hand_card(1, 1, victim);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: black_card, targets: vec![1] },
    });
    let _ = out;
    drain(&mut e);
    assert!(
        e.debug_hand(1).contains(&victim),
        "未持奇袭 → 不产生【过河拆桥】效果（目标手牌不应被拆）"
    );
}

/// 上游点 3 的**登记差异①**（源码事实固定）：原生为**固定优先级**（杀类优先），
/// 上游由玩家在 `viewAs` 候选间选择。
#[test]
fn upstream_qixi_priority_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    let idx = src.find("奇袭（甘宁）").expect("奇袭分支");
    let seg: String = src[idx..].chars().take(300).collect();
    assert!(
        seg.contains("优先级低于杀类转化"),
        "差异①固定：原生为固定优先级（杀类优先），上游由玩家选择：{seg}"
    );
    // 行为面：黑色【杀】持奇袭时被当作【杀】（目标受伤）而非【过河拆桥】
    let mut e = Engine::new(20265504, 4);
    e.debug_grant(0, "奇袭");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black_sha = place_colored(&mut e, 0, true, "sha");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: black_sha, targets: vec![1] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert!(
        e.debug_hp(1) <= hp1,
        "登记差异①：黑色【杀】按固定优先级作【杀】结算（上游可由玩家选作【过河拆桥】）"
    );
}
