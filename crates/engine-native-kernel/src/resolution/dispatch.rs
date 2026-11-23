//! 队列触发起机与动态新增技能注册。

use super::{trigger_order, OptionalGate, Park, Resolution, Slot, NO_FRAME};
use crate::catalog::{QueryCtx, SkillMachine};
use crate::event::{EventData, EventKind};
use crate::ids::{EventId, PlayerId, SkillDefId};
use crate::kernel;
use crate::machine::LogEvent;
use crate::state::GameStateCore;

pub(super) fn native_machine(
    owner: PlayerId,
    def: SkillDefId,
    effect: crate::event::NativeDamageEffect,
) -> Box<dyn SkillMachine> {
    if crate::event::uses_native_invocation(effect) {
        Box::new(kernel::NativeInvocation {
            owner,
            def,
            window: effect.window(),
            progress: crate::event::NativeInvocationProgress::Ready,
        })
    } else {
        Box::new(kernel::NativeDamageSkill {
            owner,
            def,
            window: effect.window(),
        })
    }
}

pub(super) fn start_queued_trigger(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    out: &mut Vec<LogEvent>,
    frame_id: EventId,
    def_id: SkillDefId,
    owner: PlayerId,
) {
    let Some(index) = res.frame(frame_id).and_then(|f| {
        f.queue
            .iter()
            .position(|q| q.def == def_id && q.owner == owner)
    }) else {
        return;
    };
    let qt = res.frame_mut(frame_id).expect("frame").queue.remove(index);
    let actual_event =
        crate::event::damage_event(&res.stack, res.frame(frame_id).expect("frame")).clone();
    if !trigger_order::eligible(
        core,
        catalog,
        &qt,
        &res.stack,
        res.frame(frame_id).expect("frame"),
    ) {
        return;
    }
    let def = &catalog.skills[qt.def as usize];
    let kind = trigger_order::event_kind(res.frame(frame_id).expect("frame"));
    let native_meta = def.behavior.native_damage_effect(kind).map(|e| e.meta());
    let vm_meta = def.behavior.window_trigger_meta(
        &QueryCtx {
            state: core,
            catalog,
        },
        qt.owner,
        &actual_event,
        kind,
    );
    let meta = native_meta
        .as_ref()
        .or(vm_meta.as_ref())
        .or(def.trigger.as_ref())
        .expect("eligible meta");
    let skip_optional_gate = def.name == "dddmoyan" && kind == EventKind::DamageBefore;
    let machine: Box<dyn SkillMachine> = if meta.optional && !skip_optional_gate {
        let ev = if def.behavior.confirmation_actor(qt.owner, &actual_event, kind) != qt.owner {
            actual_event
        } else {
            res.frame(frame_id).expect("frame").data.clone()
        };
        Box::new(OptionalGate::new(qt.owner, qt.def, ev, false, None))
    } else {
        if def.limits.once_per_turn {
            let inst = core.skills.get_mut(&(qt.owner, qt.def)).expect("instance");
            if inst.once_used {
                return;
            }
            inst.once_used = true;
        }
        let native = def.behavior.native_damage_effect(kind);
        let skip_pre_log = matches!(
            native,
            Some(
                crate::event::NativeDamageEffect::Qinglong
                    | crate::event::NativeDamageEffect::Zhangwu
                    | crate::event::NativeDamageEffect::Yuance
                    | crate::event::NativeDamageEffect::Jidian
                    | crate::event::NativeDamageEffect::JidingRecover
            )
        ) || matches!(
            def.name,
            crate::vocabulary::host::HUANLE_MONKEY_KEY
                | "monkey"
                | "huanle_monkey"
                | crate::vocabulary::host::HLSS_MBCMJIUSHI_KEY
                | "mbcmqingzheng"
        );
        if !skip_pre_log {
            let mut log_ev = LogEvent::SkillTriggered {
                skill: def.name,
                owner: qt.owner,
                presentation: None,
            };
            crate::machine::presentation::attach_occurrence_presentation_with_skill_def(
                core,
                res,
                catalog,
                Some(qt.def),
                &mut log_ev,
            );
            out.push(log_ev);
        }
        if let Some(effect) = native {
            res.parks.push(Park {
                resume_depth: res.stack.len(),
                slot: Slot::Trigger { frame: frame_id },
                awaiting_answer: false,
                machine: native_machine(qt.owner, qt.def, effect),
            });
            return;
        }
        def.behavior.start_in_window(
            &QueryCtx {
                state: core,
                catalog,
            },
            qt.owner,
            &actual_event,
            trigger_order::event_kind(res.frame(frame_id).expect("frame")),
        )
    };
    res.parks.push(Park {
        resume_depth: res.stack.len(),
        slot: Slot::Trigger { frame: frame_id },
        machine,
        awaiting_answer: false,
    });
}

pub(super) fn finish_slot(core: &mut GameStateCore, res: &mut Resolution, slot: Slot) {
    if let Slot::Intrinsic { frame } = slot {
        if frame == NO_FRAME {
            return;
        }
        if super::pop_cleanup::advance_jifeng_slot(core, res, frame) {
            return;
        }
        if let Some(f) = res.frame_mut(frame) {
            f.intrinsic_done = true;
        }
    }
}

pub(super) fn notify_new_skills(
    core: &mut GameStateCore,
    res: &mut Resolution,
    catalog: &crate::catalog::Catalog,
    prior: &std::collections::BTreeSet<(PlayerId, SkillDefId)>,
    out: &mut Vec<LogEvent>,
) {
    trigger_order::sync_trigger_owners(core, res);
    let added_keys: Vec<(PlayerId, SkillDefId)> = core
        .skills
        .keys()
        .filter(|key| !prior.contains(key))
        .copied()
        .collect();
    for (owner, def) in added_keys {
        if catalog.skill_name(def) == "ssr_luoyi" {
            let discarded = kernel::discard_armor_for_ssr_luoyi(core, catalog, owner);
            if !discarded.is_empty() {
                crate::machine::note_cards_lost(core, catalog, owner, &discarded);
                out.push(LogEvent::CardDiscarded {
                    player: owner,
                    cards: discarded,
                });
            }
        }
        trigger_order::register_added(core, res, catalog, owner, def);
    }
    let mut helpers = Vec::new();
    for frame in &res.stack {
        if !matches!(frame.data, EventData::DamageWindow { .. }) {
            continue;
        }
        let kind = trigger_order::event_kind(frame);
        for &def in catalog.candidates(kind) {
            let skill = &catalog.skills[def as usize];
            if !skill
                .behavior
                .native_damage_effect(kind)
                .is_some_and(|e| e.helper_name() == Some(skill.name))
            {
                continue;
            }
            for owner in 0..core.players.len() as PlayerId {
                let q = crate::event::QueuedTrigger {
                    owner,
                    def,
                    order: (0, 0, 0, 0, def),
                };
                if trigger_order::eligible(core, catalog, &q, &res.stack, frame) {
                    helpers.push((owner, def));
                }
            }
        }
    }
    helpers.sort_unstable();
    helpers.dedup();
    for (owner, def) in helpers {
        trigger_order::register_added(core, res, catalog, owner, def);
    }
}
