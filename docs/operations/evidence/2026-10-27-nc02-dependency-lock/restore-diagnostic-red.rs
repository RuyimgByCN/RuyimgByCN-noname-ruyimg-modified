use engine_native_wasm::test_support::{engine_from_archive, engine_from_packs, hex_encode};

#[test]
fn restored_archive_has_same_bytes() {
    let original = engine_from_packs(4, 17017, "[]").unwrap();
    let before = original.save();
    let before_hex = hex_encode(&engine_native::archive::encode_archive(&before));
    let restored = engine_from_archive(&before_hex, "[]").unwrap();
    let after = restored.save();
    let after_hex = hex_encode(&engine_native::archive::encode_archive(&after));
    println!("before_generation={} after_generation={} state_hash_equal={} bytes_equal={}",
        before.auth_generation, after.auth_generation, original.state_hash() == restored.state_hash(), before_hex == after_hex);
    assert_eq!(before_hex, after_hex, "archive equality is not a valid restore invariant");
}
