//! Cross-process public API for noname-next.
//!
//! Defines Command, Choice, Observer View, DomainEvent and stable entity IDs.
//! Does **not** expose GameState internal layout (that lives in `engine-core`).

#![deny(missing_docs)]

mod choice;
mod command;
mod event;
mod ids;
mod manifest;
mod outcome;
mod version;
mod view;

pub use choice::*;
pub use command::*;
pub use event::*;
pub use ids::*;
pub use manifest::*;
pub use outcome::*;
pub use version::*;
pub use view::*;

/// Engine build identity baked into replays and cross-target checks.
pub const ENGINE_BUILD_ID: &str = "noname-next-0.1.0-m2";

/// Snapshot binary format version.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 2;

/// Replay schema version.
pub const REPLAY_SCHEMA_VERSION: u32 = 1;
