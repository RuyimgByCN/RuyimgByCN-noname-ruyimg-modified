//! L25 `祖安武将` Batch C（22 技能，SSR 武将 #10–#20）——状态机与触发实现。
//!
//! 覆盖武将：
//! - 郭嘉（`ssr_bailun`、`ssr_newyice`）
//! - 曹操（`ssr_tumeng`、`ssr_zhujiu`、`ssr_hujia`）
//! - 吕布（`ssr_shenfen`、`ssr_kuangbao`、`ssr_wushuang`、`ssr_jiwu`）
//! - 黄忠（`ssr_liegong`、`ssr_pojia`）
//! - 夏侯惇（`ssr_newganglie`、`ssr_zuni`）
//! - 华佗（`ssr_chuli`、`ssr_xianfang`）
//! - 于禁（`ssr_newzhenjun`、`ssr_newyizhong`）
//! - 左慈（`ssr_shendao`、`ssr_newjiahe`）
//! - 华雄（`ssr_miedi`、`ssr_shiyong`）
//! - 黄盖（`ssr_newkurou`）

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l25_batch_c_asks.rs"]
mod l25_batch_c_asks;
#[path = "l25_batch_c_ext.rs"]
mod l25_batch_c_ext;

pub(in crate::content::host) use self::l25_batch_c_asks::ask_l25_batch_c;
use self::l25_batch_c_asks::handle_l25_batch_c_impl;

pub(super) const ASK_NEWYICE_CARD: u32 = 1;
pub(super) const ASK_NEWYICE_TARGET: u32 = 2;
pub(super) const ASK_NEWYICE_MODE: u32 = 3;
pub(super) const ASK_TUMENG_TARGET: u32 = 4;
pub(super) const ASK_ZHUJIU_DISCARD: u32 = 5;
pub(super) const ASK_ZHUJIU_TARGET: u32 = 6;
pub(super) const ASK_HUJIA_CONFIRM: u32 = 7;
pub(super) const ASK_HUJIA_GIVE: u32 = 8;
pub(super) const ASK_LIEGONG_DISCARD: u32 = 9;
pub(super) const ASK_LIEGONG_EQUIP: u32 = 10;
pub(super) const ASK_NEWGANGLIE_CONFIRM: u32 = 11;
pub(super) const ASK_NEWGANGLIE_SHA: u32 = 12;
pub(super) const ASK_ZUNI_DISCARD: u32 = 13;
pub(super) const ASK_CHULI_TARGET: u32 = 14;
pub(super) const ASK_CHULI_CARD: u32 = 15;
pub(super) const ASK_XIANFANG_CONFIRM: u32 = 16;
pub(super) const ASK_NEWZHENJUN_TARGET: u32 = 17;
pub(super) const ASK_SHENDAO_DISCARD: u32 = 18;
pub(super) const ASK_SHENDAO_TARGET: u32 = 19;
pub(super) const ASK_SHENDAO_OPTION: u32 = 20;
pub(super) const ASK_NEWJIAHE_CONFIRM: u32 = 21;
pub(super) const ASK_MIEDI_TARGET: u32 = 22;
pub(super) const ASK_MIEDI_CHOICE: u32 = 23;
pub(super) const ASK_MIEDI_GIVE: u32 = 24;
pub(super) const ASK_NEWKUROU_TARGET: u32 = 25;
pub(super) const ASK_NEWKUROU_CARD: u32 = 26;

pub(super) fn flag(core: &GameStateCore, p: PlayerId, k: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, k.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, k: &str, v: u32) {
    if v == 0 {
        core.vm_aux.flags.remove(&(p, k.to_string()));
    } else {
        core.vm_aux.flags.insert((p, k.to_string()), v);
    }
}

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn
        .round
        .saturating_mul(64)
        .saturating_add(u32::from(core.turn.current) + 1)
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

pub(super) fn he_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    let mut out = pl.hand.clone();
    out.extend(pl.equips.iter().copied());
    out
}

pub(super) fn card_suit(catalog: &Catalog, c: CardId) -> Suit {
    catalog
        .cards
        .get(c as usize)
        .map(|d| d.suit)
        .unwrap_or(Suit::Spade)
}

pub(super) fn is_black_suit(s: Suit) -> bool {
    matches!(s, Suit::Spade | Suit::Club)
}

pub(super) fn is_red_suit(s: Suit) -> bool {
    matches!(s, Suit::Heart | Suit::Diamond)
}

