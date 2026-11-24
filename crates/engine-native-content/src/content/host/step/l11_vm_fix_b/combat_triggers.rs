//! L11-E2 `taixu` VM Fix B 战斗 / 伤害 / 用牌触发技能
//! （`qingxi` / `rebeige` / `rejianxiong` / `rejieming` / `releiji` / `reshejian` /
//! `reweimu` / `spchijie` / `spdengli` / `sphuangen` / `xinliegong` / `yiyong` / `zhuandui`）。

use super::super::super::*;
use super::super::l11_common::*;
use super::super::l11_common_b2::deliver_direct_cards_to_hand;
use super::asks::{
    decode_hidden_or_cards, wait_vm_b_ask, weapon_range_sum, ASK_QINGXI_DISCARD,
    ASK_REBEIGE_DISCARD, ASK_REBEIGE_SOURCE_DISCARD, ASK_REJIEMING_TARGET, ASK_RELEIJI_TARGET,
    ASK_RESHEJIAN_DISCARD, ASK_RESHEJIAN_MODE, ASK_RESHEJIAN_PICK, ASK_SPHUANGEN_TARGETS,
    ASK_ZHUANDUI_CONFIRM, STAGE_REBEIGE_SETTLE, STAGE_RELEIJI_RECOVER, STAGE_SPCHIJIE_SETTLE,
    STAGE_ZHUANDUI_PINDIAN,
};
use super::pindian::{can_compare_pindian, step_pindian, PindianStep};
use crate::catalog::{CardKind, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature};
use crate::ids::CardId;
use crate::machine::{Ctx, Flow};

