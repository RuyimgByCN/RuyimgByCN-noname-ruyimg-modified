//! NC08：**纯武将包 × 卡牌包**的配对矩阵（沿「层级拆分 + 另立判据」的思路）。
//!
//! 背景：纯武将包（`cards: []`，只有 `generals`）在第 178 行被排除在卡牌配对之外，
//! 理由是「贡献 0 张卡 ⇒ 求和不变式无信息量」。但**它们仍有可验证的组合效应**：
//!
//! 判据（每个「纯武将包 × 有卡牌包」）：
//! 1. 两包同载**必须成功**；
//! 2. **目录卡数 = 标准牌 + 卡牌包卡数**（纯武将包贡献 **0** 张）；
//! 3. **卡牌包的每张卡都在目录里**；
//! 4. **纯武将包的每名武将都在目录的 `extension_generals` 里**（按名字）。
//!
//! 边界：只验**目录构成**；不验武将技能行为（那属第⑦层样本）。

use std::path::{Path, PathBuf};

use engine_native::content::extension::ExtensionPack;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn load(dir: &Path) -> Option<ExtensionPack> {
    let assets = dir.join("assets");
    let mut json = String::new();
    for f in ["cards.json", "character.json"] {
        let p = assets.join(f);
        if p.is_file() {
            if let Ok(t) = std::fs::read_to_string(&p) {
                json = t;
                break;
            }
        }
    }
    if json.is_empty() {
        return None;
    }
    let mut out = content_native_loader::bridge_pack(&json).ok()?;
    if let Ok(chars) = std::fs::read_to_string(assets.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            out.pack.generals = b.generals;
        }
    }
    Some(out.pack)
}

#[test]
fn general_pair_matrix_with_card_packs() {
    let root = root();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let mut generals_only: Vec<(String, ExtensionPack)> = Vec::new();
    let mut card_packs: Vec<(String, ExtensionPack)> = Vec::new();
    for d in &dirs {
        let name = d.file_name().unwrap().to_string_lossy().to_string();
        if let Some(p) = load(d) {
            if p.cards.is_empty() && !p.generals.is_empty() {
                generals_only.push((name, p));
            } else if !p.cards.is_empty() {
                card_packs.push((name, p));
            }
        }
    }
    let std_cards = engine_native::Engine::new_with_extensions(79, 4, &[])
        .expect("空包建局")
        .catalog()
        .cards
        .len();
    eprintln!(
        "DBG 武将配对：纯武将包 {} 个、卡牌包 {} 个 ⇒ {} 对；标准牌 {}",
        generals_only.len(),
        card_packs.len(),
        generals_only.len() * card_packs.len(),
        std_cards
    );

    let mut pairs = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for (gname, gp) in &generals_only {
        for (cname, cp) in &card_packs {
            pairs += 1;
            let want = std_cards + cp.cards.len();
            let e = match engine_native::Engine::new_with_extensions(79, 4, &[gp, cp]) {
                Ok(e) => e,
                Err(err) => {
                    failures.push(format!("{gname}+{cname}: 建局失败 {err:?}"));
                    continue;
                }
            };
            let got = e.catalog().cards.len();
            if got != want {
                failures.push(format!("{gname}+{cname}: 目录 {got} != 期望 {want}"));
                continue;
            }
            let card_names: std::collections::BTreeSet<String> =
                e.catalog().cards.iter().map(|c| c.name.to_string()).collect();
            for c in &cp.cards {
                if !card_names.contains(&c.name.to_string()) {
                    failures.push(format!("{gname}+{cname}: 缺卡 `{}`", c.name));
                    break;
                }
            }
            let gen_names: std::collections::BTreeSet<String> = e
                .catalog()
                .extension_generals
                .iter()
                .map(|g| g.name.to_string())
                .collect();
            for g in &gp.generals {
                if !gen_names.contains(&g.name.to_string()) {
                    failures.push(format!("{gname}+{cname}: 缺武将 `{}`", g.name));
                    break;
                }
            }
        }
    }
    eprintln!("DBG 武将配对：{} 对、失败 {} 项", pairs, failures.len());
    for f in failures.iter().take(6) {
        eprintln!("DBG 武将配对失败：{f}");
    }
    assert!(pairs > 0, "应至少有一对（否则说明没有纯武将包）");
    assert!(failures.is_empty(), "武将配对应全部通过，实得 {} 项失败", failures.len());
}
