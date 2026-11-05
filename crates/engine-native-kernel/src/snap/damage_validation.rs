//! Damage receipt, progress, child ancestry and continuation must describe one execution.
use super::*;
use crate::event::{
    DamageData, DamageOutcome as O, DamageProgress as P, DamageWindowKind as W, Frame, Stage,
};

pub(super) fn check_settlement(d: &DamageData, b: &RestoreBounds) -> Result<(), SnapshotError> {
    let s = &d.settlement;
    let receipt = s.committed_damage.is_some();
    super::validation::check_opt_pid(s.cleared_source, b)?;
    if s.cleared_source.is_some() && d.source.is_some() {
        return Err(SnapshotError::BadTag("damage_cleared_source", 0));
    }
    if s.committed_damage == Some(0) || s.hp_loss > s.committed_damage.unwrap_or(0) {
        return Err(SnapshotError::BadTag("damage_settlement", 0));
    }
    let post = matches!(
        s.progress,
        P::HpCommitted { .. }
            | P::TargetDamage { .. }
            | P::DyingPending { .. }
            | P::DyingDone { .. }
            | P::Source { .. }
            | P::End { .. }
            | P::After { .. }
            | P::Chain { .. }
    );
    let valid = match s.outcome {
        O::Running => {
            if receipt {
                post
            } else {
                s.progress.is_pre_hp()
            }
        }
        O::Completed => receipt && s.progress == P::Done,
        O::Cancelled => {
            matches!(s.progress, P::CancelledPending | P::Stopped) || s.progress.window().is_some()
        }
        O::ReplacedByLoseHp => !receipt && (s.progress.is_pre_hp() || s.progress == P::Stopped),
        O::Zero => matches!(s.progress, P::ZeroPending | P::Stopped),
    };
    if !valid || (!receipt && post) || (receipt && s.progress.is_pre_hp()) {
        return Err(SnapshotError::BadTag("damage_outcome_progress", 0));
    }
    if s.progress.chain_eligible() == Some(true)
        && (d.from_chain || d.nature == crate::event::DamageNature::Normal)
    {
        return Err(SnapshotError::BadTag("damage_chain_eligibility", 0));
    }
    if let P::Chain { remaining } = &s.progress {
        super::validation::check_pids(remaining, b)?;
        let unique: std::collections::BTreeSet<_> = remaining.iter().collect();
        if unique.len() != remaining.len()
            || remaining.contains(&d.target)
            || (!remaining.is_empty()
                && (d.from_chain || d.nature == crate::event::DamageNature::Normal))
        {
            return Err(SnapshotError::BadTag("damage_chain_targets", 0));
        }
    }
    Ok(())
}

pub(super) fn check_frame(f: &Frame, res: &ResolutionSnapshot) -> Result<(), SnapshotError> {
    if let EventData::DamageWindow { root, window } = &f.data {
        let parent = res
            .stack
            .iter()
            .find(|p| p.id == *root)
            .ok_or(SnapshotError::BadTag("damage_window_root", *root as u64))?;
        let EventData::Damage(d) = &parent.data else {
            return Err(SnapshotError::BadTag("damage_window_root_type", 0));
        };
        if f.parent != Some(*root) || *root == f.id || parent.stage != Stage::Intrinsic {
            return Err(SnapshotError::BadTag("damage_window_ancestry", 0));
        }
        let trigger_parks = res
            .parks
            .iter()
            .filter(|p| matches!(p.slot,crate::resolution::Slot::Trigger{frame}if frame==f.id))
            .count();
        let intrinsic_park = res
            .parks
            .iter()
            .any(|p| matches!(p.slot,crate::resolution::Slot::Intrinsic{frame}if frame==f.id));
        let reachable = match f.stage {
            Stage::Commit | Stage::Done => !f.planned && f.queue.is_empty() && trigger_parks == 0,
            Stage::AfterTriggers => {
                let orders=res.parks.iter().filter(|p|matches!(&p.machine,MachineSnapshot::TriggerOrder{frame,..}if *frame==f.id)).count();
                if f.planned {
                    orders == 1 || (orders == 0 && trigger_parks == 0 && f.queue.is_empty())
                } else {
                    f.queue.is_empty() && trigger_parks == 0
                }
            }
            _ => false,
        };
        if !reachable || f.intrinsic_done || intrinsic_park {
            return Err(SnapshotError::BadTag("damage_window_stage", 0));
        }
        let terminal = matches!(
            (window, d.settlement.outcome, &d.settlement.progress),
            (W::Zero, O::Zero, P::Stopped) | (W::Cancelled, O::Cancelled, P::Stopped)
        );
        if !terminal && d.settlement.progress.window() != Some(*window) {
            return Err(SnapshotError::BadTag("damage_window_progress", 0));
        }
    }
    if let EventData::Dying { player, .. } = &f.data {
        if let Some(parent) = f
            .parent
            .and_then(|id| res.stack.iter().find(|parent| parent.id == id))
        {
            if let EventData::Damage(d) = &parent.data {
                if *player != d.target || !matches!(d.settlement.progress, P::DyingPending { .. }) {
                    return Err(SnapshotError::BadTag("damage_dying_binding", 0));
                }
            }
        }
    }
    if let EventData::Damage(d) = &f.data {
        let index = res
            .stack
            .iter()
            .position(|p| p.id == f.id)
            .expect("current frame");
        let immediate = index.checked_sub(1).and_then(|i| res.stack.get(i));
        if f.parent != immediate.map(|p| p.id) {
            return Err(SnapshotError::BadTag("damage_immediate_parent", 0));
        }
        let chain = immediate.is_some_and(|p| {
            matches!(&p.data,EventData::Damage(parent)
            if p.stage==Stage::Intrinsic && matches!(parent.settlement.progress,P::Chain{..}))
        });
        if d.from_chain != chain {
            return Err(SnapshotError::BadTag("damage_chain_cause", 0));
        }
        if chain {
            let parent = immediate.unwrap().damage();
            if parent.from_chain
                || parent.nature == crate::event::DamageNature::Normal
                || parent.amount <= 0
                || parent.settlement.committed_damage.is_none()
                || parent.settlement.outcome != O::Running
            {
                return Err(SnapshotError::BadTag("damage_chain_parent", 0));
            }
            if d.settlement.progress == P::Start
                && (d.amount != parent.amount
                    || d.nature != parent.nature
                    || d.source != parent.source
                    || d.card != parent.card
                    || d.target == parent.target
                    || matches!(&parent.settlement.progress,P::Chain{remaining}if remaining.contains(&d.target)))
            {
                return Err(SnapshotError::BadTag("damage_chain_birth", 0));
            }
        }
        if !matches!(f.stage, Stage::Intrinsic | Stage::Done) || !f.queue.is_empty() || f.planned {
            return Err(SnapshotError::BadTag("damage_root_stage", 0));
        }
        let drivers = res
            .parks
            .iter()
            .filter(|p| matches!(p.machine,MachineSnapshot::DamageFlow{root}if root==f.id))
            .count();
        let terminal = matches!(d.settlement.progress, P::Done | P::Stopped);
        if drivers > 1
            || (d.settlement.progress != P::Start && !f.intrinsic_done && drivers != 1)
            || (f.intrinsic_done && !terminal)
        {
            return Err(SnapshotError::BadTag("damage_root_continuation", 0));
        }
        if f.stage == Stage::Done && (!terminal || !f.intrinsic_done || drivers != 0) {
            return Err(SnapshotError::BadTag("damage_finished_continuation", 0));
        }
    }
    check_draw_frame(f, res)?;
    Ok(())
}

