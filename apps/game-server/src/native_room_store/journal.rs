//! 永久撤销依据与显式提交决定；完整候选并不等于已提交记录。

use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
struct DiskRoom {
    #[serde(flatten)]
    record: StoredRoom,
    #[serde(default)]
    commit_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Phase {
    Prepared,
    Committed,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Journal {
    version: u8,
    room_id: u64,
    commit_id: String,
    candidate_hash: String,
    previous: Option<String>,
    phase: Phase,
    checksum: String,
}

impl Journal {
    pub(super) fn prepared(
        room_id: u64,
        commit_id: String,
        candidate: &[u8],
        previous: Option<Vec<u8>>,
    ) -> Result<Self, String> {
        let mut journal = Self {
            version: 1,
            room_id,
            commit_id,
            candidate_hash: blake3::hash(candidate).to_hex().to_string(),
            previous: previous
                .map(String::from_utf8)
                .transpose()
                .map_err(|e| format!("原确认记录不是 UTF-8: {e}"))?,
            phase: Phase::Prepared,
            checksum: String::new(),
        };
        journal.checksum = journal.digest();
        Ok(journal)
    }

    pub(super) fn committed(&self) -> Self {
        let mut committed = self.clone();
        committed.phase = Phase::Committed;
        committed.checksum = committed.digest();
        committed
    }

    fn digest(&self) -> String {
        let mut unsigned = self.clone();
        unsigned.checksum.clear();
        blake3::hash(&serde_json::to_vec(&unsigned).expect("日志仅包含 JSON 数据"))
            .to_hex()
            .to_string()
    }

    fn validate(&self, room_id: u64) -> Result<(), String> {
        if self.version != 1 || room_id == 0 || self.room_id != room_id {
            return Err("提交日志版本或房间号不匹配".into());
        }
        if self.digest() != self.checksum {
            return Err("提交日志校验和不匹配".into());
        }
        validate_commit_id(&self.commit_id)?;
        if self.candidate_hash.len() != 64
            || !self.candidate_hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("提交日志候选哈希格式错误".into());
        }
        if let Some(previous) = &self.previous {
            decode(previous.as_bytes(), room_id)?;
        }
        Ok(())
    }
}

fn validate_commit_id(id: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(id).map_err(|e| format!("提交标识格式错误: {e}"))?;
    if parsed.to_string() != id {
        return Err("提交标识不是规范 UUID".into());
    }
    Ok(())
}

pub(super) fn encode(record: &StoredRoom, commit_id: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(&DiskRoom {
        record: record.clone(),
        commit_id: Some(commit_id.into()),
    })
    .map_err(|e| format!("序列化失败: {e}"))
}

fn disk_room(bytes: &[u8], room_id: u64) -> Result<DiskRoom, String> {
    let stored: DiskRoom = serde_json::from_slice(bytes).map_err(|e| format!("解析失败: {e}"))?;
    if stored.record.room_id != room_id {
        return Err(format!(
            "文件名房间号 {room_id} 与记录 {} 不符",
            stored.record.room_id
        ));
    }
    if let Some(commit_id) = &stored.commit_id {
        validate_commit_id(commit_id)?;
        if stored.record.checksum.is_empty() {
            return Err("新格式房间记录缺少校验和".into());
        }
    }
    if !stored.record.checksum.is_empty()
        && state_checksum(&stored.record.state) != stored.record.checksum
    {
        return Err("校验和不匹配".into());
    }
    Ok(stored)
}

pub(super) fn decode(bytes: &[u8], room_id: u64) -> Result<StoredRoom, String> {
    Ok(disk_room(bytes, room_id)?.record)
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_file() => std::fs::read(path)
            .map(Some)
            .map_err(|e| format!("读取 {} 失败: {e}", path.display())),
        Ok(_) => Err(format!("{} 不是普通文件", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("读取 {} 类型失败: {e}", path.display())),
    }
}

/// 返回原确认字节以及是否存在尚未提交的候选；不修改任何恢复证据。
pub(super) fn confirmed_bytes(dir: &Path, room_id: u64) -> Result<(Option<Vec<u8>>, bool), String> {
    let candidate = read_optional(&dir.join(format!("{room_id}.json")))?;
    let Some(bytes) = read_optional(&dir.join(format!("{room_id}.json.journal")))? else {
        if let Some(candidate) = &candidate {
            if disk_room(candidate, room_id)?.commit_id.is_some() {
                return Err("新格式房间记录缺少提交日志，拒绝按旧格式恢复".into());
            }
        }
        return Ok((candidate, false));
    };
    let journal: Journal =
        serde_json::from_slice(&bytes).map_err(|e| format!("解析提交日志失败: {e}"))?;
    journal.validate(room_id)?;
    match journal.phase {
        Phase::Prepared => {
            if let Some(candidate) = &candidate {
                let is_previous =
                    journal.previous.as_deref().map(str::as_bytes) == Some(candidate.as_slice());
                if !is_previous {
                    if let Ok(stored) = serde_json::from_slice::<DiskRoom>(candidate) {
                        if stored.commit_id.as_deref() != Some(&journal.commit_id) {
                            return Err("候选标识与撤销日志不匹配，拒绝使用陈旧日志".into());
                        }
                    }
                }
            }
            Ok((journal.previous.map(String::into_bytes), true))
        }
        Phase::Committed => {
            let candidate = candidate.ok_or("已提交候选缺失，拒绝回退旧局")?;
            let stored = disk_room(&candidate, room_id)?;
            if stored.commit_id.as_deref() != Some(&journal.commit_id)
                || blake3::hash(&candidate).to_hex().as_str() != journal.candidate_hash
            {
                return Err("已提交候选与提交日志标识或完整哈希不匹配".into());
            }
            Ok((Some(candidate), false))
        }
    }
}

pub(super) fn reserve_payload_ids(bytes: &[u8], ids: &mut BTreeSet<u64>) {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(id) = value.get("room_id").and_then(serde_json::Value::as_u64) {
            ids.insert(id);
        }
        if let Some(previous) = value.get("previous").and_then(serde_json::Value::as_str) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(previous) {
                if let Some(id) = value.get("room_id").and_then(serde_json::Value::as_u64) {
                    ids.insert(id);
                }
            }
        }
    }
}

