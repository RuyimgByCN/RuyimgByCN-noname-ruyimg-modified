//! Reproducible content package (.npack logical layout as JSON for Phase 4).

use crate::{CONTENT_SCHEMA_VERSION, PACKAGE_FORMAT_VERSION};
use rule_api::CapabilitySet;
use rule_vm::{BytecodeModule, RULE_VM_ABI_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path};
mod hashing;

/// Exact dependency required by a package and pinned by [`crate::ContentLock`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageDependency {
    /// Dependency namespace.
    pub namespace: String,
    /// Dependency package id.
    pub package_id: String,
    /// Exact required version.
    pub version: String,
    /// Optional required logic hash; zero accepts the declared version's hash.
    #[serde(default, skip_serializing_if = "is_zero_hash")]
    pub logic_hash: [u8; 32],
}

/// Package-owned presentation overlays that may target characters from dependency packages.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSkinIndex {
    /// Concrete skins contributed by this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<PackageSkinEntry>,
    /// Character aliases that share another character's skins and voice keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<PackageSkinAliasEntry>,
}

impl PackageSkinIndex {
    /// Whether the index contributes no presentation overlays.
    pub fn is_empty(&self) -> bool {
        self.skins.is_empty() && self.aliases.is_empty()
    }
}

/// One skin contributed for an existing character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSkinEntry {
    /// Globally stable, non-zero skin id.
    pub id: u32,
    /// Character definition receiving this skin; it may live in a dependency package.
    pub character_id: u32,
    /// Localization key for the skin display name.
    pub name_key: String,
    /// Rendering model.
    pub presentation: PackageSkinPresentation,
    /// Logical voice key → package-local audio asset id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub voice_assets: BTreeMap<String, String>,
}

/// Safe rendering models accepted from a content package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageSkinPresentation {
    /// Ordinary selectable still image.
    Static {
        /// Package-local image asset id.
        asset: String,
    },
    /// Spine bundle with declarative actions and fixed-point layout.
    Spine {
        /// Package-local data asset containing the Spine bundle.
        bundle: String,
        /// Logical action key → Spine animation name.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        actions: BTreeMap<String, String>,
        /// Renderer-safe layout values.
        #[serde(default)]
        layout: PackageSkinLayout,
        /// Package-local assets required alongside the bundle.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        dependencies: Vec<String>,
    },
    /// Still-image substitute selected by a declarative host rule.
    Substitute {
        /// Safe host-defined rule name, such as `dead` or `awakened`.
        rule: String,
        /// Package-local image asset id.
        asset: String,
    },
}

/// Fixed-point Spine layout; no executable CSS or script is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSkinLayout {
    /// Horizontal pixel offset.
    #[serde(default)]
    pub x_pixels: i16,
    /// Vertical pixel offset.
    #[serde(default)]
    pub y_pixels: i16,
    /// Horizontal anchor in thousandths (`500` = 50%).
    #[serde(default = "default_anchor_permille")]
    pub anchor_x_permille: u16,
    /// Vertical anchor in thousandths (`500` = 50%).
    #[serde(default = "default_anchor_permille")]
    pub anchor_y_permille: u16,
    /// Scale in thousandths (`1000` = 1×).
    #[serde(default = "default_scale_permille")]
    pub scale_permille: u16,
    /// Clockwise rotation in whole degrees.
    #[serde(default)]
    pub rotation_degrees: i16,
    /// Animation speed in thousandths (`1000` = 1×).
    #[serde(default = "default_scale_permille")]
    pub speed_permille: u16,
}

impl Default for PackageSkinLayout {
    fn default() -> Self {
        Self {
            x_pixels: 0,
            y_pixels: 0,
            anchor_x_permille: default_anchor_permille(),
            anchor_y_permille: default_anchor_permille(),
            scale_permille: default_scale_permille(),
            rotation_degrees: 0,
            speed_permille: default_scale_permille(),
        }
    }
}

fn default_anchor_permille() -> u16 {
    500
}

fn default_scale_permille() -> u16 {
    1000
}

/// One character shares the skins of another without redeclaring either roster entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSkinAliasEntry {
    /// Character receiving the shared skins.
    pub character_id: u32,
    /// Character whose skins are shared.
    pub source_character_id: u32,
    /// Receiving logical voice key → source logical voice key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub voice_mappings: BTreeMap<String, String>,
}

