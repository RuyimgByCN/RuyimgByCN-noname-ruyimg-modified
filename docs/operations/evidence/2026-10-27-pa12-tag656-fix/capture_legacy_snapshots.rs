use content_schema::ContentPackage;
use engine_core::Engine;
use std::path::Path;

#[test]
fn capture_real_pre_fix_content_snapshots() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported");
    let destination = std::env::var("PA12_SNAPSHOT_DIR").unwrap();
    for (name, file) in [("onlyOL-0.1.4", "onlyOL/onlyOL.npack.json"), ("sp-0.1.0", "sp/sp.npack.json")] {
        let bytes = std::fs::read(fixtures.join(file)).unwrap();
        let pkg = ContentPackage::from_json_bytes(&bytes).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&pkg).unwrap();
        let snapshot = engine.snapshot();
        snapshot.restore().unwrap();
        let bytes = snapshot.to_bytes().unwrap();
        std::fs::write(Path::new(&destination).join(format!("{name}-legacy-snapshot.json")), bytes).unwrap();
        eprintln!("{name}: abi={}, modules={}, snapshot verified", pkg.manifest.rule_vm_abi_version, pkg.modules.len());
    }
}
