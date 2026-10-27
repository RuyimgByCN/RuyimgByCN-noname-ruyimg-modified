use engine_native::{archive::{decode_archive, encode_archive}, Engine};
#[test]
fn capture_native_v322_archive() {
    let engine = Engine::new(65101, 4);
    let snapshot = engine.save();
    assert_eq!(snapshot.ruleset_version, 322);
    let bytes = encode_archive(&snapshot);
    let decoded = decode_archive(&bytes).unwrap();
    Engine::restore(&decoded).unwrap();
    let path = std::path::Path::new(&std::env::var("PA12_SNAPSHOT_DIR").unwrap()).join("native-v322.ngsa");
    std::fs::write(path, &bytes).unwrap();
    eprintln!("native v322: format=105, archive bytes={}, restored", bytes.len());
}
