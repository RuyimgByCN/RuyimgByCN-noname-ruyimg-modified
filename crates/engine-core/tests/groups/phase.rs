//! engine-core 集成测试分组 `phase`：阶段测试（`phase*`）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../phase12_content_vm.rs"]
mod phase12_content_vm;

#[path = "../phase13_modifiers.rs"]
mod phase13_modifiers;

#[path = "../phase14_manifest.rs"]
mod phase14_manifest;

#[path = "../phase16_manifest_persistence.rs"]
mod phase16_manifest_persistence;

#[path = "../phase19_replay_capsule.rs"]
mod phase19_replay_capsule;

#[path = "../phase2_rules.rs"]
mod phase2_rules;

#[path = "../phase3_trigger_query.rs"]
mod phase3_trigger_query;

#[path = "../phase4_vm_bridge.rs"]
mod phase4_vm_bridge;

#[path = "../phase5_identity.rs"]
mod phase5_identity;

#[path = "../phase_m0_choose.rs"]
mod phase_m0_choose;

#[path = "../phase_m0_intent.rs"]
mod phase_m0_intent;

#[path = "../phase_m0_move_cards.rs"]
mod phase_m0_move_cards;

#[path = "../phase_m0_trigger_choice.rs"]
mod phase_m0_trigger_choice;

#[path = "../phase_m0_unified_skills.rs"]
mod phase_m0_unified_skills;

#[path = "../phase_m1_aoe_juedou.rs"]
mod phase_m1_aoe_juedou;

#[path = "../phase_m1_bagua.rs"]
mod phase_m1_bagua;

#[path = "../phase_m1_delayed_wuxie.rs"]
mod phase_m1_delayed_wuxie;

#[path = "../phase_m1_elemental_sha.rs"]
mod phase_m1_elemental_sha;

#[path = "../phase_m1_guanxing.rs"]
mod phase_m1_guanxing;

#[path = "../phase_m1_guicai_tiesuo.rs"]
mod phase_m1_guicai_tiesuo;

#[path = "../phase_m1_huogong_guohe.rs"]
mod phase_m1_huogong_guohe;

#[path = "../phase_m1_judge_delayed.rs"]
mod phase_m1_judge_delayed;

#[path = "../phase_m1_keji.rs"]
mod phase_m1_keji;

#[path = "../phase_m1_kongcheng.rs"]
mod phase_m1_kongcheng;

#[path = "../phase_m1_luoyi.rs"]
mod phase_m1_luoyi;

#[path = "../phase_m1_new_skills.rs"]
mod phase_m1_new_skills;

#[path = "../phase_m1_qianxun.rs"]
mod phase_m1_qianxun;

#[path = "../phase_m1_qingguo_yiji.rs"]
mod phase_m1_qingguo_yiji;

#[path = "../phase_m1_response_zhiheng.rs"]
mod phase_m1_response_zhiheng;

#[path = "../phase_m1_standard_content.rs"]
mod phase_m1_standard_content;

#[path = "../phase_m1_tengjia.rs"]
mod phase_m1_tengjia;

#[path = "../phase_m1_view_as.rs"]
mod phase_m1_view_as;

#[path = "../phase_m1_wuxie.rs"]
mod phase_m1_wuxie;
