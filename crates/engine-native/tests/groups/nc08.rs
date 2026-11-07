//! engine-native 集成测试分组 `nc08`：NC08 非上游对照样本（`nc08_*`，`nc08_upstream_*` 除外）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../nc01_nc08_multi_mode_matrix.rs"]
mod nc01_nc08_multi_mode_matrix;

#[path = "../nc08_acceptance_sweep_all_packages.rs"]
mod nc08_acceptance_sweep_all_packages;

#[path = "../nc08_acceptance_sweep_small_packages.rs"]
mod nc08_acceptance_sweep_small_packages;

#[path = "../nc08_card_name_prefix_regression.rs"]
mod nc08_card_name_prefix_regression;

#[path = "../nc08_card_ownership_partition.rs"]
mod nc08_card_ownership_partition;

#[path = "../nc08_composition_acceptance.rs"]
mod nc08_composition_acceptance;

#[path = "../nc08_composition_pair_matrix.rs"]
mod nc08_composition_pair_matrix;

#[path = "../nc08_config_acceptance.rs"]
mod nc08_config_acceptance;

#[path = "../nc08_conversion_gap_regression.rs"]
mod nc08_conversion_gap_regression;

#[path = "../nc08_cross_target_real.rs"]
mod nc08_cross_target_real;

#[path = "../nc08_dead_host_keys.rs"]
mod nc08_dead_host_keys;

#[path = "../nc08_general_pair_matrix.rs"]
mod nc08_general_pair_matrix;

#[path = "../nc08_global_cards_active_use_probe.rs"]
mod nc08_global_cards_active_use_probe;

#[path = "../nc08_identity_gate_matrix.rs"]
mod nc08_identity_gate_matrix;

#[path = "../nc08_loader_card_skills.rs"]
mod nc08_loader_card_skills;

#[path = "../nc08_manifest_matrix.rs"]
mod nc08_manifest_matrix;

#[path = "../nc08_phase_end_dispatch_probe.rs"]
mod nc08_phase_end_dispatch_probe;

#[path = "../nc08_pile_pair_matrix.rs"]
mod nc08_pile_pair_matrix;

#[path = "../nc08_prepare_phase_mapping.rs"]
mod nc08_prepare_phase_mapping;

#[path = "../nc08_registered_defect_guards.rs"]
mod nc08_registered_defect_guards;

#[path = "../nc08_replay_determinism_five_sources.rs"]
mod nc08_replay_determinism_five_sources;

#[path = "../nc08_rescue_card_set_survey.rs"]
mod nc08_rescue_card_set_survey;

#[path = "../nc08_save_restore_four_sources.rs"]
mod nc08_save_restore_four_sources;

#[path = "../nc08_save_restore_matrix.rs"]
mod nc08_save_restore_matrix;

#[path = "../nc08_save_restore_pile.rs"]
mod nc08_save_restore_pile;

#[path = "../nc08_skill_mapping_inventory.rs"]
mod nc08_skill_mapping_inventory;

#[path = "../nc08_skill_pair_matrix.rs"]
mod nc08_skill_pair_matrix;

#[path = "../nc08_standard_deck_coverage.rs"]
mod nc08_standard_deck_coverage;

#[path = "../nc08_swd_yuchan_family_probe.rs"]
mod nc08_swd_yuchan_family_probe;

#[path = "../nc08_trigger_order_counterexample.rs"]
mod nc08_trigger_order_counterexample;

#[path = "../nc08_u08_reachability.rs"]
mod nc08_u08_reachability;

#[path = "../nc08_upgrade_acceptance.rs"]
mod nc08_upgrade_acceptance;

#[path = "../nc08_upgrade_five_sources.rs"]
mod nc08_upgrade_five_sources;

#[path = "../nc08_upgrade_matrix.rs"]
mod nc08_upgrade_matrix;

#[path = "../nc08_upgrade_skill_mapping.rs"]
mod nc08_upgrade_skill_mapping;

#[path = "../nc08_zhenfa_mode_gate_survey.rs"]
mod nc08_zhenfa_mode_gate_survey;

#[path = "../c331_damage_windows.rs"]
mod c331_damage_windows;

#[path = "../c331_fellow_windows.rs"]
mod c331_fellow_windows;

#[path = "../c331_host_source_metadata.rs"]
mod c331_host_source_metadata;

#[path = "../c331_source_order_phase5.rs"]
mod c331_source_order_phase5;
