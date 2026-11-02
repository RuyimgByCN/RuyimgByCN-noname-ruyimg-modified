use content_runtime::{InstallOptions, PackageStore};
use content_schema::{PackageCharacterEntry, PackageDependency};
use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;

fn package(namespace: &str, id: &str, def_id: u32, draw: u32) -> content_schema::ContentPackage {
    let source = format!(
        "skill shared({def_id}) {{ trigger phase_end always {{ effect draw_cards(self, {draw}); }} }}"
    );
    let mut package = content_compiler::compile_source_named(id, &source).unwrap();
    package.manifest.namespace = namespace.into();
    package.manifest.version = "1.0.0".into();
    package.manifest.logic_hash = package.compute_logic_hash();
    package
}

#[test]
fn canonical_lock_loads_dependencies_and_namespaces_modules() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let base = package("community", "base", 8001, 1);
    let mut hero = package("community", "hero", 8002, 2);
    hero.manifest.dependencies.push(PackageDependency {
        namespace: "community".into(),
        package_id: "base".into(),
        version: "1.0.0".into(),
        logic_hash: base.manifest.logic_hash,
    });
    hero.manifest.logic_hash = hero.compute_logic_hash();
    let opts = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    store
        .install(&hero.to_json_bytes().unwrap(), None, &opts)
        .unwrap();
    store
        .install(&base.to_json_bytes().unwrap(), None, &opts)
        .unwrap();
    store
        .activate_packages(&[
            PackageDependency {
                namespace: "community".into(),
                package_id: "hero".into(),
                version: "1.0.0".into(),
                logic_hash: [0u8; 32],
            },
            PackageDependency {
                namespace: "community".into(),
                package_id: "base".into(),
                version: "1.0.0".into(),
                logic_hash: [0u8; 32],
            },
        ])
        .unwrap();

    let lock = store.current_lock().unwrap();
    let mut engine = Engine::from_content_lock(&store, &lock).unwrap();
    assert_eq!(engine.content().package_count(), 2);
    assert!(engine
        .content()
        .module("community/base::shared_phase_end")
        .is_some());
    assert!(engine
        .content()
        .module("community/hero::shared_phase_end")
        .is_some());
    assert!(engine.content().module("shared_phase_end").is_none());

    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 7,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        engine.manifest().unwrap().content_lock_hash.0,
        decode_hash(&lock.lock_hash)
    );
}

#[test]
fn rejected_content_lock_leaves_engine_fresh_for_retry() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let valid = package("community", "a_valid", 8101, 1);
    let mut invalid = package("community", "z_invalid", 8102, 1);
    invalid.manifest.dependencies.push(PackageDependency {
        namespace: "community".into(),
        package_id: "a_valid".into(),
        version: "1.0.0".into(),
        logic_hash: valid.manifest.logic_hash,
    });
    invalid.character_index.push(PackageCharacterEntry {
        voice_assets: Default::default(),
        id: 8103,
        name_key: "missing-skill".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![999_999],
        sex: "male".into(),
        group: "wei".into(),
        is_lord: false,
    clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    });
    invalid.manifest.logic_hash = invalid.compute_logic_hash();
    let opts = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    store
        .install(&valid.to_json_bytes().unwrap(), None, &opts)
        .unwrap();
    store
        .install(&invalid.to_json_bytes().unwrap(), None, &opts)
        .unwrap();
    store
        .activate_packages(&[
            PackageDependency {
                namespace: "community".into(),
                package_id: "a_valid".into(),
                version: "1.0.0".into(),
                logic_hash: [0; 32],
            },
            PackageDependency {
                namespace: "community".into(),
                package_id: "z_invalid".into(),
                version: "1.0.0".into(),
                logic_hash: [0; 32],
            },
        ])
        .unwrap();

    let mut engine = Engine::new();
    assert!(engine
        .load_content_lock(&store, &store.current_lock().unwrap())
        .is_err());
    assert_eq!(engine.content().package_count(), 0);

    store
        .revoke_slot("community", "z_invalid", "test", true)
        .unwrap();
    engine
        .load_content_lock(&store, &store.current_lock().unwrap())
        .unwrap();
    assert_eq!(engine.content().package_count(), 1);
}

fn decode_hash(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    out
}
