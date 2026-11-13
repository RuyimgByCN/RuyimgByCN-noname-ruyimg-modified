//! L25 祖安武将 Batch G（SSR 武将 #51–#60 共 22 个技能，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。
//!
//! 覆盖武将：
//! - 庞统（`ssr_lianhuan`、`ssr_newsance`）
//! - 张郃（`ssr_newxiebian`、`ssr_motu`）
//! - 李典（`ssr_xunxun`、`ssr_wangxi`）
//! - 徐晃（`ssr_jieliang`、`ssr_zhiyan`）
//! - 孙坚（`ssr_futu`、`ssr_newpolu`）
//! - 鲁肃（`ssr_dimeng`、`ssr_fuhui`）
//! - 张昭张纮（`ssr_zhijian`、`ssr_guzheng`）
//! - 张角（`ssr_tianlei`、`ssr_gaiming`、`ssr_newguidao`、`ssr_zhuji`）
//! - 张宝（`ssr_dihuo`、`ssr_guifu`）
//! - 张梁（`ssr_renhong`、`ssr_fangtong`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, Faction, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l25_batch_g_asks.rs"]
mod l25_batch_g_asks;
#[path = "l25_batch_g_ext.rs"]
mod l25_batch_g_ext;

pub(in crate::content::host) use self::l25_batch_g_asks::ask_l25_batch_g;
use self::l25_batch_g_asks::handle_l25_batch_g_impl;

pub(super) const ASK_LIANHUAN_DISCARD: u32 = 1;
pub(super) const ASK_LIANHUAN_TARGETS: u32 = 2;
pub(super) const ASK_NEWSANCE_MODE: u32 = 3;
pub(super) const ASK_NEWSANCE_TARGET: u32 = 4;
pub(super) const ASK_NEWSANCE_CARD: u32 = 5;
pub(super) const ASK_NEWXIEBIAN_MODE: u32 = 6;
pub(super) const ASK_XUNXUN_CARDS: u32 = 7;
pub(super) const ASK_WANGXI_CARD: u32 = 8;
pub(super) const ASK_JIELIANG_DISCARD: u32 = 9;
pub(super) const ASK_JIELIANG_TARGET: u32 = 10;
pub(super) const ASK_FUTU_TARGET: u32 = 11;
pub(super) const ASK_FUTU_PINDIAN_OWNER: u32 = 12;
pub(super) const ASK_FUTU_PINDIAN_TARGET: u32 = 13;
pub(super) const ASK_FUTU_MODE: u32 = 14;
pub(super) const ASK_DIMENG_TARGETS: u32 = 15;
pub(super) const ASK_FUHUI_CONFIRM: u32 = 16;
pub(super) const ASK_FUHUI_PINDIAN_OWNER: u32 = 17;
pub(super) const ASK_FUHUI_PINDIAN_TARGET: u32 = 18;
pub(super) const ASK_ZHIJIAN_GIVE: u32 = 19;
pub(super) const ASK_ZHIJIAN_TARGET: u32 = 20;
pub(super) const ASK_GUZHENG_CONFIRM: u32 = 21;
pub(super) const ASK_GUZHENG_CARD: u32 = 22;
pub(super) const ASK_TIANLEI_TARGET: u32 = 23;
pub(super) const ASK_GAIMING_CARD: u32 = 24;
pub(super) const ASK_NEWGUIDAO_CARD: u32 = 25;
pub(super) const ASK_NEWGUIDAO_TARGET: u32 = 26;
pub(super) const ASK_ZHUJI_DISCARD: u32 = 27;
pub(super) const ASK_DIHUO_TARGET: u32 = 28;
pub(super) const ASK_GUIFU_CARD: u32 = 29;
pub(super) const ASK_GUIFU_TARGET: u32 = 30;
pub(super) const ASK_RENHONG_TARGETS: u32 = 31;
pub(super) const ASK_FANGTONG_SHOW: u32 = 32;
pub(super) const ASK_FANGTONG_TARGET: u32 = 33;
pub(super) const ASK_FANGTONG_RESPONSE: u32 = 34;

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

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn
        .round
        .saturating_mul(64)
        .saturating_add(u32::from(core.turn.current) + 1)
}

pub(super) fn used_this_turn(core: &GameStateCore, p: PlayerId, skill_key: &str) -> u32 {
    let stamp_key = format!("l25_stamp:{skill_key}");
    if flag(core, p, &stamp_key) == turn_stamp(core) {
        flag(core, p, &format!("l25_used:{skill_key}"))
    } else {
        0
    }
}

