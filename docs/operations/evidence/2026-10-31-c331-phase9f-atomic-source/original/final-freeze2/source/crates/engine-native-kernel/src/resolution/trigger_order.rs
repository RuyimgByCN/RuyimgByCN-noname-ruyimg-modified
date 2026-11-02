//! One recoverable window over Frame.queue, including automatic and interactive dispatch.
use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceSpec};
use crate::snap::{MachineSnapshot, ResolutionSnapshot, SnapshotError};
use std::cmp::Ordering;

pub(super) fn compare(a: &QueuedTrigger, b: &QueuedTrigger) -> Ordering {
    a.order
        .0
        .cmp(&b.order.0)
        .then_with(|| {
            if a.order.0 == 1 {
                a.order
                    .3
                    .cmp(&b.order.3)
                    .then_with(|| b.order.1.cmp(&a.order.1))
            } else {
                b.order
                    .1
                    .cmp(&a.order.1)
                    .then_with(|| a.order.3.cmp(&b.order.3))
            }
        })
        .then_with(|| a.def.cmp(&b.def))
}
fn bucket(q: &QueuedTrigger, seats: usize) -> u16 {
    match q.order.0 {
        0 => 0,
        1 => u16::from(q.order.3) + 1,
        _ => seats as u16 + 1,
    }
}
pub(crate) fn event_kind(frame: &Frame) -> EventKind {
    if frame.stage == Stage::BeforeTriggers && frame.kind == EventKind::Damage {
        EventKind::DamageBefore
    } else {
        frame.kind
    }
}
fn subscribes(catalog: &Catalog, def: SkillDefId, frame: &Frame) -> bool {
    let Some(def) = catalog.skills.get(def as usize) else {
        return false;
    };
    let Some(meta) = &def.trigger else {
        return false;
    };
    let kind = event_kind(frame);
    (meta.event == kind || meta.extra_events.contains(&kind))
        && (kind != EventKind::PhaseBegin
            || def.behavior.is_vm_skill() == (frame.stage == Stage::BeforeTriggers))
}
pub(super) fn eligible(
    core: &GameStateCore,
    catalog: &Catalog,
    q: &QueuedTrigger,
    frames: &[Frame],
    frame: &Frame,
) -> bool {
    let event = crate::event::damage_event(frames, frame);
    let kind = event_kind(frame);
    if let EventData::Damage(d) = event {
        let terminal = matches!(
            (kind, d.settlement.outcome),
            (EventKind::DamageZero, crate::event::DamageOutcome::Zero)
                | (
                    EventKind::DamageCancelled,
                    crate::event::DamageOutcome::Cancelled
                )
        );
        if d.settlement.notrigger {
            return false;
        }
        if !terminal
            && (d.amount <= 0
                || d.prevented
                || d.settlement.notrigger
                || d.settlement.outcome != crate::event::DamageOutcome::Running)
        {
            return false;
        }
    }
    if matches!(event, EventData::Death { player, .. } if core.player(*player).death_cancelled) {
        return false;
    }
    let inst = core.skills.get(&(q.owner, q.def));
    let Some(def) = catalog.skills.get(q.def as usize) else {
        return false;
    };
    let Some(meta) = &def.trigger else {
        return false;
    };
    let helper = def
        .behavior
        .native_damage_effect(kind)
        .is_some_and(|e| e.helper_name() == Some(def.name));
    (helper || inst.is_some_and(|i| i.enabled && (!def.limits.once_per_turn || !i.once_used)))
        && core.player(q.owner).alive
        && (meta.event == kind || meta.extra_events.contains(&kind))
        && if let (Some(effect), EventData::Damage(d)) =
            (def.behavior.native_damage_effect(kind), event)
        {
            let root = match frame.data {
                EventData::DamageWindow { root, .. } => frames.iter().find(|f| f.id == root),
                EventData::Damage(_) => Some(frame),
                _ => None,
            };
            let parent = root.and_then(|root| {
                root.parent.and_then(|id| {
                    let index = frames.iter().position(|f| f.id == root.id)?;
                    index
                        .checked_sub(1)
                        .and_then(|i| frames.get(i))
                        .filter(|p| p.id == id)
                })
            });
            effect.matches(
                q.owner,
                d,
                &QueryCtx {
                    state: core,
                    catalog,
                },
                parent,
                root.and_then(|root| crate::event::damage_card_kind(frames, root, catalog)),
            )
        } else {
            def.behavior.filter_in_window(
                &QueryCtx {
                    state: core,
                    catalog,
                },
                q.owner,
                event,
                kind,
            )
        }
}
fn group(
    core: &GameStateCore,
    frames: &[Frame],
    frame: &Frame,
    catalog: &Catalog,
    cursor: u16,
    seats: usize,
) -> Vec<QueuedTrigger> {
    let mut available = frame
        .queue
        .iter()
        .filter(|q| bucket(q, seats) == cursor && eligible(core, catalog, q, frames, frame));
    let Some(first) = available.next().cloned() else {
        return Vec::new();
    };
    let mut result = vec![first.clone()];
    if first.order.0 == 1 {
        result.extend(available.filter(|q| q.order.1 == first.order.1).cloned());
        result.sort_by_key(|q| q.def);
    }
    result
}

