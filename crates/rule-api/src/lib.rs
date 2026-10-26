//! Rule VM–visible stable surface.
//!
//! Defines Effect requests, Query requests, and host capabilities.
//! Does **not** expose `GameState` internal layout.

#![deny(missing_docs)]
// One collection step per ordered effect segment keeps the flat Serde enum stable.
#![recursion_limit = "2048"]

mod effect;
mod host;
mod query;

pub use effect::*;
pub use host::*;
pub use query::*;

// Re-export entity ids for content/VM convenience without pulling GameState.
pub use engine_api::{
    CardDefId, CardId, CharacterDefId, ChoiceId, MarkDefId, PlayerId, SeatIndex, SkillDefId,
    SkillId, Suit, ZoneId,
};
