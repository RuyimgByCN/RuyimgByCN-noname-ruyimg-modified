//! 诊断探针：确认未修缺陷仍存在；通过表示复现成功，不表示解码器验收通过。
//! 在 noname-next 根目录编译和运行，命令见本目录 README。
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
use engine_native::Engine;

fn seal(bytes: &mut [u8]) {
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let hash = blake3::hash(&bytes[22..22 + len]);
    bytes[22 + len..].copy_from_slice(hash.as_bytes());
}

#[test]
fn valid_archive_control() {
    let snapshot = Engine::new(7, 4).save();
    assert_eq!(decode_archive(&encode_archive(&snapshot)).unwrap(), snapshot);
}

#[test]
fn unknown_card_kind_still_panics() {
    let mut snapshot = Engine::new(7, 4).save();
    snapshot.core.declared_kinds.insert(0, CardKind::Sha);
    let mut other = snapshot.clone();
    other.core.declared_kinds.insert(0, CardKind::Tao);
    let mut bytes = encode_archive(&snapshot);
    let other_bytes = encode_archive(&other);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let offsets: Vec<_> = (22..22 + len)
        .filter(|&i| bytes[i] != other_bytes[i])
        .collect();
    assert_eq!(offsets.len(), 1);
    bytes[offsets[0]] = 255;
    seal(&mut bytes);
    let result = std::panic::catch_unwind(|| decode_archive(&bytes));
    println!("unknown card kind panicked: {}", result.is_err());
    assert!(result.is_err(), "缺陷已变化，请更新进度与回归测试");
}

#[test]
fn extra_payload_bytes_still_accepted() {
    let snapshot = Engine::new(7, 4).save();
    let mut bytes = encode_archive(&snapshot);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    bytes.splice(22 + len..22 + len, [0xde, 0xad, 0xbe, 0xef]);
    bytes[14..22].copy_from_slice(&((len + 4) as u64).to_le_bytes());
    seal(&mut bytes);
    let decoded = decode_archive(&bytes);
    println!("extra payload bytes accepted: {}", decoded.is_ok());
    assert_eq!(decoded.unwrap(), snapshot, "缺陷已变化，请更新进度与回归测试");
}
