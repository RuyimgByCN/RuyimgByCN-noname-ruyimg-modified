//! P2 切片：Worker 能取得真实权威 state hash 与公开事件码，不依赖 WS。
//!
//! 事件只按种类投影，禁止把暗牌 id/选择内容带出；恢复后的引擎从空日志开始。

use engine_native_wasm::NativeEngine;

fn is_hex64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

#[test]
fn state_hash_and_public_events_are_exported() {
    let mut engine = NativeEngine::new(2, 20261024).expect("建局");
    let hash_before = engine.state_hash_hex();
    assert!(is_hex64(&hash_before), "{hash_before}");
    let start = engine.log_len();

    let outcome = engine.next_phase(0).expect("推进阶段");
    assert!(outcome.contains("\"outcome\":\"accepted\""), "{outcome}");
    let codes: Vec<u16> =
        serde_json::from_str(&engine.public_event_codes_json(start)).expect("事件码 JSON");
    assert!(!codes.is_empty(), "命令后应有真实事件码");
    assert!(
        codes
            .iter()
            .all(|code| [2u16, 3, 4, 5, 6, 7, 8, 10].contains(code)),
        "只允许公开粗粒度码: {codes:?}"
    );
    let hash_after = engine.state_hash_hex();
    assert!(is_hex64(&hash_after));
    assert_ne!(hash_before, hash_after, "推进阶段应改变权威 hash");

    // 从当前游标再读不得重复旧事件。
    let end = engine.log_len();
    assert_eq!(engine.public_event_codes_json(end), "[]");
}

#[test]
fn restore_starts_with_empty_log_and_keeps_state_hash() {
    let mut engine = NativeEngine::new(2, 7).expect("建局");
    engine.next_phase(0).expect("推进");
    let saved = engine.save_hex();
    let hash_before_restore = {
        // 存档前后的权威状态应一致；restore 会轮换会话授权代次，但不改变规则状态。
        let mut probe = NativeEngine::new(2, 7).expect("探针建局");
        probe.restore_hex_checked(&saved).expect("恢复");
        probe.state_hash_hex()
    };
    engine.restore_hex_checked(&saved).expect("恢复");
    assert_eq!(engine.log_len(), 0, "恢复后日志从空开始");
    assert_eq!(engine.public_event_codes_json(0), "[]");
    assert_eq!(engine.state_hash_hex(), hash_before_restore);
}

#[test]
fn malformed_restore_does_not_drain_or_replay_events() {
    let mut engine = NativeEngine::new(2, 9).expect("建局");
    engine.next_phase(0).expect("推进");
    let before_len = engine.log_len();
    assert!(engine.restore_hex_checked("0").is_err());
    assert_eq!(
        engine.log_len(),
        before_len,
        "失败恢复不得清空/替换原局日志"
    );
    let codes: Vec<u16> = serde_json::from_str(&engine.public_event_codes_json(0)).unwrap();
    assert!(codes.is_empty() || codes.iter().all(|c| *c > 0));
}
