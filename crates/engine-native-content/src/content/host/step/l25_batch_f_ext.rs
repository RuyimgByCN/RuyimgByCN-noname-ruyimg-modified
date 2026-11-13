//! L25 祖安武将 Batch F 后半技能步进（技能 9–22：`ssr_tuntian` 至 `ssr_xuanhuo`）。

use super::l25_batch_f_asks::{
    discard_from_hej, gain_from_player, is_confirmed, refund_once_used, wait_ask,
};
use super::*;
use crate::catalog::{CardKind, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_f_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_tuntian" => step_ssr_tuntian(m, ctx),
        "ssr_zaoxian" => step_ssr_zaoxian(m, ctx),
        "ssr_newhengzheng" => step_ssr_newhengzheng(m, ctx),
        "ssr_baonue" => step_ssr_baonue(m, ctx),
        "ssr_newjiuchi" => step_ssr_newjiuchi(m, ctx),
        "ssr_newmizhou" => step_ssr_newmizhou(m, ctx),
        "ssr_chanyuan" => step_ssr_chanyuan(m, ctx),
        "ssr_newluanji" => step_ssr_newluanji(m, ctx),
        "ssr_newdingsheng" => step_ssr_newdingsheng(m, ctx),
        "ssr_newshiyi" => step_ssr_newshiyi(m, ctx),
        "ssr_zhuhai" => step_ssr_zhuhai(m, ctx),
        "ssr_yijian" => step_ssr_yijian(m, ctx),
        "ssr_newfabing" => step_ssr_newfabing(m, ctx),
        "ssr_xuanhuo" => step_ssr_xuanhuo(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ssr_tuntian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if matches!(m.card, Some(101) | Some(111)) && m.stage == HostStage::Act {
        let count = tuntian_marks(ctx.core, owner).clamp(1, 4) as usize;
        let mut revealed = Vec::new();
        for _ in 0..count {
            if let Some(c) = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop()) {
                revealed.push(c);
            }
        }
        if revealed.is_empty() {
            return Flow::Done;
        }
        ctx.out.push(LogEvent::CardsRevealed {
            cards: revealed.clone(),
            to: Vec::new(),
        });
        m.cards = revealed;
        return wait_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_TUNTIAN_CHOOSE_CARD);
    }
    match m.stage {
        HostStage::JiefenTargetDiscard => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let chosen = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if m.cards.contains(&c) => c,
                _ => {
                    let Some(&first) = m.cards.first() else {
                        return Flow::Done;
                    };
                    first
                }
            };
            ctx.core.player_mut(owner).hand.push(chosen);
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![chosen],
            });
            for &c in &m.cards {
                if c != chosen {
                    ctx.to_discard(c);
                }
            }
            set_flag(ctx.core, owner, "l25_tuntian_gained", chosen + 1);
            Flow::Done
        }
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TUNTIAN_DISCARD),
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if chosen.len() < 2 {
                refund_once_used(ctx, owner, "ssr_tuntian");
                return Flow::Done;
            }
            discard_he_cards(ctx, owner, &chosen[..2]);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_tuntian");
            set_flag(ctx.core, owner, "l25_tuntian_used", cnt);
            let next = tuntian_marks(ctx.core, owner) + 1;
            set_tuntian_marks(ctx.core, owner, next);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zaoxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    set_flag(ctx.core, owner, "l25_zaoxian_awakened", 1);
    set_flag(ctx.core, owner, "l25_zaoxian_jixi", 1);
    let new_max = ctx.core.player(owner).max_hp.saturating_sub(1).max(1);
    ctx.core.player_mut(owner).max_hp = new_max;
    if ctx.core.player(owner).hp > new_max as i32 {
        ctx.core.player_mut(owner).hp = new_max as i32;
    }
    let cur_hujia = ctx.core.player(owner).hujia;
    ctx.core.player_mut(owner).hujia = cur_hujia.saturating_add(1).min(5);
    ctx.start_draw(owner, 2);
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn pick_hengzheng_card(
    core: &GameStateCore,
    p: PlayerId,
    mode: u8,
) -> Option<(CardId, bool)> {
    let pl = core.player(p);
    match mode {
        0 => pl
            .equips
            .first()
            .copied()
            .map(|c| (c, true))
            .or_else(|| pl.hand.first().copied().map(|c| (c, false)))
            .or_else(|| pl.judge.first().copied().map(|c| (c, false))),
        1 => pl
            .hand
            .first()
            .copied()
            .map(|c| (c, false))
            .or_else(|| pl.equips.first().copied().map(|c| (c, true)))
            .or_else(|| pl.judge.first().copied().map(|c| (c, false))),
        _ => pl
            .judge
            .first()
            .copied()
            .map(|c| (c, false))
            .or_else(|| pl.equips.first().copied().map(|c| (c, true)))
            .or_else(|| pl.hand.first().copied().map(|c| (c, false))),
    }
}

