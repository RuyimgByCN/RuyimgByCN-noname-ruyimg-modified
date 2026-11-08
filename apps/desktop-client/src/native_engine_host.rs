//! Serialized native execution of the existing frontend Worker request semantics.
use crate::native_envelope::envelope;
use engine_native_wasm::{
    prepare_locked_content_json, validate_frozen_package_json, validate_pack_json, NativeSession,
    MAX_ARCHIVE_HEX_CHARS,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
};
use tokio::sync::oneshot;

const MAX_PACK_JSON_BYTES: usize = 64 * 1024 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
type Job = (Value, oneshot::Sender<Value>);

/// One bounded queue and one owner thread; engine objects never cross the IPC thread boundary.
pub struct NativeEngineHost {
    sender: Mutex<Option<SyncSender<Job>>>,
    closed: Arc<AtomicBool>,
}

impl NativeEngineHost {
    /// Start an engine owner without blocking the WebView or requiring an engine to be Send.
    pub fn new() -> Result<Self, String> {
        let (sender, receiver) = sync_channel::<Job>(8);
        let closed = Arc::new(AtomicBool::new(false));
        let owner_closed = closed.clone();
        std::thread::Builder::new()
            .name("desktop-engine".into())
            .spawn(move || {
                let mut host = HostSession::default();
                run_requests(receiver, &owner_closed, |request| host.dispatch(request));
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            sender: Mutex::new(Some(sender)),
            closed,
        })
    }

    /// Close the queue; an in-flight command can still complete and must not be retried implicitly.
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        if let Ok(mut sender) = self.sender.lock() {
            sender.take();
        }
    }

    /// Submit a WorkerRequest and asynchronously receive its WorkerResponse.
    /// Queue saturation rejects before enqueue; channel loss reports an unknown operation result.
    pub async fn request(&self, request: Value) -> Result<Value, String> {
        validate_payload_size(&request)?;
        let (sender_response, receiver) = oneshot::channel();
        {
            let sender = self
                .sender
                .lock()
                .map_err(|_| "native engine queue poisoned")?;
            sender
                .as_ref()
                .ok_or("native engine closed; request was not submitted")?
                .try_send((request, sender_response))
                .map_err(|error| match error {
                    std::sync::mpsc::TrySendError::Full(_) => {
                        "native engine busy; request was not submitted".to_string()
                    }
                    std::sync::mpsc::TrySendError::Disconnected(_) => {
                        "native engine unavailable; request was not submitted".to_string()
                    }
                })?;
        }
        receiver.await.map_err(|_| {
            "native engine stopped; operation result unknown; inspect the current match".to_string()
        })
    }
}

fn run_requests(
    receiver: Receiver<Job>,
    closed: &AtomicBool,
    mut dispatch: impl FnMut(Value) -> Result<Value, String>,
) {
    for (request, response) in receiver {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let result = if closed.load(Ordering::Acquire) {
            Err("native engine closed before execution; request was not submitted".into())
        } else {
            dispatch(request)
        };
        let reply = match result {
            Ok(data) => json!({"id": id, "ok": true, "data": data}),
            Err(error) => json!({"id": id, "ok": false, "error": error}),
        };
        // A disconnected caller cannot undo an operation already submitted.
        let _ = response.send(reply);
    }
}

