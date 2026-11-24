//! Combat & damage trigger Part B skills (`combat_damage.rs`):
//! `satomi_luodao`, `sbliegong`, `spshiji`, `spxiangzhen`, `spyishi`, `tianming`,
//! `twhengjiang`, `twhuangjin`, `twjianming`, `twqirang`, `twxiangyu`, `twxuhe`,
//! `twxuechang` (DamageBegin1), `xinfu_kannan` (DamageBegin1),
//! `xinfu_daigong`, `xinfu_fuyin`, `xinfu_zengdao2`, `yise`, `zhidao`,
//! `zhiman`, `zhongyong`, `taipingyaoshu`.

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, he_cards, hej_cards, move_cards_to_hand,
    with_current_damage_mut,
};
use super::super::l11_common_b2::is_ordinary_trick_kind;
use super::helpers::{
    card_effective_rank_with_mods, clear_ask, damage_root_card_use_fid, flag, is_huogong_card,
    is_red_suit, is_unblocked_npc_b, nullify_current_card_on_owner, phase_stamp, pile_cards,
    set_flag, sha_dodged_shan_cards, suit_bit, turn_stamp, wait_ask,
};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, DamageWindowKind, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::d34_l11_mods_npc_b::{
    stack_active_card_use_fid, tpys_damage_prevented_npc_b,
};
use crate::rvm_bridge::modifiers::eval_card_target_canceled;
use crate::state::GameStateCore;

fn frame_damage_by_root<'a>(
    frames: &'a [crate::event::Frame],
    root: EventId,
) -> Option<&'a crate::event::DamageData> {
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

fn is_sha_kind(k: CardKind) -> bool {
    k.is_sha_like()
}

fn sbliegong_effective_mask(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &crate::event::CardUseData,
) -> u32 {
    let mut mask = flag(state, owner, "l11b_sbliegong_mask");
    if let Some(c) = u.card {
        if (c as usize) < catalog.cards.len() {
            mask |= suit_bit(catalog.effective_card_suit(state, u.player, c));
        }
    }
    mask
}

fn twhengjiang_valid_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> Vec<PlayerId> {
    let is_hg = is_huogong_card(catalog, card);
    if !(kind.is_sha_like()
        || is_hg
        || matches!(
            kind,
            CardKind::Snatch
                | CardKind::Dismantle
                | CardKind::Duel
                | CardKind::Chain
                | CardKind::Jiedao
        ))
    {
        return Vec::new();
    }
    alive_others(state, owner)
        .into_iter()
        .filter(|&t| {
            state.in_attack_range(catalog, owner, t)
                && !eval_card_target_canceled(state, catalog, owner, t, card, kind, nature)
                && !catalog.has_undist(state, t)
                && if is_hg {
                    !state.player(t).hand.is_empty()
                } else {
                    match kind {
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                            !crate::kernel::sha_user_or_target_blocked(state, catalog, owner, t)
                        }
                        CardKind::Snatch | CardKind::Dismantle => !hej_cards(state, t).is_empty(),
                        _ => true,
                    }
                }
        })
        .collect()
}

pub(super) fn twqirang_candidate_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &crate::event::CardUseData,
    kind: CardKind,
) -> Vec<PlayerId> {
    let is_hg = u.card.is_some_and(|c| is_huogong_card(catalog, c));
    if !is_hg
        && matches!(
            kind,
            CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Wuzhong
        )
    {
        return Vec::new();
    }
    let cid = u.card.unwrap_or(0);
    alive_all(state, owner)
        .into_iter()
        .filter(|&t| {
            if u.targets.contains(&t) {
                return true;
            }
            if eval_card_target_canceled(state, catalog, owner, t, cid, kind, u.nature) {
                return false;
            }
            if is_hg {
                return t != owner
                    && !state.player(t).hand.is_empty()
                    && !catalog.has_undist(state, t);
            }
            match kind {
                CardKind::Snatch | CardKind::Dismantle => {
                    t != owner && !hej_cards(state, t).is_empty() && !catalog.has_undist(state, t)
                }
                CardKind::Duel => t != owner && !catalog.has_undist(state, t),
                CardKind::Chain => true,
                CardKind::Le | CardKind::Bing => {
                    t != owner
                        && !catalog.blocks_delayed_trick(state, t)
                        && state
                            .player(t)
                            .judge
                            .iter()
                            .all(|&j| catalog.resolved_card_kind(state, j) != kind)
                }
                CardKind::Wugu => true,
                _ => false,
            }
        })
        .collect()
}

