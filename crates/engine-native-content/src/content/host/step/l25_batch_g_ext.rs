//! L25 祖安武将 Batch G 后半技能步进（技能 9–22：`ssr_futu` 至 `ssr_fangtong`）。

use super::l25_batch_g_asks::{
    is_confirmed, judge_one_card, pop_top_deck_cards, refund_once_used, wait_ask,
};
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_g_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_wangxi" => step_ssr_wangxi(m, ctx),
        "ssr_zhiyan" => step_ssr_zhiyan(m, ctx),
        "ssr_futu" => step_ssr_futu(m, ctx),
        "ssr_newpolu" => step_ssr_newpolu(m, ctx),
        "ssr_dimeng" => step_ssr_dimeng(m, ctx),
        "ssr_fuhui" => step_ssr_fuhui(m, ctx),
        "ssr_zhijian" => step_ssr_zhijian(m, ctx),
        "ssr_guzheng" => step_ssr_guzheng(m, ctx),
        "ssr_tianlei" => step_ssr_tianlei(m, ctx),
        "ssr_gaiming" => step_ssr_gaiming(m, ctx),
        "ssr_newguidao" => step_ssr_newguidao(m, ctx),
        "ssr_zhuji" => step_ssr_zhuji(m, ctx),
        "ssr_dihuo" => step_ssr_dihuo(m, ctx),
        "ssr_guifu" => step_ssr_guifu(m, ctx),
        "ssr_renhong" => step_ssr_renhong(m, ctx),
        "ssr_fangtong" => step_ssr_fangtong(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ssr_wangxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let him = if m.target == Some(owner) {
                m.targets.first().copied()
            } else {
                m.target
            };
            let Some(him) = him.filter(|&p| p != owner && ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let revealed = pop_top_deck_cards(ctx, 2);
            if revealed.is_empty() {
                return Flow::Done;
            }
            m.target = Some(him);
            m.cards = revealed;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_WANGXI_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Card(chosen)) = ctx.answer().first().copied() else {
                for &c in &m.cards {
                    ctx.to_discard(c);
                }
                return Flow::Done;
            };
            ctx.core.player_mut(owner).hand.push(chosen);
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![chosen],
            });
            if let Some(him) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let others: Vec<CardId> =
                    m.cards.iter().copied().filter(|&c| c != chosen).collect();
                if !others.is_empty() {
                    ctx.core.player_mut(him).hand.extend(others.iter().copied());
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: him,
                        cards: others,
                    });
                }
            }
            let used = flag(ctx.core, owner, "l25_wangxi_used") + 1;
            set_flag(ctx.core, owner, "l25_wangxi_used", used);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhiyan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let pl = ctx.core.player(owner);
    let cond1 = pl.hand.len() < pl.max_hp as usize;
    let cond2 = pl.equips.len() < pl.hp.max(0) as usize;
    let count = usize::from(cond1) + usize::from(cond2);
    let mut fallback_draws = 0u32;
    for _ in 0..count {
        if let Some(cid) = fetch_card_matching(ctx, |c, cat| is_black_non_trick(cat, c)) {
            ctx.core.player_mut(owner).hand.push(cid);
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![cid],
            });
        } else {
            fallback_draws += 1;
        }
        let fetched = flag(ctx.core, owner, "l25_zhiyan_fetched") + 1;
        set_flag(ctx.core, owner, "l25_zhiyan_fetched", fetched);
    }
    if fallback_draws > 0 {
        ctx.start_draw(owner, fallback_draws);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_ssr_futu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_FUTU_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_futu");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_FUTU_PINDIAN_OWNER)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_futu");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![c1];
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_FUTU_PINDIAN_TARGET)
        }
        HostStage::ComparePlayerPick => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let Some(&c1) = m.cards.first() else {
                return Flow::Done;
            };
            let c2 = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(cid)) => cid,
                _ => {
                    let Some(&fallback) = ctx.core.player(tgt).hand.first() else {
                        return Flow::Done;
                    };
                    fallback
                }
            };
            if ctx.core.player(owner).hand.contains(&c1) {
                ctx.discard_from_hand(owner, c1);
            }
            if ctx.core.player(tgt).hand.contains(&c2) {
                ctx.discard_from_hand(tgt, c2);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_futu");
            set_flag(ctx.core, owner, "l25_futu_used", cnt);
            let r1 = i32::from(card_rank(ctx.catalog, c1));
            let guijiang_cnt = alive_all(ctx.core, owner)
                .iter()
                .filter(|&&p| flag(ctx.core, p, "l25_futu_guijiang") > 0)
                .count() as i32;
            let penalty = if flag(ctx.core, tgt, "l25_futu_guijiang") == 0 {
                guijiang_cnt
            } else {
                0
            };
            let r2 = (i32::from(card_rank(ctx.catalog, c2)) - penalty).max(1);
            if r1 > r2 {
                set_flag(ctx.core, owner, "l25_futu_win", 1);
                if flag(ctx.core, tgt, "l25_futu_guijiang") > 0 {
                    for cid in [c1, c2] {
                        if let Some(pos) = ctx.core.discard.iter().rposition(|&x| x == cid) {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(owner).hand.push(cid);
                    }
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: owner,
                        cards: vec![c1, c2],
                    });
                    set_flag(ctx.core, owner, "l25_futu_gained_both", 1);
                    return Flow::Done;
                }
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_FUTU_MODE);
            }
            Flow::Done
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            if mode == 0 {
                set_flag(ctx.core, tgt, "l25_futu_guijiang", 1);
                set_flag(ctx.core, tgt, "mark:ssr_futu_win", 1);
                ctx.start_draw(owner, 1);
                ctx.start_draw(tgt, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            set_flag(ctx.core, owner, "l25_futu_virtual_sha", 1);
            ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newpolu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    if flag(ctx.core, owner, "l25_polu_awakened") == 0 {
        set_flag(ctx.core, owner, "l25_polu_awakened", 1);
        let new_max = ctx.core.player(owner).max_hp.saturating_sub(1).max(1);
        ctx.core.player_mut(owner).max_hp = new_max;
        if ctx.core.player(owner).hp > new_max as i32 {
            ctx.core.player_mut(owner).hp = new_max as i32;
        }
        let cur_hujia = ctx.core.player(owner).hujia;
        ctx.core.player_mut(owner).hujia = cur_hujia.saturating_add(1).min(5);
        ctx.start_draw(owner, 2);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_ssr_dimeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_DIMENG_TARGETS),
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if chosen.len() < 2 {
                return Flow::Done;
            }
            let p1 = chosen[0];
            let p2 = chosen[1];
            let h1 = ctx.core.player(p1).hand.clone();
            let h2 = ctx.core.player(p2).hand.clone();
            let diff = h1.len().abs_diff(h2.len()) as u32;
            let fewer_before = if h1.len() <= h2.len() { p1 } else { p2 };
            let fewer_after = if h2.len() <= h1.len() { p1 } else { p2 };
            ctx.core.player_mut(p1).hand = h2;
            ctx.core.player_mut(p2).hand = h1;
            if diff > 0 {
                let cur1 = flag(ctx.core, fewer_before, "l25_dimeng_meng") + diff;
                set_flag(ctx.core, fewer_before, "l25_dimeng_meng", cur1);
                set_flag(ctx.core, fewer_before, "mark:ssr_dimeng_draw", cur1);
                if fewer_after != fewer_before {
                    let cur2 = flag(ctx.core, fewer_after, "l25_dimeng_meng") + diff;
                    set_flag(ctx.core, fewer_after, "l25_dimeng_meng", cur2);
                    set_flag(ctx.core, fewer_after, "mark:ssr_dimeng_draw", cur2);
                }
            }
            set_flag(ctx.core, owner, "l25_dimeng_swapped", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_fuhui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let user = m.target.unwrap_or(owner);
            let him = if user == owner {
                m.targets.first().copied()
            } else {
                Some(user)
            };
            let Some(him) = him.filter(|&p| {
                p != owner && ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty()
            }) else {
                return Flow::Done;
            };
            set_flag(
                ctx.core,
                owner,
                "l25_fuhui_user_is_owner",
                u32::from(user == owner),
            );
            m.target = Some(him);
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_FUHUI_CONFIRM)
        }
        HostStage::CompareTargetPick => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            ctx.start_draw(owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(him) = m.target else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.is_empty() || ctx.core.player(him).hand.is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_FUHUI_PINDIAN_OWNER)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![c1];
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_FUHUI_PINDIAN_TARGET)
        }
        HostStage::ComparePlayerPick => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(him) = m.target else {
                return Flow::Done;
            };
            let Some(&c1) = m.cards.first() else {
                return Flow::Done;
            };
            let c2 = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(cid)) => cid,
                _ => {
                    let Some(&fallback) = ctx.core.player(him).hand.first() else {
                        return Flow::Done;
                    };
                    fallback
                }
            };
            let bonus = ctx.core.player(owner).hand.len() as u8;
            if ctx.core.player(owner).hand.contains(&c1) {
                ctx.discard_from_hand(owner, c1);
            }
            if ctx.core.player(him).hand.contains(&c2) {
                ctx.discard_from_hand(him, c2);
            }
            let r1 = card_rank(ctx.catalog, c1).saturating_add(bonus).min(13);
            let r2 = card_rank(ctx.catalog, c2);
            let user_is_owner = flag(ctx.core, owner, "l25_fuhui_user_is_owner") > 0;
            let user_won = if user_is_owner { r1 > r2 } else { r2 > r1 };
            let target_won = if user_is_owner { r2 > r1 } else { r1 > r2 };
            if user_won {
                set_flag(ctx.core, owner, "l25_fuhui_unresp", 1);
                set_flag(ctx.core, him, "l25_fuhui_unresp", 1);
            } else if target_won {
                set_flag(ctx.core, owner, "l25_fuhui_nullified", 1);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhijian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) && m.stage == HostStage::Act {
        if let Some(tgt) = m.target {
            if flag(ctx.core, tgt, "l25_zhijian_locked_card") > 0 {
                set_flag(ctx.core, tgt, "l25_zhijian_locked_card", 0);
                set_flag(ctx.core, tgt, "mark:ssr_zhijian_gain", 0);
                if let Some(&cid) = ctx.core.player(owner).hand.first() {
                    discard_he_cards(ctx, owner, &[cid]);
                }
                set_flag(ctx.core, owner, "l25_zhijian_damaged_discard", 1);
            }
        }
        return Flow::Done;
    }
    if matches!(m.card, Some(102) | Some(106) | Some(107)) && m.stage == HostStage::Act {
        if let Some(p) = m.target {
            if flag(ctx.core, p, "l25_zhijian_locked_card") > 0 {
                set_flag(ctx.core, p, "l25_zhijian_locked_card", 0);
                set_flag(ctx.core, p, "mark:ssr_zhijian_gain", 0);
                set_flag(ctx.core, owner, "l25_zhijian_lost_draw", 1);
                ctx.start_draw(owner, 2);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHIJIAN_GIVE),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhijian");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIJIAN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhijian");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.contains(&cid) {
                ctx.give_hand_card(owner, tgt, cid);
            } else if ctx.core.player(owner).equips.contains(&cid) {
                ctx.take_owned_card(owner, cid);
                ctx.core.player_mut(tgt).hand.push(cid);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_zhijian");
            set_flag(ctx.core, owner, "l25_zhijian_used", cnt);
            set_flag(ctx.core, tgt, "l25_zhijian_locked_card", cid + 1);
            set_flag(ctx.core, tgt, "mark:ssr_zhijian_gain", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_guzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.cards.is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_GUZHENG_CONFIRM)
        }
        HostStage::CompareTargetPick => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_GUZHENG_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let chosen = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if m.cards.contains(&c) => c,
                _ => {
                    let Some(&first) = m.cards.first() else {
                        return Flow::Done;
                    };
                    first
                }
            };
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_guzheng");
            set_flag(ctx.core, owner, "l25_guzheng_used", cnt);
            if let Some(pos) = ctx.core.discard.iter().rposition(|&x| x == chosen) {
                ctx.core.discard.remove(pos);
            }
            ctx.core.player_mut(tgt).hand.push(chosen);
            ctx.out.push(LogEvent::CardsDrawn {
                player: tgt,
                cards: vec![chosen],
            });
            if tgt != owner {
                let mut rest = Vec::new();
                for &c in &m.cards {
                    if c != chosen {
                        if let Some(pos) = ctx.core.discard.iter().rposition(|&x| x == c) {
                            ctx.core.discard.remove(pos);
                        }
                        rest.push(c);
                    }
                }
                if !rest.is_empty() {
                    ctx.core.player_mut(owner).hand.extend(rest.iter().copied());
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: owner,
                        cards: rest,
                    });
                }
            }
            set_flag(ctx.core, owner, "l25_guzheng_returned", chosen + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn run_tianlei_judges_and_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let mut mask = flag(ctx.core, owner, "l25_tianlei_suit_mask");
    for i in 0..2u32 {
        if let Some(cid) = judge_one_card(ctx) {
            let bit = suit_bit(card_suit(ctx.catalog, cid));
            if (mask & bit) != 0 {
                mask |= 1 << ((i + 1) % 4);
            } else {
                mask |= bit;
            }
        } else {
            mask |= 1 << (i % 4);
        }
    }
    set_flag(ctx.core, owner, "l25_tianlei_suit_mask", mask);
    set_flag(ctx.core, owner, "l25_tianlei_last_mask", mask);
    set_flag(ctx.core, owner, "mark:ssr_tianlei", mask.count_ones());
    if (mask == 0b1111 || mask.count_ones() >= 2) && !alive_others(ctx.core, owner).is_empty() {
        return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TIANLEI_TARGET);
    }
    Flow::Done
}

