//! G1 cardpile 桥接（ADR 0776）：上游 cardpile 扩展的牌堆比例数据 →
//! 原生扩展包。
//!
//! 上游 `cardpile/extension.js`（noname 与 noname-liuli 两仓数据一致）
//! 定义 160 张牌堆的比例表：每条目 = 牌名 + 各花色数量；`huosha`/
//! `leisha` 是带火/雷属性的杀，`jiu` 为独立牌种。原生映射：每个
//! 「牌名×花色×序号」单元生成一张**唯一物理牌**（目录牌 = 实体牌），
//! 比例只决定各单元的张数；点数以确定性流水号生成（上游为运行时
//! 随机点数，原生以规则随机流在建局洗牌环节承接随机性）。
//!
//! JSON 解析不在本模块（engine-native 不依赖 serde）——宿主/测试把
//! 已安装包资产解析为 `PileSpec` 后交给 [`extension_pack_from_pile`]。

use super::extension::{ExtensionDef, ExtensionError, ExtensionPack};
use crate::catalog::{CardDef, CardKind, Suit};

/// 牌堆比例表的单个条目（对应上游 `data[i] = { suit: count }`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PileEntry {
    /// 上游牌名（`sha`/`huosha`/`leisha`/`jiu`/`shan`/`tao`/`wanjian`/
    /// `nanman`/`guohe`/`shunshou`/`wuxie`/`tiesuo`）。
    pub card: String,
    /// 花色（上游命名 `spade`/`heart`/`club`/`diamond`）→ 张数。
    pub counts: Vec<(String, u32)>,
}

/// 牌堆规格（对应上游 `data` 表 + `total`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PileSpec {
    /// 来源标识（进入生成卡牌名前缀，保证双来源不冲突）。
    pub source: String,
    /// 上游目标总张数（校验用；原生牌数 = Σ各花色数量）。
    pub total: u32,
    pub entries: Vec<PileEntry>,
}

fn suit_of(name: &str) -> Option<Suit> {
    Some(match name {
        "spade" => Suit::Spade,
        "heart" => Suit::Heart,
        "club" => Suit::Club,
        "diamond" => Suit::Diamond,
        _ => return None,
    })
}

fn kind_of(card: &str) -> Option<CardKind> {
    CardKind::from_key(card)
}

/// 由牌堆规格构建扩展包（校验非法比例：未知牌名/花色、数量为 0）。
///
/// 生成的卡牌名为 `{source}_{card}_{suit}_{i}`——跨来源唯一，不与
/// 基础目录或另一来源冲突；名称经泄漏为 `'static`（开局一次性的
/// 有界分配，与目录同级生命周期）。
pub fn extension_pack_from_pile(
    def: ExtensionDef,
    name_prefix: &str,
    spec: &PileSpec,
) -> Result<ExtensionPack, ExtensionError> {
    if spec.source.is_empty() || spec.entries.is_empty() {
        return Err(ExtensionError::DuplicateCard { name: "pile:empty" });
    }
    let mut cards: Vec<CardDef> = Vec::new();
    let mut rank_counter: u8 = 0;
    for entry in &spec.entries {
        let kind = kind_of(&entry.card).ok_or(ExtensionError::DuplicateCard {
            name: "pile:unknown_card",
        })?;
        if entry.counts.is_empty() {
            return Err(ExtensionError::DuplicateCard { name: "pile:empty_entry" });
        }
        for (suit_name, count) in &entry.counts {
            let count = *count;
            let suit = suit_of(suit_name).ok_or(ExtensionError::DuplicateCard {
                name: "pile:unknown_suit",
            })?;
            if count == 0 {
                return Err(ExtensionError::DuplicateCard {
                    name: "pile:zero_count",
                });
            }
            for i in 0..count {
                rank_counter = rank_counter % 13 + 1;
                let name: &'static str =
                    String::leak(format!("{name_prefix}_{}_{}_{}", entry.card, suit_name, i));
                cards.push(
                    CardDef::basic(name, kind)
                        .with_suit_rank(suit, rank_counter),
                );
            }
        }
    }
    Ok(ExtensionPack {
        def,
        generals: Vec::new(),
        cards,
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: crate::content::extension::InitHook::None,
        enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> PileSpec {
        PileSpec {
            source: "test".into(),
            total: 3,
            entries: vec![PileEntry {
                card: "jiu".into(),
                counts: vec![("spade".into(), 2), ("club".into(), 1)],
            }],
        }
    }

    #[test]
    fn pile_spec_builds_unique_cards() {
        let def = ExtensionDef { name: "t", version: 1, description: "" };
        let pack = extension_pack_from_pile(def, "t", &spec()).unwrap();
        assert_eq!(pack.card_count(), 3);
        let names: std::collections::BTreeSet<&str> =
            pack.cards.iter().map(|c| c.name).collect();
        assert_eq!(names.len(), 3, "卡牌名必须唯一");
        assert!(pack.cards.iter().all(|c| c.kind == CardKind::Jiu));
    }

    #[test]
    fn pile_spec_rejects_invalid_entries() {
        let def = ExtensionDef { name: "t", version: 1, description: "" };
        let mut bad = spec();
        bad.entries[0].counts[0].1 = 0;
        assert!(extension_pack_from_pile(def.clone(), "t", &bad).is_err());
        bad.entries[0].card = "unknown_card".into();
        assert!(extension_pack_from_pile(def, "t", &bad).is_err());
    }
}
