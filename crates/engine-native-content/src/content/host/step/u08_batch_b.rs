//! U08 英雄杀 Batch B（26 技能 + `kanpo`，覆盖 12 名武将，`英雄杀/character/skill.js:846-1609`）。

use super::super::*;
use crate::catalog::{CardKind, Catalog, Faction, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::content::standard::Gender;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "u08_batch_b_asks.rs"]
mod u08_batch_b_asks;

pub(in crate::content::host) use self::u08_batch_b_asks::ask_u08_batch_b;
use self::u08_batch_b_asks::handle_u08_batch_b_impl;

pub(super) const ASK_BAIHE_CARD: u32 = 1;
pub(super) const ASK_BAIHE_TARGET: u32 = 2;
pub(super) const ASK_BAIHE_TARGET_DISCARD: u32 = 3;
pub(super) const ASK_YINYANG_CARDS: u32 = 4;
pub(super) const ASK_YINYANG_TARGETS: u32 = 5;
pub(super) const ASK_YANYI_CARD: u32 = 6;
pub(super) const ASK_YANYI_TARGET: u32 = 7;
pub(super) const ASK_YANYI_SUIT: u32 = 8;
pub(super) const ASK_JIEAN_CONFIRM: u32 = 9;
pub(super) const ASK_JIEAN_GIVE_CARD: u32 = 10;
pub(super) const ASK_JIEAN_GIVE_TARGET: u32 = 11;
pub(super) const ASK_BATU_CONFIRM: u32 = 12;
pub(super) const ASK_YSHESHEN_CONFIRM: u32 = 13;
pub(super) const ASK_YSHESHEN_TARGET_1: u32 = 14;
pub(super) const ASK_YSHESHEN_TARGET_2: u32 = 15;
pub(super) const ASK_YXSCHANGNIAN_TARGET: u32 = 16;
pub(super) const ASK_SANBANFU_CONFIRM: u32 = 17;
pub(super) const ASK_SANBANFU_TARGET_SHAN: u32 = 18;
pub(super) const ASK_BINGSHENG_CARDS: u32 = 19;
pub(super) const ASK_BINGSHENG_TARGET: u32 = 20;
pub(super) const ASK_SHENTAN_CARD: u32 = 21;
pub(super) const ASK_SHENTAN_TARGET: u32 = 22;
pub(super) const ASK_SHENTAN_GAIN_CARD: u32 = 23;
pub(super) const ASK_SHENTAN_GIVE_TARGET: u32 = 24;
pub(super) const ASK_HEQIN_TARGET: u32 = 25;
pub(super) const ASK_TIEWAN_CARD: u32 = 26;
pub(super) const ASK_TIEWAN_TARGET: u32 = 27;

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn has_skill_named(
    state: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| state.has_skill(p, def))
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.insert(
            (p, def),
            crate::state::SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

pub(super) fn remove_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.remove_skill_and_r05_mark(ctx.catalog, p, def);
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

pub(super) fn alive_all(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| core.player(p).alive)
        .collect()
}

pub(super) fn he_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    let mut v = pl.hand.clone();
    v.extend(pl.equips.iter().copied());
    v
}

pub(super) fn hej_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    let mut v = pl.hand.clone();
    v.extend(pl.equips.iter().copied());
    v.extend(pl.judge.iter().copied());
    v
}

pub(super) fn is_red_card(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(state, owner, c),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn is_black_card(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(state, owner, c),
        Suit::Spade | Suit::Club
    )
}

pub(super) fn is_sha_card(state: &GameStateCore, catalog: &Catalog, c: CardId) -> bool {
    state
        .declared_kinds
        .get(&c)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(c))
        .is_sha_like()
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as
        .unwrap_or_else(|| u.effective_kind(catalog))
        .is_sha_like()
}

pub(super) fn is_delayed_trick_use(catalog: &Catalog, u: &CardUseData) -> bool {
    matches!(
        u.use_as.unwrap_or_else(|| u.effective_kind(catalog)),
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    )
}

