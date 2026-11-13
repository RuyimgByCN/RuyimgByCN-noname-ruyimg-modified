//! L25 祖安武将 Batch F（SSR 武将 #41–#50 共 22 个技能，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。
//!
//! 覆盖武将：
//! - 太史慈（`ssr_kuiwei`、`ssr_newhanzhan`）
//! - 周泰（`ssr_newbuqu`、`ssr_huwei`）
//! - 夏侯渊（`ssr_shensu`、`ssr_hubu`）
//! - 曹仁（`ssr_jushou`、`ssr_zhuzhen`）
//! - 邓艾（`ssr_tuntian`、`ssr_zaoxian`）
//! - 董卓（`ssr_newhengzheng`、`ssr_baonue`、`ssr_newjiuchi`）
//! - 于吉（`ssr_newmizhou`、`ssr_chanyuan`）
//! - 袁绍（`ssr_newluanji`、`ssr_newdingsheng`、`ssr_newshiyi`）
//! - 徐庶（`ssr_zhuhai`、`ssr_yijian`）
//! - 法正（`ssr_newfabing`、`ssr_xuanhuo`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, Faction, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l25_batch_f_asks.rs"]
mod l25_batch_f_asks;
#[path = "l25_batch_f_ext.rs"]
mod l25_batch_f_ext;

pub(in crate::content::host) use self::l25_batch_f_asks::ask_l25_batch_f;
use self::l25_batch_f_asks::handle_l25_batch_f_impl;

pub(super) const ASK_KUIWEI_TARGET: u32 = 1;
pub(super) const ASK_KUIWEI_PINDIAN: u32 = 2;
pub(super) const ASK_KUIWEI_TARGET_PINDIAN: u32 = 3;
pub(super) const ASK_HUWEI_CONFIRM: u32 = 4;
pub(super) const ASK_SHENSU_MODE: u32 = 5;
pub(super) const ASK_HUBU_TARGET: u32 = 6;
pub(super) const ASK_HUBU_OPTION: u32 = 7;
pub(super) const ASK_JUSHOU_CARD: u32 = 8;
pub(super) const ASK_JUSHOU_OPTION: u32 = 9;
pub(super) const ASK_ZHUZHEN_CARD: u32 = 10;
pub(super) const ASK_ZHUZHEN_TARGET: u32 = 11;
pub(super) const ASK_TUNTIAN_DISCARD: u32 = 12;
pub(super) const ASK_TUNTIAN_CHOOSE_CARD: u32 = 13;
pub(super) const ASK_NEWHENGZHENG_MODE: u32 = 14;
pub(super) const ASK_NEWJIUCHI_CONFIRM: u32 = 15;
pub(super) const ASK_NEWMIZHOU_CARD: u32 = 16;
pub(super) const ASK_NEWMIZHOU_OPTION: u32 = 17;
pub(super) const ASK_NEWLUANJI_DISCARD: u32 = 18;
pub(super) const ASK_NEWSHIYI_CONFIRM: u32 = 19;
pub(super) const ASK_ZHUHAI_DISCARD: u32 = 20;
pub(super) const ASK_ZHUHAI_TARGET: u32 = 21;
pub(super) const ASK_YIJIAN_TARGET: u32 = 22;
pub(super) const ASK_NEWFABING_OPTION: u32 = 23;
pub(super) const ASK_XUANHUO_DISCARD: u32 = 24;
pub(super) const ASK_XUANHUO_TARGET: u32 = 25;

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

