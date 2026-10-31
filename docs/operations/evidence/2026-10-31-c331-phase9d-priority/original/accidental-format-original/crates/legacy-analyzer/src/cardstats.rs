//! 壳扩展 `const card = {}` 段统计（T9）：扫描上游扩展 JS 的卡牌定义段，
//! 枚举卡牌键名与浅层字段，区分「纯数据卡」与「含行为卡」，供转换评估。
//!
//! 段形态（上游扩展 card.js 惯例，顶层键为单 tab 缩进）：
//! ```text
//! const card = {
//! 	<name>: {
//! 		type: "trick",
//! 		fullskin: true,
//! 		filterTarget(card, player, target) { ... },
//! 	},
//! };
//! ```

use serde::Serialize;
use std::collections::BTreeMap;

/// 单张卡牌的浅层统计。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardEntry {
    /// 卡牌键名（上游 lib.card 名）。
    pub name: String,
    /// `type:` 字段值（无则 None）。
    pub card_type: Option<String>,
    /// 是否声明 `fullskin: true`。
    pub fullskin: bool,
    /// 是否含行为函数键（filter/content/precontent/contentBefore 等）。
    pub has_behavior: bool,
}

/// 一个 `const card = {}` 段的统计。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CardSegmentStats {
    /// 识别出的卡牌（按文件内出现序）。
    pub entries: Vec<CardEntry>,
    /// 卡牌总数。
    pub total: usize,
    /// 纯数据卡数（无行为函数键——可直接数据直转的面）。
    pub data_only: usize,
    /// 含行为卡数（需逐卡语义评估）。
    pub with_behavior: usize,
    /// type 值分布。
    pub type_counts: BTreeMap<String, usize>,
}

/// 行为函数键（浅层判定启发式）。
const BEHAVIOR_KEYS: &[&str] = &[
    "filter",
    "filterTarget",
    "filterCard",
    "content",
    "contentBefore",
    "contentAfter",
    "precontent",
    "mod",
    "ai",
    "selectTarget",
    "enable",
    "usable",
    "viewAs",
];

/// 扫描 `const card = {` 段并统计。无该段返回空统计。
pub fn count_card_segment(source: &str) -> CardSegmentStats {
    let mut stats = CardSegmentStats::default();
    // 定位段头（忽略行首空白差异）
    let mut in_block = false;
    for line in source.lines() {
        // 段头必须是顶层无缩进的 `const card`（词边界：后随空格/=），
        // 不把函数体内的局部 `const card = ...` 当段头
        let is_head = line.starts_with("const card")
            && line
                .trim_start_matches("const card")
                .trim_start_matches(|c: char| c == ' ' || c == '\t')
                .starts_with('=');
        if is_head && line.contains('{') {
            in_block = true;
            break;
        }
    }
    if !in_block {
        return stats;
    }
    let mut current: Option<CardEntry> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !in_block {
            continue;
        }
        // 段尾：顶层 `};`（无缩进）
        if line == "};" {
            if let Some(e) = current.take() {
                push_entry(&mut stats, e);
            }
            break;
        }
        // 顶层键：单 tab 缩进 `<name>: {` 或 `<name>: true` 等
        let depth = line.len() - line.trim_start().len();
        let is_one_tab = depth == 1 && line.starts_with('\t');
        if is_one_tab {
            if let Some(name) = top_level_key(trimmed) {
                if let Some(e) = current.take() {
                    push_entry(&mut stats, e);
                }
                current = Some(CardEntry {
                    name,
                    card_type: None,
                    fullskin: false,
                    has_behavior: false,
                });
                continue;
            }
        }
        // 键内浅层字段
        if let Some(e) = current.as_mut() {
            if e.card_type.is_none() {
                if let Some(t) = extract_string_field(trimmed, "type") {
                    e.card_type = Some(t);
                }
            }
            if trimmed.starts_with("fullskin") && trimmed.contains("true") {
                e.fullskin = true;
            }
            for k in BEHAVIOR_KEYS {
                if trimmed.starts_with(k)
                    && trimmed
                        .get(k.len()..)
                        .is_some_and(|rest| rest.starts_with('(') || rest.starts_with(':'))
                {
                    // enable/usable: true 等数据值不算行为；函数形式才算
                    let is_fn = trimmed.contains("function")
                        || trimmed.contains("=>")
                        || trimmed.ends_with('{');
                    let is_data_true = trimmed.ends_with("true") || trimmed.ends_with("false");
                    if is_fn && !is_data_true {
                        e.has_behavior = true;
                    }
                    break;
                }
            }
        }
    }
    if let Some(e) = current.take() {
        push_entry(&mut stats, e);
    }
    stats
}

