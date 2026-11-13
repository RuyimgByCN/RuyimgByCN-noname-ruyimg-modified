//! L25 祖安武将 Batch D 后半技能步进（技能 8–22：`ssr_wusheng` 至 `ssr_newluoshen`）。

use super::l25_batch_d_asks::{gain_from_player, is_confirmed, refund_once_used, wait_ask};
use super::*;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::CardId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_d_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_wusheng" => step_ssr_wusheng(m, ctx),
        "ssr_yijue" => step_ssr_yijue(m, ctx),
        "ssr_newbaiyi" => step_ssr_newbaiyi(m, ctx),
        "ssr_liecheng" => step_ssr_liecheng(m, ctx),
        "ssr_newyucong" => step_ssr_newyucong(m, ctx),
        "ssr_newyicong" => step_ssr_newyicong(m, ctx),
        "ssr_newguose" => step_ssr_newguose(m, ctx),
        "ssr_newtongque" => step_ssr_newtongque(m, ctx),
        "ssr_newtianxiang" => step_ssr_newtianxiang(m, ctx),
        "ssr_huchi" => step_ssr_huchi(m, ctx),
        "ssr_newluoyi" => step_ssr_newluoyi(m, ctx),
        "ssr_luoyi" => step_ssr_luoyi(m, ctx),
        "ssr_zhenhun" => step_ssr_zhenhun(m, ctx),
        "ssr_paoxiao" => step_ssr_paoxiao(m, ctx),
        "ssr_newluoshen" => step_ssr_newluoshen(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ssr_wusheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_wusheng_drunk_unresp", 0);
        set_flag(ctx.core, owner, "l25_wusheng_jiu", 0);
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        set_flag(ctx.core, owner, "l25_wusheng_drunk_unresp", 1);
        set_flag(ctx.core, owner, "l25_wusheng_triggered", 1);
    }
    Flow::Done
}

