//! NC08：**逐包「版本闸门」矩阵（identity 级）**——覆盖**全部 41 个包**，
//! 包括那些**没有引擎资产**的包（config/UI/VM 类）。
//!
//! 判据（每包，用它**真实的** `package.npack.json`）：
//! 1. 用 manifest 的 `namespace`/`package_id` + 语义版本构造 `ExtensionPack`（**内容为空**）；
//! 2. v1 建局 → `save()`；
//! 3. **v1+1 恢复必须被拒** ⇒ `SnapshotError::RuleVersionMismatch`；
//! 4. v1 回滚恢复成功且 `state_hash` 一致。
//!
//! **口径（重要）**：这是 **identity 级**（只验版本闸门 + 包身份），**不是 asset 级**——
//! 有资产的 22 个包另由 `nc08_upgrade_matrix`（22/22）验证真实内容装载与错版拒绝。
//! 本用例的价值在于：**19 个无引擎资产的包也能证明「内容锁按包版本拒绝错版」**，
//! 而不必假装它们有内容。

use std::path::PathBuf;

use engine_native::content::extension::{ExtensionDef, ExtensionPack};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// `"1.0.0"` ⇒ `10000 + 0 + 0`；解析失败返回 `None`（该包跳过并登记）。
fn parse_version(v: &str) -> Option<u32> {
    let mut it = v.split('.');
    let major: u32 = it.next()?.parse().ok()?;
    let minor: u32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let patch: u32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    Some(major.saturating_mul(10_000) + minor * 100 + patch)
}

fn name_of(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

#[test]
fn identity_version_gate_over_all_packages() {
    let root = root();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let mut passed = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for dir in &dirs {
        let dir_name = dir.file_name().unwrap().to_string_lossy().to_string();
        let manifest = dir.join("package.npack.json");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            skipped.push(format!("{dir_name}: 无 manifest"));
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            skipped.push(format!("{dir_name}: manifest 非 JSON"));
            continue;
        };
        let m = &v["manifest"];
        let ns = m["namespace"].as_str().unwrap_or("");
        let pid = m["package_id"].as_str().unwrap_or("");
        let ver = m["version"].as_str().unwrap_or("");
        let Some(version) = parse_version(ver) else {
            skipped.push(format!("{dir_name}: version={ver} 无法解析"));
            continue;
        };

        let ident = format!("{ns}/{pid}");
        let pack_v1 = ExtensionPack {
            def: ExtensionDef {
                name: name_of(&ident),
                version,
                description: "identity-gate",
            },
            generals: Vec::new(),
            cards: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: Vec::new(),
            enabled: true,
            init_hook: engine_native::content::extension::InitHook::None,
        };
        let pack_v2 = ExtensionPack {
            def: ExtensionDef {
                name: name_of(&ident),
                version: version + 1,
                description: "identity-gate",
            },
            generals: Vec::new(),
            cards: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: Vec::new(),
            enabled: true,
            init_hook: engine_native::content::extension::InitHook::None,
        };

        let e = match engine_native::Engine::new_with_extensions(71, 4, &[&pack_v1]) {
            Ok(e) => e,
            Err(err) => {
                failures.push(format!("{dir_name}: 建局失败 {err:?}"));
                continue;
            }
        };
        let hash = e.state_hash();
        let snap = e.save();
        match engine_native::Engine::restore_with_extensions(&snap, &[&pack_v2]) {
            Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
            Ok(_) => {
                failures.push(format!("{dir_name}: 错版恢复**不应成功**（内容锁未生效）"));
                continue;
            }
            Err(other) => {
                failures.push(format!("{dir_name}: 错版恢复应被拒，实得 {other:?}"));
                continue;
            }
        }
        match engine_native::Engine::restore_with_extensions(&snap, &[&pack_v1]) {
            Ok(r) if r.state_hash() == hash => passed += 1,
            Ok(_) => failures.push(format!("{dir_name}: 回滚后 hash 不一致")),
            Err(err) => failures.push(format!("{dir_name}: 回滚失败 {err:?}")),
        }
    }

    eprintln!(
        "DBG identity 闸门：目录 {}、通过 {}、跳过 {}、失败 {}",
        dirs.len(),
        passed,
        skipped.len(),
        failures.len()
    );
    for s in &skipped {
        eprintln!("NOTE identity 闸门跳过：{s}");
    }
    for f in &failures {
        eprintln!("DBG identity 闸门失败：{f}");
    }
    assert!(failures.is_empty(), "identity 闸门不应有失败：{failures:?}");
    assert_eq!(passed, dirs.len(), "每个包目录都应通过 identity 闸门");
}
