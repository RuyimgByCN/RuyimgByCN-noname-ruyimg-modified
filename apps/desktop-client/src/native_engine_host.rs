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
        const HUANLE_RAW: &str =
            include_str!("../../../packages/extensions/huanle-upstream-noname/package.npack.json");
        const HUANLE_CARDS_RAW: &[u8] =
            include_bytes!("../../../packages/extensions/huanle-upstream-noname/assets/cards.json");

        let mut host = HostSession::default();
        host.dispatch(json!({"id": 1, "type": "init"})).unwrap();

        let assets_json = json!({"assets/cards.json": HUANLE_CARDS_RAW}).to_string();
        let frozen = host
            .dispatch(json!({
                "id": 2,
                "type": "validateFrozenPackage",
                "packageJson": HUANLE_RAW,
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
}
