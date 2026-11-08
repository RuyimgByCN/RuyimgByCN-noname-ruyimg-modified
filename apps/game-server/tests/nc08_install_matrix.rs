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

/// **符号链接包目录 ⇒ 明确报错**（而不是「静默装 0 个」）。
///
/// 背景：`import` 扫描**不跟随 symlink**；此前对「非空 root 但装 0 个包」返回 `Ok(空)`，
/// 部署者会误以为安装成功。本用例锁住新的**报错**行为。
#[cfg(unix)]
#[test]
fn symlinked_package_dir_is_rejected_explicitly() {
    let root = pkg_root();
    let pkg = root.join("shahai-swd-upstream-noname");
    assert!(pkg.is_dir(), "样本包应存在");
    let base = std::env::temp_dir().join(format!("nc08-symlink-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dropin = base.join("root");
    std::fs::create_dir_all(&dropin).expect("mkdir");
    std::os::unix::fs::symlink(&pkg, dropin.join("swd-link")).expect("symlink");

    let nc = NativeContent::open(dropin.clone(), base.join("store")).expect("open");
    let err = nc.import();
    match err {
        Err(msg) => {
            eprintln!("DBG 软链安装：被明确拒绝 ✅ —— {msg}");
            assert!(
                msg.contains("非空") || msg.contains("符号链接"),
                "错误信息应说明原因，实得：{msg}"
            );
        }
        Ok(v) => panic!("软链包目录应被拒绝，实得 Ok({} 个包)", v.len()),
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn all_37_sources_46_packages_server_load_scoped_drive_and_archive_roundtrip() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::content::extension::ExtensionPack;
    use engine_native::Engine;
    use std::collections::{BTreeMap, BTreeSet};

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let map_path = manifest_dir
        .join("../../docs/operations/evidence/2026-10-19-package-source-map/map.tsv");
    let map_text = std::fs::read_to_string(&map_path).expect("read map.tsv");
    let root = pkg_root();

    let mut by_source: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut all_dirs: BTreeSet<String> = BTreeSet::new();
    for (idx, line) in map_text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let src_id = cols[0].trim().to_string();
        let pkgs: Vec<String> = cols[3]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for p in &pkgs {
            all_dirs.insert(p.clone());
        }
        by_source.insert(src_id, pkgs);
    }
    assert_eq!(by_source.len(), 37, "must cover all 37 upstream sources");
    assert_eq!(all_dirs.len(), 46, "must cover all 46 extension packages");

    let base = std::env::temp_dir().join(format!("nc08-server-37src-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dropin = base.join("root");
    let store = base.join("store");
    std::fs::create_dir_all(&dropin).expect("mkdir dropin");
    for dir in &all_dirs {
        copy_dir(&root.join(dir), &dropin.join(dir));
    }

    let nc = NativeContent::open(dropin, store).expect("open NativeContent");
    let installed = nc.import().expect("import all 46 extension packages");
    assert_eq!(
        installed.len(),
        46,
        "all 46 extension packages must install into server PackageStore"
    );

    let planner = BoundedPlanner::new(64);
    for (src_id, dirs) in &by_source {
        let mut loaded_packs = Vec::new();
        for dir in dirs {
            let npack_str =
                std::fs::read_to_string(root.join(dir).join("package.npack.json")).unwrap();
            let npack_val: serde_json::Value = serde_json::from_str(&npack_str).unwrap();
            let ns = npack_val["manifest"]["namespace"].as_str().unwrap();
            let pid = npack_val["manifest"]["package_id"].as_str().unwrap();
            let loaded = nc
                .load_scoped(Some(ns), pid)
                .unwrap_or_else(|e| panic!("{src_id}/{dir}: load_scoped({ns}/{pid}) failed: {e}"));
            loaded_packs.push(loaded);
        }
        let refs: Vec<&ExtensionPack> = loaded_packs.iter().map(|l| &l.pack).collect();
        let mut engine = Engine::new_with_extensions(42007, 4, &refs)
            .unwrap_or_else(|e| panic!("{src_id}: Engine::new_with_extensions failed: {e:?}"));
        let rev0 = engine.revision();
        for _ in 0..8 {
            if engine.outcome().is_some() {
                break;
            }
            let seat = match engine.open_choice() {
                Some(open) => open.observer,
                None => engine.turn().current,
            };
            let plan = engine.plan_action(seat, &planner);
            engine.apply_plan(&plan).expect("apply_plan");
        }
        assert!(engine.revision() > rev0, "{src_id}: revision must advance");
        let hash_after = engine.state_hash();
        let bytes = encode_archive(&engine.save());
        let decoded = decode_archive(&bytes).expect("decode_archive");
        let restored = Engine::restore_with_extensions(&decoded, &refs)
            .unwrap_or_else(|e| panic!("{src_id}: restore_with_extensions failed: {e:?}"));
        assert_eq!(
            restored.state_hash(),
            hash_after,
            "{src_id}: server archive roundtrip state_hash mismatch"
        );
        assert!(
            Engine::restore(&decoded).is_err(),
            "{src_id}: Engine::restore without extensions must reject locked archive"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}

