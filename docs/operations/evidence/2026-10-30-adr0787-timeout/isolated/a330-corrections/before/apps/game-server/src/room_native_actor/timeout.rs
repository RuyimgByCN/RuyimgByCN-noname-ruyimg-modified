//! One automatic command per scheduling turn, with the same durable transaction as people.
use super::*;

pub(super) async fn automatic_step(
    id: u64,
    session: &mut NativeRoomSession,
    durability: &mut RoomDurability,
    members: &mut HashMap<u64, NativeActorMember>,
    snapshot_every: u64,
    candidate: (u8, engine_native::GameAction, bool),
) -> Result<(), String> {
    let before = durability
        .checkpoint(session)?
        .or_else(|| Some(session.checkpoint()));
    let (seat, action, takeover) = candidate;
    match session.submit_automatic(seat, action, takeover) {
        Ok(result @ CommandResult::Accepted { .. }) => {
            session.compact_wal(snapshot_every);
            durability.commit(session, before)?;
            let public_event_codes = session.take_public_event_codes();
            let mut stalled = Vec::new();
            for member in members.values() {
                let Some(out) = &member.outbound else {
                    continue;
                };
                let role = member.role.into();
                let player = (member.role == ClientRole::Player).then_some(member.player_id);
                let message = ServerMessage::CommandOutcome {
                    header: MessageHeader::new(id, 0, session.engine().revision()),
                    // No player receipt is created for this unsolicited authoritative update.
                    idempotency_key: IdempotencyKey(0),
                    result: result.clone(),
                    view: observer_snapshot_for(session.engine(), role, player),
                    choice: session.choice_for(role, player),
                    legal_actions: player
                        .map(|p| legal_actions_for(session.engine(), p))
                        .unwrap_or_default(),
                    public_event_codes: public_event_codes.clone(),
                };
                if out.try_send(&message) == SendStatus::Full {
                    stalled.push(member.connection_id);
                }
            }
            drop_stalled(members, stalled);
            broadcast_roster_native(id, members, session).await;
            Ok(())
        }
        other => {
            if let Some(before) = before {
                session.rollback(before);
            }
            match other {
                Ok(_) => Err("automatic command rejected; window paused".into()),
                Err(error) => Err(error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_room_store::{NativeRoomStore, StoreFault};

    fn pending() -> NativeRoomSession {
        let mut session = NativeRoomSession::new(engine_native::Engine::new(4242, 4), 4);
        for _ in 0..1000 {
            if session.engine().open_choice().is_some() {
                return session;
            }
            let seat = session.engine().turn().current;
            let action = session
                .engine()
                .legal_actions(seat)
                .into_iter()
                .next()
                .unwrap();
            assert!(matches!(
                session
                    .submit_native(seat, action, session.engine().revision())
                    .unwrap(),
                CommandResult::Accepted { .. }
            ));
        }
        panic!("real choice required");
    }

    #[tokio::test]
    async fn automatic_storage_faults_rollback_engine_control_deadline_wal_and_dedup() {
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
            for compensation in [None, Some(StoreFault::PrepareSync)] {
                if compensation.is_some()
                    && !matches!(
                        fault,
                        StoreFault::DecisionWrite
                            | StoreFault::DecisionSync
                            | StoreFault::DecisionDirectorySync
                    )
                {
                    continue;
                }
                let mut session = pending();
                let dir = tempfile::tempdir().unwrap();
                let store = NativeRoomStore::new(dir.path());
                let mut durability = RoomDurability::new(787, String::new(), Some(store.clone()));
                durability.commit(&mut session, None).unwrap();
                let confirmed = crate::native_room_store::state_checksum(
                    &session.export_state(&session.snapshot_hex()),
                );
                let deadline = session
                    .timeout_state()
                    .deadline
                    .as_ref()
                    .unwrap()
                    .deadline_unix_ms;
                let candidate = session.automatic_command(deadline, true).unwrap();
                match compensation {
                    Some(other) => store.fail_next_write_with_compensation(fault, other),
                    None => store.fail_next_write(fault),
                }
                assert!(
                    automatic_step(
                        787,
                        &mut session,
                        &mut durability,
                        &mut HashMap::new(),
                        1000,
                        candidate
                    )
                    .await
                    .is_err(),
                    "{fault:?}/{compensation:?}"
                );
                assert_eq!(
                    crate::native_room_store::state_checksum(
                        &session.export_state(&session.snapshot_hex())
                    ),
                    confirmed,
                    "{fault:?}/{compensation:?}"
                );
                assert!(session.timeout_state().managed_seats.is_empty());
                for record in NativeRoomStore::new(dir.path()).load_all().unwrap() {
                    NativeRoomSession::from_saved(&record.state, vec![]).unwrap();
                }
            }
        }
    }
}