pub struct TriggerOrder {
    pub frame: EventId,
    pub owner: PlayerId,
    pub candidates: Vec<SkillDefId>,
    pub asked: bool,
    pub cursor: u16,
    pub done: Vec<(PlayerId, SkillDefId)>,
    pub owners: Vec<PlayerId>,
    pub normal_owners: u16,
}
impl TriggerOrder {
    pub(super) fn new(frame: EventId, core: &GameStateCore) -> Self {
        let mut owners: Vec<_> = (0..core.players.len() as PlayerId).collect();
        owners.sort_by_key(|owner| core.seat_steps_from(core.turn.current, *owner));
        Self {
            frame,
            owner: 0,
            candidates: Vec::new(),
            asked: false,
            cursor: 0,
            done: Vec::new(),
            normal_owners: owners.len() as u16,
            owners,
        }
    }
    fn start(&mut self, ctx: &mut Ctx<'_>, selected: &QueuedTrigger) {
        self.done.push((selected.owner, selected.def));
        start_queued_trigger(
            ctx.core,
            ctx.res,
            ctx.catalog,
            ctx.out,
            self.frame,
            selected.def,
            selected.owner,
        );
    }
}
impl SkillMachine for TriggerOrder {
    fn trigger_order(&self) -> Option<&TriggerOrder> {
        Some(self)
    }
    fn trigger_order_mut(&mut self) -> Option<&mut TriggerOrder> {
        Some(self)
    }
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            frame: self.frame,
            owner: self.owner,
            candidates: self.candidates.clone(),
            asked: self.asked,
            cursor: self.cursor,
            done: self.done.clone(),
            owners: self.owners.clone(),
            normal_owners: self.normal_owners,
        })
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::TriggerOrder {
            frame: self.frame,
            owner: self.owner,
            candidates: self.candidates.clone(),
            asked: self.asked,
            cursor: self.cursor,
            done: self.done.clone(),
            owners: self.owners.clone(),
            normal_owners: self.normal_owners,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(frame) = ctx.res.frame(self.frame) else {
            return Flow::Done;
        };
        let mut current = group(
            ctx.core,
            &ctx.res.stack,
            frame,
            ctx.catalog,
            self.cursor,
            self.normal_owners as usize,
        );
        if self.asked {
            let selected = match ctx.answer().first() {
                Some(ChoiceOption::Skill(def)) if self.candidates.contains(def) => current
                    .iter()
                    .find(|q| q.def == *def && q.owner == self.owner)
                    .cloned(),
                _ => None,
            };
            self.asked = false;
            self.candidates.clear();
            self.owner = 0;
            if let Some(selected) = selected.filter(|_| {
                current.iter().all(|q| {
                    ctx.catalog.skills[q.def as usize]
                        .trigger
                        .as_ref()
                        .expect("meta")
                        .order_selectable
                })
            }) {
                self.start(ctx, &selected);
                return Flow::Yield;
            }
        }
        while current.is_empty() {
            let seats = self.normal_owners as usize;
            ctx.res
                .frame_mut(self.frame)
                .expect("frame")
                .queue
                .retain(|q| bucket(q, seats) > self.cursor);
            self.cursor += 1;
            if self.cursor > seats as u16 + 1 {
                let frame = ctx.res.frame_mut(self.frame).expect("frame");
                frame.stage = next_stage(frame.kind, frame.stage);
                frame.planned = false;
                return Flow::Done;
            }
            current = group(
                ctx.core,
                &ctx.res.stack,
                ctx.res.frame(self.frame).expect("frame"),
                ctx.catalog,
                self.cursor,
                self.normal_owners as usize,
            );
        }
        let first = &current[0];
        // Retain temporarily false peers, but discard higher unavailable priorities once passed.
        ctx.res
            .frame_mut(self.frame)
            .expect("frame")
            .queue
            .retain(|q| {
                bucket(q, self.normal_owners as usize) != self.cursor || q.order.1 <= first.order.1
            });
        let automatic = if first.order.0 != 1 {
            Some(first.clone())
        } else {
            current
                .iter()
                .find(|q| {
                    !ctx.catalog.skills[q.def as usize]
                        .trigger
                        .as_ref()
                        .expect("meta")
                        .order_selectable
                })
                .cloned()
        };
        if let Some(selected) = automatic.or_else(|| (current.len() == 1).then(|| first.clone())) {
            self.start(ctx, &selected);
            return Flow::Yield;
        }
        self.owner = first.owner;
        self.candidates = current.iter().map(|q| q.def).collect();
        self.asked = true;
        Flow::WaitChoice {
            observer: self.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "trigger_order",
                options: self
                    .candidates
                    .iter()
                    .copied()
                    .map(ChoiceOption::Skill)
                    .collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }
}

