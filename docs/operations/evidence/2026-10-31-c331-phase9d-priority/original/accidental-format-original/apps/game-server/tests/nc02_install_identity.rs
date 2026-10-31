//! NC02 安装包身份与执行内容一致：以**已验证安装副本**为权威。
//!
//! 审计反例（2026-10-18 架构审查 `loader-probe.log`，使用正式
//! `NativeContent` 导入/装载流程）：
//! 1. VM 包导入后移除原始来源目录 → 已安装记录与 store 仍在，装载却因
//!    回读来源路径失败；
//! 2. 同一已安装 VM 包不重新安装，只在原始来源新增有效 `assets/cards.json`
//!    → 装载结果由 1 个 VM 技能变为 0 个，并载入 7 张卡牌。
//!
//! 本文件按 NC02 验收项逐条覆盖：删除/修改来源仍加载安装版本；同 id 不同
//! 命名空间与多版本准确选择；混合卡牌与 VM 包不丢技能；安装内容损坏明确
//! 失败；重复装载稳定；升级不改写已固定内容锁的局。

#[path = "../src/native.rs"]
mod native;

use native::NativeContent;
use std::path::{Path, PathBuf};

const JIUWEI_DSL: &str = "\
skill nc02_jiuwei(990101) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn write_vm_package(dir: &Path, name: &str) -> String {
    let pkg = content_compiler::compile_source_named(name, JIUWEI_DSL).expect("DSL 编译");
    let dropin = dir.join(name);
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(
        dropin.join("package.npack.json"),
        serde_json::to_string(&pkg).unwrap(),
    )
    .unwrap();
    dropin.to_string_lossy().to_string()
}

/// 反例 1：删除原始来源目录后仍能从安装副本装载（VM 技能不丢）。
#[test]
fn nc02_loads_installed_copy_after_source_removed() {
    let dir = tempfile::tempdir().unwrap();
    let dropin_root = dir.path().to_path_buf();
    let dropin = write_vm_package(&dropin_root, "nc02-vm-a");
    let content = NativeContent::open(dropin_root.clone(), dir.path().join("store")).expect("open");
    let installed = content.import().expect("安装");
    assert!(installed.iter().any(|p| p.package_id == "nc02-vm-a"));

    // 移除 drop-in 原始来源（模拟作者撤下/移动来源目录）
    std::fs::remove_dir_all(&dropin).expect("移除来源");

    let loaded = content.load("nc02-vm-a").expect("装载安装副本");
    assert_eq!(loaded.pack.vm_skills.len(), 1, "VM 技能来自安装副本");
    assert!(loaded.pack.vm_skills[0].name.starts_with("nc02_jiuwei"));
}

/// 反例 2：只改原始来源的 `assets/cards.json`，不重装 → 执行内容不变。
#[test]
fn nc02_source_mutation_does_not_change_execution_content() {
    let dir = tempfile::tempdir().unwrap();
    let dropin_root = dir.path().to_path_buf();
    let dropin = write_vm_package(&dropin_root, "nc02-vm-b");
    let content = NativeContent::open(dropin_root.clone(), dir.path().join("store")).expect("open");
    content.import().expect("安装");

    // 在原始来源写入一份「有效但未安装」的 cards.json（7 张卡）
    let bogus = r#"{
      "pack": { "name": "nc02-bogus", "version": 1, "card_prefix": "nc02b_" },
      "cards": [
        {"name": "nc02b_sha1", "kind": "Sha"}, {"name": "nc02b_sha2", "kind": "Sha"},
        {"name": "nc02b_sha3", "kind": "Sha"}, {"name": "nc02b_sha4", "kind": "Sha"},
        {"name": "nc02b_sha5", "kind": "Sha"}, {"name": "nc02b_sha6", "kind": "Sha"},
        {"name": "nc02b_sha7", "kind": "Sha"}
      ],
      "skills": [], "generals": []
    }"#;
    std::fs::create_dir_all(Path::new(&dropin).join("assets")).unwrap();
    std::fs::write(Path::new(&dropin).join("assets/cards.json"), bogus).unwrap();

    let loaded = content.load("nc02-vm-b").expect("装载安装副本");
    assert_eq!(loaded.pack.vm_skills.len(), 1, "来源改动不影响执行内容");
    assert_eq!(loaded.pack.cards.len(), 0, "未安装的卡牌不得进入目录");
}