pub(super) fn handle_qingxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let w_range = weapon_range_sum(ctx.core, ctx.catalog, owner);
            if w_range == 0 {
                return Flow::Done;
            }
            if ctx.core.player(t).hand.len() < w_range {
                with_current_damage_mut(ctx, |d| d.amount += 1);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_QINGXI_DISCARD)
        }
        (HostStage::SelectCard, Some(ASK_QINGXI_DISCARD)) => {
            m.card = None;
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let chosen = answered_cards(ctx);
            if chosen.is_empty() {
                with_current_damage_mut(ctx, |d| d.amount += 1);
            } else {
                discard_player_cards(ctx, t, &chosen);
                let weps: Vec<CardId> = ctx
                    .core
                    .player(owner)
                    .equips
                    .iter()
                    .copied()
                    .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon))
                    .collect();
                if !weps.is_empty() {
                    discard_player_cards(ctx, owner, &weps);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_rebeige(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_REBEIGE_DISCARD),
        (HostStage::SelectCard, Some(ASK_REBEIGE_DISCARD)) => {
            m.card = None;
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(victim) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            discard_player_cards(ctx, owner, &[cid]);
            m.stage = HostStage::Act;
            m.card = Some(STAGE_REBEIGE_SETTLE);
            ctx.emit(EventData::OrdinaryJudge {
                owner: victim,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        (HostStage::Act, Some(STAGE_REBEIGE_SETTLE)) => {
            m.card = None;
            let Some(victim) = m.target else {
                return Flow::Done;
            };
            let src = m.targets.first().copied();
            let dmg_amt = m.cards.first().copied().unwrap_or(1).max(1);
            let suit_u32 = ctx.core.vm_aux.flags.remove(&(victim, "last_judge_suit".to_string()));
            ctx.core.vm_aux.flags.remove(&(victim, "last_judge_rank".to_string()));
            match suit_u32 {
                Some(2) => {
                    // Heart
                    if ctx.core.player(victim).alive {
                        ctx.recover(victim, dmg_amt, None);
                    }
                    Flow::Done
                }
                Some(4) => {
                    // Diamond
                    if ctx.core.player(victim).alive {
                        ctx.start_draw(victim, 3);
                    }
                    Flow::Done
                }
                Some(3) => {
                    // Club
                    if src.is_some_and(|p| ctx.core.player(p).alive && !he_cards(ctx.core, p).is_empty()) {
                        return wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_REBEIGE_SOURCE_DISCARD);
                    }
                    Flow::Done
                }
                Some(1) => {
                    // Spade
                    if let Some(s) = src.filter(|&p| ctx.core.player(p).alive) {
                        ctx.turn_over(s);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        (HostStage::CardsAsk, Some(ASK_REBEIGE_SOURCE_DISCARD)) => {
            m.card = None;
            if let Some(s) = m.targets.first().copied() {
                let picked = answered_cards(ctx);
                if !picked.is_empty() && ctx.core.player(s).alive {
                    discard_player_cards(ctx, s, &picked);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_rejianxiong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(&cid) = m.cards.first() {
        let from_table = ctx.core.table.iter().position(|&x| x == cid);
        let from_discard = ctx.core.discard.iter().position(|&x| x == cid);
        if let Some(pos) = from_table {
            ctx.core.table.remove(pos);
            deliver_direct_cards_to_hand(ctx, owner, &[cid]);
            ctx.start_draw(owner, 1);
        } else if let Some(pos) = from_discard {
            ctx.core.discard.remove(pos);
            deliver_direct_cards_to_hand(ctx, owner, &[cid]);
            ctx.start_draw(owner, 1);
        }
    }
    Flow::Done
}

pub(super) fn handle_rejieming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REJIEMING_TARGET),
        (HostStage::TargetChoice, Some(ASK_REJIEMING_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(t, 2);
            Flow::Yield
        }
        (HostStage::DrawSettle, _) => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            m.target = None;
            if ctx.core.player(owner).alive
                && ctx.core.player(t).alive
                && ctx.core.player(t).hand.len() < ctx.core.player(t).max_hp as usize
            {
                m.stage = HostStage::Act;
                ctx.start_draw(owner, 1);
                if let Some(rem) = m.cards.first_mut() {
                    *rem = rem.saturating_sub(1);
                    if *rem > 0
                        && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
                        && has_named_skill(ctx.core, ctx.catalog, owner, "rejieming")
                    {
                        return Flow::Yield;
                    }
                }
                return Flow::Done;
            }
            if let Some(rem) = m.cards.first_mut() {
                *rem = rem.saturating_sub(1);
                if *rem > 0
                    && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
                    && has_named_skill(ctx.core, ctx.catalog, owner, "rejieming")
                {
                    m.stage = HostStage::Act;
                    return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REJIEMING_TARGET);
                }
            }
            Flow::Done
        }
        (HostStage::Act, _) => {
            let rem = m.cards.first().copied().unwrap_or(0);
            if rem > 0
                && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
                && has_named_skill(ctx.core, ctx.catalog, owner, "rejieming")
            {
                wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REJIEMING_TARGET)
            } else {
                Flow::Done
            }
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_releiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_RELEIJI_TARGET),
        (HostStage::TargetChoice, Some(ASK_RELEIJI_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner: t,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        (HostStage::DrawSettle, _) => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let suit = ctx.core.vm_aux.flags.remove(&(t, "last_judge_suit".to_string()));
            ctx.core.vm_aux.flags.remove(&(t, "last_judge_rank".to_string()));
            match suit {
                Some(1) => {
                    if ctx.core.player(t).alive {
                        emit_damage(ctx, Some(owner), t, 2, DamageNature::Thunder, None);
                    }
                    Flow::Done
                }
                Some(3) => {
                    if ctx.core.player(t).alive {
                        m.stage = HostStage::Act;
                        m.card = Some(STAGE_RELEIJI_RECOVER);
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
                        Flow::Yield
                    } else {
                        if ctx.core.player(owner).alive {
                            ctx.recover(owner, 1, None);
                        }
                        Flow::Done
                    }
                }
                _ => Flow::Done,
            }
        }
        (HostStage::Act, Some(STAGE_RELEIJI_RECOVER)) => {
            m.card = None;
            if ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_reshejian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_RESHEJIAN_DISCARD),
        (HostStage::CardsAsk, Some(ASK_RESHEJIAN_DISCARD)) => {
            m.card = None;
            let chosen = answered_cards(ctx);
            if chosen.len() < 2 {
                return Flow::Done;
            }
            let ts = turn_stamp(ctx.core);
            let cur_ts = flag(ctx.core, owner, "l11_vm_b:reshejian_ts");
            let cnt = if cur_ts == ts {
                flag(ctx.core, owner, "l11_vm_b:reshejian_cnt")
            } else {
                0
            };
            if cnt >= 2 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11_vm_b:reshejian_ts", ts);
            set_flag(ctx.core, owner, "l11_vm_b:reshejian_cnt", cnt + 1);
            discard_player_cards(ctx, owner, &chosen);
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            m.cards = vec![chosen.len() as u32];
            if he_cards(ctx.core, t).is_empty() {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_RESHEJIAN_MODE)
        }
        (HostStage::SelectNature, Some(ASK_RESHEJIAN_MODE)) => {
            m.card = None;
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if mode == 0 {
                if he_cards(ctx.core, t).is_empty() {
                    return Flow::Done;
                }
                wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_RESHEJIAN_PICK)
            } else {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                Flow::Done
            }
        }
        (HostStage::SelectCard, Some(ASK_RESHEJIAN_PICK)) => {
            m.card = None;
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let picked = decode_hidden_or_cards(ctx.core, t, ctx.answer());
                if !picked.is_empty() {
                    discard_player_cards(ctx, t, &picked);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_reweimu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let amt = m.cards.first().copied().unwrap_or(0);
    with_current_damage_mut(ctx, |d| d.prevented = true);
    if amt > 0 {
        ctx.start_draw(owner, amt.saturating_mul(2));
    }
    Flow::Done
}

pub(super) fn handle_spchijie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => {
            let ts = turn_stamp(ctx.core);
            set_flag(ctx.core, owner, "l11_vm_b:spchijie_used_ts", ts);
            m.card = Some(STAGE_SPCHIJIE_SETTLE);
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        (HostStage::Act, Some(STAGE_SPCHIJIE_SETTLE)) => {
            m.card = None;
            ctx.core.vm_aux.flags.remove(&(owner, "last_judge_suit".to_string()));
            let rank = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "last_judge_rank".to_string()))
                .unwrap_or(0);
            if rank > 6 {
                if let Some(u) = current_use_card_mut(ctx) {
                    u.targets.retain(|&p| p != owner);
                    if u.targets.is_empty() {
                        u.cancelled = true;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_spdengli(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let count = m.cards.first().copied().unwrap_or(1).max(1);
    ctx.start_draw(owner, count);
    Flow::Done
}

pub(super) fn handle_sphuangen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_SPHUANGEN_TARGETS),
        (HostStage::TargetChoice, Some(ASK_SPHUANGEN_TARGETS)) => {
            m.card = None;
            let chosen = answered_players(ctx);
            if chosen.is_empty() {
                return Flow::Done;
            }
            if let Some(u) = current_use_card_mut(ctx) {
                u.targets.retain(|p| !chosen.contains(p));
                if u.targets.is_empty() {
                    u.cancelled = true;
                }
            }
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_xinliegong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let owner_hand = ctx.core.player(owner).hand.len();
    let owner_hp = ctx.core.player(owner).hp;
    let Some(u) = current_use_card_mut(ctx) else {
        return Flow::Done;
    };
    let cid = u.card.unwrap_or(0);
    let targets = u.targets.clone();
    for t in targets {
        if !ctx.core.player(t).alive {
            continue;
        }
        if ctx.core.player(t).hand.len() <= owner_hand {
            set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
        }
        if ctx.core.player(t).hp >= owner_hp {
            let k = format!("sha_target_damage_bonus:{cid}:{t}");
            let cur = flag(ctx.core, owner, &k);
            set_flag(ctx.core, owner, &k, cur + 1);
        }
    }
    Flow::Done
}

pub(super) fn handle_yiyong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(&dmg_cid) = m.cards.first() else {
        return Flow::Done;
    };
    let Some(src) = m.target.filter(|&p| ctx.core.player(p).alive) else {
        return Flow::Done;
    };
    let from_table = ctx.core.table.iter().position(|&x| x == dmg_cid);
    let from_discard = ctx.core.discard.iter().position(|&x| x == dmg_cid);
    if let Some(pos) = from_table {
        ctx.core.table.remove(pos);
        deliver_direct_cards_to_hand(ctx, owner, &[dmg_cid]);
    } else if let Some(pos) = from_discard {
        ctx.core.discard.remove(pos);
        deliver_direct_cards_to_hand(ctx, owner, &[dmg_cid]);
    } else {
        return Flow::Done;
    }
    if ctx.core.player(src).alive
        && ctx.core.player(owner).hand.contains(&dmg_cid)
        && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
            ctx.core,
            ctx.catalog,
            owner,
            dmg_cid,
            CardKind::Sha,
            DamageNature::Normal,
        )
        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
            ctx.core,
            ctx.catalog,
            owner,
            src,
            dmg_cid,
            CardKind::Sha,
            DamageNature::Normal,
        )
    {
        let src_has_weapon = ctx
            .core
            .player(src)
            .equips
            .iter()
            .any(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon));
        if !src_has_weapon {
            let k = format!("sha_target_damage_bonus:{dmg_cid}:{src}");
            let cur = flag(ctx.core, owner, &k);
            set_flag(ctx.core, owner, &k, cur + 1);
        }
        ctx.core.player_mut(owner).hand.retain(|&x| x != dmg_cid);
        ctx.core.table.push(dmg_cid);
        ctx.note_cards_lost(owner, &[dmg_cid]);
        (ctx.catalog.hooks.on_hand_card_removed)(ctx, owner, dmg_cid);
        ctx.emit(EventData::CardUsed(CardUseData {
            card: Some(dmg_cid),
            player: owner,
            targets: vec![src],
            nature: DamageNature::Normal,
            use_as: Some(CardKind::Sha),
            cancelled: false,
        }));
    }
    Flow::Done
}

pub(super) fn handle_zhuandui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(STAGE_ZHUANDUI_PINDIAN)
        || matches!(
            m.stage,
            HostStage::SelectCard
                | HostStage::ComparePlayerPick
                | HostStage::CompareTargetPick
                | HostStage::SelectNature
        )
    {
        let Some(opp) = m.target else {
            return Flow::Done;
        };
        match step_pindian(m, ctx, owner, opp) {
            (PindianStep::Pending, flow) => return flow,
            (PindianStep::Aborted, _) => {
                m.target = None;
                return zhuandui_advance(m, ctx);
            }
            (PindianStep::Resolved { win, .. }, _) => {
                let is_attacker = m.cards.first().copied() == Some(1);
                if win {
                    if let Some(u) = current_use_card_mut(ctx) {
                        if is_attacker {
                            let cid = u.card.unwrap_or(0);
                            set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{opp}"), 1);
                        } else {
                            u.targets.retain(|&p| p != owner);
                            if u.targets.is_empty() {
                                u.cancelled = true;
                            }
                        }
                    }
                }
                m.target = None;
                return zhuandui_advance(m, ctx);
            }
        }
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_ZHUANDUI_CONFIRM) {
        m.card = None;
        m.stage = HostStage::Act;
        if !is_confirmed(ctx) {
            m.target = None;
            return zhuandui_advance(m, ctx);
        }
        let Some(opp) = m.target else {
            return zhuandui_advance(m, ctx);
        };
        if !can_compare_pindian(ctx.core, ctx.catalog, owner)
            || !can_compare_pindian(ctx.core, ctx.catalog, opp)
        {
            m.target = None;
            return zhuandui_advance(m, ctx);
        }
        match step_pindian(m, ctx, owner, opp) {
            (PindianStep::Pending, flow) => return flow,
            (PindianStep::Aborted, _) => {
                m.target = None;
                return zhuandui_advance(m, ctx);
            }
            (PindianStep::Resolved { win, .. }, _) => {
                let is_attacker = m.cards.first().copied() == Some(1);
                if win {
                    if let Some(u) = current_use_card_mut(ctx) {
                        if is_attacker {
                            let cid = u.card.unwrap_or(0);
                            set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{opp}"), 1);
                        } else {
                            u.targets.retain(|&p| p != owner);
                            if u.targets.is_empty() {
                                u.cancelled = true;
                            }
                        }
                    }
                }
                m.target = None;
                return zhuandui_advance(m, ctx);
            }
        }
    }
    zhuandui_advance(m, ctx)
}

fn zhuandui_advance(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive || !can_compare_pindian(ctx.core, ctx.catalog, owner) {
        return Flow::Done;
    }
    while let Some(opp) = m.targets.first().copied() {
        m.targets.remove(0);
        if !ctx.core.player(opp).alive || !can_compare_pindian(ctx.core, ctx.catalog, opp) {
            continue;
        }
        m.target = Some(opp);
        return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUANDUI_CONFIRM);
    }
    Flow::Done
}
