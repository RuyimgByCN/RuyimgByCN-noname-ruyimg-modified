//! DSL content compiler pipeline.
//!
//! ```text
//! DSL → Parse → Type Check → Capability Check → Rule IR → Bytecode → Verify → Package
//! ```

#![deny(missing_docs)]

mod ast;
mod codegen;
mod lexer;
mod lsp;
mod parse;
mod typecheck;

pub use codegen::compile_module_ir;
pub use lsp::{
    byte_offset_to_position, completion, diagnostics, hover, span_to_range, LspCompletionItem,
    LspDiagnostic, LspHover, LspPosition, LspRange, LspSeverity,
};
pub use parse::parse_source;
pub use typecheck::{filter_vocabulary, typecheck};

use content_schema::{
    ContentPackage, PackageActiveEntry, PackageManifest, PackageModifierEntry,
    PackageSelectionSpec, PackageSkillEntry, PackageTargetSpec, PackageTriggerEntry,
    PackageViewAsEntry, RuleModuleIr, StmtIr,
};
use rule_vm::bytecode::caps;
use rule_vm::verify_module;
use thiserror::Error;

/// Compiler semantic version (enters logic hash).
pub const COMPILER_VERSION: &str = "0.5.0-active-selection";

mod abi;
mod abi_versions;
use abi::required_rule_vm_abi;
use abi_versions::*;

/// Compiler error with optional source location.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Lex error.
    #[error("lex error at {pos}: {message}")]
    Lex {
        /// Byte position.
        pos: usize,
        /// Message.
        message: String,
    },
    /// Parse error.
    #[error("parse error at {pos}: {message}")]
    Parse {
        /// Byte position.
        pos: usize,
        /// Message.
        message: String,
    },
    /// Type / name / capability error.
    #[error("type error at {pos}: {message}")]
    Type {
        /// Byte position.
        pos: usize,
        /// Message.
        message: String,
    },
    /// Codegen / verify error.
    #[error("codegen: {0}")]
    Codegen(String),
}

/// Compile DSL source into a reproducible content package.
pub fn compile_source(source: &str) -> Result<ContentPackage, CompileError> {
    compile_source_named("main", source)
}

