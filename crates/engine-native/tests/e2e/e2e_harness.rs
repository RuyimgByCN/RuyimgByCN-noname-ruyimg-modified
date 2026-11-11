//! E2E Test Harness: Opaque-Box Session Utilities & Assertion Framework.
//!
//! Provides decoupled, black-box wrappers over `Engine` for integration
//! and end-to-end testing across all 23 features in `PROJECT.md`.
//!
//! Complies with project line gates (<= 600 lines total).

use engine_native::archive;
use engine_native::choice::{ChoiceOption, OpenChoice};
use engine_native::engine::{CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
use engine_native::event::Phase;
use engine_native::state::Outcome;

/// Hexadecimal string encoder for binary archives and digest byte slices.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Strict hexadecimal string decoder returning raw byte buffer.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.is_ascii() {
        return Err("hex string contains non-ascii characters".into());
    }
    if s.len() % 2 != 0 {
        return Err(format!("hex string has odd length: {}", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| format!("invalid hex sequence at {i}: {e}"))?;
        out.push(byte);
    }
    Ok(out)
}

/// Opaque wrapper around `Engine` exposing only public, state-driven interfaces.
pub struct OpaqueGameSession {
    engine: Engine,
    dedup_counter: u64,
}

impl OpaqueGameSession {
    /// Start a new identity match with deterministic seed and 4-8 seats.
    pub fn new_identity_match(seed: u64, seats: usize) -> Result<Self, String> {
        let engine = Engine::try_new_identity(seed, seats)?;
        let last_dedup = engine.save().last_dedup;
        Ok(Self {
            engine,
            dedup_counter: last_dedup,
        })
    }

    /// Start a new standard prototype match with deterministic seed.
    pub fn new_standard_match(seed: u64, seats: usize) -> Self {
        let engine = Engine::new(seed, seats);
        let last_dedup = engine.save().last_dedup;
        Self {
            engine,
            dedup_counter: last_dedup,
        }
    }

    /// Wrap an existing engine instance.
    pub fn from_engine(engine: Engine) -> Self {
        let last_dedup = engine.save().last_dedup;
        Self {
            engine,
            dedup_counter: last_dedup,
        }
    }

    /// Read-only access to inner engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Mutable access to inner engine.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Current authoritative state revision.
    pub fn revision(&self) -> u64 {
        self.engine.revision()
    }

    /// Canonical BLAKE3 state hash digest.
    pub fn state_hash(&self) -> [u8; 32] {
        self.engine.state_hash()
    }

    /// Hex-encoded canonical state hash string.
    pub fn state_hash_hex(&self) -> String {
        hex_encode(&self.state_hash())
    }

    /// Check if match has concluded.
    pub fn is_finished(&self) -> bool {
        self.engine.outcome().is_some()
    }

    /// Final match outcome if finished.
    pub fn outcome(&self) -> Option<Outcome> {
        self.engine.outcome().cloned()
    }

    /// Currently active seat index.
    pub fn current_player(&self) -> u8 {
        self.engine.turn().current
    }

    /// Current active phase.
    pub fn current_phase(&self) -> Phase {
        self.engine.turn().phase
    }

    /// Number of seats in match.
    pub fn players_count(&self) -> usize {
        self.engine.state_core().players.len()
    }

    /// Current HP for given seat.
    pub fn player_hp(&self, seat: u8) -> Option<i32> {
        self.engine.state_core().players.get(seat as usize).map(|p| p.hp)
    }

    /// Max HP for given seat.
    pub fn player_max_hp(&self, seat: u8) -> Option<u32> {
        self.engine.state_core().players.get(seat as usize).map(|p| p.max_hp)
    }

    /// Current hand card count for given seat.
    pub fn player_hand_count(&self, seat: u8) -> Option<usize> {
        self.engine.state_core().players.get(seat as usize).map(|p| p.hand.len())
    }

    /// Open interactive choice if suspended.
    pub fn open_choice(&self) -> Option<&OpenChoice> {
        self.engine.open_choice()
    }

