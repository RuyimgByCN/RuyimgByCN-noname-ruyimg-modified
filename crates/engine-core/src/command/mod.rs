//! Command pipeline.

pub(crate) mod discard_damage_options;
pub(crate) mod judge;
pub(crate) mod play;
pub(crate) mod resolve_choice;
pub(crate) mod reveal_options;
pub(crate) mod skill;
pub(crate) mod turn;

use crate::effect::{bootstrap_match, run_effects, EffectError};
use crate::hash::canonical_state_hash;
use crate::invariants::check_invariants;
use crate::state::GameState;
use crate::transaction::Transaction;
use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, FaultKind, MatchOutcome, RejectReason,
};
use rule_api::EffectRequest;

use play::handle_play_card;
use resolve_choice::handle_resolve_choice;
use skill::{handle_activate_skill, handle_use_skill_card};
use turn::handle_end_phase;

/// Full pipeline result including events for hosts/tests.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Public command result.
    pub result: CommandResult,
    /// Domain events if accepted.
    pub events: Vec<DomainEvent>,
    /// Choice if paused.
    pub choice: Option<engine_api::ChoiceRequest>,
}

/// Apply a normalized command to state.
pub fn apply_command(state: GameState, cmd: Command) -> (GameState, PipelineResult) {
    if state.outcome.is_finished() && !matches!(cmd.kind, CommandKind::AdminAbortMatch { .. }) {
        return (
            state,
            PipelineResult {
                result: CommandResult::Rejected {
                    reason: RejectReason::MatchOver,
                },
                events: vec![],
                choice: None,
            },
        );
    }

    if cmd.expected_revision.0 != 0 && cmd.expected_revision.0 != state.revision {
        let current = state.revision;
        return (
            state,
            PipelineResult {
                result: CommandResult::Rejected {
                    reason: RejectReason::StaleRevision {
                        current,
                        expected: cmd.expected_revision.0,
                    },
                },
                events: vec![],
                choice: None,
            },
        );
    }

    // While a choice is open, only ResolveChoice / TimeoutAction / AdminAbort / Surrender(dying?) allowed
    if state.has_pending_choice() {
        let allowed = matches!(
            cmd.kind,
            CommandKind::ResolveChoice { .. }
                | CommandKind::TimeoutAction { .. }
                | CommandKind::AdminAbortMatch { .. }
        );
        if !allowed {
            return (
                state,
                PipelineResult {
                    result: CommandResult::Rejected {
                        reason: RejectReason::IllegalAction {
                            code: "choice pending".into(),
                        },
                    },
                    events: vec![],
                    choice: None,
                },
            );
        }
    }

    let mut tx = Transaction::begin(state);

    let run = match &cmd.kind {
        CommandKind::StartGame {
            player_count,
            rng_seed,
            ruleset,
        } => {
            if tx.state().revision != 0 || tx.state().players.live_count() > 0 {
                Err(EffectError::Illegal("already started".into()))
            } else {
                bootstrap_match(&mut tx, *player_count, *rng_seed, ruleset)
            }
        }
        CommandKind::PlayCard {
            card,
            targets,
            card_targets,
        } => handle_play_card(&mut tx, cmd.actor, *card, targets, card_targets),
        CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets,
        } => handle_activate_skill(&mut tx, cmd.actor, *skill, targets, card_targets),
        CommandKind::UseSkillCard {
            skill,
            card_kind,
            targets,
            equip_slot,
        } => handle_use_skill_card(&mut tx, cmd.actor, *skill, *card_kind, targets, *equip_slot),
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        } => handle_resolve_choice(&mut tx, cmd.actor, *choice_id, selections, *cancelled),
        CommandKind::EndPhase => handle_end_phase(&mut tx, cmd.actor),
        CommandKind::RevealGeneral { slot } => handle_reveal_general(&mut tx, cmd.actor, *slot),
        CommandKind::Surrender => {
            if cmd.actor.is_none() || tx.state().players.get(cmd.actor).is_none() {
                Err(EffectError::Illegal("bad actor".into()))
            } else {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::KillPlayer {
                        player: cmd.actor,
                        killer: engine_api::PlayerId::NONE,
                    }],
                )
                .map(|_| ())
            }
        }
        CommandKind::TimeoutAction { choice_id } => {
            if let Some(cid) = choice_id {
                let observer = engine_api::PlayerId::new(tx.state().resolution.pending_observer);
                // Auto-resolve: cancel if allowed, else die / default
                handle_resolve_choice(&mut tx, observer, *cid, &[], true)
            } else if tx.state().has_pending_choice() {
                Err(EffectError::Illegal("timeout requires choice_id".into()))
            } else {
                handle_end_phase(&mut tx, cmd.actor)
            }
        }
        CommandKind::AdminAbortMatch { reason_code } => {
            tx.state_mut().outcome = MatchOutcome::Aborted {
                reason_code: *reason_code,
            };
            tx.push_event(DomainEvent::MatchEnded {
                outcome: MatchOutcome::Aborted {
                    reason_code: *reason_code,
                },
            });
            Ok(())
        }
    };

    finish_segment(tx, run)
}

