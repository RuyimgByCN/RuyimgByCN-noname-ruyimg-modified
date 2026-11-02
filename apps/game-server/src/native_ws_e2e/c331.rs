//! C331 over the production WebSocket actor, real package store, journal, and timer.
use super::*;
use crate::native_room_store::{NativeRoomStore, StoreFault, StoredRoom};
use crate::room_native::NativeRoomSession;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use serde_json::{json, Value};
const ROOM: u64 = 331;
// The 257-module fixtures share CPU-heavy setup; their real clocks must not race it.
static C331_FIXTURE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
struct Fixture {
    _serial: std::sync::MutexGuard<'static, ()>,
    cfg: ServerConfig,
    tokens: [String; 4],
    before: engine_native::snap::StateSnapshot,
    after: engine_native::snap::StateSnapshot,
    pack: engine_native::content::extension::ExtensionPack,
    kind: CommandKind,
}
fn fixture(base: &Path) -> Fixture {
    fixture_with_timeout(base, 90)
}
fn fixture_with_timeout(base: &Path, timeout: u8) -> Fixture {
    let serial = C331_FIXTURE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut cfg = config(base);
    cfg.native_packages_root = base.join("packages");
    cfg.native_room_packages.clear();
    let names: Vec<_> = (0..257).map(|i| format!("c331_ws_{i:03}")).collect();
    let source=names.iter().enumerate().map(|(i,n)|format!("skill {n}({}) locked {{ trigger phase_play_end owner_is_target {{ effect draw_cards(self,1); }} }}",420005000u32+i as u32)).collect::<Vec<_>>().join("\n");
    let mut p = content_compiler::compile_source_named("c331-ws", &source).unwrap();
    for t in &mut p.trigger_index {
        t.priority = 17;
    }
    p.manifest.content_schema_version = p.required_content_schema();
    let characters=serde_json::to_vec(&json!({"generals":[{"name":"C331 WS author","hp":4,"gender":"male","faction":"qun","skills":names.iter().map(|n|json!({"name":n})).collect::<Vec<_>>()}]})).unwrap();
    let mut value = serde_json::to_value(p).unwrap();
    value["asset_manifest"] = json!({"schema_version":1,"package_id":"c331-ws","total_compressed_size":0,"total_decoded_size":0,"entries":[{"asset_id":"c331-ws:characters","source_hash":blake3::hash(&characters).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}]});
    value["asset_paths"] = json!({"c331-ws:characters":"assets/character.json"});
    let mut p: content_schema::ContentPackage = serde_json::from_value(value).unwrap();
    p.manifest.logic_hash = p.compute_logic_hash();
    p.manifest.asset_hash = p.compute_asset_hash();
    p.validate().unwrap();
    let dir = cfg.native_packages_root.join("c331-ws");
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("package.npack.json"), p.to_json_bytes().unwrap()).unwrap();
    std::fs::write(dir.join("assets/character.json"), characters).unwrap();
    let content = crate::native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .unwrap();
    content.import().unwrap();
    let loaded = content.load("c331-ws").unwrap();
    let pack = loaded.pack;
    let mut e = Engine::new_with_extensions(331177, 4, &[&pack]).unwrap();
    e.assign_general_named(1, "C331 WS author").unwrap();
    for _ in 0..50 {
        if e.open_choice()
            .is_some_and(|c| c.spec.prompt == "trigger_order")
        {
            break;
        }
        let action = if let Some(c) = e.open_choice() {
            GameAction::Answer {
                choice_id: c.id,
                selection: c.spec.options.iter().take(c.spec.min).copied().collect(),
            }
        } else {
            GameAction::NextPhase
        };
        let actor = e
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or(e.turn().current);
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action,
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    let choice = e.open_choice().unwrap().clone();
    assert_eq!(choice.observer, 1);
    assert_eq!(choice.spec.options.len(), 257);
    let before = e.save();
    let mut expected = e.clone();
    assert!(!matches!(
        expected.submit(GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![choice.spec.options[256]]
            }
        }),
        CommandOutcome::Rejected(_)
    ));
    let after = expected.save();
    let mut session =
        NativeRoomSession::new_with_packs(e, vec![content.load("c331-ws").unwrap().pack], 4);
    session.set_frozen_packages(vec![loaded.frozen]);
    let other = session
        .join(10, Some(0), None)
        .unwrap()
        .member
        .reconnect_token;
    let owner = session
        .join(11, Some(1), None)
        .unwrap()
        .member
        .reconnect_token;
    session.spectate(90);
    session.spectate(91);
    session.set_choice_timeout_secs(timeout).unwrap();
    session.sync_deadline().unwrap();
    let mut state = session.export_state(&session.snapshot_hex());
    let referee = state
        .members
        .iter_mut()
        .find(|m| m.connection_id == 90)
        .unwrap();
    referee.role = ClientRole::Referee;
    let rt = referee.reconnect_token.clone();
    let st = state
        .members
        .iter()
        .find(|m| m.connection_id == 91)
        .unwrap()
        .reconnect_token
        .clone();
    NativeRoomStore::new(&cfg.native_room_store)
        .save(&StoredRoom {
            room_id: ROOM,
            password_hash: crate::native_room_store::password_hash(ROOM, PASSWORD),
            state,
            checksum: String::new(),
        })
        .unwrap();
    Fixture {
        _serial: serial,
        cfg,
        tokens: [owner, other, rt, st],
        before,
        after,
        pack,
        kind: CommandKind::ResolveChoice {
            choice_id: engine_api::ChoiceId::new(choice.id as u32),
            selections: vec![256],
            cancelled: false,
        },
    }
}
fn snapshot(record: &StoredRoom) -> engine_native::snap::StateSnapshot {
    durability::decode_snapshot(&serde_json::Value::String(
        record.state.snapshot_hex.clone(),
    ))
}
fn equal_snapshot(
    mut a: engine_native::snap::StateSnapshot,
    mut b: engine_native::snap::StateSnapshot,
) {
    a.auth_generation = 0;
    b.auth_generation = 0;
    assert_eq!(a, b);
}
fn dump(name: &str, data: Value) {
    if let Ok(root) = std::env::var("C331_PHASE3_EVIDENCE") {
        std::fs::write(
            Path::new(&root).join(format!("{name}.json")),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    }
}
async fn owner(server: &Server, f: &Fixture) -> (Peer, State) {
    let mut p = Peer::connect(server).await;
    let s = p.join(ROOM, 1, Some(f.tokens[0].clone())).await;
    (p, s)
}
async fn failed_answer(peer: &mut Peer, state: &State, key: u64, kind: CommandKind) {
    peer.seq += 1;
    let seq = peer.seq;
    peer.send(ClientMessage::SubmitCommand {
        header: MessageHeader::new(peer.room, seq, state.view.revision),
        command: Command {
            actor: peer.player,
            expected_revision: ExpectedRevision(state.view.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
        idempotency_key: IdempotencyKey(key),
        replay: false,
    })
    .await;
    let reply = peer
        .until(|m| {
            matches!(m, ServerMessage::Error { .. })
                || matches!(m,ServerMessage::CommandOutcome{header,..}if header.conn_seq==seq)
        })
        .await;
    assert!(
        matches!(
            reply,
            ServerMessage::Error {
                code: ErrorCode::Internal,
                ..
            }
        ),
        "{reply:?}"
    );
}
#[test]
fn c331_ws_permissions_257_retry_and_pending_answered_restart() {
    let temp = tempfile::tempdir().unwrap();
    let f = fixture(temp.path());
    runtime().block_on(async{
  let server=Server::start(&f.cfg,1).await;let(mut a,before)=owner(&server,&f).await;assert_eq!(before.choice.as_ref().unwrap().options.len(),257);assert!(before.legal.len()<=256);
  let mut other=Peer::connect(&server).await;let o=other.join(ROOM,0,Some(f.tokens[1].clone())).await;assert!(o.choice.is_none());let mut referee=Peer::connect(&server).await;let r=referee.join(ROOM,0,Some(f.tokens[2].clone())).await;assert_eq!(r.choice,before.choice);assert_eq!(r.view.observer,ObserverKind::Referee);assert!(r.legal.is_empty());let mut observer=Peer::connect(&server).await;observer.join(ROOM,0,Some(f.tokens[3].clone())).await.assert_spectator();
  double_failure::submit_error(&mut referee,33101,f.kind.clone()).await;double_failure::submit_error(&mut observer,33101,f.kind.clone()).await;
  other.seq+=1;let seq=other.seq;other.send(ClientMessage::SubmitCommand{header:MessageHeader::new(ROOM,seq,before.view.revision),command:Command{actor:a.player,expected_revision:ExpectedRevision(before.view.revision),idempotency_key:IdempotencyKey(33100),kind:f.kind.clone()},idempotency_key:IdempotencyKey(33100),replay:false}).await;
  let denied=other.until(|m|matches!(m,ServerMessage::Error{..})||matches!(m,ServerMessage::CommandOutcome{header,..}if header.conn_seq==seq)).await;assert!(matches!(denied,ServerMessage::Error{..})||matches!(denied,ServerMessage::CommandOutcome{result:CommandResult::Rejected{..},..}));assert_eq!(a.snapshot().await,before);
  for (key,kind) in [(33102,CommandKind::ResolveChoice{choice_id:engine_api::ChoiceId::new(f.before.resolution.open_choice.as_ref().unwrap().id as u32),selections:vec![],cancelled:false}),(33103,CommandKind::ResolveChoice{choice_id:engine_api::ChoiceId::new(f.before.resolution.open_choice.as_ref().unwrap().id as u32),selections:vec![0,1],cancelled:false}),(33104,CommandKind::ResolveChoice{choice_id:engine_api::ChoiceId::new(f.before.resolution.open_choice.as_ref().unwrap().id as u32),selections:vec![257],cancelled:false})]{a.seq+=1;let seq=a.seq;a.send(ClientMessage::SubmitCommand{header:MessageHeader::new(ROOM,seq,before.view.revision),command:Command{actor:a.player,expected_revision:ExpectedRevision(before.view.revision),idempotency_key:IdempotencyKey(key),kind},idempotency_key:IdempotencyKey(key),replay:false}).await;let response=a.until(|m|matches!(m,ServerMessage::Error{..})||matches!(m,ServerMessage::CommandOutcome{header,..}if header.conn_seq==seq)).await;assert!(matches!(response,ServerMessage::Error{..})||matches!(response,ServerMessage::CommandOutcome{result:CommandResult::Rejected{..},..}));assert_eq!(a.snapshot().await,before);}
  let(result,after)=a.submit(a.player,before.view.revision,33105,f.kind.clone()).await;assert!(matches!(result,CommandResult::Accepted{..}));assert_eq!(after.choice.as_ref().unwrap().options.len(),256);let store=server.lobby.native_room_store_for_test().unwrap();let accepted=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&accepted),f.after.clone());let wal=serde_json::to_value(&accepted.state).unwrap();assert!(wal["wal"]["entries"].as_array().unwrap().last().unwrap()["action"]["Answer"]["selection"][0].get("Skill").is_some());
  let(_,retry)=a.submit_replay(a.player,before.view.revision,33105,f.kind.clone()).await;assert_eq!(retry,after);equal_snapshot(snapshot(&store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap()),f.after.clone());
  let wrong=CommandKind::ResolveChoice{choice_id:engine_api::ChoiceId::new(f.before.resolution.open_choice.as_ref().unwrap().id as u32),selections:vec![0],cancelled:false};double_failure::submit_error(&mut a,33105,wrong).await;assert_eq!(a.snapshot().await,after);
  dump("server-ws-permissions-retry",json!({"before":format!("{:?}",f.before),"after":format!("{:?}",f.after),"owner":after.view,"readonly_referee":r.view,"noowner":o.view,"record":accepted}));a.close().await;other.close().await;referee.close().await;observer.close().await;server.stop().await;
 });
    runtime().block_on(async {
        let server = Server::start(&f.cfg, 1).await;
        let (mut a, s) = owner(&server, &f).await;
        assert_eq!(s.choice.as_ref().unwrap().options.len(), 256);
        let record = server
            .lobby
            .native_room_store_for_test()
            .unwrap()
            .load_all()
            .unwrap()
            .into_iter()
            .find(|r| r.room_id == ROOM)
            .unwrap();
        equal_snapshot(snapshot(&record), f.after.clone());
        let (_, retry) = a
            .submit_replay(a.player, f.before.core.revision, 33105, f.kind.clone())
            .await;
        assert_eq!(retry, s);
        a.close().await;
        server.stop().await;
    });
}
#[test]
fn c331_ws_all_single_storage_faults_retry_exactly_once() {
    for fault in [
        StoreFault::PartialWrite,
        StoreFault::FileSync,
        StoreFault::Rename,
        StoreFault::DirectorySync,
        StoreFault::PrepareWrite,
        StoreFault::PrepareSync,
        StoreFault::PrepareDirectorySync,
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let f = fixture(temp.path());
        runtime().block_on(async {
            let server = Server::start(&f.cfg, 1).await;
            let (mut a, before) = owner(&server, &f).await;
            let store = server.lobby.native_room_store_for_test().unwrap();
            let confirmed = store
                .load_all()
                .unwrap()
                .into_iter()
                .find(|r| r.room_id == ROOM)
                .unwrap();
            store.fail_next_write(fault);
            failed_answer(&mut a, &before, 33111, f.kind.clone()).await;
            equal_snapshot(
                snapshot(
                    &store
                        .load_all()
                        .unwrap()
                        .into_iter()
                        .find(|r| r.room_id == ROOM)
                        .unwrap(),
                ),
                snapshot(&confirmed),
            );
            assert_eq!(a.snapshot().await, before);
            let (result, _) = a
                .submit(a.player, before.view.revision, 33111, f.kind.clone())
                .await;
            assert!(matches!(result, CommandResult::Accepted { .. }));
            let after = store
                .load_all()
                .unwrap()
                .into_iter()
                .find(|r| r.room_id == ROOM)
                .unwrap();
            equal_snapshot(snapshot(&after), f.after.clone());
            dump(
                &format!("server-fault-{fault:?}"),
                json!({"confirmed":confirmed,"after_retry":after}),
            );
            a.close().await;
            server.stop().await;
        });
        runtime().block_on(async {
            let server = Server::start(&f.cfg, 1).await;
            let (mut a, s) = owner(&server, &f).await;
            equal_snapshot(
                snapshot(
                    &server
                        .lobby
                        .native_room_store_for_test()
                        .unwrap()
                        .load_all()
                        .unwrap()
                        .into_iter()
                        .find(|r| r.room_id == ROOM)
                        .unwrap(),
                ),
                f.after.clone(),
            );
            let (_, r) = a
                .submit_replay(a.player, f.before.core.revision, 33111, f.kind.clone())
                .await;
            assert_eq!(r, s);
            a.close().await;
            server.stop().await;
        });
    }
}
#[test]
fn c331_real_actor_timeout_wins_and_manual_old_choice_cannot_repeat() {
    let temp = tempfile::tempdir().unwrap();
    let f = fixture_with_timeout(temp.path(), 10);
    runtime().block_on(async {
        let server = Server::start(&f.cfg, 1).await;
        let (mut a, before) = owner(&server, &f).await;
        let initial = server
            .lobby
            .native_room_store_for_test()
            .unwrap()
            .load_all()
            .unwrap()
            .into_iter()
            .find(|r| r.room_id == ROOM)
            .unwrap();
        let automatic = a.pushed(0).await;
        assert!(automatic.view.revision > before.view.revision);
        a.close().await;
        let (mut a, _) = owner(&server, &f).await;
        let store = server.lobby.native_room_store_for_test().unwrap();
        let expired = store
            .load_all()
            .unwrap()
            .into_iter()
            .find(|r| r.room_id == ROOM)
            .unwrap();
        let wal = expired.state.wal.as_ref().unwrap();
        let base = durability::decode_snapshot(&Value::String(wal.base_snapshot_hex.clone()));
        let mut expected = Engine::restore_with_extensions(&base, &[&f.pack]).unwrap();
        let mut ids = std::collections::BTreeSet::new();
        for entry in &wal.entries {
            let command = entry.command();
            if let GameAction::Answer { choice_id, .. }
            | GameAction::TimeoutAction { choice_id, .. } = &command.action
            {
                assert!(
                    ids.insert(*choice_id),
                    "an expired choice cannot execute twice"
                );
            }
            let result = expected.submit(command);
            assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
            assert_eq!(expected.state_hash(), entry.state_hash);
        }
        equal_snapshot(snapshot(&expired), expected.save());
        let (result, _) = a
            .submit(a.player, before.view.revision, 33120, f.kind.clone())
            .await;
        assert!(matches!(result, CommandResult::Rejected { .. }));
        equal_snapshot(
            snapshot(
                &store
                    .load_all()
                    .unwrap()
                    .into_iter()
                    .find(|r| r.room_id == ROOM)
                    .unwrap(),
            ),
            snapshot(&expired),
        );
        dump(
            "server-real-timeout",
            json!({"initial":initial,"automatic_reply":automatic.view,"expired":expired}),
        );
        a.close().await;
        server.stop().await;
    });
}

#[test]
fn c331_ws_all_uncertain_decision_compensation_pairs_recover_exactly_once() {
    for primary in [
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        for compensation in [
            StoreFault::PrepareWrite,
            StoreFault::PrepareSync,
            StoreFault::PrepareDirectorySync,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let f = fixture(temp.path());
            let committed = primary == StoreFault::DecisionDirectorySync
                && compensation != StoreFault::PrepareDirectorySync;
            runtime().block_on(async{let server=Server::start(&f.cfg,1).await;let(mut a,before)=owner(&server,&f).await;let store=server.lobby.native_room_store_for_test().unwrap();let confirmed=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();store.fail_next_write_with_compensation(primary,compensation);failed_answer(&mut a,&before,33130,f.kind.clone()).await;assert_eq!(a.snapshot().await,before);failed_answer(&mut a,&before,33130,f.kind.clone()).await;dump(&format!("server-double-{primary:?}-{compensation:?}"),json!({"confirmed":confirmed,"ram_reply":format!("{before:?}"),"decision_committed_expected":committed}));a.close().await;server.stop().await;});
            runtime().block_on(async{let server=Server::start(&f.cfg,1).await;let(mut a,state)=owner(&server,&f).await;let store=server.lobby.native_room_store_for_test().unwrap();let recovered=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&recovered),if committed{f.after.clone()}else{f.before.clone()});let(result,_)=if committed{a.submit_replay(a.player,f.before.core.revision,33130,f.kind.clone()).await}else{a.submit(a.player,f.before.core.revision,33130,f.kind.clone()).await};assert!(matches!(result,CommandResult::Accepted{..}));let final_record=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&final_record),f.after.clone());dump(&format!("server-double-restart-{primary:?}-{compensation:?}"),json!({"recovered":recovered,"final":final_record,"initial_reply":format!("{state:?}")}));a.close().await;server.stop().await;});
        }
    }
}

