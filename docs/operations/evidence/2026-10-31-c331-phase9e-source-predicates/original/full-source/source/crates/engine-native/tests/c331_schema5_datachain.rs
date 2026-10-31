use content_schema::ContentPackage;
fn package() -> ContentPackage {
    content_compiler::compile_source_named("thirdparty-schema5", "skill schema5_draw(99108001) { trigger damage owner_is_target { effect draw_cards(self,1); } }").unwrap()
}
#[test]
fn schema5_explicit_default_is_data_and_runtime_compatible() {
    let mut value = serde_json::to_value(package()).unwrap();
    value["manifest"]["content_schema_version"] = 5.into();
    value["skill_index"][0]["charlotte"] = false.into();
    let mut parsed = ContentPackage::from_json_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(parsed.required_content_schema(), 5);
    parsed.manifest.logic_hash = parsed.compute_logic_hash();
    parsed.validate().unwrap();
    content_native_loader::bridge_vm_skill_decls(&parsed).unwrap();
}
#[test]
fn schema5_fields_present_in_old_schema_reject_even_defaults() {
    for (table, key, value) in [
        ("skill_index", "charlotte", serde_json::json!(false)),
        ("skill_index", "group_children", serde_json::json!([])),
        ("trigger_index", "force_die", serde_json::json!(false)),
    ] {
        let mut value_package = serde_json::to_value(package()).unwrap();
        value_package[table][0][key] = value;
        let result = ContentPackage::from_json_bytes(&serde_json::to_vec(&value_package).unwrap())
            .and_then(|p| p.validate());
        assert!(result.is_err(), "old schema must reject {key} presence");
    }
}
#[test]
fn schema5_compiler_declaration_selects_schema5() {
    let p=content_compiler::compile_source("skill schema5_trait(99108002) charlotte { trigger damage always { effect draw_cards(self,1); } }").unwrap();
    assert_eq!(p.manifest.content_schema_version, 5);
    p.validate().unwrap();
    assert!(
        content_native_loader::bridge_vm_skill_decls(&p).is_err(),
        "unsupported trait cannot enter runtime"
    );
}
fn finish(p: &mut ContentPackage) {
    p.manifest.content_schema_version = p.required_content_schema();
    p.manifest.logic_hash = p.compute_logic_hash();
    p.manifest.asset_hash = p.compute_asset_hash();
}
fn default5() -> ContentPackage {
    let mut p = package();
    p.skill_index[0].charlotte = Some(false);
    finish(&mut p);
    p
}
#[test]
fn schema5_old3_and_genuine4_bytes_hashes_and_readers_stay_positive() {
    for bytes in [
        include_bytes!("fixtures/c331-old-default-package.json").as_slice(),
        include_bytes!("fixtures/c331_phase8_genuine_schema4.json").as_slice(),
    ] {
        let p = ContentPackage::from_json_bytes(bytes).unwrap();
        assert_eq!(p.to_json_bytes().unwrap(), bytes);
        assert_eq!(p.manifest.logic_hash, p.compute_logic_hash());
        p.validate().unwrap();
        content_native_loader::bridge_vm_skill_decls(&p).unwrap();
        let mut old_host = content_runtime::tooling_host_versions();
        old_host.content_schema = 4;
        content_runtime::check_engine_compat(&p.manifest, &old_host).unwrap();
    }
    let p = default5();
    for schema in [0, 3, 4] {
        let mut host = content_runtime::tooling_host_versions();
        host.content_schema = schema;
        assert!(content_runtime::check_engine_compat(&p.manifest, &host).is_err());
    }
}
#[test]
fn schema5_false_empty_none_presence_downgrade_and_unknown_fields() {
    let fields = [
        ("skill_index", "charlotte", serde_json::json!(false)),
        ("skill_index", "persevere", serde_json::json!(false)),
        ("skill_index", "forever", serde_json::json!(false)),
        ("skill_index", "temporary", serde_json::json!(false)),
        ("skill_index", "group_children", serde_json::json!([])),
        ("skill_index", "reveal", serde_json::json!("none")),
        ("trigger_index", "force_die", serde_json::json!(false)),
        ("trigger_index", "force_out", serde_json::json!(false)),
        ("trigger_index", "repeat", serde_json::json!("once")),
        ("trigger_index", "hidden", serde_json::json!("public_only")),
    ];
    for (table, key, value) in fields {
        let mut wire = serde_json::to_value(package()).unwrap();
        wire[table][0][key] = value;
        wire["manifest"]["content_schema_version"] = 5.into();
        let mut p = ContentPackage::from_json_bytes(&serde_json::to_vec(&wire).unwrap()).unwrap();
        finish(&mut p);
        p.validate().unwrap();
        assert_eq!(
            serde_json::to_value(&p).unwrap()[table][0][key],
            wire[table][0][key]
        );
        for version in [3, 4] {
            let mut downgraded = p.clone();
            if version == 4 {
                downgraded.trigger_index[0].priority = 17;
            }
            downgraded.manifest.content_schema_version = version;
            downgraded.manifest.logic_hash = downgraded.compute_logic_hash();
            assert!(downgraded.validate().is_err());
            assert!(ContentPackage::from_json_bytes(&downgraded.to_json_bytes().unwrap()).is_err());
        }
        wire[table][0][key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<ContentPackage>(wire).is_err(),
            "direct SDK null {key}"
        );
    }
    for version in [0, 2, 6, u32::MAX] {
        let mut p = default5();
        p.manifest.content_schema_version = version;
        p.manifest.logic_hash = p.compute_logic_hash();
        assert!(p.validate().is_err());
    }
    for table in ["skill_index", "trigger_index"] {
        let mut wire = serde_json::to_value(default5()).unwrap();
        wire[table][0]["unreviewed_execution_switch"] = true.into();
        assert!(serde_json::from_value::<ContentPackage>(wire).is_err());
    }
}
#[test]
fn schema5_hash_binds_positions_traits_tuple_keys_edges_and_cleanup() {
    let p = default5();
    let hash = p.compute_logic_hash();
    for change in 0..12 {
        let mut q = p.clone();
        match change {
            0 => q.skill_index[0].temporary = Some(false),
            1 => q.skill_index[0].persevere = Some(false),
            2 => q.skill_index[0].group_children = Some(vec![]),
            3 => q.skill_index[0].reveal = Some(content_schema::RevealBinding::None),
            4 => q.skill_index[0].onremove = Some("host:sanjian_weapon".into()),
            5 => q.trigger_index[0].force_die = Some(false),
            6 => q.trigger_index[0].priority = 2,
            7 => q.trigger_index[0].entry_pc += 1,
            8 => q.trigger_index[0].host_key = Some("host:sanjian_weapon".into()),
            9 => q.trigger_index[0].event_skill = Some(99108001),
            10 => q.trigger_index[0].module.push('x'),
            11 => q.skill_index[0].name.push('x'),
            _ => unreachable!(),
        }
        assert_ne!(hash, q.compute_logic_hash(), "mutation{change}");
    }
    let mut pair = p.clone();
    let mut next = pair.trigger_index[0].clone();
    next.trigger_def_id += 100;
    next.filter = "always".into();
    pair.trigger_index.push(next);
    let h = pair.compute_logic_hash();
    pair.trigger_index.reverse();
    assert_ne!(h, pair.compute_logic_hash());
    let mut bytes = String::from_utf8(p.to_json_bytes().unwrap()).unwrap();
    bytes = bytes.replace(
        "\"charlotte\":false",
        "\"charlotte\":true,\"charlotte\":false",
    );
    assert!(ContentPackage::from_json_bytes(bytes.as_bytes()).is_err());
}
#[test]
fn schema5_runtime_rejects_unimplemented_mechanisms_before_engine_mutation() {
    let existing = engine_native::Engine::new(10801, 3);
    let before = existing.save();
    let before_hash = existing.state_hash();
    for change in 0..9 {
        let mut p = default5();
        match change {
            0 => p.trigger_index[0].force_die = Some(true),
            1 => p.trigger_index[0].force_out = Some(true),
            2 => p.trigger_index[0].repeat = Some(content_schema::TriggerRepeat::DamageAmount),
            3 => p.trigger_index[0].hidden = Some(content_schema::HiddenTriggerPolicy::RunHidden),
            4 => p.skill_index[0].temporary = Some(true),
            5 => p.skill_index[0].forever = Some(true),
            6 => p.skill_index[0].reveal = Some(content_schema::RevealBinding::ConfirmPublic),
            7 => p.skill_index[0].onremove = Some("host:sanjian_weapon".into()),
            8 => p.skill_index[0].onremove = Some("host:unknown_cleanup".into()),
            _ => unreachable!(),
        }
        finish(&mut p);
        p.validate().unwrap();
        assert!(
            content_native_loader::bridge_vm_skill_decls(&p).is_err(),
            "mechanism{change}"
        );
        assert_eq!(existing.save(), before);
        assert_eq!(existing.state_hash(), before_hash);
    }
}
#[test]
fn schema5_known_native_key_is_not_a_cleanup_or_window_permission() {
    for key in ["host:sanjian_weapon", "host:unknown_execution"] {
        let mut p = default5();
        p.trigger_index[0].module.clear();
        p.trigger_index[0].entry_pc = 0;
        p.trigger_index[0].host_key = Some(key.into());
        finish(&mut p);
        p.validate().unwrap();
        let error = format!(
            "{:?}",
            content_native_loader::bridge_vm_skill_decls(&p).unwrap_err()
        );
        assert!(error.contains(if key.ends_with("unknown_execution") {
            "unknown_native_execution_key"
        } else {
            "unsupported_explicit_native_declaration_binding"
        }));
        p.trigger_index[0].module = p.modules[0].name.clone();
        finish(&mut p);
        assert!(p.validate().is_err(), "native and RVM authorities conflict");
    }
}
#[test]
fn schema5_group_and_named_event_edges_resolve_genuine_dependency_handlers() {
    let child = content_compiler::compile_source_named(
        "thirdparty-child",
        "skill unrelated_child(99108003) { trigger damage always { effect draw_cards(self,1); } }",
    )
    .unwrap();
    let mut parent = default5();
    parent
        .manifest
        .dependencies
        .push(content_schema::PackageDependency {
            namespace: child.manifest.namespace.clone(),
            package_id: child.manifest.package_id.clone(),
            version: child.manifest.version.clone(),
            logic_hash: child.manifest.logic_hash,
        });
    parent.skill_index[0].group_children = Some(vec![99108003]);
    parent.skill_index[0].reveal = Some(content_schema::RevealBinding::TriggerHook {
        before: Some(99108003),
        after: None,
    });
    parent.trigger_index[0].event = "skill_begin".into();
    parent.trigger_index[0].event_skill = Some(99108003);
    finish(&mut parent);
    parent.validate().unwrap();
    ContentPackage::validate_native_mechanism_graph(&[child.clone(), parent.clone()]).unwrap();
    assert!(ContentPackage::validate_native_mechanism_graph(&[parent.clone()]).is_err());
    let root = tempfile::tempdir().unwrap();
    let store = content_runtime::PackageStore::open(root.path()).unwrap();
    let options = content_runtime::InstallOptions {
        allow_unsigned: true,
        ..Default::default()
    };
    store
        .install(&child.to_json_bytes().unwrap(), None, &options)
        .unwrap();
    store
        .install(&parent.to_json_bytes().unwrap(), None, &options)
        .unwrap();
    let lock = store.current_lock().unwrap();
    let loaded = store.load_locked_packages(&lock).unwrap();
    assert_eq!(loaded, [child.clone(), parent.clone()]);
    assert!(
        content_native_loader::bridge_vm_skill_decls(&parent).is_err(),
        "data installation is not runtime support"
    );
    for change in 0..4 {
        let mut q = parent.clone();
        match change {
            0 => q.skill_index[0].group_children = Some(vec![99108003, 99108003]),
            1 => q.skill_index[0].group_children = Some(vec![99108001]),
            2 => q.trigger_index[0].event_skill = Some(0),
            3 => {
                q.skill_index[0].reveal = Some(content_schema::RevealBinding::TriggerHook {
                    before: Some(0),
                    after: None,
                })
            }
            _ => unreachable!(),
        };
        finish(&mut q);
        assert!(
            q.validate().is_err()
                || ContentPackage::validate_native_mechanism_graph(&[child.clone(), q]).is_err()
        );
    }
}
#[test]
fn schema5_default_declaration_lock_is_bound_and_public_sdk_native_vm_conflict_rejects() {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    let p = default5();
    let vm = content_native_loader::bridge_vm_skill_decls(&p)
        .unwrap()
        .decls;
    let mut pack = ExtensionPack {
        def: ExtensionDef {
            name: "thirdparty-schema5",
            version: 1,
            description: "SDK equal validation",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: vm,
        init_hook: InitHook::None,
        enabled: true,
    };
    let e = engine_native::Engine::new_with_extensions(10802, 3, &[&pack]).unwrap();
    let hash = e.ruleset_version();
    let mut explicit = p.clone();
    explicit.skill_index[0].temporary = Some(false);
    finish(&mut explicit);
    pack.vm_skills = content_native_loader::bridge_vm_skill_decls(&explicit)
        .unwrap()
        .decls;
    let second = engine_native::Engine::new_with_extensions(10802, 3, &[&pack]).unwrap();
    assert_ne!(hash, second.ruleset_version());
    pack.host_skills
        .push(engine_native::content::host::HostSkillDecl {
            name: "schema5_draw",
            behavior: engine_native::content::host::HostBehavior::SanjianWeaponDamage,
        });
    assert!(engine_native::Engine::new_with_extensions(10802, 3, &[&pack]).is_err());
}

#[test]
fn schema5_invalid_data_install_is_atomic_and_default_vm_body_still_executes() {
    let dir = tempfile::tempdir().unwrap();
    let store = content_runtime::PackageStore::open(dir.path()).unwrap();
    let options = content_runtime::InstallOptions {
        allow_unsigned: true,
        ..Default::default()
    };
    let p = default5();
    store
        .install(&p.to_json_bytes().unwrap(), None, &options)
        .unwrap();
    let lock = store.current_lock().unwrap();
    let mut invalid = p.clone();
    invalid.manifest.content_schema_version = 3;
    invalid.manifest.logic_hash = invalid.compute_logic_hash();
    assert!(store
        .install(&invalid.to_json_bytes().unwrap(), None, &options)
        .is_err());
    assert_eq!(lock, store.current_lock().unwrap());
    assert_eq!(store.load_locked_packages(&lock).unwrap(), [p.clone()]);
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "thirdparty-schema5-positive",
            version: 1,
            description: "supported default VM semantics",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut engine = engine_native::Engine::new_with_extensions(10803, 3, &[&pack]).unwrap();
    engine.debug_grant(1, "schema5_draw");
    let hand = engine.state_core().player(1).hand.len();
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(engine.state_core().player(1).hand.len(), hand + 1);
}

#[test]
fn schema5_windows_policy_combinations_and_real_child_cycle_reject() {
    let mut p=content_compiler::compile_source_named("schema5-cycle", "skill cycle_parent(99108004) { trigger damage always { effect draw_cards(self,1); } } skill real_child(99108005) { trigger damage always { effect draw_cards(self,1); } }").unwrap();
    p.skill_index[0].group_children = Some(vec![99108005]);
    p.skill_index[1].group_children = Some(vec![99108004]);
    finish(&mut p);
    assert!(p.validate().unwrap_err().contains("cyclic"));
    let mut p = default5();
    p.trigger_index[0].event_skill = Some(99108001);
    finish(&mut p);
    assert!(p.validate().unwrap_err().contains("event_skill"));
    let mut p = default5();
    p.trigger_index[0].event = "cards_drawn".into();
    p.trigger_index[0].repeat = Some(content_schema::TriggerRepeat::DamageAmount);
    finish(&mut p);
    assert!(p.validate().is_err());
    let mut p = content_compiler::compile_source_named(
        "schema5-rescue",
        "skill schema5_rescue(99108006) { active play_phase { effect draw_cards(self,1); } }",
    )
    .unwrap();
    p.skill_index[0].charlotte = Some(false);
    p.active_index[0].window = "dying".into();
    finish(&mut p);
    p.validate().unwrap();
    assert!(format!(
        "{:?}",
        content_native_loader::bridge_vm_skill_decls(&p).unwrap_err()
    )
    .contains("unsupported_rescue_action_window"));
    let mut wire = serde_json::to_value(default5()).unwrap();
    wire["skill_index"][0]["onremove"] = serde_json::json!({"callback":"script"});
    assert!(ContentPackage::from_json_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}
