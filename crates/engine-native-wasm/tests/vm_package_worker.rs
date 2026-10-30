//! Worker 的安装载荷必须支持完整 npack（modules/trigger_index），
//! 否则服务端可执行的 VM 包在客户端会被“缺少 cards/characters/pile”拒绝。

use engine_native_wasm::test_support::{
    assign_general_for, engine_from_archive, engine_from_packs, general_candidates_json_for,
    hex_encode,
};
use serde_json::json;

const DSL: &str = r#"
skill worker_phase_end(991001) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
"#;

#[test]
fn vm_only_npack_loads_skills_and_generals() {
    let package = content_compiler::compile_source_named("worker-vm", DSL).expect("compile");
    let vm_skill = package.trigger_index[0].module.clone();
    let mut npack = serde_json::to_value(&package).unwrap();
    let characters = json!({
        "source": "test",
        "upstream_path": "worker-vm",
        "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "worker-vm", "version": 1, "description": "test" },
        "generals": [{
            "id": "worker_vm_probe",
            "name": "worker_vm_probe",
            "faction": "qun",
            "hp": 4,
            "gender": "male",
            "skills": [{ "name": vm_skill, "title": "探针" }]
        }]
    });
    npack["characters"] = characters;
    let mut broken = npack.clone();
    broken["characters"]["generals"][0]["skills"][0]["name"] = json!("missing_vm_skill");
    let error = engine_from_packs(4, 42, &json!([broken]).to_string())
        .err()
        .expect("未解析的 VM 武将技能必须拒绝");
    assert!(error.contains("missing_vm_skill"), "{error}");
    let mut no_triggers = npack.clone();
    no_triggers["trigger_index"] = json!([]);
    let no_trigger_package: content_schema::ContentPackage =
        serde_json::from_value(no_triggers.clone()).unwrap();
    no_triggers["manifest"]["logic_hash"] = json!(no_trigger_package.compute_logic_hash());
    let error = engine_from_packs(4, 42, &json!([no_triggers]).to_string())
        .err()
        .expect("未桥接的模块不能静默丢失武将技能");
    assert!(error.contains(&vm_skill), "{error}");
    let payload = json!([npack]).to_string();
    let mut engine = engine_from_packs(4, 42, &payload).expect("Worker 应接受完整 npack");
    assert!(
        engine
            .catalog()
            .skill_by_name("worker_phase_end_phase_end")
            .is_some(),
        "VM 技能应进入 Worker 目录"
    );
    assert!(
        general_candidates_json_for(&engine).contains("worker_vm_probe"),
        "characters 资产应进入 Worker 选将目录"
    );
    assign_general_for(&mut engine, 0, "worker_vm_probe").expect("assign VM general");
    assert!(engine.view_for(0).players[0].skills.contains(&vm_skill.as_str()));
    engine.debug_set_hp(0, 3);
    let hex = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let mut restored = engine_from_archive(&hex, &payload).expect("按原包恢复");

    // 普通摸牌不能证明技能触发；仅在结束阶段观察到回复才算通过。
    for branch in [&mut engine, &mut restored] {
        advance_until_vm_recovery(branch);
    }
    assert_eq!(engine.state_hash(), restored.state_hash());
}

fn advance_until_vm_recovery(engine: &mut engine_native::Engine) {
    for step in 0..80u64 {
        if engine.view_for(0).players[0].hp == 4 {
            assert_eq!(engine.turn().current, 0);
            assert_eq!(engine.turn().phase, engine_native::Phase::End);
            return;
        }
        if engine.outcome().is_some() {
            break;
        }
        if let Some(choice) = engine.open_choice() {
            let take = choice.spec.min as usize;
            let selection = choice.spec.options.iter().take(take).cloned().collect();
            let actor = choice.observer;
            let choice_id = choice.id;
            let result = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 20_000 + step,
                action: engine_native::GameAction::Answer {
                    choice_id,
                    selection,
                },
            });
            assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)), "{result:?}");
        } else {
            let actor = engine.turn().current;
            let result = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 20_000 + step,
                action: engine_native::GameAction::NextPhase,
            });
            assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)), "{result:?}");
        }
    }
    panic!("Worker 安装载荷的 VM 技能未在结束阶段回复 1 点体力");
}

#[test]
fn npack_without_payload_or_modules_still_refused() {
    let payload = json!([{
        "manifest": {
            "package_format_version": 1,
            "namespace": "x",
            "package_id": "empty",
            "version": "1.0.0",
            "content_schema_version": 3,
            "rule_vm_abi_version": 734,
            "min_engine_api": 1,
            "max_engine_api": 1
        }
    }])
    .to_string();
    assert!(engine_from_packs(2, 1, &payload).is_err());
}

#[test]
fn onlyol_effect_collision_and_future_abi_are_rejected_before_start_or_restore() {
    let old = include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4.npack.json");
    let new = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
    let old_archive = include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/native-v322.ngsa");
    for (bytes, expected) in [
        (old.as_slice(), "ambiguous effect 656"),
        (new.as_slice(), "phase_prepare"),
    ] {
        let mut package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        // Isolate the package check from unrelated dependency installation.
        package.manifest.dependencies.clear();
        package.manifest.logic_hash = package.compute_logic_hash();
        let payload = json!([package]).to_string();
        let start_error = engine_from_packs(4, 42, &payload).err().unwrap();
        assert!(start_error.contains(expected), "{start_error}");
        let restore_error = engine_from_archive(&hex_encode(old_archive), &payload).err().unwrap();
        assert!(restore_error.contains(expected), "{restore_error}");
    }
}

#[test]
fn characters_only_payload_still_loads_general_with_builtin_skill() {
    let characters = json!({
        "source": "test",
        "upstream_path": "worker-char-only",
        "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "worker-char-only", "version": 1, "description": "test" },
        "generals": [{
            "id": "worker_char_only",
            "name": "worker_char_only",
            "faction": "qun",
            "hp": 4,
            "gender": "male",
            "skills": [{ "name": "护驾", "mapping": "builtin:护驾" }]
        }]
    });
    let payload = json!([{
        "manifest": {
            "package_format_version": 1,
            "namespace": "test",
            "package_id": "worker-char-only",
            "version": "1.0.0",
            "content_schema_version": 3,
            "rule_vm_abi_version": 734,
            "min_engine_api": 1,
            "max_engine_api": 1
        },
        "characters": characters
    }])
    .to_string();
    let mut engine = engine_from_packs(2, 7, &payload).expect("characters-only payload");
    assert!(general_candidates_json_for(&engine).contains("worker_char_only"));
    assign_general_for(&mut engine, 0, "worker_char_only").expect("assign");
    assert!(engine
        .view_for(0)
        .players[0]
        .skills
        .iter()
        .any(|skill| *skill == "护驾"));
}
