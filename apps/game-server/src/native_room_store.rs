//! NC09 原生房间**落盘存储**：把会话状态写到磁盘并在启动时恢复。
//!
//! 设计取舍（登记）：
//! - 每条**被接受**的命令后写一次（`snapshot_every` 无关：房间状态小，写放大可接受）；
//! - 同目录独占临时文件、同步文件后替换；Unix 再同步目录，Windows 用 write-through move；
//! - 房间密码仅以带房间号盐的哈希随状态落盘；
//! - 新房间把完整原局包副本随状态原子落盘；恢复按 `pack_refs` 校验副本及撤销策略；
//!   旧精确引用只在原安装版本仍可验证时补齐副本，缺包或错锁保留记录并拒绝。

use crate::room_native::NativeRoomState;
use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "native_room_store/lock.rs"]
mod lock;
pub use lock::StoreWriterLock;

/// rename 后同步失败时，替换已可见但尚不能确认耐久性。
#[derive(Debug)]
pub struct StoreWriteError {
    pub replacement_visible: bool,
    pub decision_uncertain: bool,
    pub message: String,
}

impl std::fmt::Display for StoreWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}（replacement_visible={}, decision_uncertain={}）",
            self.message, self.replacement_visible, self.decision_uncertain
        )
    }
}

impl std::error::Error for StoreWriteError {}

/// 启动逐文件扫描；问题文件保留，不影响其他完整记录。
#[derive(Debug, Default)]
pub struct RoomScan {
    pub records: Vec<StoredRoom>,
    pub issues: Vec<RoomScanIssue>,
    pub reserved_ids: BTreeSet<u64>,
}

#[derive(Debug)]
pub struct RoomScanIssue {
    pub path: PathBuf,
    pub room_id: Option<u64>,
    pub error: String,
    pub uncommitted_temp: bool,
}

/// 实例级一次故障；生产构建没有故障控制入口。
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreFault {
    PartialWrite,
    FileSync,
    Rename,
    DirectorySync,
    PrepareWrite,
    PrepareSync,
    PrepareDirectorySync,
    DecisionWrite,
    DecisionSync,
    DecisionDirectorySync,
    InterruptAfterRename,
    InterruptAfterCompensationFailure,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct StoreFaultPlan {
    primary: StoreFault,
    compensation: Option<StoreFault>,
    interrupt_after_compensation: bool,
}

/// 落盘记录：会话状态 + 房间级参数（密码）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredRoom {
    /// 房间号。
    pub room_id: u64,
    /// 房间密码的**加盐哈希**（空串 = 无密码）——不落明文。
    ///
    /// 盐为房间号（本地磁盘口径；生产环境应改为随机盐 + 外部密钥存储，见证据待办）。
    #[serde(default)]
    pub password_hash: String,
    /// 会话状态。
    pub state: NativeRoomState,
    /// 状态校验和（`blake3(state JSON)` 的 hex）——读回时校验，防篡改/截断。
    #[serde(default)]
    pub checksum: String,
}

