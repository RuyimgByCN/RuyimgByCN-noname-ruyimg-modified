//! L11（太虚幻境）Batch L11-3 共用辅助与生命周期钩子（`l11_common_b3.rs`）。
//!
//! 承载：
//! - `on_draw_begin_l11_b3`: `_buff_txhj_xiongziyingfa`（摸牌阶段未固定摸牌数且无 `reyingzi` 时摸牌 +1 并授 `txhj_xiongziyingfa`）
//! - `on_card_use_after_l11_b3`: 用牌结算后清理临时 `txhj_kapaikeshi` 与 `card_nowuxie:*`
//! - `host_judge_settled`: `JudgePhaseMachine` `judgeEnd` 钩子（`yueyingdejiqiao`、`wuweizhile`、`jiezizhidao`）
//! - 牌组配对去重（`buff_txhj_*` vs `_buff_txhj_*`）与随机弃置/牌堆取牌辅助

use super::l11_common::*;
use super::l11_common_b2::is_ordinary_trick_kind;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

/// 若 `name` 为 `buff_txhj_<x>`，返回对应的 `_buff_txhj_<y>` 触发器名（兼容 cuo/chuo 拼写）。
pub(super) fn buff_companion_trigger(name: &str) -> Option<String> {
    match name {
        "buff_txhj_yingjicuoshijia" => Some("_buff_txhj_yingjichuoshijia".to_string()),
        "buff_txhj_yingjicuoshiyi" => Some("_buff_txhj_yingjichuoshiyi".to_string()),
        _ => name.strip_prefix("buff_txhj_").map(|rest| format!("_buff_txhj_{rest}")),
    }
}

/// 当同一角色同时持有 `buff_txhj_X` 与 `_buff_txhj_X` 时，由 `_buff_txhj_X` 单次触发，
/// `buff_txhj_X` 让步避免双倍生效；单独持有任一方时均正常触发。
pub(super) fn buff_yields_to_companion(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> bool {
    buff_companion_trigger(skill_name)
        .is_some_and(|comp| has_named_skill(state, catalog, owner, &comp))
}

pub(super) fn has_weapon_slot(state: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    state
        .player(p)
        .equips
        .iter()
        .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon))
}

pub(super) fn has_armor_slot(state: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    state
        .player(p)
        .equips
        .iter()
        .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor))
}

pub(super) fn sha_use_is_elemental(catalog: &Catalog, u: &CardUseData) -> bool {
    u.nature != DamageNature::Normal
        || matches!(
            u.effective_kind(catalog),
            CardKind::FireSha | CardKind::LeiSha
        )
}

/// 只读读取当前出牌阶段己方已使用「杀」的序号（1 起，由内核 `events.rs:450` 统一计数）。
pub(super) fn play_phase_sha_index(ctx: &Ctx<'_>, owner: PlayerId) -> u32 {
    flag(ctx.core, owner, "l11_phase_sha_count")
}

/// 从牌堆顶向下查找首张满足 `pred` 的牌并置入 `p` 手牌；牌堆未命中时再自弃牌堆查找（对齐 `get.cardPile`）。
pub(super) fn gain_first_from_deck_or_discard(
    ctx: &mut Ctx<'_>,
    p: PlayerId,
    mut pred: impl FnMut(CardKind) -> bool,
) -> Option<CardId> {
    if !ctx.core.player(p).alive {
        return None;
    }
    let cid = if let Some(pos) = ctx
        .core
        .deck
        .iter()
        .rposition(|&c| pred(ctx.catalog.card_kind(c)))
    {
        ctx.core.deck.remove(pos)
    } else {
        let pos = ctx
            .core
            .discard
            .iter()
            .rposition(|&c| pred(ctx.catalog.card_kind(c)))?;
        ctx.core.discard.remove(pos)
    };
    ctx.core.player_mut(p).hand.push(cid);
    ctx.out.push(LogEvent::CardsDrawn {
        player: p,
        cards: vec![cid],
    });
    ctx.emit(EventData::CardsDrawn {
        player: p,
        cards: vec![cid],
    });
    Some(cid)
}

/// 随机弃置 `target` 一张手牌或装备牌（对齐 `target.randomDiscard(true, 'he')`）。
pub(super) fn random_discard_he(ctx: &mut Ctx<'_>, target: PlayerId) -> bool {
    if !ctx.core.player(target).alive {
        return false;
    }
    let he = he_cards(ctx.core, target);
    if he.is_empty() {
        return false;
    }
    let idx = ctx.core.rng.gen_range(he.len() as u32) as usize;
    let c = he[idx];
    discard_player_cards(ctx, target, &[c]);
    true
}

