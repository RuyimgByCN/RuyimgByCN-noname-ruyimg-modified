//! Batch U05-1 主包 (`stone_chenshui`) 与阵法 (`card/zhenfa/card.js`) 全量实现。

use super::super::*;
use super::l27_batch_a::{alive_all, flag, set_flag};
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(in crate::content::host) fn is_zhenfa_card_name(name: &str) -> bool {
    matches!(
        name,
        "pozhenjue"
            | "zf_pozhenjue"
            | "changshezhen"
            | "zf_changshezhen"
            | "tianfuzhen"
            | "zf_tianfuzhen"
            | "dizaizhen"
            | "zf_dizaizhen"
            | "fengyangzhen"
            | "zf_fengyangzhen"
            | "yunchuizhen"
            | "zf_yunchuizhen"
            | "shepanzhen"
            | "zf_shepanzhen"
            | "longfeizhen"
            | "zf_longfeizhen"
            | "huyizhen"
            | "zf_huyizhen"
            | "qixingzhen"
            | "zf_qixingzhen"
            | "niaoxiangzhen"
            | "zf_niaoxiangzhen"
    )
}

fn finish_table_host_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) {
    if let Some(c) = m.card {
        ctx.core.table.retain(|x| *x != c);
        ctx.to_discard(c);
        m.card = None;
    }
}

pub(in crate::content::host) fn trigger_meta_u05_zhenfa(name: &str) -> Option<TriggerMeta> {
    match name {
        "stone_chenshui" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u05_zhenfa(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_u05_zhenfa(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) || !ctx.state.player(owner).alive {
        return false;
    }
    match m.name.as_str() {
        "stone_chenshui" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_zhenfa(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    if m.behavior != HostBehavior::U05ZhenfaRecastSkill || m.name != "u05_zhenfa_recast" {
        return None;
    }
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) || !ctx.state.player(owner).alive {
        return None;
    }
    if ctx.state.deck.is_empty() && ctx.state.discard.is_empty() {
        return None;
    }
    let has_zf = ctx.state.player(owner).hand.iter().any(|&c| {
        ctx.catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| is_zhenfa_card_name(cd.name))
    });
    if !has_zf {
        return None;
    }
    Some(HostMachine {
        def_key: m.name.clone(),
        behavior: m.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(in crate::content::host) fn ask_u05_zhenfa(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let key = card
        .and_then(|c| catalog.cards.get(c as usize))
        .map(|d| d.name)
        .unwrap_or("u05_zhenfa_recast");
    match key {
        "longfeizhen" | "zf_longfeizhen" if stage == HostStage::SelectCard => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .chain(core.player(t).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "u05_zhenfa_longfei_discard",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        "huyizhen" | "zf_huyizhen" if stage == HostStage::SelectCard => {
            let actor = *targets.get(1)?;
            let opts: Vec<ChoiceOption> = core
                .player(actor)
                .hand
                .iter()
                .chain(core.player(actor).equips.iter())
                .copied()
                .filter(|&c| !catalog.card_kind(c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    actor,
                    "u05_zhenfa_huyizhen_card",
                    ChoiceKind::SelectCards,
                    opts,
                    0,
                    1,
                    true,
                )
            })
        }
        "u05_zhenfa_recast" if stage == HostStage::SelectCard => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| is_zhenfa_card_name(cd.name))
                })
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "u05_zhenfa_recast_card",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        _ => None,
    }
}

fn wait_ask_zf(m: &mut HostMachine, ctx: &mut Ctx<'_>, next_stage: HostStage) -> Flow {
    let Some(exp) = ask_u05_zhenfa(
        next_stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        return Flow::Done;
    };
    m.stage = next_stage;
    Flow::WaitChoice {
        observer: exp.observer,
        spec: ChoiceSpec {
            kind: exp.kind,
            prompt: exp.prompt,
            options: exp.options,
            min: exp.min,
            max: exp.max,
            allow_cancel: exp.allow_cancel,
        },
    }
}

fn step_stone_chenshui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let leader_raw = flag(ctx.core, owner, "u05_stone_leader");
    let target = if leader_raw > 0 {
        let p = (leader_raw - 1) as PlayerId;
        if (p as usize) < ctx.core.players.len() && ctx.core.player(p).alive {
            p
        } else {
            owner
        }
    } else {
        owner
    };
    let is_dream = |c: CardId| {
        ctx.catalog.cards.get(c as usize).is_some_and(|cd| {
            matches!(
                cd.name,
                "hsmengjing_feicuiyoulong"
                    | "hsmengjing_huanxiaojiemei"
                    | "hsmengjing_suxing"
                    | "hsmengjing_mengye"
                    | "hsmengjing_mengjing"
            ) || cd.name.starts_with("hsmengjing_")
        })
    };
    let from_detached: Vec<CardId> = ctx
        .core
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| is_dream(c).then_some(c))
        .collect();
    if !from_detached.is_empty() {
        let idx = ctx.core.rng.gen_range(from_detached.len() as u32) as usize;
        let cid = from_detached[idx];
        ctx.core.detached_cards.retain(|&(c, _)| c != cid);
        ctx.core.player_mut(target).hand.push(cid);
        ctx.log(LogEvent::CardsDrawn {
            player: target,
            cards: vec![cid],
        });
        return Flow::Done;
    }
    let mut from_deck: Vec<CardId> = ctx.core.deck.iter().copied().filter(|&c| is_dream(c)).collect();
    let (chosen, from_discard) = if !from_deck.is_empty() {
        let idx = ctx.core.rng.gen_range(from_deck.len() as u32) as usize;
        (Some(from_deck.remove(idx)), false)
    } else {
        let from_disc: Vec<CardId> = ctx
            .core
            .discard
            .iter()
            .copied()
            .filter(|&c| is_dream(c))
            .collect();
        if !from_disc.is_empty() {
            let idx = ctx.core.rng.gen_range(from_disc.len() as u32) as usize;
            (Some(from_disc[idx]), true)
        } else {
            (None, false)
        }
    };
    if let Some(cid) = chosen {
        if from_discard {
            ctx.core.discard.retain(|&c| c != cid);
        } else {
            ctx.core.deck.retain(|&c| c != cid);
        }
        ctx.core.player_mut(target).hand.push(cid);
        ctx.log(LogEvent::CardsDrawn {
            player: target,
            cards: vec![cid],
        });
    }
    Flow::Done
}

