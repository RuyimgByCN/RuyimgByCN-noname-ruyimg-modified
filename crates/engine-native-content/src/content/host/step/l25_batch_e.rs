//! L25 祖安武将 Batch E（SSR 武将 #31–#40 共 22 个技能，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。
//!
//! 覆盖武将：
//! - 甄姬（`ssr_yushui`）
//! - 孙尚香（`ssr_jieyin`、`ssr_weigong`）
//! - 司马懿（`ssr_tianlang`、`ssr_newjilue`）
//! - 伊籍（`ssr_newjijie`、`ssr_newjiyuan`）
//! - 黄月英（`ssr_jiqiao`、`ssr_qicai`）
//! - 贾诩（`ssr_newweimu`、`ssr_guimou`、`ssr_newluanwu`）
//! - 蔡文姬（`ssr_nianlei`、`ssr_newduanxian`）
//! - 孟获（`ssr_manjia`、`ssr_zaiqi`）
//! - 祝融（`ssr_lieren`）
//! - 姜维（`ssr_zhiji`、`ssr_weilong`）
//! - 孙策（`ssr_hunzi`、`ssr_newjiang`、`ssr_newzhiba`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l25_batch_e_asks.rs"]
mod l25_batch_e_asks;
#[path = "l25_batch_e_ext.rs"]
mod l25_batch_e_ext;

pub(in crate::content::host) use self::l25_batch_e_asks::ask_l25_batch_e;
use self::l25_batch_e_asks::handle_l25_batch_e_impl;

pub(super) const ASK_JIEYIN_GIVE: u32 = 1;
pub(super) const ASK_JIEYIN_TARGET: u32 = 2;
pub(super) const ASK_JIEYIN_GAIN: u32 = 3;
pub(super) const ASK_WEIGONG_DISCARD: u32 = 4;
pub(super) const ASK_WEIGONG_GAIN: u32 = 5;
pub(super) const ASK_TIANLANG_CONFIRM: u32 = 6;
pub(super) const ASK_TIANLANG_STAR: u32 = 7;
pub(super) const ASK_NEWJILUE_CONFIRM: u32 = 8;
pub(super) const ASK_NEWJIJIE_CARD: u32 = 9;
pub(super) const ASK_NEWJIJIE_TARGET: u32 = 10;
pub(super) const ASK_NEWJIYUAN_CONFIRM: u32 = 11;
pub(super) const ASK_NEWJIYUAN_TARGET: u32 = 12;
pub(super) const ASK_JIQIAO_HAND: u32 = 13;
pub(super) const ASK_JIQIAO_PILE: u32 = 14;
pub(super) const ASK_QICAI_TARGET: u32 = 15;
pub(super) const ASK_NEWWEIMU_CARD: u32 = 16;
pub(super) const ASK_NEWWEIMU_TARGET: u32 = 17;
pub(super) const ASK_GUIMOU_CONFIRM: u32 = 18;
pub(super) const ASK_GUIMOU_TARGETS: u32 = 19;
pub(super) const ASK_GUIMOU_PINDIAN_1: u32 = 20;
pub(super) const ASK_GUIMOU_PINDIAN_2: u32 = 21;
pub(super) const ASK_NEWLUANWU_CONFIRM: u32 = 22;
pub(super) const ASK_NEWLUANWU_MODE: u32 = 23;
pub(super) const ASK_NIANLEI_GIVE: u32 = 24;
pub(super) const ASK_NIANLEI_TARGET: u32 = 25;
pub(super) const ASK_NEWDUANXIAN_DISCARD: u32 = 26;
pub(super) const ASK_ZAIQI_DISCARD: u32 = 27;
pub(super) const ASK_ZAIQI_CONFIRM: u32 = 28;
pub(super) const ASK_LIEREN_CARD: u32 = 29;
pub(super) const ASK_LIEREN_TARGET: u32 = 30;
pub(super) const ASK_LIEREN_RESPONSE: u32 = 31;
pub(super) const ASK_ZHIJI_OPTION: u32 = 32;
pub(super) const ASK_WEILONG_CARD: u32 = 33;
pub(super) const ASK_WEILONG_TARGET: u32 = 34;
pub(super) const ASK_HUNZI_TARGET: u32 = 35;
pub(super) const ASK_HUNZI_PINDIAN: u32 = 36;
pub(super) const ASK_HUNZI_TARGET_PINDIAN: u32 = 37;
pub(super) const ASK_NEWZHIBA_DISCARD: u32 = 38;

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

