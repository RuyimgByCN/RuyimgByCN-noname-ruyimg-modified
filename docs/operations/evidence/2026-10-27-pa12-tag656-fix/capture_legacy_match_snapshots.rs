use content_schema::ContentPackage;
use engine_core::{effect::bootstrap_match, snapshot::Snapshot, state::GameState, transaction::Transaction};
use std::path::Path;

#[test]
fn capture_real_pre_fix_match_snapshots() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let destination = std::env::var("PA12_SNAPSHOT_DIR").unwrap();
    for (name, file) in [("onlyOL-0.1.4", "content/original/ported/onlyOL/onlyOL.npack.json"), ("onlyOL-0.1.2", "docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json"), ("sp-0.1.0", "content/original/ported/sp/sp.npack.json")] {
        let pkg = ContentPackage::from_json_bytes(&std::fs::read(fixtures.join(file)).unwrap()).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 4, 656_772, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&pkg).unwrap();
        let state = tx.commit().state;
        let snapshot = Snapshot::capture(&state);
        snapshot.restore().unwrap();
        std::fs::write(Path::new(&destination).join(format!("{name}-legacy-match-snapshot.json")), snapshot.to_bytes().unwrap()).unwrap();
        eprintln!("{name}: abi={}, modules={}, players={}, match snapshot verified", pkg.manifest.rule_vm_abi_version, pkg.modules.len(), state.players.alive_by_seat().len());
    }
}