pub(super) fn inc_used_this_turn(core: &mut GameStateCore, p: PlayerId, skill_key: &str) -> u32 {
    let cur = used_this_turn(core, p, skill_key) + 1;
    let stamp = turn_stamp(core);
    set_flag(core, p, &format!("l25_stamp:{skill_key}"), stamp);
    set_flag(core, p, &format!("l25_used:{skill_key}"), cur);
    cur
}

pub(super) fn used_this_round(core: &GameStateCore, p: PlayerId, skill_key: &str) -> u32 {
    let round_stamp = core.turn.round.saturating_add(1);
    let stamp_key = format!("l25_rstamp:{skill_key}");
    if flag(core, p, &stamp_key) == round_stamp {
        flag(core, p, &format!("l25_rused:{skill_key}"))
    } else {
        0
    }
}

pub(super) fn inc_used_this_round(core: &mut GameStateCore, p: PlayerId, skill_key: &str) -> u32 {
    let cur = used_this_round(core, p, skill_key) + 1;
    let round_stamp = core.turn.round.saturating_add(1);
    set_flag(core, p, &format!("l25_rstamp:{skill_key}"), round_stamp);
    set_flag(core, p, &format!("l25_rused:{skill_key}"), cur);
    cur
}

pub(super) fn alive_others(state: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let n = state.players.len();
    (1..n)
        .map(|off| ((owner as usize + off) % n) as PlayerId)
        .filter(|&p| state.player(p).alive)
        .collect()
}

pub(super) fn alive_all(state: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let n = state.players.len();
    (0..n)
        .map(|off| ((owner as usize + off) % n) as PlayerId)
        .filter(|&p| state.player(p).alive)
        .collect()
}

pub(super) fn he_cards(state: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = state.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
}

pub(super) fn card_suit(catalog: &Catalog, c: CardId) -> Suit {
    catalog
        .cards
        .get(c as usize)
        .map(|d| d.suit)
        .unwrap_or(Suit::Spade)
}

pub(super) fn card_rank(catalog: &Catalog, c: CardId) -> u8 {
    catalog
        .cards
        .get(c as usize)
        .map(|d| d.rank)
        .unwrap_or(7)
}

pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1 << 0,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
    }
}

pub(super) fn card_type_bit(catalog: &Catalog, c: CardId) -> u32 {
    let kind = catalog.card_kind(c);
    if kind.is_basic_kind() {
        1 << 0
    } else if kind.is_equip() {
        1 << 2
    } else {
        1 << 1
    }
}

pub(super) fn is_red_suit(s: Suit) -> bool {
    matches!(s, Suit::Heart | Suit::Diamond)
}

pub(super) fn is_black_suit(s: Suit) -> bool {
    matches!(s, Suit::Spade | Suit::Club)
}

pub(super) fn is_black_non_trick(catalog: &Catalog, c: CardId) -> bool {
    let s = card_suit(catalog, c);
    let kind = catalog.card_kind(c);
    is_black_suit(s) && (kind.is_basic_kind() || kind.is_equip())
}

pub(super) fn is_sha_card(catalog: &Catalog, c: CardId) -> bool {
    matches!(
        catalog.card_kind(c),
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
    )
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let kind = u
        .use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .unwrap_or(CardKind::Sha);
    u.use_as
        .is_some_and(|k| matches!(k, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
        || (u.card.is_some()
            && matches!(kind, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
}

pub(super) fn is_sha_or_duel_use(catalog: &Catalog, u: &CardUseData) -> bool {
    is_sha_use(catalog, u)
        || u.use_as
            .or_else(|| u.card.map(|c| catalog.card_kind(c)))
            == Some(CardKind::Duel)
}

pub(super) fn is_trick_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c))) else {
        return false;
    };
    !kind.is_basic_kind() && !kind.is_equip()
}

pub(super) fn lianhuan_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let mask = flag(core, owner, "l25_lianhuan_suit_mask") | suit_bit(Suit::Club);
    let all = he_cards(core, owner);
    let filtered: Vec<CardId> = all
        .iter()
        .copied()
        .filter(|&c| (mask & suit_bit(catalog.effective_card_suit(core, owner, c))) != 0)
        .collect();
    if filtered.is_empty() {
        all
    } else {
        filtered
    }
}

