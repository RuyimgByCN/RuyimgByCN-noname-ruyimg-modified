//! GT-014 导入期深层校验（裁决第 12 项）：Worker `validatePackage` 请求所用的
//! `validate_pack_json` 复用开局装载的逐包 Rust 校验器。本测试对服务端导入
//! 核对（`apps/game-server/tests/gt014_vm_import_parity.rs`）使用的**同一夹具、
//! 同一变异集**逐条执行：合法夹具通过；六类变异全部在导入期被拒——其中
//! `local_count+1` 与篡改触发事件名两类此前只在开局被拒（GT-014 证据 §4
//! 登记差异），现在提前到导入期，与服务端导入一致。另验 ABI 窗口：要求
//! 宿主 ABI 高于 753 的真实包在导入期即拒。
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/web-client/scripts/fixtures")
        .join(name);
    let mut pack: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("package.npack.json")).unwrap())
            .unwrap();
    pack["characters"] =
        serde_json::from_str(&std::fs::read_to_string(root.join("character.json")).unwrap())
            .unwrap();
    pack
}

#[test]
fn valid_fixtures_pass_import_validation() {
    for name in ["vm-trigger", "vm-trigger-v2"] {
        engine_native_wasm::validate_pack_json(&fixture(name).to_string())
            .unwrap_or_else(|e| panic!("{name} must pass: {e}"));
    }
}

/// 与服务端导入核对同一变异集：每条都在导入期被拒；输出行供证据与
/// `GT014_SERVER_REJECT` 逐条对照。
#[test]
fn same_mutations_as_server_import_are_rejected_at_import_time() {
    let v1 = fixture("vm-trigger");
    let mutate = |f: &dyn Fn(&mut Value)| {
        let mut bad = v1.clone();
        f(&mut bad);
        bad
    };
    let cases: Vec<(&str, Value)> = vec![
        ("corrupt_module_local_count", mutate(&|p| {
            let n = p["modules"][0]["local_count"].as_u64().unwrap();
            p["modules"][0]["local_count"] = (n + 1).into();
        })),
        ("entry_pc_out_of_range", mutate(&|p| {
            let len = p["modules"][0]["code"].as_array().unwrap().len();
            p["modules"][0]["entry_pc"] = (len as u64).into();
        })),
        ("trigger_unknown_module", mutate(&|p| {
            p["trigger_index"][0]["module"] = "ghost".into();
        })),
        ("tampered_trigger_event", mutate(&|p| {
            p["trigger_index"][0]["event"] = "phase_draw".into();
        })),
        ("invalid_identity", mutate(&|p| {
            p["manifest"]["package_id"] = "../x".into();
        })),
        ("same_version_different_content", mutate(&|p| {
            let n = p["modules"][0]["caps_mask"].as_u64().unwrap();
            p["modules"][0]["caps_mask"] = (n + 1).into();
        })),
    ];
    for (name, bad) in &cases {
        let err = engine_native_wasm::validate_pack_json(&bad.to_string())
            .expect_err(&format!("{name}: import-time validation must reject"));
        eprintln!("GT014_WASM_REJECT {name}: {err}");
    }
    // 此前只在开局被拒的两类（指纹破坏）现在导入期即以指纹错因拒绝。
    for name in ["corrupt_module_local_count", "tampered_trigger_event"] {
        let bad = &cases.iter().find(|(n, _)| *n == name).unwrap().1;
        let err = engine_native_wasm::validate_pack_json(&bad.to_string()).unwrap_err();
        assert!(
            err.contains("logic_hash") || err.contains("指纹") || err.contains("hash"),
            "{name}: expected fingerprint error, got: {err}"
        );
    }
}

/// ABI 窗口：要求高于宿主 ABI 753 的真实包（onlyOL 0.1.5 要求 758）在导入期即拒。
#[test]
fn package_requiring_newer_host_abi_is_rejected_at_import_time() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/onlyOL/onlyOL.npack.json");
    let json = std::fs::read_to_string(&path).expect("onlyOL npack present");
    let err = engine_native_wasm::validate_pack_json(&json)
        .expect_err("onlyOL requires ABI 758 > host 753");
    eprintln!("GT014_WASM_REJECT abi_window: {err}");
    assert!(
        err.to_lowercase().contains("abi"),
        "expected ABI window error, got: {err}"
    );
}
