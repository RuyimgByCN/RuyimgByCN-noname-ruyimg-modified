//! L10 史诗卡牌十常侍伴生技（后半部分主动技：`scspicai`, `scsyaozhuo`, `scsxiaolu`, `scskuiji`, `scsanruo`）。

use super::*;
use crate::catalog::{CardKind, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn run_picai_judge(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    m.card = None;
    m.stage = HostStage::DrawSettle;
    ctx.emit(EventData::OrdinaryJudge {
        owner: m.owner,
        invocation: u32::MAX,
    });
    Flow::Yield
}

pub(super) fn step_picai_judge_settled(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let code = ctx
        .core
        .vm_aux
        .flags
        .remove(&(m.owner, "last_judge_suit".to_string()))
        .unwrap_or(0);
    let _ = ctx
        .core
        .vm_aux
        .flags
        .remove(&(m.owner, "last_judge_rank".to_string()));
    let Some(judged) = ctx
        .core
        .vm_aux
        .flags
        .remove(&(m.owner, "last_judge_card".to_string()))
    else {
        return ask_picai_target(m, ctx);
    };
    let j_suit = match code {
        1 => Suit::Spade,
        2 => Suit::Heart,
        3 => Suit::Club,
        4 => Suit::Diamond,
        _ => ctx.catalog.effective_card_suit(ctx.core, m.owner, judged),
    };
    let bit = suit_bit(j_suit);
    let mut seen_mask = ctx
        .core
        .vm_aux
        .flags
        .get(&(m.owner, "l10_scspicai_suits".to_string()))
        .copied()
        .unwrap_or(0);
    m.cards.push(judged);
    if (seen_mask & bit) == 0 {
        seen_mask |= bit;
        ctx.core
            .vm_aux
            .flags
            .insert((m.owner, "l10_scspicai_suits".to_string()), seen_mask);
        m.card = Some(ASK_SCSPICAI_CONTINUE);
        m.stage = HostStage::SelectNature;
        return Flow::WaitChoice {
            observer: m.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "scspicai_continue",
                options: vec![ChoiceOption::Confirm],
                min: 0,
                max: 1,
                allow_cancel: true,
            },
        };
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(m.owner, "l10_scspicai_suits".to_string()));
    m.cards.retain(|c| ctx.core.discard.contains(c));
    ask_picai_target(m, ctx)
}

pub(super) fn step_scspicai_continue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
        return run_picai_judge(m, ctx);
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(m.owner, "l10_scspicai_suits".to_string()));
    m.cards.retain(|c| ctx.core.discard.contains(c));
    ask_picai_target(m, ctx)
}

fn ask_picai_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.cards.is_empty() {
        return Flow::Done;
    }
    m.card = Some(ASK_SCSPICAI_TARGET);
    m.stage = HostStage::TargetChoice;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scspicai_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cards = std::mem::take(&mut m.cards);
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        for c in cards {
            ctx.core.table.retain(|&x| x != c);
            ctx.to_discard(c);
        }
        return Flow::Done;
    };
    let mut given = Vec::new();
    for &c in &cards {
        if ctx.core.discard.contains(&c) {
            ctx.core.discard.retain(|&x| x != c);
            ctx.core.player_mut(t).hand.push(c);
            given.push(c);
        } else if ctx.core.table.contains(&c) {
            ctx.core.table.retain(|&x| x != c);
            ctx.core.player_mut(t).hand.push(c);
            given.push(c);
        }
    }
    if !given.is_empty() {
        ctx.log(LogEvent::CardsGiven {
            from: m.owner,
            to: t,
            cards: given,
        });
    }
    Flow::Done
}

