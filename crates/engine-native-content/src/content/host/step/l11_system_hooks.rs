//! L11（太虚幻境）Batch L11-5 系统、宝物扩展与神器sync（`L11SystemHooks` tag 739：`l11_system_hooks.rs`）。
//!
//! 承载 15 项系统/宝物/神器联动：
//! - `_txhj_skill_maxCards`（主规则在 `d34_l11_mods.rs`）、`_txhj_equip5`（主规则在 `equipment.rs`）
//! - `_txhj_skill_Timelimit`、`_txhj_skill_addEffectStyle`
//! - `_txhj_skill_gaincard`、`_txhj_skill_discard`、`_txhj_skill_usecard`、`_txhj_skill_damage`、`_txhj_skills_kill`
//! - `twsidao`、`twsidao_equip`、`polu`、`polu_damage`、`_txhj_shenqi`、`_txhj_XXXshenqi`

use super::l11_common::*;
use super::l11_system_asks::*;
use super::*;
use crate::catalog::{Catalog, EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DetachedKind, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

const TWSIDAO_CHOICES: [&str; 3] = [
    "txhj_lingbaoxianhu",
    "txhj_taijifuchen",
    "txhj_chongyingshenfu",
];

fn shenqi_buff_for_card_name(name: &str) -> Option<&'static str> {
    match name {
        "txhj_feilong" => Some("buff_txhj_pozhenzhifeng"),
        "txhj_chiyanzhenhunqin" => Some("buff_txhj_yanhuozhiren"),
        "txhj_lingbaoxianhu" => Some("buff_txhj_xianhujiqu"),
        "txhj_taijifuchen" => Some("buff_txhj_fuchendangmo"),
        "txhj_chongyingshenfu" | "txhj_xuwangzhimian" => Some("buff_txhj_shenfuhuaxie"),
        "txhj_guofengyupao" => Some("buff_txhj_juejingzhice"),
        "txhj_juechenjinge" => Some("buff_txhj_wuzhongshengshan"),
        "txhj_liulongcanjia" => Some("buff_txhj_wuzhongshengsha"),
        _ => None,
    }
}

fn card_name(catalog: &Catalog, cid: CardId) -> &str {
    catalog.cards.get(cid as usize).map_or("", |cd| cd.name)
}

fn find_named_in_vec(cards: &[CardId], catalog: &Catalog, name: &str) -> Option<usize> {
    cards.iter().position(|&c| card_name(catalog, c) == name)
}

fn has_named_in_pile(core: &GameStateCore, catalog: &Catalog, name: &str) -> bool {
    find_named_in_vec(&core.deck, catalog, name).is_some()
        || find_named_in_vec(&core.discard, catalog, name).is_some()
}

fn has_named_unclaimed_detached(core: &GameStateCore, catalog: &Catalog, name: &str) -> bool {
    core.detached_cards
        .iter()
        .any(|&(c, kind)| kind == DetachedKind::Unclaimed && card_name(catalog, c) == name)
}

fn take_named_from_pile_or_detached(
    core: &mut GameStateCore,
    catalog: &Catalog,
    name: &str,
    allow_detached: bool,
) -> Option<CardId> {
    if let Some(pos) = find_named_in_vec(&core.deck, catalog, name) {
        return Some(core.deck.remove(pos));
    }
    if let Some(pos) = find_named_in_vec(&core.discard, catalog, name) {
        return Some(core.discard.remove(pos));
    }
    if allow_detached {
        if let Some(pos) = core
            .detached_cards
            .iter()
            .position(|&(c, k)| k == DetachedKind::Unclaimed && card_name(catalog, c) == name)
        {
            return Some(core.detached_cards.remove(pos).0);
        }
        if let Some(pos) = core
            .detached_cards
            .iter()
            .position(|&(c, _)| card_name(catalog, c) == name)
        {
            return Some(core.detached_cards.remove(pos).0);
        }
    }
    None
}