/// New seats are inventory only: upstream addTrigger cannot create an absent Normal map.
pub fn sync_trigger_owners(core: &GameStateCore, res: &mut Resolution) {
    for park in &mut res.parks {
        if let Some(window) = park.machine.trigger_order_mut() {
            window
                .owners
                .extend((window.owners.len()..core.players.len()).map(|p| p as PlayerId));
        }
    }
}

/// Register a genuinely new instance in each still-open ancestor window, as upstream addTrigger does.
pub fn register_added(
    core: &GameStateCore,
    res: &mut Resolution,
    catalog: &Catalog,
    owner: PlayerId,
    def: SkillDefId,
) {
    sync_trigger_owners(core, res);
    let Some(meta) = catalog
        .skills
        .get(def as usize)
        .and_then(|d| d.trigger.as_ref())
    else {
        return;
    };
    let additions: Vec<_> = res
        .parks
        .iter()
        .filter_map(|p| p.machine.trigger_order())
        .filter_map(|m| {
            let native_meta = catalog.skills[def as usize]
                .behavior
                .native_damage_effect(event_kind(res.frame(m.frame)?))
                .map(|e| e.meta());
            let meta = native_meta.as_ref().unwrap_or(meta);
            let seat = m.owners.iter().position(|p| *p == owner)? as u8;
            if meta.timing == Timing::Normal && usize::from(seat) >= m.normal_owners as usize {
                return None;
            }
            let q = QueuedTrigger {
                owner,
                def,
                order: (
                    match meta.timing {
                        Timing::First => 0,
                        Timing::Normal => 1,
                        Timing::Last => 2,
                    },
                    meta.priority,
                    0,
                    seat,
                    def,
                ),
            };
            (bucket(&q, m.normal_owners as usize) >= m.cursor && !m.done.contains(&(owner, def)))
                .then_some((m.frame, q))
        })
        .collect();
    for (frame, q) in additions {
        let Some(f) = res.frame_mut(frame) else {
            continue;
        };
        if !subscribes(catalog, def, f)
            || f.queue
                .iter()
                .any(|old| old.owner == owner && old.def == def)
        {
            continue;
        }
        f.queue.push(q);
        f.queue.sort_by(compare);
    }
}

