//! L25 祖安武将 Batch A（4 张正式卡牌 + 23 个全局/模式/神将技能 + 77 个被动展示/语音钩子，
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）。

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l25_batch_a_asks.rs"]
mod l25_batch_a_asks;
#[path = "l25_batch_a_ext.rs"]
mod l25_batch_a_ext;

pub(in crate::content::host) use self::l25_batch_a_asks::ask_l25_batch_a;
use self::l25_batch_a_asks::handle_l25_batch_a_impl;
use self::l25_batch_a_ext::{filter_ui_hook, is_l25_ui_hook, ui_hook_trigger_meta};

pub(super) const ASK_XICHENG_TARGET: u32 = 1;
pub(super) const ASK_XICHENG_SHOW_CARD: u32 = 2;
pub(super) const ASK_WEICHENG_TARGET: u32 = 3;
pub(super) const ASK_WEICHENG_GIVE_CARD: u32 = 4;
pub(super) const ASK_WEICHENG_CONFIRM_SHA: u32 = 5;
pub(super) const ASK_SHOUCHENG_CHOOSE_CARD: u32 = 6;
pub(super) const ASK_YUXI_TARGET: u32 = 7;
pub(super) const ASK_YUXI_ACTION: u32 = 8;
pub(super) const ASK_YUXI_DISCARD_TARGET: u32 = 9;
pub(super) const ASK_YUXI_GIVE_CARD: u32 = 10;
pub(super) const ASK_G_DU_TARGET: u32 = 11;
pub(super) const ASK_YONGJIN_SRC: u32 = 12;
pub(super) const ASK_YONGJIN_EQUIP: u32 = 13;
pub(super) const ASK_YONGJIN_DST: u32 = 14;
pub(super) const ASK_KONGCHENG_CONFIRM: u32 = 15;
pub(super) const ASK_LUNJIAN_TARGET: u32 = 16;
pub(super) const ASK_WUMOU_MODE: u32 = 17;
pub(super) const ASK_NEW_GUIXIN_CONFIRM: u32 = 18;
pub(super) const ASK_KUANGFENG_TARGET: u32 = 19;
pub(super) const ASK_DAWU_TARGETS: u32 = 20;
pub(super) const ASK_QINYIN_MODE: u32 = 21;
pub(super) const ASK_NEW_WUHUN_TARGET: u32 = 22;
pub(super) const ASK_JIEYING_TARGET: u32 = 23;

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

pub(super) fn hej_cards(state: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = state.player(p);
    pl.hand
        .iter()
        .chain(pl.equips.iter())
        .chain(pl.judge.iter())
        .copied()
        .collect()
}

