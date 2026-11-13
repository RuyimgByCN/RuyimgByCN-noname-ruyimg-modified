//! U08 英雄杀 — Batch C（12 将 27 技：成吉思汗 / 罗宾汉 / 亨廷顿 / 花木兰 / 李世民 /
//! 勾践 / 吕雉 / 凯撒 / 拿破仑 / 褒姒 / 杨玉环 / 貂蝉 / 埃及艳后）。
//!
//! 源码：`archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:661-748, 1610-2406`。

use super::{HostBehavior, HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::content::standard::Gender;
use crate::event::{
    CardUseData, DamageNature, DrawWindowKind, EventData, EventKind, LostOrigin, Phase, Timing,
};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::{GameMode, GameStateCore, Role};

#[path = "u08_batch_c_asks.rs"]
mod u08_batch_c_asks;
#[path = "u08_batch_c_ext.rs"]
mod u08_batch_c_ext;

use u08_batch_c_asks::{ask_u08_batch_c_impl, handle_u08_batch_c_impl};

pub(super) const ASK_QIANGLUE_CONFIRM: u32 = 1;
pub(super) const ASK_QIANGLUE_CARD: u32 = 2;
pub(super) const ASK_XIADAO_TARGET: u32 = 3;
pub(super) const ASK_XIADAO_CARD: u32 = 4;
pub(super) const ASK_TONGYU_CARD: u32 = 5;
pub(super) const ASK_LZHANGYI_TARGET: u32 = 6;
pub(super) const ASK_YIZHUANG_CARD: u32 = 7;
pub(super) const ASK_YIZHUANG_TARGET: u32 = 8;
pub(super) const ASK_TUQIANG_CONFIRM: u32 = 9;
pub(super) const ASK_XUMOU_CONFIRM: u32 = 10;
pub(super) const ASK_ZHENSHA_CARDS: u32 = 11;
pub(super) const ASK_DUCAI_CARD: u32 = 12;
pub(super) const ASK_FANPU_TARGETS: u32 = 13;
pub(super) const ASK_FENGHUO_CARD: u32 = 14;
pub(super) const ASK_NICHANG_CONFIRM: u32 = 15;
pub(super) const ASK_FENGYAN_CONFIRM: u32 = 16;
pub(super) const ASK_FENGYI_CARD: u32 = 17;
pub(super) const ASK_FENGYI_TARGET: u32 = 18;
pub(super) const ASK_WANGE_CONFIRM: u32 = 19;
pub(super) const ASK_WANGE2_TARGETS: u32 = 20;
pub(super) const ASK_SEYOU_TARGET: u32 = 21;
pub(super) const ASK_SEYOU_MALE_SHA: u32 = 22;
pub(super) const ASK_SEYOU_GAIN_CARD: u32 = 23;
pub(super) const ASK_SHESHI_SUIT: u32 = 24;

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

pub(super) fn suit_code(s: Suit) -> u32 {
    match s {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn is_red_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, c),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn is_black_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, c),
        Suit::Spade | Suit::Club
    )
}

pub(super) fn is_sha_card(_core: &GameStateCore, catalog: &Catalog, c: CardId) -> bool {
    catalog.card_kind(c).is_sha_like()
}

pub(super) fn is_shan_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    core.declared_kinds
        .get(&c)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(c))
        == CardKind::Shan
        || (has_skill_named(core, catalog, owner, "jimin") && core.player(owner).equips.is_empty())
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as
        .unwrap_or_else(|| u.effective_kind(catalog))
        .is_sha_like()
}

pub(super) fn is_shan_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as.unwrap_or_else(|| u.effective_kind(catalog)) == CardKind::Shan
}

pub(super) fn he_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
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

pub(super) fn player_gender(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Gender {
    core.player(p).general_slots[0]
        .and_then(|def| catalog.general_meta(def).map(|(_, _, _, _, gender)| gender))
        .unwrap_or_else(|| core.effective_gender(p))
}

pub(super) fn is_friendly(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    other: PlayerId,
) -> bool {
    if owner == other {
        return true;
    }
    if flag(core, owner, &format!("tongling_ally:{other}")) > 0
        || core.player(owner).yexinjia_friends.contains(&other)
    {
        return true;
    }
    match &core.mode {
        GameMode::Identity { roles } => {
            let r_owner = roles.get(owner as usize);
            let r_other = roles.get(other as usize);
            matches!(
                (r_owner, r_other),
                (Some(Role::Lord | Role::Loyalist), Some(Role::Lord | Role::Loyalist))
                    | (Some(Role::Rebel), Some(Role::Rebel))
            )
        }
        GameMode::Versus { teams, .. } => {
            teams.iter().any(|t| t.contains(&owner) && t.contains(&other))
        }
        GameMode::Boss { boss, fellows, .. } => {
            let o_side = owner == *boss || fellows.contains(&owner);
            let t_side = other == *boss || fellows.contains(&other);
            o_side == t_side
        }
        GameMode::Doudizhu { lord } => (owner == *lord) == (other == *lord),
        GameMode::Kingdoms => {
            let f1 = core.effective_faction(catalog, owner);
            let f2 = core.effective_faction(catalog, other);
            f1.is_some() && f1 == f2
        }
        _ => false,
    }
}

pub(super) fn xiadao_candidates(
    core: &GameStateCore,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<PlayerId> {
    let nh = core.player(target).hand.len();
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && p != target && core.player(p).alive && core.player(p).hand.len() <= nh)
        .collect()
}

pub(super) fn yizhuang_male_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| player_gender(core, catalog, p) == Gender::Male)
        .collect()
}

