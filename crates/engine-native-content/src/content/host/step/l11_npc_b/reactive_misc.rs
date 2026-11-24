//! Reactive triggers (lose/move, turnover, dying/death, reactive damage/use) for `L11` Part B (`reactive_misc.rs`).

use super::super::l11_common::{
    alive_all, alive_others, emit_lose_hp, has_named_skill, he_cards, move_cards_to_hand,
};
use super::super::l11_common_b2::is_ordinary_trick_kind;
use super::helpers::{
    clear_ask, flag, grant_npc_b_skill, is_locked_unblocked_npc_b, is_red_suit,
    is_unblocked_npc_b, phase_stamp, pile_cards, push_to_pile, set_flag, take_from_pile,
    take_owned_cards_from_zones, turn_stamp, wait_ask,
};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::event::{DamageWindowKind, EventData, Frame, Phase};
use crate::ids::{CardId, PlayerId};
use crate::kernel::sha_user_or_target_blocked;
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::eval_card_target_canceled;
use crate::state::GameStateCore;

fn frame_damage<'a>(frames: &'a [Frame], root: EventId) -> Option<&'a crate::event::DamageData> {
    frames.iter().rev().find_map(|f| {
        if f.id == root {
            match &f.data {
                EventData::Damage(d) => Some(d),
                _ => None,
            }
        } else {
            None
        }
    })
}

pub(super) fn current_damage_struct(ctx: &Ctx<'_>) -> Option<crate::event::DamageData> {
    ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::Damage(d) => Some(d.clone()),
        _ => None,
    })
}

fn frame_loss_deduped(
    state: &GameStateCore,
    frame: Option<&Frame>,
    player: PlayerId,
    cards: &[CardId],
) -> bool {
    let Some(f) = frame else {
        return false;
    };
    !cards.is_empty()
        && cards.iter().all(|&c| {
            state
                .vm_aux
                .flags
                .contains_key(&(player, format!("l11_loss_dedup:{}:{c}", f.id)))
        })
}

pub(super) fn owner_has_sha_use_card(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    core.player(owner).hand.iter().any(|&c| {
        catalog.resolved_card_kind(core, c).is_sha_like()
            || catalog
                .find_player_vm_view_as(core, owner, c, true, Some(CardKind::Sha))
                .is_some()
    })
}

fn can_move_any_field(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    alive_all(core, owner).into_iter().any(|src| {
        core.player(src)
            .equips
            .iter()
            .chain(core.player(src).judge.iter())
            .any(|&c| {
                alive_all(core, owner)
                    .into_iter()
                    .any(|dst| dst != src && super::helpers::can_move_card_to(core, catalog, c, dst))
            })
    })
}