fn step_shepanzhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if ctx.core.effective_faction(ctx.catalog, owner).is_some() {
        let targets: Vec<PlayerId> = alive_all(ctx.core, owner)
            .into_iter()
            .filter(|&p| ctx.core.is_friend_of(ctx.catalog, owner, p))
            .collect();
        for pair in targets.windows(2) {
            let prev = pair[0];
            let curr = pair[1];
            let nxt = ctx.core.next_alive(prev);
            if curr != nxt {
                ctx.core.swap_seat(curr, nxt);
            }
        }
    }
    finish_table_host_card(m, ctx);
    Flow::Done
}

pub(in crate::content::host) fn handle_u05_main_stone(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match m.def_key.as_str() {
        "stone_chenshui" => step_stone_chenshui(m, ctx),
        "u05_zhenfa_recast" => match m.stage {
            HostStage::Act => wait_ask_zf(m, ctx, HostStage::SelectCard),
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(m.owner).hand.contains(&c) {
                        ctx.discard_from_hand(m.owner, c);
                        ctx.emit(EventData::CardsDiscarded {
                            player: m.owner,
                            cards: vec![c],
                        });
                        ctx.start_draw(m.owner, 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengyangzhen" | "zf_fengyangzhen" => {
            for &t in &m.targets {
                if ctx.core.player(t).alive {
                    set_flag(ctx.core, t, "u05_fengyang_feiying", 1);
                }
            }
            finish_table_host_card(m, ctx);
            Flow::Done
        }
        "yunchuizhen" | "zf_yunchuizhen" => {
            for &t in &m.targets {
                if ctx.core.player(t).alive {
                    set_flag(ctx.core, t, "u05_yunchuizhen_wushuang", 1);
                }
            }
            finish_table_host_card(m, ctx);
            Flow::Done
        }
        "shepanzhen" | "zf_shepanzhen" => step_shepanzhen(m, ctx),
        "longfeizhen" | "zf_longfeizhen" => match m.stage {
            HostStage::Act => {
                while !m.targets.is_empty() {
                    let t = m.targets.remove(0);
                    let has_he = !ctx.core.player(t).hand.is_empty()
                        || !ctx.core.player(t).equips.is_empty();
                    if !ctx.core.player(t).alive || !has_he {
                        if m.targets.is_empty() {
                            ctx.start_draw(m.owner, 1);
                        }
                        continue;
                    }
                    m.target = Some(t);
                    m.cards = if m.targets.is_empty() { vec![1] } else { Vec::new() };
                    return wait_ask_zf(m, ctx, HostStage::SelectCard);
                }
                finish_table_host_card(m, ctx);
                Flow::Done
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let t = m.target.unwrap_or(m.owner);
                    super::l27_batch_a::discard_cards(ctx, t, &[c]);
                }
                if !m.cards.is_empty() {
                    m.cards.clear();
                    ctx.start_draw(m.owner, 1);
                    m.stage = HostStage::Act;
                    return Flow::Yield;
                }
                m.stage = HostStage::Act;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        "huyizhen" | "zf_huyizhen" => match m.stage {
            HostStage::Act => {
                while m.targets.len() >= 2 {
                    let victim = m.targets[0];
                    let actor = m.targets[1];
                    let has_nonbasic = ctx.core.player(victim).alive
                        && ctx.core.player(actor).alive
                        && ctx
                            .core
                            .player(actor)
                            .hand
                            .iter()
                            .chain(ctx.core.player(actor).equips.iter())
                            .any(|&c| !ctx.catalog.card_kind(c).is_basic_kind());
                    if !has_nonbasic {
                        m.targets.drain(0..2);
                        continue;
                    }
                    return wait_ask_zf(m, ctx, HostStage::SelectCard);
                }
                finish_table_host_card(m, ctx);
                Flow::Done
            }
            HostStage::SelectCard => {
                if m.targets.len() < 2 {
                    finish_table_host_card(m, ctx);
                    return Flow::Done;
                }
                let victim = m.targets.remove(0);
                let actor = m.targets.remove(0);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(victim).alive && ctx.core.player(actor).alive {
                        let had = ctx.core.player(actor).hand.contains(&c)
                            || ctx.core.player(actor).equips.contains(&c);
                        if had {
                            ctx.take_owned_card(actor, c);
                            ctx.core.table.push(c);
                            ctx.core.declared_kinds.insert(c, CardKind::Sha);
                            m.stage = HostStage::Act;
                            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                c,
                                actor,
                                vec![victim],
                                Some(CardKind::Sha),
                            )));
                            return Flow::Yield;
                        }
                    }
                }
                m.stage = HostStage::Act;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
