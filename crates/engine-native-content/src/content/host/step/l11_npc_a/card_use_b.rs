//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 用牌与响应类技执行（15 技：`card_use_b.rs`）。

use super::asks::ASK_REZHIJIAN_DRAW_CONFIRM;
use super::asks_b::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::EquipSlot;
use crate::event::EventData;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_card_use_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "bmcanshi" => step_bmcanshi(m, ctx),
        "dcmoukui_dodged" => step_dcmoukui_dodged(m, ctx),
        "gaoyuan" => step_gaoyuan(m, ctx),
        "jili" => step_jili(m, ctx),
        "jinglve" => step_jinglve_trigger(m, ctx),
        "jsrgshichong" => step_jsrgshichong(m, ctx),
        "jsrgzhenqiao" => step_jsrgzhenqiao(m, ctx),
        "kyou_zhidian" => step_kyou_zhidian_aim(m, ctx),
        "nzry_shicai" => step_nzry_shicai(m, ctx),
        "olzhuiji" => step_olzhuiji(m, ctx),
        "reduodao" => step_reduodao(m, ctx),
        "rexingluan" => step_rexingluan(m, ctx),
        "reyajiao" => step_reyajiao(m, ctx),
        "rezhijian" => step_rezhijian_draw(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_bmcanshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let is_cancel = ctx
        .res
        .stack
        .iter()
        .rev()
        .find_map(|f| match &f.data {
            EventData::CardUsed(u) => Some(u.player != owner),
            _ => None,
        })
        .unwrap_or(false);
    match m.stage {
        HostStage::Act => {
            if is_cancel {
                wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_BMCANSHI_CANCEL_CONFIRM)
            } else {
                wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_BMCANSHI_EXTRA_TARGETS)
            }
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) {
                let mut actor = None;
                if let Some(u) = current_use_card_mut(ctx) {
                    actor = Some(u.player);
                    u.targets.retain(|&p| p != owner);
                }
                if let Some(src) = actor {
                    set_flag(ctx.core, src, "l11_kui_mark", 0);
                }
            }
            clear_and_done(m)
        }
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            if !ts.is_empty() {
                if let Some(u) = current_use_card_mut(ctx) {
                    for &t in &ts {
                        if !u.targets.contains(&t) {
                            u.targets.push(t);
                        }
                    }
                }
                for t in ts {
                    set_flag(ctx.core, t, "l11_kui_mark", 0);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_dcmoukui_dodged(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_DCMOUKUI_DODGED_DISCARD)
        }
        HostStage::SelectCard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, owner, &[c]);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_gaoyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GAOYUAN_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_GAOYUAN_CARD)
        }
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            discard_player_cards(ctx, owner, &[c]);
            if let Some(u) = current_use_card_mut(ctx) {
                for slot in &mut u.targets {
                    if *slot == owner {
                        *slot = t;
                        break;
                    }
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_jili(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        if let Some(u) = current_use_card_mut(ctx) {
            if u.targets.is_empty() {
                u.targets.push(u.player);
            }
            if !u.targets.contains(&owner) {
                u.targets.push(owner);
            }
        }
    }
    clear_and_done(m)
}

fn step_jinglve_trigger(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        let mut sishi_actor = None;
        let mut sishi_cid = None;
        if let Some(u) = current_use_card_mut(ctx) {
            u.targets.clear();
            u.cancelled = true;
            sishi_actor = Some(u.player);
            sishi_cid = u.card;
        }
        if let (Some(actor), Some(cid)) = (sishi_actor, sishi_cid) {
            ctx.core
                .vm_aux
                .flags
                .remove(&(actor, format!("l11_jinglve_sishi_card:{cid}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(actor, format!("l11_jinglve_sishi_from:{owner}")));
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "l11_jinglve_active_target".to_string()));
        }
    }
    clear_and_done(m)
}

fn step_jsrgshichong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(t) = m.targets.first().copied() else {
        return clear_and_done(m);
    };
    m.target = Some(t);
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "l11_jsrgshichong_yang") > 0 {
                m.cards.clear();
                wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_JSRGSHICHONG_GIVE)
            } else {
                wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_JSRGSHICHONG_CONFIRM)
            }
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && !ctx.core.player(t).hand.is_empty() {
                set_flag(ctx.core, owner, "l11_jsrgshichong_yang", 1);
                m.cards = vec![1];
                return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_JSRGSHICHONG_STEAL);
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                if m.cards.is_empty() {
                    set_flag(ctx.core, owner, "l11_jsrgshichong_yang", 0);
                }
                move_cards_to_hand(ctx, t, owner, &[c]);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_jsrgzhenqiao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        if let Some(fid) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like() =>
            {
                Some(f.id)
            }
            _ => None,
        }) {
            inc_flag(ctx.core, owner, &format!("card_use_extra_effect:{fid}"), 1);
            set_flag(ctx.core, owner, "l11_zhenqiao_recast_guard", 1);
        }
    }
    clear_and_done(m)
}