    /// Open choice JSON representation.
    pub fn open_choice_json(&self, seat: u8) -> String {
        match self.engine.open_choice() {
            Some(c) if c.observer == seat => serde_json::json!({
                "id": c.id,
                "observer": c.observer,
                "prompt": c.spec.prompt,
                "min": c.spec.min,
                "max": c.spec.max,
                "options_count": c.spec.options.len(),
            })
            .to_string(),
            Some(c) => serde_json::json!({
                "id": c.id,
                "observer": c.observer,
                "prompt": c.spec.prompt,
                "hidden": true,
            })
            .to_string(),
            None => "null".to_string(),
        }
    }

    /// Authoritative legal actions for seat.
    pub fn legal_actions(&self, seat: u8) -> Vec<GameAction> {
        self.engine.legal_actions(seat)
    }

    /// Legal actions JSON projection for seat.
    pub fn legal_actions_json(&self, seat: u8) -> String {
        let acts: Vec<serde_json::Value> = self
            .legal_actions(seat)
            .into_iter()
            .map(|a| match a {
                GameAction::PlayCard { card, targets } => serde_json::json!({
                    "type": "PlayCard",
                    "card": card,
                    "targets": targets,
                }),
                GameAction::NextPhase => serde_json::json!({ "type": "NextPhase" }),
                GameAction::ActivateSkill { skill } => serde_json::json!({
                    "type": "ActivateSkill",
                    "skill": skill,
                }),
                GameAction::Answer { choice_id, selection } => serde_json::json!({
                    "type": "Answer",
                    "choice_id": choice_id,
                    "selection_count": selection.len(),
                }),
                _ => serde_json::json!({ "type": "Other" }),
            })
            .collect();
        serde_json::json!({
            "seat": seat,
            "revision": self.revision(),
            "actions": acts,
        })
        .to_string()
    }

    /// Next monotonically increasing dedup_id.
    pub fn next_dedup_id(&mut self) -> u64 {
        self.dedup_counter += 1;
        self.dedup_counter
    }

    /// Submit a fully prepared GameCommand.
    pub fn submit(&mut self, cmd: GameCommand) -> CommandOutcome {
        if cmd.dedup_id > self.dedup_counter {
            self.dedup_counter = cmd.dedup_id;
        }
        self.engine.submit(cmd)
    }

    /// Submit an action for actor with automatic revision and dedup_id.
    pub fn submit_action(&mut self, actor: u8, action: GameAction) -> CommandOutcome {
        let dedup = self.next_dedup_id();
        let cmd = GameCommand {
            actor,
            expected_revision: self.revision(),
            dedup_id: dedup,
            action,
        };
        self.submit(cmd)
    }

    /// Active phase advancement.
    pub fn next_phase(&mut self, actor: u8) -> CommandOutcome {
        self.submit_action(actor, GameAction::NextPhase)
    }

    /// Card play by actor targeting specific players.
    pub fn play_card(&mut self, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
        self.submit_action(actor, GameAction::PlayCard { card, targets })
    }

    /// Skill activation by actor.
    pub fn activate_skill(&mut self, actor: u8, skill: u16) -> CommandOutcome {
        self.submit_action(actor, GameAction::ActivateSkill { skill })
    }

    /// Answer open choice with explicit option selections.
    pub fn answer_choice(
        &mut self,
        choice_id: u64,
        selection: Vec<ChoiceOption>,
    ) -> CommandOutcome {
        let observer = self
            .engine
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or_else(|| self.current_player());
        self.submit_action(observer, GameAction::Answer { choice_id, selection })
    }

    /// Answer open choice by selecting option indices from `spec.options`.
    pub fn answer_choice_indices(
        &mut self,
        choice_id: u64,
        indices: &[usize],
    ) -> Result<CommandOutcome, String> {
        let (observer, selected) = {
            let choice = self
                .engine
                .open_choice()
                .ok_or_else(|| "no open choice".to_string())?;
            if choice.id != choice_id {
                return Err(format!("choice_id mismatch: expected {}, got {choice_id}", choice.id));
            }
            let mut opts = Vec::new();
            for &idx in indices {
                let opt = choice
                    .spec
                    .options
                    .get(idx)
                    .ok_or_else(|| format!("option index out of bounds: {idx}"))?;
                opts.push(opt.clone());
            }
            (choice.observer, opts)
        };
        Ok(self.submit_action(observer, GameAction::Answer { choice_id, selection: selected }))
    }