/// 密码加盐哈希（v1：房间号作盐，兼容旧档）。空密码 → 空串（表示无密码）。
pub fn password_hash(room_id: u64, password: &str) -> String {
    if password.is_empty() {
        return String::new();
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"noname-native-room-pw-v1:");
    hasher.update(&room_id.to_le_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// 密码加盐哈希（v2：房间号 + 16 字节随机盐）。空密码 → 空串（表示无密码）。
pub fn password_hash_with_salt(room_id: u64, salt: &[u8; 16], password: &str) -> String {
    if password.is_empty() {
        return String::new();
    }
    let salt_hex: String = salt.iter().map(|b| format!("{b:02x}")).collect();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"noname-native-room-pw-v2:");
    hasher.update(&room_id.to_le_bytes());
    hasher.update(b":");
    hasher.update(salt);
    hasher.update(b":");
    hasher.update(password.as_bytes());
    let digest_hex = hasher.finalize().to_hex();
    format!("v2:{salt_hex}:{digest_hex}")
}

/// 生成每房间 16 字节随机盐并计算 v2 密码哈希（空密码 → 空串）。
pub fn salted_password_hash(room_id: u64, password: &str) -> String {
    if password.is_empty() {
        return String::new();
    }
    let salt = *uuid::Uuid::new_v4().as_bytes();
    password_hash_with_salt(room_id, &salt, password)
}

/// 校验房间密码（兼容 v1 房间号盐与 `v2:{salt_hex}:{digest_hex}` 随机盐哈希）。
pub fn verify_password_hash(room_id: u64, password: &str, stored_hash: &str) -> bool {
    if stored_hash.is_empty() {
        return true;
    }
    if password.is_empty() {
        return false;
    }
    let expected = if let Some(rest) = stored_hash.strip_prefix("v2:") {
        let Some((salt_hex, _)) = rest.split_once(':') else {
            return false;
        };
        if salt_hex.len() != 32 {
            return false;
        }
        let mut salt = [0u8; 16];
        for (i, pair) in salt_hex.as_bytes().chunks_exact(2).enumerate() {
            let Ok(text) = std::str::from_utf8(pair) else {
                return false;
            };
            let Ok(byte) = u8::from_str_radix(text, 16) else {
                return false;
            };
            salt[i] = byte;
        }
        password_hash_with_salt(room_id, &salt, password)
    } else {
        password_hash(room_id, password)
    };
    expected.len() == stored_hash.len()
        && expected
            .as_bytes()
            .iter()
            .zip(stored_hash.as_bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

/// 默认每 16 步执行一次完整 2PC 状态检查点并截断 `.wal.jsonl`。
pub const DEFAULT_WAL_CHECKPOINT_EVERY: u64 = 16;

/// 追加式 `{room_id}.wal.jsonl` 单行增量记录（NC12）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalSegmentLine {
    pub revision: u64,
    pub next_dedup: u64,
    pub snapshot_hex: String,
    pub entry: Option<crate::room_native::NativeWalEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command_receipts: Vec<crate::room_native::NativeCommandReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<crate::room_native::NativeTimeoutState>,
}

/// 状态校验和（`blake3(JSON)`）。
pub fn state_checksum(state: &NativeRoomState) -> String {
    let json = serde_json::to_vec(state).unwrap_or_default();
    blake3::hash(&json).to_hex().to_string()
}

/// 房间落盘存储（目录级）。
#[derive(Debug, Clone)]
pub struct NativeRoomStore {
    dir: PathBuf,
    #[cfg(test)]
    fault: std::sync::Arc<std::sync::Mutex<Option<StoreFaultPlan>>>,
}

#[path = "native_room_store/journal.rs"]
mod journal;

#[cfg(test)]
#[path = "native_room_store/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "native_room_store/commit_tests.rs"]
mod commit_tests;

impl NativeRoomStore {
    /// 以目录建存储（目录不存在时按需创建）。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            #[cfg(test)]
            fault: Default::default(),
        }
    }

    /// 目录路径。
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 房间快照文件路径（`{room_id}.json`）。
    pub fn room_path(&self, room_id: u64) -> PathBuf {
        self.dir.join(format!("{room_id}.json"))
    }

    /// 房间追加式 WAL 段路径（`{room_id}.wal.jsonl`）。
    pub fn wal_segment_path(&self, room_id: u64) -> PathBuf {
        self.dir.join(format!("{room_id}.wal.jsonl"))
    }

    /// 是否已挂载测试故障注入计划。
    pub fn has_armed_fault(&self) -> bool {
        #[cfg(test)]
        {
            return self.fault.lock().unwrap().is_some();
        }
        #[cfg(not(test))]
        {
            false
        }
    }

    /// 追加一行增量 WAL 记录至 `{room_id}.wal.jsonl`（非检查点步单次追加 + sync）。
    pub fn append_wal_segment(
        &self,
        room_id: u64,
        line: &WalSegmentLine,
    ) -> Result<(), StoreWriteError> {
        let result = (|| -> Result<(), String> {
            if room_id == 0 {
                return Err("房间号必须非零".into());
            }
            create_directory_durable(&self.dir)?;
            let seg_path = self.wal_segment_path(room_id);
            let existed = seg_path.exists();
            let mut payload =
                serde_json::to_vec(line).map_err(|e| format!("序列化 WAL 段失败: {e}"))?;
            payload.push(b'\n');
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&seg_path)
                .map_err(|e| format!("打开 WAL 段失败: {e}"))?;
            file.write_all(&payload)
                .map_err(|e| format!("追加 WAL 段失败: {e}"))?;
            file.sync_data()
                .map_err(|e| format!("同步 WAL 段失败: {e}"))?;
            drop(file);
            #[cfg(not(windows))]
            if !existed {
                sync_directory(&self.dir).map_err(|e| format!("同步 WAL 段目录失败: {e}"))?;
            }
            Ok(())
        })();
        result.map_err(|message| StoreWriteError {
            replacement_visible: false,
            decision_uncertain: false,
            message,
        })
    }

    #[cfg(test)]
    pub fn fail_next_write(&self, fault: StoreFault) {
        let interrupt_after_compensation = fault == StoreFault::InterruptAfterCompensationFailure;
        *self.fault.lock().unwrap() = Some(StoreFaultPlan {
            primary: if interrupt_after_compensation {
                StoreFault::DecisionDirectorySync
            } else {
                fault
            },
            compensation: interrupt_after_compensation.then_some(StoreFault::PrepareWrite),
            interrupt_after_compensation,
        });
    }

    #[cfg(test)]
    pub fn fail_next_write_with_compensation(&self, primary: StoreFault, compensation: StoreFault) {
        assert!(matches!(
            primary,
            StoreFault::DecisionWrite
                | StoreFault::DecisionSync
                | StoreFault::DecisionDirectorySync
        ));
        assert!(matches!(
            compensation,
            StoreFault::PrepareWrite | StoreFault::PrepareSync | StoreFault::PrepareDirectorySync
        ));
        *self.fault.lock().unwrap() = Some(StoreFaultPlan {
            primary,
            compensation: Some(compensation),
            interrupt_after_compensation: false,
        });
    }

    /// 兼容调用方：需要分辨提交不确定性的运行期使用 `save_durable`。
    pub fn save(&self, record: &StoredRoom) -> Result<(), String> {
        self.save_durable(record).map_err(|e| e.to_string())
    }

    /// 候选写入前固定撤销依据；只有完整候选和提交决定均同步后才返回成功。
    pub fn save_durable(&self, record: &StoredRoom) -> Result<(), StoreWriteError> {
        #[cfg(test)]
        let plan = self.fault.lock().unwrap().take();
        #[cfg(test)]
        let fault = plan.map(|plan| plan.primary);
        let mut replacement_visible = false;
        let mut decision_uncertain = false;
        let mut record = record.clone();
        record.checksum = state_checksum(&record.state);
        let final_path = self.dir.join(format!("{}.json", record.room_id));
        let tmp_path = self.dir.join(format!(
            "{}.json.{}.tmp",
            record.room_id,
            uuid::Uuid::new_v4()
        ));
        let mut owns_temp = false;
        let result = (|| -> Result<(), String> {
            if record.room_id == 0 {
                return Err("房间号必须非零".into());
            }
            create_directory_durable(&self.dir)?;
            let (previous, pending) = journal::confirmed_bytes(&self.dir, record.room_id)?;
            if pending {
                journal::reconcile(&self.dir, record.room_id, previous.as_deref())?;
            }
            let commit_id = uuid::Uuid::new_v4().to_string();
            let json = journal::encode(&record, &commit_id)?;
            let prepared = journal::Journal::prepared(record.room_id, commit_id, &json, previous)?;
            journal::write(
                &self.dir,
                &prepared,
                #[cfg(test)]
                fault,
            )?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp_path)
                .map_err(|e| format!("创建独占临时文件失败: {e}"))?;
            owns_temp = true;
            #[cfg(test)]
            if fault == Some(StoreFault::PartialWrite) {
                file.write_all(&json[..json.len() / 2])
                    .map_err(|e| format!("写临时文件失败: {e}"))?;
                return Err("注入：临时文件部分写入失败".into());
            }
            file.write_all(&json)
                .map_err(|e| format!("写临时文件失败: {e}"))?;
            #[cfg(test)]
            if fault == Some(StoreFault::FileSync) {
                return Err("注入：同步临时文件失败".into());
            }
            // 状态文件不是恢复权威：committed journal 保存完整 JSON，崩溃时按
            // journal 恢复；因此状态临时文件的内容 fsync 可省略（故障注入除外）。
            let skip_state_file_sync = true;
            #[cfg(test)]
            let skip_state_file_sync = !matches!(fault, Some(StoreFault::FileSync));
            if !skip_state_file_sync {
                file.sync_all()
                    .map_err(|e| format!("同步临时文件失败: {e}"))?;
            }
            drop(file);
            #[cfg(test)]
            if fault == Some(StoreFault::Rename) {
                return Err("注入：rename 失败".into());
            }
            replace_file(&tmp_path, &final_path, &mut replacement_visible)
                .map_err(|e| format!("rename 失败: {e}"))?;
            #[cfg(test)]
            if fault == Some(StoreFault::InterruptAfterRename) {
                std::process::exit(86);
            }
            #[cfg(test)]
            if fault == Some(StoreFault::DirectorySync) {
                return Err("注入：rename 后同步目录失败".into());
            }
            // journal prepared 已保存撤销依据；committed 写入会在后面同步目录，
            // 因此这次状态文件 rename 的目录同步可省略（故障注入仍保留）。
            let skip_state_dir_sync = true;
            #[cfg(test)]
            let skip_state_dir_sync = !matches!(fault, Some(StoreFault::DirectorySync));
            #[cfg(not(windows))]
            if !skip_state_dir_sync {
                sync_directory(&self.dir).map_err(|e| format!("同步房间目录失败: {e}"))?;
            }
            let committed = prepared.committed();
            if let Err(error) = journal::write(
                &self.dir,
                &committed,
                #[cfg(test)]
                fault,
            ) {
                // 决定替换后失败也尽力固定撤销；补偿失败明确保留最终决定不确定性。
                if let Err(abort_error) = journal::write(
                    &self.dir,
                    &prepared,
                    #[cfg(test)]
                    plan.and_then(|plan| plan.compensation),
                ) {
                    #[cfg(test)]
                    if plan.is_some_and(|plan| plan.interrupt_after_compensation) {
                        std::process::exit(87);
                    }
                    decision_uncertain = true;
                    return Err(format!("{error}；固定撤销决定失败: {abort_error}"));
                }
                return Err(error);
            }
            // 完整检查点提交成功后清理旧 WAL 段文件。
            let _ = std::fs::remove_file(self.wal_segment_path(record.room_id));
            Ok(())
        })();
        result.map_err(|mut message| {
            if owns_temp && !replacement_visible {
                if let Err(e) = std::fs::remove_file(&tmp_path) {
                    message.push_str(&format!("；清理本次临时文件失败: {e}"));
                }
            }
            StoreWriteError {
                replacement_visible,
                decision_uncertain,
                message,
            }
        })
    }

    fn replay_wal_segment(&self, record: &mut StoredRoom) -> Result<(), String> {
        let seg_path = self.wal_segment_path(record.room_id);
        let text = match std::fs::read_to_string(&seg_path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("读取 WAL 段失败: {e}")),
        };
        let lines: Vec<&str> = text
            .split('\n')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        for (idx, line) in lines.iter().enumerate() {
            let parsed: WalSegmentLine = match serde_json::from_str(line) {
                Ok(seg) => seg,
                Err(_) if idx + 1 == lines.len() => {
                    // 忽略崩溃留下的尾部半行记录。
                    break;
                }
                Err(e) => return Err(format!("WAL 段第 {} 行解析失败: {e}", idx + 1)),
            };
            if parsed.next_dedup >= record.state.next_dedup {
                record.state.snapshot_hex = parsed.snapshot_hex;
                record.state.next_dedup = parsed.next_dedup;
                record.state.command_receipts = parsed.command_receipts;
                if parsed.timeout.is_some() {
                    record.state.timeout = parsed.timeout;
                }
                if let (Some(wal), Some(entry)) = (&mut record.state.wal, parsed.entry) {
                    if wal.entries.last().map(|e| e.dedup_id) != Some(entry.dedup_id) {
                        wal.entries.push(entry);
                    }
                }
            }
        }
        record.checksum = state_checksum(&record.state);
        Ok(())
    }

    /// 问题文件保留并占用 ID；Prepared 只能恢复原确认记录，不能提升候选。
    pub fn scan(&self) -> Result<RoomScan, String> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(RoomScan::default()),
            Err(e) => return Err(format!("读房间目录失败: {e}")),
        };
        let mut out = RoomScan::default();
        let mut rooms = BTreeSet::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("读目录项失败: {e}"))?;
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Some(num) = name.strip_suffix(".wal.jsonl") {
                if !num.is_empty() && num.bytes().all(|b| b.is_ascii_digit()) {
                    if let Ok(id) = num.parse::<u64>() {
                        if id > 0 && num == id.to_string() {
                            out.reserved_ids.insert(id);
                            continue;
                        }
                    }
                }
            }
            let name = name.strip_suffix(".journal").unwrap_or(name);
            let Some((room_id, uncommitted_temp, canonical)) = room_filename(name) else {
                continue;
            };
            out.reserved_ids.insert(room_id);
            if uncommitted_temp || !canonical || room_id == 0 {
                out.issues.push(RoomScanIssue {
                    path,
                    room_id: Some(room_id),
                    uncommitted_temp,
                    error: if uncommitted_temp {
                        "未提交临时文件保留，未作为已提交存档恢复"
                    } else {
                        "房间文件名必须为规范的非零房间号.json"
                    }
                    .into(),
                });
                continue;
            }
            if entry
                .file_type()
                .map_err(|e| format!("读文件类型失败: {e}"))?
                .is_file()
            {
                if let Ok(bytes) = std::fs::read(&path) {
                    journal::reserve_payload_ids(&bytes, &mut out.reserved_ids);
                }
            }
            rooms.insert(room_id);
        }
        for room_id in rooms {
            match journal::confirmed_bytes(&self.dir, room_id) {
                Ok((bytes, prepared)) => {
                    if let Some(bytes) = bytes {
                        match journal::decode(&bytes, room_id) {
                            Ok(mut record) => match self.replay_wal_segment(&mut record) {
                                Ok(()) => out.records.push(record),
                                Err(error) => out.issues.push(RoomScanIssue {
                                    path: self.wal_segment_path(room_id),
                                    room_id: Some(room_id),
                                    error,
                                    uncommitted_temp: false,
                                }),
                            },
                            Err(error) => out.issues.push(RoomScanIssue {
                                path: self.dir.join(format!("{room_id}.json")),
                                room_id: Some(room_id),
                                error,
                                uncommitted_temp: false,
                            }),
                        }
                    }
                    if prepared {
                        out.issues.push(RoomScanIssue {
                            path: self.dir.join(format!("{room_id}.json.journal")),
                            room_id: Some(room_id),
                            uncommitted_temp: true,
                            error: "未提交候选保留，仅恢复撤销日志中的原确认记录（或原先无记录）"
                                .into(),
                        });
                    }
                }
                Err(error) => out.issues.push(RoomScanIssue {
                    path: self.dir.join(format!("{room_id}.json")),
                    room_id: Some(room_id),
                    error,
                    uncommitted_temp: false,
                }),
            }
        }
        out.records.sort_by_key(|r| r.room_id);
        out.issues.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    /// 严格读取：坏档明确报错；未提交临时文件仍不作为记录。
    pub fn load_all(&self) -> Result<Vec<StoredRoom>, String> {
        let scan = self.scan()?;
        if let Some(issue) = scan.issues.iter().find(|issue| !issue.uncommitted_temp) {
            return Err(format!("{}: {}", issue.path.display(), issue.error));
        }
        Ok(scan.records)
    }

    /// 删除一个房间的落盘记录（房间关闭/结束时调用）。
    pub fn remove(&self, room_id: u64) -> Result<(), String> {
        // 先删日志：中断时留下的终局正文不能再从 Prepared 的旧局恢复。
        for name in [
            format!("{room_id}.json.journal"),
            format!("{room_id}.wal.jsonl"),
            format!("{room_id}.json"),
        ] {
            match std::fs::remove_file(self.dir.join(name)) {
                Ok(()) => {
                    #[cfg(not(windows))]
                    sync_directory(&self.dir).map_err(|e| format!("同步删除失败: {e}"))?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("删除失败: {e}")),
            }
        }
        Ok(())
    }

    /// 便捷：按房间号索引。
    pub fn by_id(&self) -> Result<HashMap<u64, StoredRoom>, String> {
        Ok(self
            .load_all()?
            .into_iter()
            .map(|r| (r.room_id, r))
            .collect())
    }
}

