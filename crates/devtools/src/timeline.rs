//! Rule Timeline Inspector (architecture §35).
//!
//! Visualizes the sequence of trigger windows, Effects, and Choice barriers
//! from a replay log. Produces a human-readable timeline that helps developers
//! understand what happened during a match.

use engine_api::{CommandKind, PlayerId};
use engine_core::ReplayLog;

/// One entry in the rule timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineEntry {
    /// Sequence number (1-based).
    pub seq: u64,
    /// Revision after this step.
    pub revision: u64,
    /// What kind of step this is.
    pub kind: TimelineStepKind,
    /// Human-readable description.
    pub description: String,
}

/// Kind of timeline step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineStepKind {
    /// A command was submitted and accepted.
    Command,
    /// A choice barrier was opened.
    ChoiceBarrier,
    /// The match completed.
    MatchCompleted,
    /// A fault occurred.
    Fault,
}

/// Build a human-readable timeline from a replay log.
pub fn build_timeline(log: &ReplayLog) -> Vec<TimelineEntry> {
    log.records
        .iter()
        .map(|rec| {
            let kind = match rec.outcome {
                engine_core::ResolutionOutcome::Committed => TimelineStepKind::Command,
                engine_core::ResolutionOutcome::ChoiceBarrierCommitted => {
                    TimelineStepKind::ChoiceBarrier
                }
                engine_core::ResolutionOutcome::MatchCompleted => TimelineStepKind::MatchCompleted,
                engine_core::ResolutionOutcome::TerminalContentFault
                | engine_core::ResolutionOutcome::TerminalEngineFault
                | engine_core::ResolutionOutcome::AdminAborted => TimelineStepKind::Fault,
            };
            TimelineEntry {
                seq: rec.record_sequence,
                revision: rec.post_revision,
                kind,
                description: describe_command(&rec.command.kind, rec.command.actor),
            }
        })
        .collect()
}

/// Render a timeline as a text table for CLI output.
pub fn render_timeline(entries: &[TimelineEntry]) -> String {
    let mut out = String::new();
    out.push_str("seq  rev  type             description\n");
    out.push_str("---- ---- ---------------- -----------\n");
    for e in entries {
        let type_str = match e.kind {
            TimelineStepKind::Command => "Command",
            TimelineStepKind::ChoiceBarrier => "ChoiceBarrier",
            TimelineStepKind::MatchCompleted => "MatchCompleted",
            TimelineStepKind::Fault => "Fault",
        };
        out.push_str(&format!(
            "{:>4} {:>4} {:<16} {}\n",
            e.seq, e.revision, type_str, e.description
        ));
    }
    out
}

fn describe_command(kind: &CommandKind, actor: PlayerId) -> String {
    match kind {
        CommandKind::StartGame {
            player_count,
            rng_seed,
            ruleset,
        } => {
            format!("StartGame(players={player_count}, seed={rng_seed}, ruleset={ruleset})")
        }
        CommandKind::PlayCard {
            card,
            targets,
            card_targets,
        } => {
            let targets_str: Vec<String> = targets.iter().map(|t| t.raw().to_string()).collect();
            format!(
                "PlayCard(card={}, targets=[{}], card_targets={})",
                card.raw(),
                targets_str.join(","),
                card_targets.len()
            )
        }
        CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets,
        } => format!(
            "ActivateSkill(skill={}, targets={}, card_targets={})",
            skill.raw(),
            targets.len(),
            card_targets.len()
        ),
        CommandKind::UseSkillCard {
            skill,
            card_kind,
            targets,
            equip_slot,
        } => format!(
            "UseSkillCard(skill={}, kind={}, targets={}, equip_slot={})",
            skill.raw(),
            card_kind,
            targets.len(),
            equip_slot
        ),
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        } => format!(
            "ResolveChoice(choice={}, selections={}, cancelled={})",
            choice_id.raw(),
            selections.len(),
            cancelled
        ),
        CommandKind::EndPhase => format!("EndPhase(actor={})", actor.raw()),
        CommandKind::RevealGeneral { slot } => {
            format!("RevealGeneral(actor={}, slot={})", actor.raw(), slot)
        }
        CommandKind::Surrender => format!("Surrender(actor={})", actor.raw()),
        CommandKind::TimeoutAction { choice_id } => {
            format!("TimeoutAction(choice={:?})", choice_id)
        }
        CommandKind::AdminAbortMatch { reason_code } => {
            format!("AdminAbortMatch(reason={reason_code})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{Command, ExpectedRevision, IdempotencyKey};
    use engine_core::Engine;

    #[test]
    fn timeline_shows_command_sequence() {
        let mut eng = Engine::new();
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 7,
                ruleset: "identity-standard".into(),
            },
        });
        eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let timeline = build_timeline(eng.replay_log());
        assert!(timeline.len() >= 2);
        assert_eq!(timeline[0].kind, TimelineStepKind::Command);
        assert!(timeline[0].description.contains("StartGame"));
        assert!(timeline[1].description.contains("EndPhase"));
    }

    #[test]
    fn render_produces_table() {
        let mut eng = Engine::new();
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 1,
                ruleset: "identity-standard".into(),
            },
        });
        let timeline = build_timeline(eng.replay_log());
        let rendered = render_timeline(&timeline);
        assert!(rendered.contains("seq"));
        assert!(rendered.contains("StartGame"));
    }
}
