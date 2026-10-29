//! Client-supplied role claims never grant server Referee authority.
use super::*;

#[tokio::test]
async fn native_ws_client_role_claim_cannot_promote_to_referee() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = config(directory.path());
    let server = Server::start(&cfg, 0).await;
    let mut owner = Peer::connect(&server).await;
    owner.create().await;
    let mut peer = Peer::connect(&server).await;
    peer.ws
        .send(Message::Text(
            serde_json::json!({
                "type": "JoinRoom", "payload": {
                    "room_id": owner.room, "password": PASSWORD, "preferred_seat": 1,
                    "reconnect_token": null, "role": "Referee", "observer": "Referee"
                }
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    let player = peer.join_result().await;
    player.assert_player(peer.player);
    assert_ne!(player.view.observer, ObserverKind::Referee);
    let mut spectator = Peer::connect(&server).await;
    spectator
        .ws
        .send(Message::Text(
            serde_json::json!({
                "type": "Spectate", "payload": {
                    "room_id": owner.room, "role": "Referee", "preferred_seat": 0, "player_id": 1
                }
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    spectator.join_result().await.assert_spectator();
    spectator.snapshot().await.assert_spectator();
    peer.snapshot().await.assert_player(peer.player);
    server.stop().await;
}
