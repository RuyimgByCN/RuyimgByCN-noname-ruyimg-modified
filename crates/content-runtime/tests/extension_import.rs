use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageCharacterEntry, PackageDependency, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use std::collections::BTreeMap;

fn package(id: &str) -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("community", id, "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::new(),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "test".into(),
    };
    package.manifest.logic_hash = package.compute_logic_hash();
    package
}

fn dependency(id: &str) -> PackageDependency {
    PackageDependency {
        namespace: "community".into(),
        package_id: id.into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    }
}

fn write_package(root: &std::path::Path, package: &ContentPackage) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(
        root.join("package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

#[test]
fn drop_in_directories_import_dependencies_metadata_and_assets() {
    let dir = tempfile::tempdir().unwrap();
    let extensions = dir.path().join("extensions");
    let dependent_dir = extensions.join("a-hero");
    let base_dir = extensions.join("z-base");
    std::fs::create_dir_all(&dependent_dir).unwrap();
    std::fs::create_dir_all(&base_dir).unwrap();

    let base = package("base");
    std::fs::write(
        base_dir.join("package.npack.json"),
        base.to_json_bytes().unwrap(),
    )
    .unwrap();

    let portrait = b"portrait bytes";
    std::fs::create_dir_all(dependent_dir.join("assets")).unwrap();
    std::fs::write(dependent_dir.join("assets/hero.png"), portrait).unwrap();
    let mut hero = package("hero");
    hero.manifest.dependencies.push(dependency("base"));
    hero.character_index.push(PackageCharacterEntry {
        voice_assets: Default::default(),
        id: 9001,
        name_key: "hero".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: "wei".into(),
        is_lord: false,
        clans: vec![],
        tags: vec!["extension".into()],
        sort_key: "community.hero".into(),
        portrait_asset: Some("hero:portrait".into()),
        skin_assets: vec![],
    });
    hero.translations.insert(
        "zh-CN".into(),
        BTreeMap::from([("hero".into(), "测试武将".into())]),
    );
    let mut assets = AssetManifest::new("hero");
    assets.add_entry(AssetEntry {
        asset_id: "hero:portrait".into(),
        source_hash: *blake3::hash(portrait).as_bytes(),
        kind: AssetKind::Image,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    hero.asset_manifest = Some(assets);
    hero.asset_paths
        .insert("hero:portrait".into(), "assets/hero.png".into());
    hero.manifest.logic_hash = hero.compute_logic_hash();
    hero.manifest.asset_hash = hero.compute_asset_hash();
    std::fs::write(
        dependent_dir.join("package.npack.json"),
        hero.to_json_bytes().unwrap(),
    )
    .unwrap();

    let store_root = dir.path().join("store");
    let store = PackageStore::open(&store_root).unwrap();
    let report = import_extensions(
        &store,
        &extensions,
        &InstallOptions {
            activate: true,
            allow_unsigned: true,
            verify_key: None,
            host: content_runtime::tooling_host_versions(),
        },
    )
    .unwrap();
    assert_eq!(report.installed.len(), 2);
    assert_eq!(report.content_lock.packages.len(), 2);
    std::fs::remove_dir_all(&extensions).unwrap();
    let info = store
        .resolve_active_asset_info("community", "hero", "hero:portrait")
        .unwrap();
    assert!(info.path.starts_with(store_root.canonicalize().unwrap()));
    assert_eq!(std::fs::read(&info.path).unwrap(), portrait);
    assert_eq!(info.source_hash, *blake3::hash(portrait).as_bytes());
    assert_eq!(info.kind, AssetKind::Image);
    assert_eq!(
        store.load_active_asset_index().unwrap()
            [&("community".into(), "hero".into(), "hero:portrait".into())],
        info
    );
    assert_eq!(
        store
            .resolve_active_asset("community", "hero", "hero:portrait")
            .unwrap(),
        info.path
    );
}

#[test]
fn exact_dependency_cycle_imports_as_an_atomic_group() {
    let dir = tempfile::tempdir().unwrap();
    let extensions = dir.path().join("extensions");
    std::fs::create_dir_all(&extensions).unwrap();

    let definitions = [
        ("base", vec![]),
        ("alpha", vec!["base", "beta"]),
        ("beta", vec!["base", "alpha"]),
        ("hero", vec!["alpha"]),
    ];
    for (id, dependencies) in definitions {
        let extension = extensions.join(id);
        std::fs::create_dir_all(&extension).unwrap();
        let mut package = package(id);
        package.manifest.dependencies = dependencies.into_iter().map(dependency).collect();
        package.manifest.logic_hash = package.compute_logic_hash();
        std::fs::write(
            extension.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
    }

    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(
        &store,
        &extensions,
        &InstallOptions {
            activate: true,
            allow_unsigned: true,
            verify_key: None,
            host: content_runtime::tooling_host_versions(),
        },
    )
    .unwrap();
    assert_eq!(report.installed.len(), 4);

    let (_, ordered) = store.load_active_packages().unwrap();
    assert_eq!(
        ordered
            .iter()
            .map(|package| package.manifest.package_id.as_str())
            .collect::<Vec<_>>(),
        ["base", "alpha", "beta", "hero"]
    );
}

#[test]
fn bad_asset_leaves_staging_and_active_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let extensions = dir.path().join("extensions");
    write_package(&extensions.join("base"), &package("base"));
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    import_extensions(&store, &extensions, &options()).unwrap();
    let active = store.read_active().unwrap();

    let bad_root = extensions.join("bad");
    std::fs::create_dir_all(bad_root.join("assets")).unwrap();
    std::fs::write(bad_root.join("assets/bad.png"), b"wrong").unwrap();
    let mut bad = package("bad");
    let mut manifest = AssetManifest::new("bad");
    manifest.add_entry(AssetEntry {
        asset_id: "bad".into(),
        source_hash: *blake3::hash(b"expected").as_bytes(),
        kind: AssetKind::Image,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    bad.asset_manifest = Some(manifest);
    bad.asset_paths
        .insert("bad".into(), "assets/bad.png".into());
    bad.manifest.logic_hash = bad.compute_logic_hash();
    bad.manifest.asset_hash = bad.compute_asset_hash();
    write_package(&bad_root, &bad);

    let error = import_extensions(&store, &extensions, &options()).unwrap_err();
    assert!(error.to_string().contains("asset hash mismatch"));
    assert_eq!(store.read_active().unwrap(), active);
    assert!(std::fs::read_dir(store.root().join("staging"))
        .unwrap()
        .next()
        .is_none());
    assert!(!store.root().join("packages/community__bad/1.0.0").exists());
}

#[test]
fn reconcile_removes_deleted_disabled_and_broken_dependents() {
    let dir = tempfile::tempdir().unwrap();
    let extensions = dir.path().join("extensions");
    let base_root = extensions.join("base");
    let hero_root = extensions.join("hero");
    write_package(&base_root, &package("base"));
    let mut hero = package("hero");
    hero.manifest.dependencies.push(dependency("base"));
    hero.manifest.logic_hash = hero.compute_logic_hash();
    write_package(&hero_root, &hero);
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    import_extensions(&store, &extensions, &options()).unwrap();
    assert_eq!(store.read_active().unwrap().packages.len(), 2);

    std::fs::remove_dir_all(&hero_root).unwrap();
    import_extensions(&store, &extensions, &options()).unwrap();
    let active = store.read_active().unwrap();
    assert_eq!(active.packages.len(), 1);
    assert!(active.packages.contains_key("community/base"));

    write_package(&hero_root, &hero);
    import_extensions(&store, &extensions, &options()).unwrap();
    assert_eq!(store.read_active().unwrap().packages.len(), 2);

    std::fs::write(
        base_root.join("manifest.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "enabled": false,
            "package": "package.npack.json"
        }))
        .unwrap(),
    )
    .unwrap();
    let report = import_extensions(&store, &extensions, &options()).unwrap();
    assert_eq!(report.skipped, vec!["base"]);
    assert!(store.read_active().unwrap().packages.is_empty());

    std::fs::remove_file(base_root.join("manifest.json")).unwrap();
    import_extensions(&store, &extensions, &options()).unwrap();
    assert_eq!(store.read_active().unwrap().packages.len(), 2);
    std::fs::remove_dir_all(&extensions).unwrap();
    import_extensions(&store, &extensions, &options()).unwrap();
    assert!(store.read_active().unwrap().packages.is_empty());
}

#[test]
fn catalog_validation_failure_does_not_change_active() {
    use content_schema::PackageSkinAliasEntry;

    let dir = tempfile::tempdir().unwrap();
    let extensions = dir.path().join("extensions");
    write_package(&extensions.join("base"), &package("base"));
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    import_extensions(&store, &extensions, &options()).unwrap();
    let active = store.read_active().unwrap();

    let mut broken = package("broken");
    broken.skin_index.aliases.push(PackageSkinAliasEntry {
        character_id: 2,
        source_character_id: 1,
        voice_mappings: BTreeMap::new(),
    });
    broken.manifest.logic_hash = broken.compute_logic_hash();
    broken.manifest.asset_hash = broken.compute_asset_hash();
    write_package(&extensions.join("broken"), &broken);

    let error = import_extensions(&store, &extensions, &options()).unwrap_err();
    assert!(error.to_string().contains("skin alias"), "{error}");
    assert_eq!(store.read_active().unwrap(), active);
}
