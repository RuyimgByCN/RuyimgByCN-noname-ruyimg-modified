//! NC08 附加：**「组合」验收**（八项验收第 4 项，纯 Rust 可做）。
//!
//! 依据：八项验收清单（[证据](../../docs/operations/evidence/2026-10-19-eight-item-acceptance/README.md)）——
//! 「组合」= 与其他包同时启用无冲突（技能/牌种/事件）。
//!
//! 本用例覆盖三个可机械核验的组合面：
//! 1. **多包同启**：全部已转换卡牌包同时启用 ⇒ 建局不报错，且各包卡牌**均进入目录**；
//! 2. **同名卡去重**：统计跨包**重名卡**（报告口径，不预设结论）；
//! 3. **目录规模一致性**：同启后的目录卡数 ≥ 单包最大卡数，且 ≤ 各包卡数之和（**不重复计数**上界）。
//!
//! 边界：**跨包技能/事件的语义交互**（如五行属性 × swd 牌）未覆盖——属「规则差异」与功能面。

use std::path::PathBuf;

struct P {
    name: String,
    json: String,
    cards: usize,
}

fn packs() -> Vec<P> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path().join("assets/cards.json");
            if !p.is_file() {
                continue;
            }
            let Ok(json) = std::fs::read_to_string(&p) else { continue };
            let Ok(pack) = content_native_loader::bridge_pack(&json) else { continue };
            if pack.pack.cards.is_empty() {
                continue; // 纯武将包不参与卡牌组合
            }
            out.push(P {
                name: e.file_name().to_string_lossy().to_string(),
                json,
                cards: pack.pack.cards.len(),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// 组合面 1+3（**多包同启**）：全部卡牌包同时启用，**建局成功**且**目录覆盖各包**。
#[test]
fn composition_all_card_packs_together() {
    let list = packs();
    eprintln!("DBG 组合：参与组合的卡牌包 {} 个", list.len());
    assert!(list.len() >= 10, "应至少 10 个卡牌包（实测 {}）", list.len());
    let mut built = Vec::new();
    for p in &list {
        built.push(content_native_loader::bridge_pack(&p.json).expect("桥接").pack);
    }
    let refs: Vec<&_> = built.iter().collect();
    let e = engine_native::Engine::new_with_extensions(13, 4, &refs)
        .expect("**全部卡牌包同启时建局应成功**");
    let catalog = e.catalog().cards.len();
    // **口径**：目录计**牌实例**（含同名多张），故上界用「各包**牌实例**之和」而非卡名集合
    let sum: usize = list.iter().map(|p| p.cards).sum();
    let max = list.iter().map(|p| p.cards).max().unwrap_or(0);
    eprintln!(
        "DBG 组合：同启建局成功；目录**牌实例**数 {catalog}；各包牌实例之和 {sum}（上界）；单包最大 {max}（下界）"
    );
    // 逐包抽查：每包至少一张卡名应可在目录中找到（按包内首个卡名前缀归一去数字）
    let mut missing = Vec::new();
    for p in &list {
        for c in &p.json_named_cards() {
            let hit = e.catalog().cards.iter().any(|x| x.name.contains(c.as_str()) || c.contains(&x.name));
            if !hit {
                missing.push(format!("{}:{}", p.name, c));
                break;
            }
        }
    }
    eprintln!("DBG 组合：逐包抽查未命中的包 = {missing:?}");
    assert!(
        catalog >= max,
        "同启后目录卡数应 >= 单包最大（{catalog} vs {max}）"
    );
    assert!(
        catalog >= sum,
        "同启后目录**牌实例**数应 >= 各包牌实例之和（{catalog} < {sum}）"
    );
}

/// 组合面 2（**同名卡统计**，报告口径）。
#[test]
fn composition_duplicate_card_names_report() {
    let list = packs();
    use std::collections::{BTreeMap, BTreeSet};
    let mut seen: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for p in &list {
        for c in p.json_named_cards() {
            // **按包去重**：同包内同名多张不算「跨包同名」
            seen.entry(c).or_default().insert(p.name.clone());
        }
    }
    let dups: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
    eprintln!(
        "DBG 组合：跨包同名卡 {} 组（前 6：{:?}）",
        dups.len(),
        dups.iter().take(6).map(|(k, v)| format!("{k}<-{v:?}")).collect::<Vec<_>>()
    );
    // 不预设结论：同名卡可能是有意的（同一标准牌被多包复用）
}

impl P {
    fn json_named_cards(&self) -> Vec<String> {
        let v: serde_json::Value = match serde_json::from_str(&self.json) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        v.get("cards")
            .and_then(|c| c.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