pub(super) fn step_scsyaozhuo_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if ctx.core.player(m.owner).hand.is_empty() || ctx.core.player(t).hand.is_empty() {
        return Flow::Done;
    }
    consume_once(m, ctx);
    m.target = Some(t);
    m.card = Some(ASK_SCSYAOZHUO_OWN_CARD);
    m.stage = HostStage::ComparePlayerPick;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsyaozhuo_own(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    m.cards = vec![c];
    m.card = Some(ASK_SCSYAOZHUO_TGT_CARD);
    m.stage = HostStage::CompareTargetPick;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsyaozhuo_tgt(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let Some(own_c) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let Some(ChoiceOption::Card(tgt_c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if ctx.core.player(m.owner).hand.contains(&own_c) {
        ctx.discard_from_hand(m.owner, own_c);
    }
    if ctx.core.player(t).hand.contains(&tgt_c) {
        ctx.discard_from_hand(t, tgt_c);
    }
    let own_rank = ctx.catalog.cards[own_c as usize].rank;
    let tgt_rank = ctx.catalog.cards[tgt_c as usize].rank;
    if own_rank > tgt_rank {
        ctx.core.player_mut(t).skip_next_draw = true;
        return Flow::Done;
    }
    let pool: Vec<CardId> = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .chain(ctx.core.player(m.owner).equips.iter())
        .copied()
        .collect();
    if pool.is_empty() {
        return Flow::Done;
    }
    if pool.len() <= 2 {
        discard_owner_he(ctx, m.owner, &pool);
        return Flow::Done;
    }
    m.card = Some(ASK_SCSYAOZHUO_DISCARD);
    m.stage = HostStage::SelectCard;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsyaozhuo_discard(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cards: Vec<CardId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    discard_owner_he(ctx, m.owner, &cards);
    Flow::Done
}

pub(super) fn after_xiaolu_draw(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let pool: Vec<CardId> = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .chain(ctx.core.player(m.owner).equips.iter())
        .copied()
        .collect();
    if pool.is_empty() {
        return Flow::Done;
    }
    if pool.len() < 2 {
        discard_owner_he(ctx, m.owner, &pool);
        return Flow::Done;
    }
    m.card = Some(ASK_SCSXIAOLU_MODE);
    m.stage = HostStage::SelectNature;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsxiaolu_mode(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let mode = match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(tag)) => u32::from(tag),
        _ => return Flow::Done,
    };
    m.cards = vec![mode];
    m.card = Some(ASK_SCSXIAOLU_CARDS);
    m.stage = HostStage::CardsAsk;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsxiaolu_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cards: Vec<CardId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    if cards.is_empty() {
        return Flow::Done;
    }
    if m.cards.first().copied() == Some(0) {
        m.cards = cards;
        m.card = Some(ASK_SCSXIAOLU_TARGET);
        m.stage = HostStage::TargetChoice;
        return wait_l10_scs(m, ctx);
    }
    discard_owner_he(ctx, m.owner, &cards);
    Flow::Done
}

pub(super) fn step_scsxiaolu_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let cards = std::mem::take(&mut m.cards);
    let mut moved = Vec::new();
    let mut origins = Vec::new();
    for c in cards {
        if ctx.core.player(m.owner).hand.contains(&c) {
            ctx.take_hand_card(m.owner, c);
            ctx.core.player_mut(t).hand.push(c);
            moved.push(c);
            origins.push(crate::event::LostOrigin::Hand);
        } else if ctx.core.player(m.owner).equips.contains(&c) {
            ctx.core.player_mut(m.owner).equips.retain(|&x| x != c);
            ctx.core.declared_slots.remove(&c);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, m.owner, c);
            ctx.core.player_mut(t).hand.push(c);
            moved.push(c);
            origins.push(crate::event::LostOrigin::Equip);
        }
    }
    if !moved.is_empty() {
        ctx.note_cards_lost(m.owner, &moved);
        ctx.log(LogEvent::CardsGiven {
            from: m.owner,
            to: t,
            cards: moved.clone(),
        });
        ctx.emit(EventData::CardsLost {
            player: m.owner,
            cards: moved.clone(),
            origins,
        });
        ctx.emit(EventData::CardsMoved {
            cards: moved,
            from: m.owner,
            to: t,
        });
    }
    Flow::Done
}

pub(super) fn step_scskuiji_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    consume_once(m, ctx);
    m.target = Some(t);
    m.cards.clear();
    m.card = Some(ASK_SCSKUIJI_CARDS);
    m.stage = HostStage::SelectCard;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scskuiji_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let chosen: Vec<CardId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    if chosen.len() != 4 {
        return Flow::Done;
    }
    let mut suit_mask = 0u32;
    for &c in &chosen {
        let holder = if ctx.core.player(m.owner).hand.contains(&c) {
            m.owner
        } else if ctx.core.player(t).hand.contains(&c) {
            t
        } else {
            return Flow::Done;
        };
        suit_mask |= suit_bit(ctx.catalog.effective_card_suit(ctx.core, holder, c));
    }
    if suit_mask.count_ones() != 4 {
        return Flow::Done;
    }
    for c in chosen {
        if ctx.core.player(m.owner).hand.contains(&c) {
            ctx.discard_from_hand(m.owner, c);
        } else if ctx.core.player(t).hand.contains(&c) {
            ctx.discard_from_hand(t, c);
        }
    }
    Flow::Done
}

