//! Loaded content package registry (bytecode + trigger + modifier catalog overlay).
//!
//! Package skill_def ids that match bootstrap defs **override** the native handler
//! (triggers → `RunVm`, modifiers → package static defs). Empty registry → bootstrap.

use crate::query::{
    bootstrap_modifier_defs, CombineMode, ModifierDefinition, ModifierTarget, ModifierValueSource,
};
use crate::trigger::{
    bootstrap_trigger_defs, EventKind, TimingBand, TriggerDefId, TriggerDefinition, TriggerFilter,
    TriggerHandler, BOOTSTRAP_JIANXIONG,
};
use content_schema::{
    ContentPackage, PackageCardEntry, PackageCharacterEntry, PackageSelectionSpec,
};
use engine_api::SkillDefId;
use rule_vm::{BytecodeModule, VerifiedModule};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Base for package-assigned trigger definition ids (avoid clashing with bootstrap 1..99).
pub const PACKAGE_TRIGGER_ID_BASE: u32 = 10_000;
/// Base for package-assigned modifier definition ids.
pub const PACKAGE_MODIFIER_ID_BASE: u32 = 10_000;

/// Content load error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContentLoadError {
    /// Invalid package data.
    #[error("invalid package: {0}")]
    Invalid(String),
    /// Module verify failed.
    #[error("verify: {0}")]
    Verify(String),
    /// Duplicate module name with different hash.
    #[error("module conflict: {0}")]
    Conflict(String),
}

/// Loaded active skill definition (from package).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSkillDef {
    /// Skill definition id.
    pub skill_def: SkillDefId,
    /// Activation window: play_phase | any
    pub window: String,
    /// Bytecode module name.
    pub module: String,
    /// Entry PC.
    pub entry_pc: u32,
    /// Optional player-target count contract.
    pub targets: Option<content_schema::PackageTargetSpec>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Optional authoritative activation limit scoped to one Play phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_play_phase: Option<u8>,
    /// Optional dynamic per-turn activation limit source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn_source: Option<String>,
    /// Optional owner-mark threshold required before activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_at_least: Option<(u32, u32)>,
    /// Require the owner not to have this skill definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_missing_skill: Option<u32>,
    /// Require the skill owner's hand count to exceed current HP.
    #[serde(default)]
    pub hand_gt_hp: bool,
    /// Require the skill owner's hand count to exceed maximum HP.
    #[serde(default)]
    pub hand_gt_max_hp: bool,
    /// Require the skill owner's current hand limit to be positive.
    #[serde(default)]
    pub hand_limit_positive: bool,
    /// Require at least one card in the skill owner's equipment zone.
    #[serde(default)]
    pub equip_nonempty: bool,
    /// Require at least one card in the skill owner's hand.
    #[serde(default)]
    pub hand_nonempty: bool,
    /// Require the skill owner's hand to hold no card carrying this gaintag.
    pub hand_missing_gaintag: Option<u32>,
    /// Require the skill owner to be face up.
    #[serde(default)]
    pub face_up: bool,
    /// Require the owner not to be linked.
    #[serde(default)]
    pub owner_unlinked: bool,
    /// Allow this active skill to resolve at most once during the match.
    #[serde(default)]
    pub once_per_game: bool,
    /// Optional card selection contract.
    pub selection: Option<PackageSelectionSpec>,
}

