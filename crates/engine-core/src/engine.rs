//! High-level engine host handle.

use crate::command::{apply_command, PipelineResult};
use crate::content::{ContentLoadError, ContentRegistry};
use crate::hash::canonical_state_hash;
use crate::replay::{ReplayFileError, ReplayLog, ResolutionOutcome, RuleRngCheckpoint};
use crate::snapshot::Snapshot;
use crate::state::GameState;
use content_runtime::{ContentLock, PackageStore, RuntimeError};
use content_schema::ContentPackage;
use engine_api::{
    ArtifactBuildId, BudgetProfile, Command, CommandKind, CommandResult, ContentLockHash,
    EngineSemanticsId, InitialPlayer, ManifestRejectionReason, MatchConfig, MatchManifest,
    MatchManifestHash, ProtocolProfile, ResolvedRulesetRef, RuleIrVersion, RuleRngSpec, SeatIndex,
    VmAbiVersion,
};
use std::collections::{BTreeMap, BTreeSet};

/// In-process deterministic engine.
#[derive(Debug, Clone)]
pub struct Engine {
    state: GameState,
    log: ReplayLog,
    last_events: Vec<engine_api::DomainEvent>,
    last_choice: Option<engine_api::ChoiceRequest>,
    /// Match manifest pinned at StartGame (architecture §5). `None` until start.
    manifest: Option<MatchManifest>,
    /// Exact external package lock loaded before the match.
    content_lock_hash: Option<[u8; 32]>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Create engine with empty pre-start state.
    pub fn new() -> Self {
        let state = GameState::empty();
        let hash = canonical_state_hash(&state);
        Self {
            state,
            log: ReplayLog::new(hash),
            last_events: Vec::new(),
            last_choice: None,
            manifest: None,
            content_lock_hash: None,
        }
    }

    /// Current state (read-only).
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Canonical hash.
    pub fn state_hash(&self) -> [u8; 32] {
        canonical_state_hash(&self.state)
    }

    /// Accepted command log (truth source).
    pub fn replay_log(&self) -> &ReplayLog {
        &self.log
    }

    /// Pinned match manifest, if the match has started (architecture §5).
    pub fn manifest(&self) -> Option<&MatchManifest> {
        self.manifest.as_ref()
    }

    /// Pinned manifest hash (zero if not started).
    pub fn manifest_hash(&self) -> MatchManifestHash {
        self.manifest
            .as_ref()
            .map(|m| m.manifest_hash())
            .unwrap_or_default()
    }

    /// Events from last command.
    pub fn last_events(&self) -> &[engine_api::DomainEvent] {
        &self.last_events
    }

    /// Pending choice from last command, if any.
    pub fn last_choice(&self) -> Option<&engine_api::ChoiceRequest> {
        self.last_choice.as_ref()
    }

    /// Submit a command through the pipeline.
    pub fn submit(&mut self, cmd: Command) -> CommandResult {
        let pre_revision = self.state.revision;
        let pre_state_hash = self.state_hash();
        let rng_before = self.rng_checkpoint();
        let previous_choice = self.last_choice.clone();
        // 双克隆路径处理（ADR 0703 第②部分）：接管所有权而非克隆——
        // 每命令的克隆次数 2 → 1（Transaction::begin 内的工作副本，
        // 回滚语义所需）。Arc 化内容目录使该克隆不含目录字节。
        let prior_state = std::mem::replace(&mut self.state, GameState::empty());
        let (new_state, pipe) = apply_command(prior_state, cmd.clone());
        self.last_events = pipe.events.clone();
        self.last_choice = if matches!(pipe.result, CommandResult::Rejected { .. }) {
            previous_choice
        } else {
            pipe.choice.clone()
        };
        if let CommandResult::Accepted {
            new_revision,
            state_hash,
        } = &pipe.result
        {
            // Pin the match manifest on StartGame (frozen thereafter).
            if matches!(cmd.kind, CommandKind::StartGame { .. }) && self.manifest.is_none() {
                self.pin_manifest(&cmd, &new_state);
            }
            let rng_after = self.extract_rng_checkpoint(&new_state);
            let outcome = self.classify_outcome(&new_state, &pipe);
            let match_outcome = if new_state.outcome.is_finished() {
                Some(new_state.outcome.clone())
            } else {
                None
            };
            self.log.push_record(
                cmd.clone(),
                *new_revision,
                pre_revision,
                *state_hash,
                pre_state_hash,
                rng_before,
                rng_after,
                outcome,
                match_outcome,
            );
        }
        // 接受 → 新状态；拒绝 → Transaction 回滚返回的基线（与原状态等价）。
        // 统一写回：每命令克隆次数 2 → 1（begin 内的工作副本）。
        self.state = new_state;
        pipe.result
    }