pub(super) fn step_scsanruo_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(m.owner).hand.contains(&c) && !ctx.core.player(m.owner).equips.contains(&c) {
        return Flow::Done;
    }
    let resp_ctx = m.cards.first().copied().unwrap_or(0);
    let steal_target = m.cards.get(1).copied().unwrap_or(0) as PlayerId;
    let suit = ctx.catalog.effective_card_suit(ctx.core, m.owner, c);
    if resp_ctx != 0 {
        let suit_ok = match resp_ctx {
            1 => suit == Suit::Club,
            2 => suit == Suit::Diamond,
            3 => suit == Suit::Spade,
            4 | 5 => suit == Suit::Heart,
            _ => false,
        };
        if !suit_ok {
            return Flow::Done;
        }
        ctx.core
            .vm_aux
            .flags
            .insert((m.owner, "l10_reguhuo_resp_done".to_string()), 1);
        ctx.core
            .vm_aux
            .flags
            .insert((m.owner, "l10_reguhuo_resp_card".to_string()), c);
        if resp_ctx == 4 || resp_ctx == 5 {
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "l10_reguhuo_resp_kind".to_string()), 4);
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, format!("l10_scsanruo_effect:{c}")), 1);
        } else {
            ctx.core.vm_aux.flags.insert(
                (m.owner, format!("l10_scsanruo_effect:{c}")),
                if resp_ctx == 3 { 3 } else { 2 },
            );
            ctx.core.vm_aux.flags.insert(
                (m.owner, format!("l10_scsanruo_steal_target:{c}")),
                u32::from(steal_target) + 1,
            );
        }
        return Flow::Done;
    }
    match suit {
        Suit::Heart => {
            if crate::kernel::tao_futai2_banned(ctx.core, ctx.catalog, m.owner)
                || ctx.core.player(m.owner).hp >= ctx.core.player(m.owner).max_hp as i32
            {
                return Flow::Done;
            }
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, format!("l10_scsanruo_effect:{c}")), 1);
            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                c,
                m.owner,
                vec![m.owner],
                Some(CardKind::Tao),
            )));
            Flow::Done
        }
        Suit::Diamond => {
            let sha_ok = !ctx.core.turn.sha_used_play_phase
                || super::super::l10_reguhuo::has_unlimited_sha(ctx.core, ctx.catalog, m.owner);
            if !sha_ok {
                return Flow::Done;
            }
            m.cards = vec![c];
            m.card = Some(ASK_SCSANRUO_TARGET);
            m.stage = HostStage::TargetChoice;
            wait_l10_scs(m, ctx)
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_scsanruo_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(c) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    ctx.core
        .vm_aux
        .flags
        .insert((m.owner, format!("l10_scsanruo_effect:{c}")), 2);
    ctx.core.vm_aux.flags.insert(
        (m.owner, format!("l10_scsanruo_steal_target:{c}")),
        u32::from(t) + 1,
    );
    ctx.emit(EventData::CardUsed(super::super::sha_card_use_data(
        ctx,
        m.owner,
        c,
        vec![t],
        Some(CardKind::FireSha),
    )));
    Flow::Done
}