/// Package manifest (architecture §10.5 + §42 licensing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package format version.
    pub package_format_version: u32,
    /// Namespace.
    pub namespace: String,
    /// Package id.
    pub package_id: String,
    /// Semver-like version string.
    pub version: String,
    /// Content schema version.
    pub content_schema_version: u32,
    /// Rule VM ABI version.
    pub rule_vm_abi_version: u32,
    /// Min engine API.
    pub min_engine_api: u32,
    /// Max engine API (inclusive).
    pub max_engine_api: u32,
    /// Capability set requested.
    pub capabilities: CapabilitySet,
    /// Exact package dependencies. Empty keeps legacy standalone-package behavior.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<PackageDependency>,
    /// Logic hash (semantic — no timestamps).
    pub logic_hash: [u8; 32],
    /// Asset hash (zeros if no assets).
    pub asset_hash: [u8; 32],
    /// Author string.
    pub author: String,
    /// License record (§42: 代码许可 / 资源许可 / 来源 / 可再分发 / 修改记录).
    /// Defaults to `Unspecified` — official distribution requires explicit license.
    #[serde(default)]
    pub license: licensing_schema::LicenseRecord,
}

impl PackageManifest {
    /// Create a bootstrap-style manifest shell (hash filled later).
    pub fn new(namespace: &str, package_id: &str, version: &str) -> Self {
        Self {
            package_format_version: PACKAGE_FORMAT_VERSION,
            namespace: namespace.into(),
            package_id: package_id.into(),
            version: version.into(),
            content_schema_version: CONTENT_SCHEMA_VERSION,
            rule_vm_abi_version: RULE_VM_ABI_VERSION,
            min_engine_api: 1,
            max_engine_api: 1,
            capabilities: CapabilitySet::standard_skill(),
            dependencies: Vec::new(),
            logic_hash: [0u8; 32],
            asset_hash: [0u8; 32],
            author: "noname-next".into(),
            license: licensing_schema::LicenseRecord::default(),
        }
    }
}

/// Player who confirms an optional trigger.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum TriggerChoiceActor {
    /// The skill owner confirms the trigger (legacy/default behavior).
    #[default]
    Owner,
    /// The primary player carried by the triggering event confirms it.
    EventPlayer,
    /// The source player carried by the triggering event confirms it.
    EventSource,
}

impl TriggerChoiceActor {
    /// Whether this is the backwards-compatible owner policy.
    pub fn is_owner(&self) -> bool {
        *self == Self::Owner
    }

    /// Stable tag used by the semantic package hash.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Owner => 0,
            Self::EventPlayer => 1,
            Self::EventSource => 2,
        }
    }
}

/// Trigger index entry for package (static).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTriggerEntry {
    /// Trigger def id.
    pub trigger_def_id: u32,
    /// Skill def id.
    pub skill_def_id: u32,
    /// Event tag string.
    pub event: String,
    /// Filter string.
    pub filter: String,
    /// Whether a player must confirm before execution.
    #[serde(default)]
    pub optional: bool,
    /// Player who confirms an optional trigger.
    #[serde(default, skip_serializing_if = "TriggerChoiceActor::is_owner")]
    pub choice_actor: TriggerChoiceActor,
    /// Module name containing handler.
    pub module: String,
    /// Entry PC in module.
    pub entry_pc: u32,
}

/// Skill-level metadata independent of any one handler module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSkillEntry {
    /// Skill definition id.
    pub skill_def_id: u32,
    /// Source-level stable name.
    pub name: String,
    /// Locked/passive skill marker.
    #[serde(default)]
    pub locked: bool,
    /// Only active while the owner has the Lord identity.
    #[serde(default, skip_serializing_if = "is_false")]
    pub lord_only: bool,
    /// Cannot be copied or gained by another skill.
    #[serde(default, skip_serializing_if = "is_false")]
    pub non_gainable: bool,
    /// Conversion skill whose branch can be toggled by generic rules.
    #[serde(default, skip_serializing_if = "is_false")]
    pub conversion: bool,
}

