//! NC08 第⑦层（上游行为对照）样本 22：`keji`（克己，**标准面** / 吕蒙）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! keji: {
//!     trigger: { player: "phaseDiscardBefore" },   // 弃牌阶段**前**
//!     frequent(event, player) { return player.needsToDiscard(); },
//!     filter(event, player) {
//!         if (player.getHistory("skipped").includes("phaseUse")) return true;  // 跳过出牌阶段 → 满足
//!         const history = player.getHistory("useCard").concat(player.getHistory("respond"));
//!         for (const evt of history) {
//!             if (evt.card.name === "sha" && evt.isPhaseUsing()) return false; // **使用或打出**过杀 → 不发动
//!         }
//!         return true;
//!     },
//!     async content(event, trigger, player) { … }   // 跳过弃牌阶段
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 弃牌阶段前**（`phaseDiscardBefore`）；
//! 2. **门槛 = 本回合未使用、也未打出【杀】**（`useCard` **与** `respond` 历史都查）；
//! 3. **效果 = 跳过弃牌阶段**（不弃牌）。
//!
//! 原生实现引用：`kernel/intrinsic.rs:1391`（弃牌流程：持有 `KE_JI` 且
//! `!turn.sha_used_play_phase` → 不做手牌上限裁剪，即视为无限手牌上限）。
//!
//! **登记差异（本文件固定）**：
//! - **①实现方式**：上游是「跳过弃牌阶段」（事件级），原生是「手牌上限视为无限」
//!   （结果等价：都不弃牌，但事件面不同）；
//! - **②门槛范围**：上游查 `useCard` **与** `respond` 两个历史；原生只看
//!   `turn.sha_used_play_phase`（**出牌阶段使用**）→ **打出【杀】（响应）是否计入未验证**。
//!
//! 未覆盖（登记）：`frequent`/`audio` 表现面。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn advance_to_phase(e: &mut Engine, seat: u8, phase: Phase) -> bool {
    for _ in 0..32 {
        if e.turn().current == seat && e.turn().phase == phase {
            return true;
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
    false
}

/// 上游点 2+3：本回合**未使用【杀】** → 克己生效（无需弃牌）。
#[test]
fn upstream_keji_no_sha_means_no_discard() {
    let mut e = Engine::new(20265001, 4);
    e.debug_grant(0, "克己");
    // **先**把 0 号手牌堆到 9 张（上限检查发生在**弃牌阶段开始**，故必须在进入该阶段前堆高）
    for _ in 0..9 {
        let c = place_by_prefix(&mut e, "shan", 0);
        e.debug_give_hand_card(0, 0, c);
    }
    let hand_before = e.debug_hand(0).len();
    assert!(
        advance_to_phase(&mut e, 0, Phase::Discard),
        "应到达 0 号弃牌阶段"
    );
    // 诊断（登记）：克己是否真的抑制了裁剪
    let at_discard = e.debug_hand(0).len();
    eprintln!(
        "DBG 克己：回合前 hand={hand_before}，弃牌阶段 hand={at_discard}，上限={}，sha_flag={}，有克己={}",
        e.state_core().hand_limit(e.catalog(), 0),
        e.state_core().turn.sha_used_play_phase,
        e.catalog()
            .skill_by_name("克己")
            .map(|d| e.state_core().has_skill(0, d))
            .unwrap_or(false)
    );
    // 克己生效的判据：进入弃牌阶段后手牌**仍高于上限**（未被裁剪）。
    // 注意不能与「回合前手牌」比较——摸牌阶段会+2（实测 13 → 15）。
    let limit = e.state_core().hand_limit(e.catalog(), 0);
    assert!(
        at_discard > limit,
        "克己生效：本回合未使用杀 → 弃牌阶段开始不裁剪（hand={at_discard} 应 > 上限 {limit}）"
    );
}

/// 上游点 2（不满足）：本回合**使用过【杀】** → 克己不生效（须按上限弃牌）。
///
/// **根因已查明（2026-10-19 同轮）**：弃牌上限的裁剪发生在**进入弃牌阶段时**（阶段开始），
/// 而初版夹具是在**进入该阶段之后**才堆手牌 → 阶段开始的检查已过，故看不到任何裁剪。
/// 探针实测（正常角色、回合前堆到 15 张）：`Play: hand=15 → Discard: hand=4`（上限 4）——
/// **弃牌确实被强制执行**，此前「上限未强制执行」的猜测**不成立**（已撤回）。
/// 本用例改为在**回合开始前**堆手牌后再比较。
#[test]
fn upstream_keji_after_using_sha_must_discard() {
    let mut e = Engine::new(20265002, 4);
    e.debug_grant(0, "克己");
    // 同样先堆手牌（进入弃牌阶段前）
    for _ in 0..9 {
        let c = place_by_prefix(&mut e, "shan", 0);
        e.debug_give_hand_card(0, 0, c);
    }
    // 出牌阶段使用一张【杀】
    assert!(
        advance_to_phase(&mut e, 0, Phase::Play),
        "应到达 0 号出牌阶段"
    );
    let sha = place_by_prefix(&mut e, "sha", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.state_core().turn.sha_used_play_phase,
        "出牌阶段使用杀应置位"
    );
    let hand_before = e.debug_hand(0).len();
    assert!(
        advance_to_phase(&mut e, 0, Phase::Discard),
        "应到达弃牌阶段"
    );
    let hand_at_discard = e.debug_hand(0).len();
    assert!(
        hand_at_discard < hand_before,
        "使用过杀 → 克己不生效，弃牌阶段开始即按上限裁剪：{hand_before} → {hand_at_discard}（上限 {}）",
        e.state_core().hand_limit(e.catalog(), 0)
    );
}

/// 上游点 2 的**登记差异②**（源码事实固定）：原生只看 `sha_used_play_phase`
/// （出牌阶段**使用**），未建模上游的 `respond`（**打出**）历史。
#[test]
fn upstream_keji_respond_history_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/intrinsic.rs"
    ))
    .expect("读 intrinsic 源码");
    let idx = src.find("克己（吕蒙）").expect("克己分支");
    let seg: String = src[idx..].chars().take(400).collect();
    assert!(
        seg.contains("sha_used_play_phase") && !seg.contains("respond"),
        "差异②固定：原生只看 sha_used_play_phase（出牌阶段使用），未含 respond（打出）历史：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("keji: {").expect("上游克己");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("getHistory(\"respond\")"),
        "上游依据：useCard 与 respond 两个历史都查：{seg}"
    );
}