pub(super) fn player_gender(state: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Gender {
    state.player(p).general_slots[0]
        .and_then(|def| catalog.general_meta(def).map(|(_, _, _, _, gender)| gender))
        .unwrap_or_else(|| state.effective_gender(p))
}

pub(super) fn count_distinct_alive_factions(state: &GameStateCore, catalog: &Catalog) -> usize {
    let mut mask = 0u32;
    for p in state.alive_players() {
        if let Some(f) = state.effective_faction(catalog, p) {
            let bit = match f {
                Faction::Wei => 1,
                Faction::Shu => 2,
                Faction::Wu => 4,
                Faction::Qun => 8,
            };
            mask |= bit;
        }
    }
    mask.count_ones() as usize
}

pub(super) fn valid_le_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            !core.player(t).judge.iter().any(|&jc| {
                core.declared_kinds
                    .get(&jc)
                    .copied()
                    .unwrap_or_else(|| catalog.card_kind(jc))
                    == CardKind::Le
            }) && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                card,
                CardKind::Le,
                DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn target_shan_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
) -> Vec<CardId> {
    core.player(target)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            catalog.card_kind(c) == CardKind::Shan
                || catalog
                    .find_player_vm_view_as(core, target, c, false, Some(CardKind::Shan))
                    .is_some()
        })
        .collect()
}

pub(super) fn draw_direct(ctx: &mut Ctx<'_>, p: PlayerId, n: u32) -> Vec<CardId> {
    let mut drawn = Vec::new();
    for _ in 0..n {
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(p).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: drawn.clone(),
        });
    }
    drawn
}

pub(super) fn pop_deck_cards(ctx: &mut Ctx<'_>, n: u32) -> Vec<CardId> {
    let mut out = Vec::new();
    for _ in 0..n {
        if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
            let mut reshuffled = std::mem::take(&mut ctx.core.discard);
            let len = reshuffled.len();
            for i in (1..len).rev() {
                let j = ctx.core.rng.gen_range((i + 1) as u32) as usize;
                reshuffled.swap(i, j);
            }
            ctx.core.deck = reshuffled;
        }
        if let Some(c) = ctx.core.deck.pop() {
            out.push(c);
        }
    }
    out
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    let mut origins = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.player(p).hand.iter().position(|&x| x == c) {
            ctx.core.player_mut(p).hand.remove(pos);
            ctx.to_discard(Some(c));
            discarded.push(c);
            origins.push(LostOrigin::Hand);
        } else if let Some(pos) = ctx.core.player(p).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(p).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
            ctx.to_discard(Some(c));
            discarded.push(c);
            origins.push(LostOrigin::Equip);
        }
    }
    if !discarded.is_empty() {
        ctx.note_cards_lost(p, &discarded);
        ctx.emit(EventData::CardsLost {
            player: p,
            cards: discarded.clone(),
            origins,
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn move_cards_to_hand(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cards: &[CardId],
) {
    let mut moved = Vec::new();
    let mut origins = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.player(from).hand.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).hand.remove(pos);
            moved.push(c);
            origins.push(LostOrigin::Hand);
        } else if let Some(pos) = ctx.core.player(from).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
            moved.push(c);
            origins.push(LostOrigin::Equip);
        } else if let Some(pos) = ctx.core.player(from).judge.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).judge.remove(pos);
            ctx.core.declared_kinds.remove(&c);
            moved.push(c);
            origins.push(LostOrigin::Judge);
        }
    }
    if !moved.is_empty() {
        ctx.note_cards_lost(from, &moved);
        ctx.core.player_mut(to).hand.extend(moved.iter().copied());
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: moved.clone(),
            origins,
        });
        ctx.emit(EventData::CardsMoved {
            cards: moved,
            from,
            to,
        });
    }
}

pub(super) fn with_current_damage_mut<F: FnOnce(&mut crate::event::DamageData)>(
    ctx: &mut Ctx<'_>,
    f: F,
) {
    let fid = ctx.current_event_frame_id();
    let root_id = match ctx.res.frame(fid).map(|fr| &fr.data) {
        Some(EventData::Damage(_)) => Some(fid),
        Some(EventData::DamageWindow { root, .. }) => Some(*root),
        _ => None,
    };
    if let Some(rid) = root_id {
        if let Some(frame) = ctx.res.frame_mut(rid) {
            if let EventData::Damage(dm) = &mut frame.data {
                f(dm);
            }
        }
    }
}