/// Rules-only character definition carried by a content package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PackageCharacterEntry {
    /// Stable character definition id.
    pub id: u32,
    /// Localization/display key.
    pub name_key: String,
    /// Base maximum HP.
    pub max_hp: u32,
    /// Starting HP; zero preserves the prior start-at-`max_hp` behavior when omitted.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub initial_hp: u32,
    /// Granted skill definition ids.
    pub skills: Vec<u32>,
    /// Character sex tag used by declarative rules and UI.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sex: String,
    /// Character group/faction tag used by declarative rules and UI.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
    /// Whether this definition is a lord character.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_lord: bool,
    /// Additional stable rule/UI tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Clan/family memberships (e.g. "太原王氏") used by clan-skill rules.
    /// Empty for non-clan characters; hash-neutral when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clans: Vec<String>,
    /// Optional package-local sort key.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sort_key: String,
    /// Logical portrait asset id from `asset_paths`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait_asset: Option<String>,
    /// Logical skin asset ids from `asset_paths`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skin_assets: Vec<String>,
}

/// Immutable card definition shipped by a content package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageCardEntry {
    /// Stable card definition id.
    pub card_def_id: u32,
    /// Source-level stable name.
    pub name: String,
    /// Rule-visible length of the localized card name; zero means legacy/unspecified.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub name_length: u8,
    /// Whether this definition carries the upstream `gifts` card tag.
    #[serde(default, skip_serializing_if = "is_false")]
    pub gift: bool,
    /// Engine card kind (`weapon`, `armor`, `horse_def`, `horse_off`, or `treasure`).
    pub kind: String,
    /// Weapon range, zero for non-weapons.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub weapon_range: u8,
    /// Skills granted while this definition is equipped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equip_skills: Vec<u32>,
    /// Additional distance adjustment from the owner to other players.
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub distance_from: i8,
    /// Additional distance adjustment from other players to the owner.
    #[serde(default, skip_serializing_if = "is_zero_i8")]
    pub distance_to: i8,
    /// Skill installed on equip to resolve this card leaving the equipment zone.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub on_leave_equip_skill_def: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

fn is_zero_i8(value: &i8) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero_hash(value: &[u8; 32]) -> bool {
    *value == [0u8; 32]
}

/// Modifier index entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageModifierEntry {
    /// Skill def id.
    pub skill_def_id: u32,
    /// Target channel.
    pub target: String,
    /// Optional card selector for card-aware targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Mode.
    pub mode: String,
    /// Optional query-time value source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_source: Option<String>,
    /// Value.
    pub value: i32,
}

/// Active skill index entry (player-activated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSelectionSpec {
    /// Source zone kind (`hand` or `hand_equip`).
    pub zone: String,
    /// Minimum selected cards.
    pub min: u8,
    /// Maximum selected cards.
    pub max: u8,
    /// Whether the activation may be cancelled at the selection barrier.
    #[serde(default)]
    pub allow_cancel: bool,
    /// Whether every selected card must have the same effective suit.
    #[serde(default, skip_serializing_if = "is_false")]
    pub same_suit: bool,
    /// Whether every selected card must have the same effective color.
    #[serde(default)]
    pub same_color: bool,
    /// Whether selected cards must have pairwise-distinct effective suits.
    #[serde(default, skip_serializing_if = "is_false")]
    pub distinct_suits: bool,
    /// Whether selected cards must have pairwise-distinct logical categories.
    #[serde(default, skip_serializing_if = "is_false")]
    pub distinct_categories: bool,
    /// Optional selected-card kind filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Reject logical card categories already recorded by this skill in the current turn.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unused_category_by_skill_this_turn: bool,
    /// Reject logical card categories already recorded by this skill in the current play phase.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unused_category_by_skill_this_play_phase: bool,
    /// Reject logical card categories used or discarded by the owner in the current turn.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unused_category_by_owner_history_this_turn: bool,
    /// Require selected cards' effective numbers to total exactly this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum_effective_number_exact: Option<u16>,
}

