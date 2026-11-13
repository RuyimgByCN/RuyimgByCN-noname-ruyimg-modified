//! L25 祖安武将 Batch D（SSR 武将 #21–#30 共 22 个技能，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。
//!
//! 覆盖武将：
//! - 黄盖（`ssr_fenchuan`）
//! - 貂蝉（`ssr_newlihun`、`ssr_biyue`）
//! - 袁术（`ssr_yjixi`、`ssr_yanji`、`ssr_cuandi`、`ssr_yongsi`）
//! - 关羽（`ssr_wusheng`、`ssr_yijue`）
//! - 吕蒙（`ssr_newbaiyi`、`ssr_liecheng`）
//! - 公孙瓒（`ssr_newyucong`、`ssr_newyicong`）
//! - 大乔（`ssr_newguose`、`ssr_newtongque`）
//! - 小乔（`ssr_newtianxiang`）
//! - 许褚（`ssr_huchi`、`ssr_newluoyi`、`ssr_luoyi`）
//! - 张飞（`ssr_zhenhun`、`ssr_paoxiao`）
//! - 甄姬（`ssr_newluoshen`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l25_batch_d_asks.rs"]
mod l25_batch_d_asks;
#[path = "l25_batch_d_ext.rs"]
mod l25_batch_d_ext;

pub(in crate::content::host) use self::l25_batch_d_asks::ask_l25_batch_d;
use self::l25_batch_d_asks::handle_l25_batch_d_impl;

pub(super) const ASK_FENCHUAN_SHOW: u32 = 1;
pub(super) const ASK_FENCHUAN_DISCARD: u32 = 2;
pub(super) const ASK_NEWLIHUN_GIVE: u32 = 3;
pub(super) const ASK_NEWLIHUN_TARGET: u32 = 4;
pub(super) const ASK_NEWLIHUN_GUESS: u32 = 5;
pub(super) const ASK_YJIXI_GAIN: u32 = 6;
pub(super) const ASK_YANJI_DISCARD: u32 = 7;
pub(super) const ASK_YANJI_TARGET: u32 = 8;
pub(super) const ASK_CUANDI_CONFIRM: u32 = 9;
pub(super) const ASK_CUANDI_DISCARD: u32 = 10;
pub(super) const ASK_YONGSI_TARGET: u32 = 11;
pub(super) const ASK_YONGSI_MODE: u32 = 12;
pub(super) const ASK_YONGSI_GIVE: u32 = 13;
pub(super) const ASK_YONGSI_CONFIRM: u32 = 14;
pub(super) const ASK_YONGSI_DISCARD: u32 = 15;
pub(super) const ASK_YIJUE_CARD: u32 = 16;
pub(super) const ASK_YIJUE_TARGET: u32 = 17;
pub(super) const ASK_YIJUE_CHOICE: u32 = 18;
pub(super) const ASK_NEWBAIYI_CARD: u32 = 19;
pub(super) const ASK_NEWBAIYI_TARGET: u32 = 20;
pub(super) const ASK_LIECHENG_TARGET: u32 = 21;
pub(super) const ASK_LIECHENG_CARD: u32 = 22;
pub(super) const ASK_NEWYUCONG_CARD: u32 = 23;
pub(super) const ASK_NEWYUCONG_TARGET: u32 = 24;
pub(super) const ASK_NEWGUOSE_MODE: u32 = 25;
pub(super) const ASK_NEWGUOSE_TARGET: u32 = 26;
pub(super) const ASK_NEWGUOSE_PINDIAN: u32 = 27;
pub(super) const ASK_NEWGUOSE_TARGET_PINDIAN: u32 = 28;
pub(super) const ASK_NEWTONGQUE_CONFIRM: u32 = 29;
pub(super) const ASK_NEWTIANXIANG_PLAY_TARGET: u32 = 30;
pub(super) const ASK_NEWTIANXIANG_DISCARD: u32 = 31;
pub(super) const ASK_NEWTIANXIANG_DEFLECT_TARGET: u32 = 32;
pub(super) const ASK_NEWLUOYI_CONFIRM: u32 = 33;
pub(super) const ASK_ZHENHUN_CONFIRM: u32 = 34;
pub(super) const ASK_ZHENHUN_TARGET: u32 = 35;
pub(super) const ASK_NEWLUOSHEN_CONFIRM: u32 = 36;

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

