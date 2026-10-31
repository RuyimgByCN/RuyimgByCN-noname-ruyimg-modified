//! E02/C3 签名信任链与多级历史（2026-10-08）：
//! 签名安装→验证→篡改拒绝→错钥拒绝→未签名拒绝→撤销→多级版本历史
//! （v1→v2 激活切换/历史保留/hash 撤销 v2 后 v1 可回激活）。

use content_runtime::{
    sign_package, tooling_host_versions, verify_package, InstallOptions, PackageSignature,
    PackageStore, RuntimeError,
};
use content_schema::ContentPackage;

fn opts_signed(key: &str) -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some(key.to_string()),
        host: tooling_host_versions(),
    }
}

fn sample_package(ns: &str, id: &str, ver: &str) -> Vec<u8> {
    let mut pkg = ContentPackage {
        manifest: content_schema::PackageManifest::new(ns, id, ver),
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

#[test]
fn signature_trust_chain_and_version_history() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let key = "publisher-key-material-2026";
    let other_key = "attacker-key-material";

    // ① 签名安装（正确密钥）→ signed=true，激活
    let v1 = sample_package("community", "trust-chain", "1.0.0");
    let sig1 = sign_package(&v1, key, "publisher-2026").expect("签名");
    assert!(verify_package(&v1, key, &sig1).unwrap(), "签名可验证");
    let r1 = store
        .install(&v1, Some(&sig1), &opts_signed(key))
        .expect("签名安装");
    assert!(r1.meta.signed, "安装记录 signed=true");
    let ns = r1.meta.namespace.as_str();
    let pid = r1.meta.package_id.as_str();

    // ② 篡改字节 → 签名验证失败拒绝
    let mut tampered = v1.clone();
    tampered[10] ^= 0xFF;
    let err = store.install(&tampered, Some(&sig1), &opts_signed(key));
    assert!(err.is_err(), "篡改字节应被拒: {err:?}");

    // ③ 错误密钥签名 → 验证失败拒绝
    let bad_sig = sign_package(&v1, other_key, "attacker").expect("错钥签名");
    assert!(
        !verify_package(&v1, key, &bad_sig).unwrap(),
        "错钥签名不可验证"
    );
    let err = store.install(&v1, Some(&bad_sig), &opts_signed(key));
    assert!(err.is_err(), "错钥签名安装应被拒: {err:?}");

    // ④ verify_key 模式下未签名 → 拒绝
    let err = store.install(&v1, None, &opts_signed(key));
    assert!(err.is_err(), "未签名安装应被拒: {err:?}");

    // ⑤ 多级历史：v2 签名安装（同槽新版本）→ activate_version 切换
    let v2 = sample_package(ns, pid, "2.0.0");
    let sig2 = sign_package(&v2, key, "publisher-2026").expect("签名 v2");
    store
        .install(&v2, Some(&sig2), &opts_signed(key))
        .expect("v2 安装");
    store.activate_version(ns, pid, "2.0.0").expect("激活 v2");
    let installed = store.list_installed().unwrap();
    let vers: Vec<&str> = installed
        .iter()
        .filter(|m| m.package_id == pid)
        .map(|m| m.version.as_str())
        .collect();
    assert!(
        vers.contains(&"1.0.0") && vers.contains(&"2.0.0"),
        "两个版本都在历史: {vers:?}"
    );
    let active = store.read_active().unwrap();
    assert_eq!(
        active
            .packages
            .get(&format!("{ns}/{pid}"))
            .map(|s| s.as_str()),
        Some("2.0.0"),
        "激活版本=v2"
    );

    // ⑥ hash 撤销 v2 → v2 重装被拒；v1 仍可回激活（多级历史回退）
    let h2 = content_runtime::content_hash_hex(&v2);
    store.revoke_content_hash(&h2, "验收：撤销 v2", true);
    let err = store.install(&v2, Some(&sig2), &opts_signed(key));
    assert!(
        matches!(err, Err(RuntimeError::Revoked(_))),
        "撤销后 v2 重装应被拒: {err:?}"
    );
    store.activate_version(ns, pid, "1.0.0").expect("回激活 v1");
    let active = store.read_active().unwrap();
    assert_eq!(
        active
            .packages
            .get(&format!("{ns}/{pid}"))
            .map(|s| s.as_str()),
        Some("1.0.0"),
        "回退激活=v1（多级历史）"
    );
}
