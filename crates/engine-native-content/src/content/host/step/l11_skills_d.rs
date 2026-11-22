//! L11（太虚幻境）武将技能 Group D（`L11GeneralSkillsD` tag 730）：
//!
//! - `oldfuman`、`oldfuman2`、`txhuao`、`hgkurou`、`hgkurou_effect`、`gdsanchen`、`txlveming`、`txdaoxi`
//! - `txlangxi`、`txkuangyi`、`txtanbei`、`txtanbei_effect3`、`txtanbei_effect1`、`txtanbei_effect2`
//! - `ljxuechi`、`txzhuxin`、`txyicheng`、`txnianrui`

use super::l11_common::*;
use super::l11_skills_d_asks::*;
use super::*;
use crate::catalog::{QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

fn wait_d_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_d_ask(
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

pub(in crate::content::host) fn trigger_meta_l11_skills_d(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "oldfuman2" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            vec![EventKind::TurnEnded],
        )),
        "txhuao" => Some(meta(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "txlangxi" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txzhuxin" => Some(meta(EventKind::Dying, Timing::First, 5, Vec::new())),
        "txyicheng" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_d(name: &str) -> SkillLimits {
    match name {
        "hgkurou" | "gdsanchen" | "txlveming" | "txtanbei" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_l11_skills_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    let state = ctx.state;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    let eligible = match skill.name.as_str() {
        "oldfuman" => {
            state
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_sha_like())
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !has_named_skill(state, ctx.catalog, p, "oldfuman2"))
        }
        "hgkurou" => !he_cards(state, owner).is_empty(),
        "gdsanchen" => alive_all(state, owner)
            .into_iter()
            .any(|p| flag(state, owner, &format!("skill_turn:l11_sanchen_done:{p}")) == 0),
        "txlveming" => {
            let my_eq = state.player(owner).equips.len();
            alive_others(state, owner)
                .into_iter()
                .any(|p| state.player(p).equips.len() > my_eq)
        }
        "txtanbei" => !alive_others(state, owner).is_empty(),
        _ => false,
    };
    if eligible {
        active_skill_machine(&skill.name, HostBehavior::L11GeneralSkillsD, owner)
    } else {
        None
    }
}

pub(in crate::content::host) fn filter_l11_skills_d(
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
        "oldfuman2" => match event {
            EventData::TurnEnded { player } => *player == owner,
            EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.card.is_some_and(|c| {
                        let enc = flag(state, owner, &format!("l11_oldfuman_card:{c}"));
                        enc > 0 && state.player((enc - 1) as PlayerId).alive
                    })
            }
            _ => false,
        },
        "txhuao" => matches!(event, EventData::TurnBegin { player } if *player == owner),
        "txlangxi" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true
                } if *player == owner
            ) && is_unblocked_l11_owner(state, ctx.catalog, owner)
                && !alive_others(state, owner).is_empty()
        }
        "txzhuxin" => matches!(
            event,
            EventData::Dying { source, .. } if *source == Some(owner)
        ),
        "txyicheng" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::CardUsed(u)
                        if !u.cancelled && is_sha_use(ctx.catalog, u) && !u.targets.is_empty()
                )
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_d(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match event {
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = u.card;
            if let Some(c) = u.card {
                cards.push(c);
            }
            targets = u.targets.clone();
        }
        EventData::Dying { player, .. } => target = Some(*player),
        EventData::TurnEnded { player } => {
            target = Some(*player);
            card = Some(1);
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

pub(in crate::content::host) fn handle_l11_skills_d(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "oldfuman" => step_oldfuman(m, ctx),
        "oldfuman2" => {
            if m.card == Some(1) && m.stage == HostStage::Act && current_use_card_mut(ctx).is_none()
            {
                let clear_keys: Vec<String> = ctx
                    .core
                    .vm_aux
                    .flags
                    .keys()
                    .filter(|(p, k)| *p == owner && k.starts_with("l11_oldfuman_card:"))
                    .map(|(_, k)| k.clone())
                    .collect();
                for k in clear_keys {
                    set_flag(ctx.core, owner, &k, 0);
                }
                remove_named_skill(ctx, owner, "oldfuman2");
                return Flow::Done;
            }
            if let Some(cid) = m.cards.first().copied() {
                let key = format!("l11_oldfuman_card:{cid}");
                let enc = flag(ctx.core, owner, &key);
                set_flag(ctx.core, owner, &key, 0);
                if enc > 0 {
                    let giver = (enc - 1) as PlayerId;
                    if ctx.core.player(giver).alive {
                        ctx.start_draw(giver, 1);
                    }
                }
            }
            Flow::Done
        }
        "txhuao" => {
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .rposition(|&c| ctx.catalog.card_kind(c).is_sha_like())
            {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(cid);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![cid],
                });
                ctx.emit(EventData::CardsDrawn {
                    player: owner,
                    cards: vec![cid],
                });
            }
            Flow::Done
        }
        "hgkurou" => step_hgkurou(m, ctx),
        "gdsanchen" => step_gdsanchen(m, ctx),
        "txlveming" => step_txlveming(m, ctx),
        "txlangxi" => step_txlangxi(m, ctx),
        "txkuangyi" => step_txkuangyi(m, ctx),
        "txtanbei" => step_txtanbei(m, ctx),
        "txzhuxin" => {
            if let Some(dp) = m.target {
                if ctx.core.player(dp).alive {
                    ctx.core.player_mut(dp).dying = false;
                    ctx.emit(EventData::Death {
                        player: dp,
                        killer: Some(owner),
                    });
                }
            }
            Flow::Done
        }
        "txyicheng" => step_txyicheng(m, ctx),
        _ => Flow::Done,
    }
}