fn step_ssr_yijue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_yijue_mashu", 0);
        for p in alive_all(ctx.core, owner) {
            set_flag(ctx.core, p, "l25_yijue_locked", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YIJUE_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_yijue");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_YIJUE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_yijue");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            if !ctx.core.player(tgt).equips.is_empty() {
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_YIJUE_CHOICE);
            }
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if !m.cards.is_empty() {
                let to_disc = m.cards.clone();
                discard_he_cards(ctx, owner, &to_disc);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_yijue");
            set_flag(ctx.core, owner, "l25_yijue_used", cnt);
            set_flag(ctx.core, owner, "l25_yijue_mashu", 1);
            set_flag(ctx.core, owner, "l25_wusheng_jiu", 1);
            ctx.core.turn.jiu_active = true;
            ctx.start_draw(tgt, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let choice = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            let tgt = m.target.unwrap_or(owner);
            if !m.cards.is_empty() {
                let to_disc = m.cards.clone();
                discard_he_cards(ctx, owner, &to_disc);
            }
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_yijue");
            set_flag(ctx.core, owner, "l25_yijue_used", cnt);
            set_flag(ctx.core, tgt, "l25_yijue_locked", 1);
            set_flag(ctx.core, tgt, "l25_yijue_shuiyan", 1);
            if choice == 0 && !ctx.core.player(tgt).equips.is_empty() {
                let eqs = ctx.core.player(tgt).equips.clone();
                discard_he_cards(ctx, tgt, &eqs);
                Flow::Done
            } else {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Thunder,
                    m.cards.first().copied(),
                )));
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            }
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newbaiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) && m.stage == HostStage::Act {
        let tgt = m.target.unwrap_or(owner);
        let cur = flag(ctx.core, tgt, "l25_baiyi_du");
        if cur == 0 {
            return Flow::Done;
        }
        let next = cur.saturating_sub(1);
        set_flag(ctx.core, tgt, "l25_baiyi_du", next);
        set_flag(ctx.core, tgt, "mark:ssr_newbaiyi", next);
        set_flag(ctx.core, tgt, "l25_baiyi_ticked", 1);
        if next == 0 {
            set_flag(ctx.core, tgt, "l25_baiyi_ward", 0);
            if let Some(pile) = ctx
                .core
                .vm_aux
                .piles
                .remove(&(tgt, "ssr_newbaiyi".to_string()))
            {
                for c in pile {
                    ctx.to_discard(c);
                }
            }
            ctx.start_draw(tgt, 2);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWBAIYI_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newbaiyi");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWBAIYI_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newbaiyi");
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
            ctx.core
                .vm_aux
                .piles
                .entry((tgt, "ssr_newbaiyi".to_string()))
                .or_default()
                .push(cid);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newbaiyi");
            set_flag(ctx.core, owner, "l25_newbaiyi_used", cnt);
            let rank = u32::from(card_rank(ctx.catalog, cid)).max(1);
            set_flag(ctx.core, tgt, "l25_baiyi_du", rank);
            set_flag(ctx.core, tgt, "mark:ssr_newbaiyi", rank);
            set_flag(ctx.core, tgt, "l25_baiyi_ward", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_liecheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIECHENG_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_liecheng");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            if hej_cards(ctx.core, tgt).is_empty() {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_LIECHENG_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_liecheng");
            set_flag(ctx.core, owner, "l25_liecheng_used", cnt);
            set_flag(ctx.core, owner, "l25_liecheng_triggered", 1);
            set_flag(ctx.core, tgt, "l25_liecheng_heart", 1);
            if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                gain_from_player(ctx, owner, tgt, cid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newyucong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) && m.stage == HostStage::Act {
        let tgt = m.target.unwrap_or(owner);
        set_flag(ctx.core, tgt, "l25_yucong_mounted", 0);
        set_flag(ctx.core, tgt, "mark:ssr_newyucong", 0);
        if let Some(pile) = ctx
            .core
            .vm_aux
            .piles
            .remove(&(tgt, "ssr_newyucong".to_string()))
        {
            for c in pile {
                ctx.to_discard(c);
            }
        }
        set_flag(ctx.core, owner, "l25_yucong_triggered", 1);
        return Flow::Done;
    }
    if m.card == Some(111) && m.stage == HostStage::Act {
        let tgt = m.target.unwrap_or(owner);
        set_flag(ctx.core, tgt, "l25_yucong_turn_ended", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWYUCONG_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newyucong");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWYUCONG_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newyucong");
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
            ctx.core
                .vm_aux
                .piles
                .entry((tgt, "ssr_newyucong".to_string()))
                .or_default()
                .push(cid);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newyucong");
            set_flag(ctx.core, owner, "l25_newyucong_used", cnt);
            set_flag(ctx.core, tgt, "l25_yucong_mounted", 1);
            set_flag(ctx.core, tgt, "mark:ssr_newyucong", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newyicong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let mounted_targets: Vec<PlayerId> = m
        .targets
        .iter()
        .copied()
        .filter(|&t| has_mount_or_yucong(ctx.core, ctx.catalog, t))
        .collect();
    for t in mounted_targets {
        let dis: Vec<CardId> = ctx.core.player(t).hand.iter().take(2).copied().collect();
        for c in dis {
            ctx.discard_from_hand(t, c);
        }
    }
    set_flag(ctx.core, owner, "l25_newyicong_triggered", 1);
    set_flag(ctx.core, owner, "l25_yicong_unresp", 1);
    ctx.start_draw(owner, 1);
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn step_ssr_newguose(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        let tgt = m.target.unwrap_or(owner);
        set_flag(ctx.core, tgt, "l25_guose_skip_play", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWGUOSE_MODE),
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguose");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.targets = vec![mode];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_NEWGUOSE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguose");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(tgt);
            let mode = m.targets.first().copied().unwrap_or(0);
            if mode == 1
                && !ctx.core.player(owner).hand.is_empty()
                && !ctx.core.player(tgt).hand.is_empty()
            {
                return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_NEWGUOSE_PINDIAN);
            }
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newguose");
            set_flag(ctx.core, owner, "l25_newguose_used", cnt);
            let cur_d = tongque_dice_count(ctx.core, owner);
            if cur_d > 0 {
                set_tongque_dice(ctx.core, owner, cur_d - 1);
            }
            set_flag(ctx.core, tgt, "l25_guose_skip_play", 1);
            set_flag(ctx.core, tgt, "l25_guose_lebu", 4);
            set_flag(ctx.core, tgt, "mark:ssr_newguose", 1);
            Flow::Done
        }
        HostStage::ComparePlayerPick => {
            let Some(ChoiceOption::Card(my_cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguose");
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![my_cid];
            wait_ask(
                m,
                ctx,
                HostStage::CompareTargetPick,
                ASK_NEWGUOSE_TARGET_PINDIAN,
            )
        }
        HostStage::CompareTargetPick => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Card(tgt_cid)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newguose");
                return Flow::Done;
            };
            let tgt = m.target.unwrap_or(owner);
            let Some(&my_cid) = m.cards.first() else {
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newguose");
            set_flag(ctx.core, owner, "l25_newguose_used", cnt);
            if ctx.core.player(owner).hand.contains(&my_cid) {
                ctx.discard_from_hand(owner, my_cid);
            }
            if ctx.core.player(tgt).hand.contains(&tgt_cid) {
                ctx.discard_from_hand(tgt, tgt_cid);
            }
            let my_r = card_rank(ctx.catalog, my_cid);
            let tgt_r = card_rank(ctx.catalog, tgt_cid);
            set_flag(ctx.core, owner, "l25_guose_pindian_done", 1);
            if my_r > tgt_r {
                set_flag(ctx.core, tgt, "l25_guose_skip_play", 1);
                set_flag(ctx.core, owner, "l25_guose_pindian_win", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newtongque(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if (m.card == Some(110) || m.card == Some(108)) && m.stage == HostStage::Act {
        set_tongque_dice(ctx.core, owner, 2);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWTONGQUE_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let cur = tongque_dice_count(ctx.core, owner);
            set_tongque_dice(ctx.core, owner, cur.saturating_sub(1));
            set_flag(ctx.core, owner, "l25_tongque_prevented", 1);
            set_flag(ctx.core, owner, "l25_tongque_triggered", 1);
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_newtianxiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) && m.stage == HostStage::Act {
        return wait_ask(m, ctx, HostStage::SelectCard, ASK_NEWTIANXIANG_DISCARD);
    }
    match m.stage {
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_ask(
                m,
                ctx,
                HostStage::TargetChoice,
                ASK_NEWTIANXIANG_DEFLECT_TARGET,
            )
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !m.cards.is_empty() {
                let to_disc = m.cards.clone();
                discard_he_cards(ctx, owner, &to_disc);
            }
            let victim = m.target.unwrap_or(owner);
            if ctx.core.player(victim).hp < ctx.core.player(victim).max_hp as i32 {
                ctx.recover(victim, 1, None);
            }
            set_flag(ctx.core, owner, "l25_tianxiang_deflected", u32::from(tgt) + 1);
            set_flag(ctx.core, owner, "l25_tianxiang_triggered", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Normal,
                m.cards.first().copied(),
            )));
            let lost = (ctx.core.player(tgt).max_hp as i32 - ctx.core.player(tgt).hp + 1)
                .clamp(1, 5) as u32;
            ctx.start_draw(tgt, lost);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        HostStage::Act => wait_ask(
            m,
            ctx,
            HostStage::JiefenTarget,
            ASK_NEWTIANXIANG_PLAY_TARGET,
        ),
        HostStage::JiefenTarget => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, owner, "ssr_newtianxiang");
                return Flow::Done;
            };
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newtianxiang");
            set_flag(ctx.core, owner, "l25_newtianxiang_used", cnt);
            let cur = tongque_dice_count(ctx.core, owner);
            if cur > 0 {
                set_tongque_dice(ctx.core, owner, cur - 1);
            }
            set_flag(ctx.core, tgt, "l25_tianxiang_die", 1);
            set_flag(ctx.core, tgt, "mark:ssr_newtianxiang", 3);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_huchi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    set_flag(ctx.core, owner, "l25_huchi_triggered", 1);
    set_flag(ctx.core, owner, "l25_huchi_duel_extra", 1);
    set_flag(ctx.core, owner, "l25_luoyi_double_req", 1);
    ctx.start_draw(owner, 1);
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn step_ssr_newluoyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(101) && m.stage == HostStage::Act {
        let ba = flag(ctx.core, owner, "l25_newluoyi_ba") + 1;
        set_flag(ctx.core, owner, "l25_newluoyi_ba", ba);
        set_flag(ctx.core, owner, "mark:ssr_newluoyi", ba);
        set_flag(ctx.core, owner, "l25_newluoyi_triggered", 1);
        if flag(ctx.core, owner, "l25_luoyi_active") > 0 {
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWLUOYI_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l25_luoyi_active", 1);
            set_flag(ctx.core, owner, "l25_luoyi_dmg_boost", 1);
            set_flag(ctx.core, owner, "l25_newluoyi_triggered", 1);
            ctx.start_draw(owner, 1);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_ssr_luoyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    set_flag(ctx.core, owner, "l25_luoyi_triggered", 1);
    set_flag(ctx.core, owner, "l25_luoyi_double_req", 1);
    set_flag(ctx.core, owner, "l25_luoyi_dmg_boost", 1);
    Flow::Done
}

fn step_ssr_zhenhun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        for p in alive_all(ctx.core, owner) {
            set_flag(ctx.core, p, "l25_zhenhun_forbid_sha", 0);
            set_flag(ctx.core, p, "l25_zhenhun_baiban", 0);
            set_flag(ctx.core, p, "mark:ssr_zhenhun", 0);
        }
        return Flow::Done;
    }
    if m.card == Some(101) && m.stage == HostStage::Act {
        let tgt = m.target.unwrap_or(owner);
        set_flag(ctx.core, tgt, "l25_zhenhun_baiban", 1);
        set_flag(ctx.core, tgt, "l25_zhenhun_forbid_sha", 1);
        set_flag(ctx.core, tgt, "mark:ssr_zhenhun", 1);
        set_flag(ctx.core, owner, "l25_zhenhun_triggered", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_ZHENHUN_CONFIRM),
        HostStage::SelectNature => {
            if !is_confirmed(ctx) {
                set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHENHUN_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
                set_flag(ctx.core, tgt, "l25_zhenhun_forbid_sha", 1);
                set_flag(ctx.core, tgt, "l25_zhenhun_baiban", 1);
                set_flag(ctx.core, tgt, "mark:ssr_zhenhun", 1);
                set_flag(ctx.core, owner, "l25_zhenhun_triggered", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ssr_paoxiao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(111) {
        set_flag(ctx.core, owner, "l25_paoxiao_sha_count", 0);
        set_flag(ctx.core, owner, "l25_paoxiao_extra_sha", 0);
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        let cnt = flag(ctx.core, owner, "l25_paoxiao_sha_count") + 1;
        set_flag(ctx.core, owner, "l25_paoxiao_sha_count", cnt);
        set_flag(ctx.core, owner, "l25_paoxiao_extra_sha", 1);
        set_flag(ctx.core, owner, "l25_paoxiao_triggered", 1);
        ctx.core.turn.sha_used_play_phase = false;
    }
    Flow::Done
}

fn step_ssr_newluoshen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWLUOSHEN_CONFIRM),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_batch_d_ask_id", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l25_newluoshen_triggered", 1);
            if let Some(jc) = ctx.core.deck.pop().or_else(|| ctx.core.discard.pop()) {
                ctx.out.push(LogEvent::CardsRevealed {
                    cards: vec![jc],
                    to: Vec::new(),
                });
                ctx.core.player_mut(owner).hand.push(jc);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![jc],
                });
                let gained = flag(ctx.core, owner, "l25_luoshen_gained") + 1;
                set_flag(ctx.core, owner, "l25_luoshen_gained", gained);
                let black = is_black_suit(card_suit(ctx.catalog, jc));
                if black && gained < 4 && !ctx.core.deck.is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_NEWLUOSHEN_CONFIRM);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
