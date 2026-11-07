//! 持久化成功才发布候选状态；rename 后同步失败先恢复最后确认状态再允许写入。

use crate::native_room_store::{NativeRoomStore, StoreWriteError, StoredRoom};
use crate::room_native::{NativeRoomCheckpoint, NativeRoomSession};

pub(super) struct RoomDurability {
    id: u64,
    password_hash: String,
    store: Option<NativeRoomStore>,
    last_sig: Option<String>,
    uncertain: bool,
    requires_restart: bool,
    last_step_failed: bool,
}

impl RoomDurability {
    pub(super) fn new(id: u64, password_hash: String, store: Option<NativeRoomStore>) -> Self {
        Self {
            id,
            password_hash,
            store,
            last_sig: None,
            uncertain: false,
            requires_restart: false,
            last_step_failed: false,
        }
    }

    pub(super) fn checkpoint(
        &mut self,
        session: &NativeRoomSession,
    ) -> Result<Option<NativeRoomCheckpoint>, String> {
        if self.requires_restart {
            return Err(
                "native room persistence failed: commit decision unresolved; restart required"
                    .into(),
            );
        }
        if self.uncertain {
            // 已替换的候选可能留在磁盘；成功重写 confirmed 状态前禁止继续变更。
            self.last_sig = None;
            if let Err(error) = self.save(session) {
                self.requires_restart |= error.decision_uncertain;
                self.uncertain |= error.replacement_visible;
                return Err(format!("native room persistence failed: {error}"));
            }
            self.uncertain = false;
        }
        Ok(Some(session.checkpoint()))
    }

    pub(super) fn commit(
        &mut self,
        session: &mut NativeRoomSession,
        before: Option<NativeRoomCheckpoint>,
    ) -> Result<(), String> {
        if let Err(error) = self.save(session) {
            // 双重失败可能已留下 Committed；旧 RAM 不能再作为下一笔写入的依据。
            self.requires_restart |= error.decision_uncertain;
            if let Some(before) = before {
                session.rollback(before);
            }
            self.uncertain |= error.replacement_visible;
            self.last_step_failed = true;
            tracing::warn!(room = self.id, error = %error, "原生房间状态未确认持久化");
            return Err(format!("native room persistence failed: {error}"));
        }
        self.last_step_failed = false;
        Ok(())
    }

    pub(super) fn commit_step(
        &mut self,
        session: &mut NativeRoomSession,
        before: Option<NativeRoomCheckpoint>,
    ) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let rev = session.engine().revision();
        let match_finished = session.engine().outcome().is_some();
        let full_checkpoint = match_finished
            || !store.room_path(self.id).exists()
            || store.has_armed_fault()
            || self.last_step_failed
            || rev == 0
            || rev % crate::native_room_store::DEFAULT_WAL_CHECKPOINT_EVERY == 0
            || session.last_wal_entry().is_none()
            || (!session.pack_refs().is_empty() && session.engine().open_choice().is_some());
        if full_checkpoint {
            return self.commit(session, before);
        }
        let line = session.wal_segment_line();
        if let Err(error) = store.append_wal_segment(self.id, &line) {
            if let Some(before) = before {
                session.rollback(before);
            }
            self.last_step_failed = true;
            tracing::warn!(room = self.id, error = %error, "原生房间 WAL 段追加未确认");
            return Err(format!("native room persistence failed: {error}"));
        }
        self.last_sig = None;
        self.last_step_failed = false;
        Ok(())
    }

    fn save(&mut self, session: &NativeRoomSession) -> Result<(), StoreWriteError> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let state = session.export_state(&session.snapshot_hex());
        let sig = crate::native_room_store::state_checksum(&state);
        if self.last_sig.as_ref() == Some(&sig) {
            return Ok(());
        }
        store.save_durable(&StoredRoom {
            room_id: self.id,
            password_hash: self.password_hash.clone(),
            state,
            checksum: String::new(),
        })?;
        self.last_sig = Some(sig);
        if session.engine().outcome().is_some() {
            // 终局先落盘，删除失败时启动扫描仍识别为已结束，不能复活旧快照。
            if let Err(error) = store.remove(self.id) {
                tracing::warn!(room = self.id, error = %error, "原生房间终局清理失败（已保存终局）");
            }
        }
        Ok(())
    }
}
