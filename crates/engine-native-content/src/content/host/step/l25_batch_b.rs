//! L25 祖安武将 Batch B（SSR 武将 #1–#10 共 22 个技能，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Faction, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l25_batch_b_asks.rs"]
mod l25_batch_b_asks;
#[path = "l25_batch_b_ext.rs"]
mod l25_batch_b_ext;

pub(in crate::content::host) use self::l25_batch_b_asks::ask_l25_batch_b;
use self::l25_batch_b_asks::handle_l25_batch_b_impl;

pub(super) const ASK_QIANXUN_CONFIRM: u32 = 1;
pub(super) const ASK_QIANXUN_DISCARD: u32 = 2;
pub(super) const ASK_JUEJI_MODE: u32 = 3;
pub(super) const ASK_JUEJI_TARGET: u32 = 4;
pub(super) const ASK_ZHIHENG_DISCARD: u32 = 5;
pub(super) const ASK_ZHIHENG_TARGET: u32 = 6;
pub(super) const ASK_ZHIHENG_MODE: u32 = 7;
pub(super) const ASK_ZHIHENG_TARGET_CHOICE: u32 = 8;
pub(super) const ASK_ZHIHENG_TARGET_DISCARD: u32 = 9;
pub(super) const ASK_JIUYUAN_CONFIRM: u32 = 10;
pub(super) const ASK_LONGDAN_CARD: u32 = 11;
pub(super) const ASK_LONGDAN_TARGET: u32 = 12;
pub(super) const ASK_POZHEN_TARGET_DISCARD: u32 = 13;
pub(super) const ASK_POZHEN_GAIN_CARD: u32 = 14;
pub(super) const ASK_JINNANG_CARD: u32 = 15;
pub(super) const ASK_JINNANG_TARGET: u32 = 16;
pub(super) const ASK_JINNANG_MODE: u32 = 17;
pub(super) const ASK_BAZHEN_CARD: u32 = 18;
pub(super) const ASK_BAZHEN_TARGET: u32 = 19;
pub(super) const ASK_TUXI_TARGET: u32 = 20;
pub(super) const ASK_MUBING_CARDS: u32 = 21;
pub(super) const ASK_JINFAN_DISCARD: u32 = 22;
pub(super) const ASK_QIXI_TARGET: u32 = 23;
pub(super) const ASK_ZHUIJI_CARD: u32 = 24;
pub(super) const ASK_TIEJI_CONFIRM: u32 = 25;
pub(super) const ASK_FANJIAN_CARD: u32 = 26;
pub(super) const ASK_FANJIAN_TARGET: u32 = 27;
pub(super) const ASK_FANJIAN_SHOW: u32 = 28;
pub(super) const ASK_SHICHOU_CONFIRM: u32 = 29;
pub(super) const ASK_RENDE_CARD: u32 = 30;
pub(super) const ASK_RENDE_TARGET: u32 = 31;
pub(super) const ASK_RENDE_TARGET_CHOICE: u32 = 32;
pub(super) const ASK_JIJIANG_TARGET: u32 = 33;
pub(super) const ASK_QIANGXI_TARGET: u32 = 34;

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

pub(super) fn suit_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
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

pub(super) fn is_trick_or_delayed_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c))) else {
        return false;
    };
    !kind.is_basic_kind() && !kind.is_equip()
}

pub(super) fn is_normal_trick_kind(kind: CardKind) -> bool {
    !kind.is_basic_kind()
        && !kind.is_equip()
        && !matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
}

pub(super) fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    )
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

pub(super) fn zhiheng_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let stamp = turn_stamp(core);
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| flag(core, owner, &format!("l25_zhiheng_forbid:{p}")) != stamp)
        .collect()
}

pub(super) fn jinfan_eligible_cards(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| {
            let suit = catalog.effective_card_suit(core, owner, c);
            matches!(suit, Suit::Spade | Suit::Club) || catalog.card_kind(c).is_equip()
        })
        .collect()
}