pub(super) fn start_scsanruo_effect(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(c) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let mode = ctx
        .core
        .vm_aux
        .flags
        .remove(&(m.owner, format!("l10_scsanruo_effect:{c}")))
        .unwrap_or(0);
    let t_flag = ctx
        .core
        .vm_aux
        .flags
        .remove(&(m.owner, format!("l10_scsanruo_steal_target:{c}")));
    match mode {
        1 => {
            if scsanruo_steal_target_candidates(ctx.core, m.owner).is_empty() {
                return Flow::Done;
            }
            m.target = None;
            m.card = Some(ASK_SCSANRUO_STEAL_TARGET);
            m.stage = HostStage::TargetChoice;
            wait_l10_scs(m, ctx)
        }
        2 | 3 => {
            let Some(t) = t_flag.map(|v| (v - 1) as PlayerId).or(m.target) else {
                return Flow::Done;
            };
            let equip_only = mode == 3 && t == m.owner;
            if scsanruo_steal_pool(ctx.core, t, equip_only).is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            m.cards = vec![c, u32::from(equip_only)];
            m.card = Some(ASK_SCSANRUO_STEAL_CONFIRM);
            m.stage = HostStage::CardsAsk;
            wait_l10_scs(m, ctx)
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_scsanruo_steal_confirm(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
        return Flow::Done;
    }
    m.card = Some(ASK_SCSANRUO_STEAL_CARD);
    m.stage = HostStage::SelectCard;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsanruo_steal_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if scsanruo_steal_pool(ctx.core, t, false).is_empty() {
        return Flow::Done;
    }
    m.target = Some(t);
    m.cards = vec![0, 0];
    m.card = Some(ASK_SCSANRUO_STEAL_CARD);
    m.stage = HostStage::SelectCard;
    wait_l10_scs(m, ctx)
}

pub(super) fn step_scsanruo_steal_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let equip_only = m.cards.get(1).copied().unwrap_or(0) == 1;
    let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let origin = if !equip_only && ctx.core.player(t).hand.contains(&cid) {
        ctx.take_hand_card(t, cid);
        crate::event::LostOrigin::Hand
    } else if ctx.core.player(t).equips.contains(&cid) {
        ctx.core.player_mut(t).equips.retain(|&x| x != cid);
        ctx.core.declared_slots.remove(&cid);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, cid);
        crate::event::LostOrigin::Equip
    } else {
        return Flow::Done;
    };
    ctx.note_cards_lost(t, &[cid]);
    ctx.core.player_mut(m.owner).hand.push(cid);
    ctx.log(LogEvent::SkillTriggered {
        skill: "scsanruo",
        owner: m.owner,
        presentation: None,
    });
    ctx.log(LogEvent::CardsGiven {
        from: t,
        to: m.owner,
        cards: vec![cid],
    });
    ctx.emit(EventData::CardsLost {
        player: t,
        cards: vec![cid],
        origins: vec![origin],
    });
    ctx.emit(EventData::CardsMoved {
        cards: vec![cid],
        from: t,
        to: m.owner,
    });
    Flow::Done
}

fn discard_owner_he(ctx: &mut Ctx<'_>, owner: PlayerId, cards: &[CardId]) {
    for &c in cards {
        if ctx.core.player(owner).hand.contains(&c) {
            ctx.discard_from_hand(owner, c);
        } else if ctx.core.player(owner).equips.contains(&c) {
            ctx.core.player_mut(owner).equips.retain(|&x| x != c);
            ctx.core.declared_slots.remove(&c);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
            ctx.to_discard(c);
            ctx.log(LogEvent::CardDiscarded {
                player: owner,
                cards: vec![c],
            });
        }
    }
}