/// Handle `CommandKind::RevealGeneral` — guozhan only. Flips a face-down general face-up
/// and re-resolves faction (incl. 野心家 judgement). Identity rulesets reject the command.
/// When the reveal flips the player to 野心家 and they have no ally yet, opens a 拉拢人心
/// choice to pick an ally target.
fn handle_reveal_general(
    tx: &mut Transaction,
    actor: engine_api::PlayerId,
    slot: u8,
) -> Result<(), EffectError> {
    use crate::state::ContinuationKind;
    use engine_api::{ChoiceKind, PlayerId};
    if actor.is_none() || tx.state().players.get(actor).is_none() {
        return Err(EffectError::Illegal("reveal: bad actor".into()));
    }
    if !tx.state().ruleset.name.starts_with("guozhan") {
        return Err(EffectError::Illegal(
            "RevealGeneral is only valid in guozhan mode".into(),
        ));
    }
    crate::mode::reveal_general(tx.state_mut(), actor, slot).map_err(EffectError::Illegal)?;

    // If the reveal made this player an 野心家 and they have no ally, offer 拉拢人心.
    let is_ambition = tx
        .state()
        .player_group(actor)
        .is_some_and(|g| g == crate::mode::AMBITION);
    let has_ally = tx
        .state()
        .players
        .iter_live()
        .any(|p| p.id != actor && crate::mode::are_allies(tx.state(), p.id, actor));
    if is_ambition && !has_ally && !tx.state().has_pending_choice() {
        let targets: Vec<u32> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|p| *p != actor)
            .map(PlayerId::raw)
            .collect();
        if !targets.is_empty() {
            crate::effect::open_choice(
                tx,
                actor,
                0,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![crate::mode::LOUREN_XIN_CHOICE, actor.raw()],
            )?;
        }
    }
    Ok(())
}

fn finish_segment(tx: Transaction, run: Result<(), EffectError>) -> (GameState, PipelineResult) {
    match run {
        Ok(()) => commit_ok(tx),
        Err(EffectError::ChoiceRequested) => commit_ok(tx),
        Err(EffectError::Illegal(msg)) => {
            let hash = tx.baseline_hash();
            let state = tx.rollback();
            let reason = classify_illegal(&msg);
            debug_assert_eq!(hash, canonical_state_hash(&state));
            (
                state,
                PipelineResult {
                    result: CommandResult::Rejected { reason },
                    events: vec![],
                    choice: None,
                },
            )
        }
        Err(e) => {
            // ContentFault / BudgetExceeded / EngineFault → roll back segment
            let state = tx.rollback();
            (
                state,
                PipelineResult {
                    result: CommandResult::Rejected {
                        reason: RejectReason::Fault {
                            kind: e.fault_kind(),
                            message: e.to_string(),
                        },
                    },
                    events: vec![],
                    choice: None,
                },
            )
        }
    }
}

fn commit_ok(tx: Transaction) -> (GameState, PipelineResult) {
    if let Err(inv) = check_invariants(tx.state()) {
        let state = tx.rollback();
        return (
            state,
            PipelineResult {
                result: CommandResult::Rejected {
                    reason: RejectReason::Fault {
                        kind: FaultKind::EngineFault,
                        message: inv.to_string(),
                    },
                },
                events: vec![],
                choice: None,
            },
        );
    }
    let committed = tx.commit();
    let new_revision = committed.state.revision;
    let choice = committed.output.choice;
    (
        committed.state,
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision,
                state_hash: committed.state_hash,
            },
            events: committed.output.events,
            choice,
        },
    )
}

fn classify_illegal(msg: &str) -> RejectReason {
    if msg.contains("invalid choice")
        || msg.contains("stale choice")
        || msg.contains("choice")
            && (msg.contains("mismatch") || msg.contains("expired") || msg.contains("wrong"))
    {
        RejectReason::InvalidChoice
    } else if msg.contains("not started") {
        RejectReason::NotStarted
    } else {
        RejectReason::IllegalAction {
            code: msg.to_string(),
        }
    }
}