pub(super) fn bazhen_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_newbazhen".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn jinfan_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_jinfan".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn tieji_pile(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ssr_tieji".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn qiangxi_marks(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "l25_qiangxi_init") == 0 {
        flag(core, owner, "l25_qiangxi_ji")
            .max(flag(core, owner, "mark:ssr_qiangxi"))
            .max(2)
    } else {
        flag(core, owner, "l25_qiangxi_ji").max(flag(core, owner, "mark:ssr_qiangxi"))
    }
}

pub(super) fn set_qiangxi_marks(core: &mut GameStateCore, owner: PlayerId, val: u32) {
    set_flag(core, owner, "l25_qiangxi_init", 1);
    set_flag(core, owner, "l25_qiangxi_ji", val);
    set_flag(core, owner, "mark:ssr_qiangxi", val);
}

pub(super) fn tuxi_marks(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "l25_tuxi_init") == 0 {
        flag(core, owner, "l25_tuxi_xi")
            .max(flag(core, owner, "mark:ssr_tuxi"))
            .max(2)
    } else {
        flag(core, owner, "l25_tuxi_xi").max(flag(core, owner, "mark:ssr_tuxi"))
    }
}

pub(super) fn set_tuxi_marks(core: &mut GameStateCore, owner: PlayerId, val: u32) {
    set_flag(core, owner, "l25_tuxi_init", 1);
    set_flag(core, owner, "l25_tuxi_xi", val);
    set_flag(core, owner, "mark:ssr_tuxi", val);
}