fn check_draw_frame(f: &Frame, res: &ResolutionSnapshot) -> Result<(), SnapshotError> {
    use crate::event::DrawGainProgress as GP;
    if let EventData::DrawWindow { root, .. } = &f.data {
        let parent = res
            .stack
            .iter()
            .find(|p| p.id == *root)
            .ok_or(SnapshotError::BadTag("draw_window_root", *root as u64))?;
        if !matches!(parent.data, EventData::DrawPhase { .. })
            || f.parent != Some(*root)
            || *root == f.id
            || parent.stage != Stage::Intrinsic
        {
            return Err(SnapshotError::BadTag("draw_window_ancestry", 0));
        }
    }
    if let EventData::DrawGain {
        draw,
        eligible,
        progress,
        history_recorded,
        cancelled,
        ..
    } = &f.data
    {
        let parent = res
            .stack
            .iter()
            .find(|p| p.id == *draw)
            .ok_or(SnapshotError::BadTag("draw_gain_root", *draw as u64))?;
        let EventData::Draw { held_cards, .. } = &parent.data else {
            return Err(SnapshotError::BadTag("draw_gain_root_type", 0));
        };
        if f.parent != Some(*draw) || *draw == f.id || parent.stage != Stage::Intrinsic {
            return Err(SnapshotError::BadTag("draw_gain_ancestry", 0));
        }
        let unique: std::collections::BTreeSet<_> = eligible.iter().copied().collect();
        if unique.len() != eligible.len() {
            return Err(SnapshotError::BadTag("draw_gain_eligible_dup", 0));
        }
        let in_held = eligible.iter().all(|c| held_cards.contains(c));
        let none_in_held = eligible.iter().all(|c| !held_cards.contains(c));
        let valid = match progress {
            GP::Filter | GP::RecordHistory => !*history_recorded && !*cancelled && in_held,
            GP::Transfer => *history_recorded && !*cancelled && !eligible.is_empty() && in_held,
            GP::After => *history_recorded && !*cancelled && !eligible.is_empty() && none_in_held,
            GP::Done => {
                held_cards.is_empty()
                    && (*cancelled || eligible.is_empty() || (*history_recorded && none_in_held))
            }
        };
        if !valid {
            return Err(SnapshotError::BadTag("draw_gain_stage_invariant", 0));
        }
    }
    Ok(())
}

pub(super) fn check_park_binding(
    p: &ParkSnapshot,
    res: &ResolutionSnapshot,
) -> Result<(), SnapshotError> {
    match p.machine {
        MachineSnapshot::DrawPhase { root } => {
            if !matches!(p.slot, crate::resolution::Slot::Intrinsic { frame } if frame == root)
                || !res.stack.iter().any(|f| {
                    f.id == root
                        && f.stage == Stage::Intrinsic
                        && matches!(f.data, EventData::DrawPhase { .. })
                })
            {
                return Err(SnapshotError::BadTag("draw_phase_binding", root as u64));
            }
        }
        MachineSnapshot::Draw { root } => {
            if !matches!(p.slot, crate::resolution::Slot::Intrinsic { frame } if frame == root)
                || !res.stack.iter().any(|f| {
                    f.id == root
                        && f.stage == Stage::Intrinsic
                        && matches!(f.data, EventData::Draw { .. })
                })
            {
                return Err(SnapshotError::BadTag("draw_binding", root as u64));
            }
        }
        MachineSnapshot::Gain { root } => {
            if !matches!(p.slot, crate::resolution::Slot::Intrinsic { frame } if frame == root)
                || !res.stack.iter().any(|f| {
                    f.id == root
                        && f.stage == Stage::Intrinsic
                        && matches!(f.data, EventData::DrawGain { .. })
                })
            {
                return Err(SnapshotError::BadTag("draw_gain_binding", root as u64));
            }
        }
        _ => {}
    }
    Ok(())
}
