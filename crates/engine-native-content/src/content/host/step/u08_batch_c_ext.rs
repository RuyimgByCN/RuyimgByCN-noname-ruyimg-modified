//! U08 英雄杀 Batch C 后半部分步骤状态机（`u08_batch_c_ext.rs`）。

use super::u08_batch_c_asks::*;
use super::*;
use crate::catalog::{CardKind, Suit};
use crate::content::standard::Gender;
use crate::event::{CardUseData, DamageNature, DrawOutcome, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

fn clear_ducai(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "ducai_suit", 0);
    set_flag(ctx.core, owner, "yxs_ducai_forbidden_suit", 0);
    set_flag(ctx.core, owner, "ducai2_active", 0);
    remove_skill_named(ctx, owner, "ducai2");
}

fn trigger_nichang2_draw(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    set_flag(ctx.core, owner, "nichang2_active", 0);
    remove_skill_named(ctx, owner, "nichang2");
    let mut suit_mask = 0u32;
    for &c in &ctx.core.player(owner).hand {
        let bit = match ctx.catalog.effective_card_suit(ctx.core, owner, c) {
            Suit::Spade => 1,
            Suit::Heart => 2,
            Suit::Club => 4,
            Suit::Diamond => 8,
        };
        suit_mask |= bit;
    }
    let n = 4u32.saturating_sub(suit_mask.count_ones());
    if n > 0 {
        draw_direct(ctx, owner, n);
    }
    Flow::Done
}

