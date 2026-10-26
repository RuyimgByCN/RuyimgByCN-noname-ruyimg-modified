//! Worker 的安装载荷必须支持完整 npack（modules/trigger_index），
//! 否则服务端可执行的 VM 包在客户端会被“缺少 cards/characters/pile”拒绝。

use engine_native_wasm::test_support::{
    assign_general_for, engine_from_packs, general_candidates_json_for,
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
            "skills": [{ "name": "worker_vm_probe_phase_end", "title": "探针" }]
        }]
    });
    npack["characters"] = characters;
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

    // 不只是目录：Worker 安装载荷里的 VM 技能要能经武将真实触发（phase_end 摸 1）。
    let hand_before = engine.view_for(0).players[0].hand_count;
    let mut converged = false;
    for step in 0..80u64 {
        if engine.view_for(0).players[0].hand_count > hand_before {
            converged = true;
            break;
        }
        if engine.outcome().is_some() {
            break;
        }
        if let Some(choice) = engine.open_choice() {
            let take = choice.spec.min as usize;
            let selection = choice.spec.options.iter().take(take).cloned().collect();
            let actor = choice.observer;
            let choice_id = choice.id;
            let _ = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 10_000 + step,
                action: engine_native::GameAction::Answer {
                    choice_id,
                    selection,
                },
            });
        } else {
            let actor = engine.turn().current;
            let _ = engine.submit(engine_native::GameCommand {
                actor,
                expected_revision: engine.revision(),
                dedup_id: 20_000 + step,
                action: engine_native::GameAction::NextPhase,
            });
        }
    }
    assert!(converged, "Worker 安装载荷的 VM 技能应在 phase_end 回复 1 点体力");
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
