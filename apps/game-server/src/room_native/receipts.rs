//! 接受结果与候选一起保存；连接号或座位重用不能继承其他身份的请求收据。

use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeCommandReceipt {
    pub reconnect_token: String,
    pub player_id: PlayerId,
    pub idempotency_key: u64,
    pub kind: CommandKind,
    pub result: CommandResult,
}

/// 每会话保留的已接受命令收据上限；长局/高频重连下防止无界增长。
const MAX_COMMAND_RECEIPTS: usize = 4096;

#[derive(Clone, Default)]
pub(super) struct CommandReceipts(BTreeMap<(String, u64), NativeCommandReceipt>);

impl CommandReceipts {
    pub(super) fn restore(
        entries: &[NativeCommandReceipt],
        revision: u64,
        max_seats: u8,
    ) -> Result<Self, String> {
        let mut receipts = Self::default();
        for entry in entries {
            if entry.reconnect_token.is_empty()
                || entry.idempotency_key == 0
                || entry.player_id.raw() == 0
                || entry.player_id.raw() > max_seats as u32
                || !matches!(entry.result, CommandResult::Accepted { new_revision, .. } if new_revision <= revision)
            {
                return Err("落盘命令收据身份、键或修订无效".into());
            }
            let key = (entry.reconnect_token.clone(), entry.idempotency_key);
            if receipts.0.insert(key, entry.clone()).is_some() {
                return Err("落盘命令收据重复".into());
            }
        }
        receipts.prune();
        Ok(receipts)
    }

    fn prune(&mut self) {
        while self.0.len() > MAX_COMMAND_RECEIPTS {
            let oldest = self
                .0
                .iter()
                .filter_map(|(key, receipt)| match receipt.result {
                    CommandResult::Accepted { new_revision, .. } => Some((key.clone(), new_revision)),
                    _ => None,
                })
                .min_by_key(|(_, revision)| *revision)
                .map(|(key, _)| key);
            match oldest {
                Some(key) => {
                    self.0.remove(&key);
                }
                None => break,
            }
        }
    }

    pub(super) fn entries(&self) -> Vec<NativeCommandReceipt> {
        self.0.values().cloned().collect()
    }
}

impl NativeRoomSession {
    pub(crate) fn command_receipt(
        &self,
        connection_id: u64,
        key: u64,
        kind: &CommandKind,
    ) -> Result<Option<CommandResult>, String> {
        if key == 0 {
            return Ok(None);
        }
        let member = self
            .members
            .get(&connection_id)
            .ok_or("unknown connection")?;
        let Some(receipt) = self
            .command_receipts
            .0
            .get(&(member.reconnect_token.clone(), key))
        else {
            return Ok(None);
        };
        if member.role != ClientRole::Player
            || receipt.player_id != member.player_id
            || receipt.kind != *kind
        {
            return Err("idempotency key conflicts with the saved command receipt".into());
        }
        Ok(Some(receipt.result.clone()))
    }

