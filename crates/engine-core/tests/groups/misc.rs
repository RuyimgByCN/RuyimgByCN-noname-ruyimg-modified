//! engine-core 集成测试分组 `misc`：其余（国战锦囊、golden、UP37、属性/安全/差分等）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../content_lock_extensions.rs"]
mod content_lock_extensions;

#[path = "../differential_replay.rs"]
mod differential_replay;

#[path = "../draw_fixed_runtime.rs"]
mod draw_fixed_runtime;

#[path = "../fuzz_stable.rs"]
mod fuzz_stable;

#[path = "../golden_regression.rs"]
mod golden_regression;

#[path = "../golden_start.rs"]
mod golden_start;

#[path = "../guozhan_trick_chiling.rs"]
mod guozhan_trick_chiling;

#[path = "../guozhan_trick_guguoanbang.rs"]
mod guozhan_trick_guguoanbang;

#[path = "../guozhan_trick_haolingtianxia.rs"]
mod guozhan_trick_haolingtianxia;

#[path = "../guozhan_trick_huoshaolianying.rs"]
mod guozhan_trick_huoshaolianying;

#[path = "../guozhan_trick_kefuzhongyuan.rs"]
mod guozhan_trick_kefuzhongyuan;

#[path = "../guozhan_trick_lianjunshengyan.rs"]
mod guozhan_trick_lianjunshengyan;

#[path = "../guozhan_trick_lulitongxin.rs"]
mod guozhan_trick_lulitongxin;

#[path = "../guozhan_trick_shuiyanqijunx.rs"]
mod guozhan_trick_shuiyanqijunx;

#[path = "../guozhan_trick_wenheluanwu.rs"]
mod guozhan_trick_wenheluanwu;

#[path = "../guozhan_trick_yiyi.rs"]
mod guozhan_trick_yiyi;

#[path = "../guozhan_trick_yuanjiao.rs"]
mod guozhan_trick_yuanjiao;

#[path = "../gwent_gwyewu.rs"]
mod gwent_gwyewu;

#[path = "../m5_community_example.rs"]
mod m5_community_example;

#[path = "../m6_ai_baseline.rs"]
mod m6_ai_baseline;

#[path = "../m8_governance.rs"]
mod m8_governance;

#[path = "../mode_single.rs"]
mod mode_single;

#[path = "../nc02_vm_module_hash.rs"]
mod nc02_vm_module_hash;

#[path = "../nc04_vm_control_flow.rs"]
mod nc04_vm_control_flow;

#[path = "../original_small_pack_asserts.rs"]
mod original_small_pack_asserts;

#[path = "../pa12_dsl_legacy_differential.rs"]
mod pa12_dsl_legacy_differential;

#[path = "../pa12_tag656_compat.rs"]
mod pa12_tag656_compat;

#[path = "../property_invariants.rs"]
mod property_invariants;

#[path = "../real_extensions_lock.rs"]
mod real_extensions_lock;

#[path = "../security_observer_leak.rs"]
mod security_observer_leak;

#[path = "../trick_card_effects.rs"]
mod trick_card_effects;

#[path = "../up37_r03_mark_modifier_validation.rs"]
mod up37_r03_mark_modifier_validation;

#[path = "../up37_r03_recast_card_identity.rs"]
mod up37_r03_recast_card_identity;

#[path = "../up37_r10_metadata.rs"]
mod up37_r10_metadata;