fn room_filename(name: &str) -> Option<(u64, bool, bool)> {
    let (number, temporary) = if let Some(number) = name.strip_suffix(".json") {
        (number, false)
    } else if let Some(number) = name.strip_suffix(".json.tmp") {
        (number, true)
    } else {
        let (number, suffix) = name.split_once(".json.")?;
        uuid::Uuid::parse_str(suffix.strip_suffix(".tmp")?).ok()?;
        (number, true)
    };
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let id = number.parse::<u64>().ok()?;
    Some((id, temporary, number == id.to_string()))
}

fn create_directory_durable(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("创建房间目录失败: {e}"))?;
    // 失败重试也同步祖先：此前 mkdir 可能成功但尚未同步父目录。
    #[cfg(not(windows))]
    for path in dir.ancestors() {
        let path = if path.as_os_str().is_empty() {
            Path::new(".")
        } else {
            path
        };
        sync_directory(path).map_err(|e| format!("同步房间目录层级失败: {e}"))?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(dir: &Path) -> std::io::Result<()> {
    std::fs::File::open(dir)?.sync_all()
}

#[cfg(not(windows))]
fn replace_file(
    source: &Path,
    destination: &Path,
    replacement_visible: &mut bool,
) -> std::io::Result<()> {
    std::fs::rename(source, destination)?;
    *replacement_visible = true;
    Ok(())
}

#[cfg(windows)]
fn replace_file(
    source: &Path,
    destination: &Path,
    replacement_visible: &mut bool,
) -> std::io::Result<()> {
    // shortcut: Windows mkdir/unlink 未取得 Unix 目录 fsync 保证；需目标文件系统断电测试后再增强承诺。
    use std::os::windows::ffi::OsStrExt;
    fn wide(path: &Path) -> std::io::Result<Vec<u16>> {
        let mut encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
        if encoded.contains(&0) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "路径包含 NUL",
            ));
        }
        encoded.push(0);
        Ok(encoded)
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, destination: *const u16, flags: u32) -> i32;
    }
    let from = wide(&source.canonicalize()?)?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = destination
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "目标缺少文件名"))?;
    let to = wide(&parent.canonicalize()?.join(name))?;
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
    // SAFETY: 两个拥有的 UTF-16 缓冲区以 NUL 结尾，并存活到同步调用返回。
    if unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        let error = std::io::Error::last_os_error();
        *replacement_visible = !matches!(source.try_exists(), Ok(true));
        return Err(error);
    }
    *replacement_visible = true;
    Ok(())
}