pub(super) fn hej_cards(state: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = state.player(p);
    pl.hand
        .iter()
        .chain(pl.equips.iter())
        .chain(pl.judge.iter())
        .copied()
        .collect()
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

pub(super) fn is_red_suit(s: Suit) -> bool {
    matches!(s, Suit::Heart | Suit::Diamond)
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

pub(super) fn newbuqu_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_newbuqu".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn jushou_pile(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(p, "ssr_jushou".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn newmizhou_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_newmizhou".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn shensu_marks(core: &GameStateCore, owner: PlayerId) -> u32 {
    flag(core, owner, "l25_shensu_su").max(flag(core, owner, "mark:ssr_shensu"))
}

pub(super) fn set_shensu_marks(core: &mut GameStateCore, owner: PlayerId, val: u32) {
    set_flag(core, owner, "l25_shensu_su", val);
    set_flag(core, owner, "mark:ssr_shensu", val);
}

pub(super) fn tuntian_marks(core: &GameStateCore, owner: PlayerId) -> u32 {
    flag(core, owner, "l25_tuntian_tian").max(flag(core, owner, "mark:ssr_tuntian"))
}

pub(super) fn set_tuntian_marks(core: &mut GameStateCore, owner: PlayerId, val: u32) {
    let clamped = val.min(4);
    set_flag(core, owner, "l25_tuntian_tian", clamped);
    set_flag(core, owner, "mark:ssr_tuntian", clamped);
}

pub(super) fn tuntian_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let all = he_cards(core, owner);
    let mut grouped = Vec::new();
    for s in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond] {
        let matching: Vec<CardId> = all
            .iter()
            .copied()
            .filter(|&c| catalog.effective_card_suit(core, owner, c) == s)
            .collect();
        if matching.len() >= 2 {
            grouped.extend(matching);
        }
    }
    if grouped.len() >= 2 {
        grouped
    } else {
        all
    }
}

pub(super) fn xuanhuo_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let non_basic: Vec<CardId> = he_cards(core, owner)
        .into_iter()
        .filter(|&c| !catalog.card_kind(c).is_basic_kind())
        .collect();
    if !non_basic.is_empty() {
        non_basic
    } else {
        he_cards(core, owner)
    }
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

pub(in crate::content::host) fn trigger_meta_l25_batch_f(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_kuiwei" => Some(meta(
            EventKind::TurnEnded,
            vec![EventKind::PhaseEnd, EventKind::TurnPhaseEnd],
        )),
        "ssr_newhanzhan" => Some(meta(
            EventKind::TurnBegin,
            vec![EventKind::PhaseBegin, EventKind::Damage, EventKind::CardUsed],
        )),
        "ssr_newbuqu" => Some(meta(EventKind::Dying, vec![EventKind::CardUsed])),
        "ssr_huwei" | "ssr_baonue" | "ssr_yijian" | "ssr_newfabing" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "ssr_shensu" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::Damage],
        )),
        "ssr_hubu" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::PhaseEnd, EventKind::TurnPhaseEnd],
        )),
        "ssr_jushou" => Some(meta(
            EventKind::TurnBegin,
            vec![
                EventKind::PhaseBegin,
                EventKind::DamageBefore,
                EventKind::Damage,
            ],
        )),
        "ssr_zhuzhen" => Some(meta(EventKind::Damage, Vec::new())),
        "ssr_tuntian" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::Damage,
                EventKind::PhaseEnd,
                EventKind::TurnPhaseEnd,
            ],
        )),
        "ssr_zaoxian" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "ssr_newhengzheng" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::CardsDrawn, EventKind::DrawPhase],
        )),
        "ssr_newjiuchi" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "ssr_newmizhou" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::PhaseEnd, EventKind::TurnPhaseEnd],
        )),
        "ssr_chanyuan" => Some(meta(EventKind::Death, vec![EventKind::Dying])),
        "ssr_newdingsheng" => Some(meta(
            EventKind::CardsDiscarded,
            vec![EventKind::CardsLost, EventKind::CardUsed],
        )),
        "ssr_newshiyi" => Some(meta(
            EventKind::CardsDiscarded,
            vec![EventKind::CardsLost],
        )),
        "ssr_zhuhai" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::PhaseEnd,
                EventKind::TurnPhaseEnd,
                EventKind::Damage,
            ],
        )),
        "ssr_xuanhuo" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnBegin],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_f(name: &str) -> SkillLimits {
    match name {
        "ssr_jushou" | "ssr_tuntian" | "ssr_newmizhou" | "ssr_xuanhuo" | "ssr_zhuhai" => {
            SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            }
        }
        _ => SkillLimits::default(),
    }
}