fn step_ssr_newhengzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWHENGZHENG_MODE),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newhengzheng");
            set_flag(ctx.core, owner, "l25_hengzheng_used", cnt);
            set_flag(ctx.core, owner, "l25_hengzheng_mode", u32::from(mode) + 1);
            set_flag(ctx.core, owner, "l25_hengzheng_triggered", 1);
            let others = alive_others(ctx.core, owner);
            for p in others {
                if let Some((cid, from_equip)) = pick_hengzheng_card(ctx.core, p, mode) {
                    if from_equip {
                        set_flag(ctx.core, p, "l25_hengzheng_taofa", 1);
                        set_flag(ctx.core, p, "mark:ssr_newhengzheng", 1);
                    }
                    gain_from_player(ctx, owner, p, cid);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_baonue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    if ctx.core.player(owner).hp > 2 {
        set_flag(ctx.core, owner, "l25_baonue_unresp", 1);
        set_flag(ctx.core, owner, "l25_baonue_double_shan", 1);
    }
    Flow::Done
}

fn step_ssr_newjiuchi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWJIUCHI_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            for p in 0..ctx.core.players.len() as PlayerId {
                set_flag(ctx.core, p, "l25_hengzheng_taofa", 0);
                set_flag(ctx.core, p, "mark:ssr_newhengzheng", 0);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newjiuchi");
            set_flag(ctx.core, owner, "l25_jiuchi_used", cnt);
            set_flag(ctx.core, owner, "l25_jiuchi_viewas_jiu", 1);
            set_flag(ctx.core, owner, "l25_jiuchi_triggered", 1);
            ctx.core.turn.jiu_active = true;
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newmizhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) && m.stage == HostStage::Act {
        let lu_cards = newmizhou_pile(ctx.core, owner);
        ctx.core
            .vm_aux
            .piles
            .remove(&(owner, "ssr_newmizhou".to_string()));
        set_flag(ctx.core, owner, "l25_mizhou_lu_count", 0);
        set_flag(ctx.core, owner, "mark:ssr_newmizhou", 0);
        if lu_cards.is_empty() {
            return Flow::Done;
        }
        let n = lu_cards.len() as u32;
        let opt = flag(ctx.core, owner, "l25_mizhou_option");
        for lu in lu_cards {
            let jc = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop());
            let matched = if let Some(j_cid) = jc {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: vec![j_cid],
                    to: Vec::new(),
                });
                ctx.to_discard(j_cid);
                let j_suit = card_suit(ctx.catalog, j_cid);
                let lu_suit = card_suit(ctx.catalog, lu);
                j_suit == lu_suit
                    || is_red_suit(j_suit) == is_red_suit(lu_suit)
                    || (opt == 1 && matches!(j_suit, Suit::Spade))
                    || (opt == 2 && matches!(j_suit, Suit::Heart))
                    || (opt == 3 && matches!(j_suit, Suit::Club))
                    || (opt == 4 && matches!(j_suit, Suit::Diamond))
            } else {
                true
            };
            if matched {
                set_flag(ctx.core, owner, "l25_mizhou_matched", 1);
                let kind = ctx.catalog.card_kind(lu);
                if kind == CardKind::Tao {
                    ctx.to_discard(lu);
                    ctx.recover(owner, 1, None);
                } else if let Some(tgt) = alive_others(ctx.core, owner).into_iter().next() {
                    if matches!(
                        kind,
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Duel
                    ) {
                        ctx.to_discard(lu);
                        ctx.use_virtual_card(owner, kind, vec![tgt]);
                    } else {
                        ctx.core.player_mut(tgt).hand.push(lu);
                        ctx.out.push(LogEvent::CardsGiven {
                            from: owner,
                            to: tgt,
                            cards: vec![lu],
                        });
                    }
                } else {
                    ctx.to_discard(lu);
                }
            } else {
                ctx.to_discard(lu);
            }
        }
        set_flag(ctx.core, owner, "l25_mizhou_judged", n);
        ctx.start_draw(owner, n);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWMIZHOU_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newmizhou");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWMIZHOU_OPTION)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let opt = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            if let Some(pos) = ctx.core.player(owner).hand.iter().position(|&x| x == cid) {
                ctx.core.player_mut(owner).hand.remove(pos);
            }
            let mut pile = newmizhou_pile(ctx.core, owner);
            pile.push(cid);
            let len = pile.len() as u32;
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "ssr_newmizhou".to_string()), pile);
            set_flag(ctx.core, owner, "l25_mizhou_lu_count", len);
            set_flag(ctx.core, owner, "mark:ssr_newmizhou", len);
            set_flag(ctx.core, owner, "l25_mizhou_option", u32::from(opt) + 1);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newmizhou");
            set_flag(ctx.core, owner, "l25_mizhou_used", cnt);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_chanyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    if matches!(m.card, Some(105) | Some(109) | Some(112)) {
        let killer = m
            .target
            .filter(|&p| p != owner && ctx.core.player(p).alive)
            .or_else(|| alive_others(ctx.core, owner).into_iter().next())
            .unwrap_or(owner);
        set_flag(ctx.core, killer, "l25_chanyuan_crazy", 2);
        set_flag(ctx.core, killer, "mark:ssr_chanyuan", 2);
        set_flag(ctx.core, owner, "l25_chanyuan_triggered", 1);
        if killer != owner && ctx.core.player(killer).alive {
            if let Some(cid) = he_cards(ctx.core, killer).first().copied() {
                discard_he_cards(ctx, killer, &[cid]);
            }
        }
        return Flow::Done;
    }
    if m.card == Some(111) {
        for p in 0..ctx.core.players.len() as PlayerId {
            let cur = flag(ctx.core, p, "l25_chanyuan_crazy");
            if cur > 0 {
                let next = cur - 1;
                set_flag(ctx.core, p, "l25_chanyuan_crazy", next);
                set_flag(ctx.core, p, "mark:ssr_chanyuan", next);
                if ctx.core.player(p).alive {
                    if let Some(cid) = he_cards(ctx.core, p).first().copied() {
                        discard_he_cards(ctx, p, &[cid]);
                    }
                }
            }
        }
    }
    Flow::Done
}