/// Parallel authoritative changes invalidate an outdated order ask before republishing a new id.
pub fn refresh_order_choice(core: &GameStateCore, res: &mut Resolution, catalog: &Catalog) {
    if !res
        .open_choice
        .as_ref()
        .is_some_and(|c| c.spec.prompt == "trigger_order")
    {
        return;
    }
    let Some(index) = res
        .parks
        .iter()
        .position(|p| p.machine.trigger_order().is_some_and(|m| m.asked))
    else {
        return;
    };
    let machine = res.parks[index].machine.trigger_order().expect("window");
    let current = res
        .frame(machine.frame)
        .map(|f| {
            group(
                core,
                &res.stack,
                f,
                catalog,
                machine.cursor,
                machine.normal_owners as usize,
            )
        })
        .unwrap_or_default();
    if current
        .iter()
        .map(|q| q.def)
        .eq(machine.candidates.iter().copied())
        && current.iter().all(|q| {
            q.owner == machine.owner
                && catalog.skills[q.def as usize]
                    .trigger
                    .as_ref()
                    .is_some_and(|m| m.order_selectable)
        })
    {
        return;
    }
    let machine = res.parks[index]
        .machine
        .trigger_order_mut()
        .expect("window");
    machine.asked = false;
    machine.candidates.clear();
    machine.owner = 0;
    res.open_choice = None;
}

