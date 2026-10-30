//! Server-private, exhaustive command codec. Recovery never reruns a policy.
use super::*;
use engine_native::ChoiceOption;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeWalState {
    pub base_snapshot_hex: String,
    pub entries: Vec<NativeWalEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeWalEntry {
    pub actor: u8,
    pub expected_revision: u64,
    pub dedup_id: u64,
    action: StoredNativeAction,
    pub revision: u64,
    pub state_hash: [u8; 32],
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
enum StoredNativeAction {
    PlayCard {
        card: u32,
        targets: Vec<u8>,
    },
    NextPhase,
    ActivateSkill {
        skill: u16,
    },
    RevealGeneral {
        slot: u8,
    },
    Answer {
        choice_id: u64,
        selection: Vec<StoredChoiceOption>,
    },
    TimeoutAction {
        choice_id: u64,
        selection: Vec<StoredChoiceOption>,
    },
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum StoredChoiceOption {
    Confirm,
    Pass,
    Card(u32),
    Player(u8),
    Tag(u8),
    Kind(u16),
}

impl From<ChoiceOption> for StoredChoiceOption {
    fn from(value: ChoiceOption) -> Self {
        match value {
            ChoiceOption::Confirm => Self::Confirm,
            ChoiceOption::Pass => Self::Pass,
            ChoiceOption::Card(v) => Self::Card(v),
            ChoiceOption::Player(v) => Self::Player(v),
            ChoiceOption::Tag(v) => Self::Tag(v),
            ChoiceOption::Kind(v) => Self::Kind(v),
        }
    }
}
impl From<StoredChoiceOption> for ChoiceOption {
    fn from(value: StoredChoiceOption) -> Self {
        match value {
            StoredChoiceOption::Confirm => Self::Confirm,
            StoredChoiceOption::Pass => Self::Pass,
            StoredChoiceOption::Card(v) => Self::Card(v),
            StoredChoiceOption::Player(v) => Self::Player(v),
            StoredChoiceOption::Tag(v) => Self::Tag(v),
            StoredChoiceOption::Kind(v) => Self::Kind(v),
        }
    }
}
impl From<GameAction> for StoredNativeAction {
    fn from(value: GameAction) -> Self {
        match value {
            GameAction::PlayCard { card, targets } => Self::PlayCard { card, targets },
            GameAction::NextPhase => Self::NextPhase,
            GameAction::ActivateSkill { skill } => Self::ActivateSkill { skill },
            GameAction::RevealGeneral { slot } => Self::RevealGeneral { slot },
            GameAction::Answer {
                choice_id,
                selection,
            } => Self::Answer {
                choice_id,
                selection: selection.into_iter().map(Into::into).collect(),
            },
            GameAction::TimeoutAction {
                choice_id,
                selection,
            } => Self::TimeoutAction {
                choice_id,
                selection: selection.into_iter().map(Into::into).collect(),
            },
        }
    }
}
impl From<StoredNativeAction> for GameAction {
    fn from(value: StoredNativeAction) -> Self {
        match value {
            StoredNativeAction::PlayCard { card, targets } => Self::PlayCard { card, targets },
            StoredNativeAction::NextPhase => Self::NextPhase,
            StoredNativeAction::ActivateSkill { skill } => Self::ActivateSkill { skill },
            StoredNativeAction::RevealGeneral { slot } => Self::RevealGeneral { slot },
            StoredNativeAction::Answer {
                choice_id,
                selection,
            } => Self::Answer {
                choice_id,
                selection: selection.into_iter().map(Into::into).collect(),
            },
            StoredNativeAction::TimeoutAction {
                choice_id,
                selection,
            } => Self::TimeoutAction {
                choice_id,
                selection: selection.into_iter().map(Into::into).collect(),
            },
        }
    }
}

impl NativeWalEntry {
    pub(super) fn new(command: GameCommand, engine: &Engine) -> Self {
        Self {
            actor: command.actor,
            expected_revision: command.expected_revision,
            dedup_id: command.dedup_id,
            action: command.action.into(),
            revision: engine.revision(),
            state_hash: engine.state_hash(),
        }
    }
    pub fn command(&self) -> GameCommand {
        GameCommand {
            actor: self.actor,
            expected_revision: self.expected_revision,
            dedup_id: self.dedup_id,
            action: self.action.clone().into(),
        }
    }
}

impl NativeWalState {
    pub(super) fn new(engine: &Engine) -> Self {
        Self {
            base_snapshot_hex: hex_encode(&engine_native::archive::encode_archive(&engine.save())),
            entries: vec![],
        }
    }
    pub(super) fn recover(
        &self,
        packs: &[&engine_native::content::extension::ExtensionPack],
    ) -> Result<Engine, String> {
        let bytes = hex_decode(&self.base_snapshot_hex).ok_or("WAL base hex invalid")?;
        let snapshot =
            engine_native::archive::decode_archive(&bytes).map_err(|e| format!("{e:?}"))?;
        let mut engine = if packs.is_empty() {
            Engine::restore(&snapshot)
        } else {
            Engine::restore_with_extensions(&snapshot, packs)
        }
        .map_err(|e| format!("{e:?}"))?;
        for entry in &self.entries {
            let command = entry.command();
            if let GameAction::Answer { selection, .. }
            | GameAction::TimeoutAction { selection, .. } = &command.action
            {
                if selection.len() > 4096 || selection.iter().any(|o| matches!(o, ChoiceOption::Kind(k) if engine_native::archive::kind_from_code(*k).is_err())) {
                    return Err("invalid WAL selection".into());
                }
            }
            // All values, including Kind, must be present in the restored authorized choice.
            // submit validates candidates, count, duplicates, observer, revision and dedup.
            if matches!(engine.submit(command), CommandOutcome::Rejected(_))
                || engine.revision() != entry.revision
                || engine.state_hash() != entry.state_hash
            {
                return Err("WAL command/revision/摘要不一致".into());
            }
        }
        Ok(engine)
    }
}

impl NativeRoomSession {
    pub(crate) fn compact_wal(&mut self, every: u64) {
        if every > 0 && self.engine.revision() % every == 0 {
            self.wal = NativeWalState::new(&self.engine);
        }
    }
    pub(super) fn validate_wal(&self) -> Result<engine_native::snap::StateSnapshot, String> {
        let engine = self.wal.recover(&self.pack_refs())?;
        let mut replayed = engine.save();
        // restore rotates authentication tokens; generation is session security, not rule state.
        replayed.auth_generation = self.engine.save().auth_generation;
        if engine_native::archive::encode_archive(&replayed)
            != engine_native::archive::encode_archive(&self.engine.save())
        {
            return Err("WAL final full snapshot mismatch".into());
        }
        Ok(replayed)
    }
    pub(crate) fn recover_wal(&mut self) -> Result<(), String> {
        let snapshot = self.validate_wal()?;
        let packs = self.pack_refs();
        let engine = if packs.is_empty() { Engine::restore(&snapshot) }
            else { Engine::restore_with_extensions(&snapshot, &packs) }.map_err(|e| format!("{e:?}"))?;
        self.replace_engine(engine);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_actions_and_all_six_choice_values_round_trip_without_debug_encoding() {
        let selection = vec![ChoiceOption::Confirm, ChoiceOption::Pass, ChoiceOption::Card(42),
            ChoiceOption::Player(3), ChoiceOption::Tag(4), ChoiceOption::Kind(0)];
        for action in [GameAction::PlayCard { card: 42, targets: vec![1, 3] }, GameAction::NextPhase,
            GameAction::ActivateSkill { skill: 65535 }, GameAction::RevealGeneral { slot: 1 },
            GameAction::Answer { choice_id: u64::MAX, selection: selection.clone() },
            GameAction::TimeoutAction { choice_id: u64::MAX, selection }] {
            let encoded = serde_json::to_vec(&StoredNativeAction::from(action.clone())).unwrap();
            let decoded: StoredNativeAction = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(GameAction::from(decoded), action);
        }
        assert!(serde_json::from_str::<StoredNativeAction>(r#"{"TimeoutAction":{"choice_id":1,"selection":[{"Skill":1}]}}"#).is_err());
        assert!(serde_json::from_str::<StoredNativeAction>(r#"{"TimeoutAction":{"choice_id":1,"selection":[],"managed":true}}"#).is_err());
    }
}
