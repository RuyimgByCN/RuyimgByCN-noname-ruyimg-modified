//! U05 仙剑 Batch B（13 技）：`feizhua`, `diesha`, `guijiang2`, `guijiang3`,
//! `fenxing`, `xfenxin`, `xfenxin2`, `qijian`, `qianfang2`, `poyun2`,
//! `zhuyue2`, `longxi`, `xjzhimeng3`。

use super::xianjian_batch_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{damage_card_kind, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "feizhua" => Some(meta(EventKind::CardUsed, Vec::new())),
        "diesha" => Some(meta(EventKind::DamageEnd, Vec::new())),
        "guijiang2" => Some(meta(EventKind::TurnBegin, vec![EventKind::Death])),
        "fenxing" => Some(meta(EventKind::TurnBegin, Vec::new())),
        "xfenxin2" => Some(meta(EventKind::Death, Vec::new())),
        "qijian" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "poyun2" => Some(meta(EventKind::DamageEnd, Vec::new())),
        "xjzhimeng3" => Some(meta(EventKind::TurnBegin, vec![EventKind::Death])),
        _ => None,
    }
}

pub(super) fn limits(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn filter_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "feizhua" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            if u.player != owner || u.cancelled || !kind.is_sha_like() || u.targets.len() != 1 {
                return false;
            }
            let primary = u.targets[0];
            ctx.state.player(primary).alive && !feizhua_candidates(ctx.state, ctx.catalog, owner, primary).is_empty()
        }
        "diesha" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.source != Some(owner) || d.prevented || d.amount <= 0 || d.settlement.notrigger || !ctx.state.player(d.target).alive {
                return false;
            }
            let root = match frame.data {
                EventData::DamageWindow { root, .. } => frames.iter().find(|f| f.id == root),
                EventData::Damage(_) => Some(frame),
                _ => None,
            };
            let is_sha = root
                .and_then(|r| damage_card_kind(frames, r, ctx.catalog))
                .or_else(|| d.card.map(|c| ctx.catalog.card_kind(c)))
                .is_some_and(|k| k.is_sha_like());
            is_sha
        }
        "guijiang2" => match event {
            EventData::TurnBegin { player } | EventData::Death { player, .. } => {
                flag(ctx.state, owner, "xj_guijiang2_src") == u32::from(*player) + 1
            }
            _ => false,
        },
        "fenxing" => match event {
            EventData::TurnBegin { player } if *player == owner => {
                ctx.state.player(owner).general_slots.iter().flatten().any(|&g| {
                    ctx.catalog.general_meta(g).is_some_and(|(name, ..)| name == "pal_longkui")
                })
            }
            _ => false,
        },
        "xfenxin2" => matches!(event, EventData::Death { killer: Some(k), .. } if *k == owner && ctx.state.player(owner).alive),
        "qijian" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Discard, begin: false }
            if *player == owner && ctx.state.turn.discard_phase_discarded.max(flag(ctx.state, owner, "xj_last_discard_count")) > 0
                && alive_others(ctx.state, owner).iter().any(|&p| sha_target_legal(ctx.state, ctx.catalog, owner, p, false))
        ),
        "poyun2" => matches!(
            event,
            EventData::Damage(d)
            if d.source == Some(owner) && !d.prevented && d.amount > 0 && !d.settlement.notrigger && ctx.state.player(owner).alive
        ),
        "xjzhimeng3" => match event {
            EventData::TurnBegin { player } | EventData::Death { player, .. } if *player == owner => {
                (0..ctx.state.players.len() as PlayerId)
                    .any(|s| has_skill_named(ctx.state, ctx.catalog, s, "xjzhimeng2"))
            }
            _ => false,
        },
        _ => false,
    }
}

fn feizhua_candidates(
    state: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    primary: PlayerId,
) -> Vec<PlayerId> {
    state
        .seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && p != primary
                && state.player(p).alive
                && state.seat_distance_alive(primary, p) == 1
                && sha_target_legal(state, catalog, owner, p, false)
        })
        .collect()
}

