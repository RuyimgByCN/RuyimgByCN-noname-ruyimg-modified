//! 结构守卫（NC07，裁决第 11 项，规则 v327）：`state_encode.rs` 写入档案的
//! `PlayerState`（`p.`）与 `TurnState`（`t.`）字段，必须满足其一：
//! ① 在 `engine/hash.rs` 中以 `.字段名` 被引用（参与规范摘要）；② 列入下方
//! **排除名单**并写明理由（证明不影响规则结算，如纯表现缓存）。
//!
//! 2026-10-29 v327 批后排除名单为**空**：字段审计的 10 个遗漏已全部入摘要。
//! 此后新增入档字段而不入摘要会让本测试失败——防止「入档未入摘要」再次积累
//! （审计与修复见[字段审计](../../../../docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash/archive-vs-hash-field-audit.log)
//! 与 [v327 证据](../../../../docs/operations/evidence/2026-10-29-nc07-archive-hash-fields/README.md)）。

/// PlayerState 排除名单：字段名 → 理由（当前为空）。
const EXCLUDED_PLAYER: &[(&str, &str)] = &[];
/// TurnState 排除名单：字段名 → 理由（当前为空）。
const EXCLUDED_TURN: &[(&str, &str)] = &[];

/// 提取 `var.field` 引用的字段名（跳过常见方法调用；口径与字段审计脚本一致）。
fn fields_referenced(src: &str, var: char) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for i in 0..bytes.len() {
        if bytes[i] == b'.' && i >= 2 && bytes[i - 1] == var as u8 {
            let prev = bytes[i - 2];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' {
                continue; // `xp.` / `.p.` 等非独立标识符
            }
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let name = &src[i + 1..j];
            if !name.is_empty()
                && name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
                && !matches!(
                    name,
                    "len" | "iter" | "is_empty" | "clone" | "as_ref" | "map" | "copied"
                        | "unwrap" | "unwrap_or" | "insert" | "push" | "contains" | "clear"
                        | "first" | "last" | "get" | "to_vec" | "as_u8" | "saturating"
                )
            {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[test]
fn archived_state_fields_are_hashed_or_excluded() {
    let encode = include_str!("../src/archive/state_encode.rs");
    let hash = include_str!("../src/engine/hash.rs");
    for (var, label, excluded) in [
        ('p', "PlayerState", EXCLUDED_PLAYER),
        ('t', "TurnState", EXCLUDED_TURN),
    ] {
        let mut missing = Vec::new();
        for field in fields_referenced(encode, var) {
            let hashed = hash.match_indices(&format!(".{field}")).next().is_some();
            let excluded_by = excluded.iter().find(|(name, _)| *name == field);
            if !hashed && excluded_by.is_none() {
                missing.push(field);
            }
            if let Some((name, reason)) = excluded_by {
                // 排除名单中的字段必须真的不在摘要里（防止名单过期）。
                assert!(
                    !hashed,
                    "{label}.{name} 已入摘要，应从排除名单移除（理由已失效：{reason}）"
                );
            }
        }
        assert!(
            missing.is_empty(),
            "{label} 入档未入摘要且不在排除名单：{missing:?}；\
             入摘要（改 engine/hash.rs）或列入排除名单并写明理由"
        );
    }
}
