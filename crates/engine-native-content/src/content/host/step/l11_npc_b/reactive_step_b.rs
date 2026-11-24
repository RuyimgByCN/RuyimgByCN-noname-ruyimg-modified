//! Split tail step handlers for `reactive_misc` (`meiyingqiang`..=`zhuide`).

use super::super::l11_common::{
    discard_player_cards, emit_lose_hp, he_cards, move_cards_to_hand, with_current_damage_mut,
};
use super::helpers::{
    can_target_sha_no_range_npc, card_type_group, clear_ask, flag,
    gain_hand_cards_from_pile_or_deck, is_red_suit, move_field_card,
    nullify_current_card_on_owner, phase_stamp, pile_cards, push_to_pile, set_flag,
    take_from_pile, take_owned_cards_from_zones, turn_stamp, wait_ask,
};
use super::reactive_misc::{current_damage_struct, owner_has_sha_use_card};
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::CardId;
use crate::machine::{card_use_with_nature, Ctx, Flow};

pub(super) fn step_reactive_step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    let pstamp = phase_stamp(ctx.core);
    match m.def_key.as_str() {
        "meiyingqiang" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11b_meiyingqiang_lost_turn", tstamp);
                if !owner_has_sha_use_card(ctx.core, ctx.catalog, owner) {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_MEIYINGQIANG_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_MEIYINGQIANG_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cid = m.card.unwrap_or(0);
                if ctx.core.player(owner).hand.contains(&cid)
                    && can_target_sha_no_range_npc(ctx.core, ctx.catalog, owner, t)
                {
                    let rk = ctx.catalog.resolved_card_kind(ctx.core, cid);
                    let ek = if rk.is_sha_like() { rk } else { CardKind::Sha };
                    let nat = match ek {
                        CardKind::FireSha => DamageNature::Fire,
                        CardKind::LeiSha => DamageNature::Thunder,
                        _ => DamageNature::Normal,
                    };
                    helpers::log_skill(ctx, owner, "meiyingqiang");
                    set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                    ctx.core.player_mut(owner).hand.retain(|&c| c != cid);
                    ctx.core.table.push(cid);
                    ctx.note_cards_lost(owner, &[cid]);
                    ctx.emit(EventData::CardUsed(card_use_with_nature(
                        cid,
                        owner,
                        vec![t],
                        nat,
                        (ek != ctx.catalog.card_kind(cid)).then_some(ek),
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xiehui" => {
            if let Some(to) = m.target {
                for &c in &m.cards {
                    if !is_red_suit(ctx.catalog.effective_card_suit(ctx.core, to, c)) {
                        set_flag(ctx.core, to, &format!("l11b_xiehui2_card:{c}"), 1);
                    }
                }
            }
            Flow::Done
        }
        "twyilie" => match m.stage {
            HostStage::Act => {
                if let Some(d_or_u) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                    EventData::ShaDodged { source, .. } if *source == owner => Some(1usize),
                    EventData::CardUsed(u) if u.player == owner => Some(
                        u.targets
                            .iter()
                            .filter(|&&t| ctx.core.player(t).linked)
                            .count(),
                    ),
                    _ => None,
                }) {
                    if d_or_u > 0 && flag(ctx.core, owner, "l11b_twyilie_opt2_turn") == tstamp {
                        helpers::log_skill(ctx, owner, "twyilie");
                        ctx.start_draw(owner, d_or_u as u32);
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_TWYILIE_MODE)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                helpers::log_skill(ctx, owner, "twyilie");
                if mode == 0 || mode == 2 {
                    set_flag(ctx.core, owner, "l11b_twyilie_phase_stamp", pstamp);
                    set_flag(ctx.core, owner, "l11b_twyilie_opt1", 1);
                }
                if mode == 1 || mode == 2 {
                    set_flag(ctx.core, owner, "l11b_twyilie_opt2_turn", tstamp);
                }
                if mode == 2 {
                    emit_lose_hp(ctx, owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "weifeng" => match m.stage {
            HostStage::Act => {
                if let Some(d) = current_damage_struct(ctx) {
                    let stored = flag(ctx.core, d.target, "l11b_weifeng_kind");
                    if stored == 0 {
                        return Flow::Done;
                    }
                    helpers::log_skill(ctx, owner, "weifeng");
                    set_flag(ctx.core, d.target, "l11b_weifeng_kind", 0);
                    let cur_kind = super::helpers::damage_effective_kind(ctx, &d)
                        .map(|k| (k as u32) + 1)
                        .unwrap_or(0);
                    if cur_kind == stored {
                        with_current_damage_mut(ctx, |dm| dm.amount += 1);
                    } else if !he_cards(ctx.core, d.target).is_empty() {
                        m.target = Some(d.target);
                        return wait_ask(m, ctx, HostStage::SelectCard, ASK_WEIFENG3_GAIN);
                    }
                    return Flow::Done;
                }
                if let Some(&first_t) = m.targets.first() {
                    if m.targets.len() == 1 {
                        let k = m.card.unwrap_or(1);
                        helpers::log_skill(ctx, owner, "weifeng");
                        set_flag(ctx.core, first_t, "l11b_weifeng_kind", k);
                        return Flow::Done;
                    }
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_WEIFENG_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let k = m.card.unwrap_or(1);
                    helpers::log_skill(ctx, owner, "weifeng");
                    set_flag(ctx.core, t, "l11b_weifeng_kind", k);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "wulie" => match m.stage {
            HostStage::Act => {
                if current_damage_struct(ctx).is_some() {
                    let cur = flag(ctx.core, owner, "l11b_wulie2_marks");
                    if cur > 0 {
                        set_flag(ctx.core, owner, "l11b_wulie2_marks", cur - 1);
                        helpers::log_skill(ctx, owner, "wulie");
                        with_current_damage_mut(ctx, |dm| {
                            dm.prevented = true;
                            dm.amount = 0;
                        });
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_WULIE_TARGETS)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                if ts.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11b_limited_wulie", 1);
                helpers::log_skill(ctx, owner, "wulie");
                emit_lose_hp(ctx, owner, ts.len() as u32);
                for t in ts {
                    let cur = flag(ctx.core, t, "l11b_wulie2_marks");
                    set_flag(ctx.core, t, "l11b_wulie2_marks", cur + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_shajue" => {
            let dying = m.target.unwrap_or(owner);
            let cur = flag(ctx.core, owner, "xinfu_xionghuo_marks");
            if cur < 3 {
                helpers::log_skill(ctx, owner, "xinfu_shajue");
                set_flag(ctx.core, owner, "xinfu_xionghuo_marks", cur + 1);
            }
            if ctx.core.player(dying).hp < 0 {
                if let Some(cid) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                    EventData::CardUsed(u) => u.card,
                    EventData::Damage(d) => d.card,
                    _ => None,
                }) {
                    if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
                        ctx.core.table.remove(pos);
                        helpers::log_skill(ctx, owner, "xinfu_shajue");
                        gain_hand_cards_from_pile_or_deck(ctx, owner, vec![cid]);
                    }
                }
            }
            Flow::Done
        }
        "xinjiewei" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XINJIEWEI_DISCARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(dc)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                helpers::log_skill(ctx, owner, "xinjiewei");
                discard_player_cards(ctx, owner, &[dc]);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINJIEWEI_SRC)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(src)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(src);
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_XINJIEWEI_CARD)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_XINJIEWEI_DST)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(dst)) = ctx.answer().first().copied() {
                    let src = m.target.unwrap_or(owner);
                    let cid = m.card.unwrap_or(0);
                    let _ = move_field_card(ctx, src, cid, dst);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinsheng" => match m.stage {
            HostStage::Act => {
                let rem = current_damage_struct(ctx).map_or(1, |d| d.amount.max(1) as u32);
                helpers::log_skill(ctx, owner, "xinsheng");
                let h = flag(ctx.core, owner, "rehuashen_pool_count") + 1;
                set_flag(ctx.core, owner, "rehuashen_pool_count", h);
                set_flag(ctx.core, owner, "rehuashen", h);
                if rem > 1 {
                    m.card = Some(rem - 1);
                    return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_XINSHENG_CONFIRM);
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let rem = m.card.unwrap_or(0);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) && rem > 0 {
                    helpers::log_skill(ctx, owner, "xinsheng");
                    let h = flag(ctx.core, owner, "rehuashen_pool_count") + 1;
                    set_flag(ctx.core, owner, "rehuashen_pool_count", h);
                    set_flag(ctx.core, owner, "rehuashen", h);
                    if rem > 1 {
                        m.card = Some(rem - 1);
                        return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_XINSHENG_CONFIRM);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yechou" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YECHOU_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "yechou");
                    set_flag(ctx.core, t, "l11b_yechou2", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yinbing" => match m.stage {
            HostStage::Act => {
                if current_damage_struct(ctx).is_some() {
                    let pile = pile_cards(ctx.core, owner, "l11b_yinbing");
                    if let Some(&cid) = pile.first() {
                        let taken = take_from_pile(ctx.core, owner, "l11b_yinbing", &[cid]);
                        for tc in &taken {
                            ctx.to_discard(*tc);
                        }
                        if !taken.is_empty() {
                            helpers::log_skill(ctx, owner, "yinbing");
                            ctx.emit(EventData::CardsDiscarded {
                                player: owner,
                                cards: taken,
                            });
                        }
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YINBING_CARDS)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if !cs.is_empty() {
                    helpers::log_skill(ctx, owner, "yinbing");
                    let moved = take_owned_cards_from_zones(ctx, owner, &cs);
                    push_to_pile(ctx.core, owner, "l11b_yinbing", &moved);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yuce" => match m.stage {
            HostStage::Act => {
                let dmg_src = current_damage_struct(ctx).and_then(|d| d.source);
                m.target = dmg_src;
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YUCE_SHOW)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                helpers::log_skill(ctx, owner, "yuce");
                let grp = card_type_group(ctx.catalog.resolved_card_kind(ctx.core, c));
                m.card = Some(u32::from(grp));
                let has_diff = m.target.is_some_and(|src| {
                    ctx.core.player(src).alive
                        && ctx.core.player(src).hand.iter().any(|&hc| {
                            card_type_group(ctx.catalog.resolved_card_kind(ctx.core, hc)) != grp
                        })
                });
                if has_diff {
                    return wait_ask(m, ctx, HostStage::CardsAsk, ASK_YUCE_SOURCE_DISCARD);
                }
                if ctx.core.player(owner).alive
                    && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let src = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(dc)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, src, &[dc]);
                } else if ctx.core.player(owner).alive
                    && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhichi" => {
            if current_damage_struct(ctx).is_some() {
                helpers::log_skill(ctx, owner, "zhichi");
                set_flag(ctx.core, owner, "l11b_zhichi2", 1);
            } else {
                helpers::log_skill(ctx, owner, "zhichi");
                nullify_current_card_on_owner(ctx, owner);
            }
            Flow::Done
        }
        "zhuide" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUIDE_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let mut seen_kinds = Vec::new();
                let mut basics = Vec::new();
                for &c in &ctx.core.deck {
                    let k = ctx.catalog.card_kind(c);
                    if k.is_basic_kind() && !seen_kinds.contains(&k) {
                        seen_kinds.push(k);
                        basics.push(c);
                    }
                }
                if basics.is_empty() {
                    return Flow::Done;
                }
                m.cards = basics;
                wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUIDE_BASICS)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .take(4)
                    .collect();
                if !picked.is_empty() && ctx.core.player(t).alive {
                    helpers::log_skill(ctx, owner, "zhuide");
                    let mut gained = Vec::new();
                    for c in picked {
                        if let Some(pos) = ctx.core.deck.iter().position(|&dc| dc == c) {
                            gained.push(ctx.core.deck.remove(pos));
                        }
                    }
                    gain_hand_cards_from_pile_or_deck(ctx, t, gained);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
