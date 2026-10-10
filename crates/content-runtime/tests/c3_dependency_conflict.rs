//! E02/C3 冲突与依赖图场景（演进提案 C3、台账 E02/G1）：
//! ① 依赖级联剪枝：包 B 依赖 A@1.0.0，A 升级到 1.1.0 后 B 因依赖不满足
//!    被级联移除（依赖图不变量，循环剪至不动点）；
//! ② 激活失败原子性：选择集合非法（版本不存在/重复选择）时激活整体失败，
//!    活动集与内容锁保持原状；
//! ③ revoke 后重新激活被拒（Revoked），策略性解除后可恢复。

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency, PackageManifest};

fn manifest_with_deps(
    namespace: &str,
    id: &str,
    version: &str,
    deps: &[(&str, &str, &str)],
) -> PackageManifest {
    let mut manifest = PackageManifest::new(namespace, id, version);
    manifest.dependencies = deps
        .iter()
        .map(|(ns, pid, ver)| PackageDependency {
            namespace: ns.to_string(),
            package_id: pid.to_string(),
            version: ver.to_string(),
            logic_hash: [0u8; 32],
        })
        .collect();
    manifest
}

fn sample_package(namespace: &str, id: &str, version: &str) -> Vec<u8> {
    sample_package_with_deps(namespace, id, version, &[])
}

fn sample_package_with_deps(
    namespace: &str,
    id: &str,
    version: &str,
    deps: &[(&str, &str, &str)],
) -> Vec<u8> {
    let mut pkg = ContentPackage {
        manifest: manifest_with_deps(namespace, id, version, deps),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: Default::default(),
        asset_manifest: None,
        asset_paths: Default::default(),
        compiler_version: "test".into(),
    };
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    pkg.to_json_bytes().unwrap()
}

fn opts() -> InstallOptions {
    InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn dep(ns: &str, id: &str, ver: &str) -> PackageDependency {
    PackageDependency {
        namespace: ns.into(),
        package_id: id.into(),
        version: ver.into(),
        logic_hash: [0u8; 32],
    }
}

#[test]
fn c3_dependency_cascade_and_activation_atomicity() {
    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();

    // 安装 A@1.0.0、A@1.1.0、B@1.0.0（B 依赖 A@1.0.0）。
    for (ns, id, ver, deps) in [
        ("ns", "pack-a", "1.0.0", Vec::new()),
        ("ns", "pack-a", "1.1.0", Vec::new()),
        ("ns", "pack-b", "1.0.0", vec![("ns", "pack-a", "1.0.0")]),
    ] {
        let bytes = sample_package_with_deps(ns, id, ver, &deps);
        store.install(&bytes, None, &opts()).unwrap();
    }

    // 依赖图成立：A@1.0.0 + B@1.0.0 同批激活成功。
    store
        .activate_packages(&[dep("ns", "pack-a", "1.0.0"), dep("ns", "pack-b", "1.0.0")])
        .unwrap();
    let active = store.read_active().unwrap();
    assert_eq!(active.packages.len(), 2, "依赖图成立时两者共同激活");

    // ① 激活强校验：激活选择使 B 的依赖（A@1.0.0）不满足 → 整体拒绝
    //    （依赖图不变量：不允许出现悬空依赖的活动集）。
    let err = store
        .activate_packages(&[dep("ns", "pack-a", "1.1.0"), dep("ns", "pack-b", "1.0.0")])
        .unwrap_err();
    assert!(
        matches!(&err, content_runtime::RuntimeError::Invalid(msg)
            if msg.contains("dependency mismatch") && msg.contains("ns/pack-a@1.0.0")),
        "依赖不满足应明确拒绝: {err:?}"
    );
    let active = store.read_active().unwrap();
    assert_eq!(active.packages.len(), 2, "拒绝后活动集不变");

    // ② 单独把 A 升到 1.1.0 被拒：激活集中的 B 仍依赖 A@1.0.0——
    //    升级必须与依赖方同批处理，不能留下悬空依赖。
    assert!(store.activate_version("ns", "pack-a", "1.1.0").is_err());

    // ③ revoke A（吊销并解除激活）→ 依赖它的 B 被级联剪除（移除路径
    //    的依赖图剪枝到不动点）；此后 A@1.1.0 可正常激活。
    let a_meta = store
        .list_installed()
        .unwrap()
        .into_iter()
        .find(|m| m.package_id == "pack-a" && m.version == "1.0.0")
        .expect("A@1.0.0 已安装");
    store.revoke_content_hash(&a_meta.content_hash, "policy", true).unwrap();
    let active = store.read_active().unwrap();
    assert!(
        !active.packages.contains_key("ns/pack-a"),
        "A 被吊销并解除激活"
    );
    assert!(
        !active.packages.contains_key("ns/pack-b"),
        "依赖 A 的 B 应被级联剪除"
    );
    store.activate_version("ns", "pack-a", "1.1.0").unwrap();
    assert_eq!(
        store.read_active().unwrap().packages.get("ns/pack-a").map(String::as_str),
        Some("1.1.0"),
        "吊销旧依赖方后 A@1.1.0 可激活"
    );

    // ③ 激活失败原子性：选择版本不存在 → 整体失败，活动集与内容锁不变。
    let lock_before = store.current_lock().unwrap();
    let active_before = store.read_active().unwrap();
    let _ = store
        .activate_packages(&[
            dep("ns", "pack-a", "1.1.0"),
            dep("ns", "pack-b", "9.9.9"),
        ])
        .unwrap_err();
    assert!(
        store.read_active().unwrap().packages.contains_key("ns/pack-a"),
        "原子性验证前置：A@1.1.0 仍激活"
    );
    assert_eq!(
        store.read_active().unwrap().packages,
        active_before.packages,
        "激活失败不得改动活动集"
    );
    assert_eq!(
        store.current_lock().unwrap().lock_hash,
        lock_before.lock_hash,
        "激活失败不得改动内容锁"
    );

    // ④ revoke slot（不解除激活）→ 重新激活被拒（Revoked）。
    store.revoke_slot("ns", "pack-b", "policy", false).unwrap();
    assert!(store
        .activate_packages(&[dep("ns", "pack-b", "1.0.0")])
        .is_err(), "被 revoke 的包重新激活应被拒绝");
}