pub(super) fn is_black_suit(s: Suit) -> bool {
    matches!(s, Suit::Spade | Suit::Club)
}

pub(super) fn red_hand_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| is_red_suit(catalog.effective_card_suit(core, p, c)))
        .collect()
}

pub(super) fn black_he_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    he_cards(core, p)
        .into_iter()
        .filter(|&c| is_black_suit(catalog.effective_card_suit(core, p, c)))
        .collect()
}

pub(super) fn is_weapon_card(catalog: &Catalog, c: CardId) -> bool {
    catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon)
}

pub(super) fn lieren_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let weapons: Vec<CardId> = he_cards(core, owner)
        .into_iter()
        .filter(|&c| is_weapon_card(catalog, c))
        .collect();
    if !weapons.is_empty() {
        weapons
    } else {
        he_cards(core, owner)
    }
}

pub(super) fn card_type_bit(catalog: &Catalog, c: CardId) -> u32 {
    let kind = catalog.card_kind(c);
    if kind.is_basic_kind() {
        1
    } else if kind.is_equip() {
        4
    } else {
        2
    }
}

pub(super) fn newjijie_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let mask = if flag(core, owner, "l25_newjijie_mask_stamp") == turn_stamp(core) {
        flag(core, owner, "l25_newjijie_type_mask")
    } else {
        0
    };
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| (mask & card_type_bit(catalog, c)) == 0)
        .collect()
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

pub(super) fn is_trick_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c))) else {
        return false;
    };
    !kind.is_basic_kind() && !kind.is_equip()
}

pub(super) fn jiqiao_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_jiqiao".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn lieren_pile(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(p, "ssr_lieren".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn zaiqi_marks(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "l25_zaiqi_init") == 0 {
        flag(core, owner, "l25_zaiqi_man")
            .max(flag(core, owner, "mark:ssr_zaiqi"))
            .max(7)
    } else {
        flag(core, owner, "l25_zaiqi_man").max(flag(core, owner, "mark:ssr_zaiqi"))
    }
}

pub(super) fn set_zaiqi_marks(core: &mut GameStateCore, owner: PlayerId, val: u32) {
    set_flag(core, owner, "l25_zaiqi_init", 1);
    set_flag(core, owner, "l25_zaiqi_man", val);
    set_flag(core, owner, "mark:ssr_zaiqi", val);
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

pub(in crate::content::host) fn trigger_meta_l25_batch_e(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_yushui" => Some(meta(EventKind::DamageBefore, vec![EventKind::Damage])),
        "ssr_jieyin" => Some(meta(
            EventKind::TurnPhaseEnd,
            vec![EventKind::PhaseBegin, EventKind::PhaseEnd, EventKind::TurnEnded],
        )),
        "ssr_weigong" | "ssr_newjilue" | "ssr_qicai" | "ssr_manjia" | "ssr_newjiang" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "ssr_tianlang" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "ssr_newjijie" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::CardResponded, EventKind::TurnBegin],
        )),
        "ssr_newjiyuan" => Some(meta(
            EventKind::CardsLost,
            vec![
                EventKind::CardsDiscarded,
                EventKind::Dying,
                EventKind::Damage,
                EventKind::PhaseBegin,
                EventKind::TurnPhaseEnd,
                EventKind::PhaseEnd,
                EventKind::TurnEnded,
            ],
        )),
        "ssr_jiqiao" => Some(meta(
            EventKind::TurnBegin,
            vec![EventKind::PhaseBegin],
        )),
        "ssr_newweimu" => Some(meta(
            EventKind::TurnOvered,
            vec![EventKind::CardUsed, EventKind::TurnBegin],
        )),
        "ssr_guimou" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "ssr_newluanwu" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "ssr_nianlei" => Some(meta(
            EventKind::TurnBegin,
            vec![EventKind::PhaseBegin, EventKind::CardUsed],
        )),
        "ssr_zaiqi" => Some(meta(
            EventKind::Damage,
            vec![
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
                EventKind::Dying,
            ],
        )),
        "ssr_lieren" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::TurnBegin,
                EventKind::Dying,
                EventKind::Death,
            ],
        )),
        "ssr_zhiji" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Dying])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_e(name: &str) -> SkillLimits {
    match name {
        "ssr_jieyin" | "ssr_newjijie" | "ssr_jiqiao" | "ssr_nianlei" | "ssr_zaiqi"
        | "ssr_lieren" | "ssr_weilong" | "ssr_newzhiba" | "ssr_newluanwu"
        | "ssr_newduanxian" => SkillLimits {
            once_per_turn: true,
            ..SkillLimits::default()
        },
        _ => SkillLimits::default(),
    }
}

