//! GT-014 同包核对：客户端正式导入入口使用的 vm-fixture（v0.1.0／v0.1.1）与同一组
//! 变异，经服务端 drop-in 导入（`NativeContent::import`）得到一致的接受／拒绝结果；
//! 失败的导入不影响已安装版本，升级与回退按版本切换装载副本。
//! 客户端侧见 `apps/web-client/scripts/native-vm-install.test.mjs` 与
//! `native-worker-smoke.mjs` 的 GT-014 段。

#[path = "../src/native.rs"]
#[allow(dead_code)]
mod native;

use native::NativeContent;
use std::path::{Path, PathBuf};

fn fixture_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../web-client/scripts/fixtures").join(name)
}

fn fixture_json(name: &str) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(fixture_dir(name).join("package.npack.json")).unwrap())
        .unwrap()
}

/// 把一个 npack（及夹具武将资产）写成服务端 drop-in 目录。
fn write_dropin(packages_root: &Path, dir: &str, npack: &serde_json::Value, characters_from: &str) {
    let dropin = packages_root.join(dir);
    std::fs::create_dir_all(dropin.join("assets")).unwrap();
    std::fs::write(
        dropin.join("package.npack.json"),
        serde_json::to_vec_pretty(npack).unwrap(),
    )
    .unwrap();
    std::fs::copy(
        fixture_dir(characters_from).join("character.json"),
        dropin.join("assets/character.json"),
    )
    .unwrap();
}

struct Server {
    _dir: tempfile::TempDir,
    packages_root: PathBuf,
    content: NativeContent,
}

fn server() -> Server {
    let dir = tempfile::tempdir().unwrap();
    let packages_root = dir.path().join("packages");
    let store_root = dir.path().join("store");
    std::fs::create_dir_all(&packages_root).unwrap();
    let content = NativeContent::open(packages_root.clone(), store_root).unwrap();
    Server {
        _dir: dir,
        packages_root,
        content,
    }
}

fn versions(content: &NativeContent) -> Vec<String> {
    let mut out: Vec<String> = content
        .installed()
        .unwrap()
        .into_iter()
        .filter(|p| p.namespace == "official" && p.package_id == "vm-fixture")
        .map(|p| p.version)
        .collect();
    out.sort();
    out
}

fn loaded_module_code(content: &NativeContent) -> (String, Vec<u8>, Vec<String>) {
    let loaded = content.load_scoped(Some("official"), "vm-fixture").unwrap();
    let package =
        content_schema::ContentPackage::from_json_bytes(loaded.frozen.package_json.as_bytes())
            .unwrap();
    (
        package.manifest.version.clone(),
        package.modules[0].code.clone(),
        loaded.pack.vm_skills.iter().map(|d| d.name.clone()).collect(),
    )
}

#[test]
fn valid_fixture_imports_activates_loads_vm_skill_and_upgrades_then_rolls_back() {
    let s = server();
    let v1 = fixture_json("vm-trigger");
    let v2 = fixture_json("vm-trigger-v2");
    write_dropin(&s.packages_root, "vm-fixture", &v1, "vm-trigger");
    let report = s.content.import().unwrap();
    assert!(report
        .iter()
        .any(|p| p.namespace == "official" && p.package_id == "vm-fixture" && p.version == "0.1.0"));
    assert_eq!(versions(&s.content), vec!["0.1.0"]);
    let (version, code, skills) = loaded_module_code(&s.content);
    assert_eq!(version, "0.1.0");
    assert_eq!(code, vec![83, 16, 1, 0, 0, 0, 80, 1, 1]);
    assert_eq!(skills, vec!["vm_fixture_phase_end".to_string()]);

    // 升级：服务端 drop-in 一包一目录，新版本覆盖同目录后再导入即自动激活；
    // 旧版本安装副本仍在存储中（阶段结束摸 2 张的字节码成为装载副本）。
    write_dropin(&s.packages_root, "vm-fixture", &v2, "vm-trigger-v2");
    s.content.import().unwrap();
    assert_eq!(versions(&s.content), vec!["0.1.0", "0.1.1"]);
    let (version, code, _) = loaded_module_code(&s.content);
    assert_eq!(version, "0.1.1");
    assert_eq!(code, vec![83, 16, 2, 0, 0, 0, 80, 1, 1]);

    // 回退：旧版本安装副本仍在，可重新成为装载副本。
    s.content.activate_version("official", "vm-fixture", "0.1.0").unwrap();
    let (version, code, _) = loaded_module_code(&s.content);
    assert_eq!(version, "0.1.0");
    assert_eq!(code, vec![83, 16, 1, 0, 0, 0, 80, 1, 1]);

    // 停用：装载失败但副本保留；卸载：所有版本移除。
    s.content.disable_package("official", "vm-fixture").unwrap();
    assert!(s.content.load_scoped(Some("official"), "vm-fixture").is_err());
    assert_eq!(versions(&s.content), vec!["0.1.0", "0.1.1"]);
    s.content.uninstall_package("official", "vm-fixture").unwrap();
    assert!(versions(&s.content).is_empty());
}

