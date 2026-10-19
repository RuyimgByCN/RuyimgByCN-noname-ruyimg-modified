//! NC08 附加：**转换缺口回归测试**——把缺口清单固化为**可执行事实**。
//!
//! 依据：`docs/operations/evidence/2026-10-19-conversion-gap-inventory/README.md`
//! （台账第 117 行清单：**10 包 / 49 项**确认缺口）。
//!
//! **做法**：**直接读取该证据文件**（`## <包>（N）` 小节 + 其 ```代码块``` 内的键列表），
//! 对每个键断言：**该包的目录中不存在对应卡**（按 `contains` 匹配，避免前缀差异漏判）。
//!
//! **价值**：缺口一旦被**转换**，本测试会**失败** ⇒ **缺口闭合可被自动发现**（不依赖人工比对文档）。

use std::path::PathBuf;

/// 解析证据文件：返回 `(包名, 键列表)`。
fn parse_inventory() -> Vec<(String, Vec<String>)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/operations/evidence/2026-10-19-conversion-gap-inventory/README.md");
    let text = std::fs::read_to_string(&path).expect("读缺口清单证据");
    let mut out = Vec::new();
    let mut cur: Option<(String, Vec<String>)> = None;
    let mut in_block = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(entry) = cur.take() {
                out.push(entry);
            }
            let pkg = rest.split('（').next().unwrap_or(rest).trim().to_string();
            cur = Some((pkg, Vec::new()));
            continue;
        }
        if line.starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block {
            if let Some((_, names)) = cur.as_mut() {
                for k in line.split_whitespace() {
                    names.push(k.to_string());
                }
            }
        }
    }
    if let Some(entry) = cur.take() {
        out.push(entry);
    }
    out.retain(|(_, n)| !n.is_empty());
    out
}

fn engine_for(pkg: &str) -> engine_native::Engine {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 cards");
    let mut pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    engine_native::Engine::new_with_extensions(7, 4, &[&pack]).expect("建局")
}

/// **缺口回归**：清单内每个键在其包目录中**应不存在**。
#[test]
fn conversion_gap_inventory_is_still_absent() {
    let inventory = parse_inventory();
    let mut total = 0usize;
    let mut now_present: Vec<(String, String)> = Vec::new();
    for (pkg, keys) in &inventory {
        let e = engine_for(pkg);
        for k in keys {
            total += 1;
            // 归一化：去前缀与尾部数字（与清单口径一致）
            let norm = |n: &str| -> String {
                let mut s = n.to_lowercase();
                for p in [
                    "swd_", "gw_", "gj_", "ycn_", "wd_", "hl_", "xz_", "ow_", "hearth_", "mtg_",
                    "zf_", "hslingjian_",
                ] {
                    if s.starts_with(p) {
                        s = s[p.len()..].to_string();
                        break;
                    }
                }
                s.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
            };
            let want = norm(k);
            let present = e.catalog().cards.iter().any(|c| norm(&c.name) == want);
            if present {
                now_present.push((pkg.clone(), k.clone()));
            }
        }
    }
    eprintln!(
        "DBG 缺口回归：清单 {} 包 / {total} 项；**已出现在目录中的（即已转换）** {} 项",
        inventory.len(),
        now_present.len()
    );
    for (pkg, k) in &now_present {
        eprintln!("DBG   已转换（请更新清单并启用对应行为面用例）：{pkg} / {k}");
    }
    assert!(
        now_present.is_empty(),
        "缺口清单中的以下键**已出现在目录中** ⇒ 缺口已闭合，应更新清单（第 117 行）与台账：{now_present:?}"
    );
}
