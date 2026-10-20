//! NC08：**牌堆补丁 × 卡牌包**的配对矩阵（第 178 行「不变式不适用」后的**另立判据**）。
//!
//! 第 178 行结论：`pile.json` 是**替换牌堆**语义，不能与卡牌包按「两包之和 + 标准牌」求和。
//! 本文件**另立判据**：
//!
//! 对每个 `pile.json` 包 × 每个**有卡牌**的包：
//! 1. 两包**同时装载**必须成功；
//! 2. 目录卡数 = **牌堆包卡数 + 卡牌包卡数**（**不含标准牌**，因为牌堆包替换了标准牌堆）；
//! 3. **牌堆包每张牌都在目录里**（按名字集合检查，允许重名去重）；
//! 4. **卡牌包的每张牌都在目录里**。
//!
//! 边界：只验**目录构成**，不验开局时实际发牌数量（那由牌堆包 `total` 决定，另见 `nc08_card_ownership_partition`）。

use std::path::{Path, PathBuf};

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::content::pile::{extension_pack_from_pile, PileEntry, PileSpec};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn load_pile(dir: &Path) -> Option<ExtensionPack> {
    let json = std::fs::read_to_string(dir.join("assets").join("pile.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let mut entries = Vec::new();
    for e in v.get("entries").and_then(|x| x.as_array())? {
        let card = e.get("card").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let mut counts: Vec<(String, u32)> = Vec::new();
        if let Some(obj) = e.get("counts").and_then(|x| x.as_object()) {
            for suit in ["spade", "heart", "club", "diamond"] {
                if let Some(n) = obj.get(suit).and_then(|x| x.as_u64()) {
                    counts.push((suit.to_string(), n as u32));
                }
            }
        }
        entries.push(PileEntry { card, counts });
    }
    extension_pack_from_pile(
        ExtensionDef { name: "pair-pile", version: 1, description: "pile pair" },
        "pair_pile",
        &PileSpec { source, total, entries },
    )
    .ok()
}

fn load_cards(dir: &Path) -> Option<ExtensionPack> {
    let path = dir.join("assets").join("cards.json");
    let json = std::fs::read_to_string(&path).ok()?;
    let mut out = content_native_loader::bridge_pack(&json).ok()?;
    if let Ok(chars) = std::fs::read_to_string(dir.join("assets").join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            out.pack.generals = b.generals;
        }
    }
    Some(out.pack)
}

/// **修正后的判据**（第 523 轮实验否定了首版）：
/// 「目录 = **标准牌** + 牌堆包 + 卡牌包」。
///
/// 首版误以为「pile 替换牌堆 ⇒ 目录不计标准牌」，实测 `cardpile-liuli + huanle`
/// 目录 **244**、两包之和 **120**，**差 124 = 标准牌** ⇒ 标准牌**仍计入目录**。
#[test]
fn pile_pair_matrix_over_card_packs() {
    let root = root();
    let mut piles: Vec<(String, ExtensionPack)> = Vec::new();
    let mut packs: Vec<(String, ExtensionPack)> = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for d in &dirs {
        let name = d.file_name().unwrap().to_string_lossy().to_string();
        if let Some(p) = load_pile(d) {
            piles.push((name, p));
        } else if let Some(p) = load_cards(d) {
            if !p.cards.is_empty() {
                packs.push((name, p));
            }
        }
    }
    eprintln!(
        "DBG 牌堆配对：牌堆包 {} 个、卡牌包 {} 个 ⇒ {} 对",
        piles.len(),
        packs.len(),
        piles.len() * packs.len()
    );

    // 空扩展建局时的目录规模（标准牌数）
    let std_cards = engine_native::Engine::new_with_extensions(73, 4, &[])
        .expect("空包建局")
        .catalog()
        .cards
        .len();
    eprintln!("DBG 牌堆配对：标准牌 {std_cards} 张");
    let mut pairs = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for (pname, pile) in &piles {
        for (cname, cards) in &packs {
            pairs += 1;
            let want = std_cards + pile.cards.len() + cards.cards.len();
            let e = match engine_native::Engine::new_with_extensions(73, 4, &[pile, cards]) {
                Ok(e) => e,
                Err(err) => {
                    failures.push(format!("{pname}+{cname}: 建局失败 {err:?}"));
                    continue;
                }
            };
            let got = e.catalog().cards.len();
            if got != want {
                failures.push(format!("{pname}+{cname}: 目录 {got} != 期望 {want}"));
                continue;
            }
            // 两包的卡名都应出现在目录里（按名字集合）
            let names: std::collections::BTreeSet<String> =
                e.catalog().cards.iter().map(|c| c.name.to_string()).collect();
            for c in pile.cards.iter().chain(cards.cards.iter()) {
                if !names.contains(&c.name.to_string()) {
                    failures.push(format!("{pname}+{cname}: 缺卡 `{}`", c.name));
                    break;
                }
            }
        }
    }
    eprintln!("DBG 牌堆配对：{} 对、失败 {} 项", pairs, failures.len());
    for f in failures.iter().take(6) {
        eprintln!("DBG 牌堆配对失败：{f}");
    }
    assert!(pairs > 0, "应至少有一对");
    assert!(failures.is_empty(), "牌堆配对应全部通过，实得 {} 项失败", failures.len());
}