/// Loaded view-as definition (from package).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewAsDef {
    /// Skill definition id.
    pub skill_def: SkillDefId,
    /// Treated as a supported basic/trick card kind, such as sha, jiu, or huogong.
    pub as_kind: String,
    /// Window: play_phase | any
    pub window: String,
    /// Filter: red|black|any|card_*|suit names
    pub filter: String,
    /// Optional source zone; omitted means hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone: Option<String>,
    /// Optional authoritative per-turn activation limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn: Option<u8>,
    /// Whether ordinary card distance checks are ignored.
    #[serde(default)]
    pub ignore_distance: bool,
    /// Whether a red source makes the converted Sha unrespondable.
    #[serde(default)]
    pub red_unrespondable: bool,
    /// Whether a black source avoids consuming the ordinary Sha use count.
    #[serde(default)]
    pub black_no_use_count: bool,
    /// Whether this converted Sha always avoids consuming the ordinary Sha use count.
    #[serde(default)]
    pub no_use_count: bool,
    /// Make each target lacking a hand card of the source suit unable to respond.
    #[serde(default)]
    pub target_lacks_source_suit_unrespondable: bool,
    /// Refund the ordinary Sha use count if this exact converted Sha deals no damage.
    #[serde(default)]
    pub refund_use_count_if_no_damage: bool,
    /// Minimum number of physical source cards.
    #[serde(default = "default_view_as_card_count")]
    pub min_cards: u8,
    /// Maximum number of physical source cards.
    pub max_cards: u8,
    /// Require exactly max(1, current HP) physical source cards.
    #[serde(default)]
    pub owner_hp_min_one_cards: bool,
    /// Minimum physical source cards = 1 + lifetime basic-card uses (max uncapped).
    pub owner_basic_uses_plus_one_cards: bool,
    /// Each rule of this skill usable once per turn (per-rule bitmap).
    pub once_per_turn_per_branch: bool,
    /// Require equal effective suits for all selected source cards.
    pub same_suit: bool,
    /// Optional logical card nature.
    pub nature: Option<String>,
    /// Optional effect applied when multiple source cards are selected.
    pub multi_card_bonus: Option<String>,
    /// Conversion branch required for a source-free family rule.
    pub branch: Option<u8>,
    /// Whether this branch may be used only once in each round.
    pub once_per_round: bool,
    /// Branch selected after a successful conversion.
    pub next_branch: Option<u8>,
    /// Declarative source-free conversion cost.
    pub cost: Option<String>,
    /// Skill-definition mark consumed by the conversion cost.
    pub mark_cost: Option<u32>,
    /// Cards drawn after the converted card is used or responded with.
    pub post_draw: u8,
}

/// Metadata for a loaded package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedPackageMeta {
    /// Namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Version.
    pub version: String,
    /// Logic hash hex.
    pub logic_hash: [u8; 32],
}

/// Loaded skill-level metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill definition id.
    pub skill_def: SkillDefId,
    /// Stable source name.
    pub name: String,
    /// Locked/passive marker.
    pub locked: bool,
    /// Only active while the owner has the Lord identity.
    #[serde(default)]
    pub lord_only: bool,
    /// Cannot be copied or gained by another skill.
    #[serde(default)]
    pub non_gainable: bool,
    /// Conversion skill whose branch can be toggled by generic rules.
    #[serde(default)]
    pub conversion: bool,
}

