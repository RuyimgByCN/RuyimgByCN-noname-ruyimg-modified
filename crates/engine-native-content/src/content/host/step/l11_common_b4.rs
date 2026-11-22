//! L11（太虚幻境）Batch L11-4 共用辅助与生命周期钩子（`l11_common_b4.rs`）。
//!
//! 承载：
//! - 批 4 配对去重：`buff_companion_trigger_b4`、`buff_yields_to_companion_b4`（含错拼兼容：`baochao`/`baochang`、`beimin`/`lianmin`、`gusoe`/`guose`）
//! - `on_card_use_after_l11_b4`：`haoshidebaochao`/`baochang`（五谷后摸1）、`mowangdejianiang`（酒后叠层与临时技能）、
//!   `hannu`（酒后牌堆取杀）、`mingyundezhiyi`（闪电后横置未横置敌人）、`hengxingwuji`（过拆后摸1）、以及桃园临时技回收
//! - `on_judge_settled_l11_b4`：`gusoejueyan`/`guosejueyan`（非红桃乐判定摸1）、`huangtianzhimin`（判定牌入弃牌堆摸1）
//! - `sha_damage_used_jiu` / `yuanjiao_draw_bonus`

use super::l11_common::*;
use super::l11_common_b3::gain_first_from_deck_or_discard;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::{CardUseData, EventData, Frame};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::Ctx;
use crate::state::{GameStateCore, SkillInstance};

pub(super) fn is_card_named(catalog: &Catalog, card: Option<CardId>, name: &str) -> bool {
    card.is_some_and(|cid| {
        catalog
            .cards
            .get(cid as usize)
            .is_some_and(|cd| cd.name == name)
    })
}

/// 若 `name` 为批 4 的 `buff_txhj_<x>`，返回对应的 `_buff_txhj_<y>` 触发器名（兼容上游历史拼写差异）。
pub(super) fn buff_companion_trigger_b4(name: &str) -> Option<String> {
    match name {
        "buff_txhj_haoshidebaochao" => Some("_buff_txhj_haoshidebaochang".to_string()),
        "buff_txhj_xiuluodebeimin" => Some("_buff_txhj_xiuluodelianmin".to_string()),
        "buff_txhj_gusoejueyan" => Some("_buff_txhj_guosejueyan".to_string()),
        _ => name.strip_prefix("buff_txhj_").map(|rest| format!("_buff_txhj_{rest}")),
    }
}

