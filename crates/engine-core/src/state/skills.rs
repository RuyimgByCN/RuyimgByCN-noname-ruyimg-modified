//! Skill runtime instances.

use engine_api::{CardId, PlayerId, SkillDefId, SkillId};
use serde::{Deserialize, Serialize};

/// One skill instance on a player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillInstance {
    /// Stable instance id.
    pub id: SkillId,
    /// Definition.
    pub def: SkillDefId,
    /// Owner.
    pub owner: PlayerId,
    /// Enabled flag.
    pub enabled: bool,
    /// Use count this window (active skills).
    pub use_count: u32,
    /// Trigger fires this turn (`once_per_turn` filter); reset before each Prepare.
    #[serde(default)]
    pub turn_fire_count: u32,
    /// Trigger fires in the current play phase; reset on every real phase transition.
    #[serde(default)]
    pub play_phase_fire_count: u32,
    /// Active-skill invocations in the current play phase; reset on entry to each play phase.
    #[serde(default)]
    pub play_phase_use_count: u32,
    /// Effective basic/trick/equipment categories recorded in the current play phase.
    #[serde(default)]
    pub play_phase_category_mask: u8,
    /// Whether this active skill is banned until the current turn ends.
    #[serde(default)]
    pub active_turn_banned: bool,
    /// Whether the whole skill is disabled until the current turn ends.
    #[serde(default)]
    pub turn_disabled: bool,
    /// Skill instances whose presence currently disables this skill.
    #[serde(default)]
    pub disabled_by_skills: Vec<SkillId>,
    /// Turn owner whose turn end removes this temporary skill; none means permanent.
    #[serde(default = "no_player")]
    pub temporary_until_turn_end_of: PlayerId,
    /// Round in which a once-per-round trigger last fired; zero means never.
    #[serde(default)]
    pub round_fire_round: u32,
    /// Match-lifetime trigger fires (`once_per_game` filters); never reset.
    #[serde(default)]
    pub lifetime_fire_count: u32,
    /// Current branch for package-declared source-free card conversions.
    #[serde(default)]
    pub view_as_branch: u8,
    /// Round associated with `view_as_used_branches`.
    #[serde(default)]
    pub view_as_use_round: u32,
    /// Bitset of source-free conversion branches used in `view_as_use_round`.
    #[serde(default)]
    pub view_as_used_branches: u32,
    /// Content-owned persistent boolean flags; bits are cleared with the skill instance.
    #[serde(default)]
    pub content_flags: u64,
    /// Opaque skill state blob (content-defined; hashed).
    pub state_blob: Vec<u8>,
    /// Physical material selected for the next source-free conversion.
    #[serde(default = "no_card")]
    pub(crate) source_free_physical_card: CardId,
    /// Generic per-instance mark counter (content-defined; hashed).
    #[serde(default)]
    pub mark: u32,
    /// Generic per-instance keyed content storage (key→value, both u32). Cleared with the
    /// skill instance. Implements upstream `player.storage.<skill>` keyed maps used by skills
    /// such as clanbaichu (`storage.clanbaichu[suit+type] = trick_name`). Hash-neutral empty.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub content_storage: std::collections::BTreeMap<u32, u32>,
    /// Guozhan: which general this skill belongs to. 0 = main (主将), 1 = lieutenant (副将).
    /// `GENERAL_SLOT_NONE` (255) in identity mode or when unattributed. A face-down general's
    /// skills are gated by `skill_effectively_enabled`. See ADR 0598 phase C.
    #[serde(default = "general_slot_none")]
    pub general_slot: u8,
}

/// Sentinel for "skill not tied to a specific general" (identity mode).
pub const GENERAL_SLOT_NONE: u8 = 255;

/// Skill instance store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillRuntimeStore {
    next_id: u32,
    skills: Vec<Option<SkillInstance>>,
}

mod general_slot_none;
pub(crate) use general_slot_none::*;

mod content_flag;

mod twxiaolian_redirect_state_tag;
