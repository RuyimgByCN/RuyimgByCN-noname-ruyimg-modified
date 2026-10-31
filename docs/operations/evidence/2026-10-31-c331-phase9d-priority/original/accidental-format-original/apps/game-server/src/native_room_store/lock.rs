//! 原生房间存档目录的**单写者**所有权。
//!
//! 多实例部署前的最低保证：一个存档目录同时只允许一个服务进程写。锁在
//! 进程存续期间持有，崩溃/退出由操作系统关闭句柄释放；因此异常退出后可安全
//! 重启，而第二个健康进程会在启动时明确失败，不会各自恢复/覆写同一房间。
//!
//! - Unix：`flock(LOCK_EX | LOCK_NB)`（advisory，但对本服务的所有写者一致）。
//! - Windows：打开锁文件时 `share_mode(0)`，其他进程无法同时打开。
//! - 锁文件名为 `.writer.lock`，`scan` 只解析 `<room>.json`，不会当作房间记录。

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// 进程级独占的存档目录写者锁。
#[derive(Debug)]
pub struct StoreWriterLock {
    path: PathBuf,
    _file: File,
}

impl StoreWriterLock {
    /// 取得目录独占锁；已有写者时返回可读错误（调用方应拒绝启动）。
    pub fn acquire(dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("创建存档目录 {} 失败: {e}", dir.display()))?;
        let path = dir.join(".writer.lock");
        let mut file = open_lock_file(&path)?;
        lock_file(&file, &path)?;
        // 仅供诊断；锁的权威性来自 OS 句柄，不依赖文件内容。
        file.set_len(0)
            .and_then(|_| file.seek(SeekFrom::Start(0)).map(|_| ()))
            .and_then(|_| write!(file, "pid={}\n", std::process::id()))
            .map_err(|e| format!("写锁文件 {} 失败: {e}", path.display()))?;
        Ok(Self { path, _file: file })
    }

    /// 锁文件路径（诊断/测试）。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(unix)]
fn open_lock_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| format!("打开锁文件 {} 失败: {e}", path.display()))
}

#[cfg(unix)]
fn lock_file(file: &File, path: &Path) -> Result<(), String> {
    use std::os::unix::io::AsRawFd;
    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }
    // SAFETY: `file` 拥有有效 fd，且在本调用期间保持打开。
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
    if rc == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    Err(format!(
        "存档目录已有另一个写者（{}）：{}",
        path.display(),
        error
    ))
}

#[cfg(windows)]
fn open_lock_file(path: &Path) -> Result<File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        // 不允许其他进程/句柄同时打开；进程退出时由 OS 关闭。
        .share_mode(0)
        .open(path)
        .map_err(|e| format!("打开锁文件 {} 失败（目录已有写者？）: {e}", path.display()))
}

#[cfg(windows)]
fn lock_file(_file: &File, _path: &Path) -> Result<(), String> {
    // 互斥已由 `share_mode(0)` 在打开时建立。
    Ok(())
}

#[cfg(all(not(unix), not(windows)))]
fn open_lock_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| format!("打开锁文件 {} 失败: {e}", path.display()))
}

#[cfg(all(not(unix), not(windows)))]
fn lock_file(_file: &File, path: &Path) -> Result<(), String> {
    Err(format!(
        "当前平台不支持目录排他锁（{}）；拒绝在没有单写者保证时启动",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_writer_is_rejected_until_first_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let first = StoreWriterLock::acquire(dir.path()).expect("首个写者");
        let error = StoreWriterLock::acquire(dir.path()).expect_err("第二写者必须拒绝");
        assert!(error.contains("写者"), "{error}");
        assert!(first.path().exists());
        drop(first);
        // 进程内释放后同一目录可重新取得（崩溃重启的等价路径）。
        StoreWriterLock::acquire(dir.path()).expect("释放后可重新取得");
    }

    #[test]
    fn different_directories_are_independent() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let _a = StoreWriterLock::acquire(a.path()).expect("a");
        let _b = StoreWriterLock::acquire(b.path()).expect("b");
    }

    #[test]
    fn lock_file_is_not_a_room_record() {
        let dir = tempfile::tempdir().unwrap();
        let _lock = StoreWriterLock::acquire(dir.path()).unwrap();
        let store = crate::native_room_store::NativeRoomStore::new(dir.path());
        assert!(store.scan().unwrap().records.is_empty());
        assert!(store.scan().unwrap().issues.is_empty());
    }

    #[test]
    fn empty_lock_content_does_not_block_reacquire() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".writer.lock"), b"stale pid=1\n").unwrap();
        let _lock = StoreWriterLock::acquire(dir.path()).expect("陈旧文件不应阻止启动");
    }
}
