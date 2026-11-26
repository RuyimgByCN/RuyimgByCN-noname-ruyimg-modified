//! Rule Timeline Inspector (architecture §35).
//!
//! Records native `Engine` command submissions and formats a human-readable
//! sequence of accepted steps, Choice barriers, match completion, or rejections.

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};

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
    /// A command was rejected or suspended.
    Fault,
}

/// One recorded native command step and post-step state hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeReplayRecord {
    /// 1-based record index.
    pub record_sequence: u64,
    /// Engine revision after command.
    pub post_revision: u64,
    /// Canonical state hash after command.
    pub post_state_hash: [u8; 32],
    /// Step kind.
    pub kind: TimelineStepKind,
    /// Command summary.
    pub description: String,
}

/// Recorded native command trace used by timeline & bisector tools.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeReplayLog {
    /// Recorded steps in submission order.
    pub records: Vec<NativeReplayRecord>,
}

impl NativeReplayLog {
    /// Create an empty trace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit `cmd` to `engine` and append the resulting trace record.
    pub fn submit_and_record(&mut self, engine: &mut Engine, cmd: GameCommand) -> CommandOutcome {
        let description = describe_action(&cmd.action, cmd.actor);
        let outcome = engine.submit(cmd);
        let kind = match &outcome {
            CommandOutcome::Accepted { .. } => {
                if engine.outcome().is_some() {
                    TimelineStepKind::MatchCompleted
                } else {
                    TimelineStepKind::Command
                }
            }
            CommandOutcome::Pending { .. } => TimelineStepKind::ChoiceBarrier,
            CommandOutcome::Rejected(_) | CommandOutcome::Suspended => TimelineStepKind::Fault,
        };
        self.records.push(NativeReplayRecord {
            record_sequence: (self.records.len() + 1) as u64,
            post_revision: engine.revision(),
            post_state_hash: engine.state_hash(),
            kind,
            description,
        });
        outcome
    }
}

/// Build a human-readable timeline from a native replay log.
pub fn build_timeline(log: &NativeReplayLog) -> Vec<TimelineEntry> {
    log.records
        .iter()
        .map(|rec| TimelineEntry {
            seq: rec.record_sequence,
            revision: rec.post_revision,
            kind: rec.kind.clone(),
            description: rec.description.clone(),
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

fn describe_action(action: &GameAction, actor: u8) -> String {
    match action {
        GameAction::NextPhase => format!("NextPhase(actor={actor})"),
        GameAction::PlayCard { card, targets } => {
            format!("PlayCard(actor={actor}, card={card}, targets={targets:?})")
        }
        GameAction::ActivateSkill { skill } => {
            format!("ActivateSkill(actor={actor}, skill={skill})")
        }
        GameAction::RevealGeneral { slot } => {
            format!("RevealGeneral(actor={actor}, slot={slot})")
        }
        GameAction::Answer {
            choice_id,
            selection,
        } => {
            format!(
                "Answer(actor={actor}, choice={choice_id}, selection={})",
                selection.len()
            )
        }
        GameAction::TimeoutAction {
            choice_id,
            selection,
        } => {
            format!(
                "TimeoutAction(actor={actor}, choice={choice_id}, selection={})",
                selection.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_shows_command_sequence() {
        let mut eng = Engine::new(7, 2);
        let mut log = NativeReplayLog::new();
        let actor = eng.turn().current;
        let rev = eng.revision();
        log.submit_and_record(
            &mut eng,
            GameCommand {
                actor,
                expected_revision: rev,
                dedup_id: 1,
                action: GameAction::NextPhase,
            },
        );
        let actor = eng.turn().current;
        let rev = eng.revision();
        log.submit_and_record(
            &mut eng,
            GameCommand {
                actor,
                expected_revision: rev,
                dedup_id: 2,
                action: GameAction::NextPhase,
            },
        );
        let timeline = build_timeline(&log);
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].kind, TimelineStepKind::Command);
        assert!(timeline[0].description.contains("NextPhase"));
    }

    #[test]
    fn render_produces_table() {
        let mut eng = Engine::new(1, 2);
        let mut log = NativeReplayLog::new();
        let actor = eng.turn().current;
        let rev = eng.revision();
        log.submit_and_record(
            &mut eng,
            GameCommand {
                actor,
                expected_revision: rev,
                dedup_id: 1,
                action: GameAction::NextPhase,
            },
        );
        let timeline = build_timeline(&log);
        let rendered = render_timeline(&timeline);
        assert!(rendered.contains("seq"));
        assert!(rendered.contains("NextPhase"));
    }
}
