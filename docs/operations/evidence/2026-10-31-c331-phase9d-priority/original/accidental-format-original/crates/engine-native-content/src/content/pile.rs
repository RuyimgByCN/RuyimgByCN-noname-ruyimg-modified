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
            return Err(ExtensionError::DuplicateCard {
                name: "pile:empty_entry",
            });
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
                cards.push(CardDef::basic(name, kind).with_suit_rank(suit, rank_counter));
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

/// 倍率追加的一组牌。`source` 是比例表里的牌名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleBucket {
    /// 比例表键，例如 `huosha`。
    pub source: String,
    /// 花色名。
    pub suit: String,
    /// 追加张数。零张的格子不出现。
    pub count: u32,
}

/// 上游 `arenaReady` 的倍率结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaleStatus {
    /// 某个牌名没有有限倍率。上游 `parseFloat` 得到 NaN，整段追加为空。
    Unconfigured,
    /// `dn <= 0`，不追加。
    NoAddition,
    /// 按花色追加的张数。
    Added(Vec<ScaleBucket>),
}

/// 倍率或点数无法变成牌。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PileScaleError {
    /// 倍率为负，或四舍五入后为负。
    Negative,
    /// 点数不在 1..=13，或条数与追加张数不一致。
    BadRank,
    /// 牌名或花色无法映射。
    BadCard,
}

/// 上游追加公式。
///
/// `getn = round(count * factor)`，`num` 为全部 `getn` 之和，
/// `dn = num * (deck_len - total) / (total - num)`，超过 1000 时截断。
/// `dn > 0` 时每个格子追加 `round(dn * getn / num)` 张。
/// 点数不在这里生成：上游用 `Math.random`，调用方另给 1..=13 的序列。
pub fn scale_plan(
    spec: &PileSpec,
    factors: &std::collections::BTreeMap<String, f64>,
    deck_len: u32,
) -> Result<ScaleStatus, PileScaleError> {
    let mut prepared = Vec::new();
    let mut num: u64 = 0;
    for entry in &spec.entries {
        let Some(factor) = factors.get(&entry.card).copied() else {
            return Ok(ScaleStatus::Unconfigured);
        };
        if !factor.is_finite() {
            return Ok(ScaleStatus::Unconfigured);
        }
        if factor < 0.0 {
            return Err(PileScaleError::Negative);
        }
        for (suit, count) in &entry.counts {
            let getn = (*count as f64 * factor).round();
            if getn < 0.0 || !getn.is_finite() || getn > u64::MAX as f64 {
                return Err(PileScaleError::Negative);
            }
            let getn = getn as u64;
            num = num.saturating_add(getn);
            prepared.push((entry.card.clone(), suit.clone(), getn));
        }
    }
    let denom = spec.total as f64 - num as f64;
    let numer = num as f64 * (deck_len as f64 - spec.total as f64);
    let mut dn = if denom == 0.0 {
        if numer > 0.0 {
            f64::INFINITY
        } else {
            f64::NAN
        }
    } else {
        numer / denom
    };
    if (dn.is_infinite() && dn.is_sign_positive()) || dn > 1000.0 {
        dn = 1000.0;
    }
    if !(dn > 0.0) || num == 0 {
        return Ok(ScaleStatus::NoAddition);
    }
    let mut added = Vec::new();
    for (source, suit, getn) in prepared {
        let n = (dn * getn as f64 / num as f64).round();
        if n <= 0.0 || !n.is_finite() || n > u32::MAX as f64 {
            continue;
        }
        added.push(ScaleBucket {
            source,
            suit,
            count: n as u32,
        });
    }
    if added.is_empty() {
        Ok(ScaleStatus::NoAddition)
    } else {
        Ok(ScaleStatus::Added(added))
    }
}

/// 把倍率结果写成可恢复的文本。顺序与 [`scale_plan`] 相同。
pub fn snapshot_scale(status: &ScaleStatus) -> String {
    match status {
        ScaleStatus::Unconfigured => "unconfigured\n".into(),
        ScaleStatus::NoAddition => "none\n".into(),
        ScaleStatus::Added(buckets) => {
            let mut text = String::from("added\n");
            for bucket in buckets {
                text.push_str(&format!(
                    "{} {} {}\n",
                    bucket.source, bucket.suit, bucket.count
                ));
            }
            text
        }
    }
}