pub(super) fn filter_reactive_misc(
    skill: &HostSkillBehavior,
    ctx: &crate::catalog::QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let tstamp = turn_stamp(state);
    let pstamp = phase_stamp(state);
    match (skill.name.as_str(), event) {
        ("baonvezhi", EventData::DamageWindow { root, window: DamageWindowKind::End | DamageWindowKind::Source }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && (d.target == owner || d.source == Some(owner))
                && flag(state, owner, "baonvezhi") < 5
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twjuntun", EventData::TurnBegin { .. }) => {
            flag(state, owner, "l11b_twjuntun_init") == 0
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !has_named_skill(state, catalog, p, "twxiongjun"))
        }
        ("twjuntun", EventData::Death { .. }) => {
            is_unblocked_npc_b(state, catalog, owner)
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !has_named_skill(state, catalog, p, "twxiongjun"))
        }
        ("twjuntun", EventData::DamageWindow { root, window: DamageWindowKind::Source }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.source.is_some_and(|s| s != owner && has_named_skill(state, catalog, s, "twxiongjun"))
                && flag(state, owner, "baonvezhi") < 5
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twxiongjun", EventData::DamageWindow { root, window: DamageWindowKind::Source }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.source == Some(owner)
                && flag(state, owner, "l11b_twxiongjun_turn") != tstamp
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("shunshi", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.target == owner
                && state.turn.current != owner
                && !he_cards(state, owner).is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| Some(p) != d.source)
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("shunshi", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && !he_cards(state, owner).is_empty()
                && !alive_others(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("tuifeng", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.target == owner
                && !he_cards(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("tuifeng", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && !pile_cards(state, owner, "l11b_tuifeng").is_empty()
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twliancai", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twliancai", EventData::TurnOvered { player, .. }) => {
            *player == owner
                && (state.player(owner).hand.len() as i32) < state.player(owner).hp
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twwuhun", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.target == owner
                && d.source.is_some_and(|s| s != owner && state.player(s).alive)
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twwuhun", EventData::DamageWindow { root, window: DamageWindowKind::Source }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.source == Some(owner)
                && d.target != owner
                && state.player(d.target).alive
                && flag(state, d.target, "l11b_twwuhun_mark") > 0
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twwuhun", EventData::Death { player, .. }) => {
            *player == owner
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| flag(state, p, "l11b_twwuhun_mark") > 0)
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twwushen", EventData::CardUsed(u)) => {
            let is_sha = u.effective_kind(catalog).is_sha_like();
            let is_heart = u.card.is_some_and(|c| {
                (c as usize) < catalog.cards.len()
                    && catalog.effective_card_suit(state, owner, c) == crate::catalog::Suit::Heart
            });
            !u.cancelled
                && u.player == owner
                && is_sha
                && is_locked_unblocked_npc_b(state, catalog, owner)
                && (is_heart || flag(state, owner, "l11b_twwushen_phase") != pstamp)
        }
        ("twjiefan", EventData::Dying { player, .. }) => {
            flag(state, owner, "l11b_twjiefan2_target") == u32::from(*player) + 1
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twxiangyu", EventData::CardsLost { player, cards, .. }) => {
            state.turn.current == owner
                && !cards.is_empty()
                && !frame_loss_deduped(state, frame, *player, cards)
                && is_locked_unblocked_npc_b(state, catalog, owner)
                && {
                    let m = if flag(state, owner, "l11b_twxiangyu_stamp") == tstamp {
                        flag(state, owner, "l11b_twxiangyu_mask")
                    } else {
                        0
                    };
                    (m & (1u32 << *player)) == 0
                }
        }
        ("twxiangyu", EventData::CardsMoved { from, cards, .. }) => {
            state.turn.current == owner
                && !cards.is_empty()
                && !frame_loss_deduped(state, frame, *from, cards)
                && is_locked_unblocked_npc_b(state, catalog, owner)
                && {
                    let m = if flag(state, owner, "l11b_twxiangyu_stamp") == tstamp {
                        flag(state, owner, "l11b_twxiangyu_mask")
                    } else {
                        0
                    };
                    (m & (1u32 << *from)) == 0
                }
        }
        ("twyuhua", EventData::CardsLost { player, cards, .. }) => {
            *player == owner
                && state.turn.current != owner
                && !frame_loss_deduped(state, frame, owner, cards)
                && is_unblocked_npc_b(state, catalog, owner)
                && cards
                    .iter()
                    .any(|&c| !catalog.resolved_card_kind(state, c).is_basic_kind())
        }
        ("twyuhua", EventData::CardsMoved { from, to, cards }) => {
            *from == owner
                && *to != owner
                && state.turn.current != owner
                && !frame_loss_deduped(state, frame, owner, cards)
                && is_unblocked_npc_b(state, catalog, owner)
                && cards
                    .iter()
                    .any(|&c| !catalog.resolved_card_kind(state, c).is_basic_kind())
        }
        ("meiyingqiang", EventData::CardsLost { player, cards, .. }) => {
            *player == owner
                && state.turn.current != owner
                && !cards.is_empty()
                && !frame_loss_deduped(state, frame, owner, cards)
                && flag(state, owner, "l11b_meiyingqiang_lost_turn") != tstamp
        }
        ("meiyingqiang", EventData::CardsMoved { from, cards, .. }) => {
            *from == owner
                && state.turn.current != owner
                && !cards.is_empty()
                && !frame_loss_deduped(state, frame, owner, cards)
                && flag(state, owner, "l11b_meiyingqiang_lost_turn") != tstamp
        }
        ("xiehui", EventData::CardsMoved { cards, from, to }) => {
            *from == owner
                && *to != owner
                && state.player(*to).alive
                && is_locked_unblocked_npc_b(state, catalog, owner)
                && cards
                    .iter()
                    .any(|&c| !is_red_suit(catalog.effective_card_suit(state, *to, c)))
        }
        ("twyilie", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twyilie", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && flag(state, owner, "l11b_twyilie_opt2_turn") == tstamp
                && u.effective_kind(catalog).is_sha_like()
                && u.targets.iter().any(|&t| state.player(t).linked)
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twyilie", EventData::ShaDodged { source, .. }) => {
            *source == owner
                && flag(state, owner, "l11b_twyilie_opt2_turn") == tstamp
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("weifeng", EventData::DamageWindow { root, window: DamageWindowKind::Begin3 }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            d.target != owner
                && !d.prevented
                && d.amount > 0
                && flag(state, d.target, "l11b_weifeng_kind") > 0
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("wulie", EventData::DamageWindow { root, window: DamageWindowKind::Begin3 }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && flag(state, owner, "l11b_wulie2_marks") > 0
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("xinfu_shajue", EventData::Dying { player, .. }) => {
            *player != owner
                && is_locked_unblocked_npc_b(state, catalog, owner)
                && (flag(state, owner, "xinfu_xionghuo_marks") < 3
                    || state.player(*player).hp < 0)
        }
        ("xinjiewei", EventData::TurnOvered { player, face_down }) => {
            *player == owner
                && !*face_down
                && !he_cards(state, owner).is_empty()
                && can_move_any_field(state, catalog, owner)
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("xinsheng", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && d.target == owner
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("yechou", EventData::Death { player, .. }) => {
            *player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| (state.player(p).max_hp as i32 - state.player(p).hp) > 1)
        }
        ("yinbing", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && he_cards(state, owner)
                    .into_iter()
                    .any(|c| !catalog.resolved_card_kind(state, c).is_basic_kind())
        }
        ("yinbing", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            let kind = frames
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::CardUsed(u) => Some(u.effective_kind(catalog)),
                    _ => None,
                })
                .or_else(|| d.card.map(|c| catalog.resolved_card_kind(state, c)));
            !d.prevented
                && d.target == owner
                && !pile_cards(state, owner, "l11b_yinbing").is_empty()
                && kind.is_some_and(|k| k.is_sha_like() || k == CardKind::Duel)
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("yuce", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.target == owner
                && !state.player(owner).hand.is_empty()
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zhichi", EventData::DamageWindow { root, window: DamageWindowKind::End }) => {
            let Some(d) = frame_damage(frames, *root) else {
                return false;
            };
            !d.prevented
                && d.target == owner
                && state.turn.current != owner
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("zhichi", EventData::CardUsed(u)) => {
            let kind = u.effective_kind(catalog);
            !u.cancelled
                && u.targets.contains(&owner)
                && flag(state, owner, "l11b_zhichi2") > 0
                && (kind.is_sha_like() || is_ordinary_trick_kind(kind))
                && is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("zhuide", EventData::Death { player, .. }) => {
            *player == owner
                && !alive_others(state, owner).is_empty()
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zhanyi", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && u.effective_kind(catalog).is_sha_like()
                && flag(state, owner, "l11b_zhanyi_equip_stamp") == pstamp
                && u.targets
                    .iter()
                    .any(|&t| state.player(t).alive && !he_cards(state, t).is_empty())
                && is_unblocked_npc_b(state, catalog, owner)
        }
        _ => false,
    }
}

pub(super) fn step_reactive_misc(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    let pstamp = phase_stamp(ctx.core);
    match m.def_key.as_str() {
        "baonvezhi" => {
            if let Some(d) = current_damage_struct(ctx) {
                let cur = flag(ctx.core, owner, "baonvezhi");
                let next = (cur + d.amount.max(0) as u32).min(5);
                if next > cur {
                    set_flag(ctx.core, owner, "baonvezhi", next);
                }
            }
            Flow::Done
        }
        "twjuntun" => {
            if m.stage == HostStage::Act && m.cards.len() == 3 {
                if let Some(d) = current_damage_struct(ctx) {
                    let cur = flag(ctx.core, owner, "baonvezhi");
                    let next = (cur + d.amount.max(0) as u32).min(5);
                    if next > cur {
                        helpers::log_skill(ctx, owner, "twjuntun");
                        set_flag(ctx.core, owner, "baonvezhi", next);
                    }
                }
                return Flow::Done;
            }
            match m.stage {
                HostStage::Act => {
                    if ctx.res.stack.iter().rev().any(|f| matches!(f.data, EventData::TurnBegin { .. })) {
                        set_flag(ctx.core, owner, "l11b_twjuntun_init", 1);
                    }
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWJUNTUN_TARGET)
                }
                HostStage::TargetChoice => {
                    clear_ask(ctx, owner);
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        helpers::log_skill(ctx, owner, "twjuntun");
                        grant_npc_b_skill(ctx, t, "twxiongjun");
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "twxiongjun" => {
            if flag(ctx.core, owner, "l11b_twxiongjun_turn") != tstamp {
                set_flag(ctx.core, owner, "l11b_twxiongjun_turn", tstamp);
                helpers::log_skill(ctx, owner, "twxiongjun");
                for &p in &alive_all(ctx.core, owner) {
                    if has_named_skill(ctx.core, ctx.catalog, p, "twxiongjun") {
                        ctx.start_draw(p, 1);
                    }
                }
            }
            Flow::Done
        }
        "shunshi" => match m.stage {
            HostStage::Act => {
                let excl = if ctx.core.turn.phase == Phase::Prepare && ctx.core.turn.current == owner
                {
                    u32::MAX
                } else {
                    current_damage_struct(ctx)
                        .and_then(|d| d.source)
                        .map_or(u32::MAX, u32::from)
                };
                m.card = Some(excl);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SHUNSHI_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards = vec![c];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHUNSHI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if let Some(&c) = m.cards.first() {
                    helpers::log_skill(ctx, owner, "shunshi");
                    move_cards_to_hand(ctx, owner, t, &[c]);
                    let st = flag(ctx.core, owner, "l11b_shunshi_stacks");
                    set_flag(ctx.core, owner, "l11b_shunshi_stacks", st + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "tuifeng" => match m.stage {
            HostStage::Act => {
                if ctx.core.turn.phase == Phase::Prepare && ctx.core.turn.current == owner {
                    let pile = pile_cards(ctx.core, owner, "l11b_tuifeng");
                    let taken = take_from_pile(ctx.core, owner, "l11b_tuifeng", &pile);
                    let n = taken.len() as u32;
                    if n > 0 {
                        helpers::log_skill(ctx, owner, "tuifeng");
                        for tc in &taken {
                            ctx.to_discard(*tc);
                        }
                        ctx.emit(EventData::CardsDiscarded {
                            player: owner,
                            cards: taken,
                        });
                        set_flag(ctx.core, owner, "l11b_tuifeng_turn_stamp", tstamp);
                        set_flag(ctx.core, owner, "l11b_tuifeng_sha_bonus", n);
                        ctx.start_draw(owner, 2 * n);
                    }
                    return Flow::Done;
                }
                let amt = current_damage_struct(ctx).map_or(1, |d| d.amount.max(1) as u32);
                m.card = Some(amt);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TUIFENG_CARDS)
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
                    helpers::log_skill(ctx, owner, "tuifeng");
                    let moved = take_owned_cards_from_zones(ctx, owner, &cs);
                    push_to_pile(ctx.core, owner, "l11b_tuifeng", &moved);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twliancai" => match m.stage {
            HostStage::Act => {
                if ctx.core.turn.phase == Phase::End && ctx.core.turn.current == owner {
                    helpers::log_skill(ctx, owner, "twliancai");
                    ctx.turn_over(owner);
                    if alive_others(ctx.core, owner)
                        .into_iter()
                        .any(|p| !ctx.core.player(p).equips.is_empty())
                    {
                        return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWLIANCAI_TARGET);
                    }
                    return Flow::Done;
                }
                let need = (ctx.core.player(owner).hp as usize)
                    .saturating_sub(ctx.core.player(owner).hand.len());
                if need > 0 {
                    helpers::log_skill(ctx, owner, "twliancai");
                    ctx.start_draw(owner, need as u32);
                }
                Flow::Done
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TWLIANCAI_EQUIP)
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
        "twwuhun" => {
            if let Some(d) = current_damage_struct(ctx) {
                if d.target == owner {
                    if let Some(src) = d.source.filter(|&s| s != owner && ctx.core.player(s).alive) {
                        helpers::log_skill(ctx, owner, "twwuhun");
                        let cur = flag(ctx.core, src, "l11b_twwuhun_mark");
                        set_flag(ctx.core, src, "l11b_twwuhun_mark", cur + d.amount.max(0) as u32);
                    }
                } else if d.source == Some(owner) && ctx.core.player(d.target).alive {
                    helpers::log_skill(ctx, owner, "twwuhun");
                    let cur = flag(ctx.core, d.target, "l11b_twwuhun_mark");
                    set_flag(ctx.core, d.target, "l11b_twwuhun_mark", cur + 1);
                }
                return Flow::Done;
            }
            match m.stage {
                HostStage::Act => {
                    helpers::log_skill(ctx, owner, "twwuhun");
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::OrdinaryJudge {
                        owner,
                        invocation: u32::MAX,
                    });
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let j_card = flag(ctx.core, owner, "last_judge_card");
                    set_flag(ctx.core, owner, "last_judge_suit", 0);
                    set_flag(ctx.core, owner, "last_judge_rank", 0);
                    set_flag(ctx.core, owner, "last_judge_card", 0);
                    let j_kind = ctx.catalog.card_kind(j_card);
                    if j_kind == CardKind::Tao || j_kind == CardKind::Taoyuan {
                        return Flow::Done;
                    }
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWWUHUN_TARGETS)
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
                    for t in ts {
                        let mk = flag(ctx.core, t, "l11b_twwuhun_mark");
                        if mk > 0 && ctx.core.player(t).alive {
                            emit_lose_hp(ctx, t, mk);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "twwushen" => {
            let first = flag(ctx.core, owner, "l11b_twwushen_phase") != pstamp;
            let heart = m.card.is_some_and(|c| {
                (c as usize) < ctx.catalog.cards.len()
                    && ctx.catalog.effective_card_suit(ctx.core, owner, c) == crate::catalog::Suit::Heart
            });
            let mut logged = false;
            if first {
                set_flag(ctx.core, owner, "l11b_twwushen_phase", pstamp);
                set_flag(ctx.core, owner, "l11b_sha_direct_hit_cur", 1);
                helpers::log_skill(ctx, owner, "twwushen");
                logged = true;
            }
            if heart {
                let cid = m.card.unwrap_or(0);
                let mut added = false;
                for f in ctx.res.stack.iter_mut().rev() {
                    if let EventData::CardUsed(ref mut u) = f.data {
                        let extra: Vec<PlayerId> = alive_others(ctx.core, owner)
                            .into_iter()
                            .filter(|&p| {
                                !u.targets.contains(&p)
                                    && flag(ctx.core, p, "l11b_twwuhun_mark") > 0
                                    && !ctx.catalog.has_undist(ctx.core, p)
                                    && !sha_user_or_target_blocked(ctx.core, ctx.catalog, owner, p)
                                    && !eval_card_target_canceled(
                                        ctx.core,
                                        ctx.catalog,
                                        owner,
                                        p,
                                        cid,
                                        u.effective_kind(ctx.catalog),
                                        u.nature,
                                    )
                            })
                            .collect();
                        if !extra.is_empty() {
                            u.targets.extend(extra);
                            added = true;
                        }
                        break;
                    }
                }
                if added && !logged {
                    helpers::log_skill(ctx, owner, "twwushen");
                }
            }
            Flow::Done
        }
        "twxiangyu" => {
            if let Some(loser) = m.target {
                let cur = if flag(ctx.core, owner, "l11b_twxiangyu_stamp") == tstamp {
                    flag(ctx.core, owner, "l11b_twxiangyu_mask")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l11b_twxiangyu_stamp", tstamp);
                set_flag(ctx.core, owner, "l11b_twxiangyu_mask", cur | (1u32 << loser));
            }
            Flow::Done
        }
        "twyuhua" => match m.stage {
            HostStage::Act => {
                let n = m
                    .cards
                    .iter()
                    .filter(|&&c| !ctx.catalog.resolved_card_kind(ctx.core, c).is_basic_kind())
                    .count()
                    .min(5);
                if n == 0 || ctx.core.deck.is_empty() {
                    return Flow::Done;
                }
                let take = n.min(ctx.core.deck.len());
                let mut top = Vec::with_capacity(take);
                for _ in 0..take {
                    if let Some(c) = ctx.core.deck.pop() {
                        top.push(c);
                    }
                }
                helpers::log_skill(ctx, owner, "twyuhua");
                m.cards = top;
                m.card = Some(take as u32);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TWYUHUA_GUANXING)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let top_sel: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if m.cards.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                let mut bottom: Vec<CardId> = m
                    .cards
                    .iter()
                    .copied()
                    .filter(|c| !top_sel.contains(c))
                    .collect();
                if !bottom.is_empty() {
                    bottom.splice(0..0, Vec::<CardId>::new());
                    let old = std::mem::replace(&mut ctx.core.deck, bottom);
                    ctx.core.deck.extend(old);
                }
                ctx.core.deck.extend(top_sel.into_iter().rev());
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWYUHUA_DRAW)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let n = m.card.unwrap_or(0);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) && n > 0 {
                    ctx.start_draw(owner, n);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => super::reactive_step_b::step_reactive_step_b(m, ctx),
    }
}
