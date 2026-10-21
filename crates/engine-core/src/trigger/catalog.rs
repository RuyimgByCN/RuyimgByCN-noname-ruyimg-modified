//! Static trigger definitions (content-compiler will populate later).

use super::{EventKind, TimingBand};
use content_schema::TriggerChoiceActor;
use engine_api::SkillDefId;
use serde::{Deserialize, Serialize};

/// Trigger definition id (stable across content packages once assigned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TriggerDefId(pub u32);

impl TriggerDefId {
    /// Raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

// Collect complete variants before declaring the enum; their order is part of the wire format.
mod owner_alive_other_phase_end_mobiledanshou;
mod owner_is_event_player;

macro_rules! collect_variants {
    ([$head:ident $(, $tail:ident)*] [$($variants:tt)*]) => {
        $head::variants! { [$($tail),*] [$($variants)*] }
    };
    ([] [$($variants:tt)*]) => {
        /// Who must match for the trigger to fire.
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum TriggerFilter { $($variants)* }
    };
}
collect_variants! { [owner_is_event_player, owner_alive_other_phase_end_mobiledanshou] [] }

/// Handlers: native primitives or Rule VM bytecode (Phase 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerHandler {
    /// Draw `count` cards for the skill owner.
    DrawSelf {
        /// Cards to draw.
        count: u32,
    },
    /// Emit a rule note (test / marker).
    EmitNote {
        /// Code.
        code: u32,
    },
    /// Recover HP for owner.
    RecoverSelf {
        /// Amount.
        amount: u32,
    },
    /// Run pre-verified bytecode module (same VM path for official & community).
    RunVm {
        /// Module name / key in the content registry.
        module: String,
        /// Entry PC.
        entry_pc: u32,
    },
}

/// One static trigger definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDefinition {
    /// Definition id.
    pub id: TriggerDefId,
    /// Skill definition this trigger belongs to.
    pub skill_def: SkillDefId,
    /// Event that wakes this trigger.
    pub event: EventKind,
    /// Timing band.
    pub timing: TimingBand,
    /// Priority (lower first).
    pub priority: i32,
    /// Optional triggers may later open ChoiceRequest; mandatory triggers auto-run.
    pub optional: bool,
    /// Player who confirms an optional trigger.
    pub choice_actor: TriggerChoiceActor,
    /// Filter.
    pub filter: TriggerFilter,
    /// Max fires of this def+instance per resolution segment (anti-loop).
    pub max_per_segment: u32,
    /// Handler.
    pub handler: TriggerHandler,
}

/// Bootstrap skill def: 奸雄 — after taking damage, draw 1.
pub const BOOTSTRAP_JIANXIONG: u32 = 1;
/// Bootstrap skill def: 马术 — distance from owner -1 (modifier, no trigger).
pub const BOOTSTRAP_MASHU: u32 = 2;
/// Bootstrap skill def: 奇策 placeholder range +1 (modifier).
pub const BOOTSTRAP_QICAO: u32 = 3;

/// Bootstrap trigger table (sorted by id).
pub fn bootstrap_trigger_defs() -> Vec<TriggerDefinition> {
    vec![TriggerDefinition {
        id: TriggerDefId(1),
        skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        event: EventKind::Damage,
        timing: TimingBand::Normal,
        priority: 0,
        optional: false,
        choice_actor: TriggerChoiceActor::Owner,
        filter: TriggerFilter::OwnerIsEventPlayer,
        max_per_segment: 8,
        handler: TriggerHandler::DrawSelf { count: 1 },
    }]
}

/// Lookup defs for a skill definition.
pub fn defs_for_skill(skill_def: SkillDefId) -> Vec<TriggerDefinition> {
    bootstrap_trigger_defs()
        .into_iter()
        .filter(|d| d.skill_def == skill_def)
        .collect()
}

/// All bootstrap defs for an event kind (content index preview).
#[allow(dead_code)]
pub fn defs_for_event(event: EventKind) -> Vec<TriggerDefinition> {
    bootstrap_trigger_defs()
        .into_iter()
        .filter(|d| d.event == event)
        .collect()
}
