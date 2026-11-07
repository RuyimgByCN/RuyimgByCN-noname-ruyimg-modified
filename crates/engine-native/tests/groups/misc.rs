//! engine-native 集成测试分组 `misc`：其余轮次（PA/T/U/D5/E02/史诗/AI/Boss 等，未归入上述前缀的文件）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../ai_content_tuning.rs"]
mod ai_content_tuning;

#[path = "../ai_planner.rs"]
mod ai_planner;

#[path = "../boss_play_eight.rs"]
mod boss_play_eight;

#[path = "../d5_longxue_mbqianlong.rs"]
mod d5_longxue_mbqianlong;

#[path = "../e02_c3_lifecycle.rs"]
mod e02_c3_lifecycle;

#[path = "../epic_rangjie_after_damage.rs"]
mod epic_rangjie_after_damage;

#[path = "../epic_reguhuo_viewas.rs"]
mod epic_reguhuo_viewas;

#[path = "../fuzz_corpus_replay.rs"]
mod fuzz_corpus_replay;

#[path = "../pa01_catalog_index_equivalence.rs"]
mod pa01_catalog_index_equivalence;

#[path = "../pa08_fullhand_dump.rs"]
mod pa08_fullhand_dump;

#[path = "../pa12_batch1_vocabulary.rs"]
mod pa12_batch1_vocabulary;

#[path = "../pa12_batch2_phase_events.rs"]
mod pa12_batch2_phase_events;

#[path = "../pa12_batch3_filter_predicates.rs"]
mod pa12_batch3_filter_predicates;

#[path = "../pa12_batch3b_once_limits.rs"]
mod pa12_batch3b_once_limits;

#[path = "../pa12_batch4_1_host_primitives.rs"]
mod pa12_batch4_1_host_primitives;

#[path = "../pa12_batch6_events_and_cards.rs"]
mod pa12_batch6_events_and_cards;

#[path = "../pa12_batch7_codec_named_and_once.rs"]
mod pa12_batch7_codec_named_and_once;

#[path = "../pa12_batch8_deterministic_effects.rs"]
mod pa12_batch8_deterministic_effects;

#[path = "../pa12_batch9_events_and_turn_effects.rs"]
mod pa12_batch9_events_and_turn_effects;

#[path = "../pa12_batch10_combat_and_state_effects.rs"]
mod pa12_batch10_combat_and_state_effects;

#[path = "../pa12_batch11_hand_draw_and_card_effects.rs"]
mod pa12_batch11_hand_draw_and_card_effects;

#[path = "../pa12_batch12_selected_and_deterministic_effects.rs"]
mod pa12_batch12_selected_and_deterministic_effects;

#[path = "../pa12_batch13_interactive_and_up37_skills.rs"]
mod pa12_batch13_interactive_and_up37_skills;

#[path = "../pa12_batch14_modifiers_actives_and_choices.rs"]
mod pa12_batch14_modifiers_actives_and_choices;

#[path = "../pa12_batch15_view_as_and_resolvers.rs"]
mod pa12_batch15_view_as_and_resolvers;

#[path = "../pa12_batch16_modifiers_actives_and_resolvers.rs"]
mod pa12_batch16_modifiers_actives_and_resolvers;

#[path = "../pa12_dsl_native_prototype.rs"]
mod pa12_dsl_native_prototype;

#[path = "../pa12_tag656_version.rs"]
mod pa12_tag656_version;

#[path = "../standard_catalog_inventory.rs"]
mod standard_catalog_inventory;

#[path = "../t11_boss_pangtong_niepan.rs"]
mod t11_boss_pangtong_niepan;

#[path = "../t11_boss_zhugeliang_hlss.rs"]
mod t11_boss_zhugeliang_hlss;

#[path = "../t12_dual_diag.rs"]
mod t12_dual_diag;

#[path = "../t12_g2g4_channels.rs"]
mod t12_g2g4_channels;

#[path = "../t12_g5_relays.rs"]
mod t12_g5_relays;

#[path = "../t13_info_isolation.rs"]
mod t13_info_isolation;

#[path = "../t8_rvm_bridge.rs"]
mod t8_rvm_bridge;

#[path = "../t9_d5_surface.rs"]
mod t9_d5_surface;

#[path = "../u01_d3_moyan.rs"]
mod u01_d3_moyan;

#[path = "../u03_cardpile_eight.rs"]
mod u03_cardpile_eight;

#[path = "../u04_coin_eight.rs"]
mod u04_coin_eight;

#[path = "../u08_hero_kill_batch1.rs"]
mod u08_hero_kill_batch1;

#[path = "../u08_menshen_batch2.rs"]
mod u08_menshen_batch2;

#[path = "../nc09_referee_projection.rs"]
mod nc09_referee_projection;

#[path = "../nc09_machine_ownership.rs"]
mod nc09_machine_ownership;
#[path = "../nc09_gaiming_cancel.rs"]
mod nc09_gaiming_cancel;
#[path = "../r05_turn_model.rs"]
mod r05_turn_model;

#[path = "../r05e_compounds.rs"]
mod r05e_compounds;

#[path = "../r05f_real_packages.rs"]
mod r05f_real_packages;

#[path = "../r05f2_semantic_gaps.rs"]
mod r05f2_semantic_gaps;

#[path = "../r05g_archive107.rs"]
mod r05g_archive107;

#[path = "../r05c_async_draw.rs"]
mod r05c_async_draw;

#[path = "../r05h_projection.rs"]
mod r05h_projection;

#[path = "../r05i_byte_chain.rs"]
mod r05i_byte_chain;

#[path = "../r05j_acceptance_matrix.rs"]
mod r05j_acceptance_matrix;

#[path = "../adr0786_j2_active_bridge.rs"]
mod adr0786_j2_active_bridge;

#[path = "../adr0786_j2_effect_choices.rs"]
mod adr0786_j2_effect_choices;

#[path = "../adr0786_j2_package_versions.rs"]
mod adr0786_j2_package_versions;

#[path = "../adr0786_j2_review_regressions.rs"]
mod adr0786_j2_review_regressions;

#[path = "../c331_package_schema.rs"]
mod c331_package_schema;

#[path = "../i02_public_presentation_closure.rs"]
mod i02_public_presentation_closure;