pub(super) fn step_kyou_zhidian_aim(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_ZHIDIAN_AIM_EFFECT),
        HostStage::SelectNature => {
            if let Some(tag) = answered_tag(ctx) {
                set_flag(ctx.core, owner, "l11_kyou_zhidian_last", u32::from(tag));
                match tag {
                    1 => set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_unresp", 1),
                    2 => set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_pierce", 1),
                    3 => {
                        set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_dmg", 1);
                        if let Some(&cid) = m.cards.first() {
                            set_flag(ctx.core, owner, &format!("l11_kyou_dmg_card:{cid}"), 1);
                        }
                    }
                    4 => {
                        let extra = flag(ctx.core, owner, "vm_sha_extra_used");
                        if extra > 0 {
                            set_flag(ctx.core, owner, "vm_sha_extra_used", extra - 1);
                        } else {
                            ctx.core.turn.sha_used_play_phase = false;
                        }
                        let cnt = flag(ctx.core, owner, "l11_phase_sha_count");
                        if cnt > 0 {
                            set_flag(ctx.core, owner, "l11_phase_sha_count", cnt - 1);
                        }
                    }
                    _ => {}
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_nzry_shicai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_SHICAI_CONFIRM),
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) {
                if let Some(&cid) = m.cards.first() {
                    if take_from_table_discard_or_equip(ctx, cid) {
                        put_on_top_of_deck(ctx, cid);
                        if ctx.core.player(owner).alive {
                            ctx.start_draw(owner, 1);
                            return clear_and_yield(m, HostStage::DrawSettle);
                        }
                    }
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn olzhuiji_next_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive
            && !he_cards(ctx.core, t).is_empty()
            && ctx.core.computed_distance(ctx.catalog, owner, t) == 1
        {
            m.target = Some(t);
            return wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_OLZHUIJI_MODE);
        }
    }
    clear_and_done(m)
}