pub(in crate::content::host) fn trigger_meta_u08_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "xiushen" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "jiean" | "ysheshen" => Some(meta(EventKind::Damage, Vec::new())),
        "batu" | "yxschangnian2" | "heqin2" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "wumu" | "biaoqi" | "tiewan" => Some(meta(EventKind::CardUsed, Vec::new())),
        "yxschangnian" | "heqin3" => Some(meta(EventKind::Death, Vec::new())),
        "yxsciqiu" => Some(meta(
            EventKind::DamageBefore,
            vec![EventKind::CardUsed],
        )),
        "sanbanfu" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::DamageBefore,
                EventKind::ShaDodged,
                EventKind::Damage,
            ],
        )),
        "sanbanfu2" => Some(meta(EventKind::ShaDodged, vec![EventKind::Damage])),
        "sanbanfu3" => Some(meta(EventKind::DamageBefore, Vec::new())),
        "wluoyan" => Some(TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::First,
            priority: 10,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "heqin" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Death])),
        "chajue" => Some(meta(
            EventKind::Damage,
            vec![EventKind::TurnBegin, EventKind::PhaseBegin],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u08_batch_b(name: &str) -> SkillLimits {
    match name {
        "baihe" | "yinyang" | "yanyi" | "bingsheng" | "shentan" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u08_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let name = skill.name.as_str();
    if name != "yxschangnian" && name != "heqin3" && name != "heqin" && !ctx.state.player(owner).alive {
        return false;
    }
    match name {
        "xiushen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } if *player == owner
                && ctx
                    .state
                    .alive_players()
                    .any(|p| ctx.state.player(p).linked)
        ),
        "jiean" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "yanyi_damage_active") > 0
                    && ctx.state.player(d.target).alive
                    && ctx.state.player(d.target).hp < ctx.state.player(d.target).max_hp as i32
        ),
        "batu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hand.len()
                    < count_distinct_alive_factions(ctx.state, ctx.catalog)
        ),
        "wumu" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && is_sha_use(ctx.catalog, u)
                    && u.card.is_some_and(|c| is_red_card(ctx.state, ctx.catalog, owner, c))
        ),
        "ysheshen" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
        ),
        "yxschangnian" => matches!(
            event,
            EventData::Death { player, .. }
                if *player == owner
                    && !alive_others(ctx.state, owner).is_empty()
                    && !hej_cards(ctx.state, owner).is_empty()
        ),
        "yxschangnian2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "yxsciqiu" => {
            if flag(ctx.state, owner, "yxsciqiu_removed") > 0 {
                return false;
            }
            match event {
                EventData::CardUsed(u) => {
                    u.player == owner
                        && is_sha_use(ctx.catalog, u)
                        && u.targets.iter().any(|&t| {
                            ctx.state.player(t).alive
                                && ctx.state.player(t).hp == ctx.state.player(t).max_hp as i32
                        })
                }
                EventData::Damage(d) => {
                    d.source == Some(owner)
                        && !d.prevented
                        && d.amount > 0
                        && (d.card
                            .is_some_and(|c| is_sha_card(ctx.state, ctx.catalog, c))
                            || flag(
                                ctx.state,
                                owner,
                                &format!("yxsciqiu_sha_active:{}", d.target),
                            ) > 0)
                        && ctx.state.player(d.target).alive
                        && ctx.state.player(d.target).hp
                            == ctx.state.player(d.target).max_hp as i32
                }
                _ => false,
            }
        }
        "sanbanfu" => match event {
            EventData::CardUsed(u) => {
                if u.player != owner || !is_sha_use(ctx.catalog, u) || u.targets.is_empty() {
                    return false;
                }
                let t = u.targets[0];
                let cid = u.card.unwrap_or(0);
                ctx.state.player(t).alive
                    && flag(ctx.state, owner, &format!("sha_unrespondable:{cid}:{t}")) == 0
                    && !ctx.state.is_card_use_unrespondable(
                        ctx.catalog,
                        owner,
                        t,
                        cid,
                        CardKind::Sha,
                        u.nature,
                    )
            }
            EventData::ShaDodged { source, target, .. } => {
                *source == owner
                    && flag(ctx.state, owner, "sanbanfu_counter_target") == u32::from(*target) + 1
            }
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && (flag(ctx.state, owner, "sanbanfu2_boost") > 0
                        || flag(ctx.state, owner, "sanbanfu_counter_target")
                            == u32::from(d.target) + 1)
            }
            _ => false,
        },
        "sanbanfu2" => match event {
            EventData::ShaDodged { source, target, .. } => {
                *source == owner
                    && flag(ctx.state, owner, "sanbanfu_counter_target") == u32::from(*target) + 1
            }
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && flag(ctx.state, owner, "sanbanfu_counter_target")
                        == u32::from(d.target) + 1
            }
            _ => false,
        },
        "sanbanfu3" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "sanbanfu2_boost") > 0
        ),
        "biaoqi" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner && is_sha_use(ctx.catalog, u) && !u.targets.is_empty()
        ),
        "wluoyan" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
        ),
        "heqin" => match event {
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => {
                !has_skill_named(ctx.state, ctx.catalog, *player, "heqin2")
                    && flag(ctx.state, *player, "heqin2_active") > 0
                    && (*player == owner
                        || flag(ctx.state, owner, "heqin_partner") == u32::from(*player) + 1)
            }
            EventData::Death { player, .. } => {
                flag(ctx.state, owner, "heqin_partner") > 0
                    && (*player == owner
                        || flag(ctx.state, owner, "heqin_partner") == u32::from(*player) + 1)
            }
            _ => false,
        },
        "heqin2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
        ),
        "heqin3" => matches!(
            event,
            EventData::Death { player, .. }
                if flag(ctx.state, owner, "heqin_partner") > 0
                    && (*player == owner
                        || flag(ctx.state, owner, "heqin_partner") == u32::from(*player) + 1)
        ),
        "chajue" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "chajue2_active") > 0
                        || has_skill_named(ctx.state, ctx.catalog, owner, "chajue2"))
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current != owner
            }
            _ => false,
        },
        "tiewan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player != owner
                    && is_delayed_trick_use(ctx.catalog, u)
                    && he_cards(ctx.state, owner).iter().any(|&c| {
                        is_red_card(ctx.state, ctx.catalog, owner, c)
                            && !valid_le_targets(ctx.state, ctx.catalog, owner, c).is_empty()
                    })
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u08_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.current != owner
        || ctx.state.turn.phase != Phase::Play
        || ctx.catalog.has_diaohulishan(ctx.state, owner)
    {
        return None;
    }
    let state = ctx.state;
    let catalog = ctx.catalog;
    let can_activate = match skill.name.as_str() {
        "baihe" => !he_cards(state, owner).is_empty() && !alive_all(state, owner).is_empty(),
        "yinyang" => {
            state.player(owner).hand.len() >= 2 && !alive_all(state, owner).is_empty()
        }
        "yanyi" => {
            he_cards(state, owner)
                .iter()
                .any(|&c| is_black_card(state, catalog, owner, c))
                && alive_others(state, owner)
                    .iter()
                    .any(|&p| !state.player(p).hand.is_empty())
        }
        "bingsheng" => {
            if state.player(owner).hand.len() < 2 {
                false
            } else {
                let mut suits = 0u32;
                for &c in &state.player(owner).hand {
                    let bit = match catalog.effective_card_suit(state, owner, c) {
                        Suit::Spade => 1,
                        Suit::Heart => 2,
                        Suit::Club => 4,
                        Suit::Diamond => 8,
                    };
                    suits |= bit;
                }
                let my_hp = state.player(owner).hp;
                suits.count_ones() >= 2
                    && alive_others(state, owner).iter().any(|&p| {
                        let tp = state.player(p);
                        tp.hp > my_hp || (tp.hp < my_hp && tp.hp < tp.max_hp as i32)
                    })
            }
        }
        "shentan" => {
            !he_cards(state, owner).is_empty()
                && alive_others(state, owner).iter().any(|&p| {
                    !state.player(p).hand.is_empty()
                        && state.computed_distance(catalog, owner, p) <= 2
                })
        }
        "heqin" => {
            flag(state, owner, "heqin_used") == 0
                && alive_others(state, owner)
                    .iter()
                    .any(|&p| player_gender(state, catalog, p) == Gender::Male)
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_u08_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("jiean", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("ysheshen", EventData::Damage(d)) => {
            cards = vec![d.amount.max(1) as u32];
        }
        ("yxsciqiu", EventData::CardUsed(u)) => {
            card = Some(1);
            targets = u.targets.clone();
        }
        ("yxsciqiu", EventData::Damage(d)) => {
            card = Some(2);
            target = Some(d.target);
        }
        ("sanbanfu", EventData::CardUsed(u)) => {
            target = u.targets.first().copied();
            cards = vec![u.card.unwrap_or(0)];
        }
        ("sanbanfu", EventData::ShaDodged { target: t, .. }) => {
            target = Some(*t);
            card = Some(2);
        }
        ("sanbanfu", EventData::Damage(d)) => {
            target = Some(d.target);
            card = Some(if d.settlement.committed_damage.is_some() { 2 } else { 1 });
        }
        ("sanbanfu2", EventData::ShaDodged { target: t, .. }) => {
            target = Some(*t);
        }
        ("sanbanfu2", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("biaoqi", EventData::CardUsed(u)) => {
            target = u.targets.first().copied();
            card = u.card.map(u32::from);
        }
        ("heqin", EventData::Phase { player, .. }) => {
            target = Some(*player);
            card = Some(1);
        }
        ("heqin", EventData::Death { .. }) => {
            card = Some(2);
        }
        ("chajue", EventData::Damage(_)) => {
            card = Some(1);
        }
        ("chajue", _) => {
            card = Some(2);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_u08_batch_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_u08_batch_b_impl(m, ctx)
}