/// Player-target contract for an active skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTargetSpec {
    /// Minimum selected players.
    pub min: u8,
    /// Maximum selected players.
    pub max: u8,
    /// Whether the skill owner may be selected.
    #[serde(default = "default_true")]
    pub allow_self: bool,
    /// Whether selected targets must tie for the lowest current HP.
    #[serde(default)]
    pub lowest_hp: bool,
    /// Whether selected targets must tie for the highest current HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub highest_hp: bool,
    /// Whether selected targets must currently be unlinked.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unlinked: bool,
    /// Whether selected targets must currently be linked.
    #[serde(default, skip_serializing_if = "is_false")]
    pub linked: bool,
    /// Optional owner mark whose count caps the selected-player count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_by_owner_mark: Option<u32>,
    /// Cap the selected-player count by the owner's current lost HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub max_by_owner_lost_hp: bool,
    /// Cap the selected-player count by the owner's current HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub max_by_owner_hp: bool,
    /// Whether selected targets must currently be wounded.
    #[serde(default, skip_serializing_if = "is_false")]
    pub wounded: bool,
    /// Whether selected targets must be male or double-sex.
    #[serde(default, skip_serializing_if = "is_false")]
    pub male: bool,
    /// Whether selected targets must have at least the skill owner's current HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hp_at_least_owner: bool,
    /// Whether selected targets must have greater current HP than the skill owner.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hp_gt_owner: bool,
    /// Whether selected targets must have at most the skill owner's current HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hp_lte_owner: bool,
    /// Optional exclusive lower bound for selected targets' current HP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_gt: Option<u8>,
    /// Optional exclusive lower bound for selected targets' maximum HP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_hp_gt: Option<u8>,
    /// Whether selected targets must have more hand cards than the skill owner.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_gt_owner: bool,
    /// Whether selected targets may have no more hand cards than the skill owner.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_lte_owner: bool,
    /// Require exactly two targets whose hand-card counts differ by at most three.
    #[serde(default, skip_serializing_if = "is_false")]
    pub pair_hand_difference_lte_three: bool,
    /// Optional exclusive upper bound for selected targets' hand-card count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_lt: Option<u8>,
    /// Require every selected target to have at least one hand card.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_nonempty: bool,
    /// Whether the owner and selected targets must each have a hand card to compare.
    #[serde(default, skip_serializing_if = "is_false")]
    pub can_compare: bool,
    /// Optional source-free card kind every selected target must be able to use on the owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_use_on_owner: Option<String>,
    /// Require the second selected target to be able to use a source-free Duel on the first.
    #[serde(default, skip_serializing_if = "is_false")]
    pub second_can_use_juedou_on_first: bool,
    /// Require exactly two targets with discardable equipment and at least one discardable armor.
    #[serde(default, skip_serializing_if = "is_false")]
    pub equipment_pair_with_armor: bool,
    /// Require every selected target to currently have an equipped card.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently discard.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_discardable_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently gain.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_gainable_equip: bool,
    /// Require every selected target to have a hand/equipment card the owner may currently gain.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_gainable_hand_equip: bool,
    /// Require every selected target to be inside the skill owner's attack range.
    #[serde(default, skip_serializing_if = "is_false")]
    pub in_attack_range: bool,
    /// Require the skill owner to be inside every selected target's attack range.
    #[serde(default, skip_serializing_if = "is_false")]
    pub owner_in_target_attack_range: bool,
    /// Require every selected target to have at least one hand/equipment card.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_equip_nonempty: bool,
    /// Require every selected target to have a hand/equipment card the owner may discard.
    #[serde(default, skip_serializing_if = "is_false")]
    pub has_discardable_hand_equip: bool,
    /// Reject targets already recorded by this skill in the current turn.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unused_by_skill_this_turn: bool,
    /// Reject targets already recorded by this skill in the current play phase.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unused_by_skill_this_play_phase: bool,
    /// Require selected-player and selected-card counts to match.
    #[serde(default, skip_serializing_if = "is_false")]
    pub match_selected_card_count: bool,
    /// Require selected cards to match the sole target's equipment-card count.
    #[serde(default, skip_serializing_if = "is_false")]
    pub selected_cards_match_target_equip_count: bool,
}

fn default_true() -> bool {
    true
}