    /// Automatically advance one deterministic game step.
    pub fn auto_step(&mut self) -> Result<CommandOutcome, String> {
        if self.is_finished() {
            return Ok(CommandOutcome::Accepted {
                state_hash: self.state_hash(),
            });
        }
        if let Some(choice) = self.open_choice().cloned() {
            let obs = choice.observer;
            let count = choice.spec.min as usize;
            let options: Vec<ChoiceOption> = choice.spec.options.into_iter().take(count).collect();
            let action = GameAction::Answer {
                choice_id: choice.id,
                selection: options,
            };
            return Ok(self.submit_action(obs, action));
        }
        let current = self.current_player();
        let actions = self.legal_actions(current);
        if let Some(card_action) = actions.iter().find(|a| matches!(a, GameAction::PlayCard { .. })) {
            return Ok(self.submit_action(current, card_action.clone()));
        }
        if actions.iter().any(|a| matches!(a, GameAction::NextPhase)) {
            return Ok(self.submit_action(current, GameAction::NextPhase));
        }
        Ok(self.submit_action(current, GameAction::NextPhase))
    }

    /// Run auto_step iteratively until finished or step quota reached.
    pub fn run_until_finished_or_limit(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        while !self.is_finished() && steps < max_steps {
            if self.auto_step().is_err() {
                break;
            }
            steps += 1;
        }
        steps
    }

    /// Serialize full state snapshot into hex-encoded NGSA binary archive.
    pub fn save_snapshot(&self) -> String {
        let snap = self.engine.save();
        let encoded = archive::encode_archive(&snap);
        hex_encode(&encoded)
    }

    /// Restore session from hex-encoded NGSA binary archive.
    pub fn restore_snapshot(hex: &str) -> Result<Self, String> {
        let bytes = hex_decode(hex)?;
        let snap = archive::decode_archive(&bytes)
            .map_err(|e| format!("decode_archive failed: {e:?}"))?;
        let engine = Engine::restore(&snap)
            .map_err(|e| format!("Engine::restore failed: {e:?}"))?;
        let last_dedup = snap.last_dedup;
        Ok(Self {
            engine,
            dedup_counter: last_dedup,
        })
    }

    /// Verify snapshot roundtrip produces 100% identical state hash and revision.
    pub fn verify_snapshot_roundtrip(&self) -> Result<(), String> {
        let hex = self.save_snapshot();
        let restored = Self::restore_snapshot(&hex)?;
        if restored.revision() != self.revision() {
            return Err(format!(
                "revision mismatch: original={}, restored={}",
                self.revision(),
                restored.revision()
            ));
        }
        if restored.state_hash() != self.state_hash() {
            return Err(format!(
                "state_hash mismatch: original={:?}, restored={:?}",
                self.state_hash_hex(),
                restored.state_hash_hex()
            ));
        }
        let orig_choice = self.open_choice().map(|c| c.id);
        let restored_choice = restored.open_choice().map(|c| c.id);
        if orig_choice != restored_choice {
            return Err(format!(
                "open_choice mismatch: original={:?}, restored={:?}",
                orig_choice, restored_choice
            ));
        }
        Ok(())
    }

    /// Assert that command is rejected AND state hash / revision remain 100% unchanged.
    pub fn assert_command_rejected(
        &mut self,
        cmd: GameCommand,
        expected_reason: Option<RejectReason>,
    ) {
        let rev_before = self.revision();
        let hash_before = self.state_hash();
        let choice_before = self.open_choice().cloned();
        let outcome = self.submit(cmd);
        match &outcome {
            CommandOutcome::Rejected(reason) => {
                if let Some(expected) = &expected_reason {
                    assert_eq!(
                        reason, expected,
                        "RejectReason mismatch: expected {expected:?}, got {reason:?}"
                    );
                }
            }
            other => {
                panic!("Expected command to be Rejected, but got outcome: {other:?}");
            }
        }
        assert_eq!(
            self.revision(),
            rev_before,
            "Zero-mutation invariant violated: revision changed on rejected command!"
        );
        assert_eq!(
            self.state_hash(),
            hash_before,
            "Zero-mutation invariant violated: state_hash changed on rejected command!"
        );
        assert_eq!(
            self.open_choice(),
            choice_before.as_ref(),
            "Zero-mutation invariant violated: open_choice changed on rejected command!"
        );
    }

