//! Cardpile proportion table (`assets/pile.json`) and scale-factor loader (`T6①`).

use std::collections::BTreeMap;

use engine_native::content::pile::{PileEntry, PileSpec};
use serde_json::Value;

use crate::{ensure_symbol_capacity, int_in, invalid, parse_json, req_array, req_int, req_str, BridgeError};

/// 牌堆比例表装载（T6①）：已安装 cardpile 包的 `assets/pile.json` →
/// `PileSpec`。字段缺失/类型非法给出明确原因；未知牌名/花色、空条目与
/// 零张数由 [`engine_native::content::pile::extension_pack_from_pile`]
/// 在构建期校验。`counts` 键经 `BTreeMap` 排序——生成卡牌序（目录编号、
/// 发牌序的一部分）与解析器实现无关，仅由数据决定。
pub fn bridge_pile(json: &str) -> Result<PileSpec, BridgeError> {
    ensure_symbol_capacity()?;
    let parsed = parse_json(json)?;
    let source = req_str(&parsed, "source", "pile.json source")?.to_string();
    let total = req_int(&parsed, "total", "pile.json total", 1..=u64::from(u32::MAX))? as u32;
    let mut entries = Vec::new();
    for e in req_array(&parsed, "entries", "pile.json entries")? {
        let card = req_str(e, "card", "pile.json entries[].card")?.to_string();
        let counts_json = match e.get("counts") {
            None | Some(Value::Null) => {
                return Err(BridgeError::MissingField("pile.json entries[].counts"))
            }
            Some(Value::Object(m)) => m,
            Some(other) => {
                return Err(invalid(
                    "pile.json entries[].counts",
                    format!("须为对象，实为 {other}"),
                ))
            }
        };
        let counts: BTreeMap<String, u32> = counts_json
            .iter()
            .map(|(suit, n)| {
                let n = int_in(n, "pile.json entries[].counts", 1..=u64::from(u32::MAX))? as u32;
                Ok((suit.clone(), n))
            })
            .collect::<Result<BTreeMap<_, _>, BridgeError>>()?;
        entries.push(PileEntry {
            card,
            counts: counts.into_iter().collect(),
        });
    }
    Ok(PileSpec {
        source,
        total,
        entries,
    })
}

const PILE_FACTOR_CARDS: &[&str] = &[
    "guohe", "huosha", "jiu", "leisha", "nanman", "sha", "shan", "shunshou", "tao", "tiesuo",
    "wanjian", "wuxie",
];

/// 解析牌堆倍率。键必须是比例表里的牌名，值必须是有限非负数。
///
/// 缺某个牌名不是这里的错误：[`engine_native::content::pile::scale_plan`]
/// 会把它看成上游未配置，从而不追加牌。
pub fn parse_pile_factors(json: &str) -> Result<BTreeMap<String, f64>, String> {
    let value: Value = serde_json::from_str(json).map_err(|err| format!("factors: {err}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "factors: not an object".to_string())?;
    let mut factors = BTreeMap::new();
    for (key, raw) in obj {
        if !PILE_FACTOR_CARDS.contains(&key.as_str()) {
            return Err(format!("unknown pile factor {key}"));
        }
        let number = raw
            .as_f64()
            .ok_or_else(|| format!("pile factor {key} is not a number"))?;
        if !number.is_finite() || number < 0.0 {
            return Err(format!(
                "pile factor {key} must be a finite non-negative number"
            ));
        }
        factors.insert(key.clone(), number);
    }
    Ok(factors)
}