fn step_ssr_newluanji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWLUANJI_DISCARD),
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if chosen.len() < 2 {
                refund_once_used(ctx, owner, "ssr_newluanji");
                return Flow::Done;
            }
            discard_he_cards(ctx, owner, &chosen[..2]);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newluanji");
            set_flag(ctx.core, owner, "l25_newluanji_used", cnt);
            let targets = alive_others(ctx.core, owner);
            if !targets.is_empty() {
                ctx.use_virtual_card(owner, CardKind::ArcheryAttack, targets);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newdingsheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newdingsheng");
    set_flag(ctx.core, owner, "l25_dingsheng_used", cnt);
    set_flag(ctx.core, owner, "l25_dingsheng_triggered", 1);
    set_flag(ctx.core, owner, "l25_dingsheng_wanjian_buff", 1);
    if let Some(eq_cid) = fetch_card_matching(ctx, |c, cat| cat.card_kind(c).is_equip()) {
        ctx.core.player_mut(owner).hand.push(eq_cid);
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: vec![eq_cid],
        });
        set_flag(ctx.core, owner, "l25_dingsheng_gained", eq_cid + 1);
        Flow::Done
    } else {
        ctx.start_draw(owner, 1);
        m.stage = HostStage::CardsAsk;
        Flow::Yield
    }
}