    /// 测试专用：可变状态访问（缓存边界等单元测试使用；发布路径不可达）。
    #[cfg(test)]
    pub(crate) fn state_mut_for_tests(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Snapshot the live rule RNG as a checkpoint.
    fn rng_checkpoint(&self) -> RuleRngCheckpoint {
        RuleRngCheckpoint {
            state: self.state.rng.state,
            call_count: self.state.rng.call_count,
        }
    }

    /// Extract an RNG checkpoint from a (possibly new) state.
    fn extract_rng_checkpoint(&self, state: &GameState) -> RuleRngCheckpoint {
        RuleRngCheckpoint {
            state: state.rng.state,
            call_count: state.rng.call_count,
        }
    }

    /// Classify a segment outcome from post-commit state + pipeline result.
    fn classify_outcome(&self, new_state: &GameState, pipe: &PipelineResult) -> ResolutionOutcome {
        if new_state.outcome.is_finished() {
            match &new_state.outcome {
                engine_api::MatchOutcome::Aborted { .. } => ResolutionOutcome::AdminAborted,
                _ => ResolutionOutcome::MatchCompleted,
            }
        } else if pipe.choice.is_some() || new_state.has_pending_choice() {
            ResolutionOutcome::ChoiceBarrierCommitted
        } else {
            ResolutionOutcome::Committed
        }
    }

    /// Build and pin a [`MatchManifest`] from the StartGame command + bootstrapped state.
    ///
    /// The manifest is frozen for the rest of the match (architecture §5, §41).
    /// Subsequent StartGame commands are rejected by the command pipeline (which
    /// only allows StartGame at revision 0).
    fn pin_manifest(&mut self, cmd: &Command, state: &GameState) {
        let (player_count, rng_seed, ruleset) = match &cmd.kind {
            CommandKind::StartGame {
                player_count,
                rng_seed,
                ruleset,
            } => (*player_count, *rng_seed, ruleset.clone()),
            _ => return,
        };
        let initial_players: Vec<InitialPlayer> = state
            .players
            .sorted_live()
            .into_iter()
            .map(|p| InitialPlayer {
                player: p.id,
                seat: SeatIndex::new(p.seat.raw()),
            })
            .collect();
        let content_lock =
            ContentLockHash(self.content_lock_hash.unwrap_or(state.ruleset.logic_hash));
        let manifest = MatchManifest {
            manifest_version: engine_api::MANIFEST_SCHEMA_VERSION,
            engine_semantics_id: EngineSemanticsId::current(),
            rule_ir_version: RuleIrVersion::CURRENT,
            vm_abi_version: VmAbiVersion::CURRENT,
            ruleset: ResolvedRulesetRef {
                id: state.ruleset.id,
                name: ruleset.clone(),
            },
            content_lock_hash: content_lock,
            protocol_profile: ProtocolProfile::Headless,
            rng_spec: RuleRngSpec::new(rng_seed),
            budget_profile: BudgetProfile {
                effect_budget: state.resolution.effect_budget_max,
                trigger_budget: state.resolution.trigger_budget_max,
                vm_fuel: 1_000_000,
            },
            match_config: MatchConfig {
                ruleset_name: ruleset,
                player_count,
                variant_tags: vec![],
            },
            initial_players,
            versions: engine_api::VersionBundle {
                engine_build_id: ArtifactBuildId::current_native().build,
                ..engine_api::VersionBundle::current()
            },
        };
        let hash = manifest.manifest_hash();
        // Rebuild log with the pinned manifest hash in its header.
        let initial_hash = self.log.header.initial_state_hash;
        self.log = ReplayLog::with_manifest(initial_hash, hash);
        self.manifest = Some(manifest);
    }

    /// Submit and return full pipeline result (tests).
    pub fn submit_full(&mut self, cmd: Command) -> PipelineResult {
        let result = self.submit(cmd);
        PipelineResult {
            result,
            events: self.last_events.clone(),
            choice: self.last_choice.clone(),
        }
    }

    /// Load a content package into the engine (modules + trigger overlay).
    ///
    /// Prefer loading **before** `StartGame` so character skill grant uses the
    /// package catalog. Reloading mid-match rebuilds trigger indexes for live skills.
    pub fn load_package(&mut self, pkg: &ContentPackage) -> Result<(), ContentLoadError> {
        std::sync::Arc::make_mut(&mut self.state.content)
            .load_package(pkg)?;
        self.content_lock_hash = None;
        if self.state.content.combined_logic_hash != [0u8; 32] {
            self.state.ruleset.logic_hash = self.state.content.combined_logic_hash;
        }
        self.state.rebuild_derived_indexes();
        Ok(())
    }

    /// Load the embedded M1 standard skill package (idempotent).
    pub fn load_standard_package(&mut self) -> Result<(), ContentLoadError> {
        self.content_lock_hash = None;
        crate::content::ensure_standard_package(std::sync::Arc::make_mut(
            &mut self.state.content,
        ));
        if self.state.content.combined_logic_hash != [0u8; 32] {
            self.state.ruleset.logic_hash = self.state.content.combined_logic_hash;
        }
        self.state.rebuild_derived_indexes();
        Ok(())
    }

    /// Load the exact packages pinned by a canonical content lock before `StartGame`.
    pub fn load_content_lock(
        &mut self,
        store: &PackageStore,
        lock: &ContentLock,
    ) -> Result<(), RuntimeError> {
        let packages = store.load_locked_packages(lock)?;
        self.load_verified_packages(&packages, lock)
    }

    /// Atomically load verified package values and pin their canonical content lock.
    ///
    /// Raw package content hashes must be checked by the caller that obtained the
    /// bytes. This boundary verifies the lock identity/version/logic bijection.
    pub fn load_verified_packages(
        &mut self,
        packages: &[ContentPackage],
        lock: &ContentLock,
    ) -> Result<(), RuntimeError> {
        // ponytail: content locks load once; stage deltas if fresh-engine clones become measurable.
        let mut staged = self.clone();
        staged.load_verified_packages_inner(packages, lock)?;
        *self = staged;
        Ok(())
    }

    fn load_verified_packages_inner(
        &mut self,
        packages: &[ContentPackage],
        lock: &ContentLock,
    ) -> Result<(), RuntimeError> {
        if self.state.revision != 0
            || self.manifest.is_some()
            || self.state.content.package_count() != 0
        {
            return Err(RuntimeError::Invalid(
                "content lock must load into a fresh pre-match engine".into(),
            ));
        }
        if &ContentLock::from_packages(lock.packages.clone()) != lock {
            return Err(RuntimeError::Invalid(
                "content lock is not canonical".into(),
            ));
        }
        if packages.len() != lock.packages.len() {
            return Err(RuntimeError::Invalid(
                "content lock and package count differ".into(),
            ));
        }

        let mut locked_by_slot = BTreeMap::new();
        for locked in &lock.packages {
            let slot = (locked.namespace.as_str(), locked.package_id.as_str());
            if locked_by_slot.insert(slot, locked).is_some() {
                return Err(RuntimeError::Invalid(format!(
                    "duplicate content-lock package {}/{}",
                    locked.namespace, locked.package_id
                )));
            }
        }
        let mut seen_slots = BTreeSet::new();
        for package in packages {
            let slot = (
                package.manifest.namespace.as_str(),
                package.manifest.package_id.as_str(),
            );
            if !seen_slots.insert(slot) {
                return Err(RuntimeError::Invalid(format!(
                    "duplicate content package {}/{}",
                    slot.0, slot.1
                )));
            }
            let Some(locked) = locked_by_slot.get(&slot) else {
                return Err(RuntimeError::Invalid(format!(
                    "package {}/{} is not in content lock",
                    slot.0, slot.1
                )));
            };
            if package.manifest.version != locked.version
                || hex_hash(&package.manifest.logic_hash) != locked.logic_hash
            {
                return Err(RuntimeError::Invalid(format!(
                    "content lock mismatch for {}/{}@{}",
                    slot.0, slot.1, package.manifest.version
                )));
            }
        }
        if seen_slots.len() != locked_by_slot.len() {
            return Err(RuntimeError::Invalid(
                "content bundle is missing a locked package".into(),
            ));
        }

        for package in packages {
            self.load_package(package)
                .map_err(|error| RuntimeError::Package(error.to_string()))?;
        }
        self.state
            .content
            .validate_character_skill_closure()
            .map_err(|error| RuntimeError::Package(error.to_string()))?;
        self.content_lock_hash = Some(parse_lock_hash(&lock.lock_hash)?);
        Ok(())
    }

    /// Create a fresh engine loaded from the exact active package lock.
    pub fn from_content_lock(
        store: &PackageStore,
        lock: &ContentLock,
    ) -> Result<Self, RuntimeError> {
        let mut engine = Self::new();
        engine.load_content_lock(store, lock)?;
        Ok(engine)
    }

    /// Content registry (read-only).
    pub fn content(&self) -> &ContentRegistry {
        &self.state.content
    }

    /// Capture snapshot, stamped with the pinned manifest hash (architecture §23).
    pub fn snapshot(&self) -> Snapshot {
        let mh = self
            .manifest
            .as_ref()
            .map(|m| m.manifest_hash())
            .unwrap_or_default();
        let mut snapshot = Snapshot::capture_with_manifest(&self.state, mh);
        snapshot.pending_choice = self.last_choice.clone();
        snapshot
    }

    /// Restore from snapshot (clears replay log; caller may keep external log).
    pub fn restore_snapshot(
        &mut self,
        snap: &Snapshot,
    ) -> Result<(), crate::snapshot::SnapshotError> {
        let state = snap.restore()?;
        let hash = canonical_state_hash(&state);
        self.state = state;
        self.log = ReplayLog::new(hash);
        self.last_events.clear();
        self.last_choice = snap.pending_choice.clone();
        self.manifest = None;
        Ok(())
    }

    /// Restore from snapshot preserving the pinned manifest hash (reconnect path).
    ///
    /// After a WAL recovery or reconnect, the manifest must stay pinned to the
    /// original match. This keeps the header manifest hash intact and verifies
    /// the snapshot's embedded manifest hash matches (architecture §23).
    pub fn restore_snapshot_with_manifest(
        &mut self,
        snap: &Snapshot,
        manifest: MatchManifest,
    ) -> Result<(), ManifestRejectionReason> {
        let manifest_hash = manifest.manifest_hash();
        let state = snap
            .restore_with_manifest(manifest_hash)
            .map_err(|e| match e {
                crate::snapshot::SnapshotError::ManifestMismatch { .. } => {
                    ManifestRejectionReason::SnapshotInconsistent
                }
                _ => ManifestRejectionReason::SnapshotInconsistent,
            })?;
        let hash = canonical_state_hash(&state);
        self.state = state;
        self.log = ReplayLog::with_manifest(hash, manifest_hash);
        self.last_events.clear();
        self.last_choice = snap.pending_choice.clone();
        self.manifest = Some(manifest);
        Ok(())
    }

    /// Replay a log of accepted commands onto a fresh engine; returns hashes per revision.
    pub fn replay_from(commands: &[Command]) -> Result<Engine, String> {
        let mut eng = Engine::new();
        for cmd in commands {
            match eng.submit(cmd.clone()) {
                CommandResult::Accepted { .. } => {}
                CommandResult::Rejected { reason } => {
                    return Err(format!("replay rejected: {reason:?}"));
                }
            }
        }
        Ok(eng)
    }

    /// Export the current replay log as self-describing JSON bytes
    /// (architecture §22.3 replay file). Carries the manifest hash in the header.
    pub fn export_replay(&self) -> Result<Vec<u8>, ReplayFileError> {
        self.log.to_bytes()
    }

    /// Import a replay file and reconstruct the engine by replaying its commands.
    ///
    /// Verifies the replay's manifest hash matches `expected_manifest` (if the
    /// replay carries one), then replays commands onto a fresh engine.
    /// Returns the reconstructed engine with the manifest pinned (Phase 16).
    pub fn import_replay(
        bytes: &[u8],
        expected_manifest: Option<MatchManifestHash>,
    ) -> Result<Engine, ReplayFileError> {
        let log = ReplayLog::from_bytes(bytes)?;
        if let Some(expected) = expected_manifest {
            log.verify_manifest(expected)?;
        }
        // Replay the compact commands onto a fresh engine.
        let commands: Vec<Command> = log.commands.iter().map(|c| c.command.clone()).collect();
        let mut eng = Engine::new();
        for cmd in commands {
            match eng.submit(cmd) {
                CommandResult::Accepted { .. } => {}
                CommandResult::Rejected { reason } => {
                    return Err(ReplayFileError::Deserialize(format!(
                        "replay rejected: {reason:?}"
                    )));
                }
            }
        }
        // If the file carried a manifest hash, assert the reconstructed engine
        // pinned the same one.
        if !log.manifest_hash().is_zero() {
            if eng.manifest_hash() != log.manifest_hash() {
                return Err(ReplayFileError::ManifestMismatch {
                    found: eng.manifest_hash(),
                    expected: log.manifest_hash(),
                });
            }
        }
        Ok(eng)
    }

    /// Build a Replay Capsule from the current engine state (architecture §22.4).
    ///
    /// The capsule bundles the manifest, artifact build ids, compiler/VM compat,
    /// replay log, optional snapshot, and content packages. In `Embedded` mode,
    /// content package bytes are included inline so the capsule is self-contained.
    ///
    /// `content_packages` is a list of (package_id, version, content_hash, json_bytes)
    /// tuples. In `Referenced` mode, pass `None` for json_bytes.
    pub fn build_capsule(
        &self,
        mode: crate::capsule::CapsuleMode,
        include_snapshot: bool,
        content_packages: &[(String, String, [u8; 32], Option<Vec<u8>>)],
    ) -> Result<crate::capsule::ReplayCapsule, crate::capsule::CapsuleError> {
        use crate::capsule::{
            CapsuleCompat, CapsuleContentEntry, CapsuleMode, ReplayCapsule, CAPSULE_SCHEMA_VERSION,
        };

        let manifest = self.manifest.clone().ok_or_else(|| {
            crate::capsule::CapsuleError::Serialize(
                "no manifest pinned — match has not started".into(),
            )
        })?;
        let manifest_hash = manifest.manifest_hash();

        let replay_bytes = self
            .log
            .to_bytes()
            .map_err(|e| crate::capsule::CapsuleError::Serialize(e.to_string()))?;

        let snapshot_bytes = if include_snapshot {
            Some(
                self.snapshot()
                    .to_bytes()
                    .map_err(|e| crate::capsule::CapsuleError::Serialize(e.to_string()))?,
            )
        } else {
            None
        };

        let content: Vec<CapsuleContentEntry> = content_packages
            .iter()
            .map(|(pid, ver, hash, bytes)| CapsuleContentEntry {
                package_id: pid.clone(),
                version: ver.clone(),
                content_hash: *hash,
                package_bytes: if mode == CapsuleMode::Embedded {
                    bytes.clone()
                } else {
                    None
                },
            })
            .collect();

        Ok(ReplayCapsule {
            capsule_schema: CAPSULE_SCHEMA_VERSION,
            mode,
            manifest_hash,
            manifest,
            engine_semantics_id: EngineSemanticsId::current(),
            native_artifact: ArtifactBuildId::current_native(),
            wasm_artifact: ArtifactBuildId::current_wasm(),
            compat: CapsuleCompat::current(),
            content,
            replay_bytes,
            snapshot_bytes,
            outcome: if self.state().outcome.is_finished() {
                Some(self.state().outcome.clone())
            } else {
                None
            },
        })
    }

    /// Open a Replay Capsule and reconstruct the engine from its embedded
    /// replay log (architecture §22.4).
    ///
    /// This is the long-term replay path: the capsule carries everything
    /// needed to reproduce the match without external artifacts (in Embedded mode).
    /// Verifies capsule integrity before replaying.
    pub fn open_capsule(
        bytes: &[u8],
    ) -> Result<(Engine, crate::capsule::ReplayCapsule), crate::capsule::CapsuleError> {
        let capsule = crate::capsule::ReplayCapsule::from_bytes(bytes)?;
        capsule.verify()?;
        let log = capsule.replay()?;
        let commands: Vec<Command> = log.commands.iter().map(|c| c.command.clone()).collect();
        let mut eng = Engine::new();
        for cmd in commands {
            match eng.submit(cmd) {
                CommandResult::Accepted { .. } => {}
                CommandResult::Rejected { reason } => {
                    return Err(crate::capsule::CapsuleError::ReplayDecode(format!(
                        "capsule replay rejected: {reason:?}"
                    )));
                }
            }
        }
        // Verify the reconstructed engine matches the capsule's manifest.
        if !capsule.manifest_hash.is_zero() && eng.manifest_hash() != capsule.manifest_hash {
            return Err(crate::capsule::CapsuleError::ManifestHashMismatch);
        }
        Ok((eng, capsule))
    }
}

fn parse_lock_hash(value: &str) -> Result<[u8; 32], RuntimeError> {
    if value.len() != 64 {
        return Err(RuntimeError::Invalid("invalid content lock hash".into()));
    }
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| RuntimeError::Invalid("invalid content lock hash".into()))?;
    }
    Ok(out)
}