pub(super) fn current_damage_data(ctx: &Ctx<'_>) -> Option<crate::event::DamageData> {
    ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::Damage(d) => Some(d.clone()),
        _ => None,
    })
}

pub(super) fn current_card_use_data(ctx: &Ctx<'_>) -> Option<(EventId, crate::event::CardUseData)> {
    ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(u) => Some((f.id, u.clone())),
        _ => None,
    })
}

fn finish_satomi(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let mut idx = flag(ctx.core, owner, "l11b_satomi_idx") as usize;
    while idx < m.targets.len() {
        let t = m.targets[idx];
        if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
            idx += 1;
            continue;
        }
        set_flag(ctx.core, owner, "l11b_satomi_idx", idx as u32);
        m.target = Some(t);
        let has_shan = ctx
            .core
            .player(t)
            .hand
            .iter()
            .any(|&c| ctx.catalog.resolved_card_kind(ctx.core, c) == CardKind::Shan);
        if has_shan {
            return wait_ask(m, ctx, HostStage::SelectCard, ASK_SATOMI_DISCARD_SHAN);
        }
        if !he_cards(ctx.core, owner).is_empty() {
            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_SATOMI_DISCARD_SELF);
        }
        idx += 1;
    }
    set_flag(ctx.core, owner, "l11b_satomi_idx", 0);
    Flow::Done
}