fn make_active_machine(
    name: &str,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    Some(Box::new(HostMachine {
        def_key: name.to_string(),
        behavior: HostBehavior::L25ZuanBatchF,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn activate_l25_batch_f(
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
        "ssr_kuiwei" => {
            if used_this_turn(state, owner, "ssr_kuiwei") >= 2
                || pl.hand.is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
            {
                return None;
            }
            make_active_machine("ssr_kuiwei", owner)
        }
        "ssr_jushou" => {
            if used_this_turn(state, owner, "ssr_jushou") >= 1
                || pl.hand.is_empty()
                || (jushou_pile(state, owner).is_empty()
                    && flag(state, owner, "l25_jushou_init") > 0)
            {
                return None;
            }
            make_active_machine("ssr_jushou", owner)
        }
        "ssr_zhuzhen" => {
            if flag(state, owner, "l25_zhuzhen_used_total") >= 2
                || jushou_pile(state, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_zhuzhen", owner)
        }
        "ssr_tuntian" => {
            if used_this_turn(state, owner, "ssr_tuntian") >= 1
                || tuntian_marks(state, owner) >= 4
                || tuntian_eligible_cards(state, catalog, owner).len() < 2
            {
                return None;
            }
            make_active_machine("ssr_tuntian", owner)
        }
        "ssr_newmizhou" => {
            if used_this_turn(state, owner, "ssr_newmizhou") >= 1
                || pl.hand.is_empty()
                || newmizhou_pile(state, owner).len() >= 4
            {
                return None;
            }
            make_active_machine("ssr_newmizhou", owner)
        }
        "ssr_newluanji" => {
            if used_this_turn(state, owner, "ssr_newluanji") >= 3
                || he_cards(state, owner).len() < 2
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_newluanji", owner)
        }
        "ssr_zhuhai" => {
            if used_this_turn(state, owner, "ssr_zhuhai") >= 1
                || he_cards(state, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_zhuhai", owner)
        }
        "ssr_xuanhuo" => {
            if used_this_turn(state, owner, "ssr_xuanhuo") >= 1
                || xuanhuo_eligible_cards(state, catalog, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_xuanhuo", owner)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_l25_batch_f(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let pl = state.player(owner);
    if !pl.alive && skill.name != "ssr_chanyuan" && skill.name != "ssr_newbuqu" {
        return false;
    }
    match skill.name.as_str() {
        "ssr_kuiwei" => match event {
            EventData::TurnEnded { .. }
            | EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } => alive_all(state, owner)
                .into_iter()
                .any(|p| flag(state, p, "l25_kuiwei_forbid_basic") > 0),
            _ => false,
        },
        "ssr_newhanzhan" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && (pl.hujia > 0 || pl.hp < pl.max_hp as i32),
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::CardUsed(u) => {
                u.player == owner && is_sha_use(catalog, u) && pl.hp < pl.max_hp as i32
            }
            _ => false,
        },
        "ssr_newbuqu" => match event {
            EventData::Dying { player, .. } => *player == owner,
            EventData::CardUsed(u) => {
                u.player == owner && is_sha_use(catalog, u) && !newbuqu_pile(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_huwei" => match event {
            EventData::CardUsed(u) => {
                used_this_round(state, owner, "ssr_huwei") < 2
                    && u.player != owner
                    && !u.targets.contains(&owner)
                    && !u.targets.is_empty()
                    && is_sha_or_duel_use(catalog, u)
            }
            _ => false,
        },
        "ssr_shensu" => match event {
            EventData::Phase {
                player,
                phase: Phase::Judge | Phase::Draw | Phase::Play | Phase::Discard,
                begin: true,
            } => *player == owner,
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            _ => false,
        },
        "ssr_hubu" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEndWindow {
                player,
                after: false,
            } => {
                *player == owner
                    && shensu_marks(state, owner) >= 2
                    && !alive_others(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_jushou" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && jushou_pile(state, owner).is_empty()
                    && flag(state, owner, "l25_jushou_init") == 0
            }
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 1,
            _ => false,
        },
        "ssr_zhuzhen" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && alive_others(state, owner).into_iter().any(|p| {
                        !jushou_pile(state, p).is_empty()
                            || flag(state, p, "l25_zhuzhen_cheng") > 0
                    })
            }
            _ => false,
        },
        "ssr_tuntian" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEndWindow {
                player,
                after: false,
            } => *player == owner && tuntian_marks(state, owner) > 0,
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && tuntian_marks(state, owner) > 0
            }
            _ => false,
        },
        "ssr_zaoxian" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::TurnBegin { player } => {
                *player == owner
                    && tuntian_marks(state, owner) >= 4
                    && flag(state, owner, "l25_zaoxian_awakened") == 0
            }
            _ => false,
        },
        "ssr_newhengzheng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            }
            | EventData::DrawPhase { player, .. }
            | EventData::CardsDrawn { player, .. } => {
                *player == owner
                    && used_this_turn(state, owner, "ssr_newhengzheng") == 0
                    && alive_others(state, owner)
                        .into_iter()
                        .any(|p| !hej_cards(state, p).is_empty())
            }
            _ => false,
        },
        "ssr_baonue" => match event {
            EventData::CardUsed(u) => {
                pl.hp > 2
                    && is_sha_use(catalog, u)
                    && (u.player == owner || u.targets.contains(&owner))
            }
            _ => false,
        },
        "ssr_newjiuchi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                let any_taofa = alive_all(state, owner)
                    .into_iter()
                    .any(|p| flag(state, p, "l25_hengzheng_taofa") > 0);
                any_taofa || (*player == owner && pl.hp < pl.max_hp as i32)
            }
            _ => false,
        },
        "ssr_newmizhou" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEndWindow {
                player,
                after: false,
            } => *player == owner && !newmizhou_pile(state, owner).is_empty(),
            _ => false,
        },
        "ssr_chanyuan" => match event {
            EventData::Death { player, .. } | EventData::Dying { player, .. } => {
                *player == owner && !alive_others(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_newdingsheng" => match event {
            EventData::CardsDiscarded { player, cards }
            | EventData::CardsLost { player, cards, .. } => {
                *player == owner
                    && cards.len() >= 2
                    && used_this_turn(state, owner, "ssr_newdingsheng") == 0
            }
            EventData::CardUsed(u) => {
                u.player == owner
                    && u.use_as
                        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
                        == Some(CardKind::ArcheryAttack)
            }
            _ => false,
        },
        "ssr_newshiyi" => match event {
            EventData::CardsDiscarded { player, cards }
            | EventData::CardsLost { player, cards, .. } => {
                cards.len() >= 2
                    && state.player(*player).alive
                    && (*player == owner
                        || state.effective_faction(catalog, *player) == Some(Faction::Qun))
            }
            _ => false,
        },
        "ssr_zhuhai" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| s != owner && state.player(s).alive)
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEndWindow {
                player,
                after: false,
            } => {
                *player != owner
                    && state.player(*player).alive
                    && !he_cards(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_yijian" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_trick_use(catalog, u)
                    && flag(state, owner, "l25_yijian_awakened") == 0
            }
            _ => false,
        },
        "ssr_newfabing" => match event {
            EventData::CardUsed(u) => u.player == owner && is_sha_use(catalog, u),
            _ => false,
        },
        "ssr_xuanhuo" => match event {
            EventData::CardUsed(u) => {
                flag(state, u.player, "l25_xuanhuo_sha_buff") > 0 && is_sha_use(catalog, u)
            }
            EventData::TurnBegin { player } => {
                *player == owner
                    && alive_all(state, owner)
                        .into_iter()
                        .any(|p| flag(state, p, "l25_xuanhuo_sha_buff") > 0)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_f(
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
            target = d.source.or(Some(d.target));
            targets = vec![d.target];
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
        } => {
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
        EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(110);
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
        behavior: HostBehavior::L25ZuanBatchF,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l25_batch_f(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_f_impl(m, ctx)
}