/// 旧 Prepared 仍在时先固定原确认正文，防止第二次失败覆盖唯一撤销依据。
pub(super) fn reconcile(dir: &Path, room_id: u64, confirmed: Option<&[u8]>) -> Result<(), String> {
    let path = dir.join(format!("{room_id}.json"));
    if read_optional(&path)?.as_deref() == confirmed {
        return Ok(());
    }
    if let Some(confirmed) = confirmed {
        let tmp = dir.join(format!("{room_id}.json.{}.tmp", uuid::Uuid::new_v4()));
        let mut owns_temp = false;
        let mut visible = false;
        let result = (|| -> Result<(), String> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)
                .map_err(|e| format!("创建撤销临时文件失败: {e}"))?;
            owns_temp = true;
            file.write_all(confirmed)
                .map_err(|e| format!("写撤销临时文件失败: {e}"))?;
            file.sync_all()
                .map_err(|e| format!("同步撤销临时文件失败: {e}"))?;
            drop(file);
            replace_file(&tmp, &path, &mut visible)
                .map_err(|e| format!("固定原确认正文失败: {e}"))?;
            #[cfg(not(windows))]
            sync_directory(dir).map_err(|e| format!("同步撤销目录失败: {e}"))?;
            Ok(())
        })();
        if result.is_err() && owns_temp && !visible {
            if let Err(error) = std::fs::remove_file(tmp) {
                return Err(format!(
                    "{}；清理撤销临时文件失败: {error}",
                    result.unwrap_err()
                ));
            }
        }
        result
    } else {
        std::fs::remove_file(path).map_err(|e| format!("撤销首次候选失败: {e}"))?;
        #[cfg(not(windows))]
        sync_directory(dir).map_err(|e| format!("同步首次候选撤销失败: {e}"))?;
        Ok(())
    }
}