#[test]
fn c331_ws_process_exit_after_replace_and_uncertain_decision_two_restarts() {
    for double in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let f = fixture(temp.path());
        runtime().block_on(async {
            let (mut child, url) = commit_crash::start_child(temp.path(), 1).await;
            let mut a = Peer::connect_url(&url).await;
            a.join(ROOM, 1, Some(f.tokens[0].clone())).await;
            std::fs::write(
                temp.path().join("arm"),
                if double { "double" } else { "rename" },
            )
            .unwrap();
            commit_crash::wait_for_file(&temp.path().join("armed")).await;
            a.send(ClientMessage::SubmitCommand {
                header: MessageHeader::new(ROOM, 51, f.before.core.revision),
                command: Command {
                    actor: a.player,
                    expected_revision: ExpectedRevision(f.before.core.revision),
                    idempotency_key: IdempotencyKey(33140),
                    kind: f.kind.clone(),
                },
                idempotency_key: IdempotencyKey(33140),
                replay: false,
            })
            .await;
            commit_crash::wait_for_exit(&mut child, if double { 87 } else { 86 }).await;
            drop(a);
            drop(child);
            let _ = std::fs::remove_file(temp.path().join("arm"));
            let _ = std::fs::remove_file(temp.path().join("armed"));
            let (child, url) = commit_crash::start_child(temp.path(), 1).await;
            let mut a = Peer::connect_url(&url).await;
            let s = a.join(ROOM, 1, Some(f.tokens[0].clone())).await;
            assert_eq!(
                s.choice.as_ref().unwrap().options.len(),
                if double { 256 } else { 257 }
            );
            let (result, _) = if double {
                a.submit_replay(a.player, f.before.core.revision, 33140, f.kind.clone())
                    .await
            } else {
                a.submit(a.player, f.before.core.revision, 33140, f.kind.clone())
                    .await
            };
            assert!(matches!(result, CommandResult::Accepted { .. }));
            a.close().await;
            drop(child);
            let (child, url) = commit_crash::start_child(temp.path(), 1).await;
            let mut a = Peer::connect_url(&url).await;
            let s = a.join(ROOM, 1, Some(f.tokens[0].clone())).await;
            let (_, r) = a
                .submit_replay(a.player, f.before.core.revision, 33140, f.kind.clone())
                .await;
            assert_eq!(r, s);
            let record = NativeRoomStore::new(&f.cfg.native_room_store)
                .load_all()
                .unwrap()
                .into_iter()
                .find(|r| r.room_id == ROOM)
                .unwrap();
            equal_snapshot(snapshot(&record), f.after.clone());
            dump(
                &format!("server-process-crash-{double}"),
                json!({"final_record":record,"reply":format!("{s:?}")}),
            );
            a.close().await;
            drop(child);
        });
    }
}