/// Active skill index entry (player-activated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageActiveEntry {
    /// Skill def id.
    pub skill_def_id: u32,
    /// Window: play_phase | any
    pub window: String,
    /// Module name.
    pub module: String,
    /// Entry PC.
    pub entry_pc: u32,
    /// Optional declarative card selection passed to the VM body.
    #[serde(default)]
    pub selection: Option<PackageSelectionSpec>,
    /// Optional declarative player-target contract.
    #[serde(default)]
    pub targets: Option<PackageTargetSpec>,
    /// Optional authoritative activation limit reset at turn start.
    #[serde(default)]
    pub uses_per_turn: Option<u8>,
    /// Optional authoritative activation limit reset at each Play phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_play_phase: Option<u8>,
    /// Optional dynamic per-turn activation limit source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn_source: Option<String>,
    /// Optional owner-mark threshold required before activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_at_least: Option<(u32, u32)>,
    /// Require the skill owner not to have this skill definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_missing_skill: Option<u32>,
    /// Require the skill owner's hand count to exceed current HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_gt_hp: bool,
    /// Require the skill owner's hand count to exceed maximum HP.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_gt_max_hp: bool,
    /// Require the skill owner's current hand limit to be positive.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_limit_positive: bool,
    /// Require at least one card in the skill owner's equipment zone.
    #[serde(default, skip_serializing_if = "is_false")]
    pub equip_nonempty: bool,
    /// Require at least one card in the skill owner's hand.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hand_nonempty: bool,
    /// Require the skill owner to be face up.
    #[serde(default, skip_serializing_if = "is_false")]
    pub face_up: bool,
    /// Require the skill owner's hand to hold no card carrying this gaintag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_missing_gaintag: Option<u32>,
    /// Require the skill owner not to be linked.
    #[serde(default, skip_serializing_if = "is_false")]
    pub owner_unlinked: bool,
    /// Allow this active skill to resolve at most once during the match.
    #[serde(default, skip_serializing_if = "is_false")]
    pub once_per_game: bool,
}

/// View-as skill index entry (play card as another kind).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageViewAsEntry {
    /// Skill def id.
    pub skill_def_id: u32,
    /// Treated as card kind tag string: sha|shan|tao|jiu
    pub as_kind: String,
    /// Window: play_phase | any
    pub window: String,
    /// Filter: colors, suits, concrete basic cards, stable card categories, or owner-state predicates.
    pub filter: String,
    /// Optional source zone; omitted means hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone: Option<String>,
    /// Optional authoritative per-turn activation limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn: Option<u8>,
    /// Whether ordinary card distance checks are ignored.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ignore_distance: bool,
    /// Whether a red source makes the converted Sha unrespondable.
    #[serde(default, skip_serializing_if = "is_false")]
    pub red_unrespondable: bool,
    /// Whether a black source avoids consuming the ordinary Sha use count.
    #[serde(default, skip_serializing_if = "is_false")]
    pub black_no_use_count: bool,
    /// Whether this converted Sha always avoids consuming the ordinary Sha use count.
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_use_count: bool,
    /// Make each target lacking a hand card of the source suit unable to respond.
    #[serde(default, skip_serializing_if = "is_false")]
    pub target_lacks_source_suit_unrespondable: bool,
    /// Refund the ordinary Sha use count after this exact converted Sha deals no damage.
    #[serde(default, skip_serializing_if = "is_false")]
    pub refund_use_count_if_no_damage: bool,
    /// Minimum number of physical source cards.
    #[serde(default = "default_one_u8", skip_serializing_if = "is_one_u8")]
    pub min_cards: u8,
    /// Maximum number of physical source cards.
    #[serde(default = "default_one_u8", skip_serializing_if = "is_one_u8")]
    pub max_cards: u8,
    /// Require exactly max(1, current HP) physical source cards.
    #[serde(default, skip_serializing_if = "is_false")]
    pub owner_hp_min_one_cards: bool,
    /// Minimum physical source cards = 1 + lifetime basic-card uses (max uncapped).
    #[serde(default, skip_serializing_if = "is_false")]
    pub owner_basic_uses_plus_one_cards: bool,
    /// Each view_as rule of this skill may be used once per turn (per-rule
    /// branch bitmap; entity-source conversions included).
    #[serde(default, skip_serializing_if = "is_false")]
    pub once_per_turn_per_branch: bool,
    /// Require equal effective suits for all selected source cards.
    #[serde(default, skip_serializing_if = "is_false")]
    pub same_suit: bool,
    /// Optional logical card nature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nature: Option<String>,
    /// Optional effect applied when multiple source cards are selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_card_bonus: Option<String>,
    /// Conversion branch required for a source-free family rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<u8>,
    /// Whether this branch may be used only once in each round.
    #[serde(default, skip_serializing_if = "is_false")]
    pub once_per_round: bool,
    /// Branch selected after a successful conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_branch: Option<u8>,
    /// Declarative source-free conversion cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    /// Skill-definition mark consumed by the conversion cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_cost: Option<u32>,
    /// Cards drawn after the converted card is used or responded with.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub post_draw: u8,
}

