//! engine-native 集成测试分组 `nc_rounds`：NC 各轮（`nc<编号>_*`，NC08 除外）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../nc01_seat_semantics.rs"]
mod nc01_seat_semantics;

#[path = "../nc03_ai_consistency.rs"]
mod nc03_ai_consistency;

#[path = "../nc03_legal_actions.rs"]
mod nc03_legal_actions;

#[path = "../nc04_cards_given_event.rs"]
mod nc04_cards_given_event;

#[path = "../nc04_vm_fault.rs"]
mod nc04_vm_fault;

#[path = "../nc04_vm_resume.rs"]
mod nc04_vm_resume;

#[path = "../nc05_general_select.rs"]
mod nc05_general_select;

#[path = "../nc06_generic_content_state.rs"]
mod nc06_generic_content_state;

#[path = "../nc06_zhucheng_restore.rs"]
mod nc06_zhucheng_restore;

#[path = "../nc07_archive_hash_fields.rs"]
mod nc07_archive_hash_fields;

#[path = "../nc07_archive_hash_guard.rs"]
mod nc07_archive_hash_guard;

#[path = "../nc07_discard_duplicate_probe.rs"]
mod nc07_discard_duplicate_probe;

#[path = "../nc07_lianyaohu_hash.rs"]
mod nc07_lianyaohu_hash;

#[path = "../nc07_state_same_source.rs"]
mod nc07_state_same_source;

#[path = "../nc09_identity_end_to_end.rs"]
mod nc09_identity_end_to_end;

#[path = "../nc09_projection_and_errors.rs"]
mod nc09_projection_and_errors;

#[path = "../nc10_ai_boundary_and_cancel.rs"]
mod nc10_ai_boundary_and_cancel;

#[path = "../nc10_ai_bounds.rs"]
mod nc10_ai_bounds;

#[path = "../nc10_ai_information_boundary.rs"]
mod nc10_ai_information_boundary;

#[path = "../nc10_duel_nullify_privacy.rs"]
mod nc10_duel_nullify_privacy;

#[path = "../nc10_multiple_hidden_traps.rs"]
mod nc10_multiple_hidden_traps;

#[path = "../nc10_pending_duel_privacy.rs"]
mod nc10_pending_duel_privacy;

#[path = "../nc10_planner_pool.rs"]
mod nc10_planner_pool;

#[path = "../nc10_single_hidden_trap.rs"]
mod nc10_single_hidden_trap;

#[path = "../nc10_skill_pile_conservation.rs"]
mod nc10_skill_pile_conservation;

#[path = "../nc12_ai_strength_baseline.rs"]
mod nc12_ai_strength_baseline;

#[path = "../nc12_dangling_reference_probe.rs"]
mod nc12_dangling_reference_probe;

#[path = "../nc12_dangling_reference_repro.rs"]
mod nc12_dangling_reference_repro;

#[path = "../nc12_swd_authoritative_stability.rs"]
mod nc12_swd_authoritative_stability;

#[path = "../nc10_live_hidden_cards.rs"]
mod nc10_live_hidden_cards;

#[path = "../nc10_opaque_vm_privacy.rs"]
mod nc10_opaque_vm_privacy;

#[path = "../nc10_redaction_helper_contract.rs"]
mod nc10_redaction_helper_contract;

#[path = "../nc10_j2_vm_privacy.rs"]
mod nc10_j2_vm_privacy;

#[path = "../nc10_source_provenance.rs"]
mod nc10_source_provenance;

#[path = "../c331_source33_integration.rs"]
mod c331_source33_integration;

#[path = "../c331_schema5_datachain.rs"]
mod c331_schema5_datachain;

#[path = "../c331_atomic_source_repair.rs"]
mod c331_atomic_source_repair;
#[path = "../c331_canonical_damage.rs"]
mod c331_canonical_damage;
#[path = "../c331_prehp_damage.rs"]
mod c331_prehp_damage;
#[path = "../c331_source_predicates.rs"]
mod c331_source_predicates;

#[path = "../c331_genuine_invocations.rs"]
mod c331_genuine_invocations;