fn meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 3,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(in crate::content::host) fn trigger_meta_l25_batch_b(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_qianxun" | "ssr_tieji" | "ssr_shichou" => {
            Some(meta(EventKind::CardUsed, vec![EventKind::TurnBegin]))
        }
        "ssr_jiuyuan" => Some(meta(EventKind::CardUsed, vec![EventKind::TurnPhaseEnd])),
        "ssr_jueji" | "ssr_newlongdan" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::CardResponded, EventKind::TurnBegin],
        )),
        "ssr_jianye" => Some(meta(
            EventKind::CardsDrawn,
            vec![EventKind::CardsDiscarded, EventKind::PhaseEnd],
        )),
        "ssr_newpozhen" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::CardResponded, EventKind::TurnBegin],
        )),
        "ssr_jinnang" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "ssr_newbazhen" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "ssr_tuxi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::Death],
        )),
        "ssr_newmubing" => Some(meta(EventKind::CardsDiscarded, Vec::new())),
        "ssr_jinfan" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_qixi" | "ssr_jijiang" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnPhaseEnd],
        )),
        "ssr_zhuiji" => Some(meta(EventKind::Damage, Vec::new())),
        "ssr_youqin" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::Draw],
        )),
        "ssr_newfanjian" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "ssr_newrende" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "ssr_qiangxi" => Some(meta(
            EventKind::Damage,
            vec![EventKind::CardsLost, EventKind::TurnBegin],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_b(name: &str) -> SkillLimits {
    match name {
        "ssr_newfanjian" | "ssr_qingji" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_l25_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return None;
    }
    let name = skill.name.as_str();
    let can = match name {
        "ssr_jueji" => {
            state.player(owner).hand.len() == 1
                && used_this_turn(state, owner, "ssr_jueji") < 3
                && !alive_others(state, owner).is_empty()
        }
        "ssr_zhiheng" => {
            !he_cards(state, owner).is_empty() && !zhiheng_targets(state, owner).is_empty()
        }
        "ssr_newlongdan" => {
            used_this_turn(state, owner, "ssr_newlongdan") < 2
                && state.player(owner).hand.iter().any(|&c| {
                    matches!(
                        catalog.card_kind(c),
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                })
        }
        "ssr_jinnang" => {
            used_this_turn(state, owner, "ssr_jinnang") < 2
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| is_normal_trick_kind(catalog.card_kind(c)) || is_delayed_trick_kind(catalog.card_kind(c)))
        }
        "ssr_newbazhen" => !bazhen_pile(state, owner).is_empty(),
        "ssr_jinfan" => {
            used_this_turn(state, owner, "ssr_jinfan") < 2
                && !jinfan_eligible_cards(state, catalog, owner).is_empty()
                && (!state.deck.is_empty() || !state.discard.is_empty())
        }
        "ssr_newfanjian" => {
            !state.player(owner).hand.is_empty()
                && alive_others(state, owner)
                    .iter()
                    .any(|&p| !state.player(p).hand.is_empty())
        }
        "ssr_newrende" => {
            used_this_turn(state, owner, "ssr_newrende") < 2
                && !state.player(owner).hand.is_empty()
                && !alive_others(state, owner).is_empty()
        }
        "ssr_qiangxi" => {
            used_this_turn(state, owner, "ssr_qiangxi") < 2
                && qiangxi_marks(state, owner) > 0
                && !alive_others(state, owner).is_empty()
        }
        "ssr_qingji" => !state.deck.is_empty() || !state.discard.is_empty(),
        _ => false,
    };
    if !can {
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

pub(in crate::content::host) fn filter_l25_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "ssr_qianxun" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player != owner
                    && u.targets.len() == 1
                    && u.targets[0] == owner
                    && is_trick_or_delayed_use(catalog, u)
                    && state.player(owner).hand.len() != 1
        ),
        "ssr_jueji" => match event {
            EventData::CardUsed(u) if u.player == owner => {
                used_this_turn(state, owner, "ssr_jueji") < 3
                    && state.player(owner).hand.len() <= 1
            }
            EventData::CardResponded { player, .. } if *player == owner => {
                used_this_turn(state, owner, "ssr_jueji") < 3
                    && state.player(owner).hand.len() <= 1
            }
            _ => false,
        },
        "ssr_jianye" => match event {
            EventData::CardsDrawn { player, cards } => {
                state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && !cards.is_empty()
                    && state.player(*player).alive
            }
            EventData::CardsDiscarded { player, cards } => {
                state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && !cards.is_empty()
                    && state.player(*player).alive
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => {
                *player == owner
                    && alive_all(state, owner)
                        .iter()
                        .any(|&p| flag(state, p, "l25_jianye_tongye") > 0 || flag(state, p, "l25_jianye_temp") > 0)
            }
            _ => false,
        },
        "ssr_jiuyuan" => match event {
            EventData::CardUsed(u)
                if is_sha_use(catalog, u) && u.targets.contains(&owner) =>
            {
                alive_others(state, owner).iter().any(|&p| {
                    state.effective_faction(catalog, p) == Some(Faction::Wu)
                        && flag(state, p, "l25_jiuyuan_jibing") == 0
                })
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => flag(state, *player, "l25_jiuyuan_jibing") > 0,
            _ => false,
        },
        "ssr_newlongdan" => match event {
            EventData::TurnBegin { player } if *player == owner => true,
            EventData::CardUsed(u) if u.player == owner => u.use_as.is_some(),
            _ => false,
        },
        "ssr_newpozhen" => match event {
            EventData::TurnBegin { player } if *player == owner => {
                flag(state, owner, "l25_pozhen_zhanyi") > 0
                    || flag(state, owner, "l25_pozhen_chongfeng") > 0
                    || flag(state, owner, "l25_pozhen_unresp") > 0
            }
            EventData::CardUsed(u) => {
                let kind = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c)));
                let is_basic = kind.is_some_and(CardKind::is_basic_kind);
                let is_sha = is_sha_use(catalog, u);
                (u.player == owner && is_basic)
                    || (is_sha
                        && flag(state, owner, "l25_pozhen_zhanyi") > 0
                        && (u.player == owner || u.targets.contains(&owner)))
            }
            EventData::CardResponded { player, card } if *player == owner => {
                catalog.card_kind(*card).is_basic_kind()
            }
            _ => false,
        },
        "ssr_jinnang" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if flag(state, *player, "l25_jinnang_pack") > 0
        ),
        "ssr_newbazhen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && (!state.deck.is_empty() || !state.discard.is_empty())
        ),
        "ssr_tuxi" => match event {
            EventData::TurnBegin { player } if *player == owner => {
                flag(state, owner, "l25_tuxi_init") == 0
            }
            EventData::Death {
                killer: Some(k), ..
            } if *k == owner => true,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner => {
                tuxi_marks(state, owner) > 0
                    && alive_others(state, owner)
                        .iter()
                        .any(|&p| !he_cards(state, p).is_empty())
            }
            _ => false,
        },
        "ssr_newmubing" => matches!(
            event,
            EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && state.turn.current == owner
                    && state.turn.phase == Phase::Discard
                    && cards.len() >= 2
                    && (!state.deck.is_empty() || !state.discard.is_empty())
        ),
        "ssr_jinfan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && is_sha_use(catalog, u)
                    && (flag(state, owner, "l25_jinfan_ling") > 0
                        || !jinfan_pile(state, owner).is_empty())
        ),
        "ssr_qixi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && (!jinfan_pile(state, owner).is_empty()
                    || flag(state, owner, "l25_jinfan_ling") > 0)
                && !alive_others(state, owner).is_empty()
        ),
        "ssr_zhuiji" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && d.target != owner
                    && state.player(d.target).alive
                    && !he_cards(state, d.target).is_empty()
                    && state.computed_distance(catalog, owner, d.target) <= 1
        ),
        "ssr_tieji" => match event {
            EventData::CardUsed(u) => {
                u.player == owner && is_sha_use(catalog, u) && !u.targets.is_empty()
            }
            EventData::TurnBegin { player } if *player == owner => {
                flag(state, owner, "l25_tieji_unresp") > 0
            }
            _ => false,
        },
        "ssr_youqin" => match event {
            EventData::TurnBegin { player } if *player == owner => {
                flag(state, owner, "l25_youqin_hun") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner => !state.deck.is_empty() || !state.discard.is_empty(),
            _ => false,
        },
        "ssr_newfanjian" => matches!(
            event,
            EventData::TurnBegin { player } if *player == owner
                && alive_all(state, owner)
                    .iter()
                    .any(|&p| flag(state, p, "l25_fanjian_forbid_suit") > 0)
        ),
        "ssr_shichou" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(catalog, u)
                    && !u.targets.is_empty()
                    && (!state.deck.is_empty() || !state.discard.is_empty())
            }
            EventData::TurnBegin { player } if *player == owner => alive_all(state, owner)
                .iter()
                .any(|&p| flag(state, p, "l25_shichou_forbid_suit") > 0),
            _ => false,
        },
        "ssr_newrende" => matches!(
            event,
            EventData::TurnEndWindow {
                player,
                after: false,
            } if flag(state, *player, "l25_rende_forbid_dmg_to") > 0
        ),
        "ssr_jijiang" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner => {
                (!state.turn.sha_used_play_phase
                    || !state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|&c| catalog.card_kind(c).is_sha_like()))
                    && alive_others(state, owner).iter().any(|&p| {
                        state.effective_faction(catalog, p) == Some(Faction::Shu)
                            && flag(state, p, "l25_jijiang_sha_plus1") == 0
                    })
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => {
                flag(state, *player, "l25_jijiang_sha_plus1") > 0
            }
            _ => false,
        },
        "ssr_qiangxi" => match event {
            EventData::TurnBegin { player } if *player == owner => {
                flag(state, owner, "l25_qiangxi_init") == 0
            }
            EventData::Damage(d) => !d.prevented && d.amount > 0 && d.target == owner,
            EventData::CardsLost { player, cards, .. } if *player == owner => cards
                .iter()
                .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon)),
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match skill.name.as_str() {
        "ssr_qianxun" => {
            if let EventData::CardUsed(u) = event {
                target = Some(u.player);
                card = u.card;
            }
        }
        "ssr_jueji" => {
            card = Some(100);
        }
        "ssr_jianye" => match event {
            EventData::CardsDrawn { player, cards: cds }
            | EventData::CardsDiscarded { player, cards: cds } => {
                target = Some(*player);
                card = Some(1);
                cards = cds.clone();
            }
            EventData::Phase { .. } => {
                card = Some(2);
            }
            _ => {}
        },
        "ssr_jiuyuan" => match event {
            EventData::CardUsed(u) => {
                card = Some(1);
                targets = u.targets.clone();
            }
            EventData::TurnEndWindow { player, .. } => {
                target = Some(*player);
                card = Some(2);
            }
            _ => {}
        },
        "ssr_newlongdan" => match event {
            EventData::TurnBegin { .. } => card = Some(100),
            EventData::CardUsed(_) => card = Some(101),
            _ => {}
        },
        "ssr_newpozhen" => match event {
            EventData::TurnBegin { .. } => card = Some(100),
            EventData::CardUsed(u) => {
                if u.player == owner {
                    target = u.targets.first().copied();
                    card = Some(if u.use_as.is_some() { 102 } else { 101 });
                    if u.card.is_some() && !u.targets.is_empty() {
                        cards = u.card.into_iter().collect();
                    }
                } else {
                    target = Some(u.player);
                    card = Some(103);
                }
            }
            EventData::CardResponded { card: c, .. } => {
                card = Some(104);
                cards = vec![*c];
            }
            _ => {}
        },
        "ssr_jinnang" => {
            if let EventData::Phase { player, .. } = event {
                target = Some(*player);
                card = Some(100);
            }
        }
        "ssr_newbazhen" => {
            if matches!(event, EventData::Phase { .. }) {
                card = Some(100);
            }
        }
        "ssr_tuxi" => match event {
            EventData::TurnBegin { .. } => card = Some(100),
            EventData::Death { .. } => card = Some(101),
            EventData::Phase { .. } => card = Some(102),
            _ => {}
        },
        "ssr_newmubing" => {
            if let EventData::CardsDiscarded { cards: cds, .. } = event {
                card = Some(cds.len() as u32);
            }
        }
        "ssr_jinfan" => {
            if let EventData::CardUsed(u) = event {
                card = Some(100);
                targets = u.targets.clone();
            }
        }
        "ssr_zhuiji" => {
            if let EventData::Damage(d) = event {
                target = Some(d.target);
            }
        }
        "ssr_tieji" => match event {
            EventData::CardUsed(u) => {
                target = u.targets.first().copied();
                targets = u.targets.clone();
                card = Some(1);
            }
            EventData::TurnBegin { .. } => card = Some(2),
            _ => {}
        },
        "ssr_youqin" => match event {
            EventData::TurnBegin { .. } => card = Some(1),
            EventData::Phase { .. } | EventData::Draw { .. } => card = Some(2),
            _ => {}
        },
        "ssr_newfanjian" => {
            if matches!(event, EventData::TurnBegin { .. }) {
                card = Some(100);
            }
        }
        "ssr_shichou" => match event {
            EventData::CardUsed(u) => {
                target = u.targets.first().copied();
                targets = u.targets.clone();
                card = Some(1);
            }
            EventData::TurnBegin { .. } => card = Some(2),
            _ => {}
        },
        "ssr_newrende" => {
            if let EventData::TurnEndWindow { player, .. } = event {
                target = Some(*player);
                card = Some(100);
            }
        }
        "ssr_jijiang" => match event {
            EventData::Phase { .. } => card = Some(1),
            EventData::TurnEndWindow { player, .. } => {
                target = Some(*player);
                card = Some(2);
            }
            _ => {}
        },
        "ssr_qiangxi" => match event {
            EventData::TurnBegin { .. } => card = Some(100),
            EventData::Damage(d) => {
                card = Some(101);
                cards = vec![d.amount.max(1) as CardId];
            }
            EventData::CardsLost { cards: cds, .. } => {
                card = Some(102);
                cards = cds.clone();
            }
            _ => {}
        },
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L25ZuanBatchB,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l25_batch_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_b_impl(m, ctx)
}