fn default_one_u8() -> u8 {
    1
}

fn is_one_u8(value: &u8) -> bool {
    *value == 1
}

/// Full content package (Phase 4: JSON-serializable; production may use pb+bin).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackage {
    /// Manifest.
    pub manifest: PackageManifest,
    /// Bytecode modules.
    pub modules: Vec<BytecodeModule>,
    /// Skill metadata index.
    #[serde(default)]
    pub skill_index: Vec<PackageSkillEntry>,
    /// Character roster shipped by this package.
    #[serde(default)]
    pub character_index: Vec<PackageCharacterEntry>,
    /// Presentation-only skin overlays, including overlays for dependency-package characters.
    #[serde(default, skip_serializing_if = "PackageSkinIndex::is_empty")]
    pub skin_index: PackageSkinIndex,
    /// Immutable card definitions shipped by this package.
    #[serde(default)]
    pub card_index: Vec<PackageCardEntry>,
    /// Trigger static index.
    pub trigger_index: Vec<PackageTriggerEntry>,
    /// Modifier static index.
    pub modifier_index: Vec<PackageModifierEntry>,
    /// Active skill static index.
    #[serde(default)]
    pub active_index: Vec<PackageActiveEntry>,
    /// View-as skill static index.
    #[serde(default)]
    pub view_as_index: Vec<PackageViewAsEntry>,
    /// Locale → translation key → translated text.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
    /// Build-time/runtime asset metadata for this package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_manifest: Option<resource_schema::AssetManifest>,
    /// Logical asset id → safe path relative to an installed extension directory.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub asset_paths: BTreeMap<String, String>,
    /// Compiler version string (semantic).
    pub compiler_version: String,
}

/// Verify the existing fingerprint of a VM module against its semantic fields.
pub fn validate_vm_module_hash(module: &BytecodeModule) -> Result<(), String> {
    let expected = BytecodeModule::compute_logic_hash(
        &module.name,
        &module.code,
        module.local_count,
        module.caps_mask,
        module.entry_pc,
    );
    if module.logic_hash != expected {
        return Err(format!("module {}: logic_hash mismatch", module.name));
    }
    Ok(())
}

impl ContentPackage {
    /// Reject packages whose effect tags cannot be interpreted under their declared ABI.
    pub fn validate_vm_effect_abi(&self) -> Result<(), String> {
        use rule_vm::bytecode::effect_tag;

        let abi = self.manifest.rule_vm_abi_version;
        for module in &self.modules {
            if (754..=757).contains(&abi)
                && rule_vm::module_uses_effect_tag(module, effect_tag::GAIN_OTHER_SKILL_PILE_CARDS)
                    .map_err(|e| format!("module {}: {e}", module.name))?
            {
                return Err(format!(
                    "module {} uses ambiguous effect 656 with ABI {abi}",
                    module.name
                ));
            }
            let uses_772 = rule_vm::module_uses_effect_tag(
                module,
                effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW,
            )
            .map_err(|e| format!("module {}: {e}", module.name))?;
            if abi < 758 && uses_772 {
                return Err(format!("module {} uses effect 772 below ABI 758", module.name));
            }
        }
        Ok(())
    }

    /// Presentation-only skin overlays contributed by this package.
    pub fn skin_index(&self) -> &PackageSkinIndex {
        &self.skin_index
    }

    /// Mutable presentation-only skin overlays contributed by this package.
    pub fn skin_index_mut(&mut self) -> &mut PackageSkinIndex {
        &mut self.skin_index
    }

    /// Validate package identity, hashes, dependencies, and presentation references.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_vm_effect_abi()?;
        for (label, value) in [
            ("namespace", self.manifest.namespace.as_str()),
            ("package_id", self.manifest.package_id.as_str()),
            ("version", self.manifest.version.as_str()),
        ] {
            if value.is_empty()
                || value == "."
                || value == ".."
                || value.contains('/')
                || value.contains('\\')
            {
                return Err(format!("invalid {label} {value:?}"));
            }
        }
        if self.manifest.logic_hash != self.compute_logic_hash() {
            return Err("logic_hash mismatch".into());
        }
        if self.manifest.asset_hash != self.compute_asset_hash() {
            return Err("asset_hash mismatch".into());
        }
        self.modules.iter().try_for_each(validate_vm_module_hash)?;

