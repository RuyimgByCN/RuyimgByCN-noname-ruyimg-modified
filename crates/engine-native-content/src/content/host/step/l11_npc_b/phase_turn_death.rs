//! Turn-begin, phase-begin, phase-end, & round-parked Part B skills (`phase_turn_death.rs`).

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_damage, he_cards, move_cards_to_hand,
};
use super::helpers::{
    card_type_group, card_use_virtual, clear_ask, emit_virtual_sha, equip_card_onto_player, flag,
    gain_hand_cards_from_pile_or_deck, grant_npc_b_skill, is_locked_unblocked_npc_b, is_red_suit,
    is_unblocked_npc_b, phase_stamp, pile_cards, pop_card_from_deck_bottom_or_discard,
    pop_detached_or_field_card, push_to_pile, set_flag, take_from_pile,
    take_owned_cards_from_zones, turn_stamp, usable_sha_targets_no_range, wait_ask,
};
use super::*;
use crate::catalog::{CardKind, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};

pub(super) fn filter_phase_turn_death(
    skill: &HostSkillBehavior,
    ctx: &crate::catalog::QueryCtx<'_>,
    _frames: &[crate::event::Frame],
    _frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let tstamp = turn_stamp(state);
    match (skill.name.as_str(), event) {
        ("twhanyu", EventData::TurnBegin { .. }) => {
            flag(state, owner, "l11b_twhanyu_init") == 0
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twqueshi", EventData::TurnBegin { .. }) => {
            flag(state, owner, "l11b_twqueshi_init") == 0
                && is_unblocked_npc_b(state, catalog, owner)
                && crate::rvm_ops::has_enabled_equip_slot(state, owner, EquipSlot::Weapon)
        }
        ("xianfu", EventData::TurnBegin { .. }) => {
            flag(state, owner, "l11b_xianfu_init") == 0
                && !alive_others(state, owner).is_empty()
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("zongkui", EventData::TurnBegin { .. }) => {
            flag(state, owner, "l11b_zongkui_init") == 0
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| flag(state, p, "l11_kui_mark") == 0)
        }
        ("shensu", EventData::Phase { player, phase: Phase::Judge, begin: true }) => {
            *player == owner
                && flag(state, owner, "skip_judge") == 0
                && is_unblocked_npc_b(state, catalog, owner)
                && !usable_sha_targets_no_range(state, catalog, owner).is_empty()
        }
        ("shensu", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && !state.turn.skip_play_phase
                && is_unblocked_npc_b(state, catalog, owner)
                && he_cards(state, owner)
                    .into_iter()
                    .any(|c| catalog.card_kind(c).equip_slot().is_some())
                && !usable_sha_targets_no_range(state, catalog, owner).is_empty()
        }
        ("juedi", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && !pile_cards(state, owner, "l11b_yinbing").is_empty()
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twfenming", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !he_cards(state, p).is_empty() || !state.player(p).linked)
        }
        ("twhongju", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_twhongju_awakened") == 0
                && pile_cards(state, owner, "l11b_twzhengrong").len() >= 3
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twqianxi", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twqianxi", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_twqianxi_watch_turn") == tstamp
                && flag(state, owner, "l11b_twqianxi_watch_target") > 0
                && {
                    let tgt = (flag(state, owner, "l11b_twqianxi_watch_target") - 1) as PlayerId;
                    state.player(tgt).alive
                        && flag(state, owner, &format!("l11b_twqianxi_sha_dmg_to:{tgt}")) == tstamp
                }
        }
        ("twxingqi", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            if *player != owner
                || flag(state, owner, "l11b_twxingqi_awakened") > 0
                || !is_locked_unblocked_npc_b(state, catalog, owner)
            {
                return false;
            }
            let total_ej: i32 = alive_all(state, owner)
                .into_iter()
                .map(|p| (state.player(p).equips.len() + state.player(p).judge.len()) as i32)
                .sum();
            total_ej > state.player(owner).hp
        }
        ("wangzun", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player != owner
                && flag(state, owner, "l11b_wangzun_active") == 0
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zili", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_zili_awakened") == 0
                && pile_cards(state, owner, "l11b_quanji").len() >= 3
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("spwenji", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !he_cards(state, p).is_empty())
        }
        ("tianshu", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && !he_cards(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_all(state, owner).into_iter().all(|p| {
                    !state
                        .player(p)
                        .equips
                        .iter()
                        .chain(state.player(p).judge.iter())
                        .any(|&c| {
                            catalog
                                .cards
                                .get(c as usize)
                                .is_some_and(|cd| cd.name == "taipingyaoshu")
                        })
                })
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| crate::rvm_ops::has_enabled_equip_slot(state, p, EquipSlot::Armor))
        }
        ("twlinglu", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && !alive_others(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zhendu", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            state.player(*player).alive
                && !state.player(owner).hand.is_empty()
                && !state.turn.jiu_used
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("yongsi", EventData::Phase { player, phase: Phase::Discard, begin: true }) => {
            *player == owner
                && !he_cards(state, owner).is_empty()
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("shefu", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && !he_cards(state, owner).is_empty()
                && flag(state, owner, "l11b_shefu_name_mask") != ((1u32 << helpers::SHEFU_NAMES.len()) - 1)
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("shefu", EventData::CardUsed(u)) => {
            let eff = u.effective_kind(catalog);
            let tag = helpers::shefu_kind_to_tag(eff);
            !u.cancelled
                && u.player != state.turn.current
                && u.player != owner
                && (flag(state, owner, "l11b_shefu_name_mask") & (1u32 << tag)) != 0
                && helpers::shefu_tag_to_kind(tag) == Some(eff)
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("sptunjiang", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_play_skipped_turn") != tstamp
                && flag(state, owner, "l11b_dmg_dealt_turn") != tstamp
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twyuanhu", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_twyuanhu_end_turn") == tstamp
                && is_unblocked_npc_b(state, catalog, owner)
                && he_cards(state, owner).into_iter().any(|c| {
                    catalog.card_slot_of(state, c).is_some_and(|slot| {
                        alive_all(state, owner)
                            .into_iter()
                            .any(|p| crate::rvm_ops::has_enabled_equip_slot(state, p, slot))
                    })
                })
        }
        ("wulie", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && flag(state, owner, "l11b_limited_wulie") == 0
                && state.player(owner).hp > 0
                && !alive_others(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("xinjushou", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zhiyan", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twxuanhuo", EventData::Phase { player, phase: Phase::Draw, begin: false }) => {
            *player == owner
                && he_cards(state, owner).len() >= 2
                && alive_all(state, owner).len() > 2
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twyingjia", EventData::Phase { phase: Phase::End, begin: false, .. }) => {
            !he_cards(state, owner).is_empty()
                && flag(state, owner, "l11b_yingjia_ready_turn") == tstamp
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("xinfu_zhaoxin", EventData::Phase { player, phase: Phase::Draw, begin: false }) => {
            state.player(*player).alive
                && !pile_cards(state, owner, "l11b_zhaoxin").is_empty()
                && (*player == owner || state.in_attack_range(catalog, owner, *player))
                && is_unblocked_npc_b(state, catalog, owner)
        }
        _ => false,
    }
}

pub(super) fn step_phase_turn_death(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    let _pstamp = phase_stamp(ctx.core);
    match m.def_key.as_str() {
        "twhanyu" => {
            set_flag(ctx.core, owner, "l11b_twhanyu_init", 1);
            helpers::log_skill(ctx, owner, "twhanyu");
            let mut gained = Vec::new();
            for grp in 0u8..=2u8 {
                if let Some(cid) = pop_card_from_deck_bottom_or_discard(ctx.core, |c| {
                    card_type_group(ctx.catalog.card_kind(c)) == grp
                }) {
                    gained.push(cid);
                }
            }
            gain_hand_cards_from_pile_or_deck(ctx, owner, gained);
            Flow::Done
        }
        "twqueshi" => {
            set_flag(ctx.core, owner, "l11b_twqueshi_init", 1);
            if crate::rvm_ops::has_enabled_equip_slot(ctx.core, owner, EquipSlot::Weapon) {
                if let Some(cid) = pop_detached_or_field_card(ctx, "meiyingqiang", Some(owner)) {
                    helpers::log_skill(ctx, owner, "twqueshi");
                    let _ = equip_card_onto_player(ctx, owner, cid);
                }
            }
            Flow::Done
        }
        "xianfu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIANFU_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                set_flag(ctx.core, owner, "l11b_xianfu_init", 1);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "xianfu");
                    set_flag(ctx.core, owner, "l11b_xianfu_target", u32::from(t) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zongkui" => match m.stage {
            HostStage::Act => {
                if m.card != Some(1) {
                    set_flag(ctx.core, owner, "l11b_zongkui_init", 1);
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZONGKUI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "zongkui");
                    set_flag(ctx.core, t, "l11_kui_mark", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shensu" => match m.stage {
            HostStage::Act => {
                if m.card == Some(Phase::Judge as u32) {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHENSU1_TARGET);
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SHENSU2_CARD)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                helpers::log_skill(ctx, owner, "shensu");
                set_flag(ctx.core, owner, "skip_judge", 1);
                ctx.core.turn.skip_draw_phase = true;
                emit_virtual_sha(ctx, owner, vec![t], true);
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_SHENSU2_TARGET)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cid = m.card.unwrap_or(0);
                helpers::log_skill(ctx, owner, "shensu");
                discard_player_cards(ctx, owner, &[cid]);
                ctx.core.turn.skip_play_phase = true;
                emit_virtual_sha(ctx, owner, vec![t], true);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "juedi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_JUEDI_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let pile = pile_cards(ctx.core, owner, "l11b_yinbing");
                let taken = take_from_pile(ctx.core, owner, "l11b_yinbing", &pile);
                if taken.is_empty() {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "juedi");
                if t == owner {
                    for tc in &taken {
                        ctx.to_discard(*tc);
                    }
                    ctx.emit(EventData::CardsDiscarded {
                        player: owner,
                        cards: taken,
                    });
                    let need = (ctx.core.player(owner).max_hp as usize)
                        .saturating_sub(ctx.core.player(owner).hand.len());
                    if need > 0 {
                        ctx.start_draw(owner, need as u32);
                    }
                } else if ctx.core.player(t).alive {
                    let n = taken.len() as u32;
                    if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                        ctx.recover(t, 1, None);
                    }
                    gain_hand_cards_from_pile_or_deck(ctx, t, taken);
                    ctx.start_draw(t, n);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twfenming" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWFENMING_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_TWFENMING_MODE)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                m.card = Some(u32::from(mode));
                helpers::log_skill(ctx, owner, "twfenming");
                if mode == 2 && !ctx.core.player(owner).linked {
                    ctx.core.player_mut(owner).linked = true;
                    ctx.emit(EventData::Linked {
                        player: owner,
                        linked: true,
                    });
                }
                if mode != 1 && !he_cards(ctx.core, t).is_empty() {
                    return wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWFENMING_DISCARD);
                }
                if mode != 0 && ctx.core.player(t).alive && !ctx.core.player(t).linked {
                    ctx.core.player_mut(t).linked = true;
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: true,
                    });
                }
                Flow::Done
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let mode = m.card.unwrap_or(0);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[c]);
                }
                if mode != 0 && ctx.core.player(t).alive && !ctx.core.player(t).linked {
                    ctx.core.player_mut(t).linked = true;
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: true,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twhongju" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11b_twhongju_awakened", 1);
                helpers::log_skill(ctx, owner, "twhongju");
                let n = pile_cards(ctx.core, owner, "l11b_twzhengrong").len() as u32;
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, n);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive && !ctx.core.player(owner).hand.is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_TWHONGJU_SWAP_HAND);
                }
                grant_npc_b_skill(ctx, owner, "twqingce");
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWHONGJU_SAOTAO)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if ctx.core.player(owner).hand.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if cs.is_empty() {
                    grant_npc_b_skill(ctx, owner, "twqingce");
                    return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWHONGJU_SAOTAO);
                }
                m.cards = cs;
                m.card = Some(m.cards.len() as u32);
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_TWHONGJU_SWAP_PILE)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let pile_pick: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if pile_pick.len() == m.cards.len() && !pile_pick.is_empty() {
                    let from_hand = take_owned_cards_from_zones(ctx, owner, &m.cards);
                    let from_pile = take_from_pile(ctx.core, owner, "l11b_twzhengrong", &pile_pick);
                    push_to_pile(ctx.core, owner, "l11b_twzhengrong", &from_hand);
                    gain_hand_cards_from_pile_or_deck(ctx, owner, from_pile);
                }
                grant_npc_b_skill(ctx, owner, "twqingce");
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWHONGJU_SAOTAO)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(owner).alive
                {
                    ctx.lose_max_hp(owner, 1);
                    if ctx.core.player(owner).alive {
                        grant_npc_b_skill(ctx, owner, "twsaotao");
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twqianxi" => match m.stage {
            HostStage::Act => {
                if ctx.core.turn.phase == Phase::End {
                    let wt = flag(ctx.core, owner, "l11b_twqianxi_watch_target");
                    set_flag(ctx.core, owner, "l11b_twqianxi_watch_turn", 0);
                    set_flag(ctx.core, owner, "l11b_twqianxi_watch_target", 0);
                    if wt > 0 {
                        let t = (wt - 1) as PlayerId;
                        let clr = flag(ctx.core, t, "l11b_twqianxi2_color");
                        if clr > 0 && ctx.core.player(t).alive {
                            helpers::log_skill(ctx, owner, "twqianxi");
                            set_flag(ctx.core, t, "l11b_twqianxi3_blocked_color", clr);
                        }
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWQIANXI_CONFIRM)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "twqianxi");
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if he_cards(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TWQIANXI_DISCARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let color = if is_red_suit(ctx.catalog.effective_card_suit(ctx.core, owner, c)) {
                    1
                } else {
                    2
                };
                discard_player_cards(ctx, owner, &[c]);
                m.card = Some(color);
                if alive_others(ctx.core, owner)
                    .into_iter()
                    .any(|p| ctx.core.computed_distance(ctx.catalog, owner, p) <= 1)
                {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWQIANXI_TARGET);
                }
                Flow::Done
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let color = m.card.unwrap_or(1);
                    set_flag(ctx.core, t, "l11b_twqianxi2_stamp", tstamp);
                    set_flag(ctx.core, t, "l11b_twqianxi2_color", color);
                    set_flag(ctx.core, owner, "l11b_twqianxi_watch_turn", tstamp);
                    set_flag(ctx.core, owner, "l11b_twqianxi_watch_target", u32::from(t) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twxingqi" => {
            set_flag(ctx.core, owner, "l11b_twxingqi_awakened", 1);
            helpers::log_skill(ctx, owner, "twxingqi");
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                ctx.recover(owner, 1, None);
            }
            if flag(ctx.core, owner, "l11b_twmibei_done") == 0 {
                let mut gained = Vec::new();
                for grp in [0u8, 2u8, 1u8] {
                    if let Some(cid) = pop_card_from_deck_bottom_or_discard(ctx.core, |c| {
                        card_type_group(ctx.catalog.card_kind(c)) == grp
                    }) {
                        gained.push(cid);
                    }
                }
                gain_hand_cards_from_pile_or_deck(ctx, owner, gained);
            } else {
                set_flag(ctx.core, owner, "l11b_twxingqi_range", 1);
            }
            Flow::Done
        }
        "wangzun" => {
            let t = m.target.unwrap_or(owner);
            helpers::log_skill(ctx, owner, "wangzun");
            set_flag(ctx.core, owner, "l11b_wangzun_active", 1);
            set_flag(ctx.core, t, "l11b_wangzun_less", 1);
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "zili" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_ZILI_MODE),
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                set_flag(ctx.core, owner, "l11b_zili_awakened", 1);
                helpers::log_skill(ctx, owner, "zili");
                let mode = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(t)) => t,
                    _ => 0,
                };
                if mode == 1 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                } else {
                    ctx.start_draw(owner, 2);
                }
                ctx.lose_max_hp(owner, 1);
                if ctx.core.player(owner).alive {
                    grant_npc_b_skill(ctx, owner, "paiyi");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "spwenji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_SPWENJI_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                helpers::log_skill(ctx, owner, "spwenji");
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SPWENJI_GIVE)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let kind = ctx.catalog.card_kind(c);
                    move_cards_to_hand(ctx, t, owner, &[c]);
                    set_flag(ctx.core, owner, "l11b_spwenji_stamp", tstamp);
                    set_flag(ctx.core, owner, "l11b_spwenji_kind", (kind as u32) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "tianshu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TIANSHU_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TIANSHU_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c = m.card.unwrap_or(0);
                helpers::log_skill(ctx, owner, "tianshu");
                discard_player_cards(ctx, owner, &[c]);
                if let Some(tpys) = pop_detached_or_field_card(ctx, "taipingyaoshu", None) {
                    let _ = equip_card_onto_player(ctx, t, tpys);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twlinglu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWLINGLU_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    helpers::log_skill(ctx, owner, "twlinglu");
                    set_flag(ctx.core, t, "l11b_linglu_active", 1);
                    set_flag(ctx.core, t, "l11b_linglu_dmg", 0);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhendu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHENDU_DISCARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                helpers::log_skill(ctx, owner, "zhendu");
                discard_player_cards(ctx, owner, &[c]);
                if ctx.core.player(t).alive && !ctx.core.turn.jiu_used {
                    ctx.emit(EventData::CardUsed(card_use_virtual(
                        t,
                        vec![t],
                        DamageNature::Normal,
                        CardKind::Jiu,
                    )));
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                if t != owner && ctx.core.player(t).alive {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yongsi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YONGSI2_DISCARD),
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
                    helpers::log_skill(ctx, owner, "yongsi");
                    discard_player_cards(ctx, owner, &cs);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => super::phase_step_b::step_phase_step_b(m, ctx),
    }
}
