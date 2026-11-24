//! Trigger filters and counterpart/target validators for `HostBehavior::L11VmFixC`.

use super::helpers::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, Suit};
use crate::content::host::HostSkillBehavior;
use crate::event::{DamageData, DamageNature, EventData, Frame, Phase};
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::d34_l11_mods_vm_c;
use crate::state::GameStateCore;

pub(super) fn filter_l11_vm_c_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    let name = skill.name.as_str();
    let locked = matches!(
        name,
        "baonu"
            | "juejing"
            | "nsyice"
            | "nzry_jieying"
            | "nzry_cunmu"
            | "ranshang"
            | "redangxian"
            | "rekuangcai"
            | "shibei"
            | "anjian"
            | "new_repaoxiao"
            | "xiandeng"
            | "reshanxi2"
    );
    if !is_vm_c_unblocked(ctx.state, ctx.catalog, owner, locked) {
        return false;
    }
    let tstamp = turn_stamp(ctx.state);
    match (name, event) {
        ("baonu", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && match frame.map(|f| f.kind) {
                    Some(crate::event::EventKind::DamageSource) => d.source == Some(owner),
                    Some(crate::event::EventKind::DamageEnd) => d.target == owner,
                    _ => false,
                }
        }
        ("baonu", EventData::TurnBegin { .. }) => {
            flag(ctx.state, owner, "l11c_baonu_init") == 0
        }
        ("chongzhen", EventData::CardUsed(u)) => {
            u.player == owner
                && !u.cancelled
                && chongzhen_counterpart_on_use(ctx.state, ctx.catalog, frames, owner, u)
                    .is_some_and(|t| !ctx.state.player(t).hand.is_empty())
        }
        ("chongzhen", EventData::CardResponded { card, player }) => {
            *player == owner
                && chongzhen_counterpart_on_respond(ctx.state, ctx.catalog, frames, owner, *card)
                    .is_some_and(|t| !ctx.state.player(t).hand.is_empty())
        }
        ("dujin", _) => draw_phase_for(frames, frame, owner).is_some_and(|(_, fixed)| !fixed),
        ("gnjinfan", EventData::Phase { player, phase: Phase::Discard, begin: true }) => {
            if *player != owner {
                return false;
            }
            let used_suits: u32 = pile_cards(ctx.state, owner, "gnjinfan")
                .into_iter()
                .fold(0, |m, c| m | suit_bit(card_suit(ctx.state, ctx.catalog, owner, c)));
            used_suits.count_ones() < 4
                && ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| (used_suits & suit_bit(card_suit(ctx.state, ctx.catalog, owner, c))) == 0)
        }
        ("juejing", _) => {
            draw_phase_for(frames, frame, owner).is_some_and(|(_, fixed)| {
                !fixed && ctx.state.player(owner).hp < (ctx.state.player(owner).max_hp as i32)
            })
        }
        ("shelie", _) => draw_phase_for(frames, frame, owner).is_some_and(|(_, fixed)| {
            !fixed && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        }),
        ("juece", EventData::CardsLost { player, cards, .. })
        | ("juece", EventData::CardsDiscarded { player, cards }) => {
            !cards.is_empty()
                && *player != owner
                && ctx.state.turn.current == owner
                && ctx.state.player(*player).alive
                && ctx.state.player(*player).hand.is_empty()
        }
        ("mieji", EventData::CardUsed(u)) => {
            u.player == owner
                && !u.cancelled
                && u.targets.len() == 1
                && u.card.is_some_and(|c| {
                    matches!(
                        card_suit(ctx.state, ctx.catalog, owner, c),
                        Suit::Spade | Suit::Club
                    )
                })
                && mieji_can_expand(u.effective_kind(ctx.catalog))
                && !mieji_extra_targets(ctx.state, ctx.catalog, owner, u).is_empty()
        }
        ("nsfeixiong", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && alive_others(ctx.state, owner).into_iter().any(|p| {
                    !ctx.state.player(p).hand.is_empty()
                        && d34_l11_mods_vm_c::can_compare_target(ctx.state, ctx.catalog, p)
                })
        }
        ("nsyice", EventData::CardsDiscarded { player, cards }) => {
            *player == owner && cards.iter().any(|c| ctx.state.discard.contains(c))
        }
        ("nzry_jieying", EventData::TurnBegin { .. }) => {
            !ctx.state.player(owner).linked && flag(ctx.state, owner, "l11c_jieying_init") == 0
        }
        ("nzry_jieying", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| !ctx.state.player(p).linked)
        }
        ("qiaoshui", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && alive_others(ctx.state, owner).into_iter().any(|p| {
                    !ctx.state.player(p).hand.is_empty()
                        && d34_l11_mods_vm_c::can_compare_target(ctx.state, ctx.catalog, p)
                })
        }
        ("qiaoshui3", EventData::CardUsed(u)) => {
            let kind = u.effective_kind(ctx.catalog);
            u.player == owner
                && !u.cancelled
                && flag(ctx.state, owner, "l11c_qiaoshui3_turn") == tstamp
                && (kind.is_basic_kind() || kind.is_ordinary_trick_kind())
                && !is_multitarget_qiaoshui3(kind)
                && (!qiaoshui3_add_targets(ctx.state, ctx.catalog, owner, u).is_empty()
                    || u.targets.len() > 1)
        }
        ("ranshang", EventData::Damage(d)) => {
            d.target == owner && !d.prevented && d.amount > 0 && d.nature == crate::event::DamageNature::Fire
        }
        ("ranshang2", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && flag(ctx.state, owner, "l11c_ranshang_marks") > 0
        }
        ("redangxian", EventData::TurnBegin { player }) => *player == owner,
        ("rejizhi", EventData::CardUsed(u)) => {
            u.player == owner
                && !u.cancelled
                && u.card.is_some()
                && u.use_as.is_none()
                && u.effective_kind(ctx.catalog).is_trick_kind()
        }
        ("rekuangcai", EventData::Phase { player, phase: Phase::Discard, begin: true }) => {
            *player == owner
                && (flag(ctx.state, owner, "l11c_turn_card_used_stamp") != tstamp
                    || flag(ctx.state, owner, "l11c_turn_source_dmg_stamp") != tstamp)
        }
        ("rekuangcai", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && flag(ctx.state, owner, "l11c_turn_source_dmg_stamp") == tstamp
                && flag(ctx.state, owner, "l11c_turn_source_dmg_amt") > 0
        }
        ("reshanxi", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
                && ctx.state.player(owner).hp > 0
                && !reshanxi_cost_cards(ctx.state, ctx.catalog, owner).is_empty()
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| !he_cards(ctx.state, p).is_empty())
        }
        ("reshanxi2", EventData::Phase { phase: Phase::End, begin: false, .. }) => {
            !pile_cards(ctx.state, owner, "reshanxi2").is_empty()
        }
        ("rezhenxing", EventData::Damage(d)) => d.target == owner && !d.prevented && d.amount > 0,
        ("rezhenxing", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        }
        ("shibei", EventData::Damage(d)) => d.target == owner && !d.prevented && d.amount > 0,
        ("anjian", EventData::Damage(d)) => {
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && !d.from_chain
                && damage_root_is_sha(frames, d, ctx.catalog)
                && !ctx.state.in_attack_range(ctx.catalog, d.target, owner)
        }
        ("xinkuanggu", EventData::Damage(d)) => {
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && ctx.state.computed_distance_including_dead(ctx.catalog, owner, d.target) <= 1
        }
        ("xinniluan", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player != owner
                && ctx.state.player(*player).alive
                && flag(ctx.state, *player, "l11c_used_other_target_turn") == tstamp
                && !xinniluan_usable_cards(ctx.state, ctx.catalog, owner).is_empty()
                && card_usable_on_target_with_card(ctx.state, ctx.catalog, owner, *player, 0, CardKind::Sha, true)
        }
        ("zhuikong", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(owner).hp < (ctx.state.player(owner).max_hp as i32)
                && !ctx.state.player(owner).hand.is_empty()
                && !ctx.state.player(*player).hand.is_empty()
                && d34_l11_mods_vm_c::can_compare_target(ctx.state, ctx.catalog, *player)
        }
        _ => false,
    }
}

