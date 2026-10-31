//! Deterministic game core for noname-next.
//!
//! Dependencies allowed: `engine-api`, `rule-api`, `rule-vm`, pure compute crates.
//! Forbidden: DOM, network, filesystem, wall-clock, host threads affecting rule order.

#![deny(missing_docs)]

pub mod capsule;
pub mod command;
pub mod content;
pub mod effect;
pub mod engine;
pub mod hash;
pub mod intent;
pub mod invariants;
pub mod mode;
pub mod projection;
pub mod query;
pub mod replay;
pub mod rng;
pub mod runner;
pub mod snapshot;
pub mod state;
pub mod transaction;
pub mod trigger;
pub mod vm_host;

pub use capsule::{
    CapsuleCompat, CapsuleContentEntry, CapsuleError, CapsuleMode, ReplayCapsule,
    CAPSULE_SCHEMA_VERSION,
};
pub use content::{
    jianxiong_skill_def, ContentLoadError, ContentRegistry, LoadedPackageMeta,
    PACKAGE_MODIFIER_ID_BASE, PACKAGE_TRIGGER_ID_BASE,
};
pub use engine::Engine;
pub use hash::canonical_state_hash;
pub use mode::{check_identity_victory, distribute_identities};
pub use projection::{project_observer, project_player};
pub use query::{query, query_explain, QueryContext, QueryExplanation};
pub use query::{CombineMode, ModifierDefinition, ModifierTarget};
pub use replay::{
    AcceptedCommand, RecordHash, ReplayFileError, ReplayHeader, ReplayLog, ResolutionOutcome,
    ResolutionRecord, RuleRngCheckpoint,
};
pub use rng::RuleRng;
pub use runner::{choose_command, run_guozhan_match, run_identity_match, MatchConfig, MatchReport};
pub use snapshot::{Snapshot, SnapshotError};
pub use state::{CardKind, ContinuationKind, EquipSlot, GameState, Identity, Phase};
pub use trigger::{
    ActiveTriggerIndex, EventKind, TimingBand, TriggerOrderKey, BOOTSTRAP_JIANXIONG,
    BOOTSTRAP_MASHU, BOOTSTRAP_QICAO,
};

/// Re-export public API types commonly needed by hosts.
pub use engine_api;
pub use rule_api;
pub use rule_vm;
