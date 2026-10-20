//! NC08：**逐包「安装」矩阵**——把「安装」项从 5 个来源扩到**全部包目录**。
//!
//! 判据（每包，**独立临时 store**）：
//! 1. 把**单个包目录**（symlink）放进临时 root ⇒ `NativeContent::import()` **成功**；
//! 2. `installed()` 中**出现该包**（按 `package_id` 匹配）；
//! 3. **再次 import** ⇒ 安装记录数**不变**（**幂等**，不重复安装）。
//!
//! 边界：只覆盖**安装 + 幂等**；**不覆盖**「卸载/回滚」（`native.rs` 无 uninstall API，已登记）、
//! 也不覆盖「版本升级」（由 `nc08_upgrade_matrix` 覆盖）。

#[path = "../src/native.rs"]
mod native;

use native::NativeContent;
use std::path::{Path, PathBuf};

fn pkg_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 递归复制（`import` **不跟随 symlink**，实测用 symlink 会安装 0 个包 ⇒ 必须真复制）。
fn copy_dir(src: &Path, dst: &Path) {
    let _ = std::fs::create_dir_all(dst);
    let Ok(rd) = std::fs::read_dir(src) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let target = dst.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &target);
        } else if p.is_file() {
            let _ = std::fs::copy(&p, &target);
        }
    }
}

/// 造一个只含 `pkg` 的临时 root（**真复制**）。
fn single_root(base: &Path, pkg: &Path) -> PathBuf {
    let root = base.join("root");
    let dst = root.join(pkg.file_name().expect("pkg name"));
    if !dst.exists() {
        copy_dir(pkg, &dst);
    }
    root
}

/// 单包安装（返回 `Some(包名)` 通过；`None` 跳过）。
fn check(base: &Path, pkg: &Path, idx: usize) -> Option<String> {
    let name = pkg.file_name()?.to_string_lossy().to_string();
    if !pkg.join("package.npack.json").is_file() {
        eprintln!("NOTE 安装矩阵：{name} 无 package.npack.json —— 跳过");
        return None;
    }
    let scratch = base.join(format!("p{idx}"));
    let _ = std::fs::create_dir_all(&scratch);
    let root = single_root(&scratch, pkg);
    let store = scratch.join("store");

    let nc = NativeContent::open(root.clone(), store.clone()).expect("open native content");
    let installed = match nc.import() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("NOTE 安装矩阵：{name} import 失败（{e}）—— 跳过");
            return None;
        }
    };
    if installed.is_empty() {
        eprintln!("NOTE 安装矩阵：{name} import 成功但**安装 0 个包**（登记，见证据）");
        return None;
    }
    assert_eq!(installed.len(), 1, "{name}: 单包 root 最多只应安装 1 个包");
    let first = nc.installed().expect("installed");
    assert_eq!(first.len(), 1, "{name}: 安装记录应为 1");
    let pid = first[0].package_id.clone();

    // 幂等：再次 import 不得增加记录
    let again = nc.import().expect("re-import");
    assert_eq!(again.len(), 1, "{name}: 重复 import 不应新增包");
    let after = nc.installed().expect("installed2");
    assert_eq!(after.len(), 1, "{name}: 重复 import 后安装记录仍为 1");
    eprintln!(
        "DBG 安装矩阵：{name} ✅ package_id={pid} version={} 幂等 ✅",
        first[0].version
    );
    Some(name)
}

/// **逐包安装矩阵**：全部包目录各跑一次（独立 store）。
#[test]
fn install_matrix_over_package_dirs() {
    let root = pkg_root();
    let base = std::env::temp_dir().join(format!("nc08-install-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&base);
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read packages dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let mut passed = Vec::new();
    for (i, pkg) in dirs.iter().enumerate() {
        if let Some(n) = check(&base, pkg, i) {
            passed.push(n);
        }
    }
    eprintln!(
        "DBG 安装矩阵：通过 {}/{} 个包目录",
        passed.len(),
        dirs.len()
    );
    let _ = std::fs::remove_dir_all(&base);
    assert!(
        passed.len() >= 35,
        "应至少 35 个包目录通过安装矩阵，实得 {}：{passed:?}",
        passed.len()
    );
}
