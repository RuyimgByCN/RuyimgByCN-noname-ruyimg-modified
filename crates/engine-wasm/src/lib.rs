//! WASM surface for the deterministic game core.
//!
//! All rule work runs here (Web Worker). UI only receives observer JSON.

use content_runtime::{content_hash_hex, ContentLock};

#[path = "fallback_closure.rs.inc"]
mod fallback_closure;
use content_schema::ContentPackage;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId,
    ReplayPrivacy,
};
use engine_core::{
    choose_command, project_observer, project_player, run_identity_match, Engine, MatchConfig,
    MatchReport, ReplayLog,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;

/// Initialize panic hook once.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// WASM-facing engine handle.
#[wasm_bindgen]
pub struct WasmEngine {
    inner: Engine,
    /// Local human player id for projection (default seat 0 / player 1).
    observer_player: u32,
    replay: Option<ReplayLog>,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Create empty engine.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEngine {
        WasmEngine {
            inner: Engine::new(),
            observer_player: 1,
            replay: None,
        }
    }

    /// Replace the fresh engine content only after every package and lock entry is verified.
    #[wasm_bindgen(js_name = loadContentBundle)]
    pub fn load_content_bundle(&mut self, bundle_json: &str) -> Result<(), JsValue> {
        let staged = engine_from_content_bundle(bundle_json).map_err(js_error)?;
        self.inner = staged;
        self.replay = None;
        Ok(())
    }

    /// Load the built-in standard package for browser-only/offline fallback.
    ///
    /// O4 内容债①修复：standard 的 roster 跨包引用 refresh 技能且
    /// refresh/shenhua/yijiang 等逐级跨引（ADR 0542 delta 的传递闭包）
    /// ——本回退路径装载 standard（编译期 Rust 目录）+ 内嵌全依赖闭包
    /// 18 npack 后统一做 roster 闭包校验（此前单包装载必拒）。
    #[wasm_bindgen(js_name = loadEmbeddedStandard)]
    pub fn load_embedded_standard(&mut self) -> Result<(), JsValue> {
        let mut staged = Engine::new();
        staged
            .load_standard_package()
            .map_err(|error| js_error(error.to_string()))?;
        for (pid, bytes) in crate::fallback_closure::FALLBACK_CLOSURE_NPACKS {
            let package = ContentPackage::from_json_bytes(bytes)
                .map_err(|error| js_error(format!("{pid}: {error}")))?;
            staged
                .load_package(&package)
                .map_err(|error| js_error(format!("{pid}: {error}")))?;
        }
        staged
            .content()
            .validate_character_skill_closure()
            .map_err(|error| js_error(error.to_string()))?;
        self.inner = staged;
        self.replay = None;
        Ok(())
    }

    /// Set which player the UI is projecting for.
    #[wasm_bindgen(js_name = setObserverPlayer)]
    pub fn set_observer_player(&mut self, player_id: u32) {
        self.observer_player = player_id;
    }

    /// Start identity match. Returns engine envelope JSON.
    /// `seed` is passed as f64 from JS (safe for seeds &lt; 2^53).
    #[wasm_bindgen(js_name = startGame)]
    pub fn start_game(&mut self, players: u8, seed: f64) -> Result<String, JsValue> {
        let cmd = Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: players.clamp(2, 8),
                rng_seed: seed as u64,
                ruleset: "identity-standard".into(),
            },
        };
        self.apply(cmd)
    }

    /// Submit a JSON-encoded Command. Returns result envelope JSON.
    #[wasm_bindgen(js_name = submitJson)]
    pub fn submit_json(&mut self, command_json: &str) -> Result<String, JsValue> {
        let cmd: Command = serde_json::from_str(command_json)
            .map_err(|e| JsValue::from_str(&format!("bad command json: {e}")))?;
        self.apply(cmd)
    }

    /// End current phase for actor (0 = current turn player).
    #[wasm_bindgen(js_name = endPhase)]
    pub fn end_phase(&mut self, actor: u32) -> Result<String, JsValue> {
        let actor = if actor == 0 {
            self.inner.state().turn.current
        } else {
            PlayerId::new(actor)
        };
        let cmd = Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        };
        self.apply(cmd)
    }

    /// Play card as actor.
    #[wasm_bindgen(js_name = playCard)]
    pub fn play_card(
        &mut self,
        actor: u32,
        card: u32,
        targets_json: &str,
    ) -> Result<String, JsValue> {
        let targets: Vec<u32> = serde_json::from_str(targets_json).unwrap_or_default();
        let cmd = Command {
            actor: PlayerId::new(actor),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::PlayCard {
                card: engine_api::CardId::new(card),
                targets: targets.into_iter().map(PlayerId::new).collect(),
                card_targets: vec![],
            },
        };
        self.apply(cmd)
    }

    /// Activate a skill with player and card targets.
    #[wasm_bindgen(js_name = activateSkill)]
    pub fn activate_skill(
        &mut self,
        actor: u32,
        skill: u32,
        targets_json: &str,
        card_targets_json: &str,
    ) -> Result<String, JsValue> {
        let targets: Vec<u32> = serde_json::from_str(targets_json).unwrap_or_default();
        let card_targets: Vec<u32> = serde_json::from_str(card_targets_json).unwrap_or_default();
        let cmd = Command {
            actor: PlayerId::new(actor),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::ActivateSkill {
                skill: engine_api::SkillId::new(skill),
                targets: targets.into_iter().map(PlayerId::new).collect(),
                card_targets: card_targets
                    .into_iter()
                    .map(engine_api::CardId::new)
                    .collect(),
            },
        };
        self.apply(cmd)
    }

    /// Use a package-declared source-free virtual card conversion.
    #[wasm_bindgen(js_name = useSkillCard)]
    pub fn use_skill_card(
        &mut self,
        actor: u32,
        skill: u32,
        card_kind: u8,
        targets_json: &str,
        equip_slot: u8,
    ) -> Result<String, JsValue> {
        let targets: Vec<u32> = serde_json::from_str(targets_json).unwrap_or_default();
        self.apply(Command {
            actor: PlayerId::new(actor),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::UseSkillCard {
                skill: engine_api::SkillId::new(skill),
                card_kind,
                targets: targets.into_iter().map(PlayerId::new).collect(),
                equip_slot,
            },
        })
    }

    /// Resolve choice.
    #[wasm_bindgen(js_name = resolveChoice)]
    pub fn resolve_choice(
        &mut self,
        actor: u32,
        choice_id: u32,
        selections_json: &str,
        cancelled: bool,
    ) -> Result<String, JsValue> {
        let selections: Vec<u32> = serde_json::from_str(selections_json).unwrap_or_default();
        let cmd = Command {
            actor: PlayerId::new(actor),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(choice_id),
                selections,
                cancelled,
            },
        };
        self.apply(cmd)
    }

    /// Let AI / auto-player take one step.
    #[wasm_bindgen(js_name = autoStep)]
    pub fn auto_step(&mut self) -> Result<String, JsValue> {
        let cmd = choose_command(&self.inner);
        self.apply(cmd)
    }

    /// Current observer snapshot JSON (no mutation).
    #[wasm_bindgen(js_name = observerJson)]
    pub fn observer_json(&self) -> Result<String, JsValue> {
        let snap = project_player(self.inner.state(), PlayerId::new(self.observer_player));
        serde_json::to_string(&snap).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Referee full view JSON (debug only).
    #[wasm_bindgen(js_name = refereeJson)]
    pub fn referee_json(&self) -> Result<String, JsValue> {
        let snap = project_observer(self.inner.state(), ObserverKind::Referee);
        serde_json::to_string(&snap).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// State revision.
    #[wasm_bindgen]
    pub fn revision(&self) -> u64 {
        self.inner.state().revision
    }

    /// Whether match finished.
    #[wasm_bindgen(js_name = isFinished)]
    pub fn is_finished(&self) -> bool {
        self.inner.state().outcome.is_finished()
    }

    /// Hex state hash.
    #[wasm_bindgen(js_name = stateHashHex)]
    pub fn state_hash_hex(&self) -> String {
        self.inner
            .state_hash()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Pending choice JSON or null string.
    #[wasm_bindgen(js_name = pendingChoiceJson)]
    pub fn pending_choice_json(&self) -> String {
        match self.inner.last_choice() {
            Some(c) => serde_json::to_string(c).unwrap_or_else(|_| "null".into()),
            None if self.inner.state().has_pending_choice() => {
                // Reconstruct minimal choice from state for UI
                serde_json::json!({
                    "choice_id": self.inner.state().resolution.pending_choice_id,
                    "observer": self.inner.state().resolution.pending_observer,
                    "revision": self.inner.state().resolution.choice_revision,
                    "kind": format!("{:?}", self.inner.state().resolution.continuation_kind),
                })
                .to_string()
            }
            None => "null".into(),
        }
    }

    /// Load and fully verify a replay before exposing any frame.
    #[wasm_bindgen(js_name = loadReplay)]
    pub fn load_replay(&mut self, replay_json: &str) -> Result<String, JsValue> {
        let log = ReplayLog::from_bytes(replay_json.as_bytes())
            .map_err(|e| js_error(format!("invalid replay: {e}")))?;
        let rebuilt = rebuild_replay(&log, log.commands.len()).map_err(js_error)?;
        let metadata = ReplayMetadata {
            total: log.commands.len(),
            manifest_hash: hex32(&log.manifest_hash().0),
            chain_verified: true,
        };
        self.inner = rebuilt;
        self.replay = Some(log);
        serde_json::to_string(&metadata).map_err(|e| js_error(e.to_string()))
    }

    /// Rebuild the engine through `step` accepted commands and return an
    /// observer-authorized frame. Every reached checkpoint is verified.
    #[wasm_bindgen(js_name = seekReplay)]
    pub fn seek_replay(
        &mut self,
        step: u32,
        privacy: &str,
        player_id: u32,
    ) -> Result<String, JsValue> {
        let log = self
            .replay
            .as_ref()
            .ok_or_else(|| js_error("no replay loaded"))?;
        let step = (step as usize).min(log.commands.len());
        let rebuilt = rebuild_replay(log, step).map_err(js_error)?;
        let observer = match privacy {
            "full" => ObserverKind::ReplayExporter {
                privacy: ReplayPrivacy::Full,
            },
            "player" => ObserverKind::ReplayExporter {
                privacy: ReplayPrivacy::Player(PlayerId::new(player_id.max(1))),
            },
            "public" => ObserverKind::ReplayExporter {
                privacy: ReplayPrivacy::Public,
            },
            _ => return Err(js_error("privacy must be full, player, or public")),
        };
        let frame = ReplayFrame {
            step,
            total: log.commands.len(),
            view: project_observer(rebuilt.state(), observer),
            hash_hex: hex32(&rebuilt.state_hash()),
            revision: rebuilt.state().revision,
            finished: rebuilt.state().outcome.is_finished(),
            verified: true,
        };
        self.inner = rebuilt;
        serde_json::to_string(&frame).map_err(|e| js_error(e.to_string()))
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmEngine {
    fn apply(&mut self, cmd: Command) -> Result<String, JsValue> {
        self.replay = None;
        let result = self.inner.submit(cmd);
        let events = self.inner.last_events().to_vec();
        let choice = self.inner.last_choice().cloned();
        let view = project_player(self.inner.state(), PlayerId::new(self.observer_player));
        let legal_actions =
            engine_ai::legal_actions(&self.inner, PlayerId::new(self.observer_player));
        let env = EngineEnvelope {
            result,
            events,
            choice,
            view,
            legal_actions,
            hash_hex: self.state_hash_hex(),
            revision: self.inner.state().revision,
            finished: self.inner.state().outcome.is_finished(),
        };
        serde_json::to_string(&env).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[derive(Debug, Serialize)]
struct ReplayMetadata {
    total: usize,
    manifest_hash: String,
    chain_verified: bool,
}

#[derive(Debug, Serialize)]
struct ReplayFrame {
    step: usize,
    total: usize,
    view: engine_api::ObserverSnapshot,
    hash_hex: String,
    revision: u64,
    finished: bool,
    verified: bool,
}

fn rebuild_replay(log: &ReplayLog, step: usize) -> Result<Engine, String> {
    validate_replay_structure(log)?;
    let mut engine = Engine::new();
    for (index, checkpoint) in log.commands.iter().take(step).enumerate() {
        if let Some(record) = log.records.get(index) {
            if engine.state().revision != record.pre_revision
                || engine.state_hash() != record.pre_state_hash
            {
                return Err(format!("replay diverged before step {}", index + 1));
            }
        }
        match engine.submit(checkpoint.command.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } if new_revision == checkpoint.revision_after
                && state_hash == checkpoint.state_hash => {}
            CommandResult::Accepted { .. } => {
                return Err(format!("replay checkpoint mismatch at step {}", index + 1));
            }
            CommandResult::Rejected { reason } => {
                return Err(format!(
                    "replay command rejected at step {}: {reason:?}",
                    index + 1
                ));
            }
        }
    }
    if step > 0 && !log.manifest_hash().is_zero() && engine.manifest_hash() != log.manifest_hash() {
        return Err("replay manifest does not match reconstructed match".into());
    }
    Ok(engine)
}

fn validate_replay_structure(log: &ReplayLog) -> Result<(), String> {
    log.verify_chain()
        .map_err(|index| format!("replay hash chain broken at step {}", index + 1))?;
    if log.has_records() {
        if log.records.len() != log.commands.len() {
            return Err("replay compact log and canonical record count differ".into());
        }
        for (index, (checkpoint, record)) in log.commands.iter().zip(log.records.iter()).enumerate()
        {
            if record.record_sequence != index as u64 + 1
                || record.command != checkpoint.command
                || record.post_revision != checkpoint.revision_after
                || record.post_state_hash != checkpoint.state_hash
            {
                return Err(format!("replay record mismatch at step {}", index + 1));
            }
        }
    }
    Ok(())
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentBundle {
    content_lock: ContentLock,
    packages: Vec<String>,
}

fn engine_from_content_bundle(bundle_json: &str) -> Result<Engine, String> {
    let bundle: ContentBundle = serde_json::from_str(bundle_json)
        .map_err(|error| format!("bad content bundle: {error}"))?;
    if ContentLock::from_packages(bundle.content_lock.packages.clone()) != bundle.content_lock {
        return Err("content lock is not canonical".into());
    }
    if bundle.packages.len() != bundle.content_lock.packages.len() {
        return Err("content lock and package count differ".into());
    }

    let mut locked_by_slot = BTreeMap::new();
    for locked in &bundle.content_lock.packages {
        let slot = content_slot(&locked.namespace, &locked.package_id);
        if locked_by_slot.insert(slot, locked).is_some() {
            return Err(format!(
                "duplicate content-lock package {}/{}",
                locked.namespace, locked.package_id
            ));
        }
    }

    let mut seen_slots = BTreeSet::new();
    let mut packages = Vec::with_capacity(bundle.packages.len());
    for raw in &bundle.packages {
        let package = ContentPackage::from_json_bytes(raw.as_bytes())
            .map_err(|error| format!("bad content package JSON: {error}"))?;
        package.validate().map_err(|error| {
            format!(
                "invalid package {}/{}: {error}",
                package.manifest.namespace, package.manifest.package_id
            )
        })?;
        let slot = content_slot(&package.manifest.namespace, &package.manifest.package_id);
        if !seen_slots.insert(slot.clone()) {
            return Err(format!("duplicate content package {slot}"));
        }
        let locked = locked_by_slot
            .get(&slot)
            .ok_or_else(|| format!("package {slot} is not in content lock"))?;
        if package.manifest.version != locked.version
            || hex32(&package.manifest.logic_hash) != locked.logic_hash
            || content_hash_hex(raw.as_bytes()) != locked.content_hash
        {
            return Err(format!(
                "content lock mismatch for {slot}@{}",
                package.manifest.version
            ));
        }
        packages.push(package);
    }
    if seen_slots.len() != locked_by_slot.len() {
        return Err("content bundle is missing a locked package".into());
    }

    for package in &packages {
        for dependency in &package.manifest.dependencies {
            let slot = content_slot(&dependency.namespace, &dependency.package_id);
            let Some(locked) = locked_by_slot.get(&slot) else {
                return Err(format!(
                    "missing dependency {slot} for {}/{}",
                    package.manifest.namespace, package.manifest.package_id
                ));
            };
            if dependency.version != locked.version
                || (dependency.logic_hash != [0u8; 32]
                    && hex32(&dependency.logic_hash) != locked.logic_hash)
            {
                return Err(format!(
                    "dependency mismatch {slot} for {}/{}",
                    package.manifest.namespace, package.manifest.package_id
                ));
            }
        }
    }

    let mut engine = Engine::new();
    engine
        .load_verified_packages(&packages, &bundle.content_lock)
        .map_err(|error| error.to_string())?;
    Ok(engine)
}

fn content_slot(namespace: &str, package_id: &str) -> String {
    format!("{namespace}/{package_id}")
}

fn js_error(message: impl AsRef<str>) -> JsValue {
    JsValue::from_str(message.as_ref())
}

/// Response envelope for UI (batch, not field-by-field FFI).
#[derive(Debug, Serialize, Deserialize)]
struct EngineEnvelope {
    result: CommandResult,
    events: Vec<engine_api::DomainEvent>,
    choice: Option<engine_api::ChoiceRequest>,
    view: engine_api::ObserverSnapshot,
    legal_actions: Vec<engine_api::LegalAction>,
    hash_hex: String,
    revision: u64,
    finished: bool,
}

/// Run a full auto match in WASM (no UI). Returns MatchReport JSON.
#[wasm_bindgen(js_name = runAutoMatch)]
pub fn run_auto_match(players: u8, seed: f64, max_steps: u32) -> Result<String, JsValue> {
    let report: MatchReport = run_identity_match(MatchConfig {
        player_count: players.clamp(2, 8),
        rng_seed: seed as u64,
        max_steps,
        ruleset: "identity-standard".into(),
    })
    .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_runtime::LockedPackage;
    use content_schema::{ContentPackage, PackageManifest};

    #[test]
    fn replay_rebuild_checks_every_checkpoint() {
        let mut source = Engine::new();
        let start = Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 7,
                ruleset: "identity-standard".into(),
            },
        };
        assert!(matches!(
            source.submit(start),
            CommandResult::Accepted { .. }
        ));
        let end = Command {
            actor: source.state().turn.current,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::EndPhase,
        };
        assert!(matches!(source.submit(end), CommandResult::Accepted { .. }));

        let log = source.replay_log().clone();
        let rebuilt = rebuild_replay(&log, 1).expect("first frame");
        assert_eq!(rebuilt.state_hash(), log.commands[0].state_hash);

        let mut tampered = log;
        tampered.commands[1].state_hash[0] ^= 0xff;
        assert!(rebuild_replay(&tampered, 2).is_err());
    }

    #[test]
    fn content_bundle_requires_an_exact_verified_package_set() {
        let mut package = ContentPackage {
            manifest: PackageManifest::new("test", "empty", "1.0.0"),
            modules: Vec::new(),
            skill_index: Vec::new(),
            character_index: Vec::new(),
            skin_index: Default::default(),
            card_index: Vec::new(),
            trigger_index: Vec::new(),
            modifier_index: Vec::new(),
            active_index: Vec::new(),
            view_as_index: Vec::new(),
            translations: Default::default(),
            asset_manifest: None,
            asset_paths: Default::default(),
            compiler_version: "test".into(),
        };
        package.manifest.asset_hash = package.compute_asset_hash();
        package.manifest.logic_hash = package.compute_logic_hash();
        let raw = String::from_utf8(package.to_json_bytes().expect("package JSON")).unwrap();
        let locked = LockedPackage {
            namespace: "test".into(),
            package_id: "empty".into(),
            version: "1.0.0".into(),
            logic_hash: hex32(&package.manifest.logic_hash),
            content_hash: content_hash_hex(raw.as_bytes()),
        };
        let lock = ContentLock::from_packages(vec![locked]);
        let valid = ContentBundle {
            content_lock: lock.clone(),
            packages: vec![raw.clone()],
        };
        let mut engine =
            engine_from_content_bundle(&serde_json::to_string(&valid).unwrap()).expect("valid");
        assert_eq!(engine.content().package_count(), 1);
        assert!(matches!(
            engine.submit(start_cmd(7, 2)),
            CommandResult::Accepted { .. }
        ));
        assert_eq!(
            hex32(
                &engine
                    .manifest()
                    .expect("match manifest")
                    .content_lock_hash
                    .0
            ),
            lock.lock_hash
        );

        let duplicate = ContentBundle {
            content_lock: lock.clone(),
            packages: vec![raw.clone(), raw.clone()],
        };
        assert!(engine_from_content_bundle(&serde_json::to_string(&duplicate).unwrap()).is_err());

        let tampered = ContentBundle {
            content_lock: lock,
            packages: vec![format!("{raw}\n")],
        };
        assert!(engine_from_content_bundle(&serde_json::to_string(&tampered).unwrap()).is_err());
    }

    fn start_cmd(seed: u64, players: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }
    }
}
