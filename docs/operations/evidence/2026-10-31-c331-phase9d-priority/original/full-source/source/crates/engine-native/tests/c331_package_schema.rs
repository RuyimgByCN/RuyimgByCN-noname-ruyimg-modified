use content_schema::{ContentPackage, TriggerTiming};

fn package() -> ContentPackage {
    content_compiler::compile_source_named("c331-schema", "skill c331_schema(4200000001) locked { trigger phase_end always { effect draw_cards(self, 1); } }").unwrap()
}
fn rehash(pkg: &mut ContentPackage) {
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
}
#[test]
fn nondefault_schema3_is_rejected_even_after_rehash() {
    let mut json = serde_json::to_value(package()).unwrap();
    json["trigger_index"][0]["priority"] = 17.into();
    let mut pkg: ContentPackage = serde_json::from_value(json).unwrap();
    rehash(&mut pkg);
    assert!(pkg.validate().is_err(), "schema3 cannot express nondefault ordering");
}
#[test]
fn public_bridge_validates_whole_package() {
    let mut pkg = package();
    pkg.manifest.logic_hash[0] ^= 1;
    assert!(content_native_loader::bridge_vm_skill_decls(&pkg).is_err());
}
#[test]
fn real_old_default_bytes_and_logic_hash_remain_identical() {
    let old = include_bytes!("fixtures/c331-old-default-package.json");
    let pkg = package();
    assert_eq!(pkg.to_json_bytes().unwrap(), old);
    pkg.validate().unwrap();
    content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    let mut explicit: serde_json::Value = serde_json::from_slice(old).unwrap();
    explicit["trigger_index"][0]["timing"] = "normal".into();
    explicit["trigger_index"][0]["priority"] = 0.into();
    explicit["trigger_index"][0]["order_selectable"] = true.into();
    let explicit: ContentPackage = serde_json::from_value(explicit).unwrap();
    assert_eq!(pkg.compute_logic_hash(), explicit.compute_logic_hash());
}
#[test]
fn schema_matrix_and_native_metadata_use_real_valid_dsl_module() {
    let default = package();
    for schema in [0, 1, 2, 4, 5, u32::MAX] {
        let mut pkg = default.clone(); pkg.manifest.content_schema_version = schema; rehash(&mut pkg);
        assert!(content_native_loader::bridge_vm_skill_decls(&pkg).is_err(), "default schema {schema}");
    }
    let mut pkg = default;
    pkg.manifest.version = "0.2.0".into();
    pkg.trigger_index[0].priority = i32::MIN;
    pkg.trigger_index[0].timing = TriggerTiming::Last;
    pkg.trigger_index[0].order_selectable = false;
    pkg.manifest.content_schema_version = pkg.required_content_schema(); rehash(&mut pkg);
    assert_eq!(pkg.manifest.content_schema_version, 4);
    let decls = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap().decls;
    assert_eq!(decls[0].name, "c331_schema");
    assert_eq!(decls[0].trigger.priority, i32::MIN);
    assert_eq!(decls[0].trigger.timing, engine_native::event::Timing::Last);
    assert!(!decls[0].trigger.order_selectable);
    if let Ok(root) = std::env::var("C331_PACKAGE_EVIDENCE") { std::fs::write(std::path::Path::new(&root).join("schema4-package.json"), pkg.to_json_bytes().unwrap()).unwrap(); }
    let mut host = content_runtime::tooling_host_versions(); host.content_schema = 0;
    assert!(content_runtime::check_engine_compat(&pkg.manifest, &host).is_err());
    for schema in [0, 3, 5] { let mut invalid = pkg.clone(); invalid.manifest.content_schema_version = schema; rehash(&mut invalid); assert!(invalid.validate().is_err()); }
}
#[test]
fn order_segment_binds_each_original_position_and_handler_even_with_duplicate_trigger_ids() {
    let mut p = package();
    let mut second = p.trigger_index[0].clone(); second.filter = "owner_alive".into();
    p.trigger_index.push(second);
    p.trigger_index[0].priority = 2; p.trigger_index[1].priority = 3;
    p.manifest.content_schema_version = 4;
    let base = p.compute_logic_hash();
    let mut swapped = p.clone(); swapped.trigger_index[0].priority = 3; swapped.trigger_index[1].priority = 2;
    assert_ne!(base, swapped.compute_logic_hash());
    for mutation in 0..6 {
        let mut p = p.clone();
        match mutation { 0 => p.trigger_index[0].timing = TriggerTiming::First, 1 => p.trigger_index[0].order_selectable = false,
            2 => p.trigger_index[0].filter = "owner_is_target".into(), 3 => p.trigger_index[0].entry_pc += 1,
            4 => p.trigger_index[0].module.push('x'), 5 => p.trigger_index.reverse(), _ => unreachable!() }
        assert_ne!(base, p.compute_logic_hash(), "mutation {mutation}");
    }
    rehash(&mut p);
    assert!(content_native_loader::bridge_vm_skill_decls(&p).is_err(), "mixed ordering tuples cannot be folded into the first handler");
}
#[test]
fn malformed_order_json_is_rejected() {
    for (field, value) in [("timing", serde_json::json!("earliest")), ("priority", serde_json::json!(2147483648u64)), ("priority", serde_json::json!(1.5)), ("order_selectable", serde_json::json!(1))] {
        let mut p = serde_json::to_value(package()).unwrap(); p["trigger_index"][0][field] = value;
        assert!(serde_json::from_value::<ContentPackage>(p).is_err());
    }
}
#[test]
fn formal_install_keeps_old_lock_bytes_and_rejects_tampered_same_version_before_idempotency() {
    let old = package(); let old_bytes = old.to_json_bytes().unwrap();
    let dir = tempfile::tempdir().unwrap(); let store = content_runtime::PackageStore::open(dir.path()).unwrap();
    let options = content_runtime::InstallOptions { allow_unsigned: true, ..Default::default() };
    store.install(&old_bytes, None, &options).unwrap(); let old_lock = store.current_lock().unwrap();
    let mut next = old.clone(); next.manifest.version = "0.2.0".into(); next.trigger_index[0].priority = 17; next.manifest.content_schema_version = 4; rehash(&mut next);
    store.install(&next.to_json_bytes().unwrap(), None, &options).unwrap();
    assert_eq!(store.load_locked_packages(&old_lock).unwrap(), vec![old]);
    let before = store.current_lock().unwrap();
    next.trigger_index[0].priority += 1;
    assert!(store.install(&next.to_json_bytes().unwrap(), None, &options).is_err());
    assert_eq!(before, store.current_lock().unwrap());
}

#[test]
fn immutable_schema3_character_module_alias_resolves_without_rewriting_package() {
    let p = package();
    let original = p.to_json_bytes().unwrap();
    let chars = serde_json::json!({"generals":[{"name":"old alias","hp":4,"faction":"qun","gender":"male","skills":[{"name":p.trigger_index[0].module}]}]}).to_string();
    let bridged = content_native_loader::bridge_generals_with_vm_package(&chars,&p).unwrap();
    assert_eq!(bridged.generals[0].skills, &["c331_schema"]);
    assert_eq!(p.to_json_bytes().unwrap(),original);
    let rejected = serde_json::json!({"generals":[{"name":"bad","hp":4,"faction":"qun","gender":"male","skills":[{"name":"missing"}]}]}).to_string();
    assert!(content_native_loader::bridge_generals_with_vm_package(&rejected,&p).is_err());
}