#[test]
fn c331_manual_wins_then_real_old_deadline_passes_without_another_commit() {
    let temp = tempfile::tempdir().unwrap();
    let f = fixture_with_timeout(temp.path(), 10);
    runtime().block_on(async {
        let server=Server::start(&f.cfg,1).await;let(mut a,before)=owner(&server,&f).await;
        let old_deadline=before.choice.as_ref().unwrap().deadline_unix_ms.unwrap();
        let now=crate::room_native::unix_ms().unwrap();assert!(old_deadline>now+2500,"fixture must join before the real deadline");
        tokio::time::sleep(Duration::from_millis(old_deadline-now-2500)).await;
        let(result,manual)=a.submit(a.player,before.view.revision,33150,f.kind.clone()).await;assert!(matches!(result,CommandResult::Accepted{..}));
        let committed_at=crate::room_native::unix_ms().unwrap();assert!(committed_at<old_deadline,"manual must commit first");
        let next_deadline=manual.choice.as_ref().unwrap().deadline_unix_ms.unwrap();assert!(next_deadline>old_deadline+1000,"new choice has its own later deadline");
        let store=server.lobby.native_room_store_for_test().unwrap();
        let accepted=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&accepted),f.after.clone());
        assert_eq!(accepted.state.wal.as_ref().unwrap().entries.len(),1);assert!(accepted.state.timeout.as_ref().unwrap().managed_seats.is_empty());
        let wait=old_deadline.saturating_add(600).saturating_sub(crate::room_native::unix_ms().unwrap());tokio::time::sleep(Duration::from_millis(wait)).await;
        let observed_at=crate::room_native::unix_ms().unwrap();assert!(observed_at>=old_deadline&&observed_at<next_deadline);
        assert_eq!(a.snapshot().await,manual,"old deadline cannot answer the new choice");
        let after_old_deadline=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&after_old_deadline),f.after.clone());
        assert_eq!(serde_json::to_value(&after_old_deadline.state).unwrap(),serde_json::to_value(&accepted.state).unwrap(),"WAL/control/receipt and full archive remain exactly the committed state");
        let(_,retry)=a.submit_replay(a.player,before.view.revision,33150,f.kind.clone()).await;assert_eq!(retry,manual);
        a.seq+=1;let seq=a.seq;a.send(ClientMessage::SubmitCommand{header:MessageHeader::new(ROOM,seq,manual.view.revision),command:Command{actor:a.player,expected_revision:ExpectedRevision(manual.view.revision),idempotency_key:IdempotencyKey(33151),kind:f.kind.clone()},idempotency_key:IdempotencyKey(33151),replay:false}).await;
        let rejected=a.until(|m|matches!(m,ServerMessage::Error{..})||matches!(m,ServerMessage::CommandOutcome{header,..}if header.conn_seq==seq)).await;
        assert!(matches!(rejected,ServerMessage::Error{..})||matches!(rejected,ServerMessage::CommandOutcome{result:CommandResult::Rejected{..},..}));assert_eq!(a.snapshot().await,manual);
        let final_record=store.load_all().unwrap().into_iter().find(|r|r.room_id==ROOM).unwrap();equal_snapshot(snapshot(&final_record),f.after.clone());assert_eq!(final_record.state.wal.as_ref().unwrap().entries.len(),1);
        dump("server-manual-before-real-old-deadline",json!({"old_deadline":old_deadline,"manual_committed_at":committed_at,"observed_after_old_deadline":observed_at,"new_choice_deadline":next_deadline,"accepted":accepted,"after_old_deadline":after_old_deadline,"final":final_record,"manual_reply":format!("{manual:?}")}));
        a.close().await;server.stop().await;
    });
}