fn make_active_machine(name: &str, owner: PlayerId) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    Some(Box::new(HostMachine {
        def_key: name.to_string(),
        behavior: HostBehavior::L25ZuanBatchE,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn activate_l25_batch_e(
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
        "ssr_jieyin" => {
            if used_this_turn(state, owner, "ssr_jieyin") >= 1
                || red_hand_cards(state, catalog, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_jieyin", owner)
        }
        "ssr_newjijie" => {
            if used_this_turn(state, owner, "ssr_newjijie") >= 1
                || newjijie_eligible_cards(state, catalog, owner).is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !hej_cards(state, p).is_empty())
            {
                return None;
            }
            make_active_machine("ssr_newjijie", owner)
        }
        "ssr_jiqiao" => {
            if used_this_turn(state, owner, "ssr_jiqiao") >= 1 {
                return None;
            }
            make_active_machine("ssr_jiqiao", owner)
        }
        "ssr_newweimu" => {
            if !pl.face_down
                || black_he_cards(state, catalog, owner).is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !hej_cards(state, p).is_empty())
            {
                return None;
            }
            make_active_machine("ssr_newweimu", owner)
        }
        "ssr_newluanwu" => {
            if flag(state, owner, "l25_newluanwu_used") > 0 {
                return None;
            }
            make_active_machine("ssr_newluanwu", owner)
        }
        "ssr_nianlei" => {
            if used_this_turn(state, owner, "ssr_nianlei") >= 1
                || pl.hand.is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_nianlei", owner)
        }
        "ssr_newduanxian" => {
            if flag(state, owner, "l25_newduanxian_used") > 0
                || he_cards(state, owner).len() < 2
            {
                return None;
            }
            make_active_machine("ssr_newduanxian", owner)
        }
        "ssr_zaiqi" => {
            if used_this_turn(state, owner, "ssr_zaiqi") >= 1
                || zaiqi_marks(state, owner) == 0
                || he_cards(state, owner).len() < 2
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_zaiqi", owner)
        }
        "ssr_lieren" => {
            if used_this_turn(state, owner, "ssr_lieren") >= 1
                || lieren_eligible_cards(state, catalog, owner).is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| lieren_pile(state, p).is_empty())
            {
                return None;
            }
            make_active_machine("ssr_lieren", owner)
        }
        "ssr_weilong" => {
            if used_this_turn(state, owner, "ssr_weilong") >= 1 || pl.hand.is_empty() {
                return None;
            }
            make_active_machine("ssr_weilong", owner)
        }
        "ssr_hunzi" => {
            if used_this_turn(state, owner, "ssr_hunzi") >= 2
                || pl.hand.is_empty()
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
            {
                return None;
            }
            make_active_machine("ssr_hunzi", owner)
        }
        "ssr_newzhiba" => {
            if used_this_turn(state, owner, "ssr_newzhiba") >= 1
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !he_cards(state, p).is_empty())
            {
                return None;
            }
            make_active_machine("ssr_newzhiba", owner)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_l25_batch_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let pl = state.player(owner);
    if !pl.alive && skill.name != "ssr_lieren" {
        return false;
    }
    match skill.name.as_str() {
        "ssr_yushui" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
        ),
        "ssr_jieyin" => match event {
            EventData::TurnEnded { player } => {
                flag(state, *player, "l25_jieyin_hongzhuang") > 0
                    || alive_all(state, owner)
                        .into_iter()
                        .any(|p| flag(state, p, "l25_jieyin_hongzhuang") > 0)
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => flag(state, *player, "l25_jieyin_hongzhuang") > 0,
            _ => false,
        },
        "ssr_weigong" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_sha_use(catalog, u)
                    && !state.player(u.player).hand.is_empty()
            }
            _ => false,
        },
        "ssr_tianlang" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "ssr_newjilue" => match event {
            EventData::CardUsed(u) => {
                u.player != owner && u.targets.contains(&owner) && is_trick_use(catalog, u)
            }
            _ => false,
        },
        "ssr_newjijie" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::CardUsed(u) => u.player == owner,
            EventData::CardResponded { player, .. } => *player == owner,
            _ => false,
        },
        "ssr_newjiyuan" => match event {
            EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                *player != owner
                    && state.player(*player).alive
                    && state.player(*player).hand.is_empty()
                    && !pl.hand.is_empty()
            }
            EventData::Dying { player, .. } => {
                *player != owner && state.player(*player).alive && !pl.hand.is_empty()
            }
            EventData::Damage(d) => {
                d.target != owner
                    && !d.prevented
                    && d.amount > 0
                    && state.player(d.target).alive
                    && state.player(d.target).hp <= 1
                    && !pl.hand.is_empty()
            }
            EventData::TurnEnded { player }
            | EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => *player == owner,
            _ => false,
        },
        "ssr_jiqiao" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && jiqiao_pile(state, owner).len() < 3,
            _ => false,
        },
        "ssr_qicai" => match event {
            EventData::CardUsed(u) => u.player == owner && is_trick_use(catalog, u),
            _ => false,
        },
        "ssr_newweimu" => match event {
            EventData::TurnOvered { player, .. } => *player == owner,
            EventData::TurnBegin { player } => *player == owner,
            EventData::CardUsed(u) => {
                (u.player == owner && pl.face_down)
                    || (u.player != owner && u.targets.contains(&owner) && pl.face_down)
            }
            _ => false,
        },
        "ssr_guimou" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && alive_all(state, owner)
                        .into_iter()
                        .filter(|&p| !state.player(p).hand.is_empty())
                        .count()
                        >= 2
            }
            _ => false,
        },
        "ssr_newluanwu" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_newluanwu_used") == 0
            }
            _ => false,
        },
        "ssr_nianlei" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_nianlei_init") == 0,
            EventData::CardUsed(u) => {
                u.player != owner
                    && is_sha_use(catalog, u)
                    && u.targets
                        .iter()
                        .any(|&t| flag(state, t, "l25_nianlei_chenqing") > 0)
            }
            _ => false,
        },
        "ssr_manjia" => match event {
            EventData::CardUsed(u) => {
                u.targets.contains(&owner)
                    && (is_sha_use(catalog, u)
                        || u.use_as
                            .or_else(|| u.card.map(|c| catalog.card_kind(c)))
                            .is_some_and(|k| {
                                matches!(k, CardKind::SavageAttack | CardKind::ArcheryAttack)
                            }))
            }
            _ => false,
        },
        "ssr_zaiqi" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_zaiqi_init") == 0,
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !pl.linked
                    && zaiqi_marks(state, owner) > 0
            }
            EventData::Dying { player, .. } => *player == owner && zaiqi_marks(state, owner) > 0,
            _ => false,
        },
        "ssr_lieren" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_lieren_init") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && (flag(state, owner, "l25_lieren_init") == 0
                        || alive_others(state, owner).into_iter().any(|p| {
                            !lieren_pile(state, p).is_empty() || flag(state, p, "l25_lieren_en") > 0
                        }))
            }
            EventData::Dying { player, .. } => {
                *player == owner
                    && alive_others(state, owner)
                        .into_iter()
                        .any(|p| !lieren_pile(state, p).is_empty() || flag(state, p, "l25_lieren_en") > 0)
            }
            EventData::Death { player, .. } => !lieren_pile(state, *player).is_empty(),
            _ => false,
        },
        "ssr_zhiji" => {
            if flag(state, owner, "l25_zhiji_stage") >= 3 {
                return false;
            }
            match event {
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true,
                } => *player == owner && (pl.hp < pl.max_hp as i32 || pl.hand.is_empty()),
                EventData::Dying { player, .. } => {
                    let bit = 1u32 << (*player as u32);
                    (flag(state, owner, "l25_zhiji_dying_mask") & bit) == 0
                }
                _ => false,
            }
        }
        "ssr_newjiang" => match event {
            EventData::CardUsed(u) => {
                let is_sha_or_duel = is_sha_use(catalog, u)
                    || u.use_as
                        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
                        == Some(CardKind::Duel);
                is_sha_or_duel && (u.player == owner || u.targets.contains(&owner))
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_e(
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
            if let Some(k) = u.use_as.or_else(|| u.card.map(|_| CardKind::Sha)) {
                let _ = k;
            }
        }
        EventData::CardResponded { player, card: c } => {
            target = Some(*player);
            card = Some(103);
            cards.push(*c);
        }
        EventData::Dying { player, .. } => {
            target = Some(*player);
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
        | EventData::Phase {
            player,
            phase: Phase::End,
            begin: false,
        } => {
            target = Some(*player);
            card = Some(111);
        }
        EventData::Death { player, .. } => {
            target = Some(*player);
            card = Some(112);
        }
        EventData::TurnOvered { player, .. } => {
            target = Some(*player);
            card = Some(113);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L25ZuanBatchE,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l25_batch_e(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_e_impl(m, ctx)
}
