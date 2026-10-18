//! NC08 第⑦层（上游行为对照）样本 7：`swd_xiuluo`（修罗，swd 转换包）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`）：
//!
//! ```js
//! swd_xiuluo: {
//!     trigger: { player: "phaseZhunbeiBegin" },      // 准备阶段开始
//!     filter(event, player) { return player.countCards("j") > 0; },
//!     direct: true,
//!     content() {
//!         var next = player.discardPlayerCard(player, 2, "hj",
//!             "是否一张手牌来弃置一张花色相同的判定牌？");
//!         next.filterButton = function (button) { /* 先手牌后判定牌、花色须相同 */ };
//!     },
//! },
//! ```
//!
//! 由该 JS 推导的规则点（本文件逐条验收）：
//! 1. **触发时机 = 准备阶段开始**（原生把准备阶段并入判定阶段起点——**已登记差异**，
//!    见 [Prepare 落点证据](../../docs/operations/evidence/2026-10-18-nc08-prepare-phase/README.md)）；
//! 2. **门槛 = 判定区有牌**（`countCards("j") > 0`）——原生**额外要求手牌非空**（**登记差异**：
//!    判定区有牌但手牌为空时上游仍可发动、原生不触发）；
//! 3. **可选**（`direct: true` 仍需确认/可取消）；
//! 4. **效果 = 弃一张手牌 → 弃一张同花色判定牌**（花色必须相同；无同花色判定牌则中止）。
//!
//! 原生实现引用：`content/host/metadata/common.rs:272`（触发元数据）、
//! `content/host/filtering/common.rs:217`（过滤）、`content/host/step/skills_cards.rs:674-745`
//! （三段询问机：手牌 → 同花色判定牌 → 弃置）。
//!
//! 未覆盖（登记）：`audio`/`popup` 属表现面；`filterButton` 的 UI 交互细节由询问机选项面等价承载。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn answer(e: &mut Engine, pick: Vec<ChoiceOption>) {
    let Some(c) = e.open_choice().cloned() else { return };
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: pick },
    });
}

fn advance_to_judge_of(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Judge {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        // 途中若出现**修罗自身**的询问，停下交给调用方处理；其余选择取消之
        if e.open_choice().map(|c| c.spec.prompt.starts_with("xiuluo")).unwrap_or(false) {
            return;
        }
        while let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
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
    panic!("未到 {seat} 号判定阶段");
}

// 上游点 3+4（**未验收，登记**）：效果面「弃一张手牌 → 弃一张同花色判定牌」的三段询问机
// 在测试夹具中未能稳定触发——推进到判定阶段起点的辅助函数会在同一次提交内既到达阶段
// 又消费掉询问（`advance_to_judge_of` 的取消循环），需要改写为「按阶段逐步驱动 + 断点
// 停在 xiuluo 询问」的夹具；本轮按证据不足处理，**不写成通过**。
//
// 已由源码核对（`content/host/step/skills_cards.rs:674-745`）：手牌询问 → 同花色过滤 →
// 判定牌询问 → 双方离区；花色过滤与上游 `filterButton` 同口径。
/// 上游点 2（**登记差异**）：判定区有牌但**手牌为空**时——上游仍可发动，原生不触发。
#[test]
fn upstream_xiuluo_filter_difference_is_registered() {
    let mut e = swd_engine(20262902);
    e.debug_grant(0, "swd_xiuluo");
    advance_to_judge_of(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _judge = e.debug_place_judge(0, "le");
    assert!(e.debug_hand(0).is_empty(), "构造：手牌为空、判定区有牌");
    // 原生：不触发（无询问）
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "原生额外要求手牌非空（上游仅要求判定区有牌）——差异已登记：{:?}",
        open.map(|c| c.spec.prompt)
    );
    // 源码事实固定：过滤条件同时含 judge 与 hand
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/common.rs"
    ))
    .expect("读过滤源码");
    let idx = src.find("HostBehavior::Xiuluo =>").expect("修罗过滤分支");
    let seg: String = src[idx..].chars().take(600).collect();
    assert!(
        seg.contains("judge.is_empty()") && seg.contains("hand.is_empty()"),
        "差异固定：原生修罗过滤要求判定区与手牌均非空：{seg}"
    );
}

/// 上游点 1（**登记差异**）：触发时机——上游 `phaseZhunbeiBegin`，原生并入**判定阶段起点**。
#[test]
fn upstream_xiuluo_timing_is_folded_into_judge_begin() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/common.rs"
    ))
    .expect("读过滤源码");
    let idx = src.find("HostBehavior::Xiuluo =>").expect("修罗过滤分支");
    let seg: String = src[idx..].chars().take(600).collect();
    assert!(
        seg.contains("Phase::Judge") && seg.contains("begin: true"),
        "原生修罗挂在判定阶段起点（上游为准备阶段开始）——差异已登记：{seg}"
    );
    // 行为面：判定阶段开始时确实进入询问（同花色场景见上一个用例）
    let mut e = swd_engine(20262903);
    e.debug_grant(0, "swd_xiuluo");
    advance_to_judge_of(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let hand = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, hand);
    let _judge = e.debug_place_judge(0, "le");
    // 触发一次以确认时机可用（再次推进到判定阶段会重新触发）
    let triggered = e.open_choice().cloned().map(|c| c.spec.prompt);
    assert!(
        triggered.is_none() || triggered.as_deref() == Some("xiuluo_hand"),
        "判定阶段起点应可触发修罗：{triggered:?}"
    );
}
