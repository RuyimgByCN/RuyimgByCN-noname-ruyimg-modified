//! Production native rooms over loopback TCP/WebSocket, including restart and content locks.

mod authority;
mod commit_crash;
mod double_failure;
mod durability;
mod referee;
mod source_skill;

use super::{bootstrap, build_router, config::ServerConfig, lobby::Lobby};
use engine_api::{
    ChoiceRequest, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    LegalAction, ObserverKind, ObserverSnapshot, PlayerId, RejectReason,
};
use engine_protocol::{
    decode_s2c, encode_c2s, ClientMessage, ClientRole, ErrorCode, MessageHeader, ServerMessage,
};
use futures_util::{SinkExt, StreamExt};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tower_http::services::ServeDir;

const PASSWORD: &str = "native-ws-test-password";
const SEED: u64 = 424_242;

struct Server {
    url: String,
    lobby: Arc<Lobby>,
    task: JoinHandle<()>,
}

impl Server {
    async fn start(cfg: &ServerConfig, restored: usize) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind loopback");
        Self::start_on(cfg, restored, listener).await
    }

    async fn start_on(
        cfg: &ServerConfig,
        restored: usize,
        listener: tokio::net::TcpListener,
    ) -> Self {
        let lobby = bootstrap(cfg);
        assert_eq!(
            lobby.restore_native_rooms().expect("restore rooms"),
            restored
        );
        let app = build_router(lobby.clone(), ServeDir::new(&cfg.original_content_root));
        let url = format!("ws://{}/ws", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self { url, lobby, task }
    }

    async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

struct Peer {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    room: u64,
    player: PlayerId,
    token: String,
    seq: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    view: ObserverSnapshot,
    legal: Vec<LegalAction>,
    choice: Option<ChoiceRequest>,
}

impl State {
    fn from_message(message: ServerMessage) -> Self {
        match message {
            ServerMessage::JoinOk {
                view,
                legal_actions,
                choice,
                ..
            }
            | ServerMessage::Snapshot {
                view,
                legal_actions,
                choice,
                ..
            }
            | ServerMessage::CommandOutcome {
                view,
                legal_actions,
                choice,
                ..
            } => Self {
                view,
                legal: legal_actions,
                choice,
            },
            other => panic!("expected personalized state, got {other:?}"),
        }
    }

    fn assert_player(&self, player: PlayerId) {
        assert_eq!(self.view.observer, ObserverKind::Player(player));
        let hand: Vec<_> = self.view.known_cards.iter()
            .filter(|card| card.zone == engine_api::CardZoneTag::Hand).collect();
        assert!(hand.iter().all(|card| card.owner == Some(player)));
        assert_eq!(
            hand.len() as u32,
            self.view.seats.iter().find(|seat| seat.player == player).unwrap().hand_count
        );
        assert!(self.view.known_cards.iter().all(|card| {
            card.def.is_some() && card.suit.is_some() && card.rank.is_some()
                && matches!(card.zone, engine_api::CardZoneTag::Hand
                    | engine_api::CardZoneTag::Equip | engine_api::CardZoneTag::Judge
                    | engine_api::CardZoneTag::DiscardPile | engine_api::CardZoneTag::Processing)
        }));
        if let Some(choice) = &self.choice {
            assert_eq!(choice.observer, player);
        }
    }

    fn assert_spectator(&self) {
        assert_eq!(self.view.observer, ObserverKind::Spectator);
        assert!(self.view.known_cards.iter().all(|card| {
            card.def.is_some() && card.suit.is_some() && card.rank.is_some()
                && matches!(card.zone, engine_api::CardZoneTag::Equip
                    | engine_api::CardZoneTag::Judge | engine_api::CardZoneTag::DiscardPile
                    | engine_api::CardZoneTag::Processing)
        }));
        assert!(self.legal.is_empty());
        assert!(self.choice.is_none());
    }
}

mod peer;

fn config(base: &Path) -> ServerConfig {
    let mut cfg = ServerConfig::default();
    cfg.original_content_root = base.join("original");
    cfg.extensions_root = base.join("legacy-extensions");
    cfg.content_store_root = base.join("legacy-store");
    cfg.native_packages_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    cfg.native_store_root = base.join("native-store");
    cfg.native_room_store = base.join("rooms").to_str().unwrap().to_owned();
    cfg.native_rooms = true;
    cfg.native_room_packages = vec!["shahai-swd".into(), "wudian".into()];
    cfg.max_cmds_per_sec = 1000;
    for path in [&cfg.original_content_root, &cfg.extensions_root] {
        std::fs::create_dir_all(path).unwrap();
    }
    cfg
}

fn local_config(base: &Path) -> ServerConfig {
    let mut cfg = config(base);
    let source = cfg.native_packages_root.clone();
    cfg.native_packages_root = base.join("packages");
    for package in ["shahai-swd-upstream-noname", "wudian-upstream-noname"] {
        copy_directory(
            &source.join(package),
            &cfg.native_packages_root.join(package),
        );
    }
    cfg
}

fn disable_packages(cfg: &ServerConfig) {
    for entry in std::fs::read_dir(&cfg.native_packages_root).unwrap() {
        let path = entry.unwrap().path().join("manifest.json");
        let mut descriptor: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        descriptor["enabled"] = serde_json::json!(false);
        std::fs::write(path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
    }
    let content = super::native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .unwrap();
    content.import().unwrap();
    assert!(content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .is_err());
    let store = content_runtime::PackageStore::open(&cfg.native_store_root).unwrap();
    assert!(store.current_lock().unwrap().packages.is_empty());
    std::fs::remove_dir_all(cfg.native_store_root.join("packages")).unwrap();
}

fn assert_synchronized(a: &State, b: &State, spectator: &State) {
    a.assert_player(PlayerId::new(1));
    b.assert_player(PlayerId::new(2));
    spectator.assert_spectator();
    for state in [b, spectator] {
        assert_eq!(state.view.revision, a.view.revision);
        assert_eq!(
            state.view.state_hash, a.view.state_hash,
            "authority hash is shared"
        );
        assert_eq!(state.view.current_player, a.view.current_player);
        assert_eq!(state.view.phase, a.view.phase);
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn native_extensions_ws_reconnects_after_disk_restart() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    // Dropping each runtime joins its workers before another bootstrap can write the same store.
    let (room, tokens, before_a, before_b, before_spectator, saved_hex, initial_hash) = runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        let mut a = Peer::connect(&server).await;
        let initial_a = a.create().await;
        let initial_hash = initial_a.view.state_hash;
        let content = super::native::NativeContent::open(
            cfg.native_packages_root.clone(),
            cfg.native_store_root.clone(),
        )
        .unwrap();
        let swd = content
            .load_scoped(Some("upstream-noname"), "shahai-swd")
            .unwrap();
        let wudian = content
            .load_scoped(Some("upstream-noname"), "wudian")
            .unwrap();
        let expected =
            engine_native::Engine::new_with_extensions(SEED, 2, &[&swd.pack, &wudian.pack]).unwrap();
        let standard = engine_native::Engine::new(SEED, 2);
        assert!(expected.catalog().cards.len() > standard.catalog().cards.len());
        assert!(expected.view_for(1).deck_count > standard.view_for(1).deck_count);
        assert_ne!(
            expected.state_hash(),
            standard.state_hash(),
            "extensions change actual initial state"
        );
        assert_eq!(
            initial_a.view.state_hash,
            expected.state_hash(),
            "WS authority uses the real extension deck"
        );
        assert_eq!(
            initial_a
                .view
                .known_cards
                .iter()
                .map(|card| card.id.raw())
                .collect::<Vec<_>>(),
            expected
                .state_core()
                .player(0)
                .hand
                .iter()
                .map(|card| *card as u32)
                .collect::<Vec<_>>(),
            "real shuffled extension hand reaches the socket projection"
        );
        assert!(initial_a.legal.contains(&LegalAction::EndPhase));
        let room = a.room;
        let mut b = Peer::connect(&server).await;
        let initial_b = b.join(room, 1, None).await;
        assert!(
            initial_b.legal.is_empty(),
            "other seat cannot advance this turn"
        );
        let mut spectator = Peer::connect(&server).await;
        let initial_spectator = spectator.spectate(room).await;
        assert_synchronized(&initial_a, &initial_b, &initial_spectator);

        let mut intruder = Peer::connect(&server).await;
        intruder
            .send(ClientMessage::JoinRoom {
                room_id: room,
                password: "wrong-password".into(),
                preferred_seat: Some(0),
                reconnect_token: Some(a.token.clone()),
            })
            .await;
        intruder.error(ErrorCode::Unauthorized).await;
        intruder.close().await;

        let (spoofed, unchanged) = b.submit(a.player, initial_a.view.revision, 100, CommandKind::EndPhase).await;
        assert!(
            matches!(spoofed, CommandResult::Rejected { .. }),
            "non-turn connection must not control A"
        );
        assert_eq!(unchanged.view.revision, initial_b.view.revision);
        assert_eq!(unchanged.view.state_hash, initial_b.view.state_hash);

        let mut latest = initial_a;
        for key in [101, 102, 103] {
            assert!(latest.legal.contains(&LegalAction::EndPhase));
            let before = latest.view.revision;
            let (result, state) = a.submit(b.player, before, key, CommandKind::EndPhase).await;
            assert!(
                matches!(result, CommandResult::Accepted { new_revision, state_hash }
                if new_revision > before && new_revision == state.view.revision && state_hash == state.view.state_hash)
            );
            let pushed_b = b.pushed(key).await;
            let pushed_spectator = spectator.pushed(key).await;
            assert_synchronized(&state, &pushed_b, &pushed_spectator);
            latest = state;
        }
        assert!(latest.view.revision >= 3);
        let choice = latest.choice.as_ref().expect("EndPhase into Discard opens a real choice");
        assert_eq!(choice.min_count, 2, "six drawn cards require discarding two at hp four");
        assert_eq!(choice.observer, a.player);
        for (index, option) in choice.options.iter().enumerate() {
            assert_eq!(option.token, index as u32, "wire answers use option indices");
        }
        let answers: Vec<_> = latest.legal.iter().filter_map(|action| match action {
            LegalAction::ResolveChoice { choice_id, selections, cancelled: false } => {
                assert_eq!(*choice_id, choice.choice_id);
                assert!(selections.len() >= choice.min_count as usize);
                assert!(selections.len() <= choice.max_count as usize);
                assert!(selections.iter().all(|token| choice.options.iter().any(|option| option.token == *token)));
                Some(action)
            }
            _ => None,
        }).collect();
        assert!(!answers.is_empty(), "owner receives a usable authoritative answer");
        let (stale, unchanged) = a.submit(a.player, 1, 104, CommandKind::EndPhase).await;
        assert!(matches!(stale, CommandResult::Rejected {
            reason: RejectReason::StaleRevision { current, expected: 1 }
        } if current == latest.view.revision));
        assert_eq!(
            unchanged, latest,
            "stale command must not mutate personalized state"
        );

        a.seq += 1;
        let stale_sequence = a.seq;
        a.send(ClientMessage::SubmitCommand {
            header: MessageHeader::new(room, stale_sequence, 0),
            command: Command {
                actor: a.player,
                expected_revision: ExpectedRevision(1),
                idempotency_key: IdempotencyKey(106),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: IdempotencyKey(106),
        replay: false,
}).await;
        let stale_body = a.until(|message| matches!(message,
            ServerMessage::CommandOutcome { header, .. } if header.conn_seq == stale_sequence
        )).await;
        assert!(matches!(&stale_body, ServerMessage::CommandOutcome {
            result: CommandResult::Rejected { reason: RejectReason::StaleRevision { current, expected: 1 } }, ..
        } if *current == latest.view.revision), "an unspecified header must preserve the body's expected revision");
        assert_eq!(State::from_message(stale_body), latest);

        let before_a = a.snapshot().await;
        assert_eq!(before_a, latest, "snapshot preserves the complete pending choice");
        let before_b = b.snapshot().await;
        let before_spectator = spectator.snapshot().await;
        assert_synchronized(&before_a, &before_b, &before_spectator);
        assert!(before_b.choice.is_none() && before_b.legal.is_empty(), "personal barrier stays with its owner");
        let tokens = [a.token.clone(), b.token.clone()];
        let stored = super::native_room_store::NativeRoomStore::new(&cfg.native_room_store)
            .load_all()
            .unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].state.pack_refs.len(), 2);
        for package in ["shahai-swd", "wudian"] {
            assert!(stored[0]
                .state
                .pack_refs
                .iter()
                .any(|reference| reference.contains(package)));
        }
        assert!(!stored[0].password_hash.is_empty());
        assert_ne!(stored[0].password_hash, PASSWORD);
        a.close().await;
        b.close().await;
        spectator.close().await;
        server.stop().await;
        (room, tokens, before_a, before_b, before_spectator, stored[0].state.snapshot_hex.clone(), initial_hash)
    });

    disable_packages(&cfg);
    for phase in ["disabled", "same-version-repacked", "upgraded"] {
        if phase != "disabled" {
            upgrade_packages(
                &cfg,
                if phase == "upgraded" {
                    "1.1.0"
                } else {
                    "1.0.0"
                },
            );
        }
        runtime().block_on(async {
            let restarted = Server::start(&cfg, 1).await;
            let mut a = Peer::connect(&restarted).await;
            assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, before_a);
            assert_eq!(a.token, tokens[0]);
            let mut b = Peer::connect(&restarted).await;
            assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, before_b);
            assert_eq!(b.token, tokens[1]);
            let mut spectator = Peer::connect(&restarted).await;
            assert_eq!(spectator.spectate(room).await, before_spectator);
            if phase != "upgraded" {
                println!("NATIVE_WS frozen_restore=pass packages={phase} choice=pending");
                a.close().await;
                b.close().await;
                spectator.close().await;
                restarted.stop().await;
                return;
            }

            let content = super::native::NativeContent::open(
                cfg.native_packages_root.clone(),
                cfg.native_store_root.clone(),
            )
            .unwrap();
            let swd = content
                .load_scoped(Some("upstream-noname"), "shahai-swd")
                .unwrap();
            let wudian = content
                .load_scoped(Some("upstream-noname"), "wudian")
                .unwrap();
            assert!(swd.pack.def.name.contains("1.1.0"));
            let expected =
                engine_native::Engine::new_with_extensions(SEED, 2, &[&swd.pack, &wudian.pack])
                    .unwrap();
            let mut new_game = Peer::connect(&restarted).await;
            let upgraded_initial = new_game.create().await;
            assert_ne!(new_game.room, room);
            assert_eq!(upgraded_initial.view.state_hash, expected.state_hash());
            assert_ne!(
                upgraded_initial.view.state_hash, initial_hash,
                "new rooms must use the changed active version rather than the old frozen copy"
            );
            assert_eq!(
                a.snapshot().await,
                before_a,
                "new active content must not change the restored room"
            );
            new_game.close().await;

            let answer = before_a
                .legal
                .iter()
                .find(|action| {
                    matches!(
                        action,
                        LegalAction::ResolveChoice {
                            cancelled: false,
                            ..
                        }
                    )
                })
                .expect("restored authoritative answer");
            let command = answer.to_command(a.player, before_a.view.revision);
            let (result, after) = a
                .submit(a.player, before_a.view.revision, 105, command.kind)
                .await;
            assert!(
                matches!(result, CommandResult::Accepted { new_revision, .. }
        if new_revision > before_a.view.revision)
            );
            let pushed_b = b.pushed(105).await;
            let pushed_spectator = spectator.pushed(105).await;
            assert_synchronized(&after, &pushed_b, &pushed_spectator);
            assert_ne!(
                super::native_room_store::NativeRoomStore::new(&cfg.native_room_store)
                    .load_all()
                    .unwrap()[0]
                    .state
                    .snapshot_hex,
                saved_hex,
                "continued WS command must reach disk"
            );
            println!(
                "NATIVE_WS lifecycle=pass real_packages=shahai-swd,wudian revision={} restored=1",
                after.view.revision
            );
            a.close().await;
            b.close().await;
            spectator.close().await;
            restarted.stop().await;
        });
    }
}