        let mut dependency_slots = std::collections::BTreeSet::new();
        for dependency in &self.manifest.dependencies {
            for (label, value) in [
                ("dependency namespace", dependency.namespace.as_str()),
                ("dependency package_id", dependency.package_id.as_str()),
                ("dependency version", dependency.version.as_str()),
            ] {
                if value.is_empty()
                    || value == "."
                    || value == ".."
                    || value.contains('/')
                    || value.contains('\\')
                {
                    return Err(format!("invalid {label} {value:?}"));
                }
            }
            let slot = (&dependency.namespace, &dependency.package_id);
            if !dependency_slots.insert(slot) {
                return Err(format!(
                    "duplicate dependency {}/{}",
                    dependency.namespace, dependency.package_id
                ));
            }
            if dependency.namespace == self.manifest.namespace
                && dependency.package_id == self.manifest.package_id
            {
                return Err("package must not depend on itself".into());
            }
        }

        for (asset_id, relative) in &self.asset_paths {
            if asset_id.is_empty() || !is_safe_relative_path(relative) {
                return Err(format!("invalid asset path {asset_id}={relative:?}"));
            }
        }
        let mut asset_kinds = BTreeMap::new();
        if let Some(manifest) = &self.asset_manifest {
            if manifest.package_id != self.manifest.package_id {
                return Err("asset manifest package_id mismatch".into());
            }
            for entry in &manifest.entries {
                if entry.asset_id.is_empty()
                    || asset_kinds
                        .insert(entry.asset_id.as_str(), entry.kind)
                        .is_some()
                {
                    return Err(format!("duplicate asset id {}", entry.asset_id));
                }
                if !self.asset_paths.contains_key(&entry.asset_id) {
                    return Err(format!("missing path for asset {}", entry.asset_id));
                }
            }
            for entry in &manifest.entries {
                let mut dependencies = std::collections::BTreeSet::new();
                for dependency in &entry.depends_on {
                    if dependency == &entry.asset_id
                        || !dependencies.insert(dependency.as_str())
                        || !asset_kinds.contains_key(dependency.as_str())
                    {
                        return Err(format!(
                            "asset {} has invalid dependency {dependency}",
                            entry.asset_id
                        ));
                    }
                }
            }
            if let Some(unknown) = self
                .asset_paths
                .keys()
                .find(|id| !asset_kinds.contains_key(id.as_str()))
            {
                return Err(format!("asset path has no manifest entry: {unknown}"));
            }
        } else if !self.asset_paths.is_empty() {
            return Err("asset_paths require asset_manifest".into());
        }
        for character in &self.character_index {
            for asset_id in character
                .portrait_asset
                .iter()
                .chain(character.skin_assets.iter())
            {
                if !self.asset_paths.contains_key(asset_id) {
                    return Err(format!(
                        "character {} references unknown asset {asset_id}",
                        character.id
                    ));
                }
            }
        }
        self.validate_skin_index(&asset_kinds)?;
        let mut card_defs = std::collections::BTreeSet::new();
        for card in &self.card_index {
            if card.card_def_id == 0 || card.name.is_empty() || !card_defs.insert(card.card_def_id)
            {
                return Err(format!(
                    "invalid or duplicate card definition {}",
                    card.card_def_id
                ));
            }
            if !matches!(
                card.kind.as_str(),
                "weapon"
                    | "armor"
                    | "horse_def"
                    | "horse_off"
                    | "treasure"
                    | "mantianguohai"
                    | "youdishenru"
                    | "diaobingqianjiang"
                    | "wangmeizhike"
            ) {
                return Err(format!(
                    "card definition {} has unsupported kind {}",
                    card.card_def_id, card.kind
                ));
            }
            if card.kind != "weapon" && card.weapon_range != 0 {
                return Err(format!(
                    "non-weapon card definition {} has weapon range",
                    card.card_def_id
                ));
            }
        }
        Ok(())
    }

    fn validate_skin_index(
        &self,
        asset_kinds: &BTreeMap<&str, resource_schema::AssetKind>,
    ) -> Result<(), String> {
        use resource_schema::AssetKind;

        let mut skin_ids = std::collections::BTreeSet::new();
        for skin in &self.skin_index().skins {
            if skin.id == 0 || !skin_ids.insert(skin.id) {
                return Err(format!("invalid or duplicate skin id {}", skin.id));
            }
            if skin.character_id == 0 {
                return Err(format!("skin {} has invalid character id", skin.id));
            }
            if !is_safe_presentation_token(&skin.name_key) {
                return Err(format!("skin {} has invalid name key", skin.id));
            }
            for (voice_key, asset_id) in &skin.voice_assets {
                if !is_safe_presentation_token(voice_key) {
                    return Err(format!("skin {} has invalid voice key", skin.id));
                }
                require_skin_asset(asset_kinds, skin.id, asset_id, AssetKind::Audio)?;
            }
            match &skin.presentation {
                PackageSkinPresentation::Static { asset } => {
                    require_skin_asset(asset_kinds, skin.id, asset, AssetKind::Image)?;
                }
                PackageSkinPresentation::Spine {
                    bundle,
                    actions,
                    layout,
                    dependencies,
                } => {
                    require_skin_asset(asset_kinds, skin.id, bundle, AssetKind::Data)?;
                    if actions.len() > 64 {
                        return Err(format!("skin {} has too many Spine actions", skin.id));
                    }
                    for (logical, animation) in actions {
                        if !is_safe_presentation_token(logical)
                            || !is_safe_presentation_token(animation)
                        {
                            return Err(format!("skin {} has unsafe Spine action", skin.id));
                        }
                    }
                    validate_skin_layout(skin.id, layout)?;
                    if dependencies.len() > 256 {
                        return Err(format!("skin {} has too many dependencies", skin.id));
                    }
                    let mut seen = std::collections::BTreeSet::new();
                    for dependency in dependencies {
                        if dependency == bundle
                            || !seen.insert(dependency.as_str())
                            || !asset_kinds.contains_key(dependency.as_str())
                        {
                            return Err(format!(
                                "skin {} has invalid dependency {dependency}",
                                skin.id
                            ));
                        }
                    }
                }
                PackageSkinPresentation::Substitute { rule, asset } => {
                    if !is_safe_presentation_token(rule) {
                        return Err(format!("skin {} has unsafe substitute rule", skin.id));
                    }
                    require_skin_asset(asset_kinds, skin.id, asset, AssetKind::Image)?;
                }
            }
        }

        let mut alias_targets = std::collections::BTreeSet::new();
        for alias in &self.skin_index().aliases {
            if alias.character_id == 0
                || alias.source_character_id == 0
                || alias.character_id == alias.source_character_id
                || !alias_targets.insert(alias.character_id)
            {
                return Err(format!(
                    "invalid or duplicate skin alias for character {}",
                    alias.character_id
                ));
            }
            for (target, source) in &alias.voice_mappings {
                if !is_safe_presentation_token(target) || !is_safe_presentation_token(source) {
                    return Err(format!(
                        "skin alias for character {} has unsafe voice mapping",
                        alias.character_id
                    ));
                }
            }
        }
        Ok(())
    }

    /// Serialize to JSON bytes (deterministic field order via serde_json).
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize.
    pub fn from_json_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