fn validate_payload_size(value: &Value) -> Result<(), String> {
    for (field, limit) in [
        ("hex", MAX_ARCHIVE_HEX_CHARS),
        ("packsJson", MAX_PACK_JSON_BYTES),
        ("packJson", MAX_PACK_JSON_BYTES),
        ("packageJson", MAX_PACK_JSON_BYTES),
        ("assetsJson", MAX_PACK_JSON_BYTES),
        ("frozenPackagesJson", MAX_PACK_JSON_BYTES),
        ("lockedContentJson", MAX_PACK_JSON_BYTES),
        ("json", MAX_PACK_JSON_BYTES),
    ] {
        if value[field].as_str().is_some_and(|text| text.len() > limit) {
            return Err(format!("native: {field} exceeds {limit} bytes"));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Request {
    Init {
        #[serde(rename = "packsJson")]
        packs: Option<String>,
    },
    Start {
        players: u8,
        seed: u64,
        observer: u8,
        #[serde(rename = "packsJson")]
        packs: Option<String>,
        mode: Option<String>,
    },
    StartLocked {
        players: u8,
        seed: u64,
        observer: u8,
        #[serde(rename = "lockedContentJson")]
        locked_content: String,
        mode: Option<String>,
    },
    EndPhase {
        actor: Option<u8>,
    },
    PlayCard {
        actor: u8,
        card: u32,
        targets: Vec<u8>,
    },
    ActivateSkill {
        actor: u8,
        skill: u16,
        #[serde(default)]
        targets: Vec<u8>,
        #[serde(default, rename = "cardTargets")]
        card_targets: Vec<u32>,
    },
    ResolveChoice {
        actor: u8,
        #[serde(rename = "choiceId")]
        choice_id: u64,
        selections: Vec<u32>,
        #[serde(default)]
        cancelled: bool,
    },
    AutoStep,
    Observer,
    Status,
    GeneralCandidates,
    ChooseGeneral {
        seat: u8,
        name: String,
    },
    Save,
    SaveLocked,
    Restore {
        hex: String,
        #[serde(rename = "packsJson")]
        packs: Option<String>,
    },
    RestoreLocked {
        hex: String,
        #[serde(rename = "lockedContentJson")]
        locked_content: String,
    },
    ValidatePackage {
        #[serde(rename = "packJson")]
        pack: String,
    },
    ValidateFrozenPackage {
        #[serde(rename = "packageJson")]
        package_json: String,
        #[serde(rename = "assetsJson")]
        assets_json: String,
    },
    PrepareLockedContent {
        #[serde(rename = "frozenPackagesJson")]
        frozen_packages_json: String,
    },
    UseSkillCard,
    LoadReplay,
    SeekReplay,
}

#[derive(Default)]
struct HostSession {
    session: Option<NativeSession>,
    observer: u8,
    cursor: u32,
}

fn parsed(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

impl HostSession {
    fn dispatch(&mut self, value: Value) -> Result<Value, String> {
        let id = value["id"].as_u64().filter(|id| *id <= MAX_SAFE_INTEGER);
        if id.is_none() {
            return Err("native: invalid request id".into());
        }
        validate_payload_size(&value)?;
        let request: Request = serde_json::from_value(value)
            .map_err(|error| format!("native: invalid request: {error}"))?;
        match request {
            Request::Init { packs } => {
                let candidate =
                    NativeSession::new_with_packs_json(4, 4242, packs.as_deref().unwrap_or("[]"))?;
                self.session = Some(candidate);
                self.observer = 0;
                self.cursor = 0;
                return Ok(json!("native ready"));
            }
            Request::ValidatePackage { pack } => {
                if pack.is_empty() {
                    return Err("native: validatePackage requires nonempty JSON".into());
                }
                validate_pack_json(&pack)?;
                return Ok(json!({"valid": true}));
            }
            Request::ValidateFrozenPackage {
                package_json,
                assets_json,
            } => {
                return parsed(validate_frozen_package_json(&package_json, &assets_json)?);
            }
            Request::PrepareLockedContent {
                frozen_packages_json,
            } => {
                return parsed(prepare_locked_content_json(&frozen_packages_json)?);
            }
            _ => {}
        }
        let session = self
            .session
            .as_mut()
            .ok_or("native engine not initialized")?;
        let result = match request {
            Request::Start {
                players,
                seed,
                observer,
                packs,
                mode,
            } => {
                if players == 0 || observer >= players || seed > MAX_SAFE_INTEGER {
                    return Err("native: invalid players, observer or seed".into());
                }
                let mut candidate = match mode.as_deref() {
                    Some("identity") => NativeSession::new_identity_with_packs_json(
                        players,
                        seed,
                        packs.as_deref().unwrap_or("[]"),
                    )?,
                    None => NativeSession::new_with_packs_json(
                        players,
                        seed,
                        packs.as_deref().unwrap_or("[]"),
                    )?,
                    _ => return Err("native: unknown match mode".into()),
                };
                let data = envelope(&mut candidate, observer, &mut 0, Value::Null, false)?;
                *session = candidate;
                self.observer = observer;
                self.cursor = 0;
                return Ok(data);
            }
            Request::StartLocked {
                players,
                seed,
                observer,
                locked_content,
                mode,
            } => {
                if players == 0 || observer >= players || seed > MAX_SAFE_INTEGER {
                    return Err("native: invalid players, observer or seed".into());
                }
                let mut candidate = match mode.as_deref() {
                    Some("identity") => NativeSession::new_identity_with_locked_content_json(
                        players,
                        seed,
                        &locked_content,
                    )?,
                    None => {
                        NativeSession::new_with_locked_content_json(players, seed, &locked_content)?
                    }
                    _ => return Err("native: unknown match mode".into()),
                };
                let data = envelope(&mut candidate, observer, &mut 0, Value::Null, false)?;
                *session = candidate;
                self.observer = observer;
                self.cursor = 0;
                return Ok(data);
            }
            Request::EndPhase { actor } => {
                parsed(session.next_phase(actor.unwrap_or(self.observer) as u32)?)?
            }
            Request::PlayCard {
                actor,
                card,
                targets,
            } => parsed(session.play_card(
                actor as u32,
                card,
                targets.into_iter().map(u32::from).collect(),
            )?)?,
            Request::ActivateSkill {
                actor,
                skill,
                targets,
                card_targets,
            } => {
                session.retire_public_presentation();
                if !targets.is_empty() || !card_targets.is_empty() {
                    return Err(
                        "native: activateSkill 不接受预选目标（目标由内核开放选择应答决定）".into(),
                    );
                }
                parsed(session.activate_skill(actor as u32, skill)?)?
            }
            Request::ResolveChoice {
                actor,
                choice_id,
                selections,
                cancelled,
            } => {
                session.retire_public_presentation();
                if choice_id > MAX_SAFE_INTEGER {
                    return Err("native: invalid choiceId".into());
                }
                let choice = parsed(session.open_choice_json(actor)?)?;
                if choice["id"].as_u64() != Some(choice_id) {
                    return Err(format!("native: choice {choice_id} 不匹配/不存在"));
                }
                let count = choice["options"].as_array().map_or(0, Vec::len);
                if !cancelled && selections.iter().any(|index| *index as usize >= count) {
                    return Err("native: 选项 token 不在候选".into());
                }
                parsed(session.answer(
                    actor as u32,
                    choice_id,
                    if cancelled { Vec::new() } else { selections },
                )?)?
            }
            Request::AutoStep => parsed(session.auto_step()?)?,
            Request::Observer => {
                return envelope(session, self.observer, &mut self.cursor, Value::Null, false);
            }
            Request::Status => {
                session.retire_public_presentation();
                return Ok(
                    json!({"kind": "status", "revision": session.revision(), "finished": session.is_finished()}),
                );
            }
            Request::GeneralCandidates => return parsed(session.general_candidates_json()),
            Request::ChooseGeneral { seat, name } => {
                session.retire_public_presentation();
                let assignment = parsed(session.assign_general(seat, &name)?)?;
                return Ok(
                    json!({"assignment": assignment, "envelope": envelope(session, self.observer, &mut self.cursor, Value::Null, false)?}),
                );
            }
            Request::Save => return Ok(json!({"kind": "save", "hex": session.save_hex()})),
            Request::SaveLocked => return parsed(session.save_locked_json()?),
            Request::Restore { hex, packs } => {
                session.retire_public_presentation();
                let mut candidate = NativeSession::new(1, 0)?;
                candidate.restore_hex_with_packs_json(&hex, packs.as_deref().unwrap_or("[]"))?;
                let data = envelope(&mut candidate, self.observer, &mut 0, Value::Null, false)?;
                // Preserve planner settings on the live session, as the Worker does.
                session.restore_hex_with_packs_json(&hex, packs.as_deref().unwrap_or("[]"))?;
                self.cursor = 0;
                return Ok(data);
            }
            Request::RestoreLocked {
                hex,
                locked_content,
            } => {
                session.retire_public_presentation();
                let mut candidate = NativeSession::new(1, 0)?;
                candidate.restore_hex_with_locked_content_json(&hex, &locked_content)?;
                let data = envelope(&mut candidate, self.observer, &mut 0, Value::Null, false)?;
                session.restore_hex_with_locked_content_json(&hex, &locked_content)?;
                self.cursor = 0;
                return Ok(data);
            }
            Request::UseSkillCard => return Err("native backend: useSkillCard 未映射".into()),
            Request::LoadReplay => return Err("native backend: loadReplay 未映射".into()),
            Request::SeekReplay => return Err("native backend: seekReplay 未映射".into()),
            Request::Init { .. }
            | Request::ValidatePackage { .. }
            | Request::ValidateFrozenPackage { .. }
            | Request::PrepareLockedContent { .. } => unreachable!(),
        };
        envelope(session, self.observer, &mut self.cursor, result, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn close_allows_the_executing_command_but_rejects_queued_commands() {
        let (sender, receiver) = sync_channel::<Job>(8);
        let closed = Arc::new(AtomicBool::new(false));
        let host = NativeEngineHost {
            sender: Mutex::new(Some(sender)),
            closed: closed.clone(),
        };
        let (entered_tx, entered_rx) = sync_channel(1);
        let (release_tx, release_rx) = sync_channel(1);
        let (finished_tx, finished_rx) = sync_channel(1);
        let owner = std::thread::spawn(move || {
            let mut session = HostSession::default();
            session.dispatch(json!({"id": 1, "type": "init"})).unwrap();
            run_requests(receiver, &closed, |request| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                session.dispatch(request)
            });
            finished_tx
                .send(
                    session
                        .dispatch(json!({"id": 4, "type": "status"}))
                        .unwrap(),
                )
                .unwrap();
        });
        let (executing_tx, executing_rx) = oneshot::channel();
        host.sender
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .try_send((
                json!({"id": 2, "type": "endPhase", "actor": 0}),
                executing_tx,
            ))
            .unwrap();
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let (queued_tx, queued_rx) = oneshot::channel();
        host.sender
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .try_send((json!({"id": 3, "type": "autoStep"}), queued_tx))
            .unwrap();
        host.close();
        release_tx.send(()).unwrap();
        let executing = executing_rx.await.unwrap();
        let queued = queued_rx.await.unwrap();
        assert_eq!(executing["ok"], true);
        assert_eq!(executing["data"]["revision"], 1);
        assert_eq!(queued["ok"], false);
        assert!(queued["error"]
            .as_str()
            .unwrap()
            .contains("closed before execution"));
        assert_eq!(
            finished_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()["revision"],
            1
        );
        assert!(host
            .request(json!({"id": 5, "type": "status"}))
            .await
            .unwrap_err()
            .contains("closed"));
        owner.join().unwrap();
    }

    #[test]
    fn start_locked_save_locked_restore_locked_and_size_bounds() {
        let huanle_raw =
            include_str!("../../../packages/extensions/huanle-upstream-noname/package.npack.json")
                .replace('\r', "");
        let huanle_cards_raw: Vec<u8> =
            include_bytes!("../../../packages/extensions/huanle-upstream-noname/assets/cards.json")
                .iter()
                .copied()
                .filter(|&b| b != b'\r')
                .collect();

        let mut host = HostSession::default();
        host.dispatch(json!({"id": 1, "type": "init"})).unwrap();

        let assets_json = json!({"assets/cards.json": huanle_cards_raw}).to_string();
        let frozen = host
            .dispatch(json!({
                "id": 2,
                "type": "validateFrozenPackage",
                "packageJson": huanle_raw,
                "assetsJson": assets_json,
            }))
            .expect("validateFrozenPackage");
        let frozen_packages_json = serde_json::to_string(&vec![frozen]).unwrap();
        let prepared = host
            .dispatch(json!({
                "id": 3,
                "type": "prepareLockedContent",
                "frozenPackagesJson": frozen_packages_json,
            }))
            .expect("prepareLockedContent");
        let locked_content_json = serde_json::to_string(&prepared).unwrap();

        // startLocked 忽略任何额外传入的 packsJson，仅以 lockedContentJson 构建会话
        let started = host
            .dispatch(json!({
                "id": 4,
                "type": "startLocked",
                "players": 2,
                "seed": 777,
                "observer": 0,
                "lockedContentJson": locked_content_json,
                "packsJson": "not-even-valid-json",
            }))
            .expect("startLocked");
        let initial_rev = started["revision"].as_u64().unwrap();

        let saved = host
            .dispatch(json!({"id": 5, "type": "saveLocked"}))
            .expect("saveLocked");
        assert_eq!(saved["kind"], "saveLocked");
        let hex = saved["hex"].as_str().unwrap().to_string();

        // 推进一步改变状态
        host.dispatch(json!({"id": 6, "type": "endPhase", "actor": 0}))
            .expect("endPhase");
        let mid_status = host
            .dispatch(json!({"id": 7, "type": "status"}))
            .expect("status");
        assert!(mid_status["revision"].as_u64().unwrap() > initial_rev);

        // 错误的 lockedContentJson 恢复失败，且保持当前会话状态与 revision 不变
        let empty_prepared = host
            .dispatch(json!({
                "id": 8,
                "type": "prepareLockedContent",
                "frozenPackagesJson": "[]",
            }))
            .unwrap();
        assert!(host
            .dispatch(json!({
                "id": 9,
                "type": "restoreLocked",
                "hex": hex,
                "lockedContentJson": serde_json::to_string(&empty_prepared).unwrap(),
            }))
            .is_err());
        let status_after_fail = host
            .dispatch(json!({"id": 10, "type": "status"}))
            .expect("status after failed restore");
        assert_eq!(status_after_fail["revision"], mid_status["revision"]);

        // 正确的 lockedContentJson 恢复成功
        let restored = host
            .dispatch(json!({
                "id": 11,
                "type": "restoreLocked",
                "hex": hex,
                "lockedContentJson": locked_content_json,
                "packsJson": "ignored-garbage",
            }))
            .expect("restoreLocked");
        assert_eq!(restored["revision"], initial_rev);

        // 超长 lockedContentJson 被大小上限拒绝
        let huge = "x".repeat(MAX_PACK_JSON_BYTES + 1);
        let err = host
            .dispatch(json!({
                "id": 12,
                "type": "startLocked",
                "players": 2,
                "seed": 1,
                "observer": 0,
                "lockedContentJson": huge,
            }))
            .unwrap_err();
        assert!(err.contains("lockedContentJson exceeds"));
    }

    #[test]
    fn all_37_sources_46_packages_desktop_host_and_ipc_start_step_save_restore() {
        use crate::{DesktopHost, ShellCapabilities};
        use std::collections::{BTreeMap, BTreeSet};
        use std::fs;
        use std::path::PathBuf;

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let map_path = manifest_dir
            .join("../../docs/operations/evidence/2026-10-19-package-source-map/map.tsv");
        let map_text = fs::read_to_string(&map_path).expect("read map.tsv");
        let raw_ext_root = manifest_dir.join("../../packages/extensions");

        let mut by_source: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut all_dirs: BTreeSet<String> = BTreeSet::new();
        for (idx, line) in map_text.lines().enumerate() {
            if idx == 0 || line.trim().is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            let src_id = cols[0].trim().to_string();
            let pkgs: Vec<String> = cols[3]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for p in &pkgs {
                all_dirs.insert(p.clone());
            }
            by_source.insert(src_id, pkgs);
        }
        assert_eq!(by_source.len(), 37, "must cover all 37 upstream sources");
        assert_eq!(all_dirs.len(), 46, "must cover all 46 extension packages");

        // 1. DesktopHost::import_extension_directory over all extension packages
        //    (normalize CRLF in .json files so Windows core.autocrlf checkouts match LF source_hash)
        let tmp = tempfile::tempdir().expect("tempdir");
        let ext_root = tmp.path().join("extensions_lf");
        for dir in &all_dirs {
            let src_pkg = raw_ext_root.join(dir);
            let dst_pkg = ext_root.join(dir);
            fs::create_dir_all(dst_pkg.join("assets")).expect("create staged assets dir");
            let npack_bytes: Vec<u8> = fs::read(src_pkg.join("package.npack.json"))
                .expect("read package.npack.json")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect();
            fs::write(dst_pkg.join("package.npack.json"), npack_bytes)
                .expect("write staged package.npack.json");
            if let Ok(entries) = fs::read_dir(src_pkg.join("assets")) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let mut bytes = fs::read(&path).expect("read asset");
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            bytes.retain(|&b| b != b'\r');
                        }
                        fs::write(dst_pkg.join("assets").join(entry.file_name()), bytes)
                            .expect("write staged asset");
                    }
                }
            }
        }
        let host_root = tmp.path().join("host_store");
        fs::create_dir_all(&host_root).expect("create host_store");
        let desktop = DesktopHost::open(
            &host_root,
            ShellCapabilities {
                manage_content: true,
                export_diagnostics: true,
                allow_unsigned_packages: true,
            },
        )
        .expect("open DesktopHost");
        let report = desktop
            .import_extension_directory(&ext_root, None)
            .expect("import_extension_directory");
        assert!(
            report.installed.len() >= 46,
            "DesktopHost must install all 46 extension packages, got {}",
            report.installed.len()
        );

        // 2. HostSession IPC over all 37 sources / 46 packages
        let mut host = HostSession::default();
        host.dispatch(json!({"id": 1, "type": "init"}))
            .expect("init");
        let empty_prepared = host
            .dispatch(json!({
                "id": 2,
                "type": "prepareLockedContent",
                "frozenPackagesJson": "[]",
            }))
            .expect("empty prepareLockedContent");
        let empty_locked_str = serde_json::to_string(&empty_prepared).unwrap();

        let mut next_id = 10u64;
        for (src_id, dirs) in &by_source {
            let mut frozen_items = Vec::new();
            let mut pack_items = Vec::new();
            for dir in dirs {
                let pkg_path = ext_root.join(dir);
                let npack_raw = fs::read_to_string(pkg_path.join("package.npack.json")).unwrap();
                let npack_val: Value = serde_json::from_str(&npack_raw).unwrap();
                let mut pack_item = json!({ "manifest": npack_val["manifest"] });
                let mut raw_assets: BTreeMap<String, Vec<u8>> = BTreeMap::new();
                if let Some(paths) = npack_val.get("asset_paths").and_then(Value::as_object) {
                    for rel in paths.values().filter_map(Value::as_str) {
                        let bytes = fs::read(pkg_path.join(rel)).unwrap();
                        raw_assets.insert(rel.to_string(), bytes);
                    }
                }
                for (rel, key) in [
                    ("assets/cards.json", "cards"),
                    ("assets/character.json", "characters"),
                    ("assets/pile.json", "pile"),
                    ("assets/config.json", "config"),
                ] {
                    let p = pkg_path.join(rel);
                    if p.is_file() {
                        let bytes = fs::read(&p).unwrap();
                        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
                        pack_item[key] = parsed;
                    }
                }
                next_id += 1;
                host.dispatch(json!({
                    "id": next_id,
                    "type": "validatePackage",
                    "packJson": pack_item.to_string(),
                }))
                .unwrap_or_else(|e| panic!("{src_id}/{dir}: validatePackage failed: {e}"));
                next_id += 1;
                let frozen = host
                    .dispatch(json!({
                        "id": next_id,
                        "type": "validateFrozenPackage",
                        "packageJson": npack_raw,
                        "assetsJson": serde_json::to_string(&raw_assets).unwrap(),
                    }))
                    .unwrap_or_else(|e| {
                        panic!("{src_id}/{dir}: validateFrozenPackage failed: {e}")
                    });
                pack_items.push(pack_item);
                frozen_items.push(frozen);
            }

            next_id += 1;
            let prepared = host
                .dispatch(json!({
                    "id": next_id,
                    "type": "prepareLockedContent",
                    "frozenPackagesJson": serde_json::to_string(&frozen_items).unwrap(),
                }))
                .unwrap_or_else(|e| panic!("{src_id}: prepareLockedContent failed: {e}"));
            let locked_content_json = serde_json::to_string(&prepared).unwrap();

            next_id += 1;
            let started = host
                .dispatch(json!({
                    "id": next_id,
                    "type": "startLocked",
                    "players": 4,
                    "seed": 42007,
                    "observer": 0,
                    "lockedContentJson": locked_content_json,
                }))
                .unwrap_or_else(|e| panic!("{src_id}: startLocked failed: {e}"));
            let rev0 = started["revision"].as_u64().unwrap();

            next_id += 1;
            let stepped = host
                .dispatch(json!({"id": next_id, "type": "autoStep"}))
                .unwrap_or_else(|e| panic!("{src_id}: autoStep failed: {e}"));
            let rev1 = stepped["revision"].as_u64().unwrap();
            assert!(rev1 > rev0, "{src_id}: desktop autoStep must advance revision");
            let state_hash = stepped["hash_hex"].as_str().unwrap().to_string();

            next_id += 1;
            let saved = host
                .dispatch(json!({"id": next_id, "type": "saveLocked"}))
                .unwrap_or_else(|e| panic!("{src_id}: saveLocked failed: {e}"));
            let hex = saved["hex"].as_str().unwrap().to_string();

            next_id += 1;
            assert!(
                host.dispatch(json!({
                    "id": next_id,
                    "type": "restoreLocked",
                    "hex": hex,
                    "lockedContentJson": empty_locked_str,
                }))
                .is_err(),
                "{src_id}: desktop restoreLocked with empty lock must be rejected"
            );

            next_id += 1;
            let restored = host
                .dispatch(json!({
                    "id": next_id,
                    "type": "restoreLocked",
                    "hex": hex,
                    "lockedContentJson": locked_content_json,
                }))
                .unwrap_or_else(|e| panic!("{src_id}: restoreLocked failed: {e}"));
            assert_eq!(restored["revision"].as_u64().unwrap(), rev1);
            assert_eq!(restored["hash_hex"].as_str().unwrap(), state_hash);
        }
    }
}