pub(super) fn machine(skill: &HostSkillBehavior, owner: PlayerId, event: &EventData) -> HostMachine {
    let mut target = None;
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("feizhua", EventData::CardUsed(u)) => target = u.targets.first().copied(),
        ("diesha", EventData::Damage(d)) => target = Some(d.target),
        ("xjzhimeng3", EventData::Death { player, .. }) => targets.push(*player),
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05XianjianBatchB,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "feizhua" => match m.stage {
            HostStage::Act => {
                let Some(primary) = m.target else {
                    return Flow::Done;
                };
                m.targets = feizhua_candidates(ctx.core, ctx.catalog, owner, primary);
                if m.targets.is_empty() {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_FEIZHUA_CONFIRM)
            }
            HostStage::SelectNature => {
                if answered_confirm(ctx) {
                    let extras = m.targets.clone();
                    for frame in ctx.res.stack.iter_mut().rev() {
                        if let EventData::CardUsed(u) = &mut frame.data {
                            for t in extras {
                                if ctx.core.player(t).alive && !u.targets.contains(&t) {
                                    u.targets.push(t);
                                }
                            }
                            break;
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "diesha" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "guijiang2" => {
            set_flag(ctx.core, owner, "u05_mod:guijiang2", 0);
            set_flag(ctx.core, owner, "xj_guijiang2_src", 0);
            remove_skill_named(ctx, owner, "guijiang2");
            Flow::Done
        }
        "fenxing" => {
            if ctx.core.rng.gen_range(2) == 0 {
                let gui = flag(ctx.core, owner, "xj_fenxing_gui") != 0;
                if gui {
                    set_flag(ctx.core, owner, "xj_fenxing_gui", 0);
                    remove_skill_named(ctx, owner, "guijiang");
                    remove_skill_named(ctx, owner, "diesha");
                    ensure_temp_skill(ctx, owner, "diewu", false);
                    ensure_temp_skill(ctx, owner, "lingyu", false);
                } else {
                    set_flag(ctx.core, owner, "xj_fenxing_gui", 1);
                    remove_skill_named(ctx, owner, "diewu");
                    remove_skill_named(ctx, owner, "lingyu");
                    ensure_temp_skill(ctx, owner, "guijiang", false);
                    ensure_temp_skill(ctx, owner, "diesha", false);
                }
            }
            Flow::Done
        }
        "xfenxin2" => {
            if ctx.core.player(owner).alive {
                ctx.core.player_mut(owner).max_hp += 1;
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        "qijian" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIJIAN_TARGETS),
            HostStage::TargetChoice => {
                let limit = ctx
                    .core
                    .turn
                    .discard_phase_discarded
                    .max(flag(ctx.core, owner, "xj_last_discard_count")) as usize;
                let picked: Vec<PlayerId> = answered_players(ctx)
                    .into_iter()
                    .filter(|&p| sha_target_legal(ctx.core, ctx.catalog, owner, p, false))
                    .take(limit)
                    .collect();
                if !picked.is_empty() {
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: owner,
                        targets: picked,
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "poyun2" => {
            ctx.start_draw(owner, 1);
            remove_skill_named(ctx, owner, "poyun2");
            Flow::Done
        }
        "xjzhimeng3" => {
            let dying_p = m.targets.first().copied();
            let seats = ctx.core.players.len() as PlayerId;
            for s in 0..seats {
                if !has_skill_named(ctx.core, ctx.catalog, s, "xjzhimeng2") {
                    continue;
                }
                let card_opt = flag(ctx.core, s, "xj_zhimeng_card")
                    .checked_sub(1)
                    .or_else(|| ctx.core.player(s).off_court.last().copied());
                set_flag(ctx.core, s, "xj_zhimeng_card", 0);
                if let Some(c) = card_opt {
                    if ctx.core.player(s).off_court.contains(&c) {
                        ctx.core.player_mut(s).off_court.retain(|&x| x != c);
                        if dying_p == Some(s) {
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded { player: s, cards: vec![c] });
                        } else if ctx.core.player(s).alive {
                            ctx.core.player_mut(s).hand.push(c);
                            ctx.log(LogEvent::CardsGiven { from: s, to: s, cards: vec![c] });
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                }
                remove_skill_named(ctx, s, "xjzhimeng2");
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