fn hex_hash(value: &[u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

    fn start_cmd(seed: u64, n: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: n,
                rng_seed: seed,
                ruleset: "bootstrap".into(),
            },
        }
    }

    #[test]
    fn start_game_is_deterministic() {
        let mut a = Engine::new();
        let mut b = Engine::new();
        let r1 = a.submit(start_cmd(12345, 4));
        let r2 = b.submit(start_cmd(12345, 4));
        assert!(matches!(r1, CommandResult::Accepted { .. }));
        assert_eq!(r1, r2);
        assert_eq!(a.state_hash(), b.state_hash());
        assert_eq!(a.state().players.live_count(), 4);
        // Phase 5 standard deck is larger than 52 basics
        assert!(a.state().cards.live_count() >= 52);
    }

    #[test]
    fn reject_does_not_mutate() {
        let mut e = Engine::new();
        let h0 = e.state_hash();
        let r = e.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        assert!(matches!(r, CommandResult::Rejected { .. }));
        assert_eq!(e.state_hash(), h0);
        assert_eq!(e.state().revision, 0);
    }

    #[test]
    fn rejected_package_preserves_engine_and_content_lock() {
        let mut package = content_compiler::compile_source_named(
            "invalid",
            "skill guarded(1) { modifier card_number override 13; }",
        )
        .unwrap();
        package.modifier_index[0].mode = "add".into();
        package.manifest.logic_hash = package.compute_logic_hash();

        let mut engine = Engine::new();
        engine.content_lock_hash = Some([7; 32]);
        let before = engine.clone();
        assert!(engine.load_package(&package).is_err());
        assert_eq!(engine.state, before.state);
        assert_eq!(engine.content_lock_hash, before.content_lock_hash);
    }

    #[test]
    fn snapshot_restore_preserves_hash() {
        let mut e = Engine::new();
        e.submit(start_cmd(99, 3));
        let h = e.state_hash();
        let snap = e.snapshot();
        let mut e2 = Engine::new();
        e2.restore_snapshot(&snap).unwrap();
        assert_eq!(e2.state_hash(), h);
        assert_eq!(e2.state().revision, e.state().revision);
    }

    #[test]
    fn end_phase_advances() {
        let mut e = Engine::new();
        e.submit(start_cmd(1, 2));
        let rev = e.state().revision;
        let r = e.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        assert!(matches!(r, CommandResult::Accepted { .. }));
        assert_eq!(e.state().revision, rev + 1);
    }

    #[test]
    fn replay_matches_live() {
        let mut live = Engine::new();
        let cmds = vec![
            start_cmd(7, 5),
            Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            },
            Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            },
        ];
        for c in &cmds {
            assert!(matches!(
                live.submit(c.clone()),
                CommandResult::Accepted { .. }
            ));
        }
        let replayed = Engine::replay_from(&cmds).unwrap();
        assert_eq!(live.state_hash(), replayed.state_hash());
    }
}