pub(super) fn is_black_suit(s: Suit) -> bool {
    matches!(s, Suit::Spade | Suit::Club)
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

pub(super) fn card_type_tag(catalog: &Catalog, c: CardId) -> u8 {
    let kind = catalog.card_kind(c);
    if kind.is_basic_kind() {
        0
    } else if kind.is_equip() {
        2
    } else {
        1
    }
}

pub(super) fn empty_zone_count(state: &GameStateCore, p: PlayerId) -> u32 {
    let pl = state.player(p);
    let mut cnt = 0;
    if pl.hand.is_empty() {
        cnt += 1;
    }
    if pl.equips.is_empty() {
        cnt += 1;
    }
    if pl.judge.is_empty() {
        cnt += 1;
    }
    cnt
}

pub(super) fn has_mount_or_yucong(
    state: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> bool {
    flag(state, p, "l25_yucong_mounted") > 0
        || state
            .vm_aux
            .piles
            .get(&(p, "ssr_newyucong".to_string()))
            .is_some_and(|pile| !pile.is_empty())
        || state.player(p).equips.iter().any(|&c| {
            matches!(
                catalog.card_kind(c).equip_slot(),
                Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            )
        })
}

pub(super) fn baiyi_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let eqs: Vec<CardId> = he_cards(core, owner)
        .into_iter()
        .filter(|&c| catalog.card_kind(c).is_equip())
        .collect();
    if !eqs.is_empty() {
        eqs
    } else {
        he_cards(core, owner)
    }
}

pub(super) fn tongque_dice_count(core: &GameStateCore, p: PlayerId) -> u32 {
    if flag(core, p, "l25_tongque_init") == 0 {
        flag(core, p, "l25_tongque_dice")
            .max(flag(core, p, "mark:ssr_newtongque"))
            .max(2)
    } else {
        flag(core, p, "l25_tongque_dice").max(flag(core, p, "mark:ssr_newtongque"))
    }
}

pub(super) fn set_tongque_dice(core: &mut GameStateCore, p: PlayerId, val: u32) {
    set_flag(core, p, "l25_tongque_init", 1);
    set_flag(core, p, "l25_tongque_dice", val);
    set_flag(core, p, "mark:ssr_newtongque", val);
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

pub(in crate::content::host) fn trigger_meta_l25_batch_d(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_fenchuan" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnEnded],
        )),
        "ssr_newlihun" => Some(meta(EventKind::TurnEnded, Vec::new())),
        "ssr_biyue" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::TurnBegin,
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
                EventKind::TurnPhaseEnd,
                EventKind::PhaseEnd,
            ],
        )),
        "ssr_yjixi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin],
        )),
        "ssr_yanji" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
                EventKind::TurnEnded,
            ],
        )),
        "ssr_cuandi" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_yongsi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::CardUsed],
        )),
        "ssr_wusheng" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnEnded],
        )),
        "ssr_yijue" => Some(meta(EventKind::TurnEnded, Vec::new())),
        "ssr_newbaiyi" => Some(meta(
            EventKind::TurnPhaseEnd,
            vec![EventKind::PhaseBegin, EventKind::PhaseEnd, EventKind::TurnEnded],
        )),
        "ssr_liecheng" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "ssr_newyucong" => Some(meta(
            EventKind::Damage,
            vec![EventKind::TurnEnded],
        )),
        "ssr_newyicong" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_newguose" => Some(meta(EventKind::TurnEnded, Vec::new())),
        "ssr_newtongque" => Some(meta(
            EventKind::DamageBefore,
            vec![
                EventKind::Damage,
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
            ],
        )),
        "ssr_newtianxiang" => Some(meta(
            EventKind::DamageBefore,
            vec![EventKind::Damage],
        )),
        "ssr_huchi" | "ssr_luoyi" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_newluoyi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::DamageBefore, EventKind::Damage],
        )),
        "ssr_zhenhun" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::Damage, EventKind::TurnEnded],
        )),
        "ssr_paoxiao" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnEnded],
        )),
        "ssr_newluoshen" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnPhaseEnd, EventKind::PhaseEnd],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_d(name: &str) -> SkillLimits {
    match name {
        "ssr_newlihun" | "ssr_yongsi" | "ssr_yijue" | "ssr_liecheng" | "ssr_newyucong"
        | "ssr_newguose" | "ssr_newtianxiang" => SkillLimits {
            once_per_turn: true,
            ..SkillLimits::default()
        },
        _ => SkillLimits::default(),
    }
}