fn copy_directory(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let destination = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &destination);
        } else {
            std::fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn change_package(root: &Path, version: &str) {
    let directory = root.join("shahai-swd-upstream-noname");
    let cards_path = directory.join("assets/cards.json");
    let mut cards: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&cards_path).unwrap()).unwrap();
    let original_rank = cards["cards"][0]["rank"].as_u64().unwrap();
    cards["cards"][0]["rank"] = serde_json::json!(original_rank % 13 + 1);
    cards["cards"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "name": "swd_ws_upgraded_hufu", "kind": "hufu", "suit": "club", "rank": 13
        }));
    let cards = serde_json::to_vec_pretty(&cards).unwrap();
    std::fs::write(cards_path, &cards).unwrap();
    let manifest_path = directory.join("package.npack.json");
    let mut package =
        content_schema::ContentPackage::from_json_bytes(&std::fs::read(&manifest_path).unwrap())
            .unwrap();
    assert_eq!(package.manifest.version, "1.0.0");
    package.manifest.version = version.into();
    let asset = package
        .asset_paths
        .iter()
        .find(|(_, path)| path.as_str() == "assets/cards.json")
        .unwrap()
        .0
        .clone();
    package
        .asset_manifest
        .as_mut()
        .unwrap()
        .entries
        .iter_mut()
        .find(|entry| entry.asset_id == asset)
        .unwrap()
        .source_hash = *blake3::hash(&cards).as_bytes();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package.validate().unwrap();
    std::fs::write(manifest_path, package.to_json_bytes().unwrap()).unwrap();
}