pub(super) fn filter_combat_damage(
    skill: &HostSkillBehavior,
    ctx: &crate::catalog::QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let tstamp = turn_stamp(state);
    let pstamp = phase_stamp(state);
    match (skill.name.as_str(), event) {
        ("satomi_luodao", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && is_sha_kind(u.effective_kind(catalog))
                && u.targets.iter().any(|&t| state.player(t).alive && !state.player(t).hand.is_empty())
        }
        ("sbliegong", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && is_sha_kind(u.effective_kind(catalog))
                && u.targets.len() == 1
                && frame.is_some_and(|f| flag(state, owner, "l11b_sbliegong_done_fid") != f.id)
                && sbliegong_effective_mask(state, catalog, owner, u) != 0
        }
        ("sbliegong", EventData::DamageWindow { window: DamageWindowKind::Begin1, .. }) => {
            let Some(root_fid) = stack_active_card_use_fid(frames) else {
                return false;
            };
            flag(state, owner, &format!("l11b_sbliegong_dmg:{root_fid}")) > 0
        }
        ("spshiji", EventData::DamageWindow { root, window: DamageWindowKind::Begin2 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            is_unblocked_npc_b(state, catalog, owner)
                && d.source == Some(owner)
                && d.target != owner
                && d.nature != DamageNature::Normal
                && !state.player(d.target).hand.is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| state.player(p).hand.len() >= state.player(owner).hand.len())
        }
        ("spxiangzhen", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.targets.contains(&owner)
                && u.effective_kind(catalog) == CardKind::SavageAttack
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("spyishi", EventData::DamageWindow { root, window: DamageWindowKind::Begin2 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            is_unblocked_npc_b(state, catalog, owner)
                && d.source == Some(owner)
                && d.target != owner
                && !state.player(d.target).equips.is_empty()
        }
        ("tianming", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.targets.contains(&owner)
                && is_sha_kind(u.effective_kind(catalog))
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twhengjiang", EventData::CardUsed(u)) => {
            let kind = u.effective_kind(catalog);
            !u.cancelled
                && u.player == owner
                && state.turn.current == owner
                && state.turn.phase == Phase::Play
                && flag(state, owner, "l11b_twhengjiang_phase") != pstamp
                && u.targets.len() == 1
                && (kind.is_basic_kind() || is_ordinary_trick_kind(kind))
                && is_unblocked_npc_b(state, catalog, owner)
                && !twhengjiang_valid_targets(
                    state,
                    catalog,
                    owner,
                    u.card.unwrap_or(0),
                    kind,
                    u.nature,
                )
                .is_empty()
        }
        ("twhuangjin", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.targets.contains(&owner)
                && is_sha_kind(u.effective_kind(catalog))
                && u.card.is_some_and(|c| (c as usize) < catalog.cards.len())
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("twjianming", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && is_sha_kind(u.effective_kind(catalog))
                && is_unblocked_npc_b(state, catalog, owner)
                && u.card.is_some_and(|c| {
                    (c as usize) < catalog.cards.len() && {
                        let mask = if flag(state, owner, "l11b_twjianming_turn") == tstamp {
                            flag(state, owner, "l11b_twjianming_suit_mask")
                        } else {
                            0
                        };
                        (mask & suit_bit(catalog.effective_card_suit(state, owner, c))) == 0
                    }
                })
        }
        ("twjianming", EventData::CardResponded { player, card, .. }) => {
            *player == owner
                && catalog.resolved_card_kind(state, *card).is_sha_like()
                && (*card as usize) < catalog.cards.len()
                && is_unblocked_npc_b(state, catalog, owner)
                && {
                    let mask = if flag(state, owner, "l11b_twjianming_turn") == tstamp {
                        flag(state, owner, "l11b_twjianming_suit_mask")
                    } else {
                        0
                    };
                    (mask & suit_bit(catalog.effective_card_suit(state, owner, *card))) == 0
                }
        }
        ("twqirang", EventData::Equipped { player, .. }) => {
            *player == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && state.deck.iter().any(|&c| {
                    let k = catalog.card_kind(c);
                    !k.is_basic_kind() && k.equip_slot().is_none()
                })
        }
        ("twqirang", EventData::CardUsed(u)) => {
            let kind = u.effective_kind(catalog);
            !u.cancelled
                && u.player == owner
                && !kind.is_basic_kind()
                && kind.equip_slot().is_none()
                && u.card
                    .is_some_and(|c| flag(state, owner, &format!("l11b_twqirang_card:{c}")) > 0)
                && is_unblocked_npc_b(state, catalog, owner)
                && !twqirang_candidate_targets(state, catalog, owner, u, kind).is_empty()
        }
        ("twxiangyu", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && is_sha_kind(u.effective_kind(catalog))
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
                && {
                    let ar = state.attack_range(catalog, owner);
                    u.targets
                        .iter()
                        .any(|&t| state.computed_distance(catalog, owner, t) < ar)
                }
        }
        ("twxuhe", EventData::ShaDodged { source, target, .. }) => {
            *source == owner && state.player(*target).alive && is_unblocked_npc_b(state, catalog, owner)
        }
        ("twxuhe", EventData::DamageWindow { root, window: DamageWindowKind::Begin1 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.source == Some(owner)
                && flag(state, owner, "l11b_twxuhe_turn") == tstamp
                && flag(state, owner, "l11b_twxuhe_target") == u32::from(d.target) + 1
                && d.card.is_some()
        }
        ("twxuechang", EventData::DamageWindow { root, window: DamageWindowKind::Begin1 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.source == Some(owner)
                && flag(state, owner, &format!("l11b_xuechang_bonus:{}", d.target)) > 0
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("xinfu_kannan", EventData::DamageWindow { root, window: DamageWindowKind::Begin1 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            let Some(src) = d.source else {
                return false;
            };
            let root_fid = stack_active_card_use_fid(frames).unwrap_or(0);
            let is_sha = frames
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::CardUsed(u) if f.id == root_fid => {
                        Some(u.effective_kind(catalog).is_sha_like())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| d.card.is_some_and(|c| catalog.resolved_card_kind(state, c).is_sha_like()));
            is_sha
                && (flag(state, src, "l11b_kannan_eff") > 0
                    || (root_fid > 0 && flag(state, src, &format!("l11b_kannan_sha:{root_fid}")) > 0))
        }
        ("xinfu_daigong", EventData::DamageWindow { root, window: DamageWindowKind::Begin4 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.source.is_some_and(|s| state.player(s).alive)
                && !state.player(owner).hand.is_empty()
                && flag(state, owner, "l11b_daigong_turn") != tstamp
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("xinfu_fuyin", EventData::CardUsed(u)) => {
            let kind = u.effective_kind(catalog);
            !u.cancelled
                && u.targets.contains(&owner)
                && (is_sha_kind(kind) || kind == CardKind::Duel)
                && state.player(u.player).hand.len() >= state.player(owner).hand.len()
                && frame.is_some_and(|f| {
                    flag(state, owner, "l11b_fuyin_turn") != tstamp
                        || flag(state, owner, "l11b_fuyin_fid") == f.id
                })
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("xinfu_zengdao2", EventData::DamageWindow { root, window: DamageWindowKind::Begin1 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.source == Some(owner) && !pile_cards(state, owner, "l11b_zengdao").is_empty()
        }
        ("yise", EventData::CardsMoved { cards, from, to }) => {
            *from == owner
                && *to != owner
                && state.player(*to).alive
                && is_unblocked_npc_b(state, catalog, owner)
                && cards.iter().any(|&c| {
                    let is_r = is_red_suit(catalog.effective_card_suit(state, *to, c));
                    !is_r || state.player(*to).hp < state.player(*to).max_hp as i32
                })
        }
        ("yise", EventData::DamageWindow { root, window: DamageWindowKind::Begin1 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            let root_fid = stack_active_card_use_fid(frames).unwrap_or(0);
            let is_sha = frames
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::CardUsed(u) if f.id == root_fid => {
                        Some(u.effective_kind(catalog).is_sha_like())
                    }
                    _ => None,
                })
                .unwrap_or(false);
            is_sha && flag(state, d.target, "l11b_yise_sha_mark") > 0
        }
        ("zhidao", EventData::DamageWindow { root, window: DamageWindowKind::Source }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.source == Some(owner)
                && state.turn.current == owner
                && d.target != owner
                && state.player(d.target).alive
                && !hej_cards(state, d.target).is_empty()
                && flag(state, owner, "l11b_zhidao2_turn_stamp") != tstamp
                && super::helpers::is_locked_unblocked_npc_b(state, catalog, owner)
        }
        ("zhiman", EventData::DamageWindow { root, window: DamageWindowKind::Begin2 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && is_unblocked_npc_b(state, catalog, owner)
        }
        ("zhongyong", EventData::ShaDodged { source, target, .. }) => {
            *source == owner
                && is_unblocked_npc_b(state, catalog, owner)
                && alive_all(state, owner).iter().any(|&p| p != *target)
                && frames.iter().rev().find_map(|f| match &f.data {
                    EventData::CardUsed(_) => Some(f.id),
                    _ => None,
                })
                .is_some_and(|fid| !sha_dodged_shan_cards(state, *target, fid).is_empty())
        }
        ("taipingyaoshu", EventData::DamageWindow { root, window: DamageWindowKind::Begin4 }) => {
            let Some(d) = frame_damage_by_root(frames, *root) else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && tpys_damage_prevented_npc_b(state, catalog, d.source, d.target, d.card, d.nature)
        }
        _ => false,
    }
}

pub(super) fn step_combat_damage(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let _tstamp = turn_stamp(ctx.core);
    let pstamp = phase_stamp(ctx.core);
    match m.def_key.as_str() {
        "satomi_luodao" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11b_satomi_idx", 0);
                finish_satomi(m, ctx, owner)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[c]);
                }
                let idx = flag(ctx.core, owner, "l11b_satomi_idx") + 1;
                set_flag(ctx.core, owner, "l11b_satomi_idx", idx);
                finish_satomi(m, ctx, owner)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, owner, &[c]);
                }
                let idx = flag(ctx.core, owner, "l11b_satomi_idx") + 1;
                set_flag(ctx.core, owner, "l11b_satomi_idx", idx);
                finish_satomi(m, ctx, owner)
            }
            _ => Flow::Done,
        },
        "sbliegong" => {
            if let Some(d) = current_damage_data(ctx) {
                let _ = d;
                if let Some(root_fid) = damage_root_card_use_fid(ctx) {
                    let bonus = flag(ctx.core, owner, &format!("l11b_sbliegong_dmg:{root_fid}"));
                    if bonus > 0 {
                        set_flag(ctx.core, owner, &format!("l11b_sbliegong_dmg:{root_fid}"), 0);
                        helpers::log_skill(ctx, owner, "sbliegong");
                        with_current_damage_mut(ctx, |dm| dm.amount += bonus as i32);
                    }
                }
                return Flow::Done;
            }
            match m.stage {
                HostStage::Act => {
                    let Some((fid, u)) = current_card_use_data(ctx) else {
                        return Flow::Done;
                    };
                    let mask = sbliegong_effective_mask(ctx.core, ctx.catalog, owner, &u);
                    set_flag(ctx.core, owner, "l11b_sbliegong_mask", mask);
                    set_flag(ctx.core, owner, "l11b_sbliegong_done_fid", fid);
                    m.card = Some(fid);
                    wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_SBLIEGONG_CONFIRM)
                }
                HostStage::ComparePlayerPick => {
                    clear_ask(ctx, owner);
                    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        return Flow::Done;
                    }
                    let Some((fid, u)) = current_card_use_data(ctx) else {
                        return Flow::Done;
                    };
                    let mask = flag(ctx.core, owner, "l11b_sbliegong_mask");
                    if mask == 0 {
                        return Flow::Done;
                    }
                    helpers::log_skill(ctx, owner, "sbliegong");
                    let peek_n = (mask.count_ones() as usize).saturating_sub(1);
                    let mut bonus = 0u32;
                    if peek_n > 0 && !ctx.core.deck.is_empty() {
                        let start = ctx.core.deck.len().saturating_sub(peek_n);
                        for &dc in &ctx.core.deck[start..] {
                            if (dc as usize) < ctx.catalog.cards.len()
                                && (mask & suit_bit(ctx.catalog.cards[dc as usize].suit)) != 0
                            {
                                bonus += 1;
                            }
                        }
                    }
                    if bonus > 0 {
                        set_flag(ctx.core, owner, &format!("l11b_sbliegong_dmg:{fid}"), bonus);
                    }
                    if let Some(&t) = u.targets.first() {
                        set_flag(ctx.core, t, &format!("l11b_sbliegong_block_mask:{fid}"), mask);
                    }
                    set_flag(ctx.core, owner, "l11b_sbliegong_clear_fid", fid);
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "spshiji" => match m.stage {
            HostStage::Act => {
                m.target = current_damage_data(ctx).map(|d| d.target);
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_SPSHIJI_CONFIRM)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let t = m.target.unwrap_or(owner);
                helpers::log_skill(ctx, owner, "spshiji");
                let reds: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| is_red_suit(ctx.catalog.effective_card_suit(ctx.core, t, c)))
                    .collect();
                let n = reds.len() as u32;
                if n > 0 {
                    discard_player_cards(ctx, t, &reds);
                    if ctx.core.player(owner).alive {
                        ctx.start_draw(owner, n);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "spxiangzhen" => {
            helpers::log_skill(ctx, owner, "spxiangzhen");
            nullify_current_card_on_owner(ctx, owner);
            Flow::Done
        }
        "spyishi" => match m.stage {
            HostStage::Act => {
                let t = current_damage_data(ctx).map_or(owner, |d| d.target);
                m.target = Some(t);
                if ctx.core.player(t).equips.is_empty() {
                    return Flow::Done;
                }
                helpers::log_skill(ctx, owner, "spyishi");
                with_current_damage_mut(ctx, |dm| {
                    dm.amount = dm.amount.saturating_sub(1);
                    if dm.amount == 0 {
                        dm.prevented = true;
                    }
                });
                wait_ask(m, ctx, HostStage::SelectCard, ASK_SPYISHI_EQUIP)
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
        "tianming" => match m.stage {
            HostStage::Act => {
                helpers::log_skill(ctx, owner, "tianming");
                if he_cards(ctx.core, owner).is_empty() {
                    m.stage = HostStage::DrawSettle;
                    ctx.start_draw(owner, 2);
                    return Flow::Yield;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_TIANMING_DISCARD)
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
                    discard_player_cards(ctx, owner, &cs);
                }
                m.stage = HostStage::DrawSettle;
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, 2);
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                let alive = alive_all(ctx.core, owner);
                let max_hp = alive.iter().map(|&p| ctx.core.player(p).hp).max().unwrap_or(i32::MIN);
                let holders: Vec<PlayerId> = alive
                    .into_iter()
                    .filter(|&p| ctx.core.player(p).hp == max_hp)
                    .collect();
                if holders.len() == 1 && holders[0] != owner {
                    m.target = Some(holders[0]);
                    return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TIANMING_OTHER_CONFIRM);
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let other = m.target.unwrap_or(owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    || !ctx.core.player(other).alive
                {
                    return Flow::Done;
                }
                if he_cards(ctx.core, other).is_empty() {
                    ctx.start_draw(other, 2);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_TIANMING_OTHER_DISCARD)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let other = m.target.unwrap_or(owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if !cs.is_empty() {
                    discard_player_cards(ctx, other, &cs);
                }
                if ctx.core.player(other).alive {
                    ctx.start_draw(other, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twhengjiang" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TWHENGJIANG_CONFIRM),
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let Some((fid, u)) = current_card_use_data(ctx) else {
                    return Flow::Done;
                };
                let kind = u.effective_kind(ctx.catalog);
                let new_targets = twhengjiang_valid_targets(
                    ctx.core,
                    ctx.catalog,
                    owner,
                    u.card.unwrap_or(0),
                    kind,
                    u.nature,
                );
                if new_targets.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11b_twhengjiang_phase", pstamp);
                set_flag(ctx.core, owner, &format!("l11b_twhengjiang_buffed:{fid}"), 1);
                helpers::log_skill(ctx, owner, "twhengjiang");
                if let Some(f) = ctx.res.frame_mut(fid) {
                    if let EventData::CardUsed(ref mut cu) = f.data {
                        cu.targets = new_targets;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twhuangjin" => match m.stage {
            HostStage::Act => {
                let cid = m.card.unwrap_or(0);
                let sha_rank = card_effective_rank_with_mods(ctx, owner, cid, None);
                m.cards = vec![u32::from(sha_rank)];
                helpers::log_skill(ctx, owner, "twhuangjin");
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let sha_rank = m.cards.first().copied().unwrap_or(0) as i32;
                let j_rank = flag(ctx.core, owner, "last_judge_rank") as i32;
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                set_flag(ctx.core, owner, "last_judge_card", 0);
                if j_rank > 0 && (j_rank - sha_rank).abs() <= 1 {
                    nullify_current_card_on_owner(ctx, owner);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => super::combat_step_b::step_combat_step_b(m, ctx),
    }
}