pub(super) fn black_he_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    he_cards(core, p)
        .into_iter()
        .filter(|&c| is_black_suit(card_suit(catalog, c)))
        .collect()
}

pub(super) fn is_weapon_card(catalog: &Catalog, c: CardId) -> bool {
    catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon)
        || catalog
            .cards
            .get(c as usize)
            .is_some_and(|d| d.name == "qibaodao" || d.name == "七宝刀")
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

pub(super) fn is_damage_card_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c))) else {
        return false;
    };
    matches!(
        kind,
        CardKind::Sha
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::Duel
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Linghunzhihuo
            | CardKind::ShouLiJian
            | CardKind::Shuigong
    )
}

pub(super) fn baonu_count(core: &GameStateCore, owner: PlayerId) -> u32 {
    flag(core, owner, "l25_kuangbao_baonu")
        .max(flag(core, owner, "baonu"))
        .max(flag(core, owner, "mark:baonu"))
}

pub(super) fn set_baonu_count(core: &mut GameStateCore, owner: PlayerId, v: u32) {
    let capped = v.min(6);
    set_flag(core, owner, "l25_kuangbao_baonu", capped);
    set_flag(core, owner, "baonu", capped);
    set_flag(core, owner, "mark:baonu", capped);
}

pub(super) fn suit_code(s: Suit) -> u32 {
    match s {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn used_this_turn(core: &GameStateCore, owner: PlayerId, skill_key: &str) -> u32 {
    let stamp_key = format!("{skill_key}_stamp");
    if flag(core, owner, &stamp_key) == turn_stamp(core) {
        flag(core, owner, skill_key)
    } else {
        0
    }
}

pub(super) fn bump_used_this_turn(core: &mut GameStateCore, owner: PlayerId, skill_key: &str) {
    let ts = turn_stamp(core);
    let stamp_key = format!("{skill_key}_stamp");
    let next = used_this_turn(core, owner, skill_key) + 1;
    set_flag(core, owner, &stamp_key, ts);
    set_flag(core, owner, skill_key, next);
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

pub(in crate::content::host) fn trigger_meta_l25_batch_c(name: &str) -> Option<TriggerMeta> {
    match name {
        "ssr_bailun" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::Dying,
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
            ],
        )),
        "ssr_newyice" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
                EventKind::Damage,
                EventKind::LoseHp,
            ],
        )),
        "ssr_tumeng" => Some(meta(
            EventKind::Damage,
            vec![EventKind::TurnBegin, EventKind::PhaseBegin],
        )),
        "ssr_zhujiu" => Some(meta(EventKind::Dying, Vec::new())),
        "ssr_hujia" => Some(meta(EventKind::Dying, Vec::new())),
        "ssr_shenfen" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::Damage],
        )),
        "ssr_kuangbao" => Some(meta(
            EventKind::Damage,
            vec![EventKind::TurnBegin, EventKind::PhaseBegin],
        )),
        "ssr_wushuang" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_liegong" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::Damage,
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
                EventKind::Death,
            ],
        )),
        "ssr_pojia" => Some(meta(EventKind::CardUsed, Vec::new())),
        "ssr_newganglie" => Some(meta(EventKind::Damage, vec![EventKind::CardUsed])),
        "ssr_zuni" => Some(meta(EventKind::CardsDiscarded, Vec::new())),
        "ssr_chuli" => Some(meta(
            EventKind::PhaseEnd,
            vec![EventKind::PhaseBegin, EventKind::TurnEnded],
        )),
        "ssr_xianfang" => Some(meta(EventKind::Dying, Vec::new())),
        "ssr_newzhenjun" => Some(meta(
            EventKind::TurnBegin,
            vec![EventKind::PhaseBegin, EventKind::Death],
        )),
        "ssr_newyizhong" => Some(TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::First,
            priority: 10,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::Damage, EventKind::PhaseBegin],
        }),
        "ssr_newjiahe" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::Damage,
                EventKind::TurnBegin,
                EventKind::PhaseBegin,
            ],
        )),
        "ssr_shiyong" => Some(meta(
            EventKind::CardResponded,
            vec![
                EventKind::ShaDodged,
                EventKind::CardUsed,
                EventKind::Damage,
            ],
        )),
        "ssr_newkurou" => Some(meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_c(name: &str) -> SkillLimits {
    match name {
        "ssr_zhujiu" | "ssr_jiwu" | "ssr_shendao" | "ssr_newkurou" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_l25_batch_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let pl = state.player(owner);
    if !pl.alive {
        return None;
    }
    let ok = match skill.name.as_str() {
        "ssr_newyice" => {
            let max_uses = 1 + flag(state, owner, "l25_yice_extra_uses");
            let used = used_this_turn(state, owner, "l25_yice_used");
            !pl.hand.is_empty()
                && !alive_others(state, owner).is_empty()
                && used < max_uses
        }
        "ssr_zhujiu" => {
            used_this_turn(state, owner, "l25_zhujiu_used") == 0
                && !black_he_cards(state, catalog, owner).is_empty()
        }
        "ssr_jiwu" => baonu_count(state, owner) >= 3,
        "ssr_chuli" => {
            flag(state, owner, "l25_chuli_forbid_turn") != turn_stamp(state)
                && used_this_turn(state, owner, "l25_chuli_used") < 2
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "ssr_newzhenjun" => {
            used_this_turn(state, owner, "l25_zhenjun_used") < 2
                && !alive_others(state, owner).is_empty()
        }
        "ssr_shendao" => pl.hand.len() >= 2,
        "ssr_miedi" => {
            flag(state, owner, "l25_miedi_forbid_turn") != turn_stamp(state)
                && used_this_turn(state, owner, "l25_miedi_used") < 2
                && !alive_others(state, owner).is_empty()
        }
        "ssr_newkurou" => alive_others(state, owner)
            .into_iter()
            .any(|p| !state.player(p).hand.is_empty()),
        _ => false,
    };
    if !ok {
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

pub(in crate::content::host) fn filter_l25_batch_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    let pl = state.player(owner);
    if !pl.alive && !matches!(event, EventData::Death { player, .. } if *player == owner) {
        return false;
    }
    match skill.name.as_str() {
        "ssr_bailun" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::Dying { player, .. } => {
                *player == owner && state.turn.phase != Phase::Discard
            }
            EventData::CardsLost { player, cards, .. }
            | EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && pl.hand.is_empty()
                    && (flag(state, owner, "l25_bailun_dying") > 0 || pl.hp <= 1)
            }
            _ => false,
        },
        "ssr_newyice" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::LoseHp { player, amount } => *player == owner && *amount > 0,
            EventData::CardUsed(u) => {
                flag(state, u.player, "l25_yice_jian") > 0
                    && (is_sha_use(catalog, u) || is_trick_use(catalog, u))
            }
            _ => false,
        },
        "ssr_tumeng" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            _ => false,
        },
        "ssr_zhujiu" => match event {
            EventData::Dying { .. } => {
                used_this_turn(state, owner, "l25_zhujiu_used") == 0
                    && !black_he_cards(state, catalog, owner).is_empty()
            }
            _ => false,
        },
        "ssr_hujia" => matches!(event, EventData::Dying { .. }),
        "ssr_shenfen" => {
            if flag(state, owner, "l25_shenfen_awakened") > 0 {
                return false;
            }
            match event {
                EventData::TurnBegin { player } => {
                    *player == owner && baonu_count(state, owner) >= 6
                }
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true,
                } => *player == owner && baonu_count(state, owner) >= 6,
                EventData::Damage(d) => {
                    !d.prevented
                        && d.amount > 0
                        && (d.source == Some(owner) || d.target == owner)
                        && (baonu_count(state, owner) >= 6
                            || baonu_count(state, owner) + (d.amount as u32) >= 6)
                }
                _ => false,
            }
        }
        "ssr_kuangbao" => match event {
            EventData::TurnBegin { .. } => flag(state, owner, "l25_kuangbao_init") == 0,
            EventData::Phase {
                phase: Phase::Prepare,
                begin: true,
                ..
            } => flag(state, owner, "l25_kuangbao_init") == 0,
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && (d.source == Some(owner) || d.target == owner)
                    && baonu_count(state, owner) < 6
            }
            _ => false,
        },
        "ssr_wushuang" => match event {
            EventData::CardUsed(u) => {
                let kind = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c)));
                (u.player == owner && (is_sha_use(catalog, u) || kind == Some(CardKind::Duel)))
                    || (u.targets.contains(&owner) && kind == Some(CardKind::Duel))
            }
            _ => false,
        },
        "ssr_liegong" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && is_sha_use(catalog, u)
                    && !he_cards(state, owner).is_empty()
            }
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.target != owner
                    && !state.player(d.target).equips.is_empty()
            }
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_liegong_has_exile") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_liegong_has_exile") > 0,
            EventData::Death { player, .. } => {
                *player == owner && flag(state, owner, "l25_liegong_has_exile") > 0
            }
            _ => false,
        },
        "ssr_pojia" => match event {
            EventData::CardUsed(u) => {
                u.player == owner && is_sha_use(catalog, u) && !u.targets.is_empty()
            }
            _ => false,
        },
        "ssr_newganglie" => match event {
            EventData::CardUsed(u) => {
                is_sha_use(catalog, u)
                    && ((u.player == owner
                        && u.targets
                            .iter()
                            .any(|&t| t != owner && state.player(t).hand.len() > 1)
                        && pl.hand.len() > 1)
                        || (u.targets.contains(&owner)
                            && u.player != owner
                            && state.player(u.player).hand.len() > 1
                            && pl.hand.len() > 1))
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source
                        .is_some_and(|s| s != owner && state.player(s).alive)
            }
            _ => false,
        },
        "ssr_zuni" => match event {
            EventData::CardsDiscarded { cards, .. } => {
                !cards.is_empty()
                    && state.turn.phase != Phase::Discard
                    && flag(state, owner, "l25_zuni_busy") == 0
                    && !pl.hand.is_empty()
            }
            _ => false,
        },
        "ssr_chuli" => match event {
            EventData::Phase {
                phase: Phase::Play,
                begin: false,
                ..
            }
            | EventData::TurnEnded { .. } => alive_all(state, owner)
                .into_iter()
                .any(|p| flag(state, p, "l25_chuli_draw_pending") > 0),
            _ => false,
        },
        "ssr_xianfang" => match event {
            EventData::Dying { .. } => {
                let pile_len = state
                    .vm_aux
                    .piles
                    .get(&(owner, "ssr_chuli".to_string()))
                    .map_or(0, Vec::len);
                (pile_len > 0 || flag(state, owner, "l25_chuli_yao") > 0)
                    && flag(state, owner, "l25_xianfang_used_round") != state.turn.round.max(1)
            }
            _ => false,
        },
        "ssr_newzhenjun" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_zhenjun_active") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_zhenjun_active") > 0,
            EventData::Death { player, .. } => {
                *player == owner && flag(state, owner, "l25_zhenjun_active") > 0
            }
            _ => false,
        },
        "ssr_newyizhong" => match event {
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0 && pl.hp > 1,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && (pl.hujia > 0 || flag(state, owner, "l25_newyizhong_red") > 0)
            }
            _ => false,
        },
        "ssr_newjiahe" => match event {
            EventData::CardUsed(u) => {
                u.player != owner
                    && u.targets.contains(&owner)
                    && is_damage_card_use(catalog, u)
            }
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_jiahe_qianxing") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_jiahe_qianxing") > 0,
            _ => false,
        },
        "ssr_shiyong" => match event {
            EventData::CardUsed(u) => {
                u.player != owner && u.targets.contains(&owner) && is_sha_use(catalog, u)
            }
            EventData::ShaDodged { target, .. } => *target == owner,
            EventData::CardResponded { player, card } => {
                *player == owner
                    && (catalog.card_kind(*card) == CardKind::Shan
                        || (!catalog.card_kind(*card).is_basic_kind()
                            && !catalog.card_kind(*card).is_equip()))
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| s != owner)
            }
            _ => false,
        },
        "ssr_newkurou" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(state, owner, "l25_kurou_active") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "l25_kurou_active") > 0,
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_c(
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
        EventData::LoseHp { player, amount } => {
            target = Some(*player);
            card = Some(200 + (*amount).min(20));
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
        EventData::ShaDodged {
            source,
            target: tgt,
            card: c,
        } => {
            target = Some(*source);
            targets = vec![*tgt];
            card = Some(104);
            if let Some(cid) = c {
                cards.push(*cid);
            }
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
            begin: false,
        } => {
            target = Some(*player);
            card = Some(109);
        }
        EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(110);
        }
        EventData::TurnEnded { player } => {
            target = Some(*player);
            card = Some(111);
        }
        EventData::Death { player, .. } => {
            target = Some(*player);
            card = Some(112);
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

pub(in crate::content::host) fn handle_l25_batch_c(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_c_impl(m, ctx)
}
