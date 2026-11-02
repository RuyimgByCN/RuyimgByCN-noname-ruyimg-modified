//! Validate the admitted operation and its child receipts, without re-querying paid providers.
use super::*;
use crate::event::{
    InvocationData, JudgePurpose, NativeDamageEffect as E, NativeInvocationProgress as P,
};
use crate::snap::{ResolutionSnapshot, SnapshotError};
fn bad() -> SnapshotError {
    SnapshotError::IdOutOfBounds {
        what: "native_operation_receipt",
        got: 0,
    }
}
pub(super) fn validate_operation(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    res: &ResolutionSnapshot,
    frame: EventId,
    context: &InvocationData,
    effect: E,
    progress: &P,
) -> Result<(), SnapshotError> {
    let allowed = match effect {
        E::Qinglong => matches!(
            progress,
            P::Body
                | P::QinglongAsk { .. }
                | P::QinglongLog { .. }
                | P::QinglongLoss { .. }
                | P::BodyDone
        ),
        E::Zhangwu => matches!(
            progress,
            P::Body
                | P::HolderAsk { .. }
                | P::HolderPay { .. }
                | P::HolderLoss { .. }
                | P::HolderAfterLog { .. }
                | P::SourceAsk { .. }
                | P::SourceLoss
        ),
        E::Jiding => matches!(progress, P::Body | P::JidingCard { .. } | P::SourceLoss),
        E::Yuance => matches!(
            progress,
            P::Body
                | P::YuanceAsk
                | P::AwaitJudge {
                    purpose: JudgePurpose::Yuance(_),
                    ..
                }
                | P::BodyDone
        ),
        E::Jidian => matches!(
            progress,
            P::Body
                | P::JidianAsk { .. }
                | P::AwaitJudge {
                    purpose: JudgePurpose::Jidian,
                    ..
                }
                | P::JidianAfterLog { .. }
                | P::BodyDone
        ),
        _ => false,
    };
    if matches!(progress, P::Body)
        && (context.jiding_card.is_some()
            || context.judge_receipt.is_some()
            || context.effect_applied
            || !context.jiding_changes.is_empty())
    {
        return Err(bad());
    }
    if effect != E::Jiding && !context.jiding_changes.is_empty() { return Err(bad()); }
    if effect == E::Jiding {
        match (context.jiding_card, context.jiding_changes.is_empty()) {
            // 有既定牌就必须验证（含「变更被剥光」的档：此时要求载荷仍是
            // 未转移的原始形态，不能靠删记录绕过证明）。
            (Some(child), _) => {
                super::jiding_card_changes::validate_jiding_card(core, catalog, res, frame, context, child)?
            }
            (None, false) => return Err(bad()),
            (None, true) => {}
        }
        if let P::JidingCard { child } = progress {
            if context.jiding_card != Some(*child) || context.effect_applied {
                return Err(bad());
            }
        }
        if matches!(progress, P::SourceLoss)
            && (!context.effect_applied
                || context.jiding_card.is_none()
                || context
                    .jiding_card
                    .is_some_and(|id| res.stack.iter().any(|f| f.id == id)))
        {
            return Err(bad());
        }
    }
    if matches!(effect, E::Yuance | E::Jidian) {
        if matches!(progress, P::BodyDone)
            && (!context.effect_applied || context.judge_receipt.is_none())
        {
            return Err(bad());
        }
        if let Some(receipt) = &context.judge_receipt {
            if receipt.child <= frame
                || receipt.child >= res.next_event_id
                || receipt.target as usize >= core.players.len()
            {
                return Err(bad());
            }
            match (effect, receipt.purpose) {
                (E::Yuance, JudgePurpose::Yuance(_)) if receipt.target == context.owner => {}
                (E::Jidian, JudgePurpose::Jidian) if receipt.target != context.target => {}
                _ => return Err(bad()),
            }
            if let P::AwaitJudge {
                child,
                target,
                purpose,
            } = progress
            {
                if (*child, *target, *purpose) != (receipt.child, receipt.target, receipt.purpose)
                    || context.effect_applied
                {
                    return Err(bad());
                }
            }
            if let P::JidianAfterLog { target } = progress {
                if *target != receipt.target || context.effect_applied {
                    return Err(bad());
                }
            }
            if context.effect_applied && res.stack.iter().any(|f| f.id == receipt.child) {
                return Err(bad());
            }
        } else if matches!(progress, P::AwaitJudge { .. } | P::JidianAfterLog { .. }) {
            return Err(bad());
        }
    }
    if !allowed {
        return Err(bad());
    }
    if let P::JidianAfterLog { target } = progress {
        if *target as usize >= core.players.len()
            || *target == context.target
            || context.judgement.is_none()
            || !context.deferred_log
        {
            return Err(bad());
        }
    }
    match progress {
        P::QinglongAsk {
            target,
            hand,
            equip,
        } => {
            if *target != context.target
                || *target as usize >= core.players.len()
                || *hand
                    != core
                        .player(*target)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| core.can_gain_card(catalog, context.owner, *target, *c))
                        .collect::<Vec<_>>()
                || *equip
                    != core
                        .player(*target)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| core.can_gain_card(catalog, context.owner, *target, *c))
                        .collect::<Vec<_>>()
            {
                return Err(bad());
            }
        }
        P::QinglongLog { target, card, .. } | P::QinglongLoss { target, card, .. } => {
            if *target != context.target
                || *card as usize >= catalog.cards.len()
                || !context.deferred_log
            {
                return Err(bad());
            }
        }
        _ => {}
    }
    if effect == E::Qinglong && matches!(progress, P::BodyDone) && !context.effect_applied {
        return Err(bad());
    }
    let committed_stage = matches!(
        progress,
        P::HolderLoss { .. } | P::HolderAfterLog { .. } | P::SourceAsk { .. } | P::SourceLoss
    );
    if effect == E::Zhangwu && !matches!(progress, P::Body | P::HolderAsk { .. }) {
        let cost = context.payment.as_ref().ok_or_else(bad)?;
        let unique: std::collections::BTreeSet<_> = cost.selected.iter().copied().collect();
        if cost.selected.is_empty()
            || unique.len() != cost.selected.len()
            || cost
                .selected
                .iter()
                .any(|c| *c as usize >= catalog.cards.len())
            || cost.committed != committed_stage
            || cost.log_begin <= frame
            || cost.log_begin >= res.next_event_id
        {
            return Err(bad());
        }
        if let P::HolderPay { cards } = progress {
            if *cards != cost.selected {
                return Err(bad());
            }
        }
        match progress {
            P::HolderLoss { count } | P::HolderAfterLog { count } | P::SourceAsk { count, .. }
                if *count as usize != cost.selected.len() =>
            {
                return Err(bad())
            }
            _ => {}
        }
        if let Some(log) = res.stack.iter().find(|f| f.id == cost.log_begin) {
            if log.parent != Some(frame)
                || !matches!(log.data,EventData::SkillLog{owner,def,begin:true}if owner==context.owner&&def==context.def)
            {
                return Err(bad());
            }
        }
        let loss_set: std::collections::BTreeSet<_> = cost.loss_children.iter().copied().collect();
        if loss_set.len() != cost.loss_children.len() {
            return Err(bad());
        }
        for id in &cost.loss_children {
            if *id <= cost.log_begin || *id >= res.next_event_id {
                return Err(bad());
            }
            if let Some(child) = res.stack.iter().find(|f| f.id == *id) {
                match &child.data {
                    EventData::CardsLost { player, cards, .. }
                    | EventData::CardsDiscarded { player, cards }
                        if *player == context.owner && cards.iter().all(|c| unique.contains(c)) => {
                    }
                    _ => return Err(bad()),
                }
            }
        }
        let after = matches!(
            progress,
            P::HolderAfterLog { .. } | P::SourceAsk { .. } | P::SourceLoss
        );
        if after != cost.after_log.is_some() {
            return Err(bad());
        }
        if let Some(log) = cost.after_log {
            if log <= cost.log_begin
                || log >= res.next_event_id
                || cost.loss_children.iter().any(|id| *id >= log)
            {
                return Err(bad());
            }
            if let Some(child) = res.stack.iter().find(|f| f.id == log) {
                if child.parent != Some(frame)
                    || !matches!(child.data,EventData::SkillLog{owner,def,begin:false}if owner==context.owner&&def==context.def)
                {
                    return Err(bad());
                }
            }
        }
    } else if context.payment.is_some() {
        return Err(bad());
    }
    if let P::AwaitJudge {
        target,
        purpose: JudgePurpose::Yuance(_),
        ..
    } = progress
    {
        if *target != context.owner {
            return Err(bad());
        }
    }
    if let P::AwaitJudge { child, target, .. } = progress {
        if *child <= frame || *child >= res.next_event_id {
            return Err(bad());
        }
        if let Some(judge) = res.stack.iter().find(|f| f.id == *child) {
            if judge.parent != Some(frame)
                || !matches!(judge.data,EventData::OrdinaryJudge{owner,invocation}if owner==*target&&invocation==frame)
            {
                return Err(bad());
            }
        }
    }
    if let P::JidianAsk { targets } = progress {
        let actual: Vec<_> = core
            .alive_players()
            .filter(|p| {
                *p != context.target && core.computed_distance(catalog, context.target, *p) <= 1
            })
            .collect();
        if *targets != actual {
            return Err(bad());
        }
    }
    Ok(())
}