pub(super) fn zhensha_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let hand = &core.player(owner).hand;
    let black_cnt = hand
        .iter()
        .filter(|&&c| is_black_card(core, catalog, owner, c))
        .count();
    hand.iter()
        .copied()
        .filter(|&c| {
            catalog.card_kind(c) == CardKind::Jiu
                || (black_cnt >= 2 && is_black_card(core, catalog, owner, c))
        })
        .collect()
}

pub(super) fn fanpu_sha_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                0,
                CardKind::Sha,
                DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn nanman_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                card,
                CardKind::SavageAttack,
                DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn wange2_candidates(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| !core.player(p).hand.is_empty())
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
        ctx.core.skills.remove(&(p, def));
    }
}

pub(super) fn has_skill_named(core: &GameStateCore, catalog: &Catalog, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

fn meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(crate) fn trigger_meta_u08_batch_c(name: &str) -> Option<TriggerMeta> {
    match name {
        "qianglue" => Some(meta(EventKind::ShaDodged, Vec::new())),
        "sheshu" => Some(meta(EventKind::CardUsed, Vec::new())),
        "xiadao" | "yizhuang2" | "tongling" | "sheshi" => {
            Some(meta(EventKind::Damage, Vec::new()))
        }
        "lguiyin" | "tongyu" => Some(meta(EventKind::TurnEnded, vec![EventKind::TurnBegin])),
        "lzhangyi" => Some(meta(EventKind::CardsDiscarded, Vec::new())),
        "yizhuang" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Damage])),
        "tuqiang" => Some(meta(EventKind::CardUsed, vec![EventKind::CardResponded])),
        "xumou" | "nichang" | "nichang2" | "wange2" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "wange" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::DrawPhaseBegin],
        )),
        "zhensha" => Some(meta(EventKind::Dying, Vec::new())),
        "ducai" | "ducai2" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "fengyan" => Some(meta(EventKind::OrdinaryJudge, Vec::new())),
        _ => None,
    }
}

