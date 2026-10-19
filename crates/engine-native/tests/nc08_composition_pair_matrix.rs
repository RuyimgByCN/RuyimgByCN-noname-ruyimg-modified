//! NC08 附加：**组合两两矩阵**（八项验收第 4 项的扩展，纯 Rust 可做）。
//!
//! 依据：组合验收（[证据](../../docs/operations/evidence/2026-10-19-composition-acceptance/README.md)）的待办
//! 「组合矩阵（两两/三三）」。本用例覆盖**全部两两组合**（10 个卡牌包 ⇒ 45 对）。
//!
//! 判据（**每对**）：
//! 1. 两包同时启用 ⇒ **建局成功**（不 panic、不 Err）；
//! 2. 目录**牌实例**数 = **两包之和 + 标准牌实例数**（标准牌数由「空包建局」测得，作为常量基准）；
//! 3. 两包各自**至少一张卡**可在目录找到（抽查）。
//!
//! 边界：**三三及更高阶**组合未覆盖；**跨包技能/事件语义交互**未覆盖（属功能面）。

use std::path::PathBuf;

struct P {
    name: String,
    json: String,
    instances: usize,
    sample: Option<String>,
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
                continue;
            }
            let instances = pack.pack.cards.len();
            let sample = pack.pack.cards.first().map(|c| c.name.to_string());
            out.push(P {
                name: e.file_name().to_string_lossy().to_string(),
                json,
                instances,
                sample,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// 标准牌实例数（空扩展建局时的目录规模）。
fn standard_cards() -> usize {
    let e = engine_native::Engine::new_with_extensions(19, 4, &[]).expect("空包建局");
    e.catalog().cards.len()
}

/// **全部两两组合**：每对建局成功且目录 = 两包之和 + 标准牌。
#[test]
fn composition_pair_matrix_all_pairs() {
    let list = packs();
    let std_cards = standard_cards();
    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();
    for i in 0..list.len() {
        for j in (i + 1)..list.len() {
            let a = content_native_loader::bridge_pack(&list[i].json).expect("桥接 a");
            let b = content_native_loader::bridge_pack(&list[j].json).expect("桥接 b");
            let pa = a.pack;
            let pb = b.pack;
            match engine_native::Engine::new_with_extensions(23, 4, &[&pa, &pb]) {
                Err(err) => failures.push(format!("{}+{}: 建局失败 {err:?}", list[i].name, list[j].name)),
                Ok(e) => {
                    let want = list[i].instances + list[j].instances + std_cards;
                    let got = e.catalog().cards.len();
                    if got != want {
                        failures.push(format!(
                            "{}+{}: 目录 {got} != 期望 {want}",
                            list[i].name, list[j].name
                        ));
                    }
                    for p in [&list[i], &list[j]] {
                        if let Some(s) = &p.sample {
                            if !e.catalog().cards.iter().any(|c| c.name == *s) {
                                failures.push(format!("{}+{}: 缺卡 `{s}`", list[i].name, list[j].name));
                            }
                        }
                    }
                }
            }
            checked += 1;
        }
    }
    eprintln!(
        "DBG 组合矩阵：包 {} 个 ⇒ 两两组合 {checked} 对；标准牌实例 {std_cards}；失败 {} 项",
        list.len(),
        failures.len()
    );
    for f in failures.iter().take(10) {
        eprintln!("DBG 组合矩阵失败：{f}");
    }
    assert!(failures.is_empty(), "两两组合应全部通过：{failures:?}");
    assert_eq!(checked, list.len() * (list.len() - 1) / 2, "组合对数应为 C(n,2)");
}