fn draw_phase_for(frames: &[Frame], frame: Option<&Frame>, owner: PlayerId) -> Option<(PlayerId, bool)> {
    let root = match frame.map(|f| &f.data) {
        Some(EventData::DrawWindow { root, .. }) => frames.iter().find(|f| f.id == *root),
        Some(EventData::DrawPhase { .. }) => frame,
        _ => frames.iter().rev().find(|f| matches!(f.data, EventData::DrawPhase { .. })),
    }?;
    if let EventData::DrawPhase { player, num_fixed, .. } = &root.data {
        if *player == owner {
            return Some((*player, *num_fixed));
        }
    }
    None
}

pub(super) fn mieji_can_expand(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Dismantle | CardKind::Snatch | CardKind::Duel | CardKind::Linghunzhihuo | CardKind::Chain
    )
}

pub(super) fn mieji_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &crate::event::CardUseData,
) -> Vec<PlayerId> {
    let kind = u.effective_kind(catalog);
    let cid = u.card.unwrap_or(0);
    single_targets_for_virtual_trick(state, catalog, owner, kind, false)
        .into_iter()
        .filter(|t| {
            !u.targets.contains(t)
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    state,
                    catalog,
                    owner,
                    *t,
                    cid,
                    kind,
                    DamageNature::Normal,
                )
        })
        .collect()
}