fn handle_wange2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "wange2_active", 0);
            remove_skill_named(ctx, owner, "wange2");
            if wange2_candidates(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_WANGE2_TARGETS)
        }
        HostStage::TargetChoice => {
            m.stage = HostStage::Act;
            let chosen = answered_players(ctx);
            for t in chosen {
                if let Some(&cid) = ctx.core.player(t).hand.first() {
                    move_cards_to_hand(ctx, t, owner, &[cid]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn advance_seyou_male(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while let Some(&male) = m.targets.first() {
        if !ctx.core.player(male).alive || !ctx.core.player(owner).alive {
            m.targets.remove(0);
            continue;
        }
        let Some(t) = m.target else {
            m.stage = HostStage::Act;
            return Flow::Done;
        };
        let has_sha = ctx.core.player(t).alive
            && ctx
                .core
                .player(male)
                .hand
                .iter()
                .any(|&c| is_sha_card(ctx.core, ctx.catalog, c));
        if has_sha {
            return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_SEYOU_MALE_SHA);
        }
        let he = he_cards(ctx.core, male);
        if he.is_empty() {
            m.targets.remove(0);
            continue;
        }
        if he.len() == 1 {
            move_cards_to_hand(ctx, male, owner, &[he[0]]);
            m.targets.remove(0);
            continue;
        }
        return wait_ask(m, ctx, HostStage::SelectCard, ASK_SEYOU_GAIN_CARD);
    }
    m.stage = HostStage::Act;
    Flow::Done
}

pub(super) fn handle_u08_batch_c_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "ducai" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    clear_ducai(ctx, owner);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_DUCAI_CARD)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "ducai");
                    return Flow::Done;
                };
                let sc = suit_code(ctx.catalog.effective_card_suit(ctx.core, owner, cid));
                discard_cards(ctx, owner, &[cid]);
                set_flag(ctx.core, owner, "ducai_suit", sc);
                set_flag(ctx.core, owner, "yxs_ducai_forbidden_suit", sc);
                set_flag(ctx.core, owner, "ducai2_active", 1);
                grant_skill_named(ctx, owner, "ducai2");
                Flow::Done
            }
            _ => Flow::Done,
        },
        "ducai2" => {
            clear_ducai(ctx, owner);
            Flow::Done
        }
        "tongling" => {
            let cur = flag(ctx.core, owner, "tongling_mark")
                .max(flag(ctx.core, owner, "yxs_tongling_marks"));
            if cur < 3 {
                let next = (cur + 1).min(3);
                set_flag(ctx.core, owner, "tongling_mark", next);
                set_flag(ctx.core, owner, "yxs_tongling_marks", next);
            }
            Flow::Done
        }
        "fanpu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_FANPU_TARGETS),
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let mut chosen = answered_players(ctx);
                chosen.retain(|&t| t != owner && ctx.core.player(t).alive);
                chosen.truncate(3);
                if chosen.is_empty() {
                    refund_once_used(ctx, owner, "fanpu");
                    return Flow::Done;
                }
                let cur = flag(ctx.core, owner, "tongling_mark")
                    .max(flag(ctx.core, owner, "yxs_tongling_marks"));
                let rem = cur.saturating_sub(3);
                set_flag(ctx.core, owner, "tongling_mark", rem);
                set_flag(ctx.core, owner, "yxs_tongling_marks", rem);
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: chosen,
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fenghuo" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FENGHUO_CARD),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                if !ctx.core.player(owner).equips.contains(&cid) {
                    return Flow::Done;
                }
                let targets = nanman_targets(ctx.core, ctx.catalog, owner, cid);
                if targets.is_empty() {
                    return Flow::Done;
                }
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: Some(cid),
                    player: owner,
                    targets,
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::SavageAttack),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "nichang" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    return trigger_nichang2_draw(ctx, owner);
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_NICHANG_CONFIRM)
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    ctx.core.turn.skip_draw_phase = true;
                    for frame in ctx.res.stack.iter_mut().rev() {
                        if let EventData::DrawPhase {
                            player,
                            pending,
                            outcome,
                            ..
                        } = &mut frame.data
                        {
                            if *player == owner {
                                *pending = 0;
                                *outcome = DrawOutcome::Skipped;
                                break;
                            }
                        }
                    }
                    set_flag(ctx.core, owner, "nichang2_active", 1);
                    grant_skill_named(ctx, owner, "nichang2");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "nichang2" => trigger_nichang2_draw(ctx, owner),
        "fengyan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_FENGYAN_CONFIRM),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    if let Some(j_card) = m.card.and_then(|_| m.cards.first().copied()) {
                        ctx.core.discard.retain(|&c| c != j_card);
                        ctx.core.deck.retain(|&c| c != j_card);
                        if !ctx.core.player(owner).hand.contains(&j_card) {
                            ctx.core.player_mut(owner).hand.push(j_card);
                        }
                        ctx.log(LogEvent::JudgementGained {
                            player: owner,
                            card: j_card,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FENGYI_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    refund_once_used(ctx, owner, "fengyi");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_FENGYI_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_players(ctx).into_iter().next()) else {
                    refund_once_used(ctx, owner, "fengyi");
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[cid]);
                draw_direct(ctx, t, 2);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "wange" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    set_flag(ctx.core, owner, "wange_draw_minus_1", 0);
                    for frame in ctx.res.stack.iter_mut().rev() {
                        if let EventData::DrawPhase {
                            player, pending, ..
                        } = &mut frame.data
                        {
                            if *player == owner {
                                *pending = (*pending - 1).max(0);
                                break;
                            }
                        }
                    }
                    return Flow::Done;
                }
                if m.card == Some(3) {
                    return handle_wange2(m, ctx);
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_WANGE_CONFIRM)
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    let mut applied = false;
                    for frame in ctx.res.stack.iter_mut().rev() {
                        if let EventData::DrawPhase {
                            player, pending, ..
                        } = &mut frame.data
                        {
                            if *player == owner {
                                *pending = (*pending - 1).max(0);
                                applied = true;
                                break;
                            }
                        }
                    }
                    if !applied {
                        set_flag(ctx.core, owner, "wange_draw_minus_1", 1);
                    }
                    set_flag(ctx.core, owner, "wange2_active", 1);
                    grant_skill_named(ctx, owner, "wange2");
                }
                Flow::Done
            }
            HostStage::TargetChoice => handle_wange2(m, ctx),
            _ => Flow::Done,
        },
        "wange2" => handle_wange2(m, ctx),
        "seyou" => match m.stage {
            HostStage::Act => {
                if m.card == Some(99) {
                    m.card = None;
                    return advance_seyou_male(m, ctx);
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_SEYOU_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(t) = answered_players(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    refund_once_used(ctx, owner, "seyou");
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "seyou_used", 1);
                m.target = Some(t);
                m.targets = ctx
                    .core
                    .seat_order_from(owner)
                    .into_iter()
                    .filter(|&p| {
                        p != owner
                            && p != t
                            && ctx.core.player(p).alive
                            && player_gender(ctx.core, ctx.catalog, p) == Gender::Male
                    })
                    .collect();
                advance_seyou_male(m, ctx)
            }
            HostStage::CompareTargetPick => {
                let Some(&male) = m.targets.first() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if let Some(sha_cid) = answered_cards(ctx).into_iter().next() {
                    m.targets.remove(0);
                    if let Some(t) = m.target {
                        if ctx.core.player(t).alive {
                            m.stage = HostStage::Act;
                            m.card = Some(99);
                            ctx.emit(EventData::CardUsed(CardUseData {
                                card: Some(sha_cid),
                                player: male,
                                targets: vec![t],
                                nature: DamageNature::Normal,
                                use_as: Some(CardKind::Sha),
                                cancelled: false,
                            }));
                            return Flow::Yield;
                        }
                    }
                    return advance_seyou_male(m, ctx);
                }
                let he = he_cards(ctx.core, male);
                if he.is_empty() {
                    m.targets.remove(0);
                    return advance_seyou_male(m, ctx);
                }
                if he.len() == 1 {
                    move_cards_to_hand(ctx, male, owner, &[he[0]]);
                    m.targets.remove(0);
                    return advance_seyou_male(m, ctx);
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SEYOU_GAIN_CARD)
            }
            HostStage::SelectCard => {
                if !m.targets.is_empty() {
                    let male = m.targets.remove(0);
                    if let Some(cid) = answered_cards(ctx)
                        .into_iter()
                        .next()
                        .or_else(|| he_cards(ctx.core, male).into_iter().next())
                    {
                        move_cards_to_hand(ctx, male, owner, &[cid]);
                    }
                }
                advance_seyou_male(m, ctx)
            }
            _ => Flow::Done,
        },
        "sheshi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_SHESHI_SUIT),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                let Some(tag) = answered_tag(ctx) else {
                    return Flow::Done;
                };
                let declared_suit = match tag {
                    0 => Suit::Spade,
                    1 => Suit::Heart,
                    2 => Suit::Club,
                    _ => Suit::Diamond,
                };
                let mut to_hand = Vec::new();
                let mut matched_card = None;
                for _ in 0..4 {
                    let popped = pop_deck_cards(ctx, 1);
                    let Some(&c) = popped.first() else {
                        break;
                    };
                    if ctx.catalog.effective_card_suit(ctx.core, owner, c) == declared_suit {
                        matched_card = Some(c);
                        break;
                    }
                    to_hand.push(c);
                }
                if let Some(mc) = matched_card {
                    ctx.to_discard(Some(mc));
                }
                if !to_hand.is_empty() {
                    ctx.core.player_mut(owner).hand.extend(to_hand.iter().copied());
                    ctx.emit(EventData::CardsDrawn {
                        player: owner,
                        cards: to_hand,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => {
            m.stage = HostStage::Act;
            Flow::Done
        }
    }
}
