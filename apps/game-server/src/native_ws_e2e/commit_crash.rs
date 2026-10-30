//! Exit inside candidate replacement, then resume the same room through a fresh WS process.

use super::*;
use crate::native_room_store::StoreFault;
use std::process::{Child, Command as ProcessCommand, Stdio};

const CHILD_ROOT: &str = "NONAME_TEST_COMMIT_CRASH_ROOT";

struct ChildServer(Child);

impl Drop for ChildServer {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_for_file(path: &Path) -> String {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if let Ok(value) = std::fs::read_to_string(path) {
                if !value.is_empty() {
                    return value;
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("child process readiness")
}

async fn start_child(base: &Path, restored: usize) -> (ChildServer, String) {
    let ready = base.join("ready");
    let _ = std::fs::remove_file(&ready);
    let child = ProcessCommand::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "native_ws_e2e::commit_crash::native_ws_commit_crash_child",
            "--nocapture",
        ])
        .env(CHILD_ROOT, base)
        .env("NONAME_TEST_COMMIT_CRASH_RESTORED", restored.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let server = ChildServer(child);
    let url = wait_for_file(&ready).await;
    (server, url)
}

async fn wait_for_exit(child: &mut ChildServer, expected: i32) {
    let exit = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(exit) = child.0.try_wait().unwrap() {
                break exit;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("exit exactly inside the requested storage test hook");
    assert_eq!(exit.code(), Some(expected));
}

#[test]
fn native_ws_commit_crash_child() {
    let Some(base) = std::env::var_os(CHILD_ROOT) else {
        return;
    };
    let base = std::path::PathBuf::from(base);
    let mut cfg = config(&base);
    cfg.native_packages_root = base.join("packages");
    let restored = std::env::var("NONAME_TEST_COMMIT_CRASH_RESTORED")
        .unwrap()
        .parse()
        .unwrap();
    runtime().block_on(async {
        let server = Server::start(&cfg, restored).await;
        std::fs::write(base.join("ready"), &server.url).unwrap();
        tokio::time::timeout(Duration::from_secs(60), async {
            while !base.join("arm").exists() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            // 父进程先 create 再写入；只看到空文件时不能当作单故障分支，
            // 否则会在双故障用例里误走 rename 中断（exit 86）。
            let arm = loop {
                let value = std::fs::read_to_string(base.join("arm")).unwrap_or_default();
                if !value.is_empty() {
                    break value;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            };
            let fault = if arm == "double" {
                StoreFault::InterruptAfterCompensationFailure
            } else {
                StoreFault::InterruptAfterRename
            };
            server
                .lobby
                .native_room_store_for_test()
                .unwrap()
                .fail_next_write(fault);
            std::fs::write(base.join("armed"), "ready").unwrap();
            std::future::pending::<()>().await;
        })
        .await
        .expect("parent must trigger the abrupt interruption");
    });
}

#[test]
fn native_extensions_ws_process_exit_after_rename_uses_previous_confirmation() {
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path();
    let cfg = local_config(base);
    runtime().block_on(async {
        let (mut child, url) = start_child(base, 0).await;
        let mut a = Peer::connect_url(&url).await;
        a.create().await;
        let room = a.room;
        let mut b = Peer::connect_url(&url).await;
        b.join(room, 1, None).await;
        let mut spectator = Peer::connect_url(&url).await;
        spectator.spectate(room).await;
        let mut before_a = a.snapshot().await;
        for key in [801, 802, 803] {
            let (result, next) = a
                .submit(a.player, before_a.view.revision, key, CommandKind::EndPhase)
                .await;
            assert!(matches!(result, CommandResult::Accepted { .. }));
            assert_synchronized(&next, &b.pushed(key).await, &spectator.pushed(key).await);
            before_a = next;
        }
        assert!(before_a.choice.is_some());
        let before_b = b.snapshot().await;
        let before_s = spectator.snapshot().await;
        let tokens = [a.token.clone(), b.token.clone()];
        let record_path = Path::new(&cfg.native_room_store).join(format!("{room}.json"));
        let confirmed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
        let kind = before_a
            .legal
            .iter()
            .find(|action| matches!(action, LegalAction::ResolveChoice { cancelled: false, .. }))
            .unwrap()
            .to_command(a.player, before_a.view.revision)
            .kind;
        std::fs::write(base.join("arm"), "interrupt next candidate rename").unwrap();
        wait_for_file(&base.join("armed")).await;
        a.seq += 1;
        a.send(ClientMessage::SubmitCommand {
            header: MessageHeader::new(room, a.seq, before_a.view.revision),
            command: Command {
                actor: a.player,
                expected_revision: ExpectedRevision(before_a.view.revision),
                idempotency_key: IdempotencyKey(804),
                kind: kind.clone(),
            },
            idempotency_key: IdempotencyKey(804),
        replay: false,
})
        .await;
        wait_for_exit(&mut child, 86).await;
        let candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
        assert_ne!(candidate["state"]["snapshot_hex"], confirmed["state"]["snapshot_hex"]);
        drop((a, b, spectator, child));
        std::fs::remove_file(base.join("arm")).unwrap();
        disable_packages(&cfg);
        let (_restarted, url) = start_child(base, 1).await;
        let mut a = Peer::connect_url(&url).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, before_a);
        let mut b = Peer::connect_url(&url).await;
        assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, before_b);
        let mut spectator = Peer::connect_url(&url).await;
        assert_eq!(spectator.spectate(room).await, before_s);
        let (result, next) = a.submit(a.player, before_a.view.revision, 804, kind).await;
        assert!(matches!(result, CommandResult::Accepted { new_revision, .. }
            if new_revision == before_a.view.revision + 1));
        assert_synchronized(&next, &b.pushed(804).await, &spectator.pushed(804).await);
        let resumed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
        for field in ["pack_refs", "frozen_packages"] {
            assert_eq!(resumed["state"][field], confirmed["state"][field]);
        }
        println!("NATIVE_WS process_exit=86 stage=after_candidate_rename before_reply=true packages=disabled confirmed_revision={} retry_key=804 continued_revision={}", before_a.view.revision, next.view.revision);
    });
}

#[test]
fn native_extensions_ws_process_exit_after_double_failure_replays_committed_receipt() {
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path();
    let cfg = local_config(base);
    runtime().block_on(async {
        let (mut child, url) = start_child(base, 0).await;
        let mut a = Peer::connect_url(&url).await;
        a.create().await;
        let room = a.room;
        let mut b = Peer::connect_url(&url).await;
        b.join(room, 1, None).await;
        let mut spectator = Peer::connect_url(&url).await;
        spectator.spectate(room).await;
        let before = super::durability::pending_choice(&mut a, &mut b, &mut spectator).await;
        let intent = super::durability::answer(&before, a.player);
        let tokens = [a.token.clone(), b.token.clone(), spectator.token.clone()];
        std::fs::write(base.join("arm"), "double").unwrap();
        wait_for_file(&base.join("armed")).await;
        a.seq += 1;
        a.send(ClientMessage::SubmitCommand {
            header: MessageHeader::new(room, a.seq, before.view.revision),
            command: Command {
                actor: a.player,
                expected_revision: ExpectedRevision(before.view.revision),
                idempotency_key: IdempotencyKey(1004),
                kind: intent.clone(),
            },
            idempotency_key: IdempotencyKey(1004),
        replay: false,
})
        .await;
        wait_for_exit(&mut child, 87).await;
        let selected = crate::native_room_store::NativeRoomStore::new(&cfg.native_room_store)
            .load_all().unwrap().pop().unwrap();
        let expected_a = super::double_failure::saved_state(&cfg, &selected, &tokens[0]);
        let expected_b = super::double_failure::saved_state(&cfg, &selected, &tokens[1]);
        let expected_s = super::double_failure::saved_state(&cfg, &selected, &tokens[2]);
        assert_eq!(expected_a.view.revision, before.view.revision + 1);
        assert!(expected_a.choice.is_none());
        drop((a, b, spectator, child));
        std::fs::remove_file(base.join("arm")).unwrap();
        disable_packages(&cfg);
        let (restarted, url) = start_child(base, 1).await;
        let mut a = Peer::connect_url(&url).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, expected_a);
        let mut b = Peer::connect_url(&url).await;
        assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, expected_b);
        let mut spectator = Peer::connect_url(&url).await;
        assert_eq!(spectator.spectate(room).await, expected_s);
        let (receipt, replayed) = a.submit(a.player, 0, 1004, intent.clone()).await;
        assert!(matches!(receipt, CommandResult::Accepted { new_revision: 4, .. }));
        assert_eq!(replayed, expected_a, "a committed request must not execute twice");
        let (advance, next) = a.submit(a.player, 4, 1005, CommandKind::EndPhase).await;
        assert!(matches!(advance, CommandResult::Accepted { new_revision: 5, .. }));
        assert_synchronized(&next, &b.pushed(1005).await, &spectator.pushed(1005).await);
        drop((a, b, spectator, restarted));
        let (_second_restart, url) = start_child(base, 1).await;
        let mut a = Peer::connect_url(&url).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, next);
        let (again, current) = a.submit(a.player, 0, 1004, intent).await;
        assert_eq!(again, receipt, "the receipt must survive another command and process restart");
        assert_eq!(current, next);
        println!("NATIVE_WS process_exit=87 stage=after_failed_decision_and_failed_compensation before_reply=true authority=Committed packages=disabled recovered_revision=4 retry_key=1004 applied=once second_restart_revision=5");
    });
}