fn step_ssr_tianlei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => run_tianlei_judges_and_ask(m, ctx),
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            set_flag(ctx.core, owner, "l25_tianlei_suit_mask", 0);
            set_flag(ctx.core, owner, "mark:ssr_tianlei", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "l25_tianlei_damaged", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    3,
                    DamageNature::Thunder,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_gaiming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_GAIMING_CARD),
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_he_cards(ctx, owner, &[cid]);
            set_flag(ctx.core, owner, "l25_gaiming_replaced", 1);
            set_flag(ctx.core, owner, "l25_gaiming_card", cid + 1);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newguidao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWGUIDAO_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguidao");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWGUIDAO_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguidao");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            if let Some(pos) = ctx.core.player(owner).hand.iter().position(|&x| x == cid) {
                ctx.core.player_mut(owner).hand.remove(pos);
            } else if ctx.core.player(owner).equips.contains(&cid) {
                ctx.take_owned_card(owner, cid);
            }
            ctx.core.player_mut(tgt).judge.push(cid);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newguidao");
            set_flag(ctx.core, owner, "l25_newguidao_used", cnt);
            set_flag(ctx.core, tgt, "l25_newguidao_bing", cid + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhuji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUJI_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            let chooser = m.target.unwrap_or(owner);
            discard_he_cards(ctx, chooser, &[cid]);
            set_flag(ctx.core, owner, "l25_zhuji_triggered", 1);
            run_tianlei_judges_and_ask(m, ctx)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            set_flag(ctx.core, owner, "l25_tianlei_suit_mask", 0);
            set_flag(ctx.core, owner, "mark:ssr_tianlei", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "l25_tianlei_damaged", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    3,
                    DamageNature::Thunder,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_dihuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let mut mask = flag(ctx.core, owner, "l25_dihuo_type_mask");
            for i in 0..2u32 {
                if let Some(cid) = judge_one_card(ctx) {
                    let bit = card_type_bit(ctx.catalog, cid);
                    if (mask & bit) != 0 {
                        mask |= 1 << ((i + 1) % 3);
                    } else {
                        mask |= bit;
                    }
                } else {
                    mask |= 1 << (i % 3);
                }
            }
            set_flag(ctx.core, owner, "l25_dihuo_type_mask", mask);
            set_flag(ctx.core, owner, "l25_dihuo_last_mask", mask);
            set_flag(ctx.core, owner, "mark:ssr_dihuo", mask.count_ones());
            if (mask == 0b111 || mask.count_ones() >= 2)
                && !alive_others(ctx.core, owner).is_empty()
            {
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_DIHUO_TARGET);
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            set_flag(ctx.core, owner, "l25_dihuo_type_mask", 0);
            set_flag(ctx.core, owner, "mark:ssr_dihuo", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                ctx.core.player_mut(tgt).linked = true;
                set_flag(ctx.core, owner, "l25_dihuo_damaged", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    2,
                    DamageNature::Fire,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_guifu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(102) && m.stage == HostStage::Act {
        let Some(p) = m.target else {
            return Flow::Done;
        };
        let used_cid = m.cards.first().copied();
        let mut pile = guifu_pile(ctx.core, p);
        let idx = match used_cid {
            Some(uc) => {
                let uc_red = is_red_suit(card_suit(ctx.catalog, uc));
                pile.iter()
                    .position(|&fc| is_red_suit(card_suit(ctx.catalog, fc)) == uc_red)
            }
            None => (!pile.is_empty()).then_some(0),
        };
        if let Some(i) = idx {
            let fc = pile.remove(i);
            ctx.to_discard(fc);
            let rem = pile.len() as u32;
            if pile.is_empty() {
                ctx.core.vm_aux.piles.remove(&(p, "ssr_guifu".to_string()));
            } else {
                ctx.core
                    .vm_aux
                    .piles
                    .insert((p, "ssr_guifu".to_string()), pile);
            }
            set_flag(ctx.core, p, "l25_guifu_fu_count", rem);
            set_flag(ctx.core, p, "mark:ssr_guifu_zhou", rem);
            ctx.core.player_mut(p).linked = true;
            set_flag(ctx.core, owner, "l25_guifu_triggered", 1);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_GUIFU_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_guifu");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUIFU_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_guifu");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            if let Some(pos) = ctx.core.player(owner).hand.iter().position(|&x| x == cid) {
                ctx.core.player_mut(owner).hand.remove(pos);
            }
            let mut pile = guifu_pile(ctx.core, tgt);
            pile.push(cid);
            let cnt = pile.len() as u32;
            ctx.core
                .vm_aux
                .piles
                .insert((tgt, "ssr_guifu".to_string()), pile);
            set_flag(ctx.core, tgt, "l25_guifu_fu_count", cnt);
            set_flag(ctx.core, tgt, "mark:ssr_guifu_zhou", cnt);
            let used = inc_used_this_turn(ctx.core, owner, "ssr_guifu");
            set_flag(ctx.core, owner, "l25_guifu_used", used);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_renhong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let mut sum = flag(ctx.core, owner, "l25_renhong_rank_sum");
            for _ in 0..2 {
                if let Some(cid) = judge_one_card(ctx) {
                    sum += u32::from(card_rank(ctx.catalog, cid));
                } else {
                    sum += 7;
                }
            }
            set_flag(ctx.core, owner, "l25_renhong_rank_sum", sum);
            set_flag(ctx.core, owner, "l25_renhong_last_sum", sum);
            set_flag(ctx.core, owner, "mark:ssr_renhong", sum);
            if sum >= 2 && !alive_others(ctx.core, owner).is_empty() {
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_RENHONG_TARGETS);
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            set_flag(ctx.core, owner, "l25_renhong_rank_sum", 0);
            set_flag(ctx.core, owner, "mark:ssr_renhong", 0);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if !chosen.is_empty() {
                for tgt in chosen {
                    let to_disc: Vec<CardId> =
                        ctx.core.player(tgt).hand.iter().copied().take(2).collect();
                    if !to_disc.is_empty() {
                        discard_he_cards(ctx, tgt, &to_disc);
                    }
                }
                set_flag(ctx.core, owner, "l25_renhong_discarded", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_fangtong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FANGTONG_SHOW),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_fangtong");
                set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_FANGTONG_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_fangtong");
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_fangtong");
            set_flag(ctx.core, owner, "l25_fangtong_used", cnt);
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![cid],
                to: Vec::new(),
            });
            let shown_rank = card_rank(ctx.catalog, cid);
            set_flag(
                ctx.core,
                owner,
                "l25_fangtong_shown_rank",
                u32::from(shown_rank),
            );
            m.target = Some(tgt);
            let has_eligible = ctx
                .core
                .player(tgt)
                .hand
                .iter()
                .any(|&c| card_rank(ctx.catalog, c) <= shown_rank);
            if !has_eligible {
                discard_he_cards(ctx, owner, &[cid]);
                set_flag(ctx.core, owner, "l25_fangtong_damaged", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_FANGTONG_RESPONSE)
        }
        HostStage::ComparePlayerPick => {
            set_flag(ctx.core, owner, "l25_batch_g_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let Some(&c1) = m.cards.first() else {
                return Flow::Done;
            };
            if let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: vec![c2],
                    to: Vec::new(),
                });
                set_flag(ctx.core, owner, "l25_fangtong_responded", 1);
                return Flow::Done;
            }
            discard_he_cards(ctx, owner, &[c1]);
            set_flag(ctx.core, owner, "l25_fangtong_damaged", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Normal,
                None,
            )));
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}