/// Check window, queue, and choice bindings against the same frozen catalog, including checkpoints.
pub fn validate_trigger_bindings(
    res: &ResolutionSnapshot,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Result<(), SnapshotError> {
    let bad = || SnapshotError::BadTag("trigger_order_binding", 0);
    for (depth, frame) in res.stack.iter().enumerate() {
        let windows: Vec<_> = res
            .parks
            .iter()
            .filter_map(|park| match &park.machine {
                MachineSnapshot::TriggerOrder {
                    frame: id, owners, ..
                } if *id == frame.id => Some((park, owners)),
                _ => None,
            })
            .collect();
        let needs_window =
            frame.planned && matches!(frame.stage, Stage::BeforeTriggers | Stage::AfterTriggers);
        if windows.len() != usize::from(needs_window) {
            return Err(bad());
        }
        if windows
            .first()
            .is_some_and(|(park, _)| park.resume_depth != depth + 1)
        {
            return Err(bad());
        }
        let mut seen = std::collections::BTreeSet::new();
        for q in &frame.queue {
            let skill = catalog.skills.get(q.def as usize).ok_or_else(bad)?;
            let native_meta = skill
                .behavior
                .native_damage_effect(event_kind(frame))
                .map(|e| e.meta());
            let meta = native_meta
                .as_ref()
                .or(skill.trigger.as_ref())
                .ok_or_else(bad)?;
            let expected = (
                match meta.timing {
                    Timing::First => 0,
                    Timing::Normal => 1,
                    Timing::Last => 2,
                },
                meta.priority,
                0,
                windows
                    .first()
                    .and_then(|(_, owners)| owners.iter().position(|p| *p == q.owner))
                    .ok_or_else(bad)? as u8,
                q.def,
            );
            if windows.first().is_some_and(|(park, _)| matches!(&park.machine, MachineSnapshot::TriggerOrder { normal_owners, .. } if q.order.0 == 1 && u16::from(q.order.3) >= *normal_owners))
                || q.order != expected
                || !seen.insert((q.owner, q.def))
                || !subscribes(catalog, q.def, frame)
            {
                return Err(bad());
            }
        }
        if frame
            .queue
            .windows(2)
            .any(|qs| compare(&qs[0], &qs[1]) == Ordering::Greater)
        {
            return Err(bad());
        }
    }
    for park in &res.parks {
        if !matches!(park.machine, MachineSnapshot::NativeDamageSkill { .. })
            && contains_native_damage(&park.machine)
        {
            return Err(bad());
        }
    }
    for park in &res.parks {
        if let MachineSnapshot::NativeDamageSkill { owner, def, window } = &park.machine {
            let Slot::Trigger { frame } = park.slot else {
                return Err(bad());
            };
            let f = res.stack.iter().find(|f| f.id == frame).ok_or_else(bad)?;
            let effect = catalog
                .skills
                .get(*def as usize)
                .and_then(|s| s.behavior.native_damage_effect(window.event_kind()))
                .ok_or_else(bad)?;
            let admitted=res.parks.iter().any(|p|matches!(&p.machine,MachineSnapshot::TriggerOrder{frame:order_frame,done,..}if *order_frame==frame&&done.contains(&(*owner,*def))));
            let q = QueuedTrigger {
                owner: *owner,
                def: *def,
                order: (0, 0, 0, 0, *def),
            };
            if !matches!(f.data,EventData::DamageWindow{window:w,..}if w==*window)
                || f.stage != Stage::AfterTriggers
                || park.resume_depth
                    != res.stack.iter().position(|frame| frame.id == f.id).unwrap() + 1
                || effect.window() != *window
                || !admitted
                || !eligible(core, catalog, &q, &res.stack, f)
            {
                return Err(bad());
            }
        }
    }
    for park in &res.parks {
        if requires_trigger_frame(&park.machine, catalog) {
            let Slot::Trigger { frame } = park.slot else {
                return Err(bad());
            };
            let (depth, frame) = res
                .stack
                .iter()
                .enumerate()
                .find(|(_, f)| f.id == frame && frame != NO_FRAME)
                .ok_or_else(bad)?;
            if park.resume_depth != depth + 1 {
                return Err(bad());
            }
            validate_vm_window(&park.machine, frame, catalog)?;
        } else if let Slot::Trigger { frame } = park.slot {
            if frame != NO_FRAME {
                let frame = res.stack.iter().find(|f| f.id == frame).ok_or_else(bad)?;
                validate_vm_window(&park.machine, frame, catalog)?;
            }
        }
        if !matches!(park.machine, MachineSnapshot::TriggerOrder { .. })
            && contains_window(&park.machine)
        {
            return Err(bad());
        }
        if let MachineSnapshot::OptionalGate {
            owner,
            def,
            event,
            inner: None,
            ..
        } = &park.machine
        {
            let id = match park.slot {
                Slot::Trigger { frame } => frame,
                _ => return Err(bad()),
            };
            let frame = if id == NO_FRAME {
                park.resume_depth
                    .checked_sub(1)
                    .and_then(|i| res.stack.get(i))
            } else {
                res.stack.iter().find(|f| f.id == id)
            }
            .ok_or_else(bad)?;
            let q = QueuedTrigger {
                owner: *owner,
                def: *def,
                order: (0, 0, 0, 0, *def),
            };
            if *event != frame.data || !eligible(core, catalog, &q, &res.stack, frame) {
                return Err(bad());
            }
        }
        if let MachineSnapshot::TriggerOrder {
            frame,
            owner,
            candidates,
            asked,
            cursor,
            done,
            owners,
            normal_owners,
        } = &park.machine
        {
            let f = res.stack.iter().find(|f| f.id == *frame).ok_or_else(bad)?;
            let mut seen = std::collections::BTreeSet::new();
            let mut player_ids = std::collections::BTreeSet::new();
            if owners.len() != core.players.len()
                || *normal_owners == 0
                || *normal_owners as usize > owners.len()
                || owners[..*normal_owners as usize]
                    .iter()
                    .any(|pid| u16::from(*pid) >= *normal_owners)
                || !owners[*normal_owners as usize..]
                    .iter()
                    .copied()
                    .eq((*normal_owners as usize..core.players.len()).map(|p| p as PlayerId))
                || owners
                    .iter()
                    .any(|pid| *pid as usize >= core.players.len() || !player_ids.insert(*pid))
            {
                return Err(bad());
            }
            if park.slot != (Slot::Trigger { frame: *frame })
                || !f.planned
                || !matches!(f.stage, Stage::BeforeTriggers | Stage::AfterTriggers)
                || *cursor > *normal_owners + 1
                || done
                    .iter()
                    .any(|key| !seen.insert(*key) || !subscribes(catalog, key.1, f))
                || f.queue.iter().any(|q| {
                    bucket(q, *normal_owners as usize) < *cursor || seen.contains(&(q.owner, q.def))
                })
            {
                return Err(bad());
            }
            if *asked {
                let qs = group(
                    core,
                    &res.stack,
                    f,
                    catalog,
                    *cursor,
                    *normal_owners as usize,
                );
                let Some(open) = &res.open_choice else {
                    return Err(bad());
                };
                if park.resume_depth != res.stack.len()
                    || res.stack.last().map(|f| f.id) != Some(*frame)
                    || qs.len() < 2
                    || qs.iter().any(|q| {
                        q.owner != *owner
                            || q.order.0 != 1
                            || !catalog.skills[q.def as usize]
                                .trigger
                                .as_ref()
                                .expect("meta")
                                .order_selectable
                    })
                    || *candidates != qs.iter().map(|q| q.def).collect::<Vec<_>>()
                    || open.observer != *owner
                    || open.spec.prompt != "trigger_order"
                    || open.spec.kind != ChoiceKind::PickTag
                    || open.spec.min != 1
                    || open.spec.max != 1
                    || open.spec.allow_cancel
                    || open.spec.options
                        != candidates
                            .iter()
                            .copied()
                            .map(ChoiceOption::Skill)
                            .collect::<Vec<_>>()
                {
                    return Err(bad());
                }
            } else if !candidates.is_empty() || *owner != 0 {
                return Err(bad());
            }
        }
    }
    Ok(())
}

fn contains_window(machine: &MachineSnapshot) -> bool {
    match machine {
        MachineSnapshot::TriggerOrder { .. } => true,
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Chain { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. } => {
            inner.as_deref().is_some_and(contains_window)
        }
        MachineSnapshot::Rvm { following, .. } => following.iter().any(contains_window),
        _ => false,
    }
}

fn contains_native_damage(machine: &MachineSnapshot) -> bool {
    match machine {
        MachineSnapshot::NativeDamageSkill { .. } => true,
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Chain { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. } => {
            inner.as_deref().is_some_and(contains_native_damage)
        }
        MachineSnapshot::Rvm { following, .. } => following.iter().any(contains_native_damage),
        _ => false,
    }
}

// A module already identifies a handler; only the owning frame decides its before/after window.
fn validate_vm_window(
    machine: &MachineSnapshot,
    frame: &Frame,
    catalog: &Catalog,
) -> Result<(), SnapshotError> {
    match machine {
        MachineSnapshot::Rvm {
            module,
            skill_name,
            event,
            following,
            ..
        } => {
            let def = catalog
                .skill_by_name(skill_name)
                .ok_or(SnapshotError::UnknownMachine)?;
            let behavior = &catalog.skills[def as usize].behavior;
            let data_kind = if matches!(frame.data, EventData::DamageWindow { .. }) {
                EventKind::Damage
            } else {
                frame.data.kind()
            };
            if event.kind() != data_kind
                || !behavior.vm_trigger_window_is_authorized(module, event_kind(frame))
            {
                return Err(SnapshotError::UnknownMachine);
            }
            for next in following {
                validate_vm_window(next, frame, catalog)?;
            }
        }
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Chain { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. } => {
            if let Some(inner) = inner {
                validate_vm_window(inner, frame, catalog)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn requires_trigger_frame(machine: &MachineSnapshot, catalog: &Catalog) -> bool {
    match machine {
        MachineSnapshot::Rvm {
            module,
            skill_name,
            following,
            ..
        } => {
            catalog.skill_by_name(skill_name).is_some_and(|def| {
                let def = &catalog.skills[def as usize];
                def.trigger.is_some() && def.behavior.vm_module_requires_trigger_frame(module)
            }) || following.iter().any(|m| requires_trigger_frame(m, catalog))
        }
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Chain { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. } => inner
            .as_deref()
            .is_some_and(|m| requires_trigger_frame(m, catalog)),
        _ => false,
    }
}
