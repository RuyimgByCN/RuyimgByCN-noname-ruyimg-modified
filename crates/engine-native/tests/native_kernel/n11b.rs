use super::*;

/// P1/N11-b（ADR 0768）：存档头部/载荷版本一致校验与长度有界解析——
/// 头部规则版本被改写 → 解码错误；payload_len 溢出 → 错误而非 panic。
#[test]
fn n11b_archive_header_and_length_validation() {
    use engine_native::archive::{decode_archive, encode_archive};
    let e = Engine::new(443, 4);
    let snap = e.save();
    let orig = encode_archive(&snap);

    // 头部规则版本 24 → 999：必须返回错误（旧实现静默接受）
    let mut bad = orig.clone();
    bad[6..14].copy_from_slice(&999u64.to_le_bytes());
    assert!(decode_archive(&bad).is_err(), "头部规则版本改写必须被拒绝");

    // payload_len = u64::MAX：必须返回错误而非溢出 panic
    let mut bad2 = orig.clone();
    bad2[14..22].copy_from_slice(&u64::MAX.to_le_bytes());
    let result = std::panic::catch_unwind(|| decode_archive(&bad2));
    assert!(
        result.map(|r| r.is_err()).unwrap_or(false),
        "恶意长度必须返回错误（不得 panic）"
    );

    // 合法存档往返不受影响
    let back = decode_archive(&orig).expect("合法存档应正常解码");
    assert_eq!(back.ruleset_version, snap.ruleset_version);
}