/// 与客户端导入入口相同的变异集：服务端导入一律拒绝，且已安装的有效版本不受影响。
#[test]
fn same_mutations_are_rejected_by_server_import_without_losing_installed_version() {
    let s = server();
    let v1 = fixture_json("vm-trigger");
    write_dropin(&s.packages_root, "vm-fixture", &v1, "vm-trigger");
    s.content.import().unwrap();
    let before = loaded_module_code(&s.content);

    let mutate = |f: &dyn Fn(&mut serde_json::Value)| {
        let mut bad = v1.clone();
        f(&mut bad);
        bad
    };
    let cases: Vec<(&str, serde_json::Value)> = vec![
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
    let mut outcomes = Vec::new();
    for (name, bad) in &cases {
        let dir = format!("bad-{name}");
        write_dropin(&s.packages_root, &dir, bad, "vm-trigger");
        let result = s.content.import();
        std::fs::remove_dir_all(s.packages_root.join(&dir)).unwrap();
        outcomes.push((name.to_string(), result.as_ref().err().cloned()));
        assert!(result.is_err(), "{name}: server import must reject: {result:?}");
        // 失败的导入不改变已安装的有效版本与装载副本。
        assert_eq!(versions(&s.content), vec!["0.1.0"], "{name}");
        assert_eq!(loaded_module_code(&s.content), before, "{name}");
    }
    for (name, error) in &outcomes {
        eprintln!("GT014_SERVER_REJECT {name}: {}", error.as_deref().unwrap_or(""));
    }

    // 同版本但重算过指纹的不同合法内容：客户端按「同版本包内容冲突」拒绝；
    // 服务端无论返回什么，已安装副本都不得被静默替换。
    // 取 v0.1.1（摸 2 张）的合法内容，把版本号改回 0.1.0 并重算包指纹。
    let mut rehashed: content_schema::ContentPackage =
        serde_json::from_value(fixture_json("vm-trigger-v2")).unwrap();
    rehashed.manifest.version = "0.1.0".into();
    rehashed.manifest.logic_hash = rehashed.compute_logic_hash();
    assert!(rehashed.validate().is_ok(), "rehashed variant must be self-consistent");
    let rehashed_value: serde_json::Value =
        serde_json::from_slice(&rehashed.to_json_bytes().unwrap()).unwrap();
    assert_ne!(rehashed_value["modules"][0]["code"], v1["modules"][0]["code"]);
    write_dropin(&s.packages_root, "vm-fixture", &rehashed_value, "vm-trigger-v2");
    let result = s.content.import();
    eprintln!("GT014_SERVER_SAME_VERSION_REHASHED import={result:?}");
    assert_eq!(versions(&s.content), vec!["0.1.0"]);
    assert_eq!(loaded_module_code(&s.content), before, "same-version re-import must not replace the installed copy");
}
