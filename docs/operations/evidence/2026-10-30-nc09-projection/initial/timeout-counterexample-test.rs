// Temporarily appended to nc09_native_room_session.rs, removed after capturing red.
#[tokio::test]
async fn nc09_timeout_default_deadline_answers_pending_choice() {
    let room = session_with_choice();
    let choice = room.engine().open_choice().unwrap().clone();
    let revision = room.engine().revision();
    let handle = room_native_actor::spawn_room_native(98, room, 8, 4);
    let (tx, mut rx) = mpsc::channel(32);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Join {
        connection_id: 1,
        display_name: "timeout-observer".into(),
        preferred_seat: Some(choice.observer),
        reconnect_token: None,
        password: String::new(),
        outbound: session::Outbound::new(tx),
        reply,
    }).await.unwrap();
    assert!(response.await.unwrap().unwrap().choice.is_some());
    let answer = tokio::time::timeout(std::time::Duration::from_secs(31), async {
        while let Some(message) = rx.recv().await {
            if let engine_protocol::ServerMessage::CommandOutcome { view, .. } =
                serde_json::from_str(&message).unwrap()
            {
                if view.revision > revision { return; }
            }
        }
        panic!("actor closed before the choice was answered");
    }).await;
    assert!(answer.is_ok(), "default 30-second room timeout must submit AI answer through ordinary actor command flow");
}