pub(super) fn write(
    dir: &Path,
    journal: &Journal,
    #[cfg(test)] fault: Option<StoreFault>,
) -> Result<(), String> {
    let path = dir.join(format!("{}.json.journal", journal.room_id));
    let tmp = dir.join(format!(
        "{}.json.{}.tmp",
        journal.room_id,
        uuid::Uuid::new_v4()
    ));
    let had_journal = path.exists();
    let mut created = false;
    let mut visible = false;
    let result = (|| -> Result<(), String> {
        let bytes =
            serde_json::to_vec_pretty(journal).map_err(|e| format!("序列化提交日志失败: {e}"))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| format!("创建提交日志临时文件失败: {e}"))?;
        created = true;
        #[cfg(test)]
        if matches!(
            (journal.phase, fault),
            (Phase::Prepared, Some(StoreFault::PrepareWrite))
                | (Phase::Committed, Some(StoreFault::DecisionWrite))
        ) {
            file.write_all(&bytes[..bytes.len() / 2])
                .map_err(|e| format!("写提交日志失败: {e}"))?;
            return Err("注入：部分写入提交日志失败".into());
        }
        file.write_all(&bytes)
            .map_err(|e| format!("写提交日志失败: {e}"))?;
        #[cfg(test)]
        if matches!(
            (journal.phase, fault),
            (Phase::Prepared, Some(StoreFault::PrepareSync))
                | (Phase::Committed, Some(StoreFault::DecisionSync))
        ) {
            return Err("注入：同步提交日志失败".into());
        }
        file.sync_all()
            .map_err(|e| format!("同步提交日志失败: {e}"))?;
        drop(file);
        replace_file(&tmp, &path, &mut visible).map_err(|e| format!("替换提交日志失败: {e}"))?;
        #[cfg(test)]
        if matches!(
            (journal.phase, fault),
            (Phase::Prepared, Some(StoreFault::PrepareDirectorySync))
                | (Phase::Committed, Some(StoreFault::DecisionDirectorySync))
        ) {
            return Err("注入：替换提交日志后同步目录失败".into());
        }
        // 已存在旧 journal 时，prepared 的 rename 即使尚未持久化，崩溃后仍可回退到
        // 旧 journal；文件内容已 sync。后续 committed 写入会再同步目录。跳过这次
        // 目录 fsync 以减少每个已接受命令的写放大。首次写入、committed 与故障注入
        // 仍保留目录同步。
        let skip_prepared_dir_sync = matches!(journal.phase, Phase::Prepared) && had_journal;
        #[cfg(test)]
        let skip_prepared_dir_sync = skip_prepared_dir_sync
            && !matches!(fault, Some(StoreFault::PrepareDirectorySync));
        #[cfg(not(windows))]
        if !skip_prepared_dir_sync {
            sync_directory(dir).map_err(|e| format!("同步提交日志目录失败: {e}"))?;
        }
        Ok(())
    })();
    result.map_err(|mut error| {
        if created && !visible {
            if let Err(cleanup) = std::fs::remove_file(&tmp) {
                error.push_str(&format!("；清理提交日志临时文件失败: {cleanup}"));
            }
        }
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_checksum_does_not_bypass_version_identity_or_previous_validation() {
        let original =
            Journal::prepared(7, uuid::Uuid::new_v4().to_string(), b"candidate", None).unwrap();
        for mutation in 0..5 {
            let mut journal = original.clone();
            match mutation {
                0 => journal.version = 2,
                1 => journal.room_id = 99,
                2 => journal.commit_id = "not-a-uuid".into(),
                3 => journal.candidate_hash = "too short".into(),
                4 => {
                    journal.previous = Some(
                        serde_json::to_string(&super::super::tests::record(99, "wrong room"))
                            .unwrap(),
                    )
                }
                _ => unreachable!(),
            }
            journal.checksum = journal.digest();
            assert!(journal.validate(7).is_err(), "mutation {mutation}");
        }
        let mut bad_previous = original;
        let mut previous = super::super::tests::record(7, "tampered");
        previous.checksum = "bad checksum".into();
        bad_previous.previous = Some(serde_json::to_string(&previous).unwrap());
        bad_previous.checksum = bad_previous.digest();
        assert!(bad_previous.validate(7).is_err());
    }
}