fn step_olzhuiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => olzhuiji_next_target(m, ctx),
        HostStage::SelectNature => {
            let Some(t) = m.target else {
                return olzhuiji_next_target(m, ctx);
            };
            let mode = answered_tag(ctx).unwrap_or(0);
            if mode == 1 && !ctx.core.player(t).equips.is_empty() {
                let eqs = ctx.core.player(t).equips.clone();
                let cnt = eqs.len() as u32;
                discard_player_cards(ctx, t, &eqs);
                if ctx.core.player(t).alive && cnt > 0 {
                    ctx.start_draw(t, cnt);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
                return olzhuiji_next_target(m, ctx);
            }
            if !he_cards(ctx.core, t).is_empty() {
                wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_OLZHUIJI_DISCARD)
            } else {
                olzhuiji_next_target(m, ctx)
            }
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                discard_player_cards(ctx, t, &[c]);
            }
            olzhuiji_next_target(m, ctx)
        }
        HostStage::DrawSettle => olzhuiji_next_target(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_reduodao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REDUODAO_DISCARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(src) = m.target else {
                return clear_and_done(m);
            };
            discard_player_cards(ctx, owner, &[c]);
            if let Some(weapon) = ctx
                .core
                .player(src)
                .equips
                .iter()
                .copied()
                .find(|&eq| ctx.catalog.card_slot_of(ctx.core, eq) == Some(EquipSlot::Weapon))
            {
                if ctx.core.player(owner).alive {
                    move_cards_to_hand(ctx, src, owner, &[weapon]);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_rexingluan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let rank = m
        .cards
        .first()
        .and_then(|&cid| ctx.catalog.cards.get(cid as usize))
        .map_or(0, |d| d.rank);
    let has_field_match = ctx.core.alive_players().any(|p| {
        ctx.core
            .player(p)
            .equips
            .iter()
            .chain(ctx.core.player(p).judge.iter())
            .any(|&c| ctx.catalog.cards.get(c as usize).is_some_and(|d| d.rank == rank))
    });
    let has_deck_match = ctx
        .core
        .deck
        .iter()
        .any(|&c| ctx.catalog.cards.get(c as usize).is_some_and(|d| d.rank == rank));
    match m.stage {
        HostStage::Act => {
            if rank == 0 {
                return clear_and_done(m);
            }
            if has_field_match {
                return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REXINGLUAN_TARGET);
            }
            if has_deck_match {
                return wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REXINGLUAN_CONFIRM);
            }
            clear_and_done(m)
        }
        HostStage::TargetChoice => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                m.target = Some(t);
                return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REXINGLUAN_EJ_CARD);
            }
            if has_deck_match {
                return wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REXINGLUAN_CONFIRM);
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
                mark_skill_used(ctx, owner, "rexingluan");
                move_cards_to_hand(ctx, t, owner, &[cid]);
            }
            clear_and_done(m)
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) {
                let matching: Vec<usize> = ctx
                    .core
                    .deck
                    .iter()
                    .enumerate()
                    .filter(|(_, &c)| {
                        ctx.catalog.cards.get(c as usize).is_some_and(|d| d.rank == rank)
                    })
                    .map(|(i, _)| i)
                    .collect();
                if !matching.is_empty() {
                    let pick_pos =
                        matching[ctx.core.rng.gen_range(matching.len() as u32) as usize];
                    let cid = ctx.core.deck.remove(pick_pos);
                    mark_skill_used(ctx, owner, "rexingluan");
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[cid]);
                    ctx.core
                        .player_mut(owner)
                        .hand
                        .extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: delivered,
                        });
                    }
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_reyajiao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REYAJIAO_CONFIRM),
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            let Some(&orig_cid) = m.cards.first() else {
                return clear_and_done(m);
            };
            let orig_cat = card_category(ctx.catalog.resolved_card_kind(ctx.core, orig_cid));
            let Some(top_cid) = pop_deck_reshuffle(ctx) else {
                return clear_and_done(m);
            };
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![top_cid],
                to: ctx.core.alive_players().collect(),
            });
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "l11_reyajiao_peek".to_string()), vec![top_cid]);
            let top_cat = card_category(ctx.catalog.card_kind(top_cid));
            if orig_cat == top_cat {
                wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REYAJIAO_GIVE_TARGET)
            } else {
                wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_REYAJIAO_DIFF_MODE)
            }
        }
        HostStage::TargetChoice => {
            let Some(top_cid) = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "l11_reyajiao_peek".to_string()))
                .and_then(|v| v.into_iter().next())
            else {
                return clear_and_done(m);
            };
            if let Some(t) = answered_players(ctx).into_iter().next() {
                let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, t, &[top_cid]);
                ctx.core
                    .player_mut(t)
                    .hand
                    .extend(delivered.iter().copied());
                if !delivered.is_empty() {
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: t,
                        cards: delivered,
                    });
                }
            } else {
                put_on_top_of_deck(ctx, top_cid);
            }
            clear_and_done(m)
        }
        HostStage::SelectNature => {
            if let Some(top_cid) = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "l11_reyajiao_peek".to_string()))
                .and_then(|v| v.into_iter().next())
            {
                if answered_tag(ctx) == Some(0) {
                    ctx.to_discard(top_cid);
                } else {
                    put_on_top_of_deck(ctx, top_cid);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

pub(super) fn step_rezhijian_draw(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REZHIJIAN_DRAW_CONFIRM)
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}