/// 当同一角色同时持有 `buff_txhj_X` 与 `_buff_txhj_Y` 时，由 `_buff_txhj_Y` 单次触发，
/// `buff_txhj_X` 让步避免双倍生效；单独持有任一方时均正常触发。
pub(super) fn buff_yields_to_companion_b4(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> bool {
    buff_companion_trigger_b4(skill_name)
        .is_some_and(|comp| has_named_skill(state, catalog, owner, &comp))
}

/// 沿当前事件帧向上查找引发本次【杀】伤害的父 `CardUsed` 帧号（`sha_used_jiu:{fid}` 匹配用）。
pub(super) fn parent_sha_card_frame(
    frames: &[Frame],
    frame: Option<&Frame>,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<EventId> {
    let mut cur = frame?;
    if let EventData::DamageWindow { root, .. } = cur.data {
        cur = frames.iter().find(|r| r.id == root)?;
    }
    let mut pid = cur.parent;
    while let Some(id) = pid {
        let f = frames.iter().find(|r| r.id == id)?;
        if let EventData::CardUsed(u) = &f.data {
            if u.player == owner && u.effective_kind(catalog).is_sha_like() {
                return Some(f.id);
            }
        }
        pid = f.parent;
    }
    None
}

/// 判定本次【杀】伤害是否受【酒】本杀加成（含 `turn.jiu_active` 与 `sha_used_jiu:{sha_fid}`）。
pub(super) fn sha_damage_used_jiu(
    state: &GameStateCore,
    frames: &[Frame],
    frame: Option<&Frame>,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    state.turn.jiu_active
        || parent_sha_card_frame(frames, frame, catalog, owner)
            .is_some_and(|fid| flag(state, owner, &format!("sha_used_jiu:{fid}")) > 0)
}

/// `buff_txhj_ximengzhili` / `_buff_txhj_ximengzhili`（`extension_skill.js:3092`）：
/// 使用【远交近攻】为自己摸牌时额外 +1。
pub(super) fn yuanjiao_draw_bonus(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> u32 {
    let active = is_locked_unblocked_l11_owner(state, catalog, owner)
        && (has_named_skill(state, catalog, owner, "buff_txhj_ximengzhili")
            || has_named_skill(state, catalog, owner, "_buff_txhj_ximengzhili"));
    u32::from(active)
}

fn ensure_turn_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    ctx.ensure_skill(
        (p, def),
        SkillInstance {
            owner: p,
            enabled: true,
            once_used: false,
            expires_end_of_turn: true,
            rounds: 0,
        },
    );
}

/// `useCardAfter` 生命周期钩子（由 `u08_batch_d_hooks::host_card_use_after_impl` 统一调用）。
pub(super) fn on_card_use_after_l11_b4(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let eff_kind = u.effective_kind(ctx.catalog);

    if eff_kind == CardKind::Taoyuan {
        for p in 0..ctx.core.players.len() as PlayerId {
            if flag(ctx.core, p, "l11_temp_taoyuanyishi") > 0 {
                set_flag(ctx.core, p, "l11_temp_taoyuanyishi", 0);
                remove_named_skill(ctx, p, "txhj_taoyuanyishi");
            }
            if flag(ctx.core, p, "l11_temp_taoyuanyishi_x") > 0 {
                set_flag(ctx.core, p, "l11_temp_taoyuanyishi_x", 0);
                remove_named_skill(ctx, p, "taoyuanyishiX");
            }
        }
    }

    if u.cancelled || !is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, u.player) {
        return;
    }

    match eff_kind {
        CardKind::Wugu => {
            if has_named_skill(ctx.core, ctx.catalog, u.player, "buff_txhj_haoshidebaochao")
                || has_named_skill(ctx.core, ctx.catalog, u.player, "_buff_txhj_haoshidebaochang")
            {
                ctx.start_draw(u.player, 1);
            }
        }
        CardKind::Jiu => {
            if has_named_skill(ctx.core, ctx.catalog, u.player, "buff_txhj_mowangdejianiang")
                || has_named_skill(ctx.core, ctx.catalog, u.player, "_buff_txhj_mowangdejianiang")
            {
                ensure_turn_skill(ctx, u.player, "txhj_mowangdejianiang");
                ensure_turn_skill(ctx, u.player, "txhj_jianiang_clear");
                let prev = flag(ctx.core, u.player, "l11_jianiang_stacks");
                set_flag(ctx.core, u.player, "l11_jianiang_stacks", prev + 1);
            }
            if has_named_skill(ctx.core, ctx.catalog, u.player, "buff_txhj_hannu")
                || has_named_skill(ctx.core, ctx.catalog, u.player, "_buff_txhj_hannu")
            {
                let _ = gain_first_from_deck_or_discard(ctx, u.player, CardKind::is_sha_like);
            }
        }
        CardKind::ShanDian => {
            if has_named_skill(ctx.core, ctx.catalog, u.player, "buff_txhj_mingyundezhiyi")
                || has_named_skill(ctx.core, ctx.catalog, u.player, "_buff_txhj_mingyundezhiyi")
            {
                let order = alive_others(ctx.core, u.player);
                for t in order {
                    if !ctx.core.is_friend_of(ctx.catalog, u.player, t) && !ctx.core.player(t).linked
                    {
                        ctx.emit(EventData::Linked {
                            player: t,
                            linked: true,
                        });
                    }
                }
            }
        }
        CardKind::Dismantle => {
            if !is_card_named(ctx.catalog, u.card, "txhj_caomu")
                && (has_named_skill(ctx.core, ctx.catalog, u.player, "buff_txhj_hengxingwuji")
                    || has_named_skill(ctx.core, ctx.catalog, u.player, "_buff_txhj_hengxingwuji"))
            {
                ctx.start_draw(u.player, 1);
            }
        }
        _ => {}
    }
}

/// `judgeEnd` 钩子扩展（由 `l11_common_b3::host_judge_settled` 转发调用）：
/// - `buff_txhj_gusoejueyan` / `_buff_txhj_guosejueyan`（`locked: false`：`extension_skill.js:3035`）
///   非红桃【乐不思蜀】判定结算后持有者摸 1。
/// - `buff_txhj_huangtianzhimin` / `_buff_txhj_huangtianzhimin`（`forced: true`：`extension_skill.js:3260`）
///   判定牌进入弃牌堆（仍在弃牌区未被他技拾取）后持有者摸 1。
pub(super) fn on_judge_settled_l11_b4(
    ctx: &mut Ctx<'_>,
    judge_owner: PlayerId,
    trick_kind: Option<CardKind>,
    suit: Suit,
    judge_card: CardId,
) -> bool {
    let mut spawned = false;

    if trick_kind == Some(CardKind::Le) && suit != Suit::Heart {
        for h in ctx.core.seat_order_from(ctx.core.turn.current) {
            if is_unblocked_l11_owner(ctx.core, ctx.catalog, h)
                && (has_named_skill(ctx.core, ctx.catalog, h, "buff_txhj_gusoejueyan")
                    || has_named_skill(ctx.core, ctx.catalog, h, "_buff_txhj_guosejueyan"))
            {
                ctx.start_draw(h, 1);
                spawned = true;
            }
        }
    }

    if ctx.core.discard.contains(&judge_card)
        && is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, judge_owner)
        && (has_named_skill(ctx.core, ctx.catalog, judge_owner, "buff_txhj_huangtianzhimin")
            || has_named_skill(ctx.core, ctx.catalog, judge_owner, "_buff_txhj_huangtianzhimin"))
    {
        ctx.start_draw(judge_owner, 1);
        spawned = true;
    }

    spawned
}

/// `wushuangfangtianji_skill` 弃牌后的 `shenhanxueyin` 盗牌与 `fuchendangmo` 追加询问判定。
pub(in crate::content::host) fn post_wushuang_discard_l11_b4(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    discarded: CardId,
    first_pass: bool,
) -> bool {
    if crate::rvm_bridge::modifiers::has_shenhanxueyin(ctx.core, ctx.catalog, owner)
        && ctx.core.player(owner).alive
        && ctx.core.discard.contains(&discarded)
    {
        gain_card_from_deck_or_discard(ctx, owner, discarded);
    }
    first_pass
        && crate::rvm_bridge::modifiers::has_fuchendangmo(ctx.core, ctx.catalog, owner)
        && ctx.core.player(target).alive
        && !he_cards(ctx.core, target).is_empty()
}
