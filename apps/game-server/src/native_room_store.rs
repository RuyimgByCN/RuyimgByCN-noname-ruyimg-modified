//! NC09 原生房间**落盘存储**：把会话状态写到磁盘并在启动时恢复。
//!
//! 设计取舍（登记）：
//! - 每条**被接受**的命令后写一次（`snapshot_every` 无关：房间状态小，写放大可接受）；
//! - 原子写：先写 `<id>.json.tmp` 再 `rename`（同目录 rename 原子）；
//! - 房间密码随状态落盘（明文，与旧线 manifest 同口径；仅本地磁盘）；
//! - 恢复时按 `pack_refs`（`包名@版本`）**重新装载**同内容包，再 `from_saved`；
//!   若包缺失/版本不符，`from_saved` 会明确失败（不静默降级）。

use crate::room_native::NativeRoomState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 落盘记录：会话状态 + 房间级参数（密码）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredRoom {
    /// 房间号。
    pub room_id: u64,
    /// 房间密码（空串 = 无密码）。
    pub password: String,
    /// 会话状态。
    pub state: NativeRoomState,
}

/// 房间落盘存储（目录级）。
#[derive(Debug, Clone)]
pub struct NativeRoomStore {
    dir: PathBuf,
}

impl NativeRoomStore {
    /// 以目录建存储（目录不存在时按需创建）。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// 目录路径。
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 原子写入一个房间（先写临时文件再 rename）。
    pub fn save(&self, record: &StoredRoom) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("创建房间目录失败: {e}"))?;
        let final_path = self.dir.join(format!("{}.json", record.room_id));
        let tmp_path = self.dir.join(format!("{}.json.tmp", record.room_id));
        let json = serde_json::to_vec_pretty(record).map_err(|e| format!("序列化失败: {e}"))?;
        std::fs::write(&tmp_path, &json).map_err(|e| format!("写临时文件失败: {e}"))?;
        std::fs::rename(&tmp_path, &final_path).map_err(|e| format!("rename 失败: {e}"))?;
        Ok(())
    }

    /// 读取全部房间（按房间号升序）。单文件损坏**明确报错**（不跳过、不静默）。
    pub fn load_all(&self) -> Result<Vec<StoredRoom>, String> {
        if !self.dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let entries = std::fs::read_dir(&self.dir).map_err(|e| format!("读目录失败: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("读目录项失败: {e}"))?;
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.ends_with(".json") || name.ends_with(".tmp") {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("读 {} 失败: {e}", path.display()))?;
            let record: StoredRoom = serde_json::from_str(&text)
                .map_err(|e| format!("解析 {} 失败: {e}", path.display()))?;
            out.push(record);
        }
        out.sort_by_key(|r| r.room_id);
        Ok(out)
    }

    /// 删除一个房间的落盘记录（房间关闭/结束时调用）。
    pub fn remove(&self, room_id: u64) -> Result<(), String> {
        let path = self.dir.join(format!("{room_id}.json"));
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))?;
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