fn step_ssr_newshiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWSHIYI_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newshiyi");
            set_flag(ctx.core, owner, "l25_shiyi_used", cnt);
            let jue = flag(ctx.core, owner, "l25_shiyi_jue") + 1;
            set_flag(ctx.core, owner, "l25_shiyi_jue", jue);
            set_flag(ctx.core, owner, "mark:ssr_newshiyi", jue);
            if jue >= 3 {
                set_flag(ctx.core, owner, "l25_shiyi_dingsheng", 1);
                set_flag(ctx.core, owner, "l25_dingsheng_wanjian_buff", 1);
            }
            ctx.start_draw(owner, 1);
            if tgt != owner && ctx.core.player(tgt).alive {
                ctx.start_draw(tgt, 1);
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_zhuhai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUHAI_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhuhai");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUHAI_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_zhuhai");
                return Flow::Done;
            };
            if !m.cards.is_empty() {
                let to_disc = m.cards.clone();
                discard_he_cards(ctx, owner, &to_disc);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_zhuhai");
            set_flag(ctx.core, owner, "l25_zhuhai_used", cnt);
            ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_yijian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let yan = flag(ctx.core, owner, "l25_yijian_yan") + 1;
            set_flag(ctx.core, owner, "l25_yijian_yan", yan);
            set_flag(ctx.core, owner, "mark:ssr_yijian", yan);
            ctx.start_draw(owner, 1);
            if yan > 2 && flag(ctx.core, owner, "l25_yijian_awakened") == 0 {
                set_flag(ctx.core, owner, "l25_yijian_awakened", 1);
                let new_max = ctx.core.player(owner).max_hp.saturating_sub(1).max(1);
                ctx.core.player_mut(owner).max_hp = new_max;
                if ctx.core.player(owner).hp > new_max as i32 {
                    ctx.core.player_mut(owner).hp = new_max as i32;
                }
                m.stage = HostStage::DrawSettle;
            } else {
                m.stage = HostStage::CardsAsk;
            }
            Flow::Yield
        }
        HostStage::DrawSettle => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YIJIAN_TARGET),
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, tgt, "l25_yijian_wolong", 1);
                set_flag(ctx.core, owner, "l25_yijian_granted", u32::from(tgt) + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newfabing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWFABING_OPTION),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let Some(ChoiceOption::Tag(opt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newfabing");
            set_flag(ctx.core, owner, "l25_fabing_used", cnt);
            set_flag(ctx.core, owner, "l25_fabing_morphed", u32::from(opt) + 1);
            let sha_tgt = m
                .targets
                .first()
                .copied()
                .or(m.target)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .or_else(|| alive_others(ctx.core, owner).into_iter().next());
            match opt {
                0 => {
                    set_flag(ctx.core, owner, "l25_fabing_fire_sha", 1);
                    if let Some(tgt) = sha_tgt {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            tgt,
                            1,
                            DamageNature::Fire,
                            None,
                        )));
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                1 => {
                    set_flag(ctx.core, owner, "l25_fabing_thunder_sha", 1);
                    if let Some(tgt) = sha_tgt {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            tgt,
                            1,
                            DamageNature::Thunder,
                            None,
                        )));
                        m.stage = HostStage::CardsAsk;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                2 => {
                    if let Some(tgt) = sha_tgt {
                        if let Some(cid) = hej_cards(ctx.core, tgt).first().copied() {
                            discard_from_hej(ctx, tgt, cid);
                        }
                    }
                    Flow::Done
                }
                _ => {
                    ctx.start_draw(owner, 1);
                    m.stage = HostStage::CardsAsk;
                    Flow::Yield
                }
            }
        }
        _ => Flow::Done,
    }
}

fn step_ssr_xuanhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(110) && m.stage == HostStage::Act {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "l25_xuanhuo_sha_buff", 0);
            set_flag(ctx.core, p, "mark:ssr_xuanhuo", 0);
        }
        return Flow::Done;
    }
    if m.card == Some(102) && m.stage == HostStage::Act {
        set_flag(ctx.core, owner, "l25_xuanhuo_draw_triggered", 1);
        ctx.start_draw(owner, 1);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XUANHUO_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_xuanhuo");
                set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_XUANHUO_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_f_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_xuanhuo");
                return Flow::Done;
            };
            if !m.cards.is_empty() {
                let to_disc = m.cards.clone();
                discard_he_cards(ctx, owner, &to_disc);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_xuanhuo");
            set_flag(ctx.core, owner, "l25_xuanhuo_used", cnt);
            set_flag(ctx.core, tgt, "l25_xuanhuo_sha_buff", 1);
            set_flag(ctx.core, tgt, "mark:ssr_xuanhuo", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
