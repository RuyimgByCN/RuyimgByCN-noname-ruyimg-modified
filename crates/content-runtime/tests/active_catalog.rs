use content_runtime::{ActiveCatalog, InstallOptions, PackageStore};
use content_schema::{
    ContentPackage, PackageCharacterEntry, PackageDependency, PackageManifest,
    PackageSkinAliasEntry, PackageSkinEntry, PackageSkinLayout, PackageSkinPresentation,
};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use std::collections::BTreeMap;

fn package(package_id: &str) -> ContentPackage {
    ContentPackage {
        manifest: PackageManifest::new("community", package_id, "1.0.0"),
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
    }
}

fn add_asset(package: &mut ContentPackage, asset_id: &str, kind: AssetKind) {
    let manifest = package
        .asset_manifest
        .get_or_insert_with(|| AssetManifest::new(&package.manifest.package_id));
    manifest.add_entry(AssetEntry {
        asset_id: asset_id.into(),
        source_hash: *blake3::hash(asset_id.as_bytes()).as_bytes(),
        kind,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package
        .asset_paths
        .insert(asset_id.into(), format!("assets/{asset_id}"));
}

fn write_assets(root: &std::path::Path, package: &ContentPackage) {
    for entry in &package.asset_manifest.as_ref().unwrap().entries {
        let path = root.join(&package.asset_paths[&entry.asset_id]);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, entry.asset_id.as_bytes()).unwrap();
    }
}

fn seal(package: &mut ContentPackage) {
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
}

fn base_package() -> ContentPackage {
    let mut base = package("heroes");
    add_asset(&mut base, "portrait", AssetKind::Image);
    add_asset(&mut base, "legacy", AssetKind::Image);
    base.character_index.push(PackageCharacterEntry {
        voice_assets: Default::default(),
        id: 1,
        name_key: "hero.one".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: "shu".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: Some("portrait".into()),
        skin_assets: vec!["legacy".into()],
    });
    base.character_index.push(PackageCharacterEntry {
        voice_assets: Default::default(),
        id: 2,
        name_key: "hero.two".into(),
        max_hp: 3,
        initial_hp: 0,
        skills: vec![],
        sex: "female".into(),
        group: "wei".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    });
    base.translations
        .entry("zh-CN".into())
        .or_default()
        .insert("hero.one".into(), "基础名".into());
    seal(&mut base);
    base
}

fn skin_package(base: &ContentPackage, package_id: &str) -> ContentPackage {
    let mut skins = package(package_id);
    skins.manifest.dependencies.push(PackageDependency {
        namespace: base.manifest.namespace.clone(),
        package_id: base.manifest.package_id.clone(),
        version: base.manifest.version.clone(),
        logic_hash: base.manifest.logic_hash,
    });
    add_asset(&mut skins, "still", AssetKind::Image);
    add_asset(&mut skins, "substitute", AssetKind::Image);
    add_asset(&mut skins, "spine_bundle", AssetKind::Data);
    add_asset(&mut skins, "spine_atlas", AssetKind::TextureAtlas);
    add_asset(&mut skins, "death_voice", AssetKind::Audio);

    skins.skin_index.skins.push(PackageSkinEntry {
        id: 100,
        character_id: 1,
        name_key: "skin.hero.still".into(),
        presentation: PackageSkinPresentation::Static {
            asset: "still".into(),
        },
        voice_assets: BTreeMap::from([("die".into(), "death_voice".into())]),
    });
    skins.skin_index.skins.push(PackageSkinEntry {
        id: 101,
        character_id: 1,
        name_key: "skin.hero.spine".into(),
        presentation: PackageSkinPresentation::Spine {
            bundle: "spine_bundle".into(),
            actions: BTreeMap::from([
                ("attack".into(), "GongJi".into()),
                ("idle".into(), "DaiJi".into()),
            ]),
            layout: PackageSkinLayout {
                x_pixels: 10,
                scale_permille: 850,
                ..Default::default()
            },
            dependencies: vec!["spine_atlas".into()],
        },
        voice_assets: BTreeMap::new(),
    });
    skins.skin_index.skins.push(PackageSkinEntry {
        id: 102,
        character_id: 1,
        name_key: "skin.hero.dead".into(),
        presentation: PackageSkinPresentation::Substitute {
            rule: "dead".into(),
            asset: "substitute".into(),
        },
        voice_assets: BTreeMap::new(),
    });
    skins.skin_index.aliases.push(PackageSkinAliasEntry {
        character_id: 2,
        source_character_id: 1,
        voice_mappings: BTreeMap::from([("new_skill".into(), "old_skill".into())]),
    });
    skins
        .translations
        .entry("zh-CN".into())
        .or_default()
        .insert("hero.one".into(), "覆盖名".into());
    seal(&mut skins);
    skins
}

#[test]
fn active_store_aggregates_cross_package_skins_and_legacy_assets() {
    let base = base_package();
    let skins = skin_package(&base, "skins");
    assert!(skins.character_index.is_empty());
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let options = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    let base_root = dir.path().join("base");
    let skins_root = dir.path().join("skins");
    write_assets(&base_root, &base);
    write_assets(&skins_root, &skins);
    store
        .install_with_assets(&base.to_json_bytes().unwrap(), None, &options, &base_root)
        .unwrap();
    store
        .install_with_assets(&skins.to_json_bytes().unwrap(), None, &options, &skins_root)
        .unwrap();
    store
        .activate_packages(&[
            PackageDependency {
                namespace: "community".into(),
                package_id: "skins".into(),
                version: "1.0.0".into(),
                logic_hash: [0u8; 32],
            },
            PackageDependency {
                namespace: "community".into(),
                package_id: "heroes".into(),
                version: "1.0.0".into(),
                logic_hash: [0u8; 32],
            },
        ])
        .unwrap();

    let (_, catalog) = store.load_active_catalog().unwrap();
    assert_eq!(catalog.translation("zh-CN", "hero.one"), Some("覆盖名"));
    assert_eq!(catalog.characters[&1].skin_ids, vec![100, 101, 102]);
    assert_eq!(catalog.characters[&2].skin_ids, vec![100, 101, 102]);
    assert_eq!(
        catalog.characters[&1].portrait_asset.as_ref().unwrap(),
        &content_runtime::ActiveAssetRef {
            namespace: "community".into(),
            package_id: "heroes".into(),
            asset_id: "portrait".into(),
        }
    );
    assert_eq!(
        catalog.characters[&1].legacy_skin_assets[0].package_id,
        "heroes"
    );
    let still = &catalog.skins[&100];
    assert_eq!(still.package_id, "skins");
    assert_eq!(still.voice_assets["die"].asset_id, "death_voice");
    match &still.presentation {
        content_runtime::ActiveSkinPresentation::Static { asset } => {
            assert_eq!(
                asset,
                &content_runtime::ActiveAssetRef {
                    namespace: "community".into(),
                    package_id: "skins".into(),
                    asset_id: "still".into(),
                }
            );
        }
        other => panic!("unexpected presentation: {other:?}"),
    }
}

#[test]
fn catalog_rejects_missing_targets_and_duplicate_active_skin_ids() {
    let base = base_package();
    let mut missing = skin_package(&base, "missing");
    missing.skin_index.skins[0].character_id = 999;
    seal(&mut missing);
    assert!(missing.validate().is_ok());
    let error =
        ActiveCatalog::from_dependency_ordered_packages(&[base.clone(), missing]).unwrap_err();
    assert!(error.to_string().contains("missing character 999"));

    let skins = skin_package(&base, "skins");
    let duplicate = skin_package(&base, "duplicate");
    let error =
        ActiveCatalog::from_dependency_ordered_packages(&[base, skins, duplicate]).unwrap_err();
    assert!(error.to_string().contains("duplicate active skin id 100"));
}

#[test]
fn package_rejects_bad_skin_resources_and_unsafe_renderer_values() {
    let base = base_package();
    let skins = skin_package(&base, "skins");

    let mut missing = skins.clone();
    missing.skin_index.skins[0].presentation = PackageSkinPresentation::Static {
        asset: "missing".into(),
    };
    seal(&mut missing);
    assert!(missing
        .validate()
        .unwrap_err()
        .contains("references unknown asset"));

    let mut wrong_kind = skins.clone();
    wrong_kind.skin_index.skins[0].presentation = PackageSkinPresentation::Static {
        asset: "death_voice".into(),
    };
    seal(&mut wrong_kind);
    assert!(wrong_kind
        .validate()
        .unwrap_err()
        .contains("expected Image"));

    let mut unsafe_action = skins.clone();
    let PackageSkinPresentation::Spine { actions, .. } =
        &mut unsafe_action.skin_index.skins[1].presentation
    else {
        unreachable!()
    };
    actions.insert("idle".into(), "alert()".into());
    seal(&mut unsafe_action);
    assert!(unsafe_action
        .validate()
        .unwrap_err()
        .contains("unsafe Spine action"));

    let mut unsafe_layout = skins;
    let PackageSkinPresentation::Spine { layout, .. } =
        &mut unsafe_layout.skin_index.skins[1].presentation
    else {
        unreachable!()
    };
    layout.scale_permille = 0;
    seal(&mut unsafe_layout);
    assert!(unsafe_layout
        .validate()
        .unwrap_err()
        .contains("unsafe Spine layout"));
}

#[test]
fn skin_index_changes_only_asset_hash_and_round_trips_at_package_root() {
    let base = base_package();
    let mut with_skin = base.clone();
    let logic_before = with_skin.compute_logic_hash();
    let asset_before = with_skin.compute_asset_hash();
    with_skin.skin_index.skins.push(PackageSkinEntry {
        id: 999,
        character_id: 1,
        name_key: "skin.hero.legacy".into(),
        presentation: PackageSkinPresentation::Static {
            asset: "legacy".into(),
        },
        voice_assets: BTreeMap::new(),
    });
    assert_eq!(with_skin.compute_logic_hash(), logic_before);
    assert_ne!(with_skin.compute_asset_hash(), asset_before);
    seal(&mut with_skin);
    with_skin.validate().unwrap();

    let json = serde_json::to_value(&with_skin).unwrap();
    assert!(json.get("skin_index").is_some());
    assert!(json["manifest"].get("skin_index").is_none());
    let round_trip: ContentPackage = serde_json::from_value(json).unwrap();
    assert_eq!(round_trip, with_skin);
}

#[test]
fn catalog_pages_are_bounded_and_keep_page_aliases_and_names() {
    let base = base_package();
    let skins = skin_package(&base, "skins");
    let catalog = ActiveCatalog::from_dependency_ordered_packages(&[base, skins]).unwrap();

    let page = catalog.page("lock", None, None);
    assert_eq!((page.offset, page.limit, page.total), (0, 64, 2));
    assert_eq!(page.characters.len(), 2);
    assert_eq!(page.skins.len(), 3);
    assert_eq!(page.aliases.len(), 1);
    assert_eq!(page.translations["zh-CN"]["hero.one"], "覆盖名");
    assert!(!page.translations["zh-CN"].contains_key("unrelated"));

    let empty = catalog.page("lock", Some(u32::MAX), Some(u32::MAX));
    assert_eq!((empty.offset, empty.limit), (2, 200));
    assert!(empty.characters.is_empty());
    assert!(empty.skins.is_empty());
    assert!(empty.aliases.is_empty());
}
