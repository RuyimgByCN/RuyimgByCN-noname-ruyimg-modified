//! P1：坏存档必须在 WASM 边界受控拒绝，且不使原句柄/原局失效。
//!
//! 反例（2026-10-24 架构审查）：旧 `hex_decode` 按字符串下标切片，
//! `restore_hex("0")` / `restore_hex("中")` 在 WASM 内 panic 为
//! `RuntimeError: unreachable`，随后同句柄 `save_hex()` 报 Rust 借用错误。
//! 本测试直接驱动与导出相同的实现路径（`restore_hex_checked`），覆盖奇数、
//! 非 ASCII、非法字符、截断归档和合法归档往返。

use engine_native_wasm::{engine_from_archive, NativeEngine};

fn assert_handle_intact(engine: &mut NativeEngine, seat: u8, before: &str, before_view: &str) {
    assert_eq!(engine.save_hex(), before, "坏恢复不得改变原局存档");
    assert_eq!(
        engine.view_json(seat).expect("坏恢复后视图仍可读"),
        before_view,
        "坏恢复不得改变座位投影"
    );
    engine
        .legal_actions_json(seat)
        .expect("坏恢复后合法动作仍可查询");
}

#[test]
fn malformed_hex_is_rejected_without_invalidating_handle() {
    let mut engine = NativeEngine::new(2, 20261024).expect("建局");
    let before = engine.save_hex();
    let before_view = engine.view_json(0).expect("初始视图");

    for bad in ["0", "中", "zz", "0g", " ", "0x"] {
        let error = engine
            .restore_hex_checked(bad)
            .expect_err("坏十六进制必须受控拒绝");
        assert!(!error.is_empty(), "{bad:?} 需要可读错误");
        assert_handle_intact(&mut engine, 0, &before, &before_view);
    }
}

#[test]
fn truncated_archive_is_rejected_and_original_match_continues() {
    let mut engine = NativeEngine::new(3, 7).expect("建局");
    let before = engine.save_hex();
    let before_view = engine.view_json(1).expect("初始视图");
    let truncated = &before[..before.len() - 2];

    let error = engine
        .restore_hex_checked(truncated)
        .expect_err("截断归档必须拒绝");
    assert!(!error.is_empty());
    assert_handle_intact(&mut engine, 1, &before, &before_view);

    // 原局仍可继续推进（不是只能读）。
    let stepped = engine.auto_step().expect("原局仍可推进");
    assert!(stepped.contains("\"kind\":\"plan\""), "{stepped}");
}

#[test]
fn valid_archive_roundtrip_still_works_after_rejections() {
    let mut engine = NativeEngine::new(2, 11).expect("建局");
    let saved = engine.save_hex();
    let before_view = engine.view_json(0).expect("恢复前视图");
    assert!(engine.restore_hex_checked("bad!").is_err());
    assert!(engine.restore_hex_checked("0").is_err());
    engine
        .restore_hex_checked(&saved)
        .expect("拒绝后合法归档仍可恢复");
    assert_eq!(
        engine.view_json(0).expect("恢复后视图"),
        before_view,
        "合法恢复应回到同一座位投影"
    );
}

#[test]
fn engine_from_archive_reports_errors_instead_of_panicking() {
    for bad in ["", "0", "中", "zz", "NGSA"] {
        assert!(
            engine_from_archive(bad, "[]").is_err(),
            "{bad:?} 必须返回错误"
        );
    }
}