fn require_skin_asset(
    asset_kinds: &BTreeMap<&str, resource_schema::AssetKind>,
    skin_id: u32,
    asset_id: &str,
    expected: resource_schema::AssetKind,
) -> Result<(), String> {
    match asset_kinds.get(asset_id) {
        Some(kind) if *kind == expected => Ok(()),
        Some(kind) => Err(format!(
            "skin {skin_id} asset {asset_id} has kind {kind:?}, expected {expected:?}"
        )),
        None => Err(format!(
            "skin {skin_id} references unknown asset {asset_id}"
        )),
    }
}

fn validate_skin_layout(skin_id: u32, layout: &PackageSkinLayout) -> Result<(), String> {
    if !(-8192..=8192).contains(&layout.x_pixels)
        || !(-8192..=8192).contains(&layout.y_pixels)
        || layout.anchor_x_permille > 1000
        || layout.anchor_y_permille > 1000
        || !(1..=10_000).contains(&layout.scale_permille)
        || !(-360..=360).contains(&layout.rotation_degrees)
        || !(1..=8_000).contains(&layout.speed_permille)
    {
        return Err(format!("skin {skin_id} has unsafe Spine layout"));
    }
    Ok(())
}

fn is_safe_presentation_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':' | '#' | '@'))
}

fn is_safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