/// 从 [`snapshot_scale`] 恢复。零张、未知牌名或坏行拒绝。
pub fn restore_scale(text: &str) -> Result<ScaleStatus, PileScaleError> {
    let mut lines = text.lines();
    match lines.next() {
        Some("unconfigured") => {
            if lines.next().is_some() {
                return Err(PileScaleError::BadCard);
            }
            Ok(ScaleStatus::Unconfigured)
        }
        Some("none") => {
            if lines.next().is_some() {
                return Err(PileScaleError::BadCard);
            }
            Ok(ScaleStatus::NoAddition)
        }
        Some("added") => {
            let mut buckets = Vec::new();
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                let mut parts = line.split(' ');
                let source = parts.next().ok_or(PileScaleError::BadCard)?;
                let suit = parts.next().ok_or(PileScaleError::BadCard)?;
                let count = parts
                    .next()
                    .ok_or(PileScaleError::BadCard)?
                    .parse::<u32>()
                    .map_err(|_| PileScaleError::BadCard)?;
                if parts.next().is_some()
                    || kind_of(source).is_none()
                    || suit_of(suit).is_none()
                    || count == 0
                {
                    return Err(PileScaleError::BadCard);
                }
                buckets.push(ScaleBucket {
                    source: source.into(),
                    suit: suit.into(),
                    count,
                });
            }
            if buckets.is_empty() {
                return Err(PileScaleError::BadCard);
            }
            Ok(ScaleStatus::Added(buckets))
        }
        _ => Err(PileScaleError::BadCard),
    }
}

/// 把倍率追加变成目录牌。`ranks` 的长度必须等于追加总张数，每项为 1..=13。
pub fn extension_pack_from_scale(
    def: ExtensionDef,
    name_prefix: &str,
    buckets: &[ScaleBucket],
    ranks: &[u8],
) -> Result<ExtensionPack, PileScaleError> {
    let total: usize = buckets.iter().map(|bucket| bucket.count as usize).sum();
    if ranks.len() != total || ranks.iter().any(|rank| !(1..=13).contains(rank)) {
        return Err(PileScaleError::BadRank);
    }
    let mut cards = Vec::with_capacity(total);
    let mut rank_at = 0usize;
    for bucket in buckets {
        let kind = kind_of(&bucket.source).ok_or(PileScaleError::BadCard)?;
        let suit = suit_of(&bucket.suit).ok_or(PileScaleError::BadCard)?;
        for i in 0..bucket.count {
            let name: &'static str = String::leak(format!(
                "{name_prefix}_scale_{}_{}_{i}",
                bucket.source, bucket.suit
            ));
            cards.push(CardDef::basic(name, kind).with_suit_rank(suit, ranks[rank_at]));
            rank_at += 1;
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
        let def = ExtensionDef {
            name: "t",
            version: 1,
            description: "",
        };
        let pack = extension_pack_from_pile(def, "t", &spec()).unwrap();
        assert_eq!(pack.card_count(), 3);
        let names: std::collections::BTreeSet<&str> = pack.cards.iter().map(|c| c.name).collect();
        assert_eq!(names.len(), 3, "卡牌名必须唯一");
        assert!(pack.cards.iter().all(|c| c.kind == CardKind::Jiu));
    }

    #[test]
    fn scale_plan_follows_upstream_formula() {
        use std::collections::BTreeMap;
        let spec = PileSpec {
            source: "t".into(),
            total: 160,
            entries: vec![PileEntry {
                card: "sha".into(),
                counts: vec![("diamond".into(), 6)],
            }],
        };
        let mut factors = BTreeMap::new();
        factors.insert("sha".into(), 1.0);
        match scale_plan(&spec, &factors, 207).unwrap() {
            ScaleStatus::Added(buckets) => {
                assert_eq!(buckets.len(), 1);
                assert_eq!(buckets[0].count, 2, "round(6 * 47 / 154) = 2");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            scale_plan(&spec, &factors, 160).unwrap(),
            ScaleStatus::NoAddition
        );
        assert_eq!(
            scale_plan(&spec, &BTreeMap::new(), 207).unwrap(),
            ScaleStatus::Unconfigured
        );
        let def = ExtensionDef {
            name: "t",
            version: 1,
            description: "",
        };
        let ScaleStatus::Added(buckets) = scale_plan(&spec, &factors, 207).unwrap() else {
            panic!("added");
        };
        let pack = extension_pack_from_scale(def, "t", &buckets, &[7, 8]).unwrap();
        assert_eq!(pack.card_count(), 2);
        assert_eq!(pack.cards[0].rank, 7);
        assert_eq!(pack.cards[1].rank, 8);
        assert!(extension_pack_from_scale(
            ExtensionDef {
                name: "t",
                version: 1,
                description: ""
            },
            "t",
            &buckets,
            &[0, 1],
        )
        .is_err());
    }

    #[test]
    fn pile_spec_rejects_invalid_entries() {
        let def = ExtensionDef {
            name: "t",
            version: 1,
            description: "",
        };
        let mut bad = spec();
        bad.entries[0].counts[0].1 = 0;
        assert!(extension_pack_from_pile(def.clone(), "t", &bad).is_err());
        bad.entries[0].card = "unknown_card".into();
        assert!(extension_pack_from_pile(def, "t", &bad).is_err());
    }
}
