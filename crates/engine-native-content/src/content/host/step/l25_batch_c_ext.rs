//! L25 `祖安武将` Batch C —— 后半技能（夏侯惇/华佗/于禁/左慈/华雄/黄盖）步进实现。

use super::l25_batch_c_asks::{discard_single_he, give_single_he, is_confirmed, wait_ask};
use super::*;
use crate::catalog::{CardKind, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_c_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_newganglie" => step_ssr_newganglie(m, ctx),
        "ssr_zuni" => step_ssr_zuni(m, ctx),
        "ssr_chuli" => step_ssr_chuli(m, ctx),
        "ssr_xianfang" => step_ssr_xianfang(m, ctx),
        "ssr_newzhenjun" => step_ssr_newzhenjun(m, ctx),
        "ssr_newyizhong" => step_ssr_newyizhong(m, ctx),
        "ssr_shendao" => step_ssr_shendao(m, ctx),
        "ssr_newjiahe" => step_ssr_newjiahe(m, ctx),
        "ssr_miedi" => step_ssr_miedi(m, ctx),
        "ssr_shiyong" => step_ssr_shiyong(m, ctx),
        "ssr_newkurou" => step_ssr_newkurou(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ssr_newganglie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => match m.card {
            Some(102) => {
                let opp = if m.target == Some(m.owner) {
                    m.targets
                        .iter()
                        .copied()
                        .find(|&t| t != m.owner)
                        .unwrap_or(m.owner)
                } else {
                    m.target.unwrap_or(m.owner)
                };
                m.target = Some(opp);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWGANGLIE_CONFIRM)
            }
            Some(101) => wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWGANGLIE_SHA),
            _ => Flow::Done,
        },
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if is_confirmed(ctx) {
                let opp = m.target.unwrap_or(m.owner);
                for p in [m.owner, opp] {
                    let hand = ctx.core.player(p).hand.clone();
                    if hand.len() > 1 {
                        for &cid in &hand[1..] {
                            discard_single_he(ctx, p, cid);
                        }
                    }
                }
                set_flag(ctx.core, m.owner, "l25_newganglie_sha_triggered", 1);
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if is_confirmed(ctx) {
                if let Some(src) = m.target {
                    if src != m.owner && ctx.core.player(src).alive {
                        set_flag(ctx.core, m.owner, "l25_newganglie_damage_triggered", 1);
                        ctx.use_virtual_card(m.owner, CardKind::Sha, vec![src]);
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zuni(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZUNI_DISCARD),
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                set_flag(ctx.core, m.owner, "l25_zuni_busy", 1);
                discard_single_he(ctx, m.owner, cid);
                set_flag(ctx.core, m.owner, "l25_zuni_busy", 0);
                let tgt = m.target.unwrap_or(m.owner);
                let mut returned = Vec::new();
                for &dc in &m.cards {
                    if let Some(pos) = ctx.core.discard.iter().position(|&x| x == dc) {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(tgt).hand.push(dc);
                        returned.push(dc);
                    } else if !ctx.core.player(tgt).hand.contains(&dc) {
                        ctx.core.player_mut(tgt).hand.push(dc);
                        returned.push(dc);
                    }
                }
                set_flag(ctx.core, m.owner, "l25_zuni_triggered", 1);
                set_flag(
                    ctx.core,
                    tgt,
                    "l25_zuni_returned",
                    returned.len().max(1) as u32,
                );
                if !returned.is_empty() {
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: tgt,
                        cards: returned,
                    });
                } else {
                    ctx.start_draw(tgt, 1);
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_chuli(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if m.card == Some(109) || m.card == Some(111) {
                let mut drew_any = false;
                for p in alive_all(ctx.core, m.owner) {
                    let pending = flag(ctx.core, p, "l25_chuli_draw_pending");
                    if pending > 0 {
                        set_flag(ctx.core, p, "l25_chuli_draw_pending", 0);
                        ctx.start_draw(p, pending);
                        drew_any = true;
                    }
                }
                if drew_any {
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_CHULI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            if ctx.core.player(tgt).hand.is_empty() {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_CHULI_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            bump_used_this_turn(ctx.core, m.owner, "l25_chuli_used");
            let tgt = m.target.unwrap_or(m.owner);
            if let Some(pos) = ctx.core.player(tgt).hand.iter().position(|&x| x == cid) {
                ctx.core.player_mut(tgt).hand.remove(pos);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((m.owner, "ssr_chuli".to_string()))
                    .or_default()
                    .push(cid);
                let yao = flag(ctx.core, m.owner, "l25_chuli_yao") + 1;
                set_flag(ctx.core, m.owner, "l25_chuli_yao", yao);
            }
            ctx.recover(tgt, 1, None);
            if tgt == m.owner || ctx.catalog.card_kind(cid).is_basic_kind() {
                let dm = flag(ctx.core, tgt, "l25_chuli_draw_mark") + 1;
                set_flag(ctx.core, tgt, "l25_chuli_draw_mark", dm);
                let pend = flag(ctx.core, tgt, "l25_chuli_draw_pending") + 1;
                set_flag(ctx.core, tgt, "l25_chuli_draw_pending", pend);
            }
            let ts = turn_stamp(ctx.core);
            if flag(ctx.core, m.owner, "l25_chuli_suits_turn") != ts {
                set_flag(ctx.core, m.owner, "l25_chuli_suits_turn", ts);
                set_flag(ctx.core, m.owner, "l25_chuli_suits_mask", 0);
            }
            let sc = suit_code(card_suit(ctx.catalog, cid));
            if sc > 0 {
                let bit = 1u32 << sc;
                let mask = flag(ctx.core, m.owner, "l25_chuli_suits_mask");
                if (mask & bit) != 0 {
                    set_flag(ctx.core, m.owner, "l25_chuli_forbid_turn", ts);
                } else {
                    set_flag(ctx.core, m.owner, "l25_chuli_suits_mask", mask | bit);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_xianfang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_XIANFANG_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if is_confirmed(ctx) {
                let round = ctx.core.turn.round.max(1);
                set_flag(ctx.core, m.owner, "l25_xianfang_used_round", round);
                set_flag(ctx.core, m.owner, "l25_xianfang_triggered", 1);
                let key = (m.owner, "ssr_chuli".to_string());
                let mut discarded = Vec::new();
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&key) {
                    let n = pile.len().min(4);
                    discarded = pile.drain(..n).collect();
                }
                for cid in discarded {
                    ctx.to_discard(cid);
                }
                let yao = flag(ctx.core, m.owner, "l25_chuli_yao");
                set_flag(ctx.core, m.owner, "l25_chuli_yao", yao.saturating_sub(4));
                let dying_p = m.target.unwrap_or(m.owner);
                ctx.recover(dying_p, 2, None);
                if ctx.core.player(dying_p).hp <= 0 {
                    ctx.core.player_mut(dying_p).hp = 1;
                }
                ctx.start_draw(dying_p, 2);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newzhenjun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if matches!(m.card, Some(108) | Some(110) | Some(112)) {
                set_flag(ctx.core, m.owner, "l25_zhenjun_active", 0);
                for p in alive_all(ctx.core, m.owner) {
                    set_flag(ctx.core, p, "l25_zhenjun_sealed", 0);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWZHENJUN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                bump_used_this_turn(ctx.core, m.owner, "l25_zhenjun_used");
                set_flag(ctx.core, tgt, "l25_zhenjun_sealed", 1);
                set_flag(ctx.core, m.owner, "l25_zhenjun_active", 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    m.owner,
                    1,
                    DamageNature::Normal,
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

fn step_ssr_newyizhong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    match m.card {
        Some(101) => {
            if m.cards.first().is_some_and(|&c| {
                is_red_suit(card_suit(ctx.catalog, c))
                    && matches!(
                        ctx.catalog.card_kind(c),
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                    )
            }) {
                set_flag(ctx.core, m.owner, "l25_newyizhong_red", 1);
            }
            let cur_hp = ctx.core.player(m.owner).hp;
            if cur_hp > 1 {
                if let Ok(d_ref) = ctx.current_damage() {
                    let prevented = d_ref.prevented;
                    let amount = d_ref.amount;
                    if !prevented && amount > 0 {
                        let k = (amount as u32).min((cur_hp - 1).max(1) as u32);
                        let d = ctx.current_damage_mut();
                        d.amount -= k as i32;
                        if d.amount <= 0 {
                            d.prevented = true;
                        }
                        let new_hj = ctx
                            .core
                            .player(m.owner)
                            .hujia
                            .saturating_add(k as u8)
                            .min(5);
                        ctx.core.player_mut(m.owner).hujia = new_hj;
                        set_flag(ctx.core, m.owner, "l25_newyizhong_hujia", u32::from(new_hj));
                        set_flag(ctx.core, m.owner, "l25_newyizhong_converted", 1);
                        ctx.emit(EventData::LoseHp {
                            player: m.owner,
                            amount: k,
                        });
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                } else if flag(ctx.core, m.owner, "l25_newyizhong_converted") == 0 {
                    let new_hj = (ctx.core.player(m.owner).hujia + 1).min(5);
                    ctx.core.player_mut(m.owner).hujia = new_hj;
                    set_flag(ctx.core, m.owner, "l25_newyizhong_hujia", u32::from(new_hj));
                    set_flag(ctx.core, m.owner, "l25_newyizhong_converted", 1);
                }
            }
            Flow::Done
        }
        Some(108) => {
            set_flag(ctx.core, m.owner, "l25_newyizhong_converted", 0);
            if flag(ctx.core, m.owner, "l25_newyizhong_red") > 0 {
                set_flag(ctx.core, m.owner, "l25_newyizhong_red", 0);
                return Flow::Done;
            }
            let hj = ctx.core.player(m.owner).hujia as usize;
            if hj > 0 {
                let top_n: Vec<CardId> = ctx.core.deck.iter().take(hj).copied().collect();
                let spades = if top_n.is_empty() {
                    1u32
                } else {
                    top_n
                        .iter()
                        .filter(|&&c| card_suit(ctx.catalog, c) == Suit::Spade)
                        .count() as u32
                };
                if !top_n.is_empty() {
                    ctx.out.push(LogEvent::CardsRevealed {
                        cards: top_n,
                        to: Vec::new(),
                    });
                }
                if spades > 0 {
                    let rem = ctx
                        .core
                        .player(m.owner)
                        .hujia
                        .saturating_sub(spades as u8);
                    ctx.core.player_mut(m.owner).hujia = rem;
                    set_flag(ctx.core, m.owner, "l25_newyizhong_hujia", u32::from(rem));
                    ctx.recover(m.owner, spades, None);
                }
                set_flag(ctx.core, m.owner, "l25_newyizhong_checked", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_shendao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SHENDAO_DISCARD),
        HostStage::SelectCard => {
            let cs: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if cs.len() < 2 {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            }
            m.cards = cs;
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHENDAO_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SHENDAO_OPTION)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            let Some(ChoiceOption::Tag(opt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(m.owner);
            let to_disc = m.cards.clone();
            for cid in to_disc {
                discard_single_he(ctx, m.owner, cid);
            }
            let val = u32::from(opt) + 1;
            set_flag(ctx.core, tgt, "l25_shendao_shenfu", val);
            set_flag(ctx.core, m.owner, "l25_shendao_shenfu", val);
            match opt {
                0 => {
                    ctx.recover(tgt, 1, None);
                    ctx.start_draw(tgt, 1);
                }
                1 => {
                    ctx.start_draw(tgt, 2);
                }
                _ => {
                    ctx.start_draw(tgt, 1);
                    ctx.start_draw(m.owner, 1);
                }
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newjiahe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if matches!(m.card, Some(108) | Some(110)) {
                set_flag(ctx.core, m.owner, "l25_jiahe_qianxing", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWJIAHE_CONFIRM)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            if is_confirmed(ctx) {
                let pl = ctx.core.player(m.owner);
                let lost = (pl.max_hp as i32 - pl.hp).max(0) as u32;
                set_flag(ctx.core, m.owner, "l25_jiahe_qianxing", 1 + lost);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_miedi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_MIEDI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            bump_used_this_turn(ctx.core, m.owner, "l25_miedi_used");
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => wait_ask(m, ctx, HostStage::SelectNature, ASK_MIEDI_CHOICE),
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            let Some(ChoiceOption::Tag(opt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(m.owner);
            if opt == 0 {
                let ts = turn_stamp(ctx.core);
                set_flag(ctx.core, m.owner, "l25_miedi_forbid_turn", ts);
                set_flag(ctx.core, m.owner, "l25_miedi_gave", 1);
                let cands = he_cards(ctx.core, tgt);
                if cands.is_empty() {
                    return Flow::Done;
                }
                if cands.len() <= 2 {
                    for cid in cands {
                        give_single_he(ctx, tgt, m.owner, cid);
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_MIEDI_GIVE)
            } else {
                set_flag(ctx.core, m.owner, "l25_miedi_sha", 1);
                ctx.use_virtual_card(tgt, CardKind::Sha, vec![m.owner]);
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            }
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            let tgt = m.target.unwrap_or(m.owner);
            let cs: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            for cid in cs {
                give_single_he(ctx, tgt, m.owner, cid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_shiyong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    match m.card {
        Some(102) => {
            if let Some(src) = m.target {
                set_flag(
                    ctx.core,
                    m.owner,
                    "l25_shiyong_sha_source",
                    u32::from(src) + 1,
                );
            }
            if m.cards
                .first()
                .is_some_and(|&c| is_red_suit(card_suit(ctx.catalog, c)))
            {
                set_flag(ctx.core, m.owner, "l25_shiyong_red_sha_double_shan", 1);
            }
            Flow::Done
        }
        Some(104) | Some(103) | Some(101) => {
            let sha_src = if m.card == Some(104) || m.card == Some(101) {
                m.target
            } else {
                let recorded = flag(ctx.core, m.owner, "l25_shiyong_sha_source");
                if recorded > 0 {
                    Some(PlayerId::from((recorded - 1) as u8))
                } else if ctx.core.turn.current != m.owner {
                    Some(ctx.core.turn.current)
                } else {
                    alive_others(ctx.core, m.owner).first().copied()
                }
            };
            if let Some(src) = sha_src {
                if src != m.owner && ctx.core.player(src).alive {
                    set_flag(ctx.core, m.owner, "l25_shiyong_triggered", 1);
                    ctx.emit(EventData::LoseHp {
                        player: src,
                        amount: 1,
                    });
                    m.stage = HostStage::CardsAsk;
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newkurou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if matches!(m.card, Some(108) | Some(110)) {
                set_flag(ctx.core, m.owner, "l25_kurou_active", 0);
                for p in alive_all(ctx.core, m.owner) {
                    set_flag(ctx.core, p, "l25_kurou_suit_lock", 0);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWKUROU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            ctx.emit(EventData::LoseHp {
                player: m.owner,
                amount: 1,
            });
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let tgt = m.target.unwrap_or(m.owner);
            if ctx.core.player(tgt).hand.is_empty() {
                set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWKUROU_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l25_c_ask_id", 0);
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(m.owner);
            if ctx.core.player(tgt).hand.contains(&cid) {
                ctx.give_hand_card(tgt, m.owner, cid);
            }
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![cid],
                to: Vec::new(),
            });
            let sc = suit_code(card_suit(ctx.catalog, cid)).max(1);
            set_flag(ctx.core, tgt, "l25_kurou_suit_lock", sc);
            set_flag(ctx.core, m.owner, "l25_kurou_active", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
