//! NC08 附加：**目录名双前缀回归**（承接台账第 162 行的普查与建议）。
//!
//! 背景：桥接把 `pack.card_prefix` 叠加到卡名；若包内名**已含同前缀**，目录名即成
//! `<prefix><prefix><name>`（如 `swd_swd_huanglinzhicong_73`）。
//!
//! 普查已确认 **3 个包**存在该现象（`shahai-mtg` 10/10、`shahai-swd` 89/90、`shahai-zhenfa` 3/8）。
//!
//! **本用例的口径**：
//! - 把「**已知存在双前缀的包**」列入白名单（基线）；
//! - 断言**白名单外没有新的双前缀** ⇒ 若将来其它包也出现，**测试失败**，提示修正桥接或包数据；
//! - 白名单内的**数量**也断言上限，防止**扩大**。
//!
//! 边界：本用例**不修复**双前缀（修复在桥接层，属 `content-native-loader` 面）；只作**回归防线**。

use std::collections::BTreeMap;
use std::path::PathBuf;

/// 已登记的「双前缀」包 → 期望数量上限（普查日 2026-10-19）。
fn whitelist() -> BTreeMap<&'static str, usize> {
    // 2026-10-19：桥接层已加「前缀去重」⇒ 白名单收紧为**空**（目录名中任何 `<prefix><prefix>` 都算违规）
    BTreeMap::new()
}

fn packs() -> Vec<(String, String, String)> {
    // (包名, card_prefix, cards_json)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path().join("assets/cards.json");
            let Ok(json) = std::fs::read_to_string(&p) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) else {
                continue;
            };
            let pref = v
                .pointer("/pack/card_prefix")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if pref.is_empty() {
                continue;
            }
            out.push((e.file_name().to_string_lossy().to_string(), pref, json));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// **回归**：白名单外不得新增双前缀；白名单内的数量不得扩大。
#[test]
fn card_name_double_prefix_regression() {
    let wl = whitelist();
    let mut rows: Vec<(String, usize, usize)> = Vec::new();
    for (pkg, pref, json) in packs() {
        // **检查目录名**（桥接后的实际名字），而不是包内名
        let pack = match content_native_loader::bridge_pack(&json) {
            Ok(b) => b.pack,
            Err(_) => continue,
        };
        let e = engine_native::Engine::new_with_extensions(31, 4, &[&pack]).expect("建局");
        let mut hit = 0usize;
        let mut total = 0usize;
        for c in &e.catalog().cards {
            total += 1;
            let doubled = format!("{pref}{pref}");
            if c.name.starts_with(&doubled) {
                hit += 1;
            }
        }
        rows.push((pkg, hit, total));
    }
    let mut failures: Vec<String> = Vec::new();
    for (pkg, hit, total) in &rows {
        match wl.get(pkg.as_str()) {
            Some(&cap) => {
                if *hit > cap {
                    failures.push(format!(
                        "{pkg}: 双前缀 {hit} 超出登记上限 {cap}（总 {total}）"
                    ));
                }
            }
            None => {
                if *hit > 0 {
                    failures.push(format!("{pkg}: **新出现**双前缀 {hit}/{total}（未登记）"));
                }
            }
        }
    }
    eprintln!(
        "DBG 双前缀回归：检查 {} 个有 card_prefix 的包；登记白名单 {} 个；违规 {} 项",
        rows.len(),
        wl.len(),
        failures.len()
    );
    for f in &failures {
        eprintln!("DBG 违规：{f}");
    }
    assert!(
        failures.is_empty(),
        "双前缀回归失败（需修正桥接或包数据，并更新白名单）：{failures:?}"
    );
}
