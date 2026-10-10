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
use rule_vm::BytecodeModule;
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

fn default_view_as_card_count() -> u8 {
    1
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
    /// Module name → bytecode.
    modules: BTreeMap<String, BytecodeModule>,
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

impl ContentRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            package_skills: BTreeMap::new(),
            skill_def_by_name: BTreeMap::new(),
            package_characters: BTreeMap::new(),
            package_cards: BTreeMap::new(),
            package_triggers: Vec::new(),
            package_modifiers: Vec::new(),
            package_actives: Vec::new(),
            package_view_as: Vec::new(),
            track_turn_card_history: false,
            track_turn_card_name_history: false,
            track_play_card_history: false,
            track_per_player_turn_card_count: false,
            track_per_player_play_phase_card_count: false,
            track_play_phase_card_suit_history: false,
            track_discard_pile_suit_history: false,
            track_per_player_turn_card_action_count: false,
            track_lifetime_card_action_count: false,
            track_per_player_turn_sha_use_count: false,
            track_turn_use_response_discard_count: false,
            track_discard_phase_discard_count: false,
            track_discard_phase_lost_cards: false,
            track_turn_discarded_cards: false,
            track_turn_haoyi_cards: false,
            track_ddd_chengjing_history: false,
            track_game_card_history: false,
            track_turn_red_card_target_history: false,
            track_turn_card_target_counts: false,
            track_turn_hand_card_suit_history: false,
            track_turn_used_card_suit_history: false,
            track_turn_basic_card_variant_history: false,
            track_play_card_category_history: false,
            track_play_card_category_history_all_users: false,
            track_play_phase_card_gain_history: false,
            track_owner_card_category_history: false,
            track_damage_taken_count: false,
            track_damaged_by_player_bitset: false,
            track_turn_damage_pair_counts: false,
            track_entered_dying_history: false,
            track_play_card_category_duplicates: false,
            track_sha_use_refund_provenance: false,
            track_opening_extra_play_lifecycle: false,
            track_phase_target_hp_history: false,
            track_turn_hp_change_history: false,
            track_turn_card_gain_count: false,
            track_turn_card_loss_history: false,
            track_card_target_history: false,
            track_basic_use_history: false,
            track_used_type_groups: false,
            packages: Vec::new(),
            next_trigger_id: PACKAGE_TRIGGER_ID_BASE,
            next_modifier_id: PACKAGE_MODIFIER_ID_BASE,
            combined_logic_hash: [0u8; 32],
        }
    }

    /// Loaded package count.
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Module lookup.
    pub fn module(&self, name: &str) -> Option<&BytecodeModule> {
        self.modules.get(name).or_else(|| {
            let mut matches = self.modules.values().filter(|module| module.name == name);
            let first = matches.next();
            if matches.next().is_none() {
                first
            } else {
                None
            }
        })
    }

    /// Package metas.
    pub fn packages(&self) -> &[LoadedPackageMeta] {
        &self.packages
    }

    /// Skill metadata, if declared by a loaded package.
    pub fn skill_metadata(&self, skill_def: SkillDefId) -> Option<&SkillMetadata> {
        self.package_skills.get(&skill_def.raw())
    }

    /// Resolve a published skill name to its definition id (Phase 3 AddSkillByName).
    pub fn skill_def_by_name(&self, name: &str) -> Option<SkillDefId> {
        self.skill_def_by_name.get(name).map(|def| SkillDefId::new(*def))
    }

    /// All loaded skill metadata in stable definition-id order.
    pub fn skills(&self) -> impl Iterator<Item = &SkillMetadata> {
        self.package_skills.values()
    }

    /// Loaded immutable card metadata.
    pub fn card_metadata(&self, card_def: engine_api::CardDefId) -> Option<&PackageCardEntry> {
        self.package_cards.get(&card_def.raw())
    }

    /// Character definition declared by a loaded package.
    pub fn character(&self, character_def: u32) -> Option<&PackageCharacterEntry> {
        self.package_characters.get(&character_def)
    }

    /// All loaded character definitions in stable id order.
    pub fn characters(&self) -> impl Iterator<Item = &PackageCharacterEntry> {
        self.package_characters.values()
    }

    /// Resolve trigger definitions for a skill (package overrides bootstrap).
    pub fn defs_for_skill(&self, skill_def: SkillDefId) -> Vec<TriggerDefinition> {
        let pkg: Vec<_> = self
            .package_triggers
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect();
        if !pkg.is_empty() {
            return pkg;
        }
        bootstrap_trigger_defs()
            .into_iter()
            .filter(|d| d.skill_def == skill_def)
            .collect()
    }

    /// Find a trigger def by id (package then bootstrap).
    pub fn find_def(&self, id: TriggerDefId) -> Option<TriggerDefinition> {
        self.package_triggers
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .or_else(|| bootstrap_trigger_defs().into_iter().find(|d| d.id == id))
    }

    /// Whether loaded content needs the current player's turn-wide card-use history.
    pub fn tracks_turn_card_history(&self) -> bool {
        self.track_turn_card_history
    }

    /// Whether loaded content compares consecutive logical card names this turn.
    pub fn tracks_turn_card_name_history(&self) -> bool {
        self.track_turn_card_name_history
    }

    /// Whether loaded content needs card-use history scoped to the current play phase.
    pub fn tracks_play_card_history(&self) -> bool {
        self.track_play_card_history
    }

    /// Whether loaded content needs a separate turn card-use count for each player.
    pub fn tracks_per_player_turn_card_count(&self) -> bool {
        self.track_per_player_turn_card_count
    }

    /// Whether loaded content needs a card-use count scoped to each play phase.
    pub fn tracks_per_player_play_phase_card_count(&self) -> bool {
        self.track_per_player_play_phase_card_count
    }

    /// Whether loaded content needs effective used-card suits for the current Play phase.
    pub fn tracks_play_phase_card_suit_history(&self) -> bool {
        self.track_play_phase_card_suit_history
    }

    /// Whether loaded content needs physical suits entering the discard pile this turn.
    pub fn tracks_discard_pile_suit_history(&self) -> bool {
        self.track_discard_pile_suit_history
    }

    /// Whether loaded content needs a turn used-or-responded card count for each player.
    pub fn tracks_per_player_turn_card_action_count(&self) -> bool {
        self.track_per_player_turn_card_action_count
    }

    /// Whether loaded content needs match-lifetime used-or-responded card counts per player.
    pub fn tracks_lifetime_card_action_count(&self) -> bool {
        self.track_lifetime_card_action_count
    }

    /// Whether loaded content needs a turn logical-Sha use count for each player.
    pub fn tracks_per_player_turn_sha_use_count(&self) -> bool {
        self.track_per_player_turn_sha_use_count
    }

    /// Whether loaded content needs the turn-wide physical-card discard count caused by card
    /// use and responses.
    pub fn tracks_turn_use_response_discard_count(&self) -> bool {
        self.track_turn_use_response_discard_count
    }

    /// Whether loaded content needs the current discard-phase discard count.
    pub fn tracks_discard_phase_discard_count(&self) -> bool {
        self.track_discard_phase_discard_count
    }

    /// Whether loaded content opts into discard-phase lost-card slice tracking (ABI 513).
    pub fn tracks_discard_phase_lost_cards(&self) -> bool {
        self.track_discard_phase_lost_cards
    }

    /// Whether loaded content needs all discarded physical cards for the current turn.
    pub fn tracks_turn_discarded_cards(&self) -> bool {
        self.track_turn_discarded_cards
    }

    /// Whether loaded content needs 豪义 turn-card tracking (ABI 601).
    pub fn tracks_turn_haoyi_cards(&self) -> bool {
        self.track_turn_haoyi_cards
    }

    /// Whether loaded content needs 管宁「承经」ordered use history.
    pub fn tracks_ddd_chengjing_history(&self) -> bool {
        self.track_ddd_chengjing_history
    }

    /// Whether loaded content needs each player's previous card across turn boundaries.
    pub fn tracks_game_card_history(&self) -> bool {
        self.track_game_card_history
    }

    /// Whether loaded content needs per-target distinct-red-card history for the current turn.
    pub fn tracks_turn_red_card_target_history(&self) -> bool {
        self.track_turn_red_card_target_history
    }

    /// Whether loaded content needs per-user/per-target card-use counts for the current turn.
    pub fn tracks_turn_card_target_counts(&self) -> bool {
        self.track_turn_card_target_counts
    }

    /// Whether loaded content needs distinct effective suits used from physical hand cards.
    pub fn tracks_turn_hand_card_suit_history(&self) -> bool {
        self.track_turn_hand_card_suit_history
    }

    /// Whether loaded content needs distinct effective suits among all cards used this turn.
    pub fn tracks_turn_used_card_suit_history(&self) -> bool {
        self.track_turn_used_card_suit_history
    }

    /// Whether loaded content needs distinct logical basic-card variants used this turn.
    pub fn tracks_turn_basic_card_variant_history(&self) -> bool {
        self.track_turn_basic_card_variant_history
    }

    /// Whether loaded content needs distinct logical categories used this play phase.
    pub fn tracks_play_card_category_history(&self) -> bool {
        self.track_play_card_category_history
    }

    /// Whether loaded content needs category history for every user in the current Play phase.
    pub fn tracks_play_card_category_history_all_users(&self) -> bool {
        self.track_play_card_category_history_all_users
    }

    /// Whether loaded content needs per-player Play-phase gain history.
    pub fn tracks_play_phase_card_gain_history(&self) -> bool {
        self.track_play_phase_card_gain_history
    }

    /// Whether loaded content needs categories each player used or discarded this turn.
    pub fn tracks_owner_card_category_history(&self) -> bool {
        self.track_owner_card_category_history
    }

    /// Whether loaded content needs per-player damage-taken counts this turn.
    pub fn tracks_damage_taken_count(&self) -> bool {
        self.track_damage_taken_count
    }

    /// Whether loaded content needs the per-source damaged-players bitset
    /// (ADR 0644 P3 霸政 opinion alignment).
    pub fn tracks_damaged_by_player_bitset(&self) -> bool {
        self.track_damaged_by_player_bitset
    }

    /// Whether loaded content needs source-target damage counts for the current turn.
    pub fn tracks_turn_damage_pair_counts(&self) -> bool {
        self.track_turn_damage_pair_counts
    }

    /// Whether loaded content needs any-player entered-dying history for the current turn.
    pub fn tracks_entered_dying_history(&self) -> bool {
        self.track_entered_dying_history
    }

    /// Whether loaded content needs to distinguish repeated Play-phase card categories.
    pub fn tracks_play_card_category_duplicates(&self) -> bool {
        self.track_play_card_category_duplicates
    }

    /// Whether saved proactive Sha continuations must retain count-refund provenance.
    pub fn tracks_sha_use_refund_provenance(&self) -> bool {
        self.track_sha_use_refund_provenance
    }

    /// Whether turn starts use the recoverable opening-phase queue.
    pub fn tracks_opening_extra_play_lifecycle(&self) -> bool {
        self.track_opening_extra_play_lifecycle
    }

    /// Whether loaded content needs phase-scoped target and actual-HP-change history.
    pub fn tracks_phase_target_hp_history(&self) -> bool {
        self.track_phase_target_hp_history
    }

    /// Whether loaded content needs per-player turn-scoped actual HP-change counts.
    pub fn tracks_turn_hp_change_history(&self) -> bool {
        self.track_turn_hp_change_history
    }

    /// Whether loaded content needs per-player turn card-gain counts.
    pub fn tracks_turn_card_gain_count(&self) -> bool {
        self.track_turn_card_gain_count
    }

    /// Whether loaded content needs per-player turn card-loss provenance.
    pub fn tracks_turn_card_loss_history(&self) -> bool {
        self.track_turn_card_loss_history
    }

    /// Whether loaded content needs per-player turn card-target counts.
    pub fn tracks_card_target_history(&self) -> bool {
        self.track_card_target_history
    }

    /// Whether loaded content needs lifetime basic-card use counts.
    pub fn tracks_basic_use_history(&self) -> bool {
        self.track_basic_use_history
    }

    /// Whether loaded content needs lifetime used-type bitmap.
    pub fn tracks_used_type_groups(&self) -> bool {
        self.track_used_type_groups
    }

    /// Resolve modifier definitions for a skill (package overrides bootstrap).
    pub fn modifier_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ModifierDefinition> {
        let pkg: Vec<_> = self
            .package_modifiers
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect();
        if !pkg.is_empty() {
            return pkg;
        }
        bootstrap_modifier_defs()
            .into_iter()
            .filter(|d| d.skill_def == skill_def)
            .collect()
    }

    /// Active skill defs for a skill definition (package only).
    pub fn active_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ActiveSkillDef> {
        self.package_actives
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect()
    }

    /// First active def for skill (if any).
    pub fn first_active_for_skill(&self, skill_def: SkillDefId) -> Option<ActiveSkillDef> {
        self.package_actives
            .iter()
            .find(|d| d.skill_def == skill_def)
            .cloned()
    }

    /// View-as defs for a skill definition.
    pub fn view_as_defs_for_skill(&self, skill_def: SkillDefId) -> Vec<ViewAsDef> {
        self.package_view_as
            .iter()
            .filter(|d| d.skill_def == skill_def)
            .cloned()
            .collect()
    }

    /// All view-as defs.
    pub fn all_view_as(&self) -> &[ViewAsDef] {
        &self.package_view_as
    }

    /// Load / merge a content package (idempotent if same logic_hash already present).
    pub fn load_package(&mut self, pkg: &ContentPackage) -> Result<(), ContentLoadError> {
        // ponytail: package loads are cold-path; stage deltas instead if clone cost is ever measured.
        let mut staged = self.clone();
        staged.load_package_inner(pkg)?;
        *self = staged;
        Ok(())
    }

    fn load_package_inner(&mut self, pkg: &ContentPackage) -> Result<(), ContentLoadError> {
        if pkg.manifest.logic_hash != pkg.compute_logic_hash() {
            return Err(ContentLoadError::Invalid(format!(
                "logic hash mismatch for {}/{}@{}",
                pkg.manifest.namespace, pkg.manifest.package_id, pkg.manifest.version
            )));
        }
        if self
            .packages
            .iter()
            .any(|p| p.logic_hash == pkg.manifest.logic_hash)
        {
            return Ok(());
        }
        if self.packages.iter().any(|loaded| {
            loaded.namespace == pkg.manifest.namespace
                && loaded.package_id == pkg.manifest.package_id
        }) {
            return Err(ContentLoadError::Conflict(format!(
                "package slot {}/{}",
                pkg.manifest.namespace, pkg.manifest.package_id
            )));
        }
        let dingpan_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN)
            })
            .map(|module| module.name.as_str())
            .collect();
        let tanlu_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_TANLU)
            })
            .map(|module| module.name.as_str())
            .collect();
        let same_color_judgment_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let current_trick_nullification_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let protected_black_judgment_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let all_resolved_sha_gift_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let reduced_tuxi_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let rejianchu_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let dcqiaomeng_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_DCQIAOMENG)
            })
            .map(|module| module.name.as_str())
            .collect();
        let repindi_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        if pkg.manifest.rule_vm_abi_version < 301
            && (pkg
                .modifier_index
                .iter()
                .any(|modifier| modifier.target == "card_number")
                || pkg.active_index.iter().any(|active| {
                    active.uses_per_play_phase.is_some()
                        || active
                            .selection
                            .as_ref()
                            .is_some_and(|selection| selection.sum_effective_number_exact.is_some())
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 301 content requires rule_vm_abi_version >= 301".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 302
            && (!dingpan_modules.is_empty()
                || pkg.active_index.iter().any(|active| {
                    active.uses_per_turn_source.as_deref() == Some("living_rebel_count")
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 302 content requires rule_vm_abi_version >= 302".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 303 && !tanlu_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 303 content requires rule_vm_abi_version >= 303".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 304
            && pkg.active_index.iter().any(|active| {
                active
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.same_suit)
            })
        {
            return Err(ContentLoadError::Invalid(
                "ABI 304 content requires rule_vm_abi_version >= 304".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 305
            && pkg.trigger_index.iter().any(|trigger| {
                trigger.filter == "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty"
            })
        {
            return Err(ContentLoadError::Invalid(
                "ABI 305 content requires rule_vm_abi_version >= 305".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 306
            && pkg
                .trigger_index
                .iter()
                .any(|trigger| trigger.filter == "owner_alive_source_target_was_linked")
        {
            return Err(ContentLoadError::Invalid(
                "ABI 306 content requires rule_vm_abi_version >= 306".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 307 && !same_color_judgment_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 307 content requires rule_vm_abi_version >= 307".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 308
            && (!current_trick_nullification_modules.is_empty()
                || pkg.trigger_index.iter().any(|trigger| {
                    trigger.filter == "owner_alive_non_multitarget_trick_targets_gt_one"
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 308 content requires rule_vm_abi_version >= 308".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 309 && !protected_black_judgment_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 309 content requires rule_vm_abi_version >= 309".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 310 && !all_resolved_sha_gift_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 310 content requires rule_vm_abi_version >= 310".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 311 && !reduced_tuxi_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 311 content requires rule_vm_abi_version >= 311".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 312 && !rejianchu_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 312 content requires rule_vm_abi_version >= 312".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 313 && !dcqiaomeng_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 313 content requires rule_vm_abi_version >= 313".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 314 && !repindi_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 314 content requires rule_vm_abi_version >= 314".into(),
            ));
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| dingpan_modules.contains(trigger.module.as_str()))
            || dingpan_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .active_index
                    .iter()
                    .filter(|active| active.module == **module)
                    .any(|active| {
                        found = true;
                        active.window != "play_phase"
                            || active.uses_per_turn_source.as_deref() != Some("living_rebel_count")
                            || !active.targets.as_ref().is_some_and(|targets| {
                                targets.min == 1
                                    && targets.max == 1
                                    && targets.allow_self
                                    && targets.has_equip
                            })
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Dingpan bytecode requires `active play_phase uses_per_turn living_rebel_count targets 1 1 has_equip`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| tanlu_modules.contains(active.module.as_str()))
            || tanlu_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_begin"
                            || trigger.filter != "owner_alive_not_target"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Tanlu bytecode requires `trigger phase_begin owner_alive_not_target optional`"
                    .into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| current_trick_nullification_modules.contains(active.module.as_str()))
            || current_trick_nullification_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_used"
                            || trigger.filter != "owner_alive_non_multitarget_trick_targets_gt_one"
                            || trigger.optional
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "current-trick target nullification bytecode requires `trigger card_used owner_alive_non_multitarget_trick_targets_gt_one`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| protected_black_judgment_modules.contains(active.module.as_str()))
            || protected_black_judgment_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_prepare"
                            || trigger.filter != "owner_is_target"
                            || !trigger.optional
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "protected black-judgment bytecode requires `trigger phase_prepare owner_is_target optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| reduced_tuxi_modules.contains(active.module.as_str()))
            || reduced_tuxi_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_draw"
                            || trigger.filter != "owner_is_target_positive_draw_other_hand_nonempty"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "reduced Tuxi bytecode requires `trigger phase_draw owner_is_target_positive_draw_other_hand_nonempty optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| rejianchu_modules.contains(active.module.as_str()))
            || rejianchu_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_targeted"
                            || trigger.filter
                                != "owner_source_sha_target_alive_has_discardable_hand_equip"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Rejianchu bytecode requires `trigger card_targeted owner_source_sha_target_alive_has_discardable_hand_equip optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| dcqiaomeng_modules.contains(active.module.as_str()))
            || dcqiaomeng_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_targeted"
                            || trigger.filter
                                != "owner_source_first_target_black_card_any_target_other_discardable_hand_equip"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Dcqiaomeng bytecode requires `trigger card_targeted owner_source_first_target_black_card_any_target_other_discardable_hand_equip optional`".into(),
            ));
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| repindi_modules.contains(trigger.module.as_str()))
            || repindi_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .active_index
                    .iter()
                    .filter(|active| active.module == **module)
                    .any(|active| {
                        found = true;
                        active.window != "play_phase"
                            || !active.targets.as_ref().is_some_and(|targets| {
                                targets.min == 1
                                    && targets.max == 1
                                    && targets.allow_self
                                    && targets.unused_by_skill_this_play_phase
                            })
                            || !active.selection.as_ref().is_some_and(|selection| {
                                selection.zone == "hand_equip"
                                    && selection.min == 1
                                    && selection.max == 1
                                    && !selection.allow_cancel
                                    && selection.unused_category_by_skill_this_play_phase
                            })
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Repindi bytecode requires `active play_phase targets 1 1 unused_by_skill_this_play_phase select_he 1 1 unused_category_by_skill_this_play_phase`".into(),
            ));
        }
        if pkg.manifest.dependencies.is_empty() {
            let available_skills: BTreeSet<_> = self
                .package_skills
                .keys()
                .copied()
                .chain(pkg.skill_index.iter().map(|skill| skill.skill_def_id))
                .collect();
            validate_character_skills(&pkg.character_index, &available_skills)?;
        }

        if pkg.manifest.rule_vm_abi_version >= 42 {
            self.track_turn_hand_card_suit_history = true;
            self.track_play_card_category_history = true;
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_play_phase_card_category_count_positive")
        {
            self.track_play_card_category_history_all_users = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 43 {
            self.track_sha_use_refund_provenance = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 44 {
            self.track_opening_extra_play_lifecycle = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 232 {
            self.track_turn_used_card_suit_history = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 285 {
            self.track_owner_card_category_history = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 347 {
            self.track_damage_taken_count = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 353 {
            self.track_turn_damage_pair_counts = true;
        }
        if pkg
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == 2_581_852_991)
        {
            self.track_entered_dying_history = true;
        }
        if pkg.modules.iter().any(|module| {
            module_uses_effect_tag(
                module,
                rule_vm::bytecode::effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS,
            )
        }) {
            self.track_turn_basic_card_variant_history = true;
        }
        if pkg.modules.iter().any(|module| {
            module_uses_effect_tag(
                module,
                rule_vm::bytecode::effect_tag::ALIGN_DEBATE_OPINIONS,
            )
        }) {
            self.track_damaged_by_player_bitset = true;
        }

        let mut local_modules = BTreeSet::new();

        for m in &pkg.modules {
            rule_vm::verify_module(m).map_err(|e| ContentLoadError::Verify(e.to_string()))?;
            if !local_modules.insert(m.name.clone()) {
                return Err(ContentLoadError::Conflict(format!(
                    "duplicate module {} in {}/{}",
                    m.name, pkg.manifest.namespace, pkg.manifest.package_id
                )));
            }
            let key =
                package_module_key(&pkg.manifest.namespace, &pkg.manifest.package_id, &m.name);
            if let Some(existing) = self.modules.get(&key) {
                if existing.logic_hash != m.logic_hash {
                    return Err(ContentLoadError::Conflict(key));
                }
            } else {
                self.modules.insert(key, m.clone());
            }
        }

        for skill in &pkg.skill_index {
            let metadata = SkillMetadata {
                skill_def: SkillDefId::new(skill.skill_def_id),
                name: skill.name.clone(),
                locked: skill.locked,
                lord_only: skill.lord_only,
                non_gainable: skill.non_gainable,
                conversion: skill.conversion,
            };
            if let Some(existing) = self.package_skills.get(&skill.skill_def_id) {
                if existing != &metadata {
                    return Err(ContentLoadError::Conflict(format!(
                        "skill definition {}",
                        skill.skill_def_id
                    )));
                }
            } else {
                self.package_skills.insert(skill.skill_def_id, metadata);
            }
            // Phase 3: reverse name→def index for AddSkillByName resolution.
            self.skill_def_by_name
                .entry(skill.name.clone())
                .or_insert(skill.skill_def_id);
        }

        for character in &pkg.character_index {
            if character.max_hp == 0 {
                return Err(ContentLoadError::Invalid(format!(
                    "character {} has zero max_hp",
                    character.id
                )));
            }
            if character.initial_hp > character.max_hp {
                return Err(ContentLoadError::Invalid(format!(
                    "character {} initial_hp exceeds max_hp",
                    character.id
                )));
            }
            if let Some(existing) = self.package_characters.get(&character.id) {
                if existing != character {
                    return Err(ContentLoadError::Conflict(format!(
                        "character definition {}",
                        character.id
                    )));
                }
            } else {
                self.package_characters
                    .insert(character.id, character.clone());
            }
        }
        for card in &pkg.card_index {
            if let Some(existing) = self.package_cards.get(&card.card_def_id) {
                if existing != card {
                    return Err(ContentLoadError::Conflict(format!(
                        "card definition {}",
                        card.card_def_id
                    )));
                }
            } else {
                self.package_cards.insert(card.card_def_id, card.clone());
            }
        }

        for t in &pkg.trigger_index {
            let event = map_event(&t.event)
                .ok_or_else(|| ContentLoadError::Invalid(format!("unknown event {}", t.event)))?;
            let filter = match map_filter(&t.filter) {
                Some(f) => f,
                None => {
                    eprintln!("DBG unknown filter: {:?}", t.filter);
                    return Err(ContentLoadError::Invalid(format!(
                        "unknown trigger filter {}",
                        t.filter
                    )));
                }
            };
            let tracks_turn_card_name = matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
            );
            self.track_turn_card_history |= matches!(
                &filter,
                TriggerFilter::OwnerSourceAlternatesColor
                    | TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
                    | TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
            );
            self.track_turn_card_name_history |= tracks_turn_card_name
                || matches!(
                    &filter,
                    TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
                        | TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
                );
            self.track_play_card_history |=
                matches!(&filter, TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank);
            self.track_per_player_turn_card_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
                    | TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
            );
            self.track_per_player_play_phase_card_count |= matches!(
                &filter,
                TriggerFilter::OwnerSourcePlayCardCountEquals(_)
                    | TriggerFilter::OwnerSourcePlayCardCountEqualsHp
                    | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
                    | TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
            );
            self.track_play_phase_card_suit_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
            );
            self.track_discard_pile_suit_history |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
                    | TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
            );
            self.track_per_player_turn_card_action_count |= matches!(
                &filter,
                TriggerFilter::OwnerCardActionCountEqualsAttackRange
                    | TriggerFilter::OwnerAliveDddZhilianAvailable
            );
            self.track_lifetime_card_action_count |=
                matches!(&filter, TriggerFilter::OwnerLifetimeCardAction);
            self.track_play_card_category_duplicates |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
            );
            self.track_per_player_turn_sha_use_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveUsedShaThisTurn
                    | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
                    | TriggerFilter::OwnerSourceFirstShaTargetThisTurn
                    | TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
            );
            self.track_turn_use_response_discard_count |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
            );
            self.track_discard_phase_discard_count |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
                    | TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
            );
            self.track_discard_phase_lost_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
                    | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
            );
            self.track_turn_discarded_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
                    | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
            );
            self.track_turn_haoyi_cards |= matches!(
                &filter,
                TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable
                    | TriggerFilter::OwnerAliveDddZhilianAvailable
            );
            self.track_ddd_chengjing_history |=
                matches!(&filter, TriggerFilter::OwnerAliveDddChengjingAvailable);
            self.track_game_card_history |=
                matches!(&filter, TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank);
            self.track_turn_red_card_target_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
            );
            self.track_turn_card_target_counts |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
                    | TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
            );
            self.track_phase_target_hp_history |= matches!(
                &filter,
                TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
                    | TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
            );
            self.track_turn_hp_change_history |= matches!(
                &filter,
                TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
                    | TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
            );
            self.track_turn_haoyi_cards |= matches!(
                &filter,
                TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
            );
            self.track_turn_card_gain_count |= matches!(
                &filter,
                TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
                    | TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
                    | TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
            );
            self.track_play_phase_card_gain_history |= matches!(
                &filter,
                TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
                    | TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
            );
            self.track_card_target_history |= matches!(
                &filter,
                TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
            );
            self.track_used_type_groups |= matches!(
                &filter,
                TriggerFilter::OwnerIsTargetSanshouTypeTracked
            );
            self.track_turn_card_loss_history |= matches!(
                &filter,
                TriggerFilter::OwnerPhaseEndLostCards
                    | TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
                    | TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
                    | TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
                    | TriggerFilter::OwnerAliveLostHandThisTurn
                    | TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
            );
            if !local_modules.contains(&t.module) {
                return Err(ContentLoadError::Invalid(format!(
                    "trigger references missing module {}",
                    t.module
                )));
            }
            let id = TriggerDefId(self.next_trigger_id);
            self.next_trigger_id = self.next_trigger_id.saturating_add(1);
            self.package_triggers.push(TriggerDefinition {
                id,
                skill_def: SkillDefId::new(t.skill_def_id),
                event,
                timing: TimingBand::Normal,
                priority: 0,
                optional: t.optional,
                choice_actor: t.choice_actor,
                filter,
                max_per_segment: 8,
                handler: TriggerHandler::RunVm {
                    module: package_module_key(
                        &pkg.manifest.namespace,
                        &pkg.manifest.package_id,
                        &t.module,
                    ),
                    entry_pc: t.entry_pc,
                },
            });
        }

        for m in &pkg.modifier_index {
            if m.target == "card_number"
                && (m.selector.is_some()
                    || m.mode != "override"
                    || m.value_source.is_some()
                    || !(1..=13).contains(&m.value))
            {
                return Err(ContentLoadError::Invalid(
                    "card_number requires `override` with an integer from 1 to 13".into(),
                ));
            }
            let target = map_mod_target(&m.target, m.selector.as_deref()).ok_or_else(|| {
                ContentLoadError::Invalid(format!("unknown modifier target {}", m.target))
            })?;
            let mode = map_mod_mode(&m.mode).ok_or_else(|| {
                ContentLoadError::Invalid(format!("unknown modifier mode {}", m.mode))
            })?;
            let value_source = m
                .value_source
                .as_deref()
                .map(map_mod_value_source)
                .transpose()
                .map_err(ContentLoadError::Invalid)?;
            self.track_per_player_turn_card_count |= matches!(
                value_source,
                Some(ModifierValueSource::OwnerCurrentTurnCardUseCount)
            );
            self.track_per_player_play_phase_card_count |= matches!(
                value_source,
                Some(ModifierValueSource::OwnerPlayPhaseCardUseCountZero)
            );
            self.track_turn_card_gain_count |=
                matches!(target, ModifierTarget::HandCardGainedThisTurnOnly);
            let id = self.next_modifier_id;
            self.next_modifier_id = self.next_modifier_id.saturating_add(1);
            self.package_modifiers.push(ModifierDefinition {
                id,
                skill_def: SkillDefId::new(m.skill_def_id),
                target,
                mode,
                value_source,
                value: m.value,
                order: 0,
                label_key: match &m.selector {
                    Some(selector) => {
                        format!("pkg.{}.{}.{}", m.target, selector, m.skill_def_id)
                    }
                    None => format!("pkg.{}.{}", m.target, m.skill_def_id),
                },
            });
        }

        for a in &pkg.active_index {
            if !local_modules.contains(&a.module) {
                return Err(ContentLoadError::Invalid(format!(
                    "active skill references missing module {}",
                    a.module
                )));
            }
            if a.targets
                .as_ref()
                .is_some_and(|targets| targets.max < targets.min)
            {
                return Err(ContentLoadError::Invalid(
                    "active target contract requires min <= max".into(),
                ));
            }
            if a.selection
                .as_ref()
                .is_some_and(|selection| !matches!(selection.zone.as_str(), "hand" | "hand_equip"))
            {
                return Err(ContentLoadError::Invalid(
                    "unsupported active selection zone".into(),
                ));
            }
            if a.selection.as_ref().is_some_and(|selection| {
                selection.filter.as_deref().is_some_and(|filter| {
                    !matches!(
                        filter,
                        "card_equip"
                            | "card_armor"
                            | "card_sha"
                            | "card_basic"
                            | "card_nonbasic"
                            | "card_non_equip"
                            | "card_red"
                            | "card_black"
                            | "card_heart"
                    )
                })
            }) {
                return Err(ContentLoadError::Invalid(
                    "unsupported active selection filter".into(),
                ));
            }
            if a.uses_per_turn == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_turn must be positive".into(),
                ));
            }
            if a.uses_per_play_phase == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_play_phase must be positive".into(),
                ));
            }
            if a.uses_per_play_phase.is_some() && a.window != "play_phase" {
                return Err(ContentLoadError::Invalid(
                    "active uses_per_play_phase requires the play_phase window".into(),
                ));
            }
            if a.uses_per_turn_source.as_deref().is_some_and(|source| {
                !matches!(
                    source,
                    "owner_hp" | "owner_lost_hp_min_one" | "living_rebel_count"
                )
            }) {
                return Err(ContentLoadError::Invalid(
                    "unsupported active uses_per_turn source".into(),
                ));
            }
            if (a.uses_per_turn.is_some() && a.uses_per_turn_source.is_some())
                || (a.uses_per_play_phase.is_some()
                    && (a.uses_per_turn.is_some() || a.uses_per_turn_source.is_some()))
            {
                return Err(ContentLoadError::Invalid(
                    "active use limits cannot combine turn and Play-phase scopes".into(),
                ));
            }
            if a.selection.as_ref().is_some_and(|selection| {
                selection.sum_effective_number_exact.is_some() && selection.zone != "hand"
            }) {
                return Err(ContentLoadError::Invalid(
                    "sum_effective_number_exact requires a hand selection".into(),
                ));
            }
            if a.selection
                .as_ref()
                .is_some_and(|selection| selection.sum_effective_number_exact == Some(0))
            {
                return Err(ContentLoadError::Invalid(
                    "sum_effective_number_exact must be positive".into(),
                ));
            }
            if a.mark_at_least
                .is_some_and(|(mark, count)| mark == 0 || count == 0)
            {
                return Err(ContentLoadError::Invalid(
                    "active mark_at_least requires non-zero mark and count".into(),
                ));
            }
            if a.owner_missing_skill == Some(0) {
                return Err(ContentLoadError::Invalid(
                    "active owner_missing_skill requires a non-zero skill definition".into(),
                ));
            }
            if a.targets.as_ref().is_some_and(|targets| {
                (targets.linked && targets.unlinked) || targets.max_by_owner_mark == Some(0)
            }) {
                return Err(ContentLoadError::Invalid(
                    "active target linked/max_by_owner_mark constraint is invalid".into(),
                ));
            }
            self.package_actives.push(ActiveSkillDef {
                skill_def: SkillDefId::new(a.skill_def_id),
                window: a.window.clone(),
                module: package_module_key(
                    &pkg.manifest.namespace,
                    &pkg.manifest.package_id,
                    &a.module,
                ),
                entry_pc: a.entry_pc,
                targets: a.targets.clone(),
                uses_per_turn: a.uses_per_turn,
                uses_per_turn_source: a.uses_per_turn_source.clone(),
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
                selection: a.selection.clone(),
            });
        }

        for v in &pkg.view_as_index {
            let source_free = v.source_zone.as_deref() == Some("none");
            let dynamic_family = matches!(v.as_kind.as_str(), "basic" | "trick" | "ordinary_trick")
                || source_free && matches!(v.as_kind.as_str(), "tao" | "jiu");
            if source_free != dynamic_family {
                return Err(ContentLoadError::Invalid(
                    "source-free view_as family contract mismatch".into(),
                ));
            }
            if source_free
                && (v.filter != "any"
                    || v.min_cards != 1
                    || v.max_cards != 1
                    || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
                    || v.owner_basic_uses_plus_one_cards
                    || match v.cost.as_deref() {
                        Some(
                            "guhuo"
                            | "face_down_hand"
                            | "dczhaowen"
                            | "draw_one_outside_turn_empty_hand",
                        ) => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.window != "any"
                        }
                        Some("turn_over") => {
                            v.as_kind != "jiu"
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("swap_lower_hand") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                        }
                        Some("dddshilie") => {
                            v.as_kind != "basic"
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                        }
                        Some("dddfusi") => {
                            !matches!(v.as_kind.as_str(), "basic" | "trick")
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("all_hand") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "play_phase"
                        }
                        Some("hand_equip_mark") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                                || v.mark_cost.is_none()
                        }
                        Some("lose_max_hp_1") => {
                            !matches!(v.as_kind.as_str(), "tao" | "jiu")
                                || v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn != Some(1)
                                || v.window != "any"
                        }
                        Some("potdaozhuan") => {
                            v.branch.is_some()
                                || v.once_per_round
                                || v.next_branch.is_some()
                                || v.uses_per_turn.is_some()
                                || v.window != "any"
                        }
                        Some("gain_max_hp_1" | "disable_equip_slot") => {
                            v.branch.is_none() || !v.once_per_round || v.next_branch.is_none()
                        }
                        _ => true,
                    })
            {
                return Err(ContentLoadError::Invalid(
                    "incomplete source-free view_as contract".into(),
                ));
            }
            if (v.mark_cost.is_some() && v.cost.as_deref() != Some("hand_equip_mark"))
                || (v.post_draw != 0
                    && !matches!(v.cost.as_deref(), Some("hand_equip_mark" | "dddzhijie")))
            {
                return Err(ContentLoadError::Invalid(
                    "view_as mark/post-draw contract mismatch".into(),
                ));
            }
            let valid_bonus = match v.multi_card_bonus.as_deref() {
                None => true,
                Some("damage_1") => v.as_kind == "sha" && v.max_cards > 1,
                Some("recover_1") => v.as_kind == "tao" && v.max_cards > 1,
                Some("discard_current_hand_equip") => {
                    matches!(v.as_kind.as_str(), "shan" | "wuxie") && v.max_cards > 1
                }
                Some(_) => false,
            };
            let potqingyan = v.cost.as_deref() == Some("potqingyan");
            let dddzhijie = v.cost.as_deref() == Some("dddzhijie");
            let dddtongyu = v.cost.as_deref() == Some("dddtongyu");
            if v.min_cards == 0
                || v.min_cards > v.max_cards
                || if potqingyan {
                    v.max_cards != 5
                } else if dddtongyu {
                    v.max_cards != 4
                } else {
                    v.max_cards > 2 && v.max_cards != u8::MAX
                }
                || (v.refund_use_count_if_no_damage && v.as_kind != "sha")
                || (v.same_suit && v.max_cards == 1 && !v.owner_hp_min_one_cards)
                || v.nature.as_deref().is_some_and(|nature| {
                    !matches!(nature, "fire" | "thunder") || v.as_kind != "sha"
                })
                || !valid_bonus
                || potqingyan
                    && (!matches!(v.as_kind.as_str(), "shan" | "wuxie")
                        || v.window != "response"
                        || v.filter != "any"
                        || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
                        || v.min_cards != 1
                        || v.uses_per_turn.is_some()
                        || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
                || dddzhijie
                    && (!matches!(v.as_kind.as_str(), "shan" | "wuxie")
                        || v.window != "response"
                        || v.filter != "any"
                        || v.source_zone.as_deref() != Some("hand_expansion")
                        || v.min_cards != 2
                        || v.max_cards != 2
                        || v.post_draw != 2
                        || v.uses_per_turn.is_some()
                        || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
                || dddtongyu
                    && (v.as_kind != "wugu"
                        || v.window != "play_phase"
                        || v.filter != "any"
                        || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
                        || v.min_cards != 1
                        || v.max_cards != 4
                        || v.uses_per_turn != Some(1)
                        || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
                        || v.same_suit
                        || v.nature.is_some()
                        || v.multi_card_bonus.is_some()
                        || v.branch.is_some()
                        || v.once_per_round
                        || v.next_branch.is_some())
            {
                return Err(ContentLoadError::Invalid(
                    "invalid multi-card view_as contract".into(),
                ));
            }
            self.track_basic_use_history |= v.owner_basic_uses_plus_one_cards;
            self.package_view_as.push(ViewAsDef {
                skill_def: SkillDefId::new(v.skill_def_id),
                as_kind: v.as_kind.clone(),
                window: v.window.clone(),
                filter: v.filter.clone(),
                source_zone: v.source_zone.clone(),
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
                nature: v.nature.clone(),
                multi_card_bonus: v.multi_card_bonus.clone(),
                branch: v.branch,
                once_per_round: v.once_per_round,
                next_branch: v.next_branch,
                cost: v.cost.clone(),
                mark_cost: v.mark_cost,
                post_draw: v.post_draw,
            });
        }

        self.packages.push(LoadedPackageMeta {
            namespace: pkg.manifest.namespace.clone(),
            package_id: pkg.manifest.package_id.clone(),
            version: pkg.manifest.version.clone(),
            logic_hash: pkg.manifest.logic_hash,
        });
        self.packages.sort_by(|a, b| {
            (&a.namespace, &a.package_id, &a.version).cmp(&(
                &b.namespace,
                &b.package_id,
                &b.version,
            ))
        });
        self.package_triggers
            .sort_by_key(|definition| definition.id);
        self.package_modifiers
            .sort_by_key(|definition| definition.id);
        self.package_actives.sort_by(|a, b| {
            (a.skill_def.raw(), &a.window, &a.module).cmp(&(
                b.skill_def.raw(),
                &b.window,
                &b.module,
            ))
        });
        self.package_view_as.sort_by(|a, b| {
            (
                a.skill_def.raw(),
                &a.window,
                &a.as_kind,
                &a.filter,
                &a.source_zone,
                a.uses_per_turn,
                a.branch,
            )
                .cmp(&(
                    b.skill_def.raw(),
                    &b.window,
                    &b.as_kind,
                    &b.filter,
                    &b.source_zone,
                    b.uses_per_turn,
                    b.branch,
                ))
        });
        self.recompute_combined_hash();
        Ok(())
    }

    /// Validate character skill references after a dependency set has fully loaded.
    pub fn validate_character_skill_closure(&self) -> Result<(), ContentLoadError> {
        validate_character_skills(
            self.package_characters.values(),
            &self.package_skills.keys().copied().collect(),
        )
    }

    fn recompute_combined_hash(&mut self) {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/content-registry/v2");
        for p in &self.packages {
            h.update(&p.logic_hash);
            h.update(p.namespace.as_bytes());
            h.update(p.package_id.as_bytes());
            h.update(p.version.as_bytes());
        }
        for (name, m) in &self.modules {
            h.update(name.as_bytes());
            h.update(&m.logic_hash);
        }
        for metadata in self.package_skills.values() {
            h.update(&metadata.skill_def.raw().to_le_bytes());
            h.update(metadata.name.as_bytes());
            h.update(&[u8::from(metadata.locked)]);
            if metadata.lord_only {
                h.update(b"lord_only/v1");
            }
            if metadata.non_gainable {
                h.update(b"non_gainable/v1");
            }
            if metadata.conversion {
                h.update(b"conversion/v1");
            }
        }
        for character in self.package_characters.values() {
            h.update(&character.id.to_le_bytes());
            h.update(character.name_key.as_bytes());
            h.update(&character.max_hp.to_le_bytes());
            h.update(&character.initial_hp.to_le_bytes());
            for skill in &character.skills {
                h.update(&skill.to_le_bytes());
            }
        }
        for m in &self.package_modifiers {
            h.update(&m.id.to_le_bytes());
            h.update(&m.skill_def.raw().to_le_bytes());
            h.update(mod_target_tag(m.target).as_bytes());
            if let ModifierTarget::CardIgnoredHandLimit(selector)
            | ModifierTarget::CardDistanceUnlimitedFor(selector)
            | ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(selector)
            | ModifierTarget::CardDistanceLimitFor(selector)
            | ModifierTarget::CardTargetsAllFor(selector)
            | ModifierTarget::CardTargetCancelFor(selector)
            | ModifierTarget::CardTargetCancelFromCurrentFor(selector)
            | ModifierTarget::CardTargetCancelTurnBannedFor(selector)
            | ModifierTarget::ArmorCardTargetCancelFor(selector)
            | ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(
                selector,
            )
            | ModifierTarget::CardUseCancelFor(selector)
            | ModifierTarget::CardResponseCancelFor(selector)
            | ModifierTarget::CardUseUnlimitedFor(selector)
            | ModifierTarget::CardTargetLimitFor(selector)
            | ModifierTarget::CardSaveCancelFor(selector)
            | ModifierTarget::CardName(selector)
            | ModifierTarget::CardNatureFor(selector)
            | ModifierTarget::CardGainProtectedFor(selector)
            | ModifierTarget::CardUseUnrespondableFor(selector) = m.target
            {
                h.update(selector.tag().as_bytes());
            }
            if let ModifierTarget::CardSuit(suit) = m.target {
                h.update(mod_suit_tag(suit).as_bytes());
            }
            h.update(mod_mode_tag(m.mode).as_bytes());
            if let Some(source) = m.value_source {
                h.update(b"value-source/v1");
                h.update(mod_value_source_tag(source).as_bytes());
            }
            h.update(&m.value.to_le_bytes());
        }
        self.combined_logic_hash = *h.finalize().as_bytes();
    }
}

fn validate_character_skills<'a>(
    characters: impl IntoIterator<Item = &'a PackageCharacterEntry>,
    available_skills: &BTreeSet<u32>,
) -> Result<(), ContentLoadError> {
    for character in characters {
        if let Some(skill) = character
            .skills
            .iter()
            .find(|skill| !available_skills.contains(skill))
        {
            return Err(ContentLoadError::Invalid(format!(
                "character {} references unavailable skill {}",
                character.id, skill
            )));
        }
    }
    Ok(())
}

fn module_uses_effect_tag(module: &BytecodeModule, target: u16) -> bool {
    use rule_vm::bytecode::op;

    let code = &module.code;
    let mut index = 0usize;
    while let Some(&opcode) = code.get(index) {
        index += 1;
        match opcode {
            op::HOST_EFFECT => {
                let Some(&tag) = code.get(index) else {
                    return false;
                };
                if u16::from(tag) == target {
                    return true;
                }
                index += 1;
            }
            op::HOST_EFFECT_EXT => {
                let Some(bytes) = code.get(index..index + 2) else {
                    return false;
                };
                if u16::from_le_bytes([bytes[0], bytes[1]]) == target {
                    return true;
                }
                index += 2;
            }
            op::CONST_I32 | op::CONST_U32 | op::CONST_BOOL => index += 4,
            op::LOAD | op::STORE | op::HOST_QUERY | op::HOST_CHOICE => index += 1,
            op::JUMP | op::JUMP_IF_FALSE => index += 2,
            op::NOP
            | op::HALT
            | op::TRAP
            | op::DUP
            | op::POP
            | op::NOT
            | op::ADD
            | op::SUB
            | op::MUL
            | op::EQ
            | op::LT
            | op::MIN
            | op::HOST_CHOICE_RANGE
            | op::HOST_SELF
            | op::HOST_EVENT_PLAYER
            | op::HOST_EVENT_SOURCE
            | op::HOST_EVENT_CARD
            | op::HOST_SKILL
            | op::HOST_EVENT_AMOUNT => {}
            _ => return false,
        }
        if index > code.len() {
            return false;
        }
    }
    false
}

/// Stable runtime key for a package-local bytecode module.
pub fn package_module_key(namespace: &str, package_id: &str, module: &str) -> String {
    format!("{namespace}/{package_id}::{module}")
}

fn mod_target_tag(t: ModifierTarget) -> &'static str {
    match t {
        ModifierTarget::AttackRange => "attack_range",
        ModifierTarget::DistanceFrom => "distance_from",
        ModifierTarget::DistanceFromHpGteTarget => "distance_from_if_hp_gte_target",
        ModifierTarget::DistanceFromSkillBoundTarget => "distance_from_skill_bound_target",
        ModifierTarget::LinkLossCancel => "link_loss_cancel",
        ModifierTarget::DistanceTo => "distance_to",
        ModifierTarget::HandLimit => "hand_limit",
        ModifierTarget::HandLimitBase => "hand_limit_base",
        ModifierTarget::DrawCount => "draw_count",
        ModifierTarget::OptionalDrawCount => "optional_draw_count",
        ModifierTarget::ShaDamage => "sha_damage",
        ModifierTarget::ShaUseLimit => "sha_use_limit",
        ModifierTarget::ShaTargetLimit => "sha_target_limit",
        ModifierTarget::CardTargetsAllFor(_) => "card_targets_all",
        ModifierTarget::CardTargetCancelFor(_) => "card_target_cancel",
        ModifierTarget::CardTargetCancelFromCurrentFor(_) => "card_target_cancel_from_current",
        ModifierTarget::CardTargetCancelTurnBannedFor(_) => "card_target_cancel_owner_turn_banned",
        ModifierTarget::CardTargetsSelfOnly => "card_targets_self_only",
        ModifierTarget::ResponseCount => "response_count",
        ModifierTarget::SkipDiscardIfNoSha => "skip_discard_if_no_sha",
        ModifierTarget::DrawReplacementDamage => "draw_replacement_damage",
        ModifierTarget::PrepareScryCount => "prepare_scry_count",
        ModifierTarget::JudgmentRewrite => "judgment_rewrite",
        ModifierTarget::DamageAmount => "damage_amount",
        ModifierTarget::DamageNature => "damage_nature",
        ModifierTarget::DamageCancel => "damage_cancel",
        ModifierTarget::CardUseCancel => "card_use_cancel",
        ModifierTarget::CardUseCancelFor(_) => "card_use_cancel",
        ModifierTarget::CardResponseCancel => "card_response_cancel",
        ModifierTarget::CardResponseCancelFor(_) => "card_response_cancel",
        ModifierTarget::CardSaveCancel => "card_save_cancel",
        ModifierTarget::CardSaveCancelFor(_) => "card_save_cancel",
        ModifierTarget::ImmuneShunshou => "immune_shunshou",
        ModifierTarget::ImmuneLe => "immune_le",
        ModifierTarget::CardAttr(_) => "card_attr",
        ModifierTarget::ShaUseCancel => "sha_use_cancel",
        ModifierTarget::HandLimitBaseMaxHp => "hand_limit_base_max_hp",
        ModifierTarget::CardDistanceUnlimited => "card_distance_unlimited",
        ModifierTarget::CardDistanceUnlimitedTargetMark(_) => "card_distance_unlimited_target_mark",
        ModifierTarget::CardDistanceUnlimitedSkillOnOwner(_) => "card_distance_unlimited_skill_on_owner",
        ModifierTarget::CardUseUnrespondableSkillOnOwner(_) => "card_use_unrespondable_skill_on_owner",
        ModifierTarget::ShaUseUnlimitedTargetMark(_) => "sha_use_unlimited_target_mark",
        ModifierTarget::CardDistanceUnlimitedFor(_) => "card_distance_unlimited",
        ModifierTarget::CardDistanceLimitFor(_) => "card_distance_limit",
        ModifierTarget::ShaResponseCancel => "sha_response_cancel",
        ModifierTarget::ShaIgnoredHandLimit => "sha_ignored_hand_limit",
        ModifierTarget::CardIgnoredHandLimit(_) => "card_ignored_hand_limit",
        ModifierTarget::CardUseUnlimited => "card_use_unlimited",
        ModifierTarget::CardUseUnlimitedFor(_) => "card_use_unlimited",
        ModifierTarget::FirstHandCardSuitUseUnlimited => "first_hand_card_suit_use_unlimited",
        ModifierTarget::CardTargetLimitFor(_) => "card_target_limit",
        ModifierTarget::DamageTakenCancel => "damage_taken_cancel",
        ModifierTarget::DamageTakenCancelForNature(_) => "damage_taken_cancel",
        ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(_) => {
            "card_distance_unlimited_if_target_hand_gte_owner"
        }
        ModifierTarget::HandCardUseResponseCancel => "hand_card_use_response_cancel",
        ModifierTarget::HandCardGainedThisTurnOnly => "hand_card_gained_this_turn_only",
        ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit => {
            "card_target_cancel_if_target_lacks_matching_suit"
        }
        ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(_) => {
            "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty"
        }
        ModifierTarget::CardSuit(_) => "card_suit",
        ModifierTarget::CardName(_) => "card_name",
        ModifierTarget::ShaUnrespondableFor(_) => "sha_unrespondable",
        ModifierTarget::DamageTakenAmount => "damage_taken_amount",
        ModifierTarget::DamageTakenAmountForNature(_) => "damage_taken_amount",
        ModifierTarget::ShaDamageFor(_) => "sha_damage",
        ModifierTarget::ShaUnrespondableWithinDistance => "sha_unrespondable_within_distance",
        ModifierTarget::ShaTargetUnrespondable => "sha_target_unrespondable",
        ModifierTarget::ShaUseUnrespondableWithinDistance => {
            "sha_use_unrespondable_within_distance"
        }
        ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance => {
            "card_use_unrespondable_within_responder_to_owner_distance"
        }
        ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance => {
            "card_use_unrespondable_within_owner_to_responder_distance"
        }
        ModifierTarget::CardUseUnrespondableFor(_) => "card_use_unrespondable",
        ModifierTarget::DamageAmountForSha => "damage_amount_sha",
        ModifierTarget::GlobalAttackRange => "global_attack_range",
        ModifierTarget::GlobalDamageTakenAmount => "global_damage_taken_amount",
        ModifierTarget::GlobalDamageTakenAmountForNature(_) => "global_damage_taken_amount",
        ModifierTarget::GlobalDamageTakenFinalAmount => "global_damage_taken_final_amount",
        ModifierTarget::GlobalDamageTakenFinalAmountForNature(_) => {
            "global_damage_taken_final_amount"
        }
        ModifierTarget::GlobalHandLimit => "global_hand_limit",
        ModifierTarget::GlobalDrawCount => "global_draw_count",
        ModifierTarget::GlobalShaUseLimit => "global_sha_use_limit",
        ModifierTarget::JudgmentRewriteHandEquipCostMark(_) => {
            "judgment_rewrite_hand_equip_cost_mark"
        }
        ModifierTarget::CardNatureFor(_) => "card_nature",
        ModifierTarget::GlobalDistanceTo => "global_distance_to",
        ModifierTarget::NanmanDamageSourceAsOwner => "nanman_damage_source_as_owner",
        ModifierTarget::DamageTakenSourceRangeThreshold => "damage_taken_source_range_threshold",
        ModifierTarget::SkipJudgePhase => "skip_judge_phase",
        ModifierTarget::InitialHandCount => "initial_hand_count",
        ModifierTarget::DamageTakenFinalAmount => "damage_taken_final_amount",
        ModifierTarget::DamageTakenFinalAmountForNature(_) => "damage_taken_final_amount",
        ModifierTarget::DamageTakenNature => "damage_taken_nature",
        ModifierTarget::DamageAmountForNature(_) => "damage_amount",
        ModifierTarget::DamageAsHpLoss => "damage_as_hp_loss",
        ModifierTarget::DamageBetweenOwnerAndOtherAsHpLoss => {
            "damage_between_owner_and_other_as_hp_loss"
        }
        ModifierTarget::CardDiscardProtectedFor(_) => "card_discard_protected",
        ModifierTarget::ShaDamageTaken => "sha_damage_taken",
        ModifierTarget::ShaDamageIfOutsideTargetRange => "sha_damage_if_outside_target_range",
        ModifierTarget::ShaDamageSourceRangeThreshold => "sha_damage_source_range_threshold",
        ModifierTarget::DamageAsHpLossForSha => "damage_as_hp_loss",
        ModifierTarget::ArmorCardTargetCancelFor(_) => "armor_card_target_cancel",
        ModifierTarget::ArmorDamageTakenAmountForNature(_) => "armor_damage_taken_amount",
        ModifierTarget::ArmorLinkGainCancel => "armor_link_gain_cancel",
        ModifierTarget::IgnoreArmor => "ignore_armor",
        ModifierTarget::ArmorEffectDisabled => "armor_effect_disabled",
        ModifierTarget::DrawPhaseScryFourSplitTwo => "draw_phase_scry_four_split_two",
        ModifierTarget::SkipDiscardPhase => "skip_discard_phase",
        ModifierTarget::DiscardPhaseIncludesEquip => "discard_phase_includes_equip",
        ModifierTarget::ObserverForeignHandFacesVisible => "observer_foreign_hand_faces_visible",
        ModifierTarget::ObserverHiddenIdentitiesVisible => "observer_hidden_identities_visible",
        ModifierTarget::JudgmentRewriteHandEquip => "judgment_rewrite_hand_equip",
        ModifierTarget::JudgmentRewriteRedHandEquipExpansion => {
            "judgment_rewrite_red_hand_equip_expansion"
        }
        ModifierTarget::JudgmentRewriteBlackHandEquip => "judgment_rewrite_black_hand_equip",
        ModifierTarget::SkillsDisabledWhenHpOne => "skills_disabled_when_hp_one",
        ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget => {
            "card_distance_unlimited_for_skill_bound_target"
        }
        ModifierTarget::ShaUseUnlimitedForSkillBoundTarget => {
            "sha_use_unlimited_for_skill_bound_target"
        }
        ModifierTarget::IgnoreArmorForSkillBoundTarget => "ignore_armor_for_skill_bound_target",
        ModifierTarget::CardActionCancelForSkillColorMask => {
            "card_action_cancel_for_skill_color_mask"
        }
        ModifierTarget::IgnoreArmorForSkillCardPolicy => "ignore_armor_for_skill_card_policy",
        ModifierTarget::CardUseUnrespondableForSkillCardPolicy => {
            "card_use_unrespondable_for_skill_card_policy"
        }
        ModifierTarget::JudgmentBestFromDrawPile => "judgment_best_from_draw_pile",
        ModifierTarget::VirtualBaguaIfArmorSlotEmpty => "virtual_bagua_if_armor_slot_empty",
        ModifierTarget::ShaResponseCountIfOtherFemale => "sha_response_count_if_other_female",
        ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn => {
            "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
        }
        ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance => {
            "tao_use_save_cancel_for_dying_beyond_distance"
        }
        ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne => {
            "sha_response_count_if_source_owner_target_distance_le_one"
        }
        ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget => {
            "card_target_cancel_if_other_user_hp_gte_target"
        }
        ModifierTarget::TurnFaceDownCancel => "turn_face_down_cancel",
        ModifierTarget::RestoreSkippedPhase => "restore_skipped_phase",
        ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy => {
            "card_distance_unlimited_for_skill_card_policy"
        }
        ModifierTarget::TurnOverCancel => "turn_over_cancel",
        ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(_) => {
            "card_target_cancel_if_other_user_distance_gt_one"
        }
        ModifierTarget::DuelOpponentResponseCount => "duel_opponent_response_count",
        ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark => {
            "sha_distance_unlimited_if_rank_lt_skill_mark"
        }
        ModifierTarget::ShaNoUseCountIfRankGtSkillMark => "sha_no_use_count_if_rank_gt_skill_mark",
        ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank => {
            "sha_distance_unlimited_if_target_distance_lte_rank"
        }
        ModifierTarget::CardGainProtectedFor(_) => "card_gain_protected",
        ModifierTarget::CardNumber => "card_number",
    }
}

fn mod_mode_tag(m: CombineMode) -> &'static str {
    match m {
        CombineMode::Add => "add",
        CombineMode::Override => "override",
        CombineMode::Min => "min",
        CombineMode::Max => "max",
        CombineMode::Multiply => "multiply",
        CombineMode::Veto => "veto",
    }
}

fn map_mod_value_source(source: &str) -> Result<ModifierValueSource, String> {
    if let Some(skill_def) = source.strip_prefix("owner_skill_pile_count_negative_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(ModifierValueSource::OwnerSkillPileCountNegative)
            .ok_or_else(|| format!("invalid owner skill-pile value source {source}"));
    }
    if source == "owner_skill_pile_count" {
        return Ok(ModifierValueSource::OwnerSkillPileCount);
    }
    if let Some(skill_def) = source.strip_prefix("owner_skill_pile_count_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(ModifierValueSource::OwnerSkillPileCountOf)
            .ok_or_else(|| format!("invalid owner skill-pile value source {source}"));
    }
    if let Some(mark) = source.strip_prefix("queried_player_mark_positive_") {
        return mark
            .parse::<u32>()
            .map(ModifierValueSource::QueriedPlayerMarkPositive)
            .map_err(|_| format!("invalid queried player mark value source {source}"));
    }
    if let Some(mark) = source.strip_prefix("owner_mark_") {
        return mark
            .parse::<u32>()
            .map(ModifierValueSource::OwnerMark)
            .map_err(|_| format!("invalid owner mark value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("alive_wounded_count_at_least_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::AliveWoundedCountAtLeast)
            .ok_or_else(|| format!("invalid alive wounded threshold value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("owner_equip_count_at_least_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::OwnerEquipCountAtLeast)
            .ok_or_else(|| format!("invalid owner equipment threshold value source {source}"));
    }
    if let Some(threshold) = source.strip_prefix("owner_equip_count_floor_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(ModifierValueSource::OwnerEquipCountFloor)
            .ok_or_else(|| format!("invalid owner equipment floor value source {source}"));
    }
    match source {
        "owner_hp" => Ok(ModifierValueSource::OwnerHp),
        "owner_max_hp" => Ok(ModifierValueSource::OwnerMaxHp),
        "owner_hp_minus_one" => Ok(ModifierValueSource::OwnerHpMinusOne),
        "owner_lost_hp" => Ok(ModifierValueSource::OwnerLostHp),
        "owner_lost_hp_minus_one" => Ok(ModifierValueSource::OwnerLostHpMinusOne),
        "owner_lost_hp_min_one" => Ok(ModifierValueSource::OwnerLostHpMinOne),
        "owner_skill_mark" => Ok(ModifierValueSource::OwnerSkillMark),
        "owner_skill_pile_count" => Ok(ModifierValueSource::OwnerSkillPileCount),
        "owner_skill_state_u32" => Ok(ModifierValueSource::OwnerSkillStateU32),
        "owner_skill_state_u32_negative" => Ok(ModifierValueSource::OwnerSkillStateU32Negative),
        "queried_player_qun_owner_lord_alive" => {
            Ok(ModifierValueSource::QueriedPlayerQunOwnerLordAlive)
        }
        "owner_linked" => Ok(ModifierValueSource::OwnerLinked),
        "owner_skill_use_count" => Ok(ModifierValueSource::OwnerSkillUseCount),
        "alive_wounded_or_guiming_wu_count_min_one" => {
            Ok(ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne)
        }
        "alive_group_count" => Ok(ModifierValueSource::AliveGroupCount),
        "alive_female_count" => Ok(ModifierValueSource::AliveFemaleCount),
        "alive_other_count_capped_four" => Ok(ModifierValueSource::AliveOtherCountCappedFour),
        "owner_no_mount" => Ok(ModifierValueSource::OwnerNoMount),
        "owner_hand_empty" => Ok(ModifierValueSource::OwnerHandEmpty),
        "owner_has_treasure" => Ok(ModifierValueSource::OwnerHasTreasure),
        "alive_mounted_player_count_min_one" => {
            Ok(ModifierValueSource::AliveMountedPlayerCountMinOne)
        }
        "owner_equip_count_odd" => Ok(ModifierValueSource::OwnerEquipCountOdd),
        "owner_equip_count_even" => Ok(ModifierValueSource::OwnerEquipCountEven),
        "owner_equip_count_half_floor" => Ok(ModifierValueSource::OwnerEquipCountHalfFloor),
        "owner_equip_count_exactly_one" => Ok(ModifierValueSource::OwnerEquipCountExactlyOne),
        "owner_equip_count" => Ok(ModifierValueSource::OwnerEquipCount),
        "owner_hp_gt_two" => Ok(ModifierValueSource::OwnerHpGtTwo),
        "owner_hp_le_two" => Ok(ModifierValueSource::OwnerHpLeTwo),
        "owner_lost_hp_at_least_two" => Ok(ModifierValueSource::OwnerLostHpAtLeastTwo),
        "owner_all_others_in_attack_range" => Ok(ModifierValueSource::OwnerAllOthersInAttackRange),
        "owner_current_turn_all_others_distance_at_most_one" => {
            Ok(ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne)
        }
        "owner_sha_used_zero" => Ok(ModifierValueSource::OwnerShaUsedZero),
        "owner_sha_used_nonzero" => Ok(ModifierValueSource::OwnerShaUsedNonzero),
        "owner_sha_used_exactly_one" => Ok(ModifierValueSource::OwnerShaUsedExactlyOne),
        "owner_sha_used_exactly_two" => Ok(ModifierValueSource::OwnerShaUsedExactlyTwo),
        "owner_max_hand_before_skill_fires_this_turn" => {
            Ok(ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn)
        }
        "owner_non_damage_hand_type_count" => Ok(ModifierValueSource::OwnerNonDamageHandTypeCount),
        "owner_sha_hand_count" => Ok(ModifierValueSource::OwnerShaHandCount),
        "owner_turn_damage_dealt" => Ok(ModifierValueSource::OwnerTurnDamageDealt),
        "owner_distinct_hand_card_suits_used_this_turn" => {
            Ok(ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn)
        }
        "owner_current_turn_card_use_count" => {
            Ok(ModifierValueSource::OwnerCurrentTurnCardUseCount)
        }
        "owner_play_phase_card_use_count_zero" => {
            Ok(ModifierValueSource::OwnerPlayPhaseCardUseCountZero)
        }
        value
            if value
                .strip_prefix("owner_skill_play_phase_fire_count_below_")
                .and_then(|limit| limit.parse::<u32>().ok())
                .is_some_and(|limit| limit > 0) =>
        {
            Ok(ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(
                value
                    .strip_prefix("owner_skill_play_phase_fire_count_below_")
                    .and_then(|limit| limit.parse::<u32>().ok())
                    .expect("validated positive play-phase fire-count limit"),
            ))
        }
        "owner_has_heart_equip" => Ok(ModifierValueSource::OwnerHasHeartEquip),
        "alive_other_qun_count" => Ok(ModifierValueSource::AliveOtherQunCount),
        "owner_has_weapon" => Ok(ModifierValueSource::OwnerHasWeapon),
        "owner_no_weapon" => Ok(ModifierValueSource::OwnerNoWeapon),
        "owner_has_armor_equip" => Ok(ModifierValueSource::OwnerHasArmorEquip),
        "round_number_capped_five" => Ok(ModifierValueSource::RoundNumberCappedFive),
        "round_at_least_alive_qun_count" => Ok(ModifierValueSource::RoundAtLeastAliveQunCount),
        "round_below_alive_qun_count" => Ok(ModifierValueSource::RoundBelowAliveQunCount),
        "alive_field_rank_thirteen_player_count" => {
            Ok(ModifierValueSource::AliveFieldRankThirteenPlayerCount)
        }
        "field_mount_card_count_ceil_half" => Ok(ModifierValueSource::FieldMountCardCountCeilHalf),
        "owner_unique_max_hand" => Ok(ModifierValueSource::OwnerUniqueMaxHand),
        "owner_hand_gt_other" => Ok(ModifierValueSource::OwnerHandGreaterThanOther),
        "owner_hand_lt_other" => Ok(ModifierValueSource::OwnerHandLessThanOther),
        "owner_qun_other_shu" => Ok(ModifierValueSource::OwnerQunOtherShu),
        "owner_turned_over_diff_other" => Ok(ModifierValueSource::OwnerTurnedOverDiffOther),
        "owner_hand_equip_gte_other" => Ok(ModifierValueSource::OwnerHandEquipGteOther),
        "owner_sha_damage_dealt_zero_this_turn" => {
            Ok(ModifierValueSource::OwnerShaDamageDealtZeroThisTurn)
        }
        "queried_player_hand_gt_hp_signed" => Ok(ModifierValueSource::QueriedPlayerHandGtHpSigned),
        "owner_linked_diff_other" => Ok(ModifierValueSource::OwnerLinkedDiffOther),
        "queried_player_judge_nonempty_other_than_owner" => {
            Ok(ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner)
        }
        "queried_player_linked_other_than_owner" => {
            Ok(ModifierValueSource::QueriedPlayerLinkedOtherThanOwner)
        }
        "queried_player_linked_first_living_skill_owner" => {
            Ok(ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner)
        }
        "queried_player_other_than_owner" => Ok(ModifierValueSource::QueriedPlayerOtherThanOwner),
        "queried_player_wounded_in_owner_attack_range_other" => {
            Ok(ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther)
        }
        "queried_source_enemy_owner_ally_target_other_than_owner" => {
            Ok(ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner)
        }
        "owner_armor_empty" => Ok(ModifierValueSource::OwnerArmorEmpty),
        "owner_healthy" => Ok(ModifierValueSource::OwnerHealthy),
        "alive_wounded_count_zero_or_at_least_four" => {
            Ok(ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour)
        }
        _ => Err(format!("unknown modifier value source {source}")),
    }
}

fn mod_value_source_tag(source: ModifierValueSource) -> String {
    match source {
        ModifierValueSource::QueriedPlayerMarkPositive(mark) => {
            return format!("queried_player_mark_positive_{mark}");
        }
        ModifierValueSource::OwnerMark(mark) => return format!("owner_mark_{mark}"),
        ModifierValueSource::OwnerSkillPileCount => {
            return format!("owner_skill_pile_count");
        }
        ModifierValueSource::OwnerSkillPileCountNegative(skill_def) => {
            return format!("owner_skill_pile_count_negative_{skill_def}");
        }
        ModifierValueSource::OwnerSkillPileCountOf(skill_def) => {
            return format!("owner_skill_pile_count_of_{skill_def}");
        }
        ModifierValueSource::AliveWoundedCountAtLeast(threshold) => {
            return format!("alive_wounded_count_at_least_{threshold}");
        }
        ModifierValueSource::OwnerEquipCountAtLeast(threshold) => {
            return format!("owner_equip_count_at_least_{threshold}");
        }
        ModifierValueSource::OwnerEquipCountFloor(threshold) => {
            return format!("owner_equip_count_floor_{threshold}");
        }
        ModifierValueSource::OwnerHp => "owner_hp",
        ModifierValueSource::OwnerMaxHp => "owner_max_hp",
        ModifierValueSource::OwnerHpMinusOne => "owner_hp_minus_one",
        ModifierValueSource::OwnerLostHp => "owner_lost_hp",
        ModifierValueSource::OwnerLostHpMinusOne => "owner_lost_hp_minus_one",
        ModifierValueSource::OwnerLostHpMinOne => "owner_lost_hp_min_one",
        ModifierValueSource::OwnerSkillMark => "owner_skill_mark",
        ModifierValueSource::OwnerSkillPileCount => "owner_skill_pile_count",
        ModifierValueSource::OwnerSkillStateU32 => "owner_skill_state_u32",
        ModifierValueSource::OwnerSkillStateU32Negative => "owner_skill_state_u32_negative",
        ModifierValueSource::QueriedPlayerQunOwnerLordAlive => {
            "queried_player_qun_owner_lord_alive"
        }
        ModifierValueSource::OwnerLinked => "owner_linked",
        ModifierValueSource::OwnerSkillUseCount => "owner_skill_use_count",
        ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne => {
            "alive_wounded_or_guiming_wu_count_min_one"
        }
        ModifierValueSource::AliveGroupCount => "alive_group_count",
        ModifierValueSource::AliveFemaleCount => "alive_female_count",
        ModifierValueSource::AliveOtherCountCappedFour => "alive_other_count_capped_four",
        ModifierValueSource::OwnerNoMount => "owner_no_mount",
        ModifierValueSource::OwnerHandEmpty => "owner_hand_empty",
        ModifierValueSource::OwnerHasTreasure => "owner_has_treasure",
        ModifierValueSource::AliveMountedPlayerCountMinOne => "alive_mounted_player_count_min_one",
        ModifierValueSource::OwnerEquipCountOdd => "owner_equip_count_odd",
        ModifierValueSource::OwnerEquipCountEven => "owner_equip_count_even",
        ModifierValueSource::OwnerEquipCountHalfFloor => "owner_equip_count_half_floor",
        ModifierValueSource::OwnerEquipCountExactlyOne => "owner_equip_count_exactly_one",
        ModifierValueSource::OwnerEquipCount => "owner_equip_count",
        ModifierValueSource::OwnerHpGtTwo => "owner_hp_gt_two",
        ModifierValueSource::OwnerHpLeTwo => "owner_hp_le_two",
        ModifierValueSource::OwnerLostHpAtLeastTwo => "owner_lost_hp_at_least_two",
        ModifierValueSource::OwnerAllOthersInAttackRange => "owner_all_others_in_attack_range",
        ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne => {
            "owner_current_turn_all_others_distance_at_most_one"
        }
        ModifierValueSource::OwnerShaUsedZero => "owner_sha_used_zero",
        ModifierValueSource::OwnerShaUsedNonzero => "owner_sha_used_nonzero",
        ModifierValueSource::OwnerShaUsedExactlyOne => "owner_sha_used_exactly_one",
        ModifierValueSource::OwnerShaUsedExactlyTwo => "owner_sha_used_exactly_two",
        ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn => {
            "owner_max_hand_before_skill_fires_this_turn"
        }
        ModifierValueSource::OwnerNonDamageHandTypeCount => "owner_non_damage_hand_type_count",
        ModifierValueSource::OwnerShaHandCount => "owner_sha_hand_count",
        ModifierValueSource::OwnerTurnDamageDealt => "owner_turn_damage_dealt",
        ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn => {
            "owner_distinct_hand_card_suits_used_this_turn"
        }
        ModifierValueSource::OwnerCurrentTurnCardUseCount => "owner_current_turn_card_use_count",
        ModifierValueSource::OwnerPlayPhaseCardUseCountZero => {
            "owner_play_phase_card_use_count_zero"
        }
        ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(limit) => {
            return format!("owner_skill_play_phase_fire_count_below_{limit}");
        }
        ModifierValueSource::OwnerHasHeartEquip => "owner_has_heart_equip",
        ModifierValueSource::AliveOtherQunCount => "alive_other_qun_count",
        ModifierValueSource::OwnerHasWeapon => "owner_has_weapon",
        ModifierValueSource::OwnerNoWeapon => "owner_no_weapon",
        ModifierValueSource::OwnerHasArmorEquip => "owner_has_armor_equip",
        ModifierValueSource::RoundNumberCappedFive => "round_number_capped_five",
        ModifierValueSource::RoundAtLeastAliveQunCount => "round_at_least_alive_qun_count",
        ModifierValueSource::RoundBelowAliveQunCount => "round_below_alive_qun_count",
        ModifierValueSource::AliveFieldRankThirteenPlayerCount => {
            "alive_field_rank_thirteen_player_count"
        }
        ModifierValueSource::FieldMountCardCountCeilHalf => "field_mount_card_count_ceil_half",
        ModifierValueSource::OwnerUniqueMaxHand => "owner_unique_max_hand",
        ModifierValueSource::OwnerHandGreaterThanOther => "owner_hand_gt_other",
        ModifierValueSource::OwnerHandLessThanOther => "owner_hand_lt_other",
        ModifierValueSource::OwnerQunOtherShu => "owner_qun_other_shu",
        ModifierValueSource::OwnerTurnedOverDiffOther => "owner_turned_over_diff_other",
        ModifierValueSource::OwnerHandEquipGteOther => "owner_hand_equip_gte_other",
        ModifierValueSource::OwnerShaDamageDealtZeroThisTurn => {
            "owner_sha_damage_dealt_zero_this_turn"
        }
        ModifierValueSource::QueriedPlayerHandGtHpSigned => "queried_player_hand_gt_hp_signed",
        ModifierValueSource::OwnerLinkedDiffOther => "owner_linked_diff_other",
        ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner => {
            "queried_player_judge_nonempty_other_than_owner"
        }
        ModifierValueSource::QueriedPlayerLinkedOtherThanOwner => {
            "queried_player_linked_other_than_owner"
        }
        ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner => {
            "queried_player_linked_first_living_skill_owner"
        }
        ModifierValueSource::QueriedPlayerOtherThanOwner => "queried_player_other_than_owner",
        ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther => {
            "queried_player_wounded_in_owner_attack_range_other"
        }
        ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner => {
            "queried_source_enemy_owner_ally_target_other_than_owner"
        }
        ModifierValueSource::OwnerArmorEmpty => "owner_armor_empty",
        ModifierValueSource::OwnerHealthy => "owner_healthy",
        ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour => {
            "alive_wounded_count_zero_or_at_least_four"
        }
    }
    .into()
}

fn map_event(s: &str) -> Option<EventKind> {
    Some(match s {
        "game_started" => EventKind::GameStarted,
        "damage_before" => EventKind::DamageBefore,
        "damage_prevented" => EventKind::DamagePrevented,
        "damage" => EventKind::Damage,
        "hp_lost" => EventKind::HpLost,
        "card_used" | "use_card" => EventKind::CardUsed,
        "card_targeted" => EventKind::CardTargeted,
        "card_responded" | "respond" => EventKind::CardResponded,
        "card_resolved" => EventKind::CardResolved,
        "phase_prepare" => EventKind::PhasePrepare,
        "phase_begin" => EventKind::PhaseBegin,
        "phase_judge" => EventKind::PhaseJudge,
        "phase_draw" => EventKind::PhaseDraw,
        "phase_play" => EventKind::PhasePlay,
        "phase_discard" => EventKind::PhaseDiscard,
        "phase_end" => EventKind::PhaseEnd,
        "phase_draw_end" => EventKind::PhaseDrawEnd,
        "debate_resolved" => EventKind::DebateResolved,
        "debate_started" => EventKind::DebateStarted,
        "phase_play_end" => EventKind::PhasePlayEnd,
        "phase_discard_end" => EventKind::PhaseDiscardEnd,
        "phase_any_end" => EventKind::PhaseAnyEnd,
        "phase_draw_skipped" => EventKind::PhaseDrawSkipped,
        "phase_any_skipped" => EventKind::PhaseAnySkipped,
        "dying" => EventKind::Dying,
        "dying_after" => EventKind::DyingAfter,
        "max_hp_changed" => EventKind::MaxHpChanged,
        "death" => EventKind::Death,
        "cards_drawn" | "draw" => EventKind::CardsDrawn,
        "cards_moved" => EventKind::CardsMoved,
        "cards_given" => EventKind::CardsGiven,
        "skill_added" => EventKind::SkillAdded,
        "turn_ended" => EventKind::TurnEnded,
        "round_started" => EventKind::RoundStart,
        "round_ended" => EventKind::RoundEnd,
        "hp_recovered" => EventKind::HpRecovered,
        "recover_before" => EventKind::RecoverBefore,
        "duel_resolved" => EventKind::DuelResolved,
        "equip_slot_disabled" => EventKind::EquipSlotDisabled,
        "equip_slot_enabled" => EventKind::EquipSlotEnabled,
        "character_state_changed" => EventKind::CharacterStateChanged,
        "sha_dodged" => EventKind::ShaDodged,
        "sha_resolved" => EventKind::ShaResolved,
        "hand_compare_started" => EventKind::HandCompareStarted,
        "hand_compare_finished" => EventKind::HandCompareFinished,
        "judgment_resolved" => EventKind::JudgmentResolved,
        _ => return None,
    })
}

fn map_filter(s: &str) -> Option<TriggerFilter> {
    // Skill-provenance family (Phase 3): `event_view_as_skill_<DEF>` — event card originated from
    // view-as skill with the given definition id (analogue of upstream `event.skill == "X"`).
    // Name-matched variant `event_view_as_skill_name_<NAME>` covers dynamic _backup helpers without
    // a static def id; checked first so it isn't shadowed by the DEF branch.
    if let Some(name) = s.strip_prefix("event_view_as_skill_name_") {
        if !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Some(TriggerFilter::EventViewAsSkillName(name.to_string()));
        }
    }
    if let Some(def) = s.strip_prefix("event_view_as_skill_") {
        if let Ok(skill_def) = def.parse::<u32>() {
            if skill_def > 0 {
                return Some(TriggerFilter::EventViewAsSkill(skill_def));
            }
        }
    }
    // Card-name-scoped family (Phase 3): `owner_source_card_<selector>`,
    // `owner_is_target_card_<selector>`, `owner_is_target_card_damage_<selector>` where
    // `<selector>` is a card-selector name (sha, nanman, juedou, trick, ...) mapped by
    // map_card_selector. Unlocks card_name_scoped placeholders.
    // NOTE: do NOT `return None` here when the selector is unknown — fall through so that
    // pre-existing filters sharing the prefix (e.g. owner_is_target_card_in_processing) still
    // resolve via the later branches.
    if let Some(selector) = s.strip_prefix("owner_source_card_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerSourceCardName(card));
        }
    }
    if let Some(selector) = s.strip_prefix("owner_is_target_card_damage_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerIsTargetCardDamage(card));
        }
    }
    if let Some(selector) = s.strip_prefix("owner_is_target_card_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerIsTargetCardName(card));
        }
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_sha_skill_pile_nonempty_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsTargetShaSkillPileNonempty);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_other_play_discard_skill_pile_nonempty_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty);
    }
    if let Some(skill_def) =
        s.strip_prefix("owner_source_own_view_as_card_targeted_target_skill_pile_")
    {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_card_targeted_marked_peer_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveCardTargetedMarkedPeer);
    }
    if let Some(mark) = s.strip_prefix("owner_source_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceTargetMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_source_play_skill_mark_mode_") {
        return mark
            .parse::<u32>()
            .ok()
            .filter(|mark| *mark > 0)
            .map(TriggerFilter::OwnerSourcePlaySkillMarkMode);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_targeted_by_other_hidden_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_hand_nonempty_hidden_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_phase_begin_skill_pile_gte_three_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerPhaseBeginSkillPileGteThree);
    }
    if let Some(skill_def) = s.strip_prefix("owner_lost_matching_revealed_skill_pile_suit_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit);
    }
    if s == "owner_source_play_card_count_eq_hp" {
        return Some(TriggerFilter::OwnerSourcePlayCardCountEqualsHp);
    }
    if let Some(count) = s.strip_prefix("owner_source_play_card_count_eq_") {
        return count
            .parse::<u32>()
            .ok()
            .filter(|count| *count > 0)
            .map(TriggerFilter::OwnerSourcePlayCardCountEquals);
    }
    if let Some(mark) = s.strip_prefix("owner_is_target_source_alive_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .filter(|mark| *mark > 0)
            .map(TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_any_field_card_no_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_source_sha_hand_equip_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_dying_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveDyingSkillPile);
    }
    if let Some(threshold) = s
        .strip_prefix("owner_is_target_active_used_at_least_")
        .and_then(|value| value.strip_suffix("_this_play_phase"))
    {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(TriggerFilter::OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase);
    }
    if let Some(threshold) = s.strip_prefix("owner_is_target_hp_below_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(TriggerFilter::OwnerIsEventPlayerHpBelow);
    }
    if let Some(mark) = s.strip_prefix("owner_is_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerIsEventPlayerMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_skill_added_self_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSkillAddedSelf);
    }
    if s == "owner_source_first_card_suit_this_turn" {
        return Some(TriggerFilter::OwnerSourceFirstCardSuitThisTurn);
    }
    if s == "owner_lost_hand_now_empty" {
        return Some(TriggerFilter::OwnerLostHandNowEmpty);
    }
    if s == "owner_lost_spade_two_to_nine" {
        return Some(TriggerFilter::OwnerLostSpadeTwoToNine);
    }
    if let Some(mark) = s.strip_prefix("owner_source_ordinary_trick_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceOrdinaryTrickMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_source_sha_or_black_trick_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceShaOrBlackTrickMarkPositive);
    }
    if let Some(value) = s
        .strip_prefix("owner_source_damage_target_mark_positive_")
        .and_then(|v| v.strip_suffix("_once"))
    {
        return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_marked_alive_above_two_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveMarkedAliveAboveTwo);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_event_target_mark_positive_") {
        if let Ok(value) = mark.parse::<u32>() {
            return Some(TriggerFilter::OwnerAliveEventTargetMarkPositive(value));
        }
        // non-numeric remainder (e.g. "..._no_damage") falls through to the
        // suffix-aware arm registered below.
    }
    if let Some(value) = s.strip_prefix("owner_source_hand_gaintag_absent_") {
        return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceHandGaintagAbsent);
    }
    if s == "owner_alive_play_end_used_categories_positive" {
        return Some(TriggerFilter::OwnerAlivePlayEndUsedCategoriesPositive);
    }
    if let Some(value) = s
        .strip_prefix("owner_alive_event_target_mark_positive_")
        .and_then(|value| value.strip_suffix("_no_damage"))
    {
        eprintln!("DBG B no_damage matched"); return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_someone_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveSomeoneMarkPositive);
    }
    if let Some(pile_def) = s.strip_prefix("owner_alive_discard_matches_own_pile_card_") {
        return pile_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard);
    }    if s == "cards_moved_into_expansion" {
        return Some(TriggerFilter::CardsMovedIntoExpansion);
    }
    if s == "cards_moved_out_of_expansion" {
        return Some(TriggerFilter::CardsMovedOutOfExpansion);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_damage_before_event_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveDamageBeforeEventTargetMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_phase_begin_event_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAlivePhaseBeginEventTargetMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_sha_resolved_city_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveShaResolvedCity);
    }
    if let Some(group) = s
        .strip_prefix("owner_alive_other_shan_outside_turn_group_")
        .and_then(|value| value.strip_suffix("_once_per_turn"))
    {
        return valid_group_filter_tag(group).then(|| {
            TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(group.to_owned())
        });
    }
    if let Some(group) = s.strip_prefix("owner_alive_event_player_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerAliveEventPlayerGroup(group.to_owned()));
    }
    // clan 中流 family: owner is the event player and no OTHER living member of <clan>
    // lost hand cards this turn. <clan> is an arbitrary (CJK) clan name, so the suffix is
    // stripped rather than validated as a group tag.
    if let Some(rest) = s.strip_prefix("owner_is_event_player_no_") {
        if let Some(clan) = rest.strip_suffix("_lost_hand_this_turn") {
            if !clan.is_empty() {
                return Some(TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(
                    clan.to_owned(),
                ));
            }
        }
    }
    // 保族 (clanbaozu) family: event player is the owner or a member of <clan>.
    // <clan> is an arbitrary (CJK) clan name.
    if let Some(clan) = s.strip_prefix("owner_alive_event_player_self_or_clan_") {
        if !clan.is_empty() {
            return Some(TriggerFilter::OwnerAliveEventPlayerSelfOrClan(clan.to_owned()));
        }
    }
    // 神君 (clanshenjun) family: same-kind hand reveal gate and tagged-card
    // phase-end use gate.
    if s == "owner_alive_hand_same_kind_event_card" {
        return Some(TriggerFilter::OwnerAliveHandHasSameKindAsEventCard);
    }
    if s == "owner_alive_has_skill_tagged_cards" {
        return Some(TriggerFilter::OwnerAliveHasSkillTaggedCards);
    }
    if s == "owner_alive_tagged_card_lost_this_turn" {
        return Some(TriggerFilter::OwnerAliveTaggedCardLostThisTurn);
    }
    if let Some(group) = s.strip_prefix("owner_source_target_not_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerSourceTargetNotGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_alive_other_damage_source_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerAliveOtherDamageSourceGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_dying_other_tao_responder_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerDyingOtherTaoResponderGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_alive_other_judgment_black_event_player_group_") {
        return valid_group_filter_tag(group).then(|| {
            TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(group.to_owned())
        });
    }
    if let Some(group) = s.strip_prefix("owner_wounded_event_player_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerWoundedEventPlayerGroup(group.to_owned()));
    }
    if let Some(mark) = s.strip_prefix("owner_gains_outside_turn_mark_below_alive_once_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn);
    }
    if let Some(skill_def) = s
        .strip_prefix("owner_source_jiu_boosted_sha_damage_skill_")
        .and_then(|value| value.strip_suffix("_enabled"))
    {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(SkillDefId::new)
            .map(TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled);
    }
    Some(match s {
        "owner_is_target" | "OwnerIsEventPlayer" | "owner_is_event_player" => {
            TriggerFilter::OwnerIsEventPlayer
        }
        "owner_source" => TriggerFilter::OwnerIsEventSource,
        "owner_is_target_not_unique_max_current_hp" => {
            TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp
        }
        "owner_is_target_wounded" => TriggerFilter::OwnerIsEventPlayerWounded,
        "owner_is_target_linked" => TriggerFilter::OwnerIsEventPlayerLinked,
        "owner_is_target_chain_damage_linked" => TriggerFilter::OwnerIsTargetChainDamageLinked,
        "owner_is_target_wounded_sha" => TriggerFilter::OwnerIsTargetWoundedSha,
        "owner_is_target_healthy_draw_not_fixed" => {
            TriggerFilter::OwnerIsTargetHealthyDrawNotFixed
        }
        "owner_is_target_alive_wounded_or_guiming_wu_exists" => {
            TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists
        }
        "owner_is_target_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerHandNonempty,
        "owner_is_target_hand_below_max_hp" => TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp,
        "owner_is_target_first_hp_change_turn_unique_trick_majority" => {
            TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
        }
        "owner_is_target_hand_above_hp" => TriggerFilter::OwnerIsEventPlayerHandAboveHp,
        "owner_is_target_hand_at_least_hp_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive
        }
        "owner_is_target_lost_cards_at_most_hp_gainable" => {
            TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
        }
        "owner_is_target_source_movable_field_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard
        }
        "owner_is_target_no_turn_damage" => TriggerFilter::OwnerIsEventPlayerNoTurnDamage,
        "owner_is_target_turn_damage" => TriggerFilter::OwnerIsEventPlayerTurnDamage,
        "owner_is_target_turn_damage_at_least_three" => {
            TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree
        }
        "owner_source_target_distance_gt_one" => TriggerFilter::OwnerSourceTargetDistanceGtOne,
        "owner_is_target_card_used_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp
        }
        "owner_is_target_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive
        }
        "owner_alive_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive
        }
        "owner_is_target_other_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerOtherHandNonempty,
        "owner_is_target_positive_draw_other_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty
        }
        "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip
        }
        "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable" => {
            TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable
        }
        "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile" => {
            TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile
        }
        "owner_is_source_or_target_xinyaoming_available" => {
            TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable
        }
        "owner_source_play_card_matches_remembered_category" => {
            TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory
        }
        "owner_source_damage_card_once_per_turn" => TriggerFilter::OwnerSourceDamageCardOncePerTurn,
        "owner_alive_event_player_hand_nonempty_once_per_round" => {
            TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
        }
        "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
            TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused
        }
        "owner_source_sha_damage_before_potyinzhan_condition" => {
            TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition
        }
        "owner_source_damage_within_one" => TriggerFilter::OwnerSourceDamageWithinOne,
        "owner_gained_cards_potzhouyu" => TriggerFilter::OwnerGainedCardsPotzhouyu,
        "owner_source_first_target_has_hand_potyanhui" => {
            TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui
        }
        "owner_alive_fire_damage_potfentao" => TriggerFilter::OwnerAliveFireDamagePotfentao,
        "owner_alive_potfentao_pending_damage" => TriggerFilter::OwnerAlivePotfentaoPendingDamage,
        "owner_alive_turn_ended_rehengjiang_pending" => {
            TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
        }
        "owner_alive_phase_play_end_zhengbi_pending" => {
            TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
        }
        "owner_is_target_source_hand_nonempty_per_damage_point" => {
            TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
        }
        "owner_source_qiaojian_extra_available" => TriggerFilter::OwnerSourceQiaojianExtraAvailable,
        "owner_xicha_watched_source_card_used" => TriggerFilter::OwnerXichaWatchedSourceCardUsed,
        "owner_qiaojian_tracked_card_left_hand" => TriggerFilter::OwnerQiaojianTrackedCardLeftHand,
        "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
            TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard
        }
        "owner_source_wounded_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceWoundedExistsOncePerTurn
        }
        "owner_is_target_discardable_hand_equip_gte_two_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
        }
        "owner_is_target_other_source_alive_hand_equip_nonempty_oldhuisheng_unused" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused
        }
        "owner_alive_other_dying_outside_turn_can_sha_current" => {
            TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent
        }
        "owner_source_oldjiefan_sha" => TriggerFilter::OwnerSourceOldJiefanSha,
        "owner_source_current_turn_first_target_non_equip" => {
            TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip
        }
        "owner_source_face_down_view_as_chenwei_available" => {
            TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable
        }
        "owner_source_bound_category_first_three_play_phase" => {
            TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase
        }
        "owner_phase_gift_target_is_source" => TriggerFilter::OwnerPhaseGiftTargetIsEventSource,
        "owner_phase_gift_target_is_target" => TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer,
        "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
            TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient
        }
        "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        }
        "owner_is_target_tied_max_hand" => TriggerFilter::OwnerIsEventPlayerTiedMaxHand,
        "owner_is_target_tied_max_hp" => TriggerFilter::OwnerIsEventPlayerTiedMaxHp,
        "owner_is_target_tied_max_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty
        }
        "owner_is_target_skipped_phase_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive
        }
        "owner_discarded_has_hand_once_per_phase" => {
            TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        }
        "owner_alive_lost_hand_this_turn" => TriggerFilter::OwnerAliveLostHandThisTurn,
        "owner_alive_phase_end_lost_cards_at_least_two" => {
            TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
        },
        "owner_source_first_sha_this_play_phase" => TriggerFilter::OwnerSourceFirstShaThisPlayPhase,
        "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
            TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        }
        "owner_source_first_sha_target_this_turn" => {
            TriggerFilter::OwnerSourceFirstShaTargetThisTurn
        }
        "owner_alive_other_phase_play_end_twzhengjian_condition" => {
            TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
        }
        "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
            TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        }
        "owner_source_first_target_damage_card_once_per_turn" => {
            TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        }
        "owner_source_first_target_hand_damage_card_discardable" => {
            TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable
        }
        "owner_lost_hand_ownerless_sha_not_use_once_per_turn" => {
            TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn
        }
        "owner_alive_phase_any_end_originating_skill_pile_nonempty" => {
            TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty
        }
        "owner_source_damage_tracked_mbduanyang_target_recastable" => {
            TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable
        }
        "owner_mbrunwei_loss_threshold" => TriggerFilter::OwnerMbrunweiLossThreshold,
        "owner_phase_play_end_mbrunwei_tracked_hand" => {
            TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand
        }
        "owner_alive_damage_before_target_distance_le_one_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn
        }
        "owner_source_damage_mbxiongtu_armed" => TriggerFilter::OwnerSourceDamageMbxiongtuArmed,
        "owner_phase_play_end_mbxiongtu_state" => TriggerFilter::OwnerPhasePlayEndMbxiongtuState,
        "owner_phase_end_mbganggeng_target_alive" => {
            TriggerFilter::OwnerPhaseEndMbganggengTargetAlive
        }
        "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
            TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        }
        "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
            TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        }
        "owner_source_first_single_target_sha_mbfeijing_twice_per_turn" => {
            TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn
        }
        "owner_source_damage_before_mbfeijing_extra" => {
            TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra
        }
        "owner_source_sha_resolved_original_single_mbxiaoge" => {
            TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge
        }
        "owner_alive_damage_target_min_hand_mbxuye_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn
        }
        "owner_mbkuangxiang_received_cards_exhausted" => {
            TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted
        }
        "owner_phase_play_mbkuangxiang_active" => TriggerFilter::OwnerPhasePlayMbkuangxiangActive,
        "owner_gained_cards_potduzuo" => TriggerFilter::OwnerGainedCardsPotduzuo,
        "owner_gained_cards_potduzuo_once_per_turn" => {
            TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn
        }
        "owner_sha_damage_before_potbihan_available" => {
            TriggerFilter::OwnerShaDamageBeforePotbihanAvailable
        }
        "owner_alive_card_targeted_healthy" => TriggerFilter::OwnerAliveCardTargetedHealthy,
        "owner_alive_card_resolved_source_current_potsifeng_pile" => {
            TriggerFilter::OwnerAliveCardResolvedSourceCurrentPotsifengPile
        }
        "owner_alive_phase_end_player_potsifeng_pile" => {
            TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile
        }
        "owner_alive_potdingpin_available_once_per_round" => {
            TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
        }
        "owner_outside_turn_discarded_red_cards_in_discard" => {
            TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard
        }
        "owner_alive_other_twkujian_used_or_responded" => {
            TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded
        }
        "owner_alive_other_twkujian_lost" => TriggerFilter::OwnerAliveOtherTwkujianLost,
        "owner_alive_twruilian_round_unprompted" => {
            TriggerFilter::OwnerAliveTwruilianRoundUnprompted
        }
        "owner_alive_twruilian_selected_target_discarded" => {
            TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded
        }
        "owner_alive_twruilian_selected_target_turn_ended" => {
            TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded
        }
        "owner_alive_lost_weapon_outside_turn" => TriggerFilter::OwnerAliveLostWeaponOutsideTurn,
        "owner_alive_other_phase_prepare_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner
        }
        "owner_is_target_discarded_at_least_two_in_discard_phase" => {
            TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
        }
        "owner_alive_other_discarded_in_discard_phase" => {
            TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
        }
        "owner_alive_other_wu_play_phase_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner
        }
        "owner_alive_other_play_phase_zhengsu_missing_in_range" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange
        }
        "owner_is_target_missing_zhengsu" => TriggerFilter::OwnerIsTargetMissingZhengsu,
        "owner_source_club" => TriggerFilter::OwnerSourceClub,
        "owner_alive_other_play_phase_discardable_gte_mark" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark
        }
        "owner_alive_other_trick_first_target_source_max_hand" => {
            TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand
        }
        "owner_alive_other_marked_xi" => TriggerFilter::OwnerAliveOtherMarkedXi,
        "owner_alive_other_recovered_outside_owner_turn" => {
            TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn
        }
        "owner_is_target_discarded_sha_in_discard_phase" => {
            TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
        }
        "owner_source_sha_damage_before_target_healthy" => {
            TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy
        }
        "owner_phase_end_originating_skill_pile_empty" => {
            TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty
        }
        "owner_prepare_group_outside_attack_range_exists" => {
            TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists
        }
        "owner_is_target_single_target_other_source_once_per_turn" => {
            TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn
        }
        "owner_lost_one_hand_equip_judge_other_current_phase_once" => {
            TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce
        }
        "owner_is_target_multi_target_trick_has_hand_equip" => {
            TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip
        }
        "owner_is_target_feijun_new_target" => TriggerFilter::OwnerIsTargetFeijunNewTarget,
        "owner_is_target_discard_phase_owe_limit_other_qun" => {
            TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun
        }
        "owner_source_sha_damage_target_alive_both_hand" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand
        }
        "owner_alive_other_nanman_cards_discard" => {
            TriggerFilter::OwnerAliveOtherNanmanCardsDiscard
        }
        "owner_alive_skill_bound_target_is_event_source" => {
            TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource
        }
        "owner_lost_cards_outside_turn" => TriggerFilter::OwnerLostCardsOutsideTurn,
        "owner_gained_cards_during_play_phase_wounded" => {
            TriggerFilter::OwnerGainedCardsDuringPlayPhaseWounded
        }
        "owner_discarded_outside_play_phase" => TriggerFilter::OwnerDiscardedOutsidePlayPhase,
        "owner_in_multitarget_sha_or_trick_targets" => {
            TriggerFilter::OwnerInMultitargetShaOrTrickTargets
        }
        "owner_targeted_by_nanman" => TriggerFilter::OwnerTargetedByNanman,
        "owner_targeted_source_hp_hand_differ_once_per_turn" => {
            TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn
        }
        "owner_source_trick_resolved_own_play_phase_once_per_turn" => {
            TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn
        }
        "owner_alive_other_dying_has_equip_card" => {
            TriggerFilter::OwnerAliveOtherDyingHasEquipCard
        }
        "owner_alive_damage_card_nanman" => TriggerFilter::OwnerAliveDamageCardNanman,
        "owner_discard_suit_no_own_mark_spade_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(1, 3_540_560_359)
        }
        "owner_discard_suit_no_own_mark_heart_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(2, 3_540_560_360)
        }
        "owner_discard_suit_no_own_mark_club_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(3, 3_540_560_361)
        }
        "owner_discard_suit_no_own_mark_diamond_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(4, 3_540_560_362)
        }
        "owner_black_card_moved_equip_judge" => TriggerFilter::OwnerBlackCardMovedEquipJudge,
        "owner_lost_non_damage_card_pottuntian_charge_not_full" => {
            TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull
        }
        "owner_source_sha_resolved_tracked_potceduan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage
        }
        "owner_alive_sha_entered_discard_potzhanlie_available" => {
            TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable
        }
        "owner_source_sha_resolved_tracked_potzhanlie_draw" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw
        }
        "owner_phase_begin_skill_pile_gte_three" => {
            TriggerFilter::OwnerPhaseBeginSkillPileGteThree(0)
        }
        "owner_phase_begin_hand_empty" => TriggerFilter::OwnerPhaseBeginHandEmpty,
        "owner_responded_shan" => TriggerFilter::OwnerRespondedShan,
        "owner_alive_other_qun_play_phase_can_give_owner" => {
            TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
        }
        "owner_source_first_damage_card_target_hand_gte_owner" => {
            TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner
        }
        "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three" => {
            TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree
        }
        "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused
        }
        "owner_is_target_male_source_has_discardable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment
        }
        "owner_alive_other_turn_ended_no_card_targeted_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther
        }
        "owner_alive_other_turn_ended_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther
        }
        "owner_alive_gained_two_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
        }
        "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther
        }
        "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
        }
        "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
        }
        "owner_source_sha_has_weapon" => TriggerFilter::OwnerSourceShaHasWeapon,
        "owner_source_sha_direct_damage_before" => TriggerFilter::OwnerSourceShaDirectDamageBefore,
        "owner_source_other_damage_before" => TriggerFilter::OwnerSourceOtherDamageBefore,
        "owner_source_other_damage_before_once_per_turn" => {
            TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        }
        "owner_source_other_direct_damage_before" => {
            TriggerFilter::OwnerSourceOtherDirectDamageBefore
        }
        "owner_is_target_discardable_hand_equip" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquip
        }
        "owner_is_target_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo
        }
        "owner_source_or_target_hand_count_differs_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn
        }
        "owner_is_target_any_alive_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty
        }
        "owner_is_target_any_alive_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty
        }
        "owner_is_target_other_hand_lte_owner" => {
            TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner
        }
        "owner_is_target_draw_reshuffle_zero_wounded" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded
        }
        "owner_is_target_draw_reshuffle_positive" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive
        }
        "owner_hand_compare_participant" => TriggerFilter::OwnerHandCompareParticipant,
        "owner_hand_compare_finished_participant" => {
            TriggerFilter::OwnerHandCompareFinishedParticipant
        }
        "owner_hand_compare_participant_has_sha" => {
            TriggerFilter::OwnerHandCompareParticipantHasSha
        }
        "owner_hand_compare_participant_result_card_in_discard" => {
            TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard
        }
        "owner_source_basic" => TriggerFilter::OwnerSourceBasic,
        "owner_source_non_basic" => TriggerFilter::OwnerSourceNonBasic,
        "owner_source_basic_or_trick" => TriggerFilter::OwnerSourceBasicOrTrick,
        "owner_source_wounded_nanman_wanjian_or_normal_spade_sha" => {
            TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha
        }
        "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
        }
        "owner_alive_other_phase_end_damage_gt_one_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip
        }
        "owner_alive_other_phase_end_has_discardable_basic_hand" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand
        }
        "owner_alive_other_phase_end_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty
        }
        "owner_alive_prepare_player_hp_gt_owner" => TriggerFilter::OwnerAlivePreparePlayerHpGtOwner,
        "owner_alive_prepare_lord_hp_gt_owner" => TriggerFilter::OwnerAlivePrepareLordHpGtOwner,
        "owner_alive_other_sha_target_can_redirect_to_owner" => {
            TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner
        }
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange
        }
        "owner_sha_target_can_discard_redirect_to_zhengjian_marked" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty_once_per_turn" => {
            TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn
        }
        "owner_is_target_trick_targets_gt_source_hp_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty
        }
        "owner_is_source_sha_used" => TriggerFilter::OwnerIsSourceShaUsed,
        "owner_is_responder_sha" => TriggerFilter::OwnerIsResponderSha,
        "owner_is_source_sha_card_damage_before" => {
            TriggerFilter::OwnerIsSourceShaCardDamageBefore
        }
        "owner_is_target_source_hp_gt_owner_per_role_turn" => {
            TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn
        }
        "owner_phase_end_lost_cards" => TriggerFilter::OwnerPhaseEndLostCards,
        "owner_alive_second_global_dying" => TriggerFilter::OwnerAliveSecondGlobalDying,
        "owner_used_and_was_targeted_this_turn_in_range" => {
            TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
        }
        "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp,
        "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner,
        "owner_is_target_sanshou_type_tracked" => TriggerFilter::OwnerIsTargetSanshouTypeTracked,
        "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner,
        "owner_is_target_damage_amount_gt_one" => {
            TriggerFilter::OwnerIsTargetDamageAmountGtOne
        }
        "owner_alive_gave_cards_to_other_alive" => TriggerFilter::OwnerAliveGaveCardsToOtherAlive,
        "owner_is_target_can_compare_other" => TriggerFilter::OwnerIsEventPlayerCanCompareOther,
        "owner_alive_play_phase_sha_user_has_discardable_hand_equip" => {
            TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip
        }
        "owner_is_target_use_response_discarded_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
        }
        "owner_is_target_damaged_other_qun" => TriggerFilter::OwnerIsEventPlayerDamagedOtherQun,
        "owner_is_target_source_alive" => TriggerFilter::OwnerIsEventPlayerSourceAlive,
        "owner_alive_other_source_single_sha_target_other" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther
        }
        "owner_alive_other_source_single_sha_target_other_owner_in_attack_range" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange
        }
        "owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_dealt_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_no_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage
        }
        "owner_tracked_skill_card_left_hand_not_use_or_respond" => {
            TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond
        }
        "owner_alive_sha_used_by_self_or_in_range_discardable_hand_equip" => {
            TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip
        }
        "owner_alive_tracked_cuijin_sha_resolved_no_damage" => {
            TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        }
        "owner_alive_tracked_cuijin_sha_resolved_dealt_damage" => {
            TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
        }
        "owner_alive_tracked_twqingkou_duel_resolved" => {
            TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved
        }
        "owner_alive_twchenxun_round_unprompted" => {
            TriggerFilter::OwnerAliveTwchenxunRoundUnprompted
        }
        "owner_alive_tracked_twchenxun_duel_target_damaged" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        }
        "owner_alive_tracked_twchenxun_duel_target_not_damaged" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged
        }
        "owner_alive_tracked_twchenxun_card_left_processing" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing
        }
        "owner_is_target_twxiaolian_redirected_damage" => {
            TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage
        }
        "owner_is_target_other_higher_hand_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
        }
        "owner_is_target_source_alive_once" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        }
        "owner_is_target_source_has_weapon" => TriggerFilter::OwnerIsEventPlayerSourceHasWeapon,
        "owner_is_target_source_alive_disabled_weapon" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon
        }
        "owner_is_target_other_source_alive" => TriggerFilter::OwnerIsEventPlayerOtherSourceAlive,
        "owner_is_target_other_source_alive_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
        }
        "owner_is_target_per_damage_point" => TriggerFilter::OwnerIsEventPlayerPerDamagePoint,
        "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
        }
        "owner_is_target_other_alive" => TriggerFilter::OwnerIsEventPlayerOtherAlive,
        "owner_is_target_source_alive_gainable_card_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
        }
        "owner_is_target_source_alive_gainable_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard
        }
        "owner_is_target_card_in_processing" => TriggerFilter::OwnerIsEventPlayerCardInProcessing,
        "owner_is_target_red_sha_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive
        }
        "owner_is_target_nonred_sha" => TriggerFilter::OwnerIsEventPlayerNonRedSha,
        "owner_is_target_red_or_jiu_sha" => TriggerFilter::OwnerIsEventPlayerRedOrJiuSha,
        "owner_is_target_sha_source_has_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip
        }
        "owner_is_target_red_card_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive
        }
        "owner_is_target_nonred_card" => TriggerFilter::OwnerIsEventPlayerNonRedCard,
        "owner_is_target_first_damage" => TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn,
        "owner_is_target_second_damage" => TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn,
        "owner_is_target_after_first_damage" => {
            TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn
        }
        "owner_source_distance_le_one_per_damage_point" => {
            TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
        }
        "owner_is_respond_to" => TriggerFilter::OwnerIsRespondTo,
        "owner_is_source" | "OwnerIsEventSource" | "owner_is_event_source" => {
            TriggerFilter::OwnerIsEventSource
        }
        "owner_alive_is_source" => TriggerFilter::OwnerAliveIsEventSource,
        "owner_source_target_alive_has_hand_or_equip" => {
            TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip
        }
        "owner_source_other_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_target_alive_owner_outside_target_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange
        }
        "owner_is_source_or_target" => TriggerFilter::OwnerIsEventSourceOrPlayer,
        "owner_is_source_or_target_other_alive_per_damage_point" => {
            TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
        }
        "owner_source_fire" => TriggerFilter::OwnerSourceFire,
        "owner_target_fire" => TriggerFilter::OwnerTargetFire,
        "owner_target_thunder" => TriggerFilter::OwnerTargetThunder,
        "owner_source_thunder" => TriggerFilter::OwnerSourceThunder,
        "owner_source_wounded_distance_le_one" => TriggerFilter::OwnerSourceWoundedDistanceLeOne,
        "owner_alive" => TriggerFilter::OwnerAlive,
        "owner_alive_killed_this_turn" => TriggerFilter::OwnerAliveKilledThisTurn,
        "owner_alive_hand_below_alive_count_capped_seven" => {
            TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven
        }
        "owner_alive_hand_below_alive_count_capped_eight" => {
            TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight
        }
        "owner_alive_delayed_trick_target_without_pending_judge_skip" => {
            TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip
        }
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
        }
        "owner_source_sha_or_juedou_single_target_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty
        }
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip
        }
        "owner_longdan_view_as_first_counterpart_hand_nonempty" => {
            TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty
        }
        "owner_is_source_hand_below_alive_count_capped_seven" => {
            TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven
        }
        "owner_alive_event_player_alive" => TriggerFilter::OwnerAliveEventPlayerAlive,
        "owner_alive_event_player_equip_empty" => TriggerFilter::OwnerAliveEventPlayerEquipEmpty,
        "owner_alive_event_player_from_equip_empty" => {
            TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty
        }
        "owner_alive_sha_target_no_other_red_card_this_turn" => {
            TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
        }
        "owner_alive_event_player_hand_empty" => TriggerFilter::OwnerAliveEventPlayerHandEmpty,
        "owner_alive_event_cards_nonempty" => TriggerFilter::OwnerAliveEventCardsNonempty,
        "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn" => {
            TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn
        }
        "owner_alive_event_player_outside_turn_from_hand_empty" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty
        }
        "owner_alive_event_player_outside_turn_from_hand_empty_once" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        }
        "owner_alive_event_player_turn_changed_or_became_linked" => {
            TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked
        }
        "owner_alive_event_player_became_face_up_or_linked" => {
            TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked
        }
        "owner_alive_event_player_distance_le_owner_hp" => {
            TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp
        }
        "owner_alive_other_normal_damage_target_unlinked" => {
            TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked
        }
        "owner_alive_damaged_event_player" => TriggerFilter::OwnerAliveDamagedEventPlayer,
        "owner_alive_event_player_owner_or_female" => {
            TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale
        }
        "owner_alive_source_target_same_sex" => TriggerFilter::OwnerAliveSourceTargetSameSex,
        "owner_alive_event_player_same_group" => TriggerFilter::OwnerAliveEventPlayerSameGroup,
        "owner_alive_event_player_same_group_hand_nonempty" => {
            TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty
        }
        "owner_tracked_juedou_original_target_damage_recoverable_in_range" => {
            TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange
        }
        "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty" => {
            TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty
        }
        "owner_is_target_ordinary_trick_cilv_available" => {
            TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable
        }
        "cilv_tracked_card_damage_before" => TriggerFilter::CilvTrackedCardDamageBefore,
        "cilv_tracked_card_resolved" => TriggerFilter::CilvTrackedCardResolved,
        "owner_dying_zhupeilan_once_per_game" => TriggerFilter::OwnerDyingZhupeilanOncePerGame,
        "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable
        }
        "owner_phase_end_dcsbhaoyi_available" => TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable,
        "owner_alive_global_fire_damage_dcsbronghuo" => {
            TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo
        }
        "owner_source_fire_sha_or_huogong_dcsbronghuo" => {
            TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo
        }
        "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn" => {
            TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn
        }
        "dcsbyingmou_tracked_card_resolved" => TriggerFilter::DcsbyingmouTrackedCardResolved,
        "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
            TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable
        }
        "dcsbmengmou_tracked_sha_damage" => TriggerFilter::DcsbmengmouTrackedShaDamage,
        "dcsbmengmou_tracked_sha_resolved" => TriggerFilter::DcsbmengmouTrackedShaResolved,
        "owner_is_target_card_damage_before" => TriggerFilter::OwnerIsTargetCardDamageBefore,
        "owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha
        }
        "dcyinjun_tracked_sha_resolved" => TriggerFilter::DcyinjunTrackedShaResolved,
        "owner_wounded_not_event_source" => TriggerFilter::OwnerWoundedNotEventSource,
        "owner_wounded_event_player_qun" => {
            TriggerFilter::OwnerWoundedEventPlayerGroup("qun".into())
        }
        "owner_alive_not_target" => TriggerFilter::OwnerAliveNotEventPlayer,
        "owner_alive_not_target_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty
        }
        "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick" => TriggerFilter::OwnerSourceShaOrOrdinaryTrick,
        "owner_source_wuxie_used" => TriggerFilter::OwnerSourceWuxieUsed,
        "owner_source_sha_once_per_turn" => TriggerFilter::OwnerSourceShaOncePerTurn,
        "owner_is_target_hej_above_hp" => TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp,
        "owner_source_play_card_repeats_previous_targets" => {
            TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
        }
        "owner_source_sha_repeats_previous_sha_targets" => {
            TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
        }
        "owner_equip_zone_changed" => TriggerFilter::OwnerEquipZoneChanged,
        "owner_alive_not_target_neither_unique_max_hand" => {
            TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand
        }
        "owner_alive_not_target_source_same_group" => {
            TriggerFilter::OwnerAliveNotEventPlayerSourceSameGroup
        }
        "owner_alive_not_target_equal_hand" => TriggerFilter::OwnerAliveNotEventPlayerEqualHand,
        "owner_alive_not_target_hand_below_hp" => {
            TriggerFilter::OwnerAliveNotEventPlayerHandBelowHp
        }
        "owner_alive_not_target_nonnegative_hp_gt_target" => {
            TriggerFilter::OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer
        }
        "owner_alive_current_turn_other_event_player" => {
            TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayer
        }
        "owner_alive_current_turn" => TriggerFilter::OwnerAliveCurrentTurn,
        "owner_is_target_current_turn" => TriggerFilter::OwnerIsEventPlayerCurrentTurn,
        "owner_alive_group_count_at_most_2" => TriggerFilter::OwnerAliveGroupCountAtMostTwo,
        "debate_owner_is_initiator" => TriggerFilter::DebateOwnerIsInitiator,
        "debate_owner_participated" => TriggerFilter::DebateOwnerParticipated,
        "owner_debate_participant" => TriggerFilter::OwnerDebateParticipant,
        "owner_cards_moved_contain_ying" => TriggerFilter::OwnerCardsMovedContainYing,
        "owner_is_lord" => TriggerFilter::OwnerIsLord,
        "owner_alive_other_targeted_by_sha" => TriggerFilter::OwnerAliveOtherTargetedBySha,
        "owner_alive_event_player_lord" => TriggerFilter::OwnerAliveEventPlayerLord,
        "owner_alive_event_player_lord_no_damage" => TriggerFilter::OwnerAliveEventPlayerLordNoDamage,
        "owner_alive_event_player_lord_damaged" => TriggerFilter::OwnerAliveEventPlayerLordDamaged,
        "owner_alive_turn_used_all_suits" => TriggerFilter::OwnerAliveTurnUsedAllSuits,
        "cards_moved_discard_at_least_three" => TriggerFilter::CardsMovedDiscardAtLeastThree,
        "owner_alive_targeted_by_other_sha_judge_xumou" => {
            TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou
        }
        "debate_owner_opinion_differs_initiator" => TriggerFilter::DebateOwnerOpinionDiffersInitiator,
        "debate_owner_is_initiator_result_red" => TriggerFilter::DebateOwnerInitiatorResultRed,
        "owner_alive_other_source_damage" => TriggerFilter::OwnerAliveOtherSourceDamage,
        "debate_owner_is_initiator_result_black" => TriggerFilter::DebateOwnerInitiatorResultBlack,
        "owner_lost_package_card_from_equip" => TriggerFilter::OwnerLostPackageCardFromEquip,
        "owner_alive_current_turn_other_event_player_from_hand_empty" => {
            TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty
        }
        "owner_gains_multiple_outside_own_draw" => TriggerFilter::OwnerGainsMultipleOutsideOwnDraw,
        "owner_gained_or_lost_at_least_two" => TriggerFilter::OwnerGainedOrLostAtLeastTwo,
        "owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn" => {
            TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        }
        "owner_is_event_player_discard_pile_four_suits_below_alive_cap" => {
            TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
        }
        "owner_card_responded_to_by_player_in_range" => {
            TriggerFilter::OwnerCardRespondedToByPlayerInRange
        }
        "owner_alive_first_cards_moved_to_hand" => TriggerFilter::OwnerAliveFirstCardsMovedToHand,
        "owner_draws_during_own_play_not_self" => TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf,
        "owner_source_non_equip_no_basic_hand_has_targets" => {
            TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets
        }
        "owner_equip_zone_changed_alive_wounded_at_least_two" => {
            TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded
        }
        "owner_current_turn_other_equipment_lost" => {
            TriggerFilter::OwnerCurrentTurnOtherEquipmentLost
        }
        "owner_alive_other_discarded_basic_outside_turn" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn
        }
        "owner_alive_other_discarded_basic_outside_turn_once" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        }
        "owner_first_hand_discard_has_sha_in_discard" => {
            TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard
        }
        "owner_alive_other_discarded_black_to_discard" => {
            TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
        }
        "owner_alive_dealt_or_received_damage_this_turn" => {
            TriggerFilter::OwnerAliveDealtOrReceivedDamageThisTurn
        }
        "owner_received_gift_from_other" => TriggerFilter::OwnerReceivedGiftFromOther,
        "owner_initiated_duel_target_alive_loser_has_hand_or_equip" => {
            TriggerFilter::OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip
        }
        "owner_alive_equip_used" => TriggerFilter::OwnerAliveEquipUsed,
        "owner_alive_equip_used_skill_mark_below_three" => {
            TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree
        }
        "owner_alive_source_alive_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveOncePerRound
        }
        "owner_alive_source_alive_event_player_dying_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
        }
        "owner_alive_once_per_round" => TriggerFilter::OwnerAliveOncePerRound,
        "owner_alive_not_target_once" => TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn,
        "owner_alive_current_or_turn_fires_below_hp" => {
            TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        }
        "owner_alive_card_is_sha" => TriggerFilter::OwnerAliveCardIsSha,
        "owner_alive_other_responded_sha_gainable" => {
            TriggerFilter::OwnerAliveOtherRespondedShaGainable
        }
        "owner_alive_card_targets_gt_one" => TriggerFilter::OwnerAliveCardTargetsGtOne,
        "owner_alive_non_multitarget_trick_targets_gt_one" => {
            TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne
        }
        "owner_alive_wounded_card_targets_gt_one" => {
            TriggerFilter::OwnerAliveWoundedCardTargetsGtOne
        }
        "owner_source_sha_or_trick_current_play_phase" => {
            TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase
        }
        "owner_source_sha_current_play_phase" => TriggerFilter::OwnerSourceShaCurrentPlayPhase,
        "owner_source_card_has_other_target_current_play_phase_first_two" => {
            TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo
        }
        "owner_alive_first_target_virtual_or_converted_card" => {
            TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard
        }
        "owner_alive_targeted_by_other_single_card_discardable" => {
            TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable
        }
        "owner_alive_targeted_by_trick_single_or_discardable" => {
            TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable
        }
        "owner_alive_targeted_by_other_trick" => TriggerFilter::OwnerAliveTargetedByOtherTrick,
        "owner_alive_targeted_by_other_matching_equip_suit" => {
            TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit
        }
        "owner_alive_targeted_by_other" => TriggerFilter::OwnerAliveTargetedByOther,
        "owner_alive_targeted_by_other_black_card" => {
            TriggerFilter::OwnerAliveTargetedByOtherBlackCard
        }
        "owner_alive_targeted_by_other_red_card" => {
            TriggerFilter::OwnerAliveTargetedByOtherRedCard
        }
        "owner_alive_other_source_death" => TriggerFilter::OwnerAliveOtherSourceDeath,
        "owner_is_target_play_phase_hand_no_sha" => {
            TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha
        }
        "owner_alive_other_phase_end_player_outside_owner_range" => {
            TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange
        }
        "owner_source_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip
        }
        "owner_source_card_used_targets_self" => {
            TriggerFilter::OwnerSourceCardUsedTargetsSelf
        }
        "owner_source_current_phase_card_targeted_other_healthy" => {
            TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy
        }
        "owner_min_hp" => TriggerFilter::OwnerMinHp,
        "owner_is_target_not_min_hp" => TriggerFilter::OwnerIsEventPlayerNotMinHp,
        "owner_target_min_hand" => TriggerFilter::OwnerIsEventPlayerMinHand,
        "owner_target_not_min_hand" => TriggerFilter::OwnerIsEventPlayerNotMinHand,
        "owner_sha_used_at_least_two" => TriggerFilter::OwnerShaUsedAtLeastTwo,
        "owner_alive_used_sha_this_turn" => TriggerFilter::OwnerAliveUsedShaThisTurn,
        "owner_max_hp_gt_one_hand_or_equip_empty" => TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty,
        "owner_hand_empty" | "hand_empty" => TriggerFilter::OwnerHandEmpty,
        "owner_turned_over" => TriggerFilter::OwnerTurnedOver,
        "owner_alive_other_wei_recovery_own_phase_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn
        }
        "owner_alive_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed
        }
        "owner_alive_other_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed
        }
        "owner_skill_pile_nonempty" => {
            TriggerFilter::OwnerSkillPileNonempty(u32::MAX)
        }
        s_token if s_token
            .strip_prefix("owner_skill_pile_nonempty_")
            .and_then(|rest| rest.parse::<u32>().ok())
            .is_some() =>
        {
            TriggerFilter::OwnerSkillPileNonempty(
                s_token
                    .strip_prefix("owner_skill_pile_nonempty_")
                    .and_then(|rest| rest.parse::<u32>().ok())
                    .unwrap_or(0),
            )
        }
        "owner_from_hand" | "from_hand" => TriggerFilter::OwnerFromHand,
        "owner_from_equip" | "from_equip" => TriggerFilter::OwnerFromEquip,
        "owner_from_equip_per_card" => TriggerFilter::OwnerFromEquipPerCard,
        "owner_from_hand_empty" | "from_hand_empty" => TriggerFilter::OwnerFromHandEmpty,
        "owner_from_hand_empty_other_discardable_hand_equip" => {
            TriggerFilter::OwnerFromHandEmptyOtherDiscardableHandEquip
        }
        "owner_from_field_empty" | "from_field_empty" => TriggerFilter::OwnerFromFieldEmpty,
        "owner_targeted_by_trick_equip_empty" => TriggerFilter::OwnerTargetedByTrickEquipEmpty,
        "owner_targeted_by_sha_hand_below_hp" => TriggerFilter::OwnerTargetedByShaHandBelowHp,
        "owner_target_physical_sha_source_hp_gt_owner" => {
            TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner
        }
        "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
            TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition
        }
        "owner_alive_phase_end_caused_two_distinct_dying" => {
            TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying
        }
        "owner_gained_event_cards_from_other_personal_zone" => {
            TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone
        }
        "owner_alive_other_single_target_used_tracked_twyizhu" => {
            TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu
        }
        "owner_tracked_twyizhu_entered_discard" => TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard,
        "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
            TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused
        }
        "owner_phase_begin_playable_skill_pile_nonempty" => {
            TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty
        }
        "owner_used_playable_skill_pile_card_unused" => {
            TriggerFilter::OwnerUsedPlayableSkillPileCardUnused
        }
        "owner_target_damage_before_playable_skill_piles_empty" => {
            TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty
        }
        "owner_alive_damage_before_target_min_hp_unused" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused
        }
        "owner_alive_global_phase_end" => TriggerFilter::OwnerAliveGlobalPhaseEnd,
        "owner_alive_other_phase_end_mobiledanshou" => {
            TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
        }
        "owner_alive_other_phase_end_not_damaged_owner_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn
        }
        "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
        }
        "owner_alive_other_source_damage_card_resolved_targeted_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        }
        "owner_alive_other_source_damage_card_resolved_targeted_not_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged
        }
        "owner_alive_other_dying_mbchenshe_available" => {
            TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable
        }
        "owner_is_target_discardable_hand_equip_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
        }
        "owner_alive_first_non_discard_phase_discard_mbrenxing_available" => {
            TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
        }
        "owner_alive_global_phase_end_lost_cards_target_other_alive" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
        }
        "owner_alive_global_phase_end_discarded_cards_available" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
        }
        "owner_source_active_dczhaowen_tracked_red" => {
            TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed
        }
        "owner_is_target_event_card_jiu" => TriggerFilter::OwnerIsTargetEventCardJiu,
        "owner_is_target_other_source_black_dcjiudun_available" => {
            TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable
        }
        "owner_alive_phase_end_used_card_no_damage" => {
            TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
        }
        "owner_cards_entered_discard_not_use_unused" => {
            TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused
        }
        "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once" => {
            TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        }
        "owner_targeted_outside_turn_by_sha_or_trick_hand_empty" => {
            TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty
        }
        "owner_targeted_hand_below_max_hp" => TriggerFilter::OwnerTargetedHandBelowMaxHp,
        "owner_targeted_by_other_single_target_trick" => {
            TriggerFilter::OwnerTargetedByOtherSingleTargetTrick
        }
        "owner_source_or_targeted_by_juedou_or_red_sha" => {
            TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha
        }
        "owner_source_or_targeted_by_juedou" => TriggerFilter::OwnerSourceOrTargetedByJuedou,
        "owner_source_or_targeted_by_sha_equal_hp" => {
            TriggerFilter::OwnerSourceOrTargetedByShaEqualHp
        }
        "owner_source_or_targeted_by_other_diamond" => {
            TriggerFilter::OwnerSourceOrTargetedByOtherDiamond
        }
        "owner_source_red" => TriggerFilter::OwnerSourceRed,
        "owner_source_alternates_color" => TriggerFilter::OwnerSourceAlternatesColor,
        "owner_source_repeats_color_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
        }
        "owner_alive_skill_bound_target_is_event_player" => {
            TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer
        }
        "owner_is_target_source_group_qun_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo
        }
        "owner_source_matches_previous_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank
        }
        "owner_source_ordinary_trick" => TriggerFilter::OwnerSourceOrdinaryTrick,
        "owner_source_physical_ordinary_trick" => TriggerFilter::OwnerSourcePhysicalOrdinaryTrick,
        "owner_hand_below_lost_hp_after_relevant_change" => {
            TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange
        }
        "owner_alive_source_first_card_ordinary_trick" => {
            TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
        }
        "owner_source_matches_last_used_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank
        }
        "owner_alive_equip_source_max_equip" => TriggerFilter::OwnerAliveEquipSourceMaxEquip,
        "owner_alive_other_current_source_repeats_card_name" => {
            TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
        }
        "owner_source_hand_basic_equals_non_basic" => {
            TriggerFilter::OwnerSourceHandBasicEqualsNonBasic
        }
        "owner_is_target_event_card_sha" => TriggerFilter::OwnerIsEventPlayerEventCardSha,
        "owner_is_target_sha_or_trick" => TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick,
        "owner_source_or_within_distance_one_sha_or_juedou" => {
            TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou
        }
        "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn" => {
            TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        }
        "owner_source_or_targeted_club_linked_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        }
        "owner_source_sha_single_trick_source_card" => {
            TriggerFilter::OwnerSourceShaSingleTrickSourceCard
        }
        "owner_source_sha_single_equip_source_card" => {
            TriggerFilter::OwnerSourceShaSingleEquipSourceCard
        }
        "owner_is_target_hand_above_hp_no_liubei_lord" => {
            TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord
        }
        "owner_alive_other_dying_first_per_target" => {
            TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
        }
        "owner_is_target_trick_used_zero" => TriggerFilter::OwnerIsEventPlayerTrickUsedZero,
        "owner_is_target_hp_le_one" => TriggerFilter::OwnerIsEventPlayerHpLeOne,
        "owner_is_target_hp_le_two" => TriggerFilter::OwnerIsEventPlayerHpLeTwo,
        "owner_source_or_target_second_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn
        }
        "owner_source_or_target_first_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn
        }
        "owner_source_target_distance_le_one_first_damage_to_target_this_turn" => {
            TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn
        }
        "owner_first_transformed_sha_used_this_turn" => {
            TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        }
        "owner_target_hand_empty_elemental_or_trick_damage_before" => {
            TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore
        }
        "owner_alive_draw_player_owner_or_min_hp_twjiaohua_available" => {
            TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable
        }
        "owner_source_sha" => TriggerFilter::OwnerSourceSha,
        "owner_target_sha" => TriggerFilter::OwnerTargetSha,
        "owner_source_heart_sha" => TriggerFilter::OwnerSourceHeartSha,
        "owner_source_sha_first_target" => TriggerFilter::OwnerSourceShaFirstTarget,
        "owner_alive_other_single_target_juedou_or_red_sha" => {
            TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha
        }
        "owner_alive_hand_gained_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
        }
        "owner_source_sha_or_juedou_single_target" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTarget
        }
        "owner_source_sha_damage_recorded_by_skill" => {
            TriggerFilter::OwnerSourceShaDamageRecordedBySkill
        }
        "owner_source_normal_sha" => TriggerFilter::OwnerSourceNormalSha,
        "owner_source_fire_sha" => TriggerFilter::OwnerSourceFireSha,
        "owner_is_target_hand_sha_skill_pile_empty" => {
            TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty
        }
        "owner_alive_dying_has_skill_pile_cards" => TriggerFilter::OwnerAliveDyingHasSkillPileCards,
        "owner_source_single_target_sha_or_ordinary_trick_current_turn" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn
        }
        "owner_source_event_card_policy_draw" => TriggerFilter::OwnerSourceEventCardPolicyDraw,
        "owner_source_card_resolved_phase_policy" => {
            TriggerFilter::OwnerSourceCardResolvedPhasePolicy
        }
        "owner_phase_play_end_potguansha_available_once_per_game" => {
            TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
        }
        "owner_discarded_sha_to_discard" => TriggerFilter::OwnerDiscardedShaToDiscard,
        "owner_alive_other_turn_discarded_recorded_potfuan_suit" => {
            TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
        }
        "owner_source_sha_resolved_potfuan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage
        }
        "owner_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerDiscardedBlackBasicToDiscard
        }
        "owner_alive_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard
        }
        "owner_source_card_resolved_ddd_xuyu_pending" => {
            TriggerFilter::OwnerSourceCardResolvedDddXuyuPending
        }
        "ddd_xuyu_pending" => TriggerFilter::DddXuyuPending,
        "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
        }
        "ddd_shijian_watched_source_card_used_targeted" => {
            TriggerFilter::DddShijianWatchedSourceCardUsedTargeted
        }
        "ddd_shijian_pending" => TriggerFilter::DddShijianPending,
        "owner_alive_hp_changed_this_turn_entered_discard_available" => {
            TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
        }
        "owner_alive_other_phase_begin_both_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty
        }
        "owner_alive_ddd_dongcha_pile_nonempty" => TriggerFilter::OwnerAliveDddDongchaPileNonempty,
        "owner_alive_other_phase_begin_unshown_hand_gte_hp" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp
        }
        "owner_play_dddzhengjun_hp_available" => TriggerFilter::OwnerPlayDddZhengjunHpAvailable,
        "owner_play_dddzhengjun_cards_moved_available" => {
            TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable
        }
        "owner_alive_other_phase_play_dddchashi_available" => {
            TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable
        }
        "dddchashi_matching_card" => TriggerFilter::DddChashiMatchingCard,
        "owner_alive_dddqice_ready" => TriggerFilter::OwnerAliveDddQiceReady,
        "owner_alive_ddd_fenji_available" => TriggerFilter::OwnerAliveDddFenjiAvailable,
        "owner_alive_ddd_fengzheng_round_pending" => {
            TriggerFilter::OwnerAliveDddFengzhengRoundPending
        }
        "owner_alive_ddd_yulv_discard_matches" => TriggerFilter::OwnerAliveDddYulvDiscardMatches,
        "owner_alive_ddd_youxue_round_pending" => TriggerFilter::OwnerAliveDddYouxueRoundPending,
        "owner_alive_ddd_chengjing_available" => TriggerFilter::OwnerAliveDddChengjingAvailable,
        "owner_alive_ddd_zhuanshe_gift" => TriggerFilter::OwnerAliveDddZhuansheGift,
        "owner_alive_ddd_zhuanshe_card_used" => TriggerFilter::OwnerAliveDddZhuansheCardUsed,
        "owner_alive_ddd_zhuanshe_phase_end" => TriggerFilter::OwnerAliveDddZhuanshePhaseEnd,
        "owner_alive_ddd_weiqiu" => TriggerFilter::OwnerAliveDddWeiqiu,
        "owner_alive_ddd_lianer_available" => TriggerFilter::OwnerAliveDddLianerAvailable,
        "owner_alive_ddd_lianer_ceiling" => TriggerFilter::OwnerAliveDddLianerCeiling,
        "owner_alive_ddd_anzhi_available" => TriggerFilter::OwnerAliveDddAnzhiAvailable,
        "owner_alive_ddd_anzhi_tracking" => TriggerFilter::OwnerAliveDddAnzhiTracking,
        "owner_alive_ddd_anzhi_target_black" => TriggerFilter::OwnerAliveDddAnzhiTargetBlack,
        "owner_alive_ddd_anzhi_turn_end" => TriggerFilter::OwnerAliveDddAnzhiTurnEnd,
        "owner_alive_ddd_shixing_turn_refresh" => TriggerFilter::OwnerAliveDddShixingTurnRefresh,
        "owner_alive_ddd_shixing_female_changed" => {
            TriggerFilter::OwnerAliveDddShixingFemaleChanged
        }
        "owner_alive_ddd_shixing_first_trick" => TriggerFilter::OwnerAliveDddShixingFirstTrick,
        "owner_alive_ddd_danggu_available" => TriggerFilter::OwnerAliveDddDangguAvailable,
        "owner_alive_ddd_fuzong_death" => TriggerFilter::OwnerAliveDddFuzongDeath,
        "owner_is_target_face_down" => TriggerFilter::OwnerIsTargetFaceDown,
        "owner_alive_ddd_xuanlun_pending" => TriggerFilter::OwnerAliveDddXuanlunPending,
        "owner_alive_ddd_jiexing_recover_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
        }
        "owner_alive_ddd_jiexing_equip_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
        }
        "owner_alive_ddd_jiexing_recovery_reset" => {
            TriggerFilter::OwnerAliveDddJiexingRecoveryReset
        }
        "owner_alive_ddd_jiexing_equip_reset" => TriggerFilter::OwnerAliveDddJiexingEquipReset,
        "owner_alive_ddd_bailei_available" => TriggerFilter::OwnerAliveDddBaileiAvailable,
        "owner_source_own_view_as_ddd_tongyu_used" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed
        }
        "owner_source_own_view_as_ddd_tongyu_targeted" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted
        }
        "owner_alive_ddd_tongyu_remained" => TriggerFilter::OwnerAliveDddTongyuRemained,
        "dddchashi_pending_phase_play_end" => TriggerFilter::DddChashiPendingPhasePlayEnd,
        "owner_alive_dddzhilian_available" => TriggerFilter::OwnerAliveDddZhilianAvailable,
        "owner_alive_source_basic_dddfushi_available" => {
            TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable
        }
        "owner_alive_other_judge_phase_in_range_has_judge_card" => {
            TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard
        }
        "owner_source_play_sha_other_alive_once_per_play_phase" => {
            TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        }
        "owner_source_jiu" => TriggerFilter::OwnerSourceJiu,
        "owner_is_target_hp_odd" => TriggerFilter::OwnerIsEventPlayerHpOdd,
        "owner_is_target_hp_even_wounded" => TriggerFilter::OwnerIsEventPlayerHpEvenWounded,
        "owner_source_trick" | "card_is_trick" => TriggerFilter::OwnerSourceTrick,
        "once_per_turn" => TriggerFilter::OncePerTurn,
        "owner_source_once" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_once_per_turn" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_target_alive_has_equip" => TriggerFilter::OwnerSourceTargetAliveHasEquip,
        "owner_source_black_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip
        }
        "owner_source_sha_damage_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip
        }
        "owner_source_equip_used_skill_mark_below_seven" => {
            TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven
        }
        "owner_source_outside_turn_card_used_or_responded" => {
            TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded
        }
        "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
            TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn
        }
        "owner_alive_other_phase_play_skill_pile_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty
        }
        "owner_source_sha_target_alive_has_discardable_region_card" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard
        }
        "owner_source_sha_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip
        }
        "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty" => {
            TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty
        }
        "owner_alive_source_target_was_linked" => TriggerFilter::OwnerAliveSourceTargetWasLinked,
        "owner_source_sha_target_alive_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaTargetAliveHandNonempty
        }
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange
        }
        "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" => {
            TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner
        }
        "owner_is_target_judge_nonempty" => TriggerFilter::OwnerIsEventPlayerJudgeNonempty,
        "owner_is_target_draw_not_replaced" => TriggerFilter::OwnerIsEventPlayerDrawNotReplaced,
        "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty
        }
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip
        }
        "owner_source_sha_dodged" => TriggerFilter::OwnerSourceShaDodged,
        "owner_source_sha_target_alive_distance_le_one" => {
            TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne
        }
        "owner_alive_sha_target_distance_le_one" => TriggerFilter::OwnerAliveShaTargetDistanceLeOne,
        "owner_alive_sha_target_distance_le_one_has_equip_card" => {
            TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty
        }
        "owner_alive_wdb_fuchou_pending_living" => TriggerFilter::OwnerAliveWdbFuchouPendingLiving,
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable" => {
            TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable
        }
        "owner_alive_wugu_remained_in_discard" => TriggerFilter::OwnerAliveWuguRemainedInDiscard,
        "owner_is_target_non_multitarget_trick_multiple_targets" => {
            TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets
        }
        "owner_source_play_phase_sha_has_additional_target_ignoring_distance" => {
            TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance
        }
        "owner_source_card_resolved_tracked_dddxianxi" => {
            TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi
        }
        "owner_source_card_damage_target_can_receive_owner_bingliang" => {
            TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang
        }
        "owner_alive_other_dying_hand_nonempty_once_per_game" => {
            TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
        }
        "owner_dying_other_hand_equip_nonempty_once_per_game" => {
            TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
        }
        "owner_alive_other_dying_once_per_round" => TriggerFilter::OwnerAliveOtherDyingOncePerRound,
        "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty
        }
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty
        }
        "owner_alive_source_ally_sha_damage_before" => {
            TriggerFilter::OwnerAliveSourceAllyShaDamageBefore
        }
        "owner_is_target_wounded_dead_ally" => TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly,
        "owner_is_target_any_alive_self_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip
        }
        "owner_is_target_any_alive_discardable_by_owner_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip
        }
        "owner_is_target_any_alive_self_discardable_hand" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand
        }
        "owner_is_target_any_alive_wounded" => TriggerFilter::OwnerIsEventPlayerAnyAliveWounded,
        "owner_is_target_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive
        }
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive
        }
        "owner_alive_gained_three_this_turn_other_alive" => {
            TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
        }
        "owner_targeted_by_other_sha_once" => TriggerFilter::OwnerTargetedByOtherShaOncePerTurn,
        "owner_targeted_by_other_card_once_per_turn" => {
            TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        }
        "owner_twfeifu_single_sha_party_target_has_hand_equip" => {
            TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip
        }
        "owner_targeted_by_other_sha" => TriggerFilter::OwnerTargetedByOtherSha,
        "owner_card_action_count_eq_attack_range" => {
            TriggerFilter::OwnerCardActionCountEqualsAttackRange
        }
        "owner_lifetime_card_action" => TriggerFilter::OwnerLifetimeCardAction,
        "owner_is_target_once" => TriggerFilter::OwnerIsEventPlayerOncePerTurn,
        "owner_source_healthy" => TriggerFilter::OwnerSourceHealthy,
        "owner_source_amount_gt_one" => TriggerFilter::OwnerSourceAmountGtOne,
        "owner_source_other_twice" => TriggerFilter::OwnerSourceOtherTwicePerTurn,
        "owner_alive_other_source_sha_twice" => TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn,
        "owner_has_enabled_equip_slot" => TriggerFilter::OwnerHasEnabledEquipSlot,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
        }
        "owner_is_target_has_movable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerHasMovableEquipment
        }
        "owner_alive_is_target_current_play_phase" => {
            TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase
        }
        "owner_alive_other_play_phase_pothuilv_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable
        }
        "owner_is_target_can_use_wuzhong" => TriggerFilter::OwnerIsTargetCanUseWuzhong,
        "owner_alive_discard_batch" => TriggerFilter::OwnerAliveDiscardBatch,
        "owner_alive_other_responded_to_owner" => TriggerFilter::OwnerAliveOtherRespondedToOwner,
        "owner_source_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable
        }
        "owner_target_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable
        }
        "owner_alive_dying_potjianzhuan_available" => {
            TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable
        }
        "owner_source_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered
        }
        "owner_target_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered
        }
        "owner_alive_dying_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered
        }
        "owner_used_potnizun_pile_card" => TriggerFilter::OwnerUsedPotnizunPileCard,
        "owner_target_damage_before_potnizun_used" => {
            TriggerFilter::OwnerTargetDamageBeforePotnizunUsed
        }
        "owner_alive_turn_ended_used_card_tagged_discard" => {
            TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard
        }
        "always" => TriggerFilter::Always,
        _ => return None,
    })
}

fn valid_group_filter_tag(group: &str) -> bool {
    !group.is_empty()
        && group
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn map_mod_target(s: &str, selector: Option<&str>) -> Option<ModifierTarget> {
    if selector.is_none() {
        if let Some(mark) = s.strip_prefix("judgment_rewrite_hand_equip_cost_mark_") {
            return mark
                .parse::<u32>()
                .ok()
                .filter(|mark| *mark > 0)
                .map(ModifierTarget::JudgmentRewriteHandEquipCostMark);
        }
    }
    if let Some(value) = s.strip_prefix("card_distance_unlimited_target_mark_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardDistanceUnlimitedTargetMark);
    }
    if let Some(value) = s.strip_prefix("card_distance_unlimited_skill_on_owner_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardDistanceUnlimitedSkillOnOwner);
    }
    if let Some(value) = s.strip_prefix("card_use_unrespondable_skill_on_owner_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::CardUseUnrespondableSkillOnOwner);
    }
    if let Some(value) = s.strip_prefix("sha_use_unlimited_target_mark_") {
        return value
            .parse::<u32>()
            .ok()
            .filter(|value| *value > 0)
            .map(ModifierTarget::ShaUseUnlimitedTargetMark);
    }
    if s == "card_suit" {
        return Some(ModifierTarget::CardSuit(map_suit_selector(selector?)?));
    }
    if s == "card_number" && selector.is_none() {
        return Some(ModifierTarget::CardNumber);
    }
    if s == "card_name" {
        return Some(ModifierTarget::CardName(map_card_selector(selector?)?));
    }
    if s == "card_nature" {
        return Some(ModifierTarget::CardNatureFor(map_card_selector(selector?)?));
    }
    if s == "sha_unrespondable" {
        return Some(ModifierTarget::ShaUnrespondableFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_use_unrespondable" {
        return Some(ModifierTarget::CardUseUnrespondableFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "sha_damage" && selector.is_some() {
        return Some(ModifierTarget::ShaDamageFor(map_card_selector(selector?)?));
    }
    if s == "damage_taken_cancel" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenCancel,
            Some("normal") => ModifierTarget::DamageTakenCancelForNature(0),
            Some("fire") => ModifierTarget::DamageTakenCancelForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenCancelForNature(2),
            _ => return None,
        });
    }
    if s == "damage_amount" && selector.is_some() {
        return Some(match selector {
            Some("any") => ModifierTarget::DamageAmount,
            Some("normal") => ModifierTarget::DamageAmountForNature(0),
            Some("fire") => ModifierTarget::DamageAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageAmountForNature(2),
            Some("sha") => ModifierTarget::DamageAmountForSha,
            _ => return None,
        });
    }
    if s == "damage_taken_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenAmount,
            Some("normal") => ModifierTarget::DamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::DamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "global_damage_taken_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::GlobalDamageTakenAmount,
            Some("normal") => ModifierTarget::GlobalDamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::GlobalDamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::GlobalDamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "armor_damage_taken_amount" {
        return Some(match selector {
            Some("normal") => ModifierTarget::ArmorDamageTakenAmountForNature(0),
            Some("fire") => ModifierTarget::ArmorDamageTakenAmountForNature(1),
            Some("thunder") => ModifierTarget::ArmorDamageTakenAmountForNature(2),
            _ => return None,
        });
    }
    if s == "damage_taken_final_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::DamageTakenFinalAmount,
            Some("normal") => ModifierTarget::DamageTakenFinalAmountForNature(0),
            Some("fire") => ModifierTarget::DamageTakenFinalAmountForNature(1),
            Some("thunder") => ModifierTarget::DamageTakenFinalAmountForNature(2),
            _ => return None,
        });
    }
    if s == "global_damage_taken_final_amount" {
        return Some(match selector {
            None | Some("any") => ModifierTarget::GlobalDamageTakenFinalAmount,
            Some("normal") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(0),
            Some("fire") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(1),
            Some("thunder") => ModifierTarget::GlobalDamageTakenFinalAmountForNature(2),
            _ => return None,
        });
    }
    if s == "damage_taken_nature" && selector.is_none() {
        return Some(ModifierTarget::DamageTakenNature);
    }
    if s == "damage_as_hp_loss" && selector.is_some() {
        return match selector {
            Some("sha") => Some(ModifierTarget::DamageAsHpLossForSha),
            _ => None,
        };
    }
    if s == "card_use_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardUseCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardUseCancel,
        });
    }
    if s == "card_response_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardResponseCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardResponseCancel,
        });
    }
    if s == "card_save_cancel" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardSaveCancelFor(map_card_selector(selector)?),
            None => ModifierTarget::CardSaveCancel,
        });
    }
    if s == "card_ignored_hand_limit" {
        return Some(ModifierTarget::CardIgnoredHandLimit(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_unlimited" {
        return Some(match selector {
            Some(selector) => {
                ModifierTarget::CardDistanceUnlimitedFor(map_card_selector(selector)?)
            }
            None => ModifierTarget::CardDistanceUnlimited,
        });
    }
    if s == "card_discard_protected" {
        return Some(ModifierTarget::CardDiscardProtectedFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_gain_protected" {
        return Some(ModifierTarget::CardGainProtectedFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_unlimited_if_target_hand_gte_owner" {
        return Some(
            ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(map_card_selector(
                selector?,
            )?),
        );
    }
    if s == "card_use_unlimited" {
        return Some(match selector {
            Some(selector) => ModifierTarget::CardUseUnlimitedFor(map_card_selector(selector)?),
            None => ModifierTarget::CardUseUnlimited,
        });
    }
    if s == "first_hand_card_suit_use_unlimited" && selector.is_none() {
        return Some(ModifierTarget::FirstHandCardSuitUseUnlimited);
    }
    if s == "card_target_limit" {
        return Some(ModifierTarget::CardTargetLimitFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_distance_limit" {
        return Some(ModifierTarget::CardDistanceLimitFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_targets_all" {
        return Some(ModifierTarget::CardTargetsAllFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_target_cancel" {
        return Some(ModifierTarget::CardTargetCancelFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "armor_card_target_cancel" {
        return Some(ModifierTarget::ArmorCardTargetCancelFor(map_card_selector(
            selector?,
        )?));
    }
    if s == "card_target_cancel_owner_turn_banned" {
        return Some(ModifierTarget::CardTargetCancelTurnBannedFor(
            map_card_selector("any").expect("any selector"),
        ));
    }
    if s == "card_target_cancel_from_current" {
        return Some(ModifierTarget::CardTargetCancelFromCurrentFor(
            map_card_selector(selector?)?,
        ));
    }
    if s == "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty" {
        return Some(
            ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(
                map_card_selector(selector?)?,
            ),
        );
    }
    if s == "card_target_cancel_if_other_user_distance_gt_one" {
        return Some(ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(
            map_card_selector(selector?)?,
        ));
    }
    if selector.is_some() {
        return None;
    }
    Some(match s {
        "distance_from" => ModifierTarget::DistanceFrom,
        "distance_from_if_hp_gte_target" => ModifierTarget::DistanceFromHpGteTarget,
        "distance_from_skill_bound_target" => ModifierTarget::DistanceFromSkillBoundTarget,
        "link_loss_cancel" => ModifierTarget::LinkLossCancel,
        "distance_to" => ModifierTarget::DistanceTo,
        "global_distance_to" => ModifierTarget::GlobalDistanceTo,
        "nanman_damage_source_as_owner" => ModifierTarget::NanmanDamageSourceAsOwner,
        "attack_range" => ModifierTarget::AttackRange,
        "global_attack_range" => ModifierTarget::GlobalAttackRange,
        "hand_limit" => ModifierTarget::HandLimit,
        "global_hand_limit" => ModifierTarget::GlobalHandLimit,
        "global_draw_count" => ModifierTarget::GlobalDrawCount,
        "global_sha_use_limit" => ModifierTarget::GlobalShaUseLimit,
        "hand_limit_base" => ModifierTarget::HandLimitBase,
        "draw_count" | "draw_phase" => ModifierTarget::DrawCount,
        "optional_draw_count" => ModifierTarget::OptionalDrawCount,
        "sha_damage" => ModifierTarget::ShaDamage,
        "sha_damage_taken" => ModifierTarget::ShaDamageTaken,
        "sha_damage_if_outside_target_range" => ModifierTarget::ShaDamageIfOutsideTargetRange,
        "sha_damage_source_range_threshold" => ModifierTarget::ShaDamageSourceRangeThreshold,
        "sha_unrespondable_within_distance" => ModifierTarget::ShaUnrespondableWithinDistance,
        "sha_target_unrespondable" => ModifierTarget::ShaTargetUnrespondable,
        "sha_use_unrespondable_within_distance" => {
            ModifierTarget::ShaUseUnrespondableWithinDistance
        }
        "card_use_unrespondable_within_responder_to_owner_distance" => {
            ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance
        }
        "card_use_unrespondable_within_owner_to_responder_distance" => {
            ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance
        }
        "damage_taken_source_range_threshold" => ModifierTarget::DamageTakenSourceRangeThreshold,
        "skip_judge_phase" => ModifierTarget::SkipJudgePhase,
        "skip_discard_phase" => ModifierTarget::SkipDiscardPhase,
        "restore_skipped_phase" => ModifierTarget::RestoreSkippedPhase,
        "turn_face_down_cancel" => ModifierTarget::TurnFaceDownCancel,
        "turn_over_cancel" => ModifierTarget::TurnOverCancel,
        "discard_phase_includes_equip" => ModifierTarget::DiscardPhaseIncludesEquip,
        "observer_foreign_hand_faces_visible" => ModifierTarget::ObserverForeignHandFacesVisible,
        "observer_hidden_identities_visible" => ModifierTarget::ObserverHiddenIdentitiesVisible,
        "draw_phase_scry_four_split_two" => ModifierTarget::DrawPhaseScryFourSplitTwo,
        "initial_hand_count" => ModifierTarget::InitialHandCount,
        "sha_use_limit" => ModifierTarget::ShaUseLimit,
        "sha_target_limit" => ModifierTarget::ShaTargetLimit,
        "card_targets_self_only" => ModifierTarget::CardTargetsSelfOnly,
        "response_count" => ModifierTarget::ResponseCount,
        "duel_opponent_response_count" => ModifierTarget::DuelOpponentResponseCount,
        "skip_discard_if_no_sha" => ModifierTarget::SkipDiscardIfNoSha,
        "draw_replacement_damage" => ModifierTarget::DrawReplacementDamage,
        "prepare_scry_count" => ModifierTarget::PrepareScryCount,
        "judgment_rewrite" => ModifierTarget::JudgmentRewrite,
        "judgment_rewrite_hand_equip" => ModifierTarget::JudgmentRewriteHandEquip,
        "judgment_rewrite_red_hand_equip_expansion" => {
            ModifierTarget::JudgmentRewriteRedHandEquipExpansion
        }
        "judgment_rewrite_black_hand_equip" => ModifierTarget::JudgmentRewriteBlackHandEquip,
        "skills_disabled_when_hp_one" => ModifierTarget::SkillsDisabledWhenHpOne,
        "judgment_best_from_draw_pile" => ModifierTarget::JudgmentBestFromDrawPile,
        "virtual_bagua_if_armor_slot_empty" => ModifierTarget::VirtualBaguaIfArmorSlotEmpty,
        "sha_response_count_if_other_female" => ModifierTarget::ShaResponseCountIfOtherFemale,
        "sha_response_count_if_source_owner_target_distance_le_one" => {
            ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne
        }
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn" => {
            ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn
        }
        "tao_use_save_cancel_for_dying_beyond_distance" => {
            ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance
        }
        "damage_amount" => ModifierTarget::DamageAmount,
        "damage_as_hp_loss" => ModifierTarget::DamageAsHpLoss,
        "damage_between_owner_and_other_as_hp_loss" => {
            ModifierTarget::DamageBetweenOwnerAndOtherAsHpLoss
        }
        "damage_nature" => ModifierTarget::DamageNature,
        "damage_cancel" => ModifierTarget::DamageCancel,
        "card_response_cancel" => ModifierTarget::CardResponseCancel,
        "immune_shunshou" => ModifierTarget::ImmuneShunshou,
        "immune_le" => ModifierTarget::ImmuneLe,
        "sha_use_cancel" => ModifierTarget::ShaUseCancel,
        "hand_limit_base_max_hp" => ModifierTarget::HandLimitBaseMaxHp,
        "sha_response_cancel" => ModifierTarget::ShaResponseCancel,
        "sha_ignored_hand_limit" => ModifierTarget::ShaIgnoredHandLimit,
        "hand_card_use_response_cancel" => ModifierTarget::HandCardUseResponseCancel,
        "hand_card_gained_this_turn_only" => ModifierTarget::HandCardGainedThisTurnOnly,
        "card_target_cancel_if_target_lacks_matching_suit" => {
            ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit
        }
        "card_target_cancel_if_other_user_hp_gte_target" => {
            ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget
        }
        "ignore_armor" => ModifierTarget::IgnoreArmor,
        "armor_effect_disabled" => ModifierTarget::ArmorEffectDisabled,
        "armor_link_gain_cancel" => ModifierTarget::ArmorLinkGainCancel,
        "card_distance_unlimited_for_skill_bound_target" => {
            ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget
        }
        "sha_use_unlimited_for_skill_bound_target" => {
            ModifierTarget::ShaUseUnlimitedForSkillBoundTarget
        }
        "ignore_armor_for_skill_bound_target" => ModifierTarget::IgnoreArmorForSkillBoundTarget,
        "card_action_cancel_for_skill_color_mask" => {
            ModifierTarget::CardActionCancelForSkillColorMask
        }
        "ignore_armor_for_skill_card_policy" => ModifierTarget::IgnoreArmorForSkillCardPolicy,
        "card_use_unrespondable_for_skill_card_policy" => {
            ModifierTarget::CardUseUnrespondableForSkillCardPolicy
        }
        "card_distance_unlimited_for_skill_card_policy" => {
            ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy
        }
        "sha_distance_unlimited_if_rank_lt_skill_mark" => {
            ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark
        }
        "sha_no_use_count_if_rank_gt_skill_mark" => ModifierTarget::ShaNoUseCountIfRankGtSkillMark,
        "sha_distance_unlimited_if_target_distance_lte_rank" => {
            ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank
        }
        _ => return None,
    })
}

fn map_suit_selector(s: &str) -> Option<engine_api::Suit> {
    Some(match s {
        "spade" => engine_api::Suit::Spade,
        "heart" => engine_api::Suit::Heart,
        "club" => engine_api::Suit::Club,
        "diamond" => engine_api::Suit::Diamond,
        _ => return None,
    })
}

fn mod_suit_tag(suit: engine_api::Suit) -> &'static str {
    match suit {
        engine_api::Suit::None => "none",
        engine_api::Suit::Spade => "spade",
        engine_api::Suit::Heart => "heart",
        engine_api::Suit::Club => "club",
        engine_api::Suit::Diamond => "diamond",
    }
}

fn map_card_selector(s: &str) -> Option<crate::query::CardSelector> {
    use crate::query::CardSelector;
    Some(match s {
        "any" => CardSelector::Any,
        "sha" => CardSelector::Sha,
        "tao" => CardSelector::Tao,
        "bingliang" => CardSelector::Bingliang,
        "lebu" => CardSelector::Lebu,
        "basic" => CardSelector::Basic,
        "non_basic" => CardSelector::NonBasic,
        "trick" => CardSelector::Trick,
        "equip" => CardSelector::Equip,
        "spade" => CardSelector::Spade,
        "heart" => CardSelector::Heart,
        "club" => CardSelector::Club,
        "diamond" => CardSelector::Diamond,
        "red" => CardSelector::Red,
        "black" => CardSelector::Black,
        "juedou" => CardSelector::Juedou,
        "black_trick" => CardSelector::BlackTrick,
        "damage_trick" => CardSelector::DamageTrick,
        "sha_fire" => CardSelector::ShaFire,
        "huogong" => CardSelector::Huogong,
        "jiu" => CardSelector::Jiu,
        "physical_jiu" => CardSelector::PhysicalJiu,
        "sha_red" => CardSelector::ShaRed,
        "nanman" => CardSelector::Nanman,
        "ordinary_trick" => CardSelector::OrdinaryTrick,
        "sha_black" => CardSelector::ShaBlack,
        "sha_elemental" => CardSelector::ShaElemental,
        "equip_non_mount" => CardSelector::EquipNonMount,
        "armor_or_treasure" => CardSelector::ArmorOrTreasure,
        "wanjian" => CardSelector::Wanjian,
        "sha_normal" => CardSelector::ShaNormal,
        "sha_heart" => CardSelector::ShaHeart,
        "delayed_trick" => CardSelector::DelayedTrick,
        "wuxie" => CardSelector::Wuxie,
        "guohe" => CardSelector::Guohe,
        "shunshou" => CardSelector::Shunshou,
        "sha_thunder" => CardSelector::ShaThunder,
        "hand_rank_letter" => CardSelector::HandRankLetter,
        "sole_hand_card" => CardSelector::SoleHandCard,
        "sha_diamond" => CardSelector::ShaDiamond,
        "skill_tracked" => CardSelector::SkillTracked,
        "damage_card" => CardSelector::DamageCard,
        "face_up" => CardSelector::FaceUp,
        _ => return None,
    })
}

fn map_mod_mode(s: &str) -> Option<CombineMode> {
    Some(match s {
        "add" => CombineMode::Add,
        "override" => CombineMode::Override,
        "min" => CombineMode::Min,
        "max" => CombineMode::Max,
        "multiply" => CombineMode::Multiply,
        "veto" => CombineMode::Veto,
        _ => return None,
    })
}

/// Convenience: skill id used by official jianxiong sample.
pub fn jianxiong_skill_def() -> SkillDefId {
    SkillDefId::new(BOOTSTRAP_JIANXIONG)
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