pub(super) fn jieliang_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let all = he_cards(core, owner);
    let filtered: Vec<CardId> = all
        .iter()
        .copied()
        .filter(|&c| is_black_non_trick(catalog, c))
        .collect();
    if filtered.is_empty() {
        all
    } else {
        filtered
    }
}

pub(super) fn newguidao_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let all = he_cards(core, owner);
    let filtered: Vec<CardId> = all
        .iter()
        .copied()
        .filter(|&c| is_black_suit(catalog.effective_card_suit(core, owner, c)))
        .collect();
    if filtered.is_empty() {
        all
    } else {
        filtered
    }
}

pub(super) fn jieliang_pile(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(p, "ssr_jieliang".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn guifu_pile(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(p, "ssr_guifu".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn discard_he_cards(ctx: &mut Ctx<'_>, player: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    for &c in cards {
        if ctx.core.player(player).hand.contains(&c) {
            ctx.discard_from_hand(player, c);
            discarded.push(c);
        } else if ctx.core.player(player).equips.contains(&c) {
            ctx.take_owned_card(player, c);
            ctx.to_discard(c);
            ctx.out.push(LogEvent::CardDiscarded {
                player,
                cards: vec![c],
            });
            discarded.push(c);
        }
    }
    if !discarded.is_empty() {
        ctx.emit(EventData::CardsDiscarded {
            player,
            cards: discarded,
        });
    }
}

pub(super) fn fetch_card_matching<F>(ctx: &mut Ctx<'_>, mut pred: F) -> Option<CardId>
where
    F: FnMut(CardId, &Catalog) -> bool,
{
    if let Some(pos) = ctx.core.deck.iter().rposition(|&c| pred(c, ctx.catalog)) {
        return Some(ctx.core.deck.remove(pos));
    }
    if let Some(pos) = ctx.core.discard.iter().rposition(|&c| pred(c, ctx.catalog)) {
        return Some(ctx.core.discard.remove(pos));
    }
    None
}

fn meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::First,
        priority: 8,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(in crate::content::host) fn trigger_meta_l25_batch_g(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_lianhuan" | "ssr_fuhui" | "ssr_guifu" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_newsance" => Some(meta(
            EventKind::TurnEnded,
            vec![EventKind::PhaseEnd, EventKind::TurnPhaseEnd],
        )),
        "ssr_newxiebian" | "ssr_xunxun" | "ssr_newpolu" | "ssr_dimeng" | "ssr_tianlei"
        | "ssr_zhuji" | "ssr_dihuo" | "ssr_renhong" => {
            Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin]))
        }
        "ssr_motu" | "ssr_wangxi" | "ssr_zhiyan" => Some(meta(EventKind::Damage, Vec::new())),
        "ssr_jieliang" => Some(meta(
            EventKind::CardsDrawn,
            vec![
                EventKind::DrawPhase,
                EventKind::Damage,
                EventKind::PhaseEnd,
                EventKind::TurnPhaseEnd,
                EventKind::TurnEnded,
            ],
        )),
        "ssr_zhijian" => Some(meta(
            EventKind::Damage,
            vec![
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
                EventKind::CardUsed,
            ],
        )),
        "ssr_guzheng" => Some(meta(
            EventKind::CardsDiscarded,
            vec![EventKind::CardsLost],
        )),
        "ssr_gaiming" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::OrdinaryJudge],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_g(name: &str) -> SkillLimits {
    match name {
        "ssr_newsance" | "ssr_zhijian" | "ssr_newguidao" | "ssr_fangtong" => SkillLimits {
            once_per_turn: true,
            active_outside_play: false,
        },
        _ => SkillLimits::default(),
    }
}

fn make_active_machine(
    name: &str,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    Some(Box::new(HostMachine {
        def_key: name.to_string(),
        behavior: HostBehavior::L25ZuanBatchG,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn activate_l25_batch_g(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if state.turn.current != owner || state.turn.phase != Phase::Play {
        return None;
    }
    let pl = state.player(owner);
    if !pl.alive {
        return None;
    }
    match skill.name.as_str() {
        "ssr_lianhuan" => {
            if used_this_turn(state, owner, "ssr_lianhuan") >= 2
                || lianhuan_eligible_cards(state, catalog, owner).is_empty()
                || alive_all(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_lianhuan", owner)
        }
        "ssr_newsance" => {
            if used_this_turn(state, owner, "ssr_newsance") >= 1 {
                return None;
            }
            make_active_machine("ssr_newsance", owner)
        }
        "ssr_jieliang" => {
            if used_this_turn(state, owner, "ssr_jieliang") >= 2
                || jieliang_eligible_cards(state, catalog, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_jieliang", owner)
        }
        "ssr_futu" => {
            if used_this_turn(state, owner, "ssr_futu") >= 2
                || pl.hand.is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
            {
                return None;
            }
            make_active_machine("ssr_futu", owner)
        }
        "ssr_zhijian" => {
            if used_this_turn(state, owner, "ssr_zhijian") >= 1
                || he_cards(state, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_zhijian", owner)
        }
        "ssr_newguidao" => {
            if used_this_turn(state, owner, "ssr_newguidao") >= 1
                || newguidao_eligible_cards(state, catalog, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_newguidao", owner)
        }
        "ssr_guifu" => {
            if used_this_turn(state, owner, "ssr_guifu") >= 2
                || pl.hand.is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_guifu", owner)
        }
        "ssr_fangtong" => {
            if used_this_turn(state, owner, "ssr_fangtong") >= 1
                || pl.hand.is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_fangtong", owner)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_l25_batch_g(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let pl = state.player(owner);
    if !pl.alive {
        return false;
    }
    match skill.name.as_str() {
        "ssr_lianhuan" => match event {
            EventData::CardUsed(u) => {
                (is_sha_use(catalog, u) || is_trick_use(catalog, u))
                    && u.targets.iter().any(|&t| {
                        state.player(t).alive
                            && (state.player(t).linked
                                || flag(state, t, "l25_lianhuan_relink") > 0)
                    })
            }
            _ => false,
        },
        "ssr_newsance" => match event {
            EventData::TurnEnded { player }
            | EventData::Phase {
                player,
                phase: Phase::End,
                ..
            } => *player == owner && flag(state, owner, "l25_newsance_hand_plus2") > 0,
            _ => false,
        },
        "ssr_newxiebian" | "ssr_tianlei" | "ssr_dihuo" | "ssr_renhong" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => *player == owner,
            _ => false,
        },
        "ssr_motu" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && pl.hp <= 1
                    && !pl.hand.is_empty()
                    && used_this_round(state, owner, "ssr_motu") < 1
            }
            _ => false,
        },
        "ssr_xunxun" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => *player == owner && (pl.hujia == 0 || pl.hujia > 1),
            _ => false,
        },
        "ssr_wangxi" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && ((d.target == owner
                        && d.source.is_some_and(|s| s != owner && state.player(s).alive))
                        || (d.source == Some(owner)
                            && d.target != owner
                            && state.player(d.target).alive))
            }
            _ => false,
        },
        "ssr_jieliang" => match event {
            EventData::CardsDrawn { player, cards } => {
                !cards.is_empty() && flag(state, *player, "l25_jieliang_jie") > 0
            }
            EventData::DrawPhase { player, .. } => flag(state, *player, "l25_jieliang_jie") > 0,
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source.is_some()
                    && !jieliang_pile(state, d.target).is_empty()
            }
            EventData::TurnEnded { player }
            | EventData::Phase {
                player,
                phase: Phase::End,
                ..
            } => {
                flag(state, *player, "l25_jieliang_jie") > 0
                    || !jieliang_pile(state, *player).is_empty()
            }
            _ => false,
        },
        "ssr_zhiyan" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && (d.target == owner || d.source == Some(owner))
                    && (pl.hand.len() < pl.max_hp as usize
                        || pl.equips.len() < pl.hp.max(0) as usize)
            }
            _ => false,
        },
        "ssr_newpolu" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => {
                *player == owner
                    && flag(state, owner, "l25_polu_awakened") == 0
                    && (flag(state, owner, "l25_polu_ready") > 0 || {
                        let others = alive_others(state, owner);
                        !others.is_empty()
                            && others
                                .iter()
                                .all(|&o| flag(state, o, "l25_futu_guijiang") > 0)
                    })
            }
            _ => false,
        },
        "ssr_dimeng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => {
                *player == owner && alive_others(state, owner).len() >= 2
            }
            _ => false,
        },
        "ssr_fuhui" => match event {
            EventData::CardUsed(u) => {
                if !is_sha_or_duel_use(catalog, u) {
                    return false;
                }
                let him = if u.player == owner && !u.targets.is_empty() {
                    Some(u.targets[0])
                } else if u.targets.contains(&owner) && u.player != owner {
                    Some(u.player)
                } else {
                    None
                };
                him.is_some_and(|h| state.player(h).alive && !state.player(h).hand.is_empty())
            }
            _ => false,
        },
        "ssr_zhijian" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && flag(state, d.target, "l25_zhijian_locked_card") > 0
            }
            EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                let locked = flag(state, *player, "l25_zhijian_locked_card");
                locked > 0 && !state.player(*player).hand.contains(&(locked - 1))
            }
            EventData::CardUsed(u) => {
                let locked = flag(state, u.player, "l25_zhijian_locked_card");
                locked > 0
                    && (u.card == Some(locked - 1)
                        || !state.player(u.player).hand.contains(&(locked - 1)))
            }
            _ => false,
        },
        "ssr_guzheng" => match event {
            EventData::CardsDiscarded { player, cards }
            | EventData::CardsLost { player, cards, .. } => {
                !cards.is_empty()
                    && state.player(*player).alive
                    && used_this_turn(state, owner, "ssr_guzheng") == 0
            }
            _ => false,
        },
        "ssr_gaiming" => match event {
            EventData::Phase {
                phase: Phase::Judge,
                begin: true,
                ..
            }
            | EventData::OrdinaryJudge { .. } => !he_cards(state, owner).is_empty(),
            _ => false,
        },
        "ssr_zhuji" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => {
                state.player(*player).alive
                    && !he_cards(state, *player).is_empty()
                    && (*player == owner
                        || state.effective_faction(catalog, *player) == Some(Faction::Qun))
            }
            _ => false,
        },
        "ssr_guifu" => match event {
            EventData::CardUsed(u) => {
                let pile = guifu_pile(state, u.player);
                !pile.is_empty()
                    && match u.card {
                        Some(cid) => {
                            let c_red = is_red_suit(card_suit(catalog, cid));
                            pile.iter().any(|&fc| is_red_suit(card_suit(catalog, fc)) == c_red)
                        }
                        None => true,
                    }
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_g(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match event {
        EventData::Damage(d) => {
            target = Some(d.target);
            if let Some(src) = d.source {
                targets.push(src);
            }
            card = Some(101);
            if let Some(c) = d.card {
                cards.push(c);
            }
        }
        EventData::CardUsed(u) => {
            target = Some(u.player);
            targets = u.targets.clone();
            card = Some(102);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        EventData::CardResponded { player, card: c } => {
            target = Some(*player);
            card = Some(103);
            cards.push(*c);
        }
        EventData::CardsDrawn { player, cards: cs } => {
            target = Some(*player);
            cards = cs.clone();
            card = Some(104);
        }
        EventData::Dying { player, source } => {
            target = (*source).or(Some(*player));
            targets = vec![*player];
            card = Some(105);
        }
        EventData::CardsDiscarded { player, cards: cs } => {
            target = Some(*player);
            cards = cs.clone();
            card = Some(106);
        }
        EventData::CardsLost {
            player, cards: cs, ..
        } => {
            target = Some(*player);
            cards = cs.clone();
            card = Some(107);
        }
        EventData::Phase {
            player,
            phase: Phase::Prepare,
            begin: true,
        }
        | EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(108);
        }
        EventData::Phase {
            player,
            phase: Phase::Play,
            begin: true,
        } => {
            target = Some(*player);
            card = Some(109);
        }
        EventData::TurnEnded { player }
        | EventData::TurnEndWindow { player, .. }
        | EventData::Phase {
            player,
            phase: Phase::End,
            ..
        } => {
            target = Some(*player);
            card = Some(111);
        }
        EventData::Death { player, killer } => {
            target = *killer;
            targets = vec![*player];
            card = Some(112);
        }
        EventData::OrdinaryJudge { owner: p, .. } => {
            target = Some(*p);
            card = Some(113);
        }
        EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } => {
            target = Some(*player);
            card = Some(114);
        }
        EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        }
        | EventData::DrawPhase { player, .. } => {
            target = Some(*player);
            card = Some(115);
        }
        EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        } => {
            target = Some(*player);
            card = Some(116);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L25ZuanBatchG,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l25_batch_g(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_g_impl(m, ctx)
}
