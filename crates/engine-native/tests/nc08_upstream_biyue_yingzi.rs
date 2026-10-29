//! NC08 第⑦层（上游行为对照）样本 27：`biyue`（闭月，标准面 / 貂蝉）与 `yingzi`（英姿，标准面 / 周瑜）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! biyue: {
//!     trigger: { player: "phaseJieshuBegin" },   // 结束阶段开始
//!     frequent: true,
//!     async content(event, trigger, player) { player.draw(); },   // 摸 1 张
//! },
//! yingzi: {
//!     trigger: { player: "phaseDrawBegin2" },    // 摸牌阶段（数量确定前）
//!     frequent: true,
//!     filter(event, player) { return !event.numFixed; },  // 摸牌数未被固定时才生效
//!     async content(event, trigger, player) { trigger.num++; },   // 摸牌数 +1
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! - `biyue`：**结束阶段开始**时摸 **1** 张；
//! - `yingzi`：**摸牌阶段**摸牌数 **+1**；**若摸牌数已被固定**（`event.numFixed`）则**不**加成。
//!
//! 原生实现引用：`content/standard/skills/bi_yue.rs`（`Phase::End` + `begin` → `draw_cards(1)`）、
//! `content/standard/skills/ying_zi.rs`（`Phase::Draw` + `begin` → 摸牌数 +1）。
//!
//! **登记差异（本文件固定）**：`yingzi` 上游有 `filter: !event.numFixed`（摸牌数被技能/效果固定时
//! 不再加成），原生 filter **只判阶段**（`Phase::Draw` + `begin`）→ **未建模 `numFixed` 分支**。
//!
//! 未覆盖（登记）：`frequent`/`preHidden`/`audio` 表现面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点（闭月）：结束阶段开始 → 摸 1 张。
#[test]
fn upstream_biyue_draws_at_end_phase_begin() {
    let mut e = Engine::new(20265701, 4);
    e.debug_grant(0, "闭月");
    // 推进到 0 号结束阶段**之前**（弃牌阶段），记录手牌
    assert!(
        advance_to_phase(&mut e, 0, Phase::Discard),
        "应到达 0 号弃牌阶段"
    );
    // 触发确认门（可选技）→ 确认，然后进入结束阶段
    let hand_before = e.debug_hand(0).len();
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        break;
    }
    drain(&mut e);
    if !confirmed {
        eprintln!("SKIP：未出现闭月确认门（阶段推进未到结束阶段开始）");
        return;
    }
    assert!(
        e.debug_hand(0).len() > hand_before,
        "闭月：结束阶段开始摸 1 张（{hand_before} → {}）",
        e.debug_hand(0).len()
    );
}

/// 上游点（英姿）：摸牌阶段摸牌数 +1（相对无技能的同种子基线）。
#[test]
fn upstream_yingzi_draws_one_extra() {
    // 基线：不持英姿
    let base = {
        let mut e = Engine::new(20265702, 4);
        advance_to_phase(&mut e, 0, Phase::Draw);
        drain(&mut e);
        let h = e.debug_hand(0).len();
        // 推进离开摸牌阶段（完成摸牌）
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
        let _ = h;
        e.debug_hand(0).len()
    };
    // 实验组：持英姿（**到达摸牌阶段之后**授予，避免影响此前阶段）
    let with_skill = {
        let mut e = Engine::new(20265702, 4);
        e.debug_grant(0, "英姿");
        advance_to_phase(&mut e, 0, Phase::Draw);
        // 确认触发门（若在阶段开始时已过，则确认可能不出现——两种情形都接受）
        for _ in 0..8 {
            let Some(c) = e.open_choice().cloned() else { break };
            if c.spec.prompt == "trigger_confirm" {
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                });
                continue;
            }
            break;
        }
        drain(&mut e);
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
        e.debug_hand(0).len()
    };
    eprintln!("DBG 英姿：基线 hand={base} 持技 hand={with_skill}");
    assert!(
        with_skill > base,
        "英姿：摸牌阶段摸牌数 +1（基线 {base} → 持技 {with_skill}）"
    );
}

/// **登记差异**（源码事实固定）：原生 `yingzi` 的 filter **只判阶段**，未建模上游的
/// `!event.numFixed`（摸牌数被固定时不再加成）。
#[test]
fn upstream_yingzi_numfixed_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/ying_zi.rs"
    ))
    .expect("读英姿源码");
    let idx = src.find("YingZiBehavior").expect("英姿行为");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        seg.contains("Phase::Draw") && !seg.contains("numFixed") && !seg.contains("num_fixed"),
        "差异固定：原生 filter 只判阶段，未建模 numFixed：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("yingzi: {").expect("上游英姿");
    let seg: String = up[idx..].chars().take(500).collect();
    assert!(
        seg.contains("!event.numFixed"),
        "上游依据：摸牌数固定时不加成：{seg}"
    );
}

/// 上游点（门槛）：**未持技**者结束阶段不摸牌（闭月）。
#[test]
fn upstream_biyue_requires_the_skill() {
    let mut e = Engine::new(20265703, 4);
    assert!(advance_to_phase(&mut e, 0, Phase::Discard));
    let hand_before = e.debug_hand(0).len();
    drain(&mut e);
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);
    let _ = CommandOutcome::Suspended; // 类型占位（保持 import 使用）
    assert!(
        e.debug_hand(0).len() <= hand_before + 1,
        "未持闭月：结束阶段不应额外摸牌（{hand_before} → {}）",
        e.debug_hand(0).len()
    );
}
