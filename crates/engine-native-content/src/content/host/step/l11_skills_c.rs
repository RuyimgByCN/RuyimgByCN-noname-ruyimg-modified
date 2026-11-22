//! L11（太虚幻境）武将技能 Batch C（S58–S71：`l11_skills_c.rs`）：
//!
//! - `cxy_BaoYing`、`cxyYangWu`、`cxyJingQi`、`cxyRuiQi`、`cxyJiaoXia`、`cxyTunJun`、`cxyFengYing`
//! - `cxyFanGong`、`cxyJieLve`、`cxyMoQu`、`cxyYaoWu`、`cxyYingHun`、`cxyPoLu`

use super::l11_common::*;
use super::l11_skills_c_asks::*;
use super::*;
use crate::catalog::QueryCtx;
use crate::catalog::{SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

fn wait_c_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_c_ask(
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

pub(in crate::content::host) fn trigger_meta_l11_skills_c(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "cxy_BaoYing" => Some(meta(EventKind::Dying, Timing::Normal, 1, Vec::new())),
        "cxyYangWu" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "cxyTunJun" => Some(meta(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "cxyJieLve" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "cxyMoQu" => Some(meta(
            EventKind::TurnPhaseEnd,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "cxyYaoWu" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1, Vec::new())),
        "cxyYingHun" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "cxyPoLu" => Some(meta(EventKind::Death, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_c(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_skills_c(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_skills_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if skill.name != "cxyPoLu" && !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "cxy_BaoYing" => {
            let EventData::Dying { player, .. } = event else {
                return false;
            };
            flag(state, owner, "l11_baoying_used") == 0
                && state.is_friend_of(catalog, owner, *player)
        }
        "cxyYangWu" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
        }
        "cxyTunJun" => {
            let EventData::TurnBegin { .. } = event else {
                return false;
            };
            state.player(owner).max_hp > 1
                && !state.player(owner).infinite_hp()
                && flag(state, owner, "l11_tunjun_round") != state.turn.round
        }
        "cxyJieLve" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && state.player(d.target).alive
                && !hej_cards(state, d.target).is_empty()
        }
        "cxyMoQu" => match event {
            EventData::TurnEndWindow { after: false, .. } => {
                state.player(owner).hand.len() <= state.player(owner).hp.max(0) as usize
            }
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.target != owner
                    && state.is_friend_of(catalog, owner, d.target)
                    && !he_cards(state, owner).is_empty()
            }
            _ => false,
        },
        "cxyYaoWu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.target != owner || d.prevented || d.amount <= 0 {
                return false;
            }
            let Some(src) = d.source.filter(|&s| state.player(s).alive) else {
                return false;
            };
            let Some(cid) = d.card else {
                return false;
            };
            catalog.card_kind(cid).is_sha_like() && effective_color_is_red(state, catalog, src, cid)
        }
        "cxyYingHun" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && !alive_others(state, owner).is_empty()
        }
        "cxyPoLu" => {
            let EventData::Death { player, killer } = event else {
                return false;
            };
            let self_die = *player == owner;
            let enemy_slain = *player != owner
                && state.player(owner).alive
                && !state.is_friend_of(catalog, owner, *player)
                && killer.is_some_and(|k| {
                    state.player(k).alive && state.is_friend_of(catalog, owner, k)
                });
            self_die || enemy_slain
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut cards = Vec::new();
    match event {
        EventData::Dying { player, .. } => target = Some(*player),
        EventData::Damage(d) => {
            target = if skill.name == "cxyYaoWu" {
                d.source
            } else {
                Some(d.target)
            };
        }
        EventData::TurnEndWindow { .. } => {
            cards.push(1);
        }
        EventData::Death { player, .. } => {
            target = Some(*player);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11GeneralSkillsC,
        owner,
        target,
        card: None,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_skills_c(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "cxy_BaoYing" => match m.stage {
            HostStage::Act => wait_c_ask(m, ctx, HostStage::TargetChoice, ASK_BAOYING_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    set_flag(ctx.core, owner, "l11_baoying_used", 1);
                    if let Some(t) = m.target {
                        let cur = ctx.core.player(t).hp;
                        if cur < 1 {
                            ctx.recover(t, (1 - cur) as u32, None);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyYangWu" => match m.stage {
            HostStage::Act => {
                m.targets = alive_others(ctx.core, owner);
                m.stage = HostStage::TargetChoice;
                handle_l11_skills_c(m, ctx)
            }
            HostStage::TargetChoice => {
                while let Some(t) = m.targets.first().copied() {
                    m.targets.remove(0);
                    if ctx.core.player(t).alive {
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                        return Flow::Yield;
                    }
                }
                m.stage = HostStage::DrawSettle;
                if ctx.core.player(owner).alive {
                    emit_lose_hp(ctx, owner, 1);
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyTunJun" => {
            set_flag(ctx.core, owner, "l11_tunjun_round", ctx.core.turn.round);
            if ctx.core.player(owner).max_hp > 1 {
                ctx.lose_max_hp(owner, 1);
                let n = ctx.core.player(owner).max_hp;
                if n > 0 {
                    ctx.start_draw(owner, n);
                }
            }
            Flow::Done
        }
        "cxyFanGong" => match m.stage {
            HostStage::Act => wait_c_ask(m, ctx, HostStage::SelectCard, ASK_FANGONG_SHA),
            HostStage::SelectCard => {
                if let (Some(ChoiceOption::Card(sha_c)), Some(t)) = (
                    ctx.answer().first().copied(),
                    m.target.filter(|&p| ctx.core.player(p).alive),
                ) {
                    if ctx.core.player(owner).hand.contains(&sha_c) {
                        ctx.core.player_mut(owner).hand.retain(|&c| c != sha_c);
                        ctx.core.table.push(sha_c);
                        ctx.note_cards_lost(owner, &[sha_c]);
                        let eff = effective_card_kind(ctx.core, ctx.catalog, sha_c);
                        ctx.emit(EventData::CardUsed(super::sha_card_use_data(
                            ctx,
                            owner,
                            sha_c,
                            vec![t],
                            Some(eff),
                        )));
                        m.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyJieLve" => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            match m.stage {
                HostStage::Act => wait_c_ask(m, ctx, HostStage::TargetChoice, ASK_JIELVE_CONFIRM),
                HostStage::TargetChoice => {
                    if !is_confirmed(ctx) {
                        return Flow::Done;
                    }
                    let h = &ctx.core.player(t).hand;
                    if h.len() > 1 {
                        return wait_c_ask(m, ctx, HostStage::SelectCard, ASK_JIELVE_H);
                    }
                    if let Some(&c) = h.first() {
                        move_cards_to_hand(ctx, t, owner, &[c]);
                    }
                    m.stage = HostStage::SelectCard;
                    m.card = None;
                    jielve_next_equip_or_judge(m, ctx, t, false)
                }
                HostStage::SelectCard => {
                    if let Some(c) = answered_cards(ctx).into_iter().next() {
                        move_cards_to_hand(ctx, t, owner, &[c]);
                    }
                    jielve_next_equip_or_judge(m, ctx, t, false)
                }
                HostStage::SelectNature => {
                    if let Some(c) = answered_cards(ctx).into_iter().next() {
                        move_cards_to_hand(ctx, t, owner, &[c]);
                    }
                    jielve_next_equip_or_judge(m, ctx, t, true)
                }
                HostStage::CompareTargetPick => {
                    if let Some(c) = answered_cards(ctx).into_iter().next() {
                        move_cards_to_hand(ctx, t, owner, &[c]);
                    }
                    toggle_link_owner(ctx, owner);
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "cxyMoQu" => match m.stage {
            HostStage::Act => {
                if m.cards.first().copied() == Some(1) {
                    if ctx.core.player(owner).hand.len() <= ctx.core.player(owner).hp.max(0) as usize
                    {
                        ctx.start_draw(owner, 2);
                    }
                    return Flow::Done;
                }
                let pool = he_cards(ctx.core, owner);
                if pool.len() == 1 {
                    discard_player_cards(ctx, owner, &pool);
                    return Flow::Done;
                }
                wait_c_ask(m, ctx, HostStage::SelectCard, ASK_MOQU_DISCARD)
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if !chosen.is_empty() {
                    discard_player_cards(ctx, owner, &chosen);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyYaoWu" => {
            let Some(src) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            match m.stage {
                HostStage::Act => {
                    let pl = ctx.core.player(src);
                    if pl.hp >= pl.max_hp as i32 {
                        ctx.start_draw(src, 1);
                        return Flow::Done;
                    }
                    wait_c_ask(m, ctx, HostStage::SelectNature, ASK_YAOWU_MODE)
                }
                HostStage::SelectNature => {
                    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Tag(0))) {
                        ctx.recover(src, 1, None);
                    } else {
                        ctx.start_draw(src, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "cxyYingHun" => match m.stage {
            HostStage::Act => wait_c_ask(m, ctx, HostStage::TargetChoice, ASK_YINGHUN_TARGET),
            HostStage::TargetChoice => {
                let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                else {
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_c_ask(m, ctx, HostStage::SelectNature, ASK_YINGHUN_MODE)
            }
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                let x = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1)
                    as u32;
                let (draw_n, disc_n) = if mode == 0 { (x, 1) } else { (1, x) };
                m.cards = vec![disc_n];
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(t, draw_n);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                let disc_n = m.cards.first().copied().unwrap_or(1) as usize;
                let pool = he_cards(ctx.core, t);
                if pool.len() <= disc_n {
                    discard_player_cards(ctx, t, &pool);
                    return Flow::Done;
                }
                wait_c_ask(m, ctx, HostStage::SelectCard, ASK_YINGHUN_DISCARD)
            }
            HostStage::SelectCard => {
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    let chosen = answered_cards(ctx);
                    if !chosen.is_empty() {
                        discard_player_cards(ctx, t, &chosen);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyPoLu" => {
            let cnt = flag(ctx.core, owner, "l11_polu_count") + 1;
            set_flag(ctx.core, owner, "l11_polu_count", cnt);
            let self_die = m.target == Some(owner);
            let friends: Vec<PlayerId> = alive_all(ctx.core, owner)
                .into_iter()
                .filter(|&p| {
                    (!self_die || p != owner) && ctx.core.is_friend_of(ctx.catalog, owner, p)
                })
                .collect();
            for f in friends {
                ctx.start_draw(f, cnt);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn jielve_next_equip_or_judge(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    t: PlayerId,
    skip_equip: bool,
) -> Flow {
    let owner = m.owner;
    if !skip_equip {
        let eq = &ctx.core.player(t).equips;
        if eq.len() > 1 {
            return wait_c_ask(m, ctx, HostStage::SelectNature, ASK_JIELVE_E);
        }
        if let Some(&c) = eq.first() {
            move_cards_to_hand(ctx, t, owner, &[c]);
        }
    }
    let j = &ctx.core.player(t).judge;
    if j.len() > 1 {
        return wait_c_ask(m, ctx, HostStage::CompareTargetPick, ASK_JIELVE_J);
    }
    if let Some(&c) = j.first() {
        move_cards_to_hand(ctx, t, owner, &[c]);
    }
    toggle_link_owner(ctx, owner);
    Flow::Done
}

fn toggle_link_owner(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.player(owner).alive {
        let next = !ctx.core.player(owner).linked;
        ctx.emit(EventData::Linked {
            player: owner,
            linked: next,
        });
    }
}