#[test]
fn native_vm_frozen_room_recovers_after_process_exit_and_uninstall() {
    use crate::room::RoomMsg;
    use crate::session::Outbound;

    let directory = tempfile::tempdir().unwrap();
    let base = directory.path();
    let mut cfg = config(base);
    cfg.native_packages_root = base.join("packages");
    let dropin = cfg.native_packages_root.join("vm-fixture");
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../web-client/scripts/fixtures/vm-trigger");
    std::fs::copy(
        fixture.join("package.npack.json"),
        dropin.join("package.npack.json"),
    )
    .unwrap();
    std::fs::copy(
        fixture.join("character.json"),
        dropin.join("assets/character.json"),
    )
    .unwrap();
    let content = crate::native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .unwrap();
    content.import().unwrap();
    let room = 42;
    let connection = 71;
    let store = crate::native_room_store::NativeRoomStore::new(&cfg.native_room_store);
    let (token, before) = runtime().block_on(async {
        let handle = crate::room_native_actor::spawn_native_room_if_configured(
            room,
            2,
            SEED,
            PASSWORD.into(),
            cfg.room_mailbox,
            cfg.snapshot_every,
            &content,
            true,
            "official",
            &["vm-fixture".into()],
            Some(store.clone()),
            30,
        )
        .unwrap()
        .unwrap();
        let (outbound, _rx) = tokio::sync::mpsc::channel(64);
        let (reply, result) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Join {
                connection_id: connection,
                display_name: "probe".into(),
                preferred_seat: Some(0),
                reconnect_token: None,
                password: PASSWORD.into(),
                outbound: Outbound::new(outbound),
                reply,
            })
            .await
            .unwrap();
        let joined = result.await.unwrap().unwrap();
        let (reply, result) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::ChooseGeneral {
                connection_id: connection,
                name: "vm_fixture".into(),
                reply,
            })
            .await
            .unwrap();
        result.await.unwrap().unwrap();
        let (reply, result) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: connection,
                header: MessageHeader::new(room, 1, 0),
                reply,
            })
            .await
            .unwrap();
        let mut state = State::from_message(result.await.unwrap().unwrap());
        assert!(!state.view.seats[0].skills.is_empty());
        for key in [701, 702, 703] {
            let (reply, result) = tokio::sync::oneshot::channel();
            let revision = state.view.revision;
            handle
                .send(RoomMsg::Submit {
                    connection_id: connection,
                    header: MessageHeader::new(room, key, revision),
                    command: Command {
                        actor: PlayerId::new(1),
                        expected_revision: ExpectedRevision(revision),
                        idempotency_key: IdempotencyKey(key),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(key),
                    replay: false,
                    reply,
                })
                .await
                .unwrap();
            let message = result.await.unwrap().unwrap();
            assert!(matches!(
                &message,
                ServerMessage::CommandOutcome {
                    result: CommandResult::Accepted { .. },
                    ..
                }
            ));
            state = State::from_message(message);
        }
        assert!(state.choice.is_some(), "save while discard choice is open");
        (joined.reconnect_token, state)
    });
    let saved = store.load_all().unwrap().pop().unwrap();
    assert_eq!(saved.room_id, room);
    assert_eq!(saved.state.frozen_packages.as_ref().unwrap().len(), 1);

    runtime().block_on(async {
        let (first, url) = start_child(base, 1).await;
        let mut peer = Peer::connect_url(&url).await;
        assert_eq!(peer.join(room, 0, Some(token.clone())).await, before);
        drop((peer, first));

        std::fs::write(
            dropin.join("manifest.json"),
            br#"{"schema_version":1,"enabled":false}"#,
        )
        .unwrap();
        content.uninstall_package("official", "vm-fixture").unwrap();
        assert!(content.load_scoped(Some("official"), "vm-fixture").is_err());
        assert!(content
            .installed()
            .unwrap()
            .iter()
            .all(|p| p.package_id != "vm-fixture"));

        let (restarted, url) = start_child(base, 1).await;
        assert!(content
            .installed()
            .unwrap()
            .iter()
            .all(|p| p.package_id != "vm-fixture"));
        assert!(content
            .load_scoped(Some("official"), "vm-fixture")
            .is_err());
        let mut peer = Peer::connect_url(&url).await;
        assert_eq!(peer.join(room, 0, Some(token)).await, before);
        let kind = super::durability::answer(&before, peer.player);
        let discarded = match &kind {
            CommandKind::ResolveChoice { selections, .. } => selections.len() as u32,
            _ => unreachable!(),
        };
        let hand_before = before.view.seats[0].hand_count;
        let (result, after_discard) = peer
            .submit(peer.player, before.view.revision, 704, kind)
            .await;
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert_eq!(after_discard.view.phase, engine_api::TurnPhaseTag::Discard);
        assert_eq!(
            after_discard.view.seats[0].hand_count,
            hand_before - discarded
        );
        let (result, end) = peer
            .submit(peer.player, after_discard.view.revision, 705, CommandKind::EndPhase)
            .await;
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert_eq!(end.view.phase, engine_api::TurnPhaseTag::End);
        assert_eq!(
            end.view.seats[0].hand_count,
            after_discard.view.seats[0].hand_count + 1
        );
        assert_eq!(
            end.view.seats[0].skills[0].def,
            before.view.seats[0].skills[0].def
        );
        println!("NATIVE_VM frozen_room process_restart=2 uninstalled=true pending_choice=restored discard={discarded} end_phase_vm_draw=1 revision={}", end.view.revision);
        drop((peer, restarted));
    });
}
