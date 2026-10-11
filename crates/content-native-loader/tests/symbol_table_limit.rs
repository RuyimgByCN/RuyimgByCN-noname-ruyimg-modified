//! 符号表总量上限验收（ADR 0779，T6②）。
//!
//! 独立测试目标：名称符号表是进程级全局，本文件会把表填充到
//! [`SYMBOL_TABLE_LIMIT`]——必须独占进程运行，不能并入其他测试二进制
//! （同进程并行测试会互相污染读数；见 loader_acceptance 的内存界测试）。

use content_native_loader::{bridge_pack, BridgeError};

/// 与 loader_acceptance 同源的最小合法包：超限时唯一可能的原因是上限门。
const DEMO_JSON: &str = r#"{
  "pack": { "name": "demo-pack", "version": 1, "description": "装载器验收样例", "card_prefix": "dm_" },
  "cards": [
    { "name": "sha", "kind": "sha", "suit": "spade", "rank": 7 }
  ]
}"#;

#[test]
fn bridge_rejected_at_symbol_table_limit() {
    // ① 上限内：正常装载通过（证明拒装原因唯一=上限门）。
    assert!(
        bridge_pack(DEMO_JSON).is_ok(),
        "上限内装载应成功"
    );
    let before = engine_native::content::names::interned_count();
    assert!(before > 0, "装载后符号表应非空");

    // ② 填充到上限：unique 名称逐个 intern，直到 remaining 为 0。
    let limit = engine_native::content::names::SYMBOL_TABLE_LIMIT;
    let mut i = 0u64;
    while engine_native::content::names::remaining_capacity() > 0 {
        engine_native::content::names::intern(format!("symbol_limit_probe_{i}"));
        i += 1;
    }
    assert_eq!(
        engine_native::content::names::interned_count(),
        limit,
        "填充后应恰好达到总量上限"
    );

    // ③ 超限：装载被拒，错误为 SymbolTableLimit（fail-fast，非 panic）。
    let err = bridge_pack(DEMO_JSON).err();
    assert!(
        matches!(&err, Some(BridgeError::SymbolTableLimit { limit: l }) if *l == limit),
        "达上限后装载应返回 SymbolTableLimit: {err:?}"
    );

    // ④ 上限语义：remaining 为 0、计数不再增长（intern 已存在名称不新增）。
    assert_eq!(engine_native::content::names::remaining_capacity(), 0);
    let snapshot = engine_native::content::names::interned_count();
    engine_native::content::names::intern("symbol_limit_probe_0".to_string());
    assert_eq!(
        engine_native::content::names::interned_count(),
        snapshot,
        "重复 intern 同名不应增长"
    );
}