pub(super) fn effective_sha_kind(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> CardKind {
    crate::rvm_bridge::modifiers::d34_flags::find_d34_flag_view_as(
        state,
        catalog,
        owner,
        c,
        true,
        Some(CardKind::Sha),
    )
    .map(|s| s.as_kind)
    .unwrap_or_else(|| catalog.card_kind(c))
}

pub(super) fn is_sha_card(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        effective_sha_kind(state, catalog, owner, c),
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
    )
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let kind = u
        .use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .unwrap_or(CardKind::Sha);
    u.use_as.is_some_and(|k| matches!(k, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
        || (u.card.is_some() && matches!(kind, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
}

pub(super) fn is_normal_trick_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|c| catalog.card_kind(c))) else {
        return false;
    };
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

pub(super) fn is_du_card(state: &GameStateCore, catalog: &Catalog, owner: PlayerId, c: CardId) -> bool {
    catalog
        .cards
        .get(c as usize)
        .is_some_and(|d| d.name == "du" || d.name == "毒")
        || flag(state, owner, &format!("is_du_card:{c}")) > 0
}

pub(super) fn yongjin_movable_equips(
    state: &GameStateCore,
    catalog: &Catalog,
    src: PlayerId,
) -> Vec<CardId> {
    let others = alive_others(state, src);
    state
        .player(src)
        .equips
        .iter()
        .copied()
        .filter(|&eq| {
            let Some(slot) = catalog.card_kind(eq).equip_slot() else {
                return false;
            };
            others.iter().any(|&dst| {
                !state
                    .player(dst)
                    .equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(slot))
            })
        })
        .collect()
}

pub(super) fn yongjin_sources(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    alive_all(state, owner)
        .into_iter()
        .filter(|&p| !yongjin_movable_equips(state, catalog, p).is_empty())
        .collect()
}

pub(super) fn yongjin_destinations(
    state: &GameStateCore,
    catalog: &Catalog,
    src: PlayerId,
    eq: CardId,
) -> Vec<PlayerId> {
    let Some(slot) = catalog.card_kind(eq).equip_slot() else {
        return Vec::new();
    };
    alive_others(state, src)
        .into_iter()
        .filter(|&dst| {
            !state
                .player(dst)
                .equips
                .iter()
                .any(|&c| catalog.card_kind(c).equip_slot() == Some(slot))
        })
        .collect()
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

pub(in crate::content::host) fn trigger_meta_l25_batch_a(name: &str) -> Option<TriggerMeta> {
    if name == "_zuansaidskill" {
        // Upstream `extension.js:46503` is commented out (`/* ... */`); never bind as active or trigger.
        return None;
    }
    if is_l25_ui_hook(name) {
        return ui_hook_trigger_meta(name);
    }
    match name {
        "ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "g_du" => Some(meta(
            EventKind::CardsLost,
            vec![EventKind::CardsDiscarded, EventKind::PhaseBegin],
        )),
        "_kongchengskill" => Some(meta(EventKind::CardsLost, vec![EventKind::CardsDiscarded])),
        "_anyeskill" => Some(TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::First,
            priority: 10,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "_taohuaskill" | "wumou" | "ol_wuqian" | "wushen" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "_lunjianskill" | "_thunderjudgemodeskill" | "nzry_longnu" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "_jianghunskill" => Some(meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin])),
        "_thundermodeskill" => Some(meta(EventKind::PhaseEnd, vec![EventKind::PhaseBegin])),
        "_goodluckjuedgeskill" => Some(meta(EventKind::OrdinaryJudge, Vec::new())),
        "new_guixin" => Some(meta(EventKind::Damage, Vec::new())),
        "kuangfeng" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::First,
            priority: 8,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::TurnBegin, EventKind::DamageBefore],
        }),
        "dawu" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "dawu2" => Some(TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::First,
            priority: 10,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "qinyin" => Some(meta(
            EventKind::PhaseEnd,
            vec![EventKind::PhaseBegin, EventKind::CardsDiscarded],
        )),
        "xinjuejing" => Some(meta(
            EventKind::Dying,
            vec![EventKind::Draw, EventKind::PhaseBegin],
        )),
        "renjie" => Some(meta(EventKind::Damage, vec![EventKind::CardsDiscarded])),
        "new_wuhun" => Some(meta(EventKind::Damage, vec![EventKind::Death])),
        "g_nzry_jieying" => Some(meta(
            EventKind::TurnBegin,
            vec![EventKind::PhaseBegin, EventKind::TurnPhaseEnd],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l25_batch_a(name: &str) -> SkillLimits {
    match name {
        "ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi" | "_yongjinskill" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

fn discard_phase_count(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "qinyin_discard_stamp") == turn_stamp(core) {
        flag(core, owner, "qinyin_discard_count")
    } else {
        0
    }
}

pub(in crate::content::host) fn activate_l25_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let name = skill.name.as_str();
    // All 77 passive UI/VFX/speech hooks and `_zuansaidskill` MUST never activate in Play phase.
    if name == "_zuansaidskill" || is_l25_ui_hook(name) {
        return None;
    }
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return None;
    }
    let can_activate = match name {
        "ssr_newsance_xicheng" | "ssr_newsance_weicheng" => !alive_others(state, owner).is_empty(),
        "ssr_newsance_shoucheng" => !state.deck.is_empty() || !state.discard.is_empty(),
        "ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi" => {
            let others = alive_others(state, owner);
            !others.is_empty()
                && (!state.player(owner).hand.is_empty()
                    || others.iter().any(|&p| !state.player(p).hand.is_empty()))
        }
        "_yongjinskill" => !yongjin_sources(state, catalog, owner).is_empty(),
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

pub(in crate::content::host) fn filter_l25_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let name = skill.name.as_str();
    if name == "_zuansaidskill" {
        return false;
    }
    if is_l25_ui_hook(name) {
        return filter_ui_hook(name, ctx, owner, event);
    }
    if name != "new_wuhun" && !ctx.state.player(owner).alive {
        return false;
    }
    match name {
        "ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        "g_du" => match event {
            EventData::CardsLost {
                player,
                cards,
                origins,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "g_du_gifting") == 0
                    && origins.contains(&LostOrigin::Hand)
                    && (cards
                        .iter()
                        .any(|&c| is_du_card(ctx.state, ctx.catalog, owner, c))
                        || flag(ctx.state, owner, "g_du_lose_count") > 0)
            }
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && flag(ctx.state, owner, "g_du_gifting") == 0
                    && (cards
                        .iter()
                        .any(|&c| is_du_card(ctx.state, ctx.catalog, owner, c))
                        || flag(ctx.state, owner, "g_du_lose_count") > 0)
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && !alive_others(ctx.state, owner).is_empty()
                    && (ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|&c| is_du_card(ctx.state, ctx.catalog, owner, c))
                        || (flag(ctx.state, owner, "has_du_in_hand") > 0
                            && !ctx.state.player(owner).hand.is_empty()))
            }
            _ => false,
        },
        "_kongchengskill" => match event {
            EventData::CardsLost { player, cards, .. }
            | EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.player(owner).hand.is_empty()
                    && ctx.state.turn.current != owner
            }
            _ => false,
        },
        "_anyeskill" => matches!(
            event,
            EventData::Damage(d)
                if (d.target == owner || d.source == Some(owner))
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Normal
        ),
        "_taohuaskill" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && u.card.is_some_and(|c| {
                        ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Heart
                    })
        ),
        "_lunjianskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && !alive_others(ctx.state, owner).is_empty()
        ),
        "_jianghunskill" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "l25_jianghun_granted") == 0,
            _ => false,
        },
        "_thunderjudgemodeskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Judge,
                begin: true,
            } if *player == owner
        ),
        "_thundermodeskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } if *player == owner
        ),
        "_goodluckjuedgeskill" => matches!(
            event,
            EventData::OrdinaryJudge {
                owner: judge_owner,
                ..
            } if *judge_owner == owner
        ),
        "wumou" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner && is_normal_trick_use(ctx.catalog, u)
        ),
        "ol_wuqian" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.targets.is_empty()
                    && (is_sha_use(ctx.catalog, u)
                        || u.use_as == Some(CardKind::Duel)
                        || u.card.is_some_and(|c| ctx.catalog.card_kind(c) == CardKind::Duel))
        ),
        "new_guixin" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && alive_others(ctx.state, owner)
                        .iter()
                        .any(|&p| !hej_cards(ctx.state, p).is_empty())
        ),
        "kuangfeng" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && (!ctx.state.player(owner).off_court.is_empty()
                        || flag(ctx.state, owner, "mark:qixing") > 0)
            }
            EventData::TurnBegin { player } => {
                *player == owner
                    && ctx
                        .state
                        .alive_players()
                        .any(|p| flag(ctx.state, p, "kuangfeng_marked") > 0)
            }
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Fire
                    && flag(ctx.state, d.target, "kuangfeng_marked") > 0
            }
            _ => false,
        },
        "dawu" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && (!ctx.state.player(owner).off_court.is_empty()
                        || flag(ctx.state, owner, "mark:qixing") > 0)
            }
            EventData::TurnBegin { player } => {
                *player == owner
                    && ctx
                        .state
                        .alive_players()
                        .any(|p| flag(ctx.state, p, "dawu_marked") > 0)
            }
            _ => false,
        },
        "dawu2" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Thunder
                    && (d.target == owner || flag(ctx.state, d.target, "dawu_marked") > 0)
        ),
        "qinyin" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Discard
            }
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } => *player == owner && discard_phase_count(ctx.state, owner) >= 2,
            _ => false,
        },
        "xinjuejing" => match event {
            EventData::Dying { player, .. } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "renjie" => match event {
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Discard
            }
            _ => false,
        },
        "wushen" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner && is_sha_use(ctx.catalog, u)
        ),
        "new_wuhun" => match event {
            EventData::Damage(d) => {
                ctx.state.player(owner).alive
                    && d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source
                        .is_some_and(|s| s != owner && ctx.state.player(s).alive)
            }
            EventData::Death { player, .. } => {
                *player == owner
                    && alive_others(ctx.state, owner)
                        .iter()
                        .any(|&p| flag(ctx.state, p, "mark:mengyan") > 0)
            }
            _ => false,
        },
        "g_nzry_jieying" => match event {
            EventData::TurnBegin { player } => {
                *player == owner
                    && ctx
                        .state
                        .alive_players()
                        .all(|p| flag(ctx.state, p, "mark:ying") == 0)
            }
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => flag(ctx.state, *player, "mark:ying") > 0,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "mark:ying") > 0
                    && !alive_others(ctx.state, owner).is_empty()
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => *player != owner && flag(ctx.state, *player, "mark:ying") > 0,
            _ => false,
        },
        "nzry_longnu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l25_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("ssr_yongsi_yuxi_card" | "ssr_yongsi_yuxi", EventData::Phase { .. }) => {
            card = Some(100);
        }
        ("g_du", EventData::CardsLost { cards: cs, .. } | EventData::CardsDiscarded { cards: cs, .. }) => {
            card = Some(1);
            cards = cs.clone();
        }
        ("g_du", EventData::Phase { .. }) => {
            card = Some(2);
        }
        ("ol_wuqian", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
        }
        ("new_guixin", EventData::Damage(d)) => {
            cards = vec![d.amount.max(1) as u32];
        }
        ("kuangfeng", EventData::Phase { .. }) => {
            card = Some(1);
        }
        ("kuangfeng", EventData::TurnBegin { .. }) => {
            card = Some(2);
        }
        ("kuangfeng", EventData::Damage(d)) => {
            card = Some(3);
            target = Some(d.target);
        }
        ("dawu", EventData::Phase { .. }) => {
            card = Some(1);
        }
        ("dawu", EventData::TurnBegin { .. }) => {
            card = Some(2);
        }
        ("dawu2", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("qinyin", EventData::CardsDiscarded { cards: cs, .. }) => {
            cards = cs.clone();
        }
        ("xinjuejing", EventData::Dying { .. }) => {
            card = Some(1);
        }
        ("xinjuejing", EventData::Phase { .. }) => {
            card = Some(2);
        }
        ("renjie", EventData::Damage(d)) => {
            cards = vec![d.amount.max(1) as u32];
        }
        ("renjie", EventData::CardsDiscarded { cards: cs, .. }) => {
            cards = vec![cs.len().max(1) as u32];
        }
        ("wushen", EventData::CardUsed(u)) => {
            card = u.card.map(u32::from);
        }
        ("new_wuhun", EventData::Damage(d)) => {
            card = Some(1);
            target = d.source;
            cards = vec![d.amount.max(1) as u32];
        }
        ("new_wuhun", EventData::Death { .. }) => {
            card = Some(2);
        }
        ("g_nzry_jieying", EventData::TurnBegin { .. }) => {
            card = Some(1);
        }
        ("g_nzry_jieying", EventData::Phase { phase: Phase::Draw, player, .. }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("g_nzry_jieying", EventData::Phase { phase: Phase::End, .. }) => {
            card = Some(3);
        }
        ("g_nzry_jieying", EventData::TurnEndWindow { player, .. }) => {
            card = Some(4);
            target = Some(*player);
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

pub(in crate::content::host) fn handle_l25_batch_a(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_l25_batch_a_impl(m, ctx)
}
