//! Host capability whitelist for the Rule VM.

use serde::{Deserialize, Serialize};

/// Capabilities a content package may request.
///
/// Anything not listed is denied by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    /// May emit primitive effects.
    pub emit_effects: bool,
    /// May run typed queries.
    pub run_queries: bool,
    /// May request choices.
    pub request_choice: bool,
    /// May read own skill state.
    pub read_skill_state: bool,
    /// May write own skill state.
    pub write_skill_state: bool,
    /// May read marks on self.
    pub read_marks_self: bool,
    /// May read public seat info.
    pub read_public_seats: bool,
    /// Forbidden: network, fs, clock, threads, DOM — never granted.
    pub _reserved: (),
}

impl CapabilitySet {
    /// Standard official skill capabilities.
    pub fn standard_skill() -> Self {
        Self {
            emit_effects: true,
            run_queries: true,
            request_choice: true,
            read_skill_state: true,
            write_skill_state: true,
            read_marks_self: true,
            read_public_seats: true,
            _reserved: (),
        }
    }

    /// Empty / sandbox default.
    pub fn none() -> Self {
        Self::default()
    }
}

/// Host trap reasons (deterministic).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostTrap {
    /// Capability not granted.
    CapabilityDenied {
        /// Capability name.
        name: String,
    },
    /// Fuel exhausted.
    OutOfFuel,
    /// Stack depth exceeded.
    StackOverflow,
    /// Heap / collection limit.
    MemoryLimit,
    /// Invalid effect / query.
    InvalidRequest {
        /// Message.
        message: String,
    },
}
