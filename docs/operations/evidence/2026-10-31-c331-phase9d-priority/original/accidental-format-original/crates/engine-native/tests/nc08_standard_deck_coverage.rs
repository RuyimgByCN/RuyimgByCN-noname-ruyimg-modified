//! NC08 附加：**标准牌组构成核对**——「`CardKind` 已实现」vs「牌组含实例」。
//!
//! 背景（第 54 项附带发现）：`SavageAttack`/`ArcheryAttack` 的 **kind 与行为已实现**，
//! 但**标准牌组 124 张不含实例** ⇒ 夹具不可达。本用例**量化**这一现象：
//! 逐个 `CardKind` 检查标准建局（无扩展）的目录中**是否有实例**。
//!
//! 判据：**不预设「应当全有」**——只输出「**kind 有实现但牌组无实例**」的清单，作为**覆盖面台账**。

/// **正确口径**：直接从目录实例反查 `CardKind`，**不手写 kind 名**（手写名极易与枚举变体不符）。
///
/// 判据：输出「标准建局目录中**实际出现**的 kind 集合与实例数」；**不判定应否出现**——
/// 目录构成的正确性由既有 `catalog_inventory`（`standard_catalog_matches_declared_inventory`）守护。
#[test]
fn standard_deck_kind_instance_coverage() {
    let e = engine_native::Engine::new_with_extensions(29, 4, &[]).expect("标准建局");
    let cards = e.catalog().cards.len();
    use std::collections::BTreeMap;
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for i in 0..cards {
        let k = e.catalog().card_kind(i as engine_native::CardId);
        *by_kind.entry(format!("{k:?}")).or_insert(0) += 1;
    }
    eprintln!(
        "DBG 标准牌组：{cards} 张实例、{} 个不同 kind",
        by_kind.len()
    );
    let mut list: Vec<_> = by_kind.iter().collect();
    list.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("DBG 各 kind 实例数（降序）：{:?}", list);
    // **不设硬断言**：目录构成由 catalog_inventory 守护；本用例只提供可核的分布快照
    // 输出 AOE/关键锦囊的**目录名**（用于核对「按名字找卡」的口径）
    let mut probe_names: Vec<(String, String)> = Vec::new();
    for i in 0..cards {
        let k = e.catalog().card_kind(i as engine_native::CardId);
        let dbg = format!("{k:?}");
        if [
            "SavageAttack",
            "ArcheryAttack",
            "Duel",
            "Dismantle",
            "Snatch",
            "Nullify",
            "Bing",
            "Le",
            "ShanDian",
        ]
        .contains(&dbg.as_str())
        {
            probe_names.push((dbg, e.catalog().cards[i].name.to_string()));
        }
    }
    eprintln!("DBG 关键锦囊的目录名：{probe_names:?}");
    eprintln!(
        "NOTE 口径：本用例**只输出分布**；目录构成的正确性由 \
         `standard_catalog_matches_declared_inventory`（覆盖清单 `catalog_inventory`）守护"
    );
}
