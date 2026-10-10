//! Real-extensions gate: install every deployed extension package into a fresh
//! store, pin the active lock, load it into an engine, and start a match.
//!
//! Guards the contract "all 31 deployed extension packages load together and a
//! canonical match boots" — the same path game servers use (ADR 0582 roster
//! closure and later batches must keep this green).

use content_runtime::{InstallOptions, PackageStore};
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
};
use engine_core::Engine;

const EXTENSIONS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../extensions");

#[test]
fn all_deployed_extensions_load_into_engine_and_start_match() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let opts = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };

    let mut packages: Vec<(String, [u8; 32])> = Vec::new();
    for entry in std::fs::read_dir(EXTENSIONS_ROOT).unwrap() {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            continue;
        }
        let pkg_path = entry.path().join("package.npack.json");
        if !pkg_path.exists() {
            continue;
        }
        let bytes = std::fs::read(&pkg_path).unwrap();
        let package: content_schema::ContentPackage =
            serde_json::from_slice(&bytes).expect("deployed npack parses");
        packages.push((
            format!(
                "{}/{}",
                package.manifest.namespace, package.manifest.package_id
            ),
            package.manifest.logic_hash,
        ));
        store
            .install_with_assets(&bytes, None, &opts, &entry.path())
            .expect("extension installs");
    }
    assert!(packages.len() >= 30, "expected >=30 extension packages");

    let deps: Vec<content_schema::PackageDependency> = packages
        .iter()
        .map(|(id, hash)| content_schema::PackageDependency {
            namespace: id.split('/').next().unwrap().into(),
            package_id: id.split('/').nth(1).unwrap().into(),
            version: "0.1.0".into(),
            logic_hash: *hash,
        })
        .collect();
    store.activate_packages(&deps).unwrap();

    let lock = store.current_lock().unwrap();
    let mut engine = Engine::from_content_lock(&store, &lock).expect("full lock loads");

    let r = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20260804,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(r, CommandResult::Accepted { .. }),
        "start rejected: {r:?}"
    );
    assert_eq!(engine.state().players.live_count(), 2);
}