/// Compile with package id.
pub fn compile_source_named(
    package_id: &str,
    source: &str,
) -> Result<ContentPackage, CompileError> {
    let ast = parse_source(source)?;
    let ir = typecheck(&ast)?;
    let (modules, triggers, modifiers, actives, view_as, maps, module_required_abi) =
        codegen::compile_module_ir_with_required_abi(&ir)?;

    for m in &modules {
        verify_module(m).map_err(|e| CompileError::Codegen(e.to_string()))?;
    }

    let mut manifest = PackageManifest::new("official", package_id, "0.1.0");
    manifest.rule_vm_abi_version = required_rule_vm_abi(&ir, module_required_abi);
    manifest.capabilities = rule_api::CapabilitySet::none();
    for module in &modules {
        manifest.capabilities.emit_effects |= module.caps_mask & caps::EMIT_EFFECTS != 0;
        manifest.capabilities.run_queries |= module.caps_mask & caps::RUN_QUERIES != 0;
        manifest.capabilities.request_choice |= module.caps_mask & caps::REQUEST_CHOICE != 0;
    }

    let trigger_index: Vec<PackageTriggerEntry> = triggers
        .into_iter()
        .map(|t| PackageTriggerEntry {
            host_key: None,
            force_die: None,
            force_out: None,
            repeat: None,
            hidden: None,
            event_skill: None,

            trigger_def_id: t.trigger_def_id,
            skill_def_id: t.skill_def_id,
            event: t.event,
            filter: t.filter,
            optional: t.optional,
            timing: Default::default(),
            priority: 0,
            order_selectable: true,
            choice_actor: t.choice_actor,
            module: t.module,
            entry_pc: t.entry_pc,
        })
        .collect();

    let modifier_index: Vec<PackageModifierEntry> = modifiers
        .into_iter()
        .map(|m| PackageModifierEntry {
            skill_def_id: m.skill_def_id,
            target: m.target,
            selector: m.selector,
            mode: m.mode,
            value_source: m.value_source,
            value: m.value,
        })
        .collect();

    let active_index: Vec<PackageActiveEntry> = actives
        .into_iter()
        .map(|a| PackageActiveEntry {
            host_key: None,

            skill_def_id: a.skill_def_id,
            window: a.window,
            module: a.module,
            entry_pc: a.entry_pc,
            targets: a.targets.map(|targets| PackageTargetSpec {
                min: targets.min,
                max: targets.max,
                allow_self: targets.allow_self,
                lowest_hp: targets.lowest_hp,
                highest_hp: targets.highest_hp,
                unlinked: targets.unlinked,
                linked: targets.linked,
                max_by_owner_mark: targets.max_by_owner_mark,
                max_by_owner_lost_hp: targets.max_by_owner_lost_hp,
                max_by_owner_hp: targets.max_by_owner_hp,
                wounded: targets.wounded,
                male: targets.male,
                hp_at_least_owner: targets.hp_at_least_owner,
                hp_gt_owner: targets.hp_gt_owner,
                hp_lte_owner: targets.hp_lte_owner,
                hp_gt: targets.hp_gt,
                max_hp_gt: targets.max_hp_gt,
                hand_gt_owner: targets.hand_gt_owner,
                hand_lte_owner: targets.hand_lte_owner,
                pair_hand_difference_lte_three: targets.pair_hand_difference_lte_three,
                hand_lt: targets.hand_lt,
                hand_nonempty: targets.hand_nonempty,
                can_compare: targets.can_compare,
                can_use_on_owner: targets.can_use_on_owner,
                second_can_use_juedou_on_first: targets.second_can_use_juedou_on_first,
                equipment_pair_with_armor: targets.equipment_pair_with_armor,
                has_equip: targets.has_equip,
                has_discardable_equip: targets.has_discardable_equip,
                has_gainable_equip: targets.has_gainable_equip,
                has_gainable_hand_equip: targets.has_gainable_hand_equip,
                in_attack_range: targets.in_attack_range,
                owner_in_target_attack_range: targets.owner_in_target_attack_range,
                hand_equip_nonempty: targets.hand_equip_nonempty,
                has_discardable_hand_equip: targets.has_discardable_hand_equip,
                unused_by_skill_this_turn: targets.unused_by_skill_this_turn,
                unused_by_skill_this_play_phase: targets.unused_by_skill_this_play_phase,
                match_selected_card_count: targets.match_selected_card_count,
                selected_cards_match_target_equip_count: targets
                    .selected_cards_match_target_equip_count,
            }),
            uses_per_turn: a.uses_per_turn,
            uses_per_turn_source: a.uses_per_turn_source,
            uses_per_play_phase: a.uses_per_play_phase,
            mark_at_least: a.mark_at_least,
            owner_missing_skill: a.owner_missing_skill,
            hand_gt_hp: a.hand_gt_hp,
            hand_gt_max_hp: a.hand_gt_max_hp,
            hand_limit_positive: a.hand_limit_positive,
            equip_nonempty: a.equip_nonempty,
            hand_nonempty: a.hand_nonempty,
            hand_missing_gaintag: a.hand_missing_gaintag,
            face_up: a.face_up,
            owner_unlinked: a.owner_unlinked,
            once_per_game: a.once_per_game,
            selection: a.selection.map(|selection| PackageSelectionSpec {
                zone: selection.zone,
                min: selection.min,
                max: selection.max,
                allow_cancel: selection.allow_cancel,
                same_suit: selection.same_suit,
                same_color: selection.same_color,
                distinct_suits: selection.distinct_suits,
                distinct_categories: selection.distinct_categories,
                filter: selection.filter,
                unused_category_by_skill_this_turn: selection.unused_category_by_skill_this_turn,
                unused_category_by_skill_this_play_phase: selection
                    .unused_category_by_skill_this_play_phase,
                unused_category_by_owner_history_this_turn: selection
                    .unused_category_by_owner_history_this_turn,
                sum_effective_number_exact: selection.sum_effective_number_exact,
            }),
        })
        .collect();

    let view_as_index: Vec<PackageViewAsEntry> = view_as
        .into_iter()
        .map(|v| PackageViewAsEntry {
            skill_def_id: v.skill_def_id,
            as_kind: v.as_kind,
            window: v.window,
            filter: v.filter,
            source_zone: v.source_zone,
            uses_per_turn: v.uses_per_turn,
            ignore_distance: v.ignore_distance,
            red_unrespondable: v.red_unrespondable,
            black_no_use_count: v.black_no_use_count,
            no_use_count: v.no_use_count,
            target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
            refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
            min_cards: v.min_cards,
            max_cards: v.max_cards,
            owner_hp_min_one_cards: v.owner_hp_min_one_cards,
            owner_basic_uses_plus_one_cards: v.owner_basic_uses_plus_one_cards,
            once_per_turn_per_branch: v.once_per_turn_per_branch,
            same_suit: v.same_suit,
            nature: v.nature,
            multi_card_bonus: v.multi_card_bonus,
            branch: v.branch,
            once_per_round: v.once_per_round,
            next_branch: v.next_branch,
            cost: v.cost,
            mark_cost: v.mark_cost,
            post_draw: v.post_draw,
        })
        .collect();

    let skill_index = ir
        .skills
        .iter()
        .map(|skill| PackageSkillEntry {
            charlotte: skill.charlotte,
            persevere: skill.persevere,
            forever: skill.forever,
            temporary: skill.temporary,
            group_children: None,
            reveal: None,
            onremove: None,

            voice_assets: Default::default(),
            skill_def_id: skill.def_id,
            name: skill.name.clone(),
            locked: skill.locked,
            lord_only: skill.lord_only,
            non_gainable: skill.non_gainable,
            conversion: skill.conversion,
        })
        .collect();

    let mut pkg = ContentPackage {
        manifest,
        modules,
        skill_index,
        character_index: Vec::new(),
        skin_index: Default::default(),
        card_index: Vec::new(),
        trigger_index,
        modifier_index,
        active_index,
        view_as_index,
        translations: Default::default(),
        asset_manifest: None,
        asset_paths: Default::default(),
        compiler_version: COMPILER_VERSION.into(),
    };
    pkg.manifest.content_schema_version = pkg.required_content_schema();
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    let _ = maps; // source maps available for debugger tooling
    Ok(pkg)
}

#[cfg(test)]
mod tests;