fn pop_card_id_from_pile(core: &mut GameStateCore, cid: CardId) -> bool {
    if let Some(pos) = core.deck.iter().position(|&c| c == cid) {
        core.deck.remove(pos);
        return true;
    }
    if let Some(pos) = core.discard.iter().position(|&c| c == cid) {
        core.discard.remove(pos);
        return true;
    }
    false
}

fn still_equips_artifact_for_buff(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    buff: &str,
) -> bool {
    core.player(owner)
        .equips
        .iter()
        .any(|&c| shenqi_buff_for_card_name(card_name(catalog, c)) == Some(buff))
}

fn wait_sys_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>, stage: HostStage, ask_id: u32) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_system_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_system_hooks(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "_txhj_skill_maxCards" | "_txhj_skill_Timelimit" | "twsidao" => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            5,
            if name == "twsidao" {
                vec![EventKind::PhaseBegin]
            } else {
                Vec::new()
            },
        )),
        "_txhj_skill_addEffectStyle" => {
            Some(meta(EventKind::PhaseBegin, Timing::First, 16, vec![EventKind::TurnBegin]))
        }
        "_txhj_equip5" | "twsidao_equip" | "polu" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new()))
        }
        "_txhj_skill_gaincard" => Some(meta(
            EventKind::CardsDrawn,
            Timing::Normal,
            0,
            vec![EventKind::CardsMoved],
        )),
        "_txhj_skill_discard" => Some(meta(EventKind::CardsDiscarded, Timing::Normal, 0, Vec::new())),
        "_txhj_skill_usecard" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, Vec::new())),
        "_txhj_skill_damage" | "polu_damage" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, Vec::new()))
        }
        "_txhj_skills_kill" => Some(meta(EventKind::Death, Timing::First, 100, Vec::new())),
        "_txhj_shenqi" => Some(meta(EventKind::Equipped, Timing::Normal, 7, Vec::new())),
        "_txhj_XXXshenqi" => Some(meta(EventKind::CardsLost, Timing::Normal, 2, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_system_hooks(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

fn timelimit_outcome(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Option<u32> {
    if flag(state, owner, "l11_timelimit_outcome") != 0 {
        return None;
    }
    if flag(state, owner, "l11_timelimit_repair") > 0 {
        let repaired = (0..state.players.len() as PlayerId).any(|p| {
            p != owner
                && state.player(p).alive
                && state.is_friend_of(catalog, owner, p)
                && (flag(state, p, "l11_timelimit_repair_boss") > 0
                    || flag(state, p, "l11_npc_boss") > 0)
                && state.player(p).hp == state.player(p).max_hp as i32
        });
        if repaired {
            return Some(1);
        }
    }
    let max_r = flag(state, owner, "l11_timelimit_max");
    if max_r > 1 && state.turn.round > max_r {
        Some(if flag(state, owner, "l11_timelimit_subsist") > 0 {
            1
        } else {
            2
        })
    } else {
        None
    }
}

pub(in crate::content::host) fn filter_l11_system_hooks(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "_txhj_skill_Timelimit" => {
            matches!(event, EventData::TurnBegin { .. })
                && timelimit_outcome(state, ctx.catalog, owner).is_some()
        }
        "_txhj_skill_addEffectStyle" => {
            flag(state, owner, "l11_effect_style_init") == 0
                && matches!(event, EventData::TurnBegin { .. } | EventData::Phase { .. })
        }
        "_txhj_skill_gaincard" => match event {
            EventData::CardsDrawn { player, cards } => *player == owner && !cards.is_empty(),
            EventData::CardsMoved { cards, from, to } => {
                *to == owner && *from != owner && !cards.is_empty()
            }
            _ => false,
        },
        "_txhj_skill_discard" => matches!(
            event,
            EventData::CardsDiscarded { player, cards } if *player == owner && !cards.is_empty()
        ),
        "_txhj_skill_usecard" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner && !u.cancelled
        ),
        "_txhj_skill_damage" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && (d.source == Some(owner) || d.target == owner)
        ),
        "_txhj_skills_kill" => matches!(
            event,
            EventData::Death { killer: Some(k), .. } if *k == owner
        ),
        "twsidao" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && flag(state, owner, "l11_twsidao_done") == 0
                && flag(state, owner, "l11_twsidao_card") == 0
                && matches!(event, EventData::TurnBegin { .. } | EventData::Phase { begin: true, .. })
        }
        "twsidao_equip" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
                && flag(state, owner, "l11_twsidao_card")
                    .checked_sub(1)
                    .is_some_and(|cid| state.deck.contains(&cid) || state.discard.contains(&cid))
        }
        "polu" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
                && (has_named_in_pile(state, ctx.catalog, "txhj_piliche")
                    || (flag(state, owner, "l11_polu_created") == 0
                        && has_named_unclaimed_detached(state, ctx.catalog, "txhj_piliche")))
        }
        "polu_damage" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && !has_equip_named(state, ctx.catalog, owner, "txhj_piliche")
                && matches!(
                    event,
                    EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
                )
        }
        "_txhj_shenqi" => matches!(
            event,
            EventData::Equipped { player, card, .. }
                if *player == owner && shenqi_buff_for_card_name(card_name(ctx.catalog, *card)).is_some()
        ),
        "_txhj_XXXshenqi" => match event {
            EventData::CardsLost {
                player,
                cards,
                origins,
            } if *player == owner => cards
                .iter()
                .zip(origins.iter())
                .any(|(&c, &orig)| {
                    orig == LostOrigin::Equip
                        && shenqi_buff_for_card_name(card_name(ctx.catalog, c)).is_some()
                }),
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_system_hooks(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut cards = Vec::new();
    let mut target = None;
    let mut card = None;
    match event {
        EventData::CardsDrawn { cards: cs, .. }
        | EventData::CardsMoved { cards: cs, .. }
        | EventData::CardsDiscarded { cards: cs, .. } => cards = cs.clone(),
        EventData::CardsLost {
            cards: cs, origins, ..
        } => {
            cards = cs
                .iter()
                .zip(origins.iter())
                .filter(|&(_, &o)| o == LostOrigin::Equip)
                .map(|(&c, _)| c)
                .collect();
        }
        EventData::Equipped { card: c, .. } => cards.push(*c),
        EventData::Damage(d) => {
            card = Some(d.amount.max(0) as u32);
            target = Some(d.target);
            if d.source == Some(owner) {
                cards.push(1);
            }
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11SystemHooks,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn activate_l11_system_hooks(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    None
}

pub(in crate::content::host) fn handle_l11_system_hooks(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "_txhj_skill_Timelimit" => {
            if let Some(outcome) = timelimit_outcome(ctx.core, ctx.catalog, owner) {
                set_flag(ctx.core, owner, "l11_timelimit_outcome", outcome);
            }
            Flow::Done
        }
        "_txhj_skill_addEffectStyle" => {
            set_flag(ctx.core, owner, "l11_effect_style_init", 1);
            Flow::Done
        }
        "_txhj_skill_gaincard" => {
            let n = flag(ctx.core, owner, "txhj_score_gaincard") + m.cards.len() as u32;
            set_flag(ctx.core, owner, "txhj_score_gaincard", n);
            Flow::Done
        }
        "_txhj_skill_discard" => {
            let n = flag(ctx.core, owner, "txhj_score_discard") + m.cards.len() as u32;
            set_flag(ctx.core, owner, "txhj_score_discard", n);
            Flow::Done
        }
        "_txhj_skill_usecard" => {
            let n = flag(ctx.core, owner, "txhj_score_usecard") + 1;
            set_flag(ctx.core, owner, "txhj_score_usecard", n);
            Flow::Done
        }
        "_txhj_skill_damage" => {
            let amt = m.card.unwrap_or(0);
            if !m.cards.is_empty() {
                let n = flag(ctx.core, owner, "txhj_score_damage") + amt;
                set_flag(ctx.core, owner, "txhj_score_damage", n);
            } else if m.target == Some(owner) {
                let n = flag(ctx.core, owner, "txhj_score_damaged") + amt;
                set_flag(ctx.core, owner, "txhj_score_damaged", n);
            }
            Flow::Done
        }
        "_txhj_skills_kill" => {
            let n = flag(ctx.core, owner, "txhj_score_kill") + 1;
            set_flag(ctx.core, owner, "txhj_score_kill", n);
            Flow::Done
        }
        "twsidao" => match m.stage {
            HostStage::Act => wait_sys_ask(m, ctx, HostStage::TargetChoice, ASK_TWSIDAO_CHOOSE),
            HostStage::TargetChoice => {
                m.card = None;
                let idx = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(t)) if usize::from(t) < TWSIDAO_CHOICES.len() => {
                        usize::from(t)
                    }
                    _ => return Flow::Done,
                };
                set_flag(ctx.core, owner, "l11_twsidao_done", 1);
                let name = TWSIDAO_CHOICES[idx];
                if let Some(cid) = take_named_from_pile_or_detached(ctx.core, ctx.catalog, name, true) {
                    set_flag(ctx.core, owner, "l11_twsidao_card", cid + 1);
                    ctx.core.player_mut(owner).hand.push(cid);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::Equipped {
                        player: owner,
                        card: cid,
                        replaced: None,
                    });
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twsidao_equip" => match m.stage {
            HostStage::Act => {
                if let Some(cid) = flag(ctx.core, owner, "l11_twsidao_card").checked_sub(1) {
                    if pop_card_id_from_pile(ctx.core, cid) {
                        ctx.core.player_mut(owner).hand.push(cid);
                        m.stage = HostStage::DrawSettle;
                        ctx.emit(EventData::Equipped {
                            player: owner,
                            card: cid,
                            replaced: None,
                        });
                        return Flow::Yield;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "polu" => match m.stage {
            HostStage::Act => {
                if let Some(cid) = take_named_from_pile_or_detached(
                    ctx.core,
                    ctx.catalog,
                    "txhj_piliche",
                    flag(ctx.core, owner, "l11_polu_created") == 0,
                ) {
                    set_flag(ctx.core, owner, "l11_polu_created", 1);
                    ctx.core.player_mut(owner).hand.push(cid);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::Equipped {
                        player: owner,
                        card: cid,
                        replaced: None,
                    });
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "polu_damage" => handle_polu_damage(m, ctx),
        "_txhj_shenqi" => {
            for &cid in &m.cards {
                if ctx.core.player(owner).equips.contains(&cid) {
                    if let Some(buff) = shenqi_buff_for_card_name(card_name(ctx.catalog, cid)) {
                        if !has_named_skill(ctx.core, ctx.catalog, owner, buff) {
                            ensure_perm_skill(ctx, owner, buff);
                        }
                    }
                }
            }
            Flow::Done
        }
        "_txhj_XXXshenqi" => {
            for &cid in &m.cards {
                if let Some(buff) = shenqi_buff_for_card_name(card_name(ctx.catalog, cid)) {
                    if !still_equips_artifact_for_buff(ctx.core, ctx.catalog, owner, buff) {
                        remove_named_skill(ctx, owner, buff);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_polu_damage(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let rem = m.card.take().unwrap_or_else(|| flag(ctx.core, owner, "l11_polu_rem"));
            if rem == 0
                || !ctx.core.player(owner).alive
                || has_equip_named(ctx.core, ctx.catalog, owner, "txhj_piliche")
            {
                set_flag(ctx.core, owner, "l11_polu_rem", 0);
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11_polu_rem", rem - 1);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                set_flag(ctx.core, owner, "l11_polu_rem", 0);
                return Flow::Done;
            }
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .rposition(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon))
            {
                let weapon = ctx.core.deck.remove(pos);
                m.stage = HostStage::Act;
                ctx.core.player_mut(owner).hand.push(weapon);
                ctx.emit(EventData::Equipped {
                    player: owner,
                    card: weapon,
                    replaced: None,
                });
                return Flow::Yield;
            }
            m.stage = HostStage::Act;
            handle_polu_damage(m, ctx)
        }
        _ => {
            set_flag(ctx.core, owner, "l11_polu_rem", 0);
            Flow::Done
        }
    }
}
