//! Split tail step handlers for `combat_damage` (`twjianming`..=`taipingyaoshu`).

use super::super::l11_common::{
    emit_damage, he_cards, move_cards_to_hand, with_current_damage_mut,
};
use super::combat_damage::{current_card_use_data, current_damage_data, twqirang_candidate_targets};
use super::helpers::{
    clear_ask, current_card_use_fid, damage_effective_kind, damage_root_card_use_fid, flag,
    gain_hand_cards_from_pile_or_deck, is_red_suit, nullify_current_card_on_owner, pile_cards,
    remove_npc_b_skill, set_flag, sha_dodged_shan_cards, suit_bit, turn_stamp, wait_ask,
};
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::machine::{card_use_with_nature, Ctx, Flow};
use crate::rvm_bridge::modifiers::d34_l11_mods_npc_b::tpys_damage_prevented_npc_b;

pub(super) fn step_combat_step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "twjianming" => {
            let cid = m.card.unwrap_or(0);
            if (cid as usize) >= ctx.catalog.cards.len() {
                return Flow::Done;
            }
            let bit = suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, cid));
            let mask = if flag(ctx.core, owner, "l11b_twjianming_turn") == tstamp {
                flag(ctx.core, owner, "l11b_twjianming_suit_mask")
            } else {
                0
            };
            if (mask & bit) != 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11b_twjianming_turn", tstamp);
            set_flag(ctx.core, owner, "l11b_twjianming_suit_mask", mask | bit);
            helpers::log_skill(ctx, owner, "twjianming");
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "twqirang" => match m.stage {
            HostStage::Act => {
                if let Some((_fid, u)) = current_card_use_data(ctx) {
                    let kind = u.effective_kind(ctx.catalog);
                    let cands = twqirang_candidate_targets(ctx.core, ctx.catalog, owner, &u, kind);
                    if cands.is_empty() {
                        return Flow::Done;
                    }
                    m.cards = cands.into_iter().map(u32::from).collect();
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWQIRANG_TARGET);
                }
                let Some(pos) = ctx.core.deck.iter().rposition(|&c| {
                    let k = ctx.catalog.card_kind(c);
                    !k.is_basic_kind() && k.equip_slot().is_none()
                }) else {
                    return Flow::Done;
                };
                let cid = ctx.core.deck.remove(pos);
                helpers::log_skill(ctx, owner, "twqirang");
                gain_hand_cards_from_pile_or_deck(ctx, owner, vec![cid]);
                set_flag(ctx.core, owner, &format!("l11b_twqirang_card:{cid}"), 1);
                Flow::Done
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if let Some((fid, _)) = current_card_use_data(ctx) {
                    helpers::log_skill(ctx, owner, "twqirang");
                    if let Some(f) = ctx.res.frame_mut(fid) {
                        if let EventData::CardUsed(ref mut cu) = f.data {
                            if cu.targets.contains(&t) {
                                cu.targets.retain(|&x| x != t);
                                if cu.targets.is_empty() {
                                    cu.cancelled = true;
                                }
                            } else {
                                cu.targets.push(t);
                            }
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twxiangyu" => {
            let Some((fid, u)) = current_card_use_data(ctx) else {
                return Flow::Done;
            };
            let ar = ctx.core.attack_range(ctx.catalog, owner);
            let mut logged = false;
            for &t in &u.targets {
                if ctx.core.computed_distance(ctx.catalog, owner, t) < ar {
                    if !logged {
                        helpers::log_skill(ctx, owner, "twxiangyu");
                        logged = true;
                    }
                    set_flag(ctx.core, t, &format!("l11b_twxiangyu_extra_shan:{fid}"), 1);
                }
            }
            Flow::Done
        }
        "twxuhe" => {
            if let Some(d) = current_damage_data(ctx) {
                if d.source == Some(owner)
                    && flag(ctx.core, owner, "l11b_twxuhe_turn") == tstamp
                    && flag(ctx.core, owner, "l11b_twxuhe_target") == u32::from(d.target) + 1
                {
                    helpers::log_skill(ctx, owner, "twxuhe");
                    with_current_damage_mut(ctx, |dm| dm.amount += 2);
                    set_flag(ctx.core, owner, "l11b_twxuhe_turn", 0);
                    set_flag(ctx.core, owner, "l11b_twxuhe_target", 0);
                    set_flag(ctx.core, owner, "l11b_twxuhe_sha_fid", 0);
                }
                return Flow::Done;
            }
            match m.stage {
                HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TWXUHE_TARGET_CHOOSE),
                HostStage::SelectNature => {
                    clear_ask(ctx, owner);
                    let t = m.target.unwrap_or(owner);
                    helpers::log_skill(ctx, owner, "twxuhe");
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Tag(1)) => {
                            let cur_fid = current_card_use_fid(ctx).unwrap_or(0);
                            set_flag(ctx.core, owner, "l11b_twxuhe_turn", tstamp);
                            set_flag(ctx.core, owner, "l11b_twxuhe_target", u32::from(t) + 1);
                            set_flag(ctx.core, owner, "l11b_twxuhe_sha_fid", cur_fid);
                        }
                        _ => {
                            if ctx.core.player(t).alive {
                                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "twxuechang" => {
            if let Some(d) = current_damage_data(ctx) {
                let key = format!("l11b_xuechang_bonus:{}", d.target);
                let bonus = flag(ctx.core, owner, &key);
                if d.source == Some(owner) && bonus > 0 {
                    set_flag(ctx.core, owner, &key, 0);
                    helpers::log_skill(ctx, owner, "twxuechang");
                    with_current_damage_mut(ctx, |dm| dm.amount += bonus as i32);
                }
            }
            Flow::Done
        }
        "xinfu_kannan" => {
            if let Some(d) = current_damage_data(ctx) {
                if let Some(src) = d.source {
                    let root_fid = damage_root_card_use_fid(ctx).unwrap_or(0);
                    let eff = if root_fid > 0
                        && flag(ctx.core, src, &format!("l11b_kannan_sha:{root_fid}")) > 0
                    {
                        flag(ctx.core, src, &format!("l11b_kannan_sha:{root_fid}"))
                    } else {
                        let e = flag(ctx.core, src, "l11b_kannan_eff");
                        if e > 0 {
                            set_flag(ctx.core, src, "l11b_kannan_eff", 0);
                            if root_fid > 0 {
                                set_flag(ctx.core, src, &format!("l11b_kannan_sha:{root_fid}"), e);
                            }
                        }
                        e
                    };
                    if eff > 0 {
                        helpers::log_skill(ctx, owner, "xinfu_kannan");
                        with_current_damage_mut(ctx, |dm| dm.amount += eff as i32);
                    }
                }
            }
            Flow::Done
        }
        "xinfu_daigong" => match m.stage {
            HostStage::Act => {
                let Some(d) = current_damage_data(ctx) else {
                    return Flow::Done;
                };
                let Some(src) = d.source.filter(|&s| ctx.core.player(s).alive) else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_daigong_turn", tstamp);
                helpers::log_skill(ctx, owner, "xinfu_daigong");
                let mut mask = 0u32;
                for &c in &ctx.core.player(owner).hand {
                    if (c as usize) < ctx.catalog.cards.len() {
                        mask |= suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, c));
                    }
                }
                let src_can = he_cards(ctx.core, src).into_iter().any(|c| {
                    (c as usize) < ctx.catalog.cards.len()
                        && (mask & suit_bit(ctx.catalog.effective_card_suit(ctx.core, src, c))) == 0
                });
                if !src_can {
                    with_current_damage_mut(ctx, |dm| {
                        dm.prevented = true;
                        dm.amount = 0;
                    });
                    return Flow::Done;
                }
                m.target = Some(src);
                m.card = Some(mask);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_XINFU_DAIGONG_GIVE)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let src = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, src, owner, &[c]);
                } else {
                    with_current_damage_mut(ctx, |dm| {
                        dm.prevented = true;
                        dm.amount = 0;
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_fuyin" => {
            if let Some(fid) = current_card_use_fid(ctx) {
                set_flag(ctx.core, owner, "l11b_fuyin_turn", tstamp);
                set_flag(ctx.core, owner, "l11b_fuyin_fid", fid);
            }
            helpers::log_skill(ctx, owner, "xinfu_fuyin");
            nullify_current_card_on_owner(ctx, owner);
            Flow::Done
        }
        "xinfu_zengdao2" => {
            let pile = pile_cards(ctx.core, owner, "l11b_zengdao");
            if let Some(&cid) = pile.first() {
                let taken = super::helpers::take_from_pile(ctx.core, owner, "l11b_zengdao", &[cid]);
                for tc in &taken {
                    ctx.to_discard(*tc);
                }
                if !taken.is_empty() {
                    ctx.emit(EventData::CardsDiscarded {
                        player: owner,
                        cards: taken,
                    });
                }
                helpers::log_skill(ctx, owner, "xinfu_zengdao2");
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
                if pile_cards(ctx.core, owner, "l11b_zengdao").is_empty() {
                    remove_npc_b_skill(ctx, owner, "xinfu_zengdao2");
                }
            }
            Flow::Done
        }
        "yise" => {
            if let Some(d) = current_damage_data(ctx) {
                let _ = damage_effective_kind(ctx, &d);
                let marks = flag(ctx.core, d.target, "l11b_yise_sha_mark");
                if marks > 0 {
                    set_flag(ctx.core, d.target, "l11b_yise_sha_mark", 0);
                    helpers::log_skill(ctx, owner, "yise");
                    with_current_damage_mut(ctx, |dm| dm.amount += marks as i32);
                }
                return Flow::Done;
            }
            match m.stage {
                HostStage::Act => {
                    let t = m.target.unwrap_or(owner);
                    let mut has_red = false;
                    let mut has_black = false;
                    for &c in &m.cards {
                        if (c as usize) < ctx.catalog.cards.len() {
                            if is_red_suit(ctx.catalog.effective_card_suit(ctx.core, t, c)) {
                                has_red = true;
                            } else {
                                has_black = true;
                            }
                        }
                    }
                    m.card = Some(u32::from(has_black));
                    if has_red && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                        return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_YISE_RECOVER);
                    }
                    if has_black {
                        helpers::log_skill(ctx, owner, "yise");
                        let cur = flag(ctx.core, t, "l11b_yise_sha_mark");
                        set_flag(ctx.core, t, "l11b_yise_sha_mark", cur + 1);
                    }
                    Flow::Done
                }
                HostStage::ComparePlayerPick => {
                    clear_ask(ctx, owner);
                    let t = m.target.unwrap_or(owner);
                    let has_black = m.card == Some(1);
                    let rec = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
                    if rec || has_black {
                        helpers::log_skill(ctx, owner, "yise");
                    }
                    if rec && ctx.core.player(t).alive {
                        ctx.recover(t, 1, None);
                    }
                    if has_black && ctx.core.player(t).alive {
                        let cur = flag(ctx.core, t, "l11b_yise_sha_mark");
                        set_flag(ctx.core, t, "l11b_yise_sha_mark", cur + 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "zhidao" => match m.stage {
            HostStage::Act => {
                let t = current_damage_data(ctx).map_or(owner, |d| d.target);
                m.target = Some(t);
                set_flag(ctx.core, owner, "l11b_zhidao2_turn_stamp", tstamp);
                helpers::log_skill(ctx, owner, "zhidao");
                if !ctx.core.player(t).hand.is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHIDAO_HAND);
                }
                if !ctx.core.player(t).equips.is_empty() {
                    return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_ZHIDAO_EQUIP);
                }
                if !ctx.core.player(t).judge.is_empty() {
                    return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_ZHIDAO_JUDGE);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                if ctx.core.player(t).alive && !ctx.core.player(t).equips.is_empty() {
                    return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_ZHIDAO_EQUIP);
                }
                if ctx.core.player(t).alive && !ctx.core.player(t).judge.is_empty() {
                    return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_ZHIDAO_JUDGE);
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                if ctx.core.player(t).alive && !ctx.core.player(t).judge.is_empty() {
                    return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_ZHIDAO_JUDGE);
                }
                Flow::Done
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhiman" => match m.stage {
            HostStage::Act => {
                let t = current_damage_data(ctx).map_or(owner, |d| d.target);
                m.target = Some(t);
                helpers::log_skill(ctx, owner, "zhiman");
                with_current_damage_mut(ctx, |dm| {
                    dm.prevented = true;
                    dm.amount = 0;
                });
                if !ctx.core.player(t).equips.is_empty() || !ctx.core.player(t).judge.is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHIMAN_CARD);
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
        "zhongyong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHONGYONG_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(gainer)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let sha_t = m.target.unwrap_or(owner);
                let Some(fid) = current_card_use_fid(ctx) else {
                    return Flow::Done;
                };
                let shan_cards = sha_dodged_shan_cards(ctx.core, sha_t, fid);
                if shan_cards.is_empty() {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "zhongyong");
                ctx.core.discard.retain(|c| !shan_cards.contains(c));
                ctx.core.table.retain(|c| !shan_cards.contains(c));
                gain_hand_cards_from_pile_or_deck(ctx, gainer, shan_cards);
                if gainer != owner
                    && ctx.core.player(owner).alive
                    && ctx.core.player(sha_t).alive
                    && !ctx.core.player(owner).hand.is_empty()
                {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHONGYONG_USE_SHA);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let sha_t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    if ctx.core.player(owner).hand.contains(&cid)
                        && ctx.core.player(sha_t).alive
                        && super::helpers::can_target_sha_no_range_npc(
                            ctx.core,
                            ctx.catalog,
                            owner,
                            sha_t,
                        )
                    {
                        let raw_kind = ctx.catalog.resolved_card_kind(ctx.core, cid);
                        let eff_kind = if raw_kind.is_sha_like() {
                            raw_kind
                        } else {
                            CardKind::Sha
                        };
                        let nat = match eff_kind {
                            CardKind::FireSha => DamageNature::Fire,
                            CardKind::LeiSha => DamageNature::Thunder,
                            _ => DamageNature::Normal,
                        };
                        set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                        ctx.core.player_mut(owner).hand.retain(|&c| c != cid);
                        ctx.core.table.push(cid);
                        ctx.note_cards_lost(owner, &[cid]);
                        ctx.emit(EventData::CardUsed(card_use_with_nature(
                            cid,
                            owner,
                            vec![sha_t],
                            nat,
                            (eff_kind != ctx.catalog.card_kind(cid)).then_some(eff_kind),
                        )));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "taipingyaoshu" => {
            if let Some(d) = current_damage_data(ctx) {
                if d.target == owner
                    && d.nature != DamageNature::Normal
                    && tpys_damage_prevented_npc_b(
                        ctx.core,
                        ctx.catalog,
                        d.source,
                        d.target,
                        d.card,
                        d.nature,
                    )
                {
                    helpers::log_skill(ctx, owner, "taipingyaoshu");
                    with_current_damage_mut(ctx, |dm| {
                        dm.prevented = true;
                        dm.amount = 0;
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