/// 混合包：`package.npack.json`（VM 声明）与 `assets/cards.json` 并存时两者都在。
///
/// npack 由 DSL 编译后**追加卡牌资产登记**（`asset_manifest` + `asset_paths`），
/// 与真实转换包（如 epic-cards）的资产形状一致；安装走 `import_extensions`。
#[test]
fn nc02_mixed_package_keeps_cards_and_vm_skills() {
    let dir = tempfile::tempdir().unwrap();
    let dropin_root = dir.path().to_path_buf();
    let name = "nc02-mixed";
    let cards = r#"{
      "pack": { "name": "nc02-mixed", "version": 1, "card_prefix": "nc02m_" },
      "cards": [ {"name": "nc02m_sha", "kind": "sha", "suit": "spade", "rank": 7} ],
      "skills": [], "generals": []
    }"#;
    // 编译 VM 包并追加卡牌资产登记
    let mut pkg = content_compiler::compile_source_named(name, JIUWEI_DSL).expect("DSL 编译");
    let raw = String::from_utf8(pkg.to_json_bytes().unwrap()).expect("npack UTF-8");
    let mut value: serde_json::Value = serde_json::from_str(&raw).expect("npack JSON");
    let hash: Vec<u8> = blake3::hash(cards.as_bytes()).as_bytes().to_vec();
    value["asset_manifest"] = serde_json::json!({
        "schema_version": 1,
        "package_id": name,
        "total_compressed_size": 0,
        "total_decoded_size": 0,
        "entries": [{
            "asset_id": format!("{name}:cards"),
            "source_hash": hash,
            "kind": "Data",
            "image_variants": [],
            "audio_variants": [],
            "priority": "Lazy",
            "depends_on": [],
            "platforms": [],
        }],
    });
    value["asset_paths"] = serde_json::json!({ format!("{name}:cards"): "assets/cards.json" });
    pkg = content_schema::ContentPackage::from_json_bytes(
        serde_json::to_string(&value).unwrap().as_bytes(),
    )
    .expect("重建 npack");
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();

    let dropin = dropin_root.join(name);
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    std::fs::write(
        dropin.join("package.npack.json"),
        pkg.to_json_bytes().unwrap(),
    )
    .unwrap();
    std::fs::write(dropin.join("assets/cards.json"), cards).unwrap();

    let content = NativeContent::open(dropin_root.clone(), dir.path().join("store")).expect("open");
    content.import().expect("安装混合包");

    let loaded = content.load(name).expect("装载混合包");
    assert_eq!(loaded.pack.vm_skills.len(), 1, "VM 技能保留");
    assert_eq!(loaded.pack.cards.len(), 1, "卡牌并入目录");
}

/// 同 id 不同命名空间：默认装载明确失败，指定命名空间可准确选择。
#[test]
fn nc02_same_id_different_namespace_requires_explicit_selection() {
    let dir = tempfile::tempdir().unwrap();
    let store_root = dir.path().join("store");
    let content = NativeContent::open(repo_packages_root(), store_root.clone()).expect("open");
    content.import().expect("安装仓库包");

    // 构造第二个命名空间下的同 id 包：直接用 store 安装
    let pkg = content_compiler::compile_source_named("huanle-cards", JIUWEI_DSL).expect("DSL 编译");
    let mut dup = pkg.clone();
    dup.manifest.namespace = "nc02-alt".into();
    dup.manifest.logic_hash = dup.compute_logic_hash();
    dup.manifest.asset_hash = dup.compute_asset_hash();
    let store = content_runtime::PackageStore::open(&store_root).expect("store");
    store
        .install(
            &dup.to_json_bytes().unwrap(),
            None,
            &content_runtime::InstallOptions {
                activate: true,
                allow_unsigned: true,
                verify_key: None,
                host: content_runtime::tooling_host_versions(),
            },
        )
        .expect("安装同 id 异命名空间");

    let err = match content.load("huanle-cards") {
        Ok(_) => panic!("同 id 多命名空间应明确失败"),
        Err(e) => e,
    };
    assert!(err.contains("命名空间"), "错误信息应提示命名空间: {err}");

    let picked = content
        .load_scoped(Some("upstream-noname"), "huanle-cards")
        .expect("指定命名空间可装载");
    assert_eq!(picked.package_id, "huanle-cards");
}

