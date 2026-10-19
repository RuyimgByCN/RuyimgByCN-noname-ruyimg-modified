//! NC08 上游对照样本的**共享夹具助手**（2026-10-19）。
//!
//! 立此模块的直接原因：`debug_place(player, name)` **只按名字查找**，同名牌可能有多花色实例
//! （【无懈可击】有 ♦ 红与 ♣ 黑；【杀】有黑桃/梅花/红桃/方块）→ 依赖名字布阵的**颜色/花色断言**
//! 会命中非预期实例，产生**伪差异**或**脆弱通过**（武圣、奇袭两样本各踩一次）。
//!
//! 因此凡涉及颜色/花色/牌区的布阵，一律走本模块的 **按 id** 助手。
//!
//! 用法（在样本文件中）：
//! ```ignore
//! mod nc08_common;
//! use nc08_common::{advance_to_play, drain, place_by, put_judge, put_equips};
//! ```

#![allow(dead_code)]

use engine_native::catalog::Suit;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

/// 下一个去重号（供各样本共用，避免各自维护计数器）。
pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 处理所有开放询问（触发确认一律**确认**；可选/可取消的取放弃）。
pub fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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

/// 推进到指定座位的指定阶段（每步处理询问）。
pub fn advance_to_phase(e: &mut Engine, seat: u8, phase: Phase) -> bool {
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

/// 推进到出牌阶段（断言式；失败 panic）。
pub fn advance_to_play(e: &mut Engine, seat: u8) {
    assert!(advance_to_phase(e, seat, Phase::Play), "未到 {seat} 号出牌阶段");
}

/// 按**名字谓词 + 花色谓词**取 card id（**不**经 `debug_place` 的名字查找）。
pub fn card_id(e: &Engine, name: &str, want_red: Option<bool>) -> engine_native::CardId {
    let idx = e
        .catalog()
        .cards
        .iter()
        .position(|c| {
            let red = matches!(c.suit, Suit::Heart | Suit::Diamond);
            c.name == name && want_red.map(|w| w == red).unwrap_or(true)
        })
        .unwrap_or_else(|| panic!("目录中不存在 {name}（颜色约束 {want_red:?}）"));
    idx as engine_native::CardId
}

/// 取一张指定颜色（`Some(true)`=红 / `Some(false)`=黑 / `None`=不限）的牌 id。
pub fn card_id_colored(e: &Engine, name: &str, red: Option<bool>) -> engine_native::CardId {
    card_id(e, name, red)
}

/// 把指定 id 的牌**直接放入手牌**（绕开 `debug_place`）。
pub fn put_hand(e: &mut Engine, player: u8, id: engine_native::CardId) {
    let core = e.state_core_mut();
    core.player_mut(player).hand.retain(|c| *c != id);
    core.player_mut(player).hand.push(id);
}

/// 把指定 id 的牌放入**判定区**。
pub fn put_judge(e: &mut Engine, player: u8, id: engine_native::CardId) {
    let core = e.state_core_mut();
    core.player_mut(player).hand.retain(|c| *c != id);
    core.player_mut(player).judge.push(id);
}

/// 把指定 id 的牌放入**装备区**（空槽前提，调用方自证）。
pub fn put_equips(e: &mut Engine, player: u8, id: engine_native::CardId) {
    let core = e.state_core_mut();
    core.player_mut(player).hand.retain(|c| *c != id);
    core.player_mut(player).equips.push(id);
}

/// 便捷：按颜色取牌并放入手牌，返回 id。
pub fn give_colored(
    e: &mut Engine,
    player: u8,
    name: &str,
    red: Option<bool>,
) -> engine_native::CardId {
    let id = card_id_colored(e, name, red);
    put_hand(e, player, id);
    id
}