pub(crate) fn limits_u08_batch_c(name: &str) -> SkillLimits {
    match name {
        "lguiyin" | "tongyu" | "ducai" | "fanpu" | "fengyi" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u08_batch_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "qianglue" => matches!(
            event,
            EventData::ShaDodged { source, target, .. }
                if *source == owner
                    && ctx.state.player(*target).alive
                    && !he_cards(ctx.state, *target).is_empty()
        ),
        "sheshu" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && is_sha_use(ctx.catalog, u)
                    && u.targets.iter().any(|&t| {
                        ctx.state.player(t).alive && ctx.state.player(t).hp >= 3
                    })
        ),
        "xiadao" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
                    && !ctx.state.player(d.target).hand.is_empty()
                    && !xiadao_candidates(ctx.state, owner, d.target).is_empty()
        ),
        "lguiyin" | "tongyu" => matches!(
            event,
            EventData::TurnEnded { .. } | EventData::TurnBegin { .. }
                if flag(ctx.state, owner, "tongyu_guiyin_used") > 0
                    || flag(ctx.state, owner, "skill_turn:tongyu_guiyin_used") > 0
                    || flag(ctx.state, owner, "turn_damage_dealt") > 0
        ),
        "lzhangyi" => matches!(
            event,
            EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && !alive_others(ctx.state, owner).is_empty()
                    && cards.iter().any(|c| ctx.state.discard.contains(c))
        ),
        "yizhuang" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && !he_cards(ctx.state, owner).is_empty()
                    && !yizhuang_male_targets(ctx.state, ctx.catalog, owner).is_empty()
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "yizhuang2")
                    && flag(ctx.state, owner, "yizhuang2_active") > 0
            }
            _ => false,
        },
        "yizhuang2" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
        ),
        "tuqiang" => match event {
            EventData::CardUsed(u) => u.player == owner && is_shan_use(ctx.catalog, u),
            EventData::CardResponded { player, card } => {
                *player == owner && is_shan_card(ctx.state, ctx.catalog, owner, *card)
            }
            _ => false,
        },
        "xumou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "zhensha" => matches!(
            event,
            EventData::Dying { player, .. }
                if *player != owner
                    && ctx.state.player(*player).alive
                    && ctx.state.player(*player).hp <= 0
                    && !zhensha_eligible_cards(ctx.state, ctx.catalog, owner).is_empty()
        ),
        "ducai" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player }
                if *player == owner
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "ducai2")
                    && (flag(ctx.state, owner, "ducai_suit") > 0
                        || flag(ctx.state, owner, "yxs_ducai_forbidden_suit") > 0
                        || flag(ctx.state, owner, "ducai2_active") > 0)
        ),
        "ducai2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player }
                if *player == owner
        ),
        "tongling" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "tongling_mark")
                        .max(flag(ctx.state, owner, "yxs_tongling_marks"))
                        < 3
                    && d.source
                        .is_some_and(|src| is_friendly(ctx.state, ctx.catalog, owner, src))
        ),
        "nichang" => match event {
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "nichang2")
                    && flag(ctx.state, owner, "nichang2_active") > 0
            }
            _ => false,
        },
        "nichang2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "fengyan" => matches!(
            event,
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            } if *invocation != u32::MAX
                && (*invocation as usize) < ctx.catalog.cards.len()
                && ctx.state.player(*j_owner).alive
                && player_gender(ctx.state, ctx.catalog, *j_owner) == Gender::Male
                && is_red_card(ctx.state, ctx.catalog, *j_owner, *invocation)
        ),
        "wange" => match event {
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => *player == owner,
            EventData::DrawWindow {
                window: DrawWindowKind::Begin,
                ..
            } => {
                ctx.state.turn.current == owner
                    && flag(ctx.state, owner, "wange_draw_minus_1") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "wange2")
                    && flag(ctx.state, owner, "wange2_active") > 0
            }
            _ => false,
        },
        "wange2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "sheshi" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u08_batch_c(
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
        "lguiyin" => {
            flag(state, owner, "tongyu_guiyin_used") == 0
                && flag(state, owner, "skill_turn:tongyu_guiyin_used") == 0
                && state.player(owner).turn_damage == 0
                && flag(state, owner, "turn_damage_dealt") == 0
        }
        "tongyu" => {
            flag(state, owner, "tongyu_guiyin_used") == 0
                && flag(state, owner, "skill_turn:tongyu_guiyin_used") == 0
                && !he_cards(state, owner).is_empty()
        }
        "ducai" => !he_cards(state, owner).is_empty(),
        "fanpu" => {
            let marks = flag(state, owner, "tongling_mark")
                .max(flag(state, owner, "yxs_tongling_marks"));
            marks >= 3 && !fanpu_sha_targets(state, catalog, owner).is_empty()
        }
        "fenghuo" => state
            .player(owner)
            .equips
            .first()
            .is_some_and(|&first_eq| !nanman_targets(state, catalog, owner, first_eq).is_empty()),
        "fengyi" => !state.player(owner).hand.is_empty() && !alive_all(state, owner).is_empty(),
        "seyou" => flag(state, owner, "seyou_used") == 0 && !alive_all(state, owner).is_empty(),
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

pub(in crate::content::host) fn machine_u08_batch_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("qianglue", EventData::ShaDodged { target: t, .. }) => {
            target = Some(*t);
        }
        ("sheshu", EventData::CardUsed(u)) => {
            cards = vec![u.card.unwrap_or(0)];
            targets = u.targets.clone();
        }
        ("xiadao", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("lguiyin" | "tongyu", _) => {
            card = Some(2);
        }
        ("lzhangyi", EventData::CardsDiscarded { cards: disc, .. }) => {
            cards = disc.clone();
        }
        ("yizhuang", EventData::Damage(_)) => {
            card = Some(2);
        }
        ("zhensha", EventData::Dying { player, .. }) => {
            target = Some(*player);
        }
        ("ducai", _) => {
            card = Some(2);
        }
        (
            "nichang",
            EventData::Phase {
                phase: Phase::End, ..
            },
        ) => {
            card = Some(2);
        }
        (
            "fengyan",
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            },
        ) => {
            target = Some(*j_owner);
            card = Some(*invocation);
            cards = vec![*invocation];
        }
        (
            "wange",
            EventData::DrawWindow {
                window: DrawWindowKind::Begin,
                ..
            },
        ) => {
            card = Some(2);
        }
        (
            "wange",
            EventData::Phase {
                phase: Phase::End, ..
            },
        ) => {
            card = Some(3);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U08YxsBatchC,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_u08_batch_c(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_u08_batch_c_impl(m, ctx)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u08_batch_c(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    ask_u08_batch_c_impl(stage, owner, target, targets, card, cards, core, catalog)
}