/// 安装内容损坏：装载明确失败（不静默降级）。
#[test]
fn nc02_corrupt_installed_copy_fails_clearly() {
    let dir = tempfile::tempdir().unwrap();
    let dropin_root = dir.path().to_path_buf();
    write_vm_package(&dropin_root, "nc02-corrupt");
    let store_root = dir.path().join("store");
    let content = NativeContent::open(dropin_root.clone(), store_root.clone()).expect("open");
    content.import().expect("安装");

    // 破坏安装副本的 package.json（逻辑指纹不再匹配内容锁）
    let pkg_dir = std::fs::read_dir(store_root.join("packages"))
        .expect("packages 目录")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().ends_with("__nc02-corrupt"))
                .unwrap_or(false)
        })
        .expect("nc02-corrupt 包目录");
    let version_dir = std::fs::read_dir(&pkg_dir)
        .expect("包目录")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("版本目录");
    std::fs::write(version_dir.join("package.json"), b"{ not json").unwrap();

    let err = match content.load("nc02-corrupt") {
        Ok(_) => panic!("损坏安装副本应失败"),
        Err(e) => e,
    };
    assert!(
        !err.is_empty(),
        "损坏安装副本必须明确失败（不静默降级）: {err}"
    );
    assert!(
        err.contains("装载安装副本失败")
            || err.contains("失败")
            || err.contains("key must be a string")
            || err.contains("content lock"),
        "错误应指向安装副本或内容锁: {err}"
    );
}

/// 重复装载稳定：两次装载的 VM 技能与卡牌一致。
#[test]
fn nc02_repeated_loads_are_stable() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装仓库包");
    let a = content.load("huanle-cards").expect("首次装载");
    let b = content.load("huanle-cards").expect("二次装载");
    assert_eq!(a.pack.cards.len(), b.pack.cards.len(), "卡牌数稳定");
    assert_eq!(
        a.pack.vm_skills.len(),
        b.pack.vm_skills.len(),
        "VM 技能数稳定"
    );
    assert_eq!(a.pack.def.name, b.pack.def.name, "包身份稳定");
}

/// 升级不改写已固定内容锁的局：旧锁仍装载旧版本。
#[test]
fn nc02_upgrade_does_not_rewrite_pinned_lock() {
    let dir = tempfile::tempdir().unwrap();
    let store_root = dir.path().join("store");
    let content = NativeContent::open(repo_packages_root(), store_root.clone()).expect("open");
    content.import().expect("安装仓库包");
    let store = content_runtime::PackageStore::open(&store_root).expect("store");

    let pinned = store.current_lock().expect("内容锁");
    let pinned_entry = pinned
        .packages
        .iter()
        .find(|p| p.package_id == "huanle-cards")
        .expect("锁含 huanle-cards")
        .clone();

    // 升级到 2.0.0（改写活动集）
    let bytes = store
        .load_active_package_bytes("upstream-noname", "huanle-cards")
        .expect("读活动包");
    let mut upgraded = content_schema::ContentPackage::from_json_bytes(&bytes).expect("解析活动包");
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    let version_dir = store
        .root()
        .join("packages")
        .join("upstream-noname__huanle-cards")
        .join(pinned_entry.version.clone());
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &content_runtime::InstallOptions {
                activate: true,
                allow_unsigned: true,
                verify_key: None,
                host: content_runtime::tooling_host_versions(),
            },
            &version_dir,
        )
        .expect("安装 2.0.0");

    // 旧锁仍可装载旧版本（不改写已固定内容锁的局）
    let locked = store.load_locked_packages(&pinned).expect("按旧锁装载");
    let old = locked
        .iter()
        .find(|p| p.manifest.package_id == "huanle-cards")
        .expect("旧版本仍在锁内");
    assert_eq!(old.manifest.version, pinned_entry.version, "旧锁版本不变");

    // 新装载（活动集）取到新版本
    let active = store.current_lock().expect("新锁");
    let active_entry = active
        .packages
        .iter()
        .find(|p| p.package_id == "huanle-cards")
        .expect("活动集含 huanle-cards");
    assert_eq!(active_entry.version, "2.0.0", "活动集已升级");
}