pub(super) fn is_multitarget_qiaoshui3(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Jiedao
            | CardKind::Wuzhong
    )
}

pub(super) fn qiaoshui3_add_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &crate::event::CardUseData,
) -> Vec<PlayerId> {
    let kind = u.effective_kind(catalog);
    let cid = u.card.unwrap_or(0);
    if kind.is_sha_like() {
        sha_targets_for_card(state, catalog, owner, u.card, kind, true)
            .into_iter()
            .filter(|t| !u.targets.contains(t))
            .collect()
    } else if matches!(kind, CardKind::Le | CardKind::Bing) {
        alive_others(state, owner)
            .into_iter()
            .filter(|t| {
                !u.targets.contains(t)
                    && !state
                        .player(*t)
                        .judge
                        .iter()
                        .any(|&jc| catalog.card_kind(jc) == kind)
            })
            .collect()
    } else if kind.is_ordinary_trick_kind() {
        single_targets_for_virtual_trick(state, catalog, owner, kind, true)
            .into_iter()
            .filter(|t| !u.targets.contains(t))
            .filter(|t| {
                !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    state,
                    catalog,
                    owner,
                    *t,
                    cid,
                    kind,
                    DamageNature::Normal,
                )
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub(super) fn damage_root_is_sha(frames: &[Frame], d: &DamageData, catalog: &Catalog) -> bool {
    if let Some(dmg_frame) = frames
        .iter()
        .rev()
        .find(|f| matches!(&f.data, EventData::Damage(fd) if fd.target == d.target && fd.amount == d.amount))
    {
        if let Some(k) = crate::event::damage_card_kind(frames, dmg_frame, catalog) {
            return k.is_sha_like();
        }
    }
    d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
}

fn has_longdan_family(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    ["longdan", crate::content::standard::LONG_DAN, "ollongdan", "fz_new_longdan", "zylongdan"]
        .into_iter()
        .any(|s| has_named_skill(state, catalog, owner, s) && is_vm_c_unblocked(state, catalog, owner, false))
}

pub(super) fn chongzhen_counterpart_on_use(
    state: &GameStateCore,
    catalog: &Catalog,
    frames: &[Frame],
    owner: PlayerId,
    u: &crate::event::CardUseData,
) -> Option<PlayerId> {
    if !has_longdan_family(state, catalog, owner) {
        return None;
    }
    let eff = u.effective_kind(catalog);
    let phys_kind = u.card.map(|c| catalog.card_kind(c));
    if eff.is_sha_like() && phys_kind == Some(CardKind::Shan) {
        return u.targets.first().copied().filter(|&t| t != owner && state.player(t).alive);
    }
    if eff == CardKind::Shan && phys_kind.is_some_and(|k| k.is_sha_like()) {
        let from_stack = frames.iter().rev().find_map(|f| match &f.data {
            EventData::CardUsed(su)
                if su.player != owner && su.effective_kind(catalog).is_sha_like() =>
            {
                Some(su.player)
            }
            _ => None,
        });
        let fallback = flag(state, owner, "responding_to_sha_from").checked_sub(1).map(|v| v as PlayerId);
        return from_stack.or(fallback).filter(|&t| t != owner && state.player(t).alive);
    }
    None
}

pub(super) fn chongzhen_counterpart_on_respond(
    state: &GameStateCore,
    catalog: &Catalog,
    frames: &[Frame],
    owner: PlayerId,
    card: CardId,
) -> Option<PlayerId> {
    if !has_longdan_family(state, catalog, owner) {
        return None;
    }
    let eff_val = flag(state, owner, &format!("l11c_chongzhen_resp_eff:{card}"));
    let phys_kind = catalog.card_kind(card);
    let eff_sha = eff_val == (CardKind::Sha as u32) + 1
        || eff_val == (CardKind::FireSha as u32) + 1
        || eff_val == (CardKind::LeiSha as u32) + 1;
    let eff_shan = eff_val == (CardKind::Shan as u32) + 1;
    let valid_conv =
        (phys_kind == CardKind::Shan && eff_sha) || (phys_kind.is_sha_like() && eff_shan);
    if !valid_conv {
        return None;
    }
    let from_stack = frames.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(su) if su.player != owner => Some(su.player),
        _ => None,
    });
    from_stack.filter(|&t| t != owner && state.player(t).alive)
}