fn make_active_machine(name: &str, owner: PlayerId) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    Some(Box::new(HostMachine {
        def_key: name.to_string(),
        behavior: HostBehavior::L25ZuanBatchD,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn activate_l25_batch_d(
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
        "ssr_newlihun" => {
            if used_this_turn(state, owner, "ssr_newlihun") >= 1
                || pl.hand.is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_newlihun", owner)
        }
        "ssr_yongsi" => {
            if used_this_turn(state, owner, "ssr_yongsi") >= 1
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
            {
                return None;
            }
            make_active_machine("ssr_yongsi", owner)
        }
        "ssr_yijue" => {
            if used_this_turn(state, owner, "ssr_yijue") >= 1
                || he_cards(state, owner).is_empty()
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_yijue", owner)
        }
        "ssr_newbaiyi" => {
            if used_this_turn(state, owner, "ssr_newbaiyi") >= 2
                || baiyi_eligible_cards(state, catalog, owner).is_empty()
                || !alive_all(state, owner)
                    .into_iter()
                    .any(|p| flag(state, p, "l25_baiyi_du") == 0)
            {
                return None;
            }
            make_active_machine("ssr_newbaiyi", owner)
        }
        "ssr_liecheng" => {
            if used_this_turn(state, owner, "ssr_liecheng") >= 1
                || !alive_others(state, owner)
                    .into_iter()
                    .any(|p| !he_cards(state, p).is_empty())
            {
                return None;
            }
            make_active_machine("ssr_liecheng", owner)
        }
        "ssr_newyucong" => {
            if used_this_turn(state, owner, "ssr_newyucong") >= 1
                || he_cards(state, owner).is_empty()
                || !alive_all(state, owner)
                    .into_iter()
                    .any(|p| flag(state, p, "l25_yucong_mounted") == 0)
            {
                return None;
            }
            make_active_machine("ssr_newyucong", owner)
        }
        "ssr_newguose" => {
            if used_this_turn(state, owner, "ssr_newguose") >= 1
                || alive_others(state, owner).is_empty()
            {
                return None;
            }
            make_active_machine("ssr_newguose", owner)
        }
        "ssr_newtianxiang" => {
            if used_this_turn(state, owner, "ssr_newtianxiang") >= 1
                || tongque_dice_count(state, owner) == 0
            {
                return None;
            }
            make_active_machine("ssr_newtianxiang", owner)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_l25_batch_d(
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
        "ssr_fenchuan" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(catalog, u)
                    && u.targets
                        .iter()
                        .any(|&t| t != owner && !state.player(t).hand.is_empty())
            }
            EventData::TurnEnded { player } => {
                *player == owner
                    && (flag(state, owner, "l25_fenchuan_unresp") > 0
                        || flag(state, owner, "l25_fenchuan_dmg_boost") > 0)
            }
            _ => false,
        },
        "ssr_newlihun" => match event {
            EventData::TurnEnded { .. } => {
                flag(state, owner, "l25_lihun_zero_hand") > 0
                    || flag(state, owner, "l25_lihun_qianxing") > 0
                    || alive_all(state, owner)
                        .into_iter()
                        .any(|p| flag(state, p, "l25_lihun_controlled") > 0)
            }
            _ => false,
        },
        "ssr_biyue" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                *player == owner
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            }
            | EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => *player == owner && empty_zone_count(state, owner) > 0,
            _ => false,
        },
        "ssr_yjixi" => match event {
            EventData::TurnBegin { player } => *player == owner && flag(state, owner, "l25_yjixi_init") == 0,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                if *player == owner {
                    flag(state, owner, "l25_yjixi_init") == 0
                } else {
                    flag(state, *player, "l25_yjixi_has_yuxi") > 0
                        && (state.player(*player).hand.len() as i32) > state.player(*player).hp
                        && !he_cards(state, *player).is_empty()
                }
            }
            _ => false,
        },
        "ssr_yanji" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_sha_use(catalog, u)
                    && !he_cards(state, owner).is_empty()
                    && !alive_others(state, owner).is_empty()
            }
            EventData::TurnEnded { .. } => {
                flag(state, owner, "l25_yanji_extra_target") > 0
                    || alive_all(state, owner)
                        .into_iter()
                        .any(|p| flag(state, p, "l25_yanji_taunted_by") > 0)
            }
            _ => false,
        },
        "ssr_cuandi" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_sha_use(catalog, u)
                    && !he_cards(state, owner).is_empty()
                    && !alive_others(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_yongsi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && pl.hp < pl.max_hp as i32,
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_sha_use(catalog, u)
                    && !he_cards(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_wusheng" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(catalog, u)
                    && (state.turn.jiu_active || flag(state, owner, "l25_wusheng_jiu") > 0)
            }
            EventData::TurnEnded { .. } => flag(state, owner, "l25_wusheng_drunk_unresp") > 0,
            _ => false,
        },
        "ssr_yijue" => match event {
            EventData::TurnEnded { .. } => {
                flag(state, owner, "l25_yijue_mashu") > 0
                    || alive_all(state, owner)
                        .into_iter()
                        .any(|p| flag(state, p, "l25_yijue_locked") > 0)
            }
            _ => false,
        },
        "ssr_newbaiyi" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            }
            | EventData::TurnEnded { player } => flag(state, *player, "l25_baiyi_du") > 0,
            _ => false,
        },
        "ssr_liecheng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && alive_others(state, owner)
                        .into_iter()
                        .any(|p| !he_cards(state, p).is_empty())
            }
            _ => false,
        },
        "ssr_newyucong" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && flag(state, d.target, "l25_yucong_mounted") > 0
            }
            EventData::TurnEnded { player } => flag(state, *player, "l25_yucong_mounted") > 0,
            _ => false,
        },
        "ssr_newyicong" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(catalog, u)
                    && u.targets
                        .iter()
                        .any(|&t| has_mount_or_yucong(state, catalog, t))
            }
            _ => false,
        },
        "ssr_newguose" => match event {
            EventData::TurnEnded { player } => flag(state, *player, "l25_guose_skip_play") > 0,
            _ => false,
        },
        "ssr_newtongque" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_tongque_init") == 0,
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && tongque_dice_count(state, owner) > 0
            }
            _ => false,
        },
        "ssr_newtianxiang" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && (d.target == owner || flag(state, d.target, "l25_tianxiang_die") > 0)
                    && !he_cards(state, owner).is_empty()
                    && !alive_others(state, owner).is_empty()
            }
            _ => false,
        },
        "ssr_huchi" | "ssr_luoyi" => match event {
            EventData::CardUsed(u) => u.player == owner && is_sha_or_duel_use(catalog, u),
            _ => false,
        },
        "ssr_newluoyi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_luoyi_active") == 0,
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            _ => false,
        },
        "ssr_zhenhun" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner && !alive_others(state, owner).is_empty(),
            EventData::Damage(d) => {
                d.source == Some(owner) && !d.prevented && d.amount > 0 && d.target != owner
            }
            EventData::TurnEnded { .. } => alive_all(state, owner)
                .into_iter()
                .any(|p| flag(state, p, "l25_zhenhun_forbid_sha") > 0),
            _ => false,
        },
        "ssr_paoxiao" => match event {
            EventData::CardUsed(u) => u.player == owner && is_sha_use(catalog, u),
            EventData::TurnEnded { player } => {
                *player == owner && flag(state, owner, "l25_paoxiao_sha_count") > 0
            }
            _ => false,
        },
        "ssr_newluoshen" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            }
            | EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => *player == owner,
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_d(
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
            begin: true,
        }
        | EventData::Phase {
            player,
            phase: Phase::End,
            begin: false,
        } => {
            target = Some(*player);
            card = Some(111);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L25ZuanBatchD,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l25_batch_d(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_d_impl(m, ctx)
}
