//! Isolated AI simulation API (Phase 8).
//!
//! ```text
//! InformationSetView
//!   → Legal Action Generator
//!   → State Fork / Determinization
//!   → Heuristic / Beam / MCTS
//!   → Selected Command (with base_revision)
//! ```
//!
//! Search never mutates authority `Engine`. AI RNG ≠ RuleRng.
//! Stale results (authority advanced) must not be submitted.

#![deny(missing_docs)]

mod fork;
mod fork_strategy;
mod info_set;
mod legal;
mod rng;
mod search;
mod submit;

pub use engine_api::LegalAction;
pub use fork::StateFork;
pub use fork_strategy::{
    compare_strategies, render_comparison, ArenaFork, ArenaPool, CloneFork, CowFork, ForkMetrics,
    ForkStrategy, SnapshotDeltaFork, UndoJournalFork,
};
pub use info_set::{build_information_set, InformationSetView};
pub use legal::legal_actions;
pub use rng::SearchRng;
pub use search::{
    BeamSearch, HeuristicSearch, MctsSearch, SearchBudget, SearchPolicy, SearchResult,
};
pub use submit::{apply_if_fresh, AiDecision, AiError, AiRuntime};

/// Re-export for hosts.
pub use engine_api;
pub use engine_core;
