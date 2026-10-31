//! NC08：**全库技能映射清单**（数据驱动的「规则差异」基线 + 回归护栏）。
//!
//! 每个包在 `assets/cards.json` 的 `skills[]` 里逐条声明：有 `mapping` 的**已映射到宿主实现**，
//! 无 `mapping` 的带 `reason`（**已知缺口**）。
//!
//! 判据（跨全部包）：
//! 1. **总技能数 94**、**已映射 89**、**未映射 5**（2026-10-19 基线）；
//! 2. **每个未映射条目都有非空 `reason`**（**不允许静默缺口**）；
//! 3. 未映射的 5 条**逐一列出**（`longxue` 3 条 + `wudian` 2 条）。
//!
//! **注意**：本用例只验**声明面**（映射有无），**不验已映射者与原上游等价** ——
//! 等价性属各来源的「规则差异」逐条比对（U08 已证：**有实现 ≠ 可达 ≠ 等价**）。

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

#[test]
fn skill_mapping_inventory_matches_baseline() {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root())
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let mut total = 0usize;
    let mut mapped = 0usize;
    let mut unmapped: Vec<String> = Vec::new();
    let mut missing_reason: Vec<String> = Vec::new();

    for d in &dirs {
        let pkg = d.file_name().unwrap().to_string_lossy().to_string();
        let path = d.join("assets").join("cards.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(skills) = v.get("skills").and_then(|s| s.as_array()) else {
            continue;
        };
        for s in skills {
            total += 1;
            let name = s.get("name").and_then(|x| x.as_str()).unwrap_or("?");
            match s.get("mapping").and_then(|x| x.as_str()) {
                Some(m) if !m.is_empty() => mapped += 1,
                _ => {
                    unmapped.push(format!("{pkg}/{name}"));
                    let reason = s.get("reason").and_then(|x| x.as_str()).unwrap_or("");
                    if reason.trim().is_empty() {
                        missing_reason.push(format!("{pkg}/{name}"));
                    }
                }
            }
        }
    }

    eprintln!(
        "DBG 技能清单：总 {total}、已映射 {mapped}、未映射 {}",
        unmapped.len()
    );
    for u in &unmapped {
        eprintln!("DBG 未映射：{u}");
    }

    assert_eq!(total, 94, "总技能数应为基线 94");
    assert_eq!(mapped, 89, "已映射应为基线 89");
    assert_eq!(unmapped.len(), 5, "未映射应为基线 5：{unmapped:?}");
    assert!(
        missing_reason.is_empty(),
        "未映射条目必须有 reason：{missing_reason:?}"
    );
    // 基线里未映射的 5 条：longxue 3 + wudian 2
    let longxue = unmapped.iter().filter(|u| u.contains("longxue")).count();
    let wudian = unmapped.iter().filter(|u| u.contains("wudian")).count();
    assert_eq!(longxue, 3, "longxue 应有 3 条未映射：{unmapped:?}");
    assert_eq!(wudian, 2, "wudian 应有 2 条未映射：{unmapped:?}");
}