fn push_entry(stats: &mut CardSegmentStats, e: CardEntry) {
    stats.total += 1;
    if e.has_behavior {
        stats.with_behavior += 1;
    } else {
        stats.data_only += 1;
    }
    if let Some(t) = &e.card_type {
        *stats.type_counts.entry(t.clone()).or_insert(0) += 1;
    }
    stats.entries.push(e);
}

/// 顶层键名：`<name>: {`（对象开形式才算卡牌键）。
fn top_level_key(trimmed: &str) -> Option<String> {
    let (name, rest) = trimmed.split_once(':')?;
    let name = name.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }
    let rest = rest.trim();
    if rest == "{" || rest.starts_with("{") {
        Some(name.to_string())
    } else {
        None
    }
}

/// 提取 `<field>: "value"` 字段值。
fn extract_string_field(trimmed: &str, field: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(field)?.strip_prefix(':')?.trim();
    let v = rest.strip_prefix('"')?;
    let end = v.find('"')?;
    Some(v[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
import { lib, game, ui, get, ai, _status } from "noname";

const card = {
	diaobingqianjiang: {
		fullskin: true,
		type: "trick",
		enable: true,
		selectTarget: -1,
		filterTarget(card, player, target) {
			return player === target;
		},
		content() {
			"step 0";
		},
	},
	wangmeizhike: {
		fullskin: true,
		type: "basic",
	},
	sanjian: {
		type: "equip",
		fullskin: true,
		enable: true,
	},
};
"#;

    #[test]
    fn counts_names_types_and_behavior() {
        let s = count_card_segment(FIXTURE);
        assert_eq!(s.total, 3);
        assert_eq!(s.with_behavior, 1, "调兵遣将含 filterTarget/content");
        assert_eq!(s.data_only, 2, "望梅止渴/三尖为纯数据");
        assert_eq!(s.type_counts.get("trick"), Some(&1));
        assert_eq!(s.type_counts.get("basic"), Some(&1));
        assert_eq!(s.type_counts.get("equip"), Some(&1));
        assert_eq!(s.entries[0].name, "diaobingqianjiang");
        assert!(s.entries[0].has_behavior);
        assert_eq!(s.entries[1].card_type.as_deref(), Some("basic"));
        assert!(s.entries[1].fullskin);
        assert!(!s.entries[2].has_behavior);
    }

    #[test]
    fn no_segment_returns_empty() {
        let s = count_card_segment("const skill = {};");
        assert_eq!(s.total, 0);
        assert!(s.entries.is_empty());
    }

    #[test]
    fn ignores_local_const_card_inside_functions() {
        // 函数体内的局部 `const card = ...` 不构成段头
        let s = count_card_segment(
            "const skill = {\n\tx: {\n\t\tcontent() {\n\t\t\tconst card = get.autoViewAs({});\n\t\t}\n\t}\n};",
        );
        assert_eq!(s.total, 0);
    }

    #[test]
    fn plural_cards_declaration_counts_zero() {
        // `const cards = {};`（复数）不是卡牌段
        let s = count_card_segment(
            "import {} from \"noname\";\n\nconst cards = {};\nexport default cards;\n",
        );
        assert_eq!(s.total, 0);
    }
}
