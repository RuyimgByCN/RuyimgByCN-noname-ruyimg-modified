//! NC08：**逐包「配置」矩阵**——把配置验收从 19 个卡牌包扩到**全部包目录**。
//!
//! 判据（每个 `packages/extensions/*/`）：
//! 1. `package.npack.json` 可解析，`manifest` 必备字段齐全（`namespace`/`package_id`/`version`）；
//! 2. **`(namespace, package_id)` 全库唯一**（`package_id` 允许跨 namespace 重名，
//!    见 `cardpile`/`boss-play`/`coin`）；
//! 3. `assets/config.json`（若有）**可解析**且为非空对象；
//! 4. **目录名与 namespace 一致**（`*-upstream-noname` ⇒ `upstream-noname`、`*-upstream-liuli` ⇒ `upstream-liuli`）。
//!
//! 边界：只验**元数据与配置文件的形状**；不验配置项语义（各包自有差异写在规则基准）。

use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 期望的 namespace（由目录名后缀决定）。
fn expected_ns(dir: &str) -> Option<&'static str> {
    if dir.ends_with("-upstream-noname") {
        Some("upstream-noname")
    } else if dir.ends_with("-upstream-liuli") {
        Some("upstream-liuli")
    } else {
        None
    }
}

#[test]
fn manifest_matrix_over_package_dirs() {
    let root = root();
    let mut dirs: Vec<String> = std::fs::read_dir(&root)
        .expect("read packages")
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    dirs.sort();

    let mut seen: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut problems: Vec<String> = Vec::new();
    let mut config_count = 0usize;
    let mut ok = 0usize;

    for dir in &dirs {
        let man = root.join(dir).join("package.npack.json");
        if !man.is_file() {
            problems.push(format!("{dir}: 缺 package.npack.json"));
            continue;
        }
        let text = match std::fs::read_to_string(&man) {
            Ok(t) => t,
            Err(e) => {
                problems.push(format!("{dir}: 读 manifest 失败 {e}"));
                continue;
            }
        };
        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                problems.push(format!("{dir}: manifest 非 JSON {e}"));
                continue;
            }
        };
        let m = match v.get("manifest") {
            Some(m) => m,
            None => {
                problems.push(format!("{dir}: manifest 缺 manifest 段"));
                continue;
            }
        };
        let ns = m.get("namespace").and_then(|x| x.as_str()).unwrap_or("");
        let pid = m.get("package_id").and_then(|x| x.as_str()).unwrap_or("");
        let ver = m.get("version").and_then(|x| x.as_str()).unwrap_or("");
        if ns.is_empty() || pid.is_empty() || ver.is_empty() {
            problems.push(format!("{dir}: namespace/package_id/version 有空值"));
            continue;
        }
        if let Some(exp) = expected_ns(dir) {
            if ns != exp {
                problems.push(format!("{dir}: namespace={ns} 与目录名期望 {exp} 不一致"));
            }
        }
        let key = (ns.to_string(), pid.to_string());
        if let Some(prev) = seen.insert(key.clone(), dir.clone()) {
            problems.push(format!(
                "{dir}: (namespace={ns}, package_id={pid}) 与 {prev} 重复"
            ));
        }
        // config.json 形状
        let cfg = root.join(dir).join("assets/config.json");
        if cfg.is_file() {
            config_count += 1;
            match std::fs::read_to_string(&cfg)
                .ok()
                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            {
                Some(serde_json::Value::Object(o)) if !o.is_empty() => {}
                Some(serde_json::Value::Object(_)) => {
                    problems.push(format!("{dir}: config.json 是空对象"))
                }
                Some(_) => problems.push(format!("{dir}: config.json 不是对象")),
                None => problems.push(format!("{dir}: config.json 无法解析")),
            }
        }
        ok += 1;
    }

    eprintln!(
        "DBG 配置矩阵：目录 {} 个、manifest 合格 {} 个、含 config.json {} 个、问题 {} 项",
        dirs.len(),
        ok,
        config_count,
        problems.len()
    );
    for p in &problems {
        eprintln!("DBG 配置矩阵问题：{p}");
    }
    assert!(
        problems.is_empty(),
        "配置矩阵不应有问题，实得 {} 项：{problems:?}",
        problems.len()
    );
    assert_eq!(ok, dirs.len(), "每个包目录都应通过 manifest 检查");
    assert!(config_count >= 3, "应至少有 3 个包带 assets/config.json");
}
