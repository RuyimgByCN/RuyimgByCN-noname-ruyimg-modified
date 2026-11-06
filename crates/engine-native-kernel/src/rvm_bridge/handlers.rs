//! Package active entry constraints and trigger handlers sharing one logical skill.
use super::*;
use crate::catalog::TriggerMeta;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmActiveLimits {
    pub once_per_turn: bool,
    pub once_per_play_phase: bool,
    pub hand_nonempty: bool,
    pub equip_nonempty: bool,
    pub hand_gt_hp: bool,
    pub hand_gt_max_hp: bool,
    pub face_up: bool,
    pub owner_unlinked: bool,
    pub mark_at_least: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmTriggerProgram {
    pub program: Arc<VmSkillProgram>,
    pub meta: TriggerMeta,
    pub filter: VmFilterPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmHandlerSet {
    pub skill_def_id: u32,
    pub active: Option<Arc<VmSkillProgram>>,
    pub limits: VmActiveLimits,
    pub triggers: Vec<VmTriggerProgram>,
}

impl VmHandlerSet {
    pub(super) fn available(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
        let p = ctx.state.player(owner);
        let Some(active) = &self.active else {
            return false;
        };
        let name = &active.skill_name;
        p.alive
            && ctx.state.turn.current == owner
            && ctx.state.turn.phase == crate::event::Phase::Play
            && ctx
                .state
                .vm_aux
                .flags
                .get(&(owner, format!("skill_banned_turn:{name}")))
                .copied()
                .unwrap_or(0)
                == 0
            && (!self.limits.once_per_turn
                || !ctx.state.vm_aux.once.contains(&(owner, name.clone(), 1)))
            && (!self.limits.once_per_play_phase
                || !ctx.state.vm_aux.once.contains(&(owner, name.clone(), 4)))
            && (!self.limits.hand_nonempty || !p.hand.is_empty())
            && (!self.limits.equip_nonempty || !p.equips.is_empty())
            && (!self.limits.hand_gt_hp || p.hand.len() as i64 > p.hp as i64)
            && (!self.limits.hand_gt_max_hp || p.hand.len() > p.max_hp as usize)
            && (!self.limits.face_up || !p.face_down)
            && (!self.limits.owner_unlinked || !p.linked)
            && self.limits.mark_at_least.is_none_or(|(mark, count)| {
                ctx.state
                    .vm_aux
                    .marks
                    .get(&(owner, mark))
                    .copied()
                    .unwrap_or(0) as i64
                    >= count as i64
            })
    }

    pub(super) fn matching<'a>(
        &'a self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> Vec<&'a VmTriggerProgram> {
        self.triggers
            .iter()
            .filter(|entry| {
                (entry.meta.event == kind || entry.meta.extra_events.contains(&kind))
                    && VmSkillBehavior {
                        program: entry.program.clone(),
                        filter: entry.filter.clone(),
                    }
                    .filter(ctx, owner, event)
            })
            .collect()
    }
}

pub(super) fn machine(
    program: Arc<VmSkillProgram>,
    filter: &VmFilterPolicy,
    owner: PlayerId,
    event: &EventData,
) -> VmMachine {
    fn named_meta(f: &VmFilterPolicy) -> (Vec<(String, u8)>, bool) {
        match f {
            VmFilterPolicy::Named(p) => (p.consume.clone(), p.count_hits),
            VmFilterPolicy::AtPhase { inner, .. } | VmFilterPolicy::Declared { inner, .. } => {
                named_meta(inner)
            }
            _ => (Vec::new(), false),
        }
    }
    let (consume, count_hits) = named_meta(filter);
    VmMachine {
        owner,
        program,
        trig: trigger_ctx_of(event),
        event: event.clone(),
        pending_effects: Vec::new(),
        paused: None,
        asked: false,
        settled: false,
        consume,
        count_hits,
        commit_pause: None,
        following: Vec::new(),
        commit_state: Vec::new(),
        committing: false,
    }
}