    pub(crate) fn remember_command(
        &mut self,
        connection_id: u64,
        key: u64,
        kind: &CommandKind,
        result: &CommandResult,
    ) -> Result<(), String> {
        if key == 0 {
            return Ok(());
        }
        let member = self
            .members
            .get(&connection_id)
            .ok_or("unknown connection")?;
        if member.role != ClientRole::Player || !matches!(result, CommandResult::Accepted { .. }) {
            return Err("only accepted player commands have receipts".into());
        }
        self.command_receipts.0.insert(
            (member.reconnect_token.clone(), key),
            NativeCommandReceipt {
                reconnect_token: member.reconnect_token.clone(),
                player_id: member.player_id,
                idempotency_key: key,
                kind: kind.clone(),
                result: result.clone(),
            },
        );
        self.command_receipts.prune();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted(session: &mut NativeRoomSession, key: u64) -> CommandResult {
        let result = session.submit(1, &CommandKind::EndPhase, 0).unwrap();
        assert!(matches!(result, CommandResult::Accepted { .. }));
        session
            .remember_command(1, key, &CommandKind::EndPhase, &result)
            .unwrap();
        result
    }

    #[test]
    fn accepted_receipt_survives_restore_and_rebinding_without_executing_again() {
        let mut session = NativeRoomSession::new(Engine::new(4242, 2), 2);
        let token = session
            .join(1, Some(0), None)
            .unwrap()
            .member
            .reconnect_token;
        let result = accepted(&mut session, 91);
        let state = session.export_state(&session.snapshot_hex());
        let mut restored = NativeRoomSession::from_saved(&state, vec![]).unwrap();
        restored.join(10, Some(0), Some(token)).unwrap();
        assert_eq!(
            restored
                .command_receipt(10, 91, &CommandKind::EndPhase)
                .unwrap(),
            Some(result)
        );
        assert_eq!(restored.engine().revision(), 1);
        assert!(restored
            .command_receipt(
                10,
                91,
                &CommandKind::StartGame {
                    rng_seed: 4242,
                    player_count: 2,
                    ruleset: String::new()
                }
            )
            .is_err());
    }

    #[test]
    fn rollback_drops_candidate_receipt_and_reclaimed_seat_does_not_inherit_it() {
        let mut session = NativeRoomSession::new(Engine::new(4242, 2), 2);
        session.join(1, Some(0), None).unwrap();
        let before = session.checkpoint();
        accepted(&mut session, 91);
        session.rollback(before);
        assert!(session
            .command_receipt(1, 91, &CommandKind::EndPhase)
            .unwrap()
            .is_none());
        accepted(&mut session, 91);
        session.reclaim_seat(1);
        session.join(2, Some(0), None).unwrap();
        assert!(session
            .command_receipt(2, 91, &CommandKind::EndPhase)
            .unwrap()
            .is_none());
    }

    #[test]
    fn restore_prunes_receipts_above_cap() {
        let entries: Vec<_> = (1..=MAX_COMMAND_RECEIPTS as u64 + 2)
            .map(|key| NativeCommandReceipt {
                reconnect_token: "token".into(),
                player_id: PlayerId::new(1),
                idempotency_key: key,
                kind: CommandKind::EndPhase,
                result: CommandResult::Accepted {
                    new_revision: key,
                    state_hash: [0; 32],
                },
            })
            .collect();
        let receipts =
            CommandReceipts::restore(&entries, MAX_COMMAND_RECEIPTS as u64 + 2, 2).unwrap();
        assert_eq!(receipts.0.len(), MAX_COMMAND_RECEIPTS);
        assert!(!receipts.0.contains_key(&("token".into(), 1)));
        assert!(receipts
            .0
            .contains_key(&("token".into(), MAX_COMMAND_RECEIPTS as u64 + 2)));
    }

    #[test]
    fn receipts_are_bounded_and_keep_newest_revisions() {
        let mut session = NativeRoomSession::new(Engine::new(4242, 2), 2);
        session.join(1, Some(0), None).unwrap();
        let total = MAX_COMMAND_RECEIPTS + 3;
        for key in 1..=total as u64 {
            session
                .remember_command(
                    1,
                    key,
                    &CommandKind::EndPhase,
                    &CommandResult::Accepted {
                        new_revision: key,
                        state_hash: [0; 32],
                    },
                )
                .unwrap();
        }
        assert_eq!(session.command_receipts.0.len(), MAX_COMMAND_RECEIPTS);
        assert!(session
            .command_receipt(1, 1, &CommandKind::EndPhase)
            .unwrap()
            .is_none());
        assert!(session
            .command_receipt(1, total as u64, &CommandKind::EndPhase)
            .unwrap()
            .is_some());
    }

    #[test]
    fn malformed_receipts_fail_restore_and_old_empty_state_keeps_its_json_shape() {
        let mut session = NativeRoomSession::new(Engine::new(4242, 2), 2);
        session.join(1, Some(0), None).unwrap();
        let empty = session.export_state(&session.snapshot_hex());
        assert!(serde_json::to_value(&empty)
            .unwrap()
            .get("command_receipts")
            .is_none());
        accepted(&mut session, 91);
        let state = session.export_state(&session.snapshot_hex());
        for mutation in 0..6 {
            let mut invalid = state.clone();
            let receipt = &mut invalid.command_receipts[0];
            match mutation {
                0 => receipt.idempotency_key = 0,
                1 => receipt.reconnect_token.clear(),
                2 => receipt.player_id = PlayerId::NONE,
                3 => receipt.player_id = PlayerId::new(3),
                4 => {
                    receipt.result = CommandResult::Accepted {
                        new_revision: 99,
                        state_hash: [0; 32],
                    }
                }
                5 => invalid
                    .command_receipts
                    .push(invalid.command_receipts[0].clone()),
                _ => unreachable!(),
            }
            assert!(
                NativeRoomSession::from_saved(&invalid, vec![]).is_err(),
                "mutation {mutation}"
            );
        }
    }
}