/// Match-scoped content registry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRegistry {
    /// Module name → bytecode（PA06/B1：装载期校验一次并持有已校验模块，
    /// 执行期不再重复 `verify_module`；序列化格式与裸 `BytecodeModule` 相同）。
    modules: BTreeMap<String, VerifiedModule>,
    /// Skill definition metadata from packages.
    #[serde(default)]
    package_skills: BTreeMap<u32, SkillMetadata>,
    /// Reverse lookup: skill name → definition id (Phase 3 AddSkillByName resolution).
    #[serde(default)]
    skill_def_by_name: BTreeMap<String, u32>,
    /// Character definitions from packages.
    #[serde(default)]
    package_characters: BTreeMap<u32, PackageCharacterEntry>,
    /// Card definitions from packages.
    #[serde(default)]
    package_cards: BTreeMap<u32, PackageCardEntry>,
    /// Triggers from packages (override bootstrap when skill_def matches).
    package_triggers: Vec<TriggerDefinition>,
    /// Modifiers from packages (override bootstrap when skill_def matches).
    package_modifiers: Vec<ModifierDefinition>,
    /// Active skills from packages.
    package_actives: Vec<ActiveSkillDef>,
    /// View-as rules from packages.
    package_view_as: Vec<ViewAsDef>,
    /// Cached capability: loaded triggers need turn-wide card-use history.
    #[serde(default)]
    track_turn_card_history: bool,
    /// Cached capability: loaded triggers compare consecutive logical card names.
    #[serde(default)]
    track_turn_card_name_history: bool,
    /// Cached capability: loaded triggers need play-phase card-use history.
    #[serde(default)]
    track_play_card_history: bool,
    /// Cached capability: loaded triggers need per-player turn card-use counts.
    #[serde(default)]
    track_per_player_turn_card_count: bool,
    /// Cached capability: loaded triggers need per-player play-phase card-use counts.
    #[serde(default)]
    track_per_player_play_phase_card_count: bool,
    track_play_phase_card_suit_history: bool,
    track_discard_pile_suit_history: bool,
    /// Cached capability: loaded triggers need per-player turn card-use plus response counts.
    #[serde(default)]
    track_per_player_turn_card_action_count: bool,
    /// Cached capability: loaded triggers need match-lifetime card-use plus response counts.
    #[serde(default)]
    track_lifetime_card_action_count: bool,
    /// Cached capability: loaded triggers need per-player turn logical-Sha use counts.
    #[serde(default)]
    track_per_player_turn_sha_use_count: bool,
    /// Cached capability: loaded triggers need the turn-wide physical-card discard count caused
    /// by card use and responses.
    #[serde(default)]
    track_turn_use_response_discard_count: bool,
    /// Cached capability: loaded triggers need the current discard-phase discard count.
    #[serde(default)]
    track_discard_phase_discard_count: bool,
    /// ABI 513: record the discard-phase lost-card slice base (固政).
    track_discard_phase_lost_cards: bool,
    /// ABI 583: record all physical cards discarded during the current turn.
    track_turn_discarded_cards: bool,
    /// ABI 601: record all cards entering discard and materials of card uses that dealt damage.
    track_turn_haoyi_cards: bool,
    /// ABI 634: retain ordered eligible physical uses for 管宁「承经」.
    track_ddd_chengjing_history: bool,
    /// Cached capability: loaded triggers need each player's game-wide previous card.
    #[serde(default)]
    track_game_card_history: bool,
    /// Cached capability: loaded triggers need per-target red-card history for the current turn.
    #[serde(default)]
    track_turn_red_card_target_history: bool,
    /// Cached capability: loaded filters need per-user/per-target card-use counts this turn.
    #[serde(default)]
    track_turn_card_target_counts: bool,
    /// Cached capability: ABI-42 content needs hand-origin suit history.
    #[serde(default)]
    track_turn_hand_card_suit_history: bool,
    /// Cached capability: ABI-232 content needs effective suits of every used card this turn.
    #[serde(default)]
    track_turn_used_card_suit_history: bool,
    /// Cached capability: loaded effects need distinct logical basic-card variants this turn.
    #[serde(default)]
    track_turn_basic_card_variant_history: bool,
    /// Cached capability: ABI-42 content needs logical play-phase category history.
    #[serde(default)]
    track_play_card_category_history: bool,
    /// Cached capability: loaded filters need logical Play-phase category history for card
    /// users other than the current player.
    #[serde(default)]
    track_play_card_category_history_all_users: bool,
    /// Cached capability: loaded filters need per-player Play-phase gain history.
    #[serde(default)]
    track_play_phase_card_gain_history: bool,
    track_owner_card_category_history: bool,
    /// Cached capability: at least one loaded package opts into per-player damage-taken counting.
    track_damage_taken_count: bool,
    /// ADR 0644 P3 (ABI 659): content uses align_debate_opinions → record the
    /// per-source damaged-players seat bitset.
    track_damaged_by_player_bitset: bool,
    /// Cached capability: loaded filters need source-target damage counts for the current turn.
    #[serde(default)]
    track_turn_damage_pair_counts: bool,
    /// Cached capability: loaded content needs any-player entered-dying history this turn.
    #[serde(default)]
    track_entered_dying_history: bool,
    /// Cached capability: loaded filters need a bounded duplicate-category summary for the
    /// current Play phase.
    #[serde(default)]
    track_play_card_category_duplicates: bool,
    /// Cached capability: ABI-43 content needs counted/refunded provenance in saved Sha uses.
    #[serde(default)]
    track_sha_use_refund_provenance: bool,
    /// Cached capability: ABI-44 content needs an explicit recoverable turn-opening phase queue.
    #[serde(default)]
    track_opening_extra_play_lifecycle: bool,
    /// Cached capability: ABI-98 content needs bounded card-target and HP-change histories for
    /// each concrete phase.
    #[serde(default)]
    track_phase_target_hp_history: bool,
    /// Cached capability: loaded filters need per-player actual HP-change counts this turn.
    #[serde(default)]
    track_turn_hp_change_history: bool,
    /// Cached capability: loaded filters need per-player turn card-gain counts.
    #[serde(default)]
    track_turn_card_gain_count: bool,
    /// Cached capability: loaded filters need per-player turn card-loss provenance.
    #[serde(default)]
    track_turn_card_loss_history: bool,
    /// Cached capability: loaded filters need per-player turn card-target provenance.
    #[serde(default)]
    track_card_target_history: bool,
    /// Cached capability: loaded view_as rules need lifetime basic-use counts.
    #[serde(default)]
    track_basic_use_history: bool,
    /// Cached capability: loaded triggers need lifetime used-type bitmap.
    #[serde(default)]
    track_used_type_groups: bool,
    /// Loaded package metas (order of load).
    packages: Vec<LoadedPackageMeta>,
    /// Next free package trigger id.
    next_trigger_id: u32,
    /// Next free package modifier id.
    next_modifier_id: u32,
    /// Combined content logic hash (zeros if empty).
    pub combined_logic_hash: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trigger::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    use content_schema::{PackageCharacterEntry, PackageDependency, TriggerChoiceActor};

    #[test]
    fn empty_falls_back_to_bootstrap() {
        let reg = ContentRegistry::new();
        let defs = reg.defs_for_skill(SkillDefId::new(BOOTSTRAP_JIANXIONG));
        assert_eq!(defs.len(), 1);
        assert!(matches!(
            defs[0].handler,
            TriggerHandler::DrawSelf { count: 1 }
        ));
        let mods = reg.modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_MASHU));
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].value, -1);
        let q = reg.modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_QICAO));
        assert_eq!(q[0].value, 1);
    }

    #[test]
    fn rejects_character_with_unavailable_skill_before_mutating_registry() {
        let mut package =
            content_compiler::compile_source_named("closure", "skill present(1) {}").unwrap();
        package.character_index.push(PackageCharacterEntry {
            id: 1,
            name_key: "incomplete".into(),
            max_hp: 4,
            initial_hp: 0,
            skills: vec![2],
            sex: "male".into(),
            group: "wei".into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        });
        package.manifest.logic_hash = package.compute_logic_hash();
        let mut registry = ContentRegistry::new();
        assert!(matches!(
            registry.load_package(&package),
            Err(ContentLoadError::Invalid(message))
                if message == "character 1 references unavailable skill 2"
        ));
        assert_eq!(registry.package_count(), 0);
        assert!(registry.skill_metadata(SkillDefId::new(1)).is_none());
    }

    #[test]
    fn rejects_new_contract_violations_without_mutating_registry() {
        let package = content_compiler::compile_source_named(
            "contracts",
            "skill guarded(1) {
                active play_phase uses_per_play_phase 1 select_hand 1 1 sum_effective_number_exact 13 {
                    effect draw_cards(self, 1);
                }
                modifier card_number override 13;
            }",
        )
        .unwrap();
        let mut invalid_packages = Vec::new();

        let mut invalid_number = package.clone();
        invalid_number.modifier_index[0].mode = "add".into();
        invalid_number.manifest.logic_hash = invalid_number.compute_logic_hash();
        invalid_packages.push(invalid_number);

        let mut invalid_scope = package.clone();
        invalid_scope.active_index[0].window = "any".into();
        invalid_scope.manifest.logic_hash = invalid_scope.compute_logic_hash();
        invalid_packages.push(invalid_scope);

        let mut invalid_sum = package.clone();
        invalid_sum.active_index[0]
            .selection
            .as_mut()
            .unwrap()
            .sum_effective_number_exact = Some(0);
        invalid_sum.manifest.logic_hash = invalid_sum.compute_logic_hash();
        invalid_packages.push(invalid_sum);

        let mut invalid_abi = package.clone();
        invalid_abi.manifest.rule_vm_abi_version = 300;
        invalid_abi.manifest.logic_hash = invalid_abi.compute_logic_hash();
        invalid_packages.push(invalid_abi);

        let dingpan_package = content_compiler::compile_source_named(
            "dingpan-contract",
            "skill dingpan(3739676314) {
                active play_phase uses_per_turn living_rebel_count targets 1 1 has_equip {
                    effect resolve_dingpan(self, event_source);
                }
            }",
        )
        .unwrap();
        let mut invalid_dynamic_abi = dingpan_package.clone();
        invalid_dynamic_abi.active_index[0].uses_per_turn_source = None;
        invalid_dynamic_abi.manifest.rule_vm_abi_version = 301;
        invalid_dynamic_abi.manifest.logic_hash = invalid_dynamic_abi.compute_logic_hash();
        invalid_packages.push(invalid_dynamic_abi);

        let mut invalid_dingpan_window = dingpan_package.clone();
        invalid_dingpan_window.active_index[0].window = "any".into();
        invalid_dingpan_window.manifest.logic_hash = invalid_dingpan_window.compute_logic_hash();
        invalid_packages.push(invalid_dingpan_window);

        let mut invalid_dingpan_limit = dingpan_package.clone();
        invalid_dingpan_limit.active_index[0].uses_per_turn_source = Some("owner_hp".into());
        invalid_dingpan_limit.manifest.logic_hash = invalid_dingpan_limit.compute_logic_hash();
        invalid_packages.push(invalid_dingpan_limit);

        let mut invalid_dingpan_target = dingpan_package.clone();
        invalid_dingpan_target.active_index[0]
            .targets
            .as_mut()
            .unwrap()
            .has_equip = false;
        invalid_dingpan_target.manifest.logic_hash = invalid_dingpan_target.compute_logic_hash();
        invalid_packages.push(invalid_dingpan_target);

        let mut invalid_dingpan_orphan = dingpan_package;
        invalid_dingpan_orphan.active_index.clear();
        invalid_dingpan_orphan.manifest.logic_hash = invalid_dingpan_orphan.compute_logic_hash();
        invalid_packages.push(invalid_dingpan_orphan);

        let tanlu_package = content_compiler::compile_source_named(
            "tanlu-contract",
            "skill xktanlu(4005608784) {
                trigger phase_begin owner_alive_not_target optional {
                    effect resolve_tanlu(self, event_player);
                }
            }",
        )
        .unwrap();
        let mut invalid_tanlu_abi = tanlu_package.clone();
        invalid_tanlu_abi.manifest.rule_vm_abi_version = 302;
        invalid_tanlu_abi.manifest.logic_hash = invalid_tanlu_abi.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_abi);

        let mut invalid_tanlu_event = tanlu_package.clone();
        invalid_tanlu_event.trigger_index[0].event = "phase_prepare".into();
        invalid_tanlu_event.manifest.logic_hash = invalid_tanlu_event.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_event);

        let mut invalid_tanlu_filter = tanlu_package.clone();
        invalid_tanlu_filter.trigger_index[0].filter = "owner_alive".into();
        invalid_tanlu_filter.manifest.logic_hash = invalid_tanlu_filter.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_filter);

        let mut invalid_tanlu_optional = tanlu_package.clone();
        invalid_tanlu_optional.trigger_index[0].optional = false;
        invalid_tanlu_optional.manifest.logic_hash = invalid_tanlu_optional.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_optional);

        let mut invalid_tanlu_actor = tanlu_package.clone();
        invalid_tanlu_actor.trigger_index[0].choice_actor = TriggerChoiceActor::EventPlayer;
        invalid_tanlu_actor.manifest.logic_hash = invalid_tanlu_actor.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_actor);

        let mut invalid_tanlu_orphan = tanlu_package;
        invalid_tanlu_orphan.trigger_index.clear();
        invalid_tanlu_orphan.manifest.logic_hash = invalid_tanlu_orphan.compute_logic_hash();
        invalid_packages.push(invalid_tanlu_orphan);

        let same_suit_package = content_compiler::compile_source_named(
            "same-suit-contract",
            "skill same_suit(2) {
                active play_phase select_he 3 3 same_suit {
                    effect discard_selected(self);
                }
            }",
        )
        .unwrap();
        let mut invalid_same_suit_abi = same_suit_package;
        invalid_same_suit_abi.manifest.rule_vm_abi_version = 303;
        invalid_same_suit_abi.manifest.logic_hash = invalid_same_suit_abi.compute_logic_hash();
        invalid_packages.push(invalid_same_suit_abi);

        let jdlianhuan_package = content_compiler::compile_source_named(
            "jdlianhuan-contract",
            "skill jdlianhuan(3773194665) {
                trigger card_targeted owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty {
                    effect choose_optional_discard_target_card(self, event_player, hand_equip);
                }
            }",
        )
        .unwrap();
        let mut invalid_jdlianhuan_abi = jdlianhuan_package;
        invalid_jdlianhuan_abi.manifest.rule_vm_abi_version = 304;
        invalid_jdlianhuan_abi.manifest.logic_hash = invalid_jdlianhuan_abi.compute_logic_hash();
        invalid_packages.push(invalid_jdlianhuan_abi);

        let jyliezhou_package = content_compiler::compile_source_named(
            "jyliezhou-contract",
            "skill jyliezhou(3307262280) locked {
                trigger damage owner_alive_source_target_was_linked {
                    effect draw_cards(self, event_amount);
                }
            }",
        )
        .unwrap();
        let mut invalid_jyliezhou_abi = jyliezhou_package;
        invalid_jyliezhou_abi.manifest.rule_vm_abi_version = 305;
        invalid_jyliezhou_abi.manifest.logic_hash = invalid_jyliezhou_abi.compute_logic_hash();
        invalid_packages.push(invalid_jyliezhou_abi);

        for invalid in invalid_packages {
            let mut registry = ContentRegistry::new();
            let before = registry.clone();
            assert!(matches!(
                registry.load_package(&invalid),
                Err(ContentLoadError::Invalid(_))
            ));
            assert_eq!(registry, before);
        }
    }

    #[test]
    fn rejects_tampered_idempotent_package_before_hash_shortcut() {
        let package = content_compiler::compile_source_named(
            "idempotent",
            "skill guarded(1) { modifier card_number override 13; }",
        )
        .unwrap();
        let mut registry = ContentRegistry::new();
        registry.load_package(&package).unwrap();
        let before = registry.clone();

        let mut tampered = package;
        tampered.modifier_index[0].value = 12;
        assert!(matches!(
            registry.load_package(&tampered),
            Err(ContentLoadError::Invalid(_))
        ));
        assert_eq!(registry, before);
    }

    #[test]
    fn defers_dependency_character_closure_until_full_set_is_loaded() {
        let mut package =
            content_compiler::compile_source_named("closure", "skill present(1) {}").unwrap();
        package.manifest.dependencies.push(PackageDependency {
            namespace: "official".into(),
            package_id: "dependency".into(),
            version: "0.1.0".into(),
            logic_hash: [0; 32],
        });
        package.character_index.push(PackageCharacterEntry {
            id: 1,
            name_key: "dependent".into(),
            max_hp: 4,
            initial_hp: 0,
            skills: vec![2],
            sex: "male".into(),
            group: "wei".into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        });
        package.manifest.logic_hash = package.compute_logic_hash();
        let mut registry = ContentRegistry::new();
        registry.load_package(&package).unwrap();
        assert!(matches!(
            registry.validate_character_skill_closure(),
            Err(ContentLoadError::Invalid(message))
                if message == "character 1 references unavailable skill 2"
        ));
    }
}

mod default_view_as_card_count;
pub use default_view_as_card_count::*;
mod map_filter;
pub(crate) use map_filter::*;
mod valid_group_filter_tag;
pub use valid_group_filter_tag::*;

mod new;

mod load_package_inner;