fn step_oldfuman(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::SelectCard, ASK_OLDFUMAN_CARD),
        HostStage::SelectCard => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_OLDFUMAN_TARGET)
        }
        HostStage::TargetChoice => {
            let (Some(tgt), Some(&cid)) = (
                answered_players(ctx).into_iter().next(),
                m.cards.first(),
            ) else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&cid) {
                return Flow::Done;
            }
            set_flag(ctx.core, tgt, &format!("l11_oldfuman_card:{cid}"), u32::from(owner) + 1);
            ensure_perm_skill(ctx, tgt, "oldfuman2");
            ctx.give_hand_cards(owner, tgt, &[cid]);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hgkurou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::SelectCard, ASK_HGKUROU_CARD),
        HostStage::SelectCard => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "hgkurou");
                return Flow::Done;
            };
            discard_player_cards(ctx, owner, &[cid]);
            m.stage = HostStage::DrawSettle;
            emit_lose_hp(ctx, owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 3);
                if let Some(def) = ctx.catalog.skill_by_name("hgkurou_effect") {
                    ctx.add_skill_until_turn_end(owner, def);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gdsanchen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_GDSANCHEN_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "gdsanchen");
                return Flow::Done;
            };
            m.target = Some(tgt);
            set_flag(ctx.core, owner, &format!("skill_turn:l11_sanchen_done:{tgt}"), 1);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(tgt, 3);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(tgt).alive || he_cards(ctx.core, tgt).is_empty() {
                return Flow::Done;
            }
            wait_d_ask(m, ctx, HostStage::CardsAsk, ASK_GDSANCHEN_DISCARD)
        }
        HostStage::CardsAsk => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            let picked = answered_cards(ctx);
            if picked.is_empty() {
                return Flow::Done;
            }
            let mut cat_mask = 0u8;
            for &c in &picked {
                let k = ctx.catalog.card_kind(c);
                let bit = if k.is_basic_kind() {
                    1
                } else if k.is_equip() {
                    4
                } else {
                    2
                };
                cat_mask |= bit;
            }
            let all_distinct = (cat_mask.count_ones() as usize) == picked.len();
            discard_player_cards(ctx, tgt, &picked);
            if all_distinct {
                if ctx.core.player(tgt).alive {
                    ctx.start_draw(tgt, 1);
                }
                refund_once_used(ctx, owner, "gdsanchen");
                let marks = flag(ctx.core, owner, "l11_gdsanchen_marks") + 1;
                set_flag(ctx.core, owner, "l11_gdsanchen_marks", marks);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txlveming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_TXLVEMING_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "txlveming");
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_d_ask(m, ctx, HostStage::SelectNature, ASK_TXLVEMING_RANK)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let chosen_rank = ((idx.min(12)) + 1) as u8;
            m.cards = vec![u32::from(chosen_rank)];
            let cnt = flag(ctx.core, owner, "l11_txlveming_count") + 1;
            set_flag(ctx.core, owner, "l11_txlveming_count", cnt);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            let chosen_rank = m.cards.first().copied().unwrap_or(1) as u8;
            let judge_rank = flag(ctx.core, owner, "last_judge_rank");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            let hit = judge_rank == u32::from(chosen_rank);
            if hit {
                emit_damage(ctx, Some(owner), tgt, 3, DamageNature::Normal, None);
            } else {
                let pool = hej_cards(ctx.core, tgt);
                if !pool.is_empty() {
                    let pick = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                    move_cards_to_hand(ctx, tgt, owner, &[pick]);
                }
                emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txlangxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_TXLANGXI_TARGET),
        HostStage::TargetChoice => {
            if let Some(tgt) = answered_players(ctx).into_iter().next() {
                let dmg = 2 + ctx.core.rng.gen_range(3);
                emit_damage(ctx, Some(owner), tgt, dmg, DamageNature::Normal, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txkuangyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_TXKUANGYI_CONFIRM),
        HostStage::TargetChoice => {
            if is_confirmed(ctx) {
                set_flag(ctx.core, owner, "skill_turn:l11_kuangyi_used", 1);
                let next_chain = !ctx.core.player(owner).linked;
                ctx.emit(EventData::Linked {
                    player: owner,
                    linked: next_chain,
                });
                if let Some(&cid) = m.cards.first() {
                    if ctx.core.discard.contains(&cid) || ctx.core.table.contains(&cid) {
                        ctx.core.discard.retain(|&c| c != cid);
                        ctx.core.table.retain(|&c| c != cid);
                        ctx.core.player_mut(owner).hand.push(cid);
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: vec![cid],
                        });
                        ctx.emit(EventData::CardsDrawn {
                            player: owner,
                            cards: vec![cid],
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txtanbei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_TXTANBEI_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "txtanbei");
                return Flow::Done;
            };
            m.target = Some(tgt);
            if let Some(def3) = ctx.catalog.skill_by_name("txtanbei_effect3") {
                ctx.add_skill_until_turn_end(owner, def3);
            }
            if hej_cards(ctx.core, tgt).is_empty() {
                if let Some(def1) = ctx.catalog.skill_by_name("txtanbei_effect1") {
                    ctx.add_skill_until_turn_end(tgt, def1);
                }
                return Flow::Done;
            }
            wait_d_ask(m, ctx, HostStage::SelectNature, ASK_TXTANBEI_MODE)
        }
        HostStage::SelectNature => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 1,
            };
            if mode == 0 {
                let pool = hej_cards(ctx.core, tgt);
                if !pool.is_empty() {
                    let cid = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                    move_cards_to_hand(ctx, tgt, owner, &[cid]);
                }
                if let Some(def2) = ctx.catalog.skill_by_name("txtanbei_effect2") {
                    ctx.add_skill_until_turn_end(tgt, def2);
                }
            } else if let Some(def1) = ctx.catalog.skill_by_name("txtanbei_effect1") {
                ctx.add_skill_until_turn_end(tgt, def1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txyicheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.cards = vec![0];
            advance_txyicheng_target(m, ctx)
        }
        HostStage::TargetChoice => {
            if !is_confirmed(ctx) {
                return advance_txyicheng_next(m, ctx);
            }
            let Some(tgt) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return advance_txyicheng_next(m, ctx);
            };
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(tgt, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(tgt).alive || he_cards(ctx.core, tgt).is_empty() {
                return advance_txyicheng_next(m, ctx);
            }
            wait_d_ask(m, ctx, HostStage::CardsAsk, ASK_TXYICHENG_DISCARD)
        }
        HostStage::CardsAsk => {
            if let (Some(tgt), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
                discard_player_cards(ctx, tgt, &[cid]);
            }
            advance_txyicheng_next(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_txyicheng_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let idx = m.cards.first().copied().unwrap_or(0) + 1;
    m.cards = vec![idx];
    advance_txyicheng_target(m, ctx)
}

fn advance_txyicheng_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let mut idx = m.cards.first().copied().unwrap_or(0) as usize;
    while idx < m.targets.len() {
        let tgt = m.targets[idx];
        if ctx.core.player(tgt).alive {
            m.cards = vec![idx as u32];
            m.target = Some(tgt);
            return wait_d_ask(m, ctx, HostStage::TargetChoice, ASK_TXYICHENG_CONFIRM);
        }
        idx += 1;
    }
    Flow::Done
}