    /// Assert that action for actor is rejected with zero state mutation.
    pub fn assert_action_rejected(
        &mut self,
        actor: u8,
        action: GameAction,
        expected_reason: Option<RejectReason>,
    ) {
        let dedup = self.next_dedup_id();
        let cmd = GameCommand {
            actor,
            expected_revision: self.revision(),
            dedup_id: dedup,
            action,
        };
        self.assert_command_rejected(cmd, expected_reason);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_harness_match_startup() {
        let session = OpaqueGameSession::new_identity_match(42, 5).expect("5-player match");
        assert_eq!(session.players_count(), 5);
        assert!(!session.is_finished());
        assert_eq!(session.revision(), 0);
        assert_eq!(session.current_player(), 0);
        // Lord starts with 5 HP in 5-player identity mode
        assert_eq!(session.player_hp(0), Some(5));
        assert_eq!(session.player_max_hp(0), Some(5));
        // Loyalists/Rebels start with 4 HP
        for seat in 1..5 {
            assert_eq!(session.player_hp(seat), Some(4));
            assert_eq!(session.player_max_hp(seat), Some(4));
        }
    }

    #[test]
    fn test_harness_step_resolution() {
        let mut session = OpaqueGameSession::new_identity_match(1042, 5).expect("5-player match");
        let initial_rev = session.revision();
        let initial_hash = session.state_hash();
        let outcome = session.auto_step().expect("step succeeds");
        assert!(
            matches!(outcome, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "auto_step produced invalid outcome: {outcome:?}"
        );
        assert!(session.revision() > initial_rev, "Revision must increment after valid step");
        assert_ne!(session.state_hash(), initial_hash, "State hash must evolve after valid step");
    }

    #[test]
    fn test_harness_choice_answering() {
        let mut session = OpaqueGameSession::new_identity_match(77, 4).expect("4-player match");
        for _ in 0..10 {
            if session.open_choice().is_some() {
                let choice = session.open_choice().unwrap().clone();
                let outcome = session
                    .answer_choice_indices(choice.id, &[0])
                    .expect("answer choice index 0");
                assert!(
                    matches!(outcome, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                    "Answering choice failed: {outcome:?}"
                );
                break;
            }
            let _ = session.auto_step();
        }
    }

    #[test]
    fn test_harness_snapshot_roundtrip() {
        let mut session = OpaqueGameSession::new_identity_match(999, 5).expect("5-player match");
        // Step forward a few phases
        for _ in 0..5 {
            let _ = session.auto_step();
        }
        let roundtrip_res = session.verify_snapshot_roundtrip();
        assert!(roundtrip_res.is_ok(), "Roundtrip verification failed: {:?}", roundtrip_res);

        let hex = session.save_snapshot();
        assert!(hex.len() > 100, "Archive hex should be non-trivial");
        let mut restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore succeeds");
        assert_eq!(restored.revision(), session.revision());
        assert_eq!(restored.state_hash(), session.state_hash());
        assert_eq!(restored.state_hash_hex(), session.state_hash_hex());

        // Verify restored session continues execution cleanly
        let next_outcome = restored.auto_step().expect("step on restored session");
        assert!(matches!(next_outcome, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    }

    #[test]
    fn test_harness_rejection_zero_mutation() {
        let mut session = OpaqueGameSession::new_identity_match(12345, 5).expect("5-player match");
        let valid_rev = session.revision();
        let valid_dedup = session.next_dedup_id();

        // 1. Stale revision rejection
        let stale_cmd = GameCommand {
            actor: 0,
            expected_revision: valid_rev + 999,
            dedup_id: valid_dedup,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(stale_cmd, Some(RejectReason::StaleRevision));

        // 2. Duplicate dedup_id rejection
        let dup_cmd = GameCommand {
            actor: 0,
            expected_revision: valid_rev,
            dedup_id: 0,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(dup_cmd, Some(RejectReason::DuplicateCommand));

        // 3. Unknown actor rejection (actor 99 out of 5 seats)
        let unknown_actor_cmd = GameCommand {
            actor: 99,
            expected_revision: valid_rev,
            dedup_id: valid_dedup + 1,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(unknown_actor_cmd, Some(RejectReason::UnknownActor));

        // 4. Not your turn rejection (actor 2 when seat 0 is active)
        let not_your_turn_cmd = GameCommand {
            actor: 2,
            expected_revision: valid_rev,
            dedup_id: valid_dedup + 2,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(not_your_turn_cmd, Some(RejectReason::NotYourTurn));
    }
}
