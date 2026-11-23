use super::helpers::*;
use super::super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

const ASK_JIEHUO_TARGET: u32 = 401;
const ASK_YULING3_DISCARD: u32 = 402;

pub(in crate::content::host) fn trigger_meta_char_d(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "zhaolu" => Some(meta(
            EventKind::TurnPhaseEnd,
            Timing::Normal,
            0,
            vec![EventKind::Damage, EventKind::Death],
        )),
        "jiehuo" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new())),
        "jiehuo2" => Some(meta(EventKind::PhaseEnd, Timing::Normal, 0, Vec::new())),
        "yuling" => Some(meta(
            EventKind::CardsLost,
            Timing::Normal,
            10,
            vec![
                EventKind::CardsDiscarded,
                EventKind::CardsDrawn,
                EventKind::CardsGiven,
                EventKind::CardsMoved,
            ],
        )),
        "yuling2" => Some(meta(
            EventKind::CardsLost,
            Timing::Normal,
            10,
            vec![
                EventKind::CardsDiscarded,
                EventKind::CardsDrawn,
                EventKind::CardsGiven,
                EventKind::CardsMoved,
            ],
        )),
        "yuling3" => Some(meta(
            EventKind::CardsDrawn,
            Timing::Normal,
            10,
            vec![EventKind::CardsGiven, EventKind::CardsMoved],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_d(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_char_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.name.as_str() {
        "jiehuo" => {
            if flag(ctx.state, owner, "gj_jiehuo_used") > 0
                || !ctx.state.alive_players().any(|p| p != owner)
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchD,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_char_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "zhaolu" => match event {
            EventData::TurnEndWindow { player, after: false } => *player == owner,
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            EventData::Death { .. } => true,
            _ => false,
        },
        "jiehuo2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } if *player == owner
        ),
        "yuling" => match event {
            EventData::CardsLost { player, .. }
            | EventData::CardsDiscarded { player, .. }
            | EventData::CardsDrawn { player, .. } => {
                *player == owner && ctx.state.player(owner).hand.len() != 5
            }
            EventData::CardsGiven {
                recipient: player,
                cards,
                ..
            }
            | EventData::CardsMoved {
                to: player, cards, ..
            } => *player == owner && !cards.is_empty() && ctx.state.player(owner).hand.len() != 5,
            _ => false,
        },
        "yuling2" => match event {
            EventData::CardsLost { player, .. }
            | EventData::CardsDiscarded { player, .. }
            | EventData::CardsDrawn { player, .. } => {
                *player == owner && ctx.state.player(owner).hand.len() < 5
            }
            EventData::CardsGiven {
                recipient: player,
                cards,
                ..
            }
            | EventData::CardsMoved {
                to: player, cards, ..
            } => *player == owner && !cards.is_empty() && ctx.state.player(owner).hand.len() < 5,
            _ => false,
        },
        "yuling3" => match event {
            EventData::CardsDrawn { player, .. } => {
                *player == owner && ctx.state.player(owner).hand.len() > 5
            }
            EventData::CardsGiven {
                recipient: player,
                cards,
                ..
            }
            | EventData::CardsMoved {
                to: player, cards, ..
            } => *player == owner && !cards.is_empty() && ctx.state.player(owner).hand.len() > 5,
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_char_d(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let card = match event {
        EventData::TurnEndWindow { .. } => Some(1),
        EventData::Damage(d) => Some((d.amount.max(0) as u32).saturating_mul(2)),
        EventData::Death { .. } => Some(2),
        _ => None,
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05GujianCharBatchD,
        owner,
        target: None,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn ensure_zhaolu_init(core: &mut GameStateCore, owner: PlayerId) -> u32 {
    let cur = flag(core, owner, "gj_zhaolu_count");
    if cur == 0 && flag(core, owner, "gj_zhaolu_inited") == 0 {
        let init = (core.alive_count().min(5) as u32).max(1);
        set_flag(core, owner, "gj_zhaolu_inited", 1);
        set_flag(core, owner, "gj_zhaolu_count", init);
        init
    } else {
        cur
    }
}

pub(super) fn step_char_d(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.def_key.as_str() {
        "zhaolu" => {
            let cur = ensure_zhaolu_init(ctx.core, machine.owner) as i32;
            let dec = machine.card.unwrap_or(1) as i32;
            let next = cur - dec;
            if next <= 0 {
                ctx.lose_max_hp(machine.owner, 1);
                if ctx.core.player(machine.owner).max_hp == 0 {
                    die_player(ctx, machine.owner, None);
                }
                let reset = (ctx.core.alive_count().min(5) as u32).max(1);
                set_flag(ctx.core, machine.owner, "gj_zhaolu_count", reset);
            } else {
                set_flag(ctx.core, machine.owner, "gj_zhaolu_count", next as u32);
            }
            Flow::Done
        }
        "jiehuo" => match machine.stage {
            HostStage::Act => {
                let opts = push_non_self_alive(ctx.core, machine.owner);
                if opts.is_empty() {
                    return Flow::Done;
                }
                wait_ask(
                    machine,
                    ctx,
                    HostStage::TargetChoice,
                    machine.owner,
                    ASK_JIEHUO_TARGET,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "gj_jiehuo_target",
                        options: opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                )
            }
            HostStage::TargetChoice => {
                let ChoiceOption::Player(t) = answer_first_or_pass(ctx) else {
                    return Flow::Done;
                };
                set_flag(ctx.core, machine.owner, "gj_jiehuo_used", 1);
                if let Some(def) = ctx.catalog.skill_by_name("jiehuo") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                        inst.once_used = true;
                        inst.enabled = false;
                    }
                }
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(damage_data_with(
                        Some(machine.owner),
                        t,
                        2,
                        DamageNature::Fire,
                        None,
                    )));
                    machine.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                die_player(ctx, machine.owner, None);
                Flow::Done
            }
            HostStage::DrawSettle => {
                die_player(ctx, machine.owner, None);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jiehuo2" => {
            die_player(ctx, machine.owner, None);
            Flow::Done
        }
        "yuling" => {
            if ctx.core.player(machine.owner).hand.len() < 5 {
                if ctx.core.player(machine.owner).alive {
                    let have = ctx.core.player(machine.owner).hand.len();
                    ctx.start_draw(machine.owner, (5 - have) as u32);
                }
                Flow::Done
            } else {
                step_yuling3(machine, ctx)
            }
        }
        "yuling2" => {
            let have = ctx.core.player(machine.owner).hand.len();
            if have < 5 && ctx.core.player(machine.owner).alive {
                ctx.start_draw(machine.owner, (5 - have) as u32);
            }
            Flow::Done
        }
        "yuling3" => step_yuling3(machine, ctx),
        _ => Flow::Done,
    }
}

fn step_yuling3(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
            HostStage::Act => {
                let have = ctx.core.player(machine.owner).hand.len();
                if have <= 5 {
                    return Flow::Done;
                }
                let need = have - 5;
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .player(machine.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                wait_ask(
                    machine,
                    ctx,
                    HostStage::SelectCard,
                    machine.owner,
                    ASK_YULING3_DISCARD,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gj_yuling3_discard",
                        options: opts,
                        min: need,
                        max: need,
                        allow_cancel: false,
                    },
                )
            }
            HostStage::SelectCard => {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
            discard_cards_from_he(ctx, machine.owner, &picked);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn ask_char_d(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCharBatchD);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCharBatchD);
    match (stage, ask_id) {
        (HostStage::TargetChoice, ASK_JIEHUO_TARGET) => build_ask(
            obs,
            "gj_jiehuo_target",
            ChoiceKind::SelectPlayers,
            push_non_self_alive(core, owner),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_YULING3_DISCARD) => {
            let need = core.player(owner).hand.len().saturating_sub(5);
            if need == 0 {
                return None;
            }
            build_ask(
                obs,
                "gj_yuling3_discard",
                ChoiceKind::SelectCards,
                core.player(owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect(),
                need,
                need,
                false,
            )
        }
        _ => None,
    }
}
