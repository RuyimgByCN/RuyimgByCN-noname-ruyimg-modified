use super::*;

use rule_api::{EffectRequest, Suit};

use rule_vm::{CollectingHost, ExecResult, HostContext, RuleVm};

/// PA06/B1：优化后端只接受装载期已校验模块——编译产物在此一次性校验再执行
/// （同 rule-vm 自测助手 `vmod`）。
fn vmod(module: &rule_vm::BytecodeModule) -> rule_vm::VerifiedModule {
    rule_vm::VerifiedModule::verify(module.clone()).expect("compiled module must verify")
}

fn legacy_effect_tag(tag: u16) -> u8 {
    u8::try_from(tag).expect("legacy HOST_EFFECT tag must fit in one byte")
}

const JIANXIONG: &str = r#"
skill jianxiong(1) {
  trigger damage owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;

mod active_other_target_and_same_color_contract;
mod choose_add_missing_skill_requires_abi79;
mod ddd_zhouchu_contract_requires_abi618;
mod distinct_suit_draw_and_heart_inspection_require;
mod global_incoming_damage_auras;
mod hand_empty_target_scope_requires_abi162;
mod jianxiong_reproducible;
mod mbzhuguo_requires_abi331_and_exact_active_contract;
mod reveal_card_rejects_player_operand;
mod reveal_hand_discard_kind_requires_abi144;
mod skill_pile_storage_and_dying_jiu_require;
mod twqingkou_requires_abi396;