fn upgrade_packages(cfg: &ServerConfig, version: &str) {
    let original = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    for package in ["shahai-swd-upstream-noname", "wudian-upstream-noname"] {
        let target = cfg.native_packages_root.join(package);
        std::fs::remove_dir_all(&target).unwrap();
        copy_directory(&original.join(package), &target);
    }
    change_package(&cfg.native_packages_root, version);
    let content = super::native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .unwrap();
    content.import().unwrap();
    let lock = content_runtime::PackageStore::open(&cfg.native_store_root)
        .unwrap()
        .current_lock()
        .unwrap();
    assert!(lock
        .packages
        .iter()
        .any(|package| package.package_id == "shahai-swd" && package.version == version));
}

#[test]
fn native_extensions_ws_fails_closed_for_invalid_frozen_content() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let (room, token, record) = runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        let mut a = Peer::connect(&server).await;
        a.create().await;
        let room = a.room;
        let token = a.token.clone();
        let record_path = Path::new(&cfg.native_room_store).join(format!("{room}.json"));
        let record: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
        assert_eq!(
            record["state"]["frozen_packages"].as_array().unwrap().len(),
            2
        );
        a.close().await;
        server.stop().await;
        (room, token, record)
    });

    for failure in [
        "package-tampered",
        "asset-tampered",
        "asset-missing",
        "copy-missing",
        "wrong-lock",
        "old-lock-without-copy",
        "revoked-hash",
        "revoked-slot",
    ] {
        let base = directory.path().join(failure);
        let has_active = matches!(
            failure,
            "package-tampered" | "asset-tampered" | "asset-missing" | "copy-missing" | "wrong-lock"
        );
        let mut variant = if has_active {
            local_config(&base)
        } else {
            config(&base)
        };
        if !has_active {
            variant.native_packages_root = base.join("packages");
            std::fs::create_dir_all(&variant.native_packages_root).unwrap();
        }
        let mut damaged = record.clone();
        let copies = damaged["state"]["frozen_packages"].as_array_mut().unwrap();
        let binding = copies[0]["binding"].clone();
        match failure {
            "package-tampered" => {
                let json = copies[0]["package_json"].as_str().unwrap().to_owned() + " ";
                copies[0]["package_json"] = serde_json::json!(json);
            }
            "asset-tampered" => {
                let bytes = copies[0]["assets"]
                    .as_object_mut()
                    .unwrap()
                    .values_mut()
                    .next()
                    .unwrap()
                    .as_array_mut()
                    .unwrap();
                bytes[0] = serde_json::json!(bytes[0].as_u64().unwrap() ^ 1);
            }
            "asset-missing" => {
                copies[0]["assets"] = serde_json::json!({});
            }
            "copy-missing" => {
                copies.clear();
            }
            "wrong-lock" => {
                let mut hash = copies[0]["binding"]["content_hash"]
                    .as_str()
                    .unwrap()
                    .to_owned();
                hash.replace_range(..1, if &hash[..1] == "0" { "1" } else { "0" });
                copies[0]["binding"]["content_hash"] = serde_json::json!(hash);
            }
            "old-lock-without-copy" => {
                damaged["state"]
                    .as_object_mut()
                    .unwrap()
                    .remove("frozen_packages");
            }
            "revoked-hash" | "revoked-slot" => {
                let store =
                    content_runtime::PackageStore::open(&variant.native_store_root).unwrap();
                if failure == "revoked-hash" {
                    let hash = binding["content_hash"].as_str().unwrap();
                    store
                        .revoke_content_hash(hash, "WS frozen regression", true)
                        .unwrap();
                } else {
                    store
                        .revoke_slot(
                            binding["namespace"].as_str().unwrap(),
                            binding["package_id"].as_str().unwrap(),
                            "WS frozen regression",
                            true,
                        )
                        .unwrap();
                }
            }
            _ => unreachable!(),
        }
        let record_path = Path::new(&variant.native_room_store).join(format!("{room}.json"));
        // 内容策略反例须先通过存储完整性，避免仅因缺少提交日志而被拒绝。
        let storage = super::native_room_store::NativeRoomStore::new(&variant.native_room_store);
        storage.save(&serde_json::from_value(damaged).unwrap()).unwrap();
        assert_eq!(storage.load_all().unwrap().len(), 1, "storage-valid {failure}");
        let bytes = std::fs::read(&record_path).unwrap();
        runtime().block_on(async {
            let unavailable = Server::start(&variant, 0).await;
            assert_eq!(
                std::fs::read(&record_path).unwrap(),
                bytes,
                "failed restore must preserve the save ({failure})"
            );
            let mut peer = Peer::connect(&unavailable).await;
            peer.send(ClientMessage::JoinRoom {
                room_id: room,
                password: PASSWORD.into(),
                preferred_seat: Some(0),
                reconnect_token: Some(token.clone()),
            })
            .await;
            peer.error(ErrorCode::RoomNotFound).await;
            assert_eq!(
                unavailable.lobby.stats().active_rooms,
                0,
                "no fallback room"
            );
            peer.send(ClientMessage::CreateRoom {
                choice_timeout_secs: None,
                players: 2,
                seed: SEED,
                password: PASSWORD.into(),
            })
            .await;
            if has_active {
                peer.join_result().await;
                assert_ne!(
                    peer.room, room,
                    "valid active copies may create a new room, never replace a broken frozen save"
                );
            } else {
                peer.error(ErrorCode::Internal).await;
                assert_eq!(
                    unavailable.lobby.stats().active_rooms,
                    0,
                    "a frozen save must not activate its packages for a new room"
                );
            }
            assert_eq!(std::fs::read(&record_path).unwrap(), bytes);
            println!("NATIVE_WS frozen_fail_closed={failure} saved_record=preserved");
            peer.close().await;
            unavailable.stop().await;
        });
    }
}

