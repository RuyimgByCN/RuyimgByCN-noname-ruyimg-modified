//! NC08 第⑦层（上游行为对照）样本 159：**来源 U06（swd）** 法器
//! `baishouzhihu`（白兽之琥）——**`Phase{ begin: true }` 家族的第三个成员**。
//!
//! 上游源码（`skill.js` 的 `g_baishouzhihu`，**逐字**）：
//! ```js
//! baishouzhihu: { type: "jiqi", global: "g_baishouzhihu" },
//! g_baishouzhihu: {
//!     trigger: { player: "discardEnd" },        // **自己弃牌阶段结束**
//!     direct: true,
//!     filter(event, player) { return player.countCards("h") > 0; },
//!     content() { /* 选一名角色，弃其一张红桃/方块牌，然后摸 1 */ },
//! },
//! ```
//!
//! **原生实现**（`filtering/swd.rs:409-421`）：
//! ```rust
//! EventData::Phase { player, begin, .. }
//!     if *player == owner && *begin
//!         && 手牌有红桃/方块 && hand_has_kind(owner, Baishouzhihu)
//! ```
//!
//! **注意**：原生的 filter **只判 `begin`、不判阶段名**（上游是 `discardEnd`）——
//! 这是**登记差异**（触发面比上游宽）。本样本据此**在任意阶段开始**观察窗口是否开启。
//!
//! **本样本的定位**：与第 55/56 项同属 `Phase{begin:true}` 家族，构成**第三个数据点**：
//! - 若**窗口开启** ⇒ 问题特异于 `Phase::Judge`（第 55/56 项）；
//! - 若**窗口不开启** ⇒ 问题在 `Phase` 事件派发本身（第 55/56 项的共同根因）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{next_dedup, put_hand};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 找一张**红桃或方块**的普通牌（非白兽之琥本身）。
fn find_red(e: &Engine) -> Option<engine_native::CardId> {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| {
            !c.name.contains("baishouzhihu")
                && matches!(
                    c.suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                )
                && *i > 0
        })
        .map(|(i, _)| i as engine_native::CardId)
}

/// 白兽之琥：手持本卡 + 手牌有红牌 ⇒ **任意阶段开始**应开启 `baishouzhihu_target` 询问。
#[test]
fn upstream_baishouzhihu_opens_target_window() {
    let mut e = swd_engine(20289002);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("baishouzhihu"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("白兽之琥在目录");
    put_hand(&mut e, 0, card);
    let red = find_red(&e).expect("目录有红桃/方块牌");
    put_hand(&mut e, 0, red);

    // 逐步推进，观察是否出现 baishouzhihu_target
    // ⚠ 更正：`Phase { begin: true }` 只在**回合起点的判定阶段**发射（`kernel/events.rs:753/789`），
    // 建局时第 1 轮已发过 ⇒ 必须推进到**下一轮**（`round >= 2`）才可能看到窗口。
    let mut seen: Option<String> = None;
    for _ in 0..260 {
        if e.turn().round >= 2 && seen.is_none() {
            // 到达下一轮判定阶段后，再多走一步让阶段开始事件进入结算
        }
        if let Some(c) = e.open_choice() {
            if c.spec.prompt.contains("baishouzhihu") {
                seen = Some(c.spec.prompt.to_string());
                break;
            }
        }
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        if !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            break;
        }
        if let Some(c) = e.open_choice() {
            if c.spec.prompt.contains("baishouzhihu") {
                seen = Some(c.spec.prompt.to_string());
                break;
            }
            // 其它询问：确认/放弃
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.max.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        }
    }
    eprintln!(
        "DBG 白兽之琥：回合={} 阶段={:?}；窗口 prompt={:?}",
        e.turn().round,
        e.turn().phase,
        seen
    );
    assert!(
        seen.is_some(),
        "白兽之琥应开启 `baishouzhihu_target` 询问（`Phase{{begin:true}}` 家族）"
    );
}

/// **诊断探针**：跑一个完整回合后，确认「本卡仍在手牌」「阶段/回合推进正常」，
/// 用以区分「窗口未开」是「手牌被清空」还是「Phase 事件未派发给该技能」。
#[test]
fn probe_baishouzhihu_hand_persists() {
    let mut e = swd_engine(20289003);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("baishouzhihu"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("白兽之琥在目录");
    put_hand(&mut e, 0, card);
    let red = find_red(&e).expect("红牌");
    put_hand(&mut e, 0, red);
    let hand_before = e.debug_hand(0).len();
    for _ in 0..14 {
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        while let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.max.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        }
    }
    let hand = e.debug_hand(0);
    eprintln!(
        "DBG 诊断：手牌 {hand_before} → {}；仍含本卡={}；仍含红牌={}；回合={} 阶段={:?}",
        hand.len(),
        hand.contains(&card),
        hand.contains(&red),
        e.turn().round,
        e.turn().phase
    );
    assert!(hand.contains(&card), "白兽之琥应仍在手牌（否则窗口不触发是手牌被清）");
}
