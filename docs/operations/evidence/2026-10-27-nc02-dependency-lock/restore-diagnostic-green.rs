use engine_native_wasm::test_support::{engine_from_archive, engine_from_packs, hex_encode};

#[test]
fn restore_rotates_auth_generation() {
    let original = engine_from_packs(4, 17017, "[]").unwrap();
    let before = original.save();
    let before_hex = hex_encode(&engine_native::archive::encode_archive(&before));
    let restored = engine_from_archive(&before_hex, "[]").unwrap();
    let after = restored.save();
    let after_hex = hex_encode(&engine_native::archive::encode_archive(&after));
    println!("before_generation={} after_generation={} state_hash_equal={} bytes_equal={}",
        before.auth_generation, after.auth_generation, original.state_hash() == restored.state_hash(), before_hex == after_hex);
    assert_eq!(after.auth_generation, before.auth_generation + 1);
    assert_eq!(original.state_hash(), restored.state_hash());
    assert_ne!(before_hex, after_hex);
    let mut normalized = after;
    normalized.auth_generation = before.auth_generation;
    assert_eq!(before_hex, hex_encode(&engine_native::archive::encode_archive(&normalized)));
    println!("only auth_generation changed");
}