/// P1 回归（真 TCP/WS）：旧档无收据的重放必须受控拒绝，不能推进 revision。
#[test]
fn native_ws_replay_without_receipt_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        let mut a = Peer::connect(&server).await;
        let initial = a.create().await;
        let before = initial.view.revision;
        let (result, state) = a
            .submit_replay(a.player, before, 7_777, CommandKind::EndPhase)
            .await;
        assert!(matches!(
            result,
            CommandResult::Rejected {
                reason: engine_api::RejectReason::IllegalAction { ref code }
            } if code == "legacy_replay_no_receipt"
        ));
        assert_eq!(state.view.revision, before, "拒绝不得推进 revision");
    });
}

/// P1 回归（真 TCP/WS）：一个停止读取的旁观者让出站队列饱和后，房间 actor
/// 仍必须继续处理其他玩家的命令；慢连接被断开并可重新同步。
#[test]
fn native_extensions_ws_slow_spectator_does_not_block_commands() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    runtime().block_on(async {
        // Bound both TCP buffers: an unread loopback socket otherwise fits all broadcasts
        // on some OSes, regardless of an artificial delay between writer sends.
        let listener = tokio::net::TcpSocket::new_v4().unwrap();
        listener.set_send_buffer_size(1024).unwrap();
        let send_buffer = listener.send_buffer_size().unwrap();
        assert!(send_buffer <= 64 * 1024, "unexpected TCP send buffer: {send_buffer}");
        listener.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let listener = listener.listen(128).unwrap();
        let address = listener.local_addr().unwrap();
        let server = Server::start_on(&cfg, 0, listener).await;
        let mut a = Peer::connect(&server).await;
        let initial = a.create().await;
        let mut b = Peer::connect(&server).await;
        b.join(a.room, 1, None).await;
        // B 只是占座；持续读掉广播，避免它自己成为慢连接干扰断言。
        let b_reader = tokio::spawn(async move {
            let mut outcomes = 0;
            let mut text_bytes = 0;
            while let Some(frame) = b.ws.next().await {
                match frame.unwrap() {
                    Message::Text(text) => {
                        text_bytes += text.len();
                        let message = decode_s2c(text.as_ref()).unwrap();
                        if let ServerMessage::CommandOutcome { idempotency_key, .. } = &message {
                            outcomes += 1;
                            State::from_message(message.clone()).assert_player(PlayerId::new(2));
                            if idempotency_key.0 == 9999 {
                                return (outcomes, text_bytes);
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            panic!("reading player disconnected before the accepted command");
        });
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        socket.set_recv_buffer_size(1024).unwrap();
        let recv_buffer = socket.recv_buffer_size().unwrap();
        assert!(recv_buffer <= 64 * 1024, "unexpected TCP receive buffer: {recv_buffer}");
        let stream = socket.connect(address).await.unwrap();
        let (ws, response) = tokio_tungstenite::client_async(
            &server.url,
            MaybeTlsStream::Plain(stream),
        ).await.unwrap();
        assert_eq!(response.status().as_u16(), 101);
        let mut slow = Peer::from_ws(ws).await;
        slow.spectate(a.room).await.assert_spectator();
        let token = slow.token.clone();
        let store = super::native_room_store::NativeRoomStore::new(&cfg.native_room_store);
        let original_member = store.load_all().unwrap()[0].state.members.iter()
            .find(|member| member.reconnect_token == token).unwrap().connection_id;
        // From here until the close check, the real Spectator socket is never read.
        let mut revision = initial.view.revision;
        for key in 1..=300u64 {
            let (result, state) = tokio::time::timeout(
                Duration::from_secs(3),
                a.submit(
                    a.player,
                    revision,
                    key,
                    CommandKind::PlayCard {
                        card: engine_api::CardId::new(999_999),
                        targets: Vec::new(),
                        card_targets: Vec::new(),
                    },
                ),
            )
            .await
            .unwrap_or_else(|_| panic!("慢旁观者阻塞了第 {key} 条命令"));
            assert!(
                matches!(result, CommandResult::Rejected { .. }),
                "无效牌应被拒绝，实得 {result:?}"
            );
            revision = state.view.revision;
            // A 每命令还会收到名册广播；用短等待排空，保持其队列不因测试
            // 自身的读取节奏而饱和。
            loop {
                match tokio::time::timeout(Duration::from_millis(2), a.ws.next()).await {
                    Err(_) => break,
                    Ok(Some(Ok(Message::Text(_)))) => {}
                    other => panic!("active player must remain connected: {other:?}"),
                }
            }
        }
        let (accepted, after) = tokio::time::timeout(
            Duration::from_secs(3),
            a.submit(a.player, revision, 9999, CommandKind::EndPhase),
        )
        .await
        .expect("慢连接断开后当前玩家仍能推进");
        assert!(
            matches!(accepted, CommandResult::Accepted { .. }),
            "{accepted:?}"
        );
        // Missing broadcasts plus a real close prove saturation; starting to read a
        // merely buffered socket here must not turn the test green.
        let mut outcomes = 0;
        let mut wire_bytes = 0;
        tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(frame) = slow.ws.next().await {
                match frame {
                    Ok(Message::Close(_)) | Err(_) => break,
                    Ok(Message::Text(text)) => {
                        wire_bytes += text.len();
                        let message = decode_s2c(text.as_ref()).unwrap();
                        if matches!(message, ServerMessage::CommandOutcome { .. }) {
                            State::from_message(message).assert_spectator();
                            outcomes += 1;
                        }
                    }
                    Ok(_) => {}
                }
            }
        })
        .await
        .expect("慢连接应在期限内被断开");
        assert!(outcomes > 0 && outcomes < 301,
            "backpressure must drop broadcasts before reading resumes: {outcomes}");

        let mut reconnected = Peer::connect(&server).await;
        let restored = reconnected.join(a.room, 0, Some(token.clone())).await;
        restored.assert_spectator();
        assert_eq!(restored.view.revision, after.view.revision);
        assert_eq!(restored.view.state_hash, after.view.state_hash);
        assert_eq!(reconnected.snapshot().await, restored);
        let records = store.load_all().unwrap();
        let members = &records[0].state.members;
        assert_eq!(members.len(), 3, "reconnect must reclaim the old member");
        assert!(!members.iter().any(|member| member.connection_id == original_member));
        assert_eq!(members.iter().filter(|member| member.reconnect_token == token).count(), 1);
        reconnected.close().await;
        let (healthy_outcomes, healthy_bytes) = tokio::time::timeout(Duration::from_secs(3), b_reader)
            .await.expect("reading player receives all commands").unwrap();
        assert_eq!(healthy_outcomes, 301);
        assert!(healthy_bytes < 1024 * 1024, "broadcast load stays below 1 MiB");
        println!("NATIVE_WS slow_spectator=disconnected commands=301 send_buffer={send_buffer} recv_buffer={recv_buffer} buffered_outcomes={outcomes} text_bytes={wire_bytes} healthy_outcomes={healthy_outcomes} healthy_bytes={healthy_bytes} reconnect=pass members=3");
        server.stop().await;
    });
}
