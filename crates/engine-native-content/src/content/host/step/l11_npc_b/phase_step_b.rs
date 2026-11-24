//! Split tail step handlers for `phase_turn_death` (`shefu`..=`twzhengrong`).

use super::super::l11_common::{alive_all, alive_others, discard_player_cards, emit_damage, he_cards, move_cards_to_hand};
use super::helpers::{
    card_use_virtual, clear_ask, distinct_alive_factions, emit_virtual_sha,
    equip_card_onto_player, flag, gain_hand_cards_from_pile_or_deck, grant_npc_b_skill,
    push_to_pile, remove_npc_b_skill, set_flag, take_from_pile,
    take_owned_cards_from_zones, turn_stamp, wait_ask,
};
use super::phase_hooks;
use super::*;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};

pub(super) fn step_phase_step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "shefu" => match m.stage {
            HostStage::Act => {
                if ctx.core.turn.phase == Phase::End && ctx.core.turn.current == owner {
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_SHEFU_NAME);
                }
                let eff = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                    EventData::CardUsed(u) => Some(u.effective_kind(ctx.catalog)),
                    _ => None,
                });
                let Some(eff) = eff else {
                    return Flow::Done;
                };
                let tag = helpers::shefu_kind_to_tag(eff);
                if (flag(ctx.core, owner, "l11b_shefu_name_mask") & (1u32 << tag)) == 0
                    || helpers::shefu_tag_to_kind(tag) != Some(eff)
                {
                    return Flow::Done;
                }
                m.card = Some(u32::from(tag));
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_SHEFU2_CONFIRM)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(u32::from(tag));
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SHEFU_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let tag = m.card.unwrap_or(0) as u8;
                helpers::log_skill(ctx, owner, "shefu");
                let moved = take_owned_cards_from_zones(ctx, owner, &[c]);
                if let Some(&mc) = moved.first() {
                    push_to_pile(ctx.core, owner, "l11b_shefu", &[mc]);
                    let mask = flag(ctx.core, owner, "l11b_shefu_name_mask");
                    set_flag(ctx.core, owner, "l11b_shefu_name_mask", mask | (1u32 << tag));
                    set_flag(ctx.core, owner, &format!("l11b_shefu_tag_card:{tag}"), mc + 1);
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let idx = m.card.unwrap_or(0) as u8;
                let mask = flag(ctx.core, owner, "l11b_shefu_name_mask");
                let stored = flag(ctx.core, owner, &format!("l11b_shefu_tag_card:{idx}"));
                if stored == 0 {
                    return Flow::Done;
                }
                let cid = stored - 1;
                set_flag(ctx.core, owner, "l11b_shefu_name_mask", mask & !(1u32 << idx));
                set_flag(ctx.core, owner, &format!("l11b_shefu_tag_card:{idx}"), 0);
                let taken = take_from_pile(ctx.core, owner, "l11b_shefu", &[cid]);
                for tc in &taken {
                    ctx.to_discard(*tc);
                }
                if !taken.is_empty() {
                    ctx.emit(EventData::CardsDiscarded {
                        player: owner,
                        cards: taken,
                    });
                }
                helpers::log_skill(ctx, owner, "shefu");
                super::helpers::cancel_all_current_card_targets(ctx);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "sptunjiang" => {
            helpers::log_skill(ctx, owner, "sptunjiang");
            let n = distinct_alive_factions(ctx.core, ctx.catalog);
            if n > 0 {
                ctx.start_draw(owner, n);
            }
            Flow::Done
        }
        "xinjushou" => match m.stage {
            HostStage::Act => {
                helpers::log_skill(ctx, owner, "xinjushou");
                ctx.turn_over(owner);
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 4);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive && !ctx.core.player(owner).hand.is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_XINJUSHOU_CARD);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) {
                        if crate::rvm_ops::has_enabled_equip_slot(ctx.core, owner, slot) {
                            ctx.take_owned_card(owner, c);
                            ctx.note_cards_lost(owner, &[c]);
                            let _ = equip_card_onto_player(ctx, owner, c);
                            return Flow::Done;
                        }
                    }
                    discard_player_cards(ctx, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhiyan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIYAN_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                helpers::log_skill(ctx, owner, "zhiyan");
                if let Some(cid) = ctx.core.deck.pop() {
                    m.card = Some(cid);
                    gain_hand_cards_from_pile_or_deck(ctx, t, vec![cid]);
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                let cid = m.card.unwrap_or(0);
                if ctx.core.player(t).alive
                    && ctx.catalog.card_kind(cid).equip_slot().is_some()
                {
                    if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, cid) {
                        if ctx.core.player(t).hand.contains(&cid)
                            && crate::rvm_ops::has_enabled_equip_slot(ctx.core, t, slot)
                        {
                            ctx.take_owned_card(t, cid);
                            ctx.note_cards_lost(t, &[cid]);
                            let _ = equip_card_onto_player(ctx, t, cid);
                        }
                    }
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                    {
                        ctx.recover(t, 1, None);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twxuanhuo" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWXUANHUO_CARDS),
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
                if cs.len() != 2 {
                    return Flow::Done;
                }
                m.cards = cs;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXUANHUO_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(a)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(a);
                helpers::log_skill(ctx, owner, "twxuanhuo");
                let cs = std::mem::take(&mut m.cards);
                move_cards_to_hand(ctx, owner, a, &cs);
                if alive_others(ctx.core, owner).iter().any(|&p| p != a) {
                    return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_TWXUANHUO_DUEL_TARGET);
                }
                Flow::Done
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(b)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(u32::from(b));
                wait_ask(m, ctx, HostStage::SelectNature, ASK_TWXUANHUO_TARGET_CHOICE)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let a = m.target.unwrap_or(owner);
                let b = m.card.unwrap_or(0) as PlayerId;
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(0)) => {
                        emit_virtual_sha(ctx, a, vec![b], true);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(1)) => {
                        if ctx.core.player(b).alive {
                            ctx.emit(EventData::CardUsed(card_use_virtual(
                                a,
                                vec![b],
                                DamageNature::Normal,
                                CardKind::Duel,
                            )));
                        }
                        Flow::Done
                    }
                    _ => {
                        if !he_cards(ctx.core, a).is_empty() {
                            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWXUANHUO_TAKE_CARDS);
                        }
                        Flow::Done
                    }
                }
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let a = m.target.unwrap_or(owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if !cs.is_empty() {
                    move_cards_to_hand(ctx, a, owner, &cs);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twyingjia" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWYINGJIA_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWYINGJIA_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c = m.card.unwrap_or(0);
                helpers::log_skill(ctx, owner, "twyingjia");
                discard_player_cards(ctx, owner, &[c]);
                if ctx.core.player(t).alive {
                    set_flag(ctx.core, t, "extra_turn_insert_queued", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_zhaoxin" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XINFU_ZHAOXIN_PILE),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                helpers::log_skill(ctx, owner, "xinfu_zhaoxin");
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_XINFU_ZHAOXIN_TARGET_GAIN)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let c = m.card.unwrap_or(0);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    || !ctx.core.player(t).alive
                {
                    return Flow::Done;
                }
                let taken = take_from_pile(ctx.core, owner, "l11b_zhaoxin", &[c]);
                if taken.is_empty() {
                    return Flow::Done;
                }
                gain_hand_cards_from_pile_or_deck(ctx, t, taken);
                if t != owner && ctx.core.player(owner).alive && ctx.core.player(t).alive {
                    return wait_ask(m, ctx, HostStage::CardsAsk, ASK_XINFU_ZHAOXIN_DAMAGE);
                }
                Flow::Done
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(t).alive
                {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twlingfa" => match m.stage {
            HostStage::Act => {
                let r = m.card.unwrap_or(ctx.core.turn.round);
                if r >= 3 {
                    helpers::log_skill(ctx, owner, "twlingfa");
                    remove_npc_b_skill(ctx, owner, "twlingfa");
                    grant_npc_b_skill(ctx, owner, "twzhian");
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWLINGFA_MODE)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let r = m.card.unwrap_or(ctx.core.turn.round);
                helpers::log_skill(ctx, owner, "twlingfa");
                if r == 1 {
                    set_flag(ctx.core, owner, "l11b_twlingfa_sha_round", r);
                } else if r == 2 {
                    set_flag(ctx.core, owner, "l11b_twlingfa_tao_round", r);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twlingfa_sha" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWLINGFA_SHA_DISCARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "twlingfa");
                    discard_player_cards(ctx, t, &[c]);
                } else if ctx.core.player(t).alive {
                    helpers::log_skill(ctx, owner, "twlingfa");
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twlingfa_tao" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWLINGFA_TAO_GIVE),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "twlingfa");
                    move_cards_to_hand(ctx, t, owner, &[c]);
                } else if ctx.core.player(t).alive {
                    helpers::log_skill(ctx, owner, "twlingfa");
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twzhian" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TWZHIAN_MODE),
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_twzhian_turn", tstamp);
                helpers::log_skill(ctx, owner, "twzhian");
                let cid = m.card.unwrap_or(0);
                match mode {
                    0 => {
                        let _ = phase_hooks::discard_field_card(ctx, cid);
                    }
                    1 => {
                        if !ctx.core.player(owner).hand.is_empty() {
                            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWZHIAN_DISCARD_HAND);
                        }
                    }
                    _ => {
                        if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                            emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                        }
                    }
                }
                Flow::Done
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(hc)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(owner).hand.contains(&hc) {
                    return Flow::Done;
                }
                discard_player_cards(ctx, owner, &[hc]);
                let cid = m.card.unwrap_or(0);
                let _ = phase_hooks::steal_field_card(ctx, owner, cid);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twxiongzheng" => match m.stage {
            HostStage::Act => {
                let prev_t = flag(ctx.core, owner, "l11b_xz_target");
                if prev_t > 0 {
                    set_flag(ctx.core, owner, "l11b_xz_target", 0);
                    let tgt = (prev_t - 1) as PlayerId;
                    let atk_raw = flag(ctx.core, 0, &format!("l11b_xz_prev_dmg_to:{tgt}"));
                    let mut atk = 0u32;
                    let mut non_atk = 0u32;
                    for &p in &alive_all(ctx.core, owner) {
                        if (atk_raw & (1u32 << p)) != 0 {
                            atk |= 1u32 << p;
                        } else if p != owner {
                            non_atk |= 1u32 << p;
                        }
                    }
                    set_flag(ctx.core, owner, "l11b_xz_atk_mask", atk);
                    set_flag(ctx.core, owner, "l11b_xz_non_atk_mask", non_atk);
                    if atk != 0 || non_atk != 0 {
                        return wait_ask(m, ctx, HostStage::SelectNature, ASK_TWXIONGZHENG_ACTION);
                    }
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGZHENG_TARGET)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(0)) => wait_ask(
                        m,
                        ctx,
                        HostStage::CompareTargetPick,
                        ASK_TWXIONGZHENG_SHA_TARGETS,
                    ),
                    Some(ChoiceOption::Tag(1)) => wait_ask(
                        m,
                        ctx,
                        HostStage::CardsAsk,
                        ASK_TWXIONGZHENG_DRAW_TARGETS,
                    ),
                    _ => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGZHENG_TARGET),
                }
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                if !ts.is_empty() {
                    helpers::log_skill(ctx, owner, "twxiongzheng");
                    for t in ts {
                        if ctx.core.player(owner).alive && ctx.core.player(t).alive {
                            emit_virtual_sha(ctx, owner, vec![t], true);
                        }
                    }
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGZHENG_TARGET)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                if !ts.is_empty() {
                    helpers::log_skill(ctx, owner, "twxiongzheng");
                    for t in ts {
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 2);
                        }
                    }
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGZHENG_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "twxiongzheng");
                    let used = flag(ctx.core, owner, "l11b_xz_used_targets");
                    set_flag(ctx.core, owner, "l11b_xz_used_targets", used | (1u32 << t));
                    set_flag(ctx.core, owner, "l11b_xz_target", u32::from(t) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twzhengrong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWZHENGRONG_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                helpers::log_skill(ctx, owner, "twzhengrong");
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TWZHENGRONG_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let moved = take_owned_cards_from_zones(ctx, t, &[c]);
                    push_to_pile(ctx.core, owner, "l11b_twzhengrong", &moved);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
