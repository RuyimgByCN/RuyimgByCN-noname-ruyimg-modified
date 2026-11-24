//! Active (`phaseUse`) Part B skills + respond skill hooks (`active_a.rs`):
//! `shangyi`, `twqingce`, `twjiefan`, `twxiongxi`, `twxuechang`, `twyuanhu`,
//! `waishi`, `weijing`, `twmouli`, `shizhi`, `twwushen`.

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_damage, he_cards,
    hej_cards, move_cards_to_hand,
};
use super::helpers::{
    clear_ask, equip_card_onto_player, flag, is_red_suit, is_unblocked_npc_b, phase_stamp,
    pile_cards, pindian_resolve_cards, set_flag, take_from_pile, turn_stamp, wait_ask,
};
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{card_use_with_nature, Ctx, Flow};
use crate::state::GameStateCore;

pub(super) fn has_phase_use_a(name: &str) -> bool {
    matches!(
        name,
        "shangyi"
            | "twqingce"
            | "twjiefan"
            | "twxiongxi"
            | "twxuechang"
            | "twyuanhu"
            | "waishi"
            | "weijing"
            | "twmouli"
            | "shizhi"
            | "twwushen"
    )
}

pub(super) fn activate_a(
    skill: &HostSkillBehavior,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<HostMachine> {
    if !is_unblocked_npc_b(state, catalog, owner) {
        return None;
    }
    let pstamp = phase_stamp(state);
    let eligible = match skill.name.as_str() {
        "shangyi" => {
            flag(state, owner, "l11b_shangyi_stamp") != pstamp
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "twqingce" => {
            !pile_cards(state, owner, "l11b_twzhengrong").is_empty()
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !hej_cards(state, p).is_empty())
        }
        "twjiefan" => flag(state, owner, "l11b_limited_twjiefan") == 0,
        "twxiongxi" => {
            let need = 5u32.saturating_sub(flag(state, owner, "baonvezhi")) as usize;
            flag(state, owner, "l11b_twxiongxi_stamp") != pstamp
                && he_cards(state, owner).len() >= need
                && !alive_others(state, owner).is_empty()
        }
        "twxuechang" => {
            flag(state, owner, "l11b_twxuechang_stamp") != pstamp
                && !state.player(owner).hand.is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "twyuanhu" => {
            flag(state, owner, "l11b_twyuanhu_stamp") != pstamp
                && he_cards(state, owner).into_iter().any(|c| {
                    let Some(slot) = catalog.card_slot_of(state, c) else {
                        return false;
                    };
                    alive_all(state, owner)
                        .into_iter()
                        .any(|p| crate::rvm_ops::has_enabled_equip_slot(state, p, slot))
                })
        }
        "waishi" => {
            let max_x = helpers::distinct_alive_factions(state, catalog) as usize;
            flag(state, owner, "l11b_waishi_stamp") != pstamp
                && max_x > 0
                && !he_cards(state, owner).is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "weijing" => {
            flag(state, owner, "l11b_weijing_round") != state.turn.round
                && helpers::can_use_sha_this_play_phase(state, catalog, owner)
        }
        "twmouli" => {
            flag(state, owner, "l11b_twmouli_turn") != turn_stamp(state)
                && state
                    .deck
                    .iter()
                    .any(|&c| helpers::can_use_basic_in_play(state, catalog, owner, c))
        }
        "shizhi" => {
            state.player(owner).hp == 1
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| catalog.resolved_card_kind(state, c) == CardKind::Shan)
                && helpers::can_use_sha_this_play_phase(state, catalog, owner)
        }
        "twwushen" => state
            .player(owner)
            .hand
            .iter()
            .any(|&c| catalog.effective_card_suit(state, owner, c) == Suit::Heart)
            && !helpers::usable_sha_targets_no_range(state, catalog, owner).is_empty(),
        _ => false,
    };
    eligible.then(|| HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11CrossNpcB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn step_active_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let pstamp = phase_stamp(ctx.core);
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "shangyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHANGYI_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_shangyi_stamp", pstamp);
                helpers::log_skill(ctx, owner, "shangyi");
                let has_black = ctx.core.player(t).hand.iter().any(|&c| {
                    !is_red_suit(ctx.catalog.effective_card_suit(ctx.core, t, c))
                });
                if !has_black {
                    return Flow::Done;
                }
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SHANGYI_DISCARD_BLACK)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twqingce" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWQINGCE_PILE),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWQINGCE_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let pile_c = m.card.unwrap_or(0);
                let taken = take_from_pile(ctx.core, owner, "l11b_twzhengrong", &[pile_c]);
                if taken.is_empty() {
                    return Flow::Done;
                }
                for tc in &taken {
                    ctx.to_discard(*tc);
                }
                ctx.emit(EventData::CardsDiscarded {
                    player: owner,
                    cards: taken,
                });
                helpers::log_skill(ctx, owner, "twqingce");
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWQINGCE_DISCARD)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(dc)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[dc]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twjiefan" => match m.stage {
            HostStage::Act => {
                if m.target.is_some()
                    || flag(ctx.core, owner, "l11b_jiefan_dying_triggered") > 0
                {
                    set_flag(ctx.core, owner, "l11b_jiefan_dying_triggered", 0);
                    set_flag(ctx.core, owner, "l11b_limited_twjiefan", 0);
                    set_flag(ctx.core, owner, "l11b_twjiefan2_target", 0);
                    helpers::log_skill(ctx, owner, "twjiefan");
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWJIEFAN_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_limited_twjiefan", 1);
                set_flag(ctx.core, owner, "l11b_twjiefan2_target", u32::from(t) + 1);
                helpers::log_skill(ctx, owner, "twjiefan");
                let responders: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(owner)
                    .into_iter()
                    .filter(|&p| {
                        p != t
                            && ctx.core.player(p).alive
                            && ctx.core.in_attack_range(ctx.catalog, p, t)
                    })
                    .collect();
                m.target = Some(t);
                m.targets = responders;
                set_flag(ctx.core, owner, "l11b_jiefan_idx", 0);
                advance_twjiefan(m, ctx)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let idx = flag(ctx.core, owner, "l11b_jiefan_idx") as usize;
                let t = m.target.unwrap_or(owner);
                if let Some(&resp) = m.targets.get(idx) {
                    if let Some(ChoiceOption::Card(wc)) = ctx.answer().first().copied() {
                        discard_player_cards(ctx, resp, &[wc]);
                    } else if ctx.core.player(t).alive {
                        ctx.start_draw(t, 1);
                    }
                }
                set_flag(ctx.core, owner, "l11b_jiefan_idx", (idx + 1) as u32);
                advance_twjiefan(m, ctx)
            }
            _ => Flow::Done,
        },
        "twxiongxi" => match m.stage {
            HostStage::Act => {
                let need = 5u32.saturating_sub(flag(ctx.core, owner, "baonvezhi"));
                if need == 0 {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGXI_TARGET)
                } else {
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_TWXIONGXI_DISCARD)
                }
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
                let need = 5u32.saturating_sub(flag(ctx.core, owner, "baonvezhi")) as usize;
                if cs.len() < need {
                    return Flow::Done;
                }
                m.cards = cs;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXIONGXI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_twxiongxi_stamp", pstamp);
                helpers::log_skill(ctx, owner, "twxiongxi");
                let to_disc = std::mem::take(&mut m.cards);
                if !to_disc.is_empty() {
                    discard_player_cards(ctx, owner, &to_disc);
                }
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twxuechang" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXUECHANG_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                set_flag(ctx.core, owner, "l11b_twxuechang_stamp", pstamp);
                helpers::log_skill(ctx, owner, "twxuechang");
                wait_ask(
                    m,
                    ctx,
                    HostStage::ComparePlayerPick,
                    ASK_TWXUECHANG_PINDIAN_OWNER,
                )
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_TWXUECHANG_PINDIAN_TARGET,
                )
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                let c1 = m.card.unwrap_or(0);
                let (won, _, _) = pindian_resolve_cards(ctx, owner, c1, t, c2);
                if won {
                    if ctx.core.player(owner).alive
                        && ctx.core.player(t).alive
                        && !he_cards(ctx.core, t).is_empty()
                    {
                        return wait_ask(m, ctx, HostStage::SelectCard, ASK_TWXUECHANG_GAIN);
                    }
                    Flow::Done
                } else {
                    if ctx.core.player(owner).alive {
                        let cur = flag(ctx.core, owner, &format!("l11b_xuechang_bonus:{t}"));
                        set_flag(ctx.core, owner, &format!("l11b_xuechang_bonus:{t}"), cur + 1);
                        emit_damage(ctx, Some(t), owner, 1, DamageNature::Normal, None);
                    }
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let is_eq = ctx.catalog.card_kind(c).equip_slot().is_some();
                    move_cards_to_hand(ctx, t, owner, &[c]);
                    if is_eq && ctx.core.player(owner).alive && ctx.core.player(t).alive {
                        helpers::emit_virtual_sha(ctx, owner, vec![t], true);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twyuanhu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWYUANHU_EQUIP),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWYUANHU_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c = m.card.unwrap_or(0);
                if ctx.core.turn.phase == crate::event::Phase::Play {
                    set_flag(ctx.core, owner, "l11b_twyuanhu_stamp", pstamp);
                }
                helpers::log_skill(ctx, owner, "twyuanhu");
                let slot = ctx.catalog.card_slot_of(ctx.core, c);
                ctx.take_owned_card(owner, c);
                ctx.note_cards_lost(owner, &[c]);
                let _ = equip_card_onto_player(ctx, t, c);
                m.target = Some(t);
                match slot {
                    Some(EquipSlot::Weapon) => {
                        let has_v = alive_all(ctx.core, owner).into_iter().any(|p| {
                            p != t
                                && ctx.core.computed_distance(ctx.catalog, t, p) == 1
                                && !hej_cards(ctx.core, p).is_empty()
                        });
                        if has_v {
                            return wait_ask(
                                m,
                                ctx,
                                HostStage::CompareTargetPick,
                                ASK_TWYUANHU_WEAPON_TGT,
                            );
                        }
                        finish_twyuanhu_draw(ctx, owner, t);
                        Flow::Done
                    }
                    Some(EquipSlot::Armor) => {
                        m.stage = HostStage::DrawSettle;
                        ctx.start_draw(t, 1);
                        Flow::Yield
                    }
                    Some(_) => {
                        if ctx.core.player(t).alive
                            && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                        {
                            ctx.recover(t, 1, None);
                        }
                        finish_twyuanhu_draw(ctx, owner, t);
                        Flow::Done
                    }
                    None => Flow::Done,
                }
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let orig_t = m.target.unwrap_or(owner);
                let Some(ChoiceOption::Player(vt)) = ctx.answer().first().copied() else {
                    finish_twyuanhu_draw(ctx, owner, orig_t);
                    return Flow::Done;
                };
                m.targets = vec![orig_t];
                m.target = Some(vt);
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWYUANHU_WEAPON_DISCARD)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let vt = m.target.unwrap_or(owner);
                let orig_t = m.targets.first().copied().unwrap_or(vt);
                if let Some(ChoiceOption::Card(dc)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, vt, &[dc]);
                }
                finish_twyuanhu_draw(ctx, owner, orig_t);
                Flow::Done
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                finish_twyuanhu_draw(ctx, owner, t);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "waishi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_WAISHI_CARDS),
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
                if cs.is_empty() {
                    return Flow::Done;
                }
                m.cards = cs;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_WAISHI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                set_flag(ctx.core, owner, "l11b_waishi_stamp", pstamp);
                helpers::log_skill(ctx, owner, "waishi");
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_WAISHI_TARGET_CARDS)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let t_cards: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if ctx.core.player(t).hand.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if t_cards.len() != m.cards.len() {
                    return Flow::Done;
                }
                let o_cards = m.cards.clone();
                move_cards_to_hand(ctx, owner, t, &o_cards);
                move_cards_to_hand(ctx, t, owner, &t_cards);
                if ctx.core.player(owner).alive
                    && ctx.core.player(t).alive
                    && (ctx.core.player(t).hand.len() > ctx.core.player(owner).hand.len()
                        || helpers::same_faction(ctx.catalog, ctx.core, owner, t))
                {
                    ctx.start_draw(owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "weijing" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_WEIJING_MODE),
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                if ctx.answer().first().is_none() {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_WEIJING_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_weijing_round", ctx.core.turn.round);
                helpers::log_skill(ctx, owner, "weijing");
                helpers::emit_virtual_sha(ctx, owner, vec![t], false);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twmouli" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWMOU_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWMOU_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cid = m.card.unwrap_or(0);
                let Some(pos) = ctx.core.deck.iter().position(|&c| c == cid) else {
                    return Flow::Done;
                };
                ctx.core.deck.remove(pos);
                ctx.core.table.push(cid);
                set_flag(ctx.core, owner, "l11b_twmouli_turn", tstamp);
                helpers::log_skill(ctx, owner, "twmouli");
                let kind = ctx.catalog.card_kind(cid);
                let nat = match kind {
                    CardKind::FireSha => DamageNature::Fire,
                    CardKind::LeiSha => DamageNature::Thunder,
                    _ => DamageNature::Normal,
                };
                let targets = if kind.is_sha_like() { vec![t] } else { vec![owner] };
                ctx.emit(EventData::CardUsed(card_use_with_nature(
                    cid, owner, targets, nat, None,
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shizhi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SHIZHI_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHIZHI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cid = m.card.unwrap_or(0);
                if !ctx.core.player(owner).hand.contains(&cid) {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "shizhi");
                ctx.core.player_mut(owner).hand.retain(|&c| c != cid);
                ctx.core.table.push(cid);
                ctx.note_cards_lost(owner, &[cid]);
                ctx.emit(EventData::CardUsed(card_use_with_nature(
                    cid,
                    owner,
                    vec![t],
                    DamageNature::Normal,
                    Some(CardKind::Sha),
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twwushen" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TWWUSHEN_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWWUSHEN_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cid = m.card.unwrap_or(0);
                if !ctx.core.player(owner).hand.contains(&cid) {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "twwushen");
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                ctx.take_owned_card(owner, cid);
                ctx.core.table.push(cid);
                ctx.note_cards_lost(owner, &[cid]);
                ctx.emit(EventData::CardUsed(card_use_with_nature(
                    cid,
                    owner,
                    vec![t],
                    DamageNature::Normal,
                    Some(CardKind::Sha),
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn advance_twjiefan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let t = m.target.unwrap_or(owner);
    let mut idx = flag(ctx.core, owner, "l11b_jiefan_idx") as usize;
    while idx < m.targets.len() {
        let resp = m.targets[idx];
        if !ctx.core.player(resp).alive {
            idx += 1;
            continue;
        }
        let has_weapon = he_cards(ctx.core, resp)
            .into_iter()
            .any(|c| helpers::is_weapon_card(ctx.catalog, ctx.core, c));
        if has_weapon {
            set_flag(ctx.core, owner, "l11b_jiefan_idx", idx as u32);
            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWJIEFAN_WEAPON);
        }
        if ctx.core.player(t).alive {
            ctx.start_draw(t, 1);
        }
        idx += 1;
    }
    Flow::Done
}

fn finish_twyuanhu_draw(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) {
    if ctx.core.player(owner).alive
        && ctx.core.player(target).alive
        && (ctx.core.player(target).hp <= ctx.core.player(owner).hp
            || ctx.core.player(target).hand.len() <= ctx.core.player(owner).hand.len())
    {
        ctx.start_draw(owner, 1);
        if ctx.core.turn.phase == crate::event::Phase::Play {
            set_flag(ctx.core, owner, "l11b_twyuanhu_end_turn", turn_stamp(ctx.core));
        }
    }
}

pub(super) fn respond_skill_options_l11_npc_b(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    if !is_unblocked_npc_b(core, catalog, player) {
        return;
    }
    if matches!(need, CardKind::Sha | CardKind::Shan) {
        if let Some(def) = catalog.skill_by_name("weijing") {
            if core.has_skill(player, def)
                && flag(core, player, "l11b_weijing_round") != core.turn.round
            {
                out.push(ChoiceOption::Skill(def));
            }
        }
    }
    if is_use && matches!(need, CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu) {
        if let Some(def) = catalog.skill_by_name("twmouli") {
            if core.has_skill(player, def)
                && flag(core, player, "l11b_twmouli_turn") != turn_stamp(core)
                && core.deck.iter().any(|&c| {
                    let k = catalog.card_kind(c);
                    k == need || (need == CardKind::Sha && k.is_sha_like())
                })
            {
                out.push(ChoiceOption::Skill(def));
            }
        }
    }
}

pub(super) fn try_activate_respond_skill_l11_npc_b(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    _fid: EventId,
    is_use: bool,
) -> Option<bool> {
    let name = ctx.catalog.skill_name(def);
    match name {
        "weijing" if matches!(need, CardKind::Sha | CardKind::Shan) => {
            if !is_unblocked_npc_b(ctx.core, ctx.catalog, player)
                || flag(ctx.core, player, "l11b_weijing_round") == ctx.core.turn.round
            {
                return Some(false);
            }
            set_flag(ctx.core, player, "l11b_weijing_round", ctx.core.turn.round);
            helpers::log_skill(ctx, player, "weijing");
            Some(true)
        }
        "twmouli"
            if is_use
                && matches!(
                    need,
                    CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                ) =>
        {
            if !is_unblocked_npc_b(ctx.core, ctx.catalog, player)
                || flag(ctx.core, player, "l11b_twmouli_turn") == turn_stamp(ctx.core)
            {
                return Some(false);
            }
            let pos = ctx.core.deck.iter().rposition(|&c| {
                let k = ctx.catalog.card_kind(c);
                k == need || (need == CardKind::Sha && k.is_sha_like())
            });
            let Some(idx) = pos else {
                return Some(false);
            };
            let cid = ctx.core.deck.remove(idx);
            ctx.core.table.push(cid);
            set_flag(ctx.core, player, "l11b_twmouli_turn", turn_stamp(ctx.core));
            set_flag(ctx.core, player, "u01_respond_card_override", cid + 1);
            helpers::log_skill(ctx, player, "twmouli");
            Some(true)
        }
        _ => None,
    }
}