/// `DrawPhase::Begin` 钩子：`_buff_txhj_xiongziyingfa`（`extension_skill.js:1931`）。
pub(super) fn on_draw_begin_l11_b3(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, player) {
        return;
    }
    if (has_named_skill(ctx.core, ctx.catalog, player, "buff_txhj_xiongziyingfa")
        || has_named_skill(ctx.core, ctx.catalog, player, "_buff_txhj_xiongziyingfa"))
        && !has_named_skill(ctx.core, ctx.catalog, player, "reyingzi")
    {
        if !has_named_skill(ctx.core, ctx.catalog, player, "txhj_xiongziyingfa") {
            ensure_perm_skill(ctx, player, "txhj_xiongziyingfa");
        }
        *pending += 1;
    }
}

/// `useCardAfter` 钩子：清理 `_buff_txhj_dongchazhiyan`/`_buff_txhj_mingjingzhiyan`
/// 临时授予的 `txhj_kapaikeshi` 以及 `card_nowuxie:{cid}` 标记。
pub(super) fn on_card_use_after_l11_b3(ctx: &mut Ctx<'_>, u: &CardUseData) {
    if flag(ctx.core, u.player, "l11_temp_kapaikeshi") > 0 {
        set_flag(ctx.core, u.player, "l11_temp_kapaikeshi", 0);
        remove_named_skill(ctx, u.player, "txhj_kapaikeshi");
    }
    let cid = u.card.unwrap_or(0);
    ctx.core
        .vm_aux
        .flags
        .remove(&(u.player, format!("card_nowuxie:{cid}")));
}

/// 判定结算完成钩子（`judgeEnd`）：
/// - `_buff_txhj_yueyingdejiqiao`（`extension_skill.js:2226`）：判定牌仍在弃牌堆（未被天妒等拾取）且有装备 → 摸 1
/// - `_buff_txhj_wuweizhile`（`extension_skill.js:2337`）：他人非红桃乐判定且非己方回合 → 判定者得 `txhj_wuweizhile` 且手牌上限 -1
/// - `_buff_txhj_jiezizhidao`（`extension_skill.js:2389`）：场上非梅花兵粮判定后 → 持有者摸 1
pub(crate) fn host_judge_settled(
    ctx: &mut Ctx<'_>,
    judge_owner: PlayerId,
    trick_kind: Option<CardKind>,
    suit: Suit,
    judge_card: CardId,
) -> bool {
    let mut spawned = false;

    // 28. 月英的机巧
    if is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, judge_owner)
        && ctx.core.discard.contains(&judge_card)
        && !ctx.core.player(judge_owner).equips.is_empty()
        && (has_named_skill(ctx.core, ctx.catalog, judge_owner, "buff_txhj_yueyingdejiqiao")
            || has_named_skill(ctx.core, ctx.catalog, judge_owner, "_buff_txhj_yueyingdejiqiao"))
    {
        ctx.start_draw(judge_owner, 1);
        spawned = true;
    }

    let order = ctx.core.seat_order_from(ctx.core.turn.current);

    // 33. 无为之乐（`locked: false` → 受 `tieji` 封印影响）
    if trick_kind == Some(CardKind::Le) && suit != Suit::Heart {
        for h in order.iter().copied() {
            if h != judge_owner
                && h != ctx.core.turn.current
                && is_unblocked_l11_owner(ctx.core, ctx.catalog, h)
                && (has_named_skill(ctx.core, ctx.catalog, h, "buff_txhj_wuweizhile")
                    || has_named_skill(ctx.core, ctx.catalog, h, "_buff_txhj_wuweizhile"))
            {
                ensure_perm_skill(ctx, judge_owner, "txhj_wuweizhile");
                let prev = flag(ctx.core, judge_owner, "l11_wuweizhile_minus1");
                set_flag(ctx.core, judge_owner, "l11_wuweizhile_minus1", prev + 1);
            }
        }
    }

    // 35. 截辎之道（`locked: false` → 受 `tieji` 封印影响）
    if trick_kind == Some(CardKind::Bing) && suit != Suit::Club {
        for h in order {
            if is_unblocked_l11_owner(ctx.core, ctx.catalog, h)
                && (has_named_skill(ctx.core, ctx.catalog, h, "buff_txhj_jiezizhidao")
                    || has_named_skill(ctx.core, ctx.catalog, h, "_buff_txhj_jiezizhidao"))
            {
                ctx.start_draw(h, 1);
                spawned = true;
            }
        }
    }

    spawned |= super::l11_common_b4::on_judge_settled_l11_b4(
        ctx,
        judge_owner,
        trick_kind,
        suit,
        judge_card,
    );

    let _ = is_ordinary_trick_kind;
    spawned
}
