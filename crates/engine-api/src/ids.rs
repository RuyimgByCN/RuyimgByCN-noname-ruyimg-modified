//! Stable numeric entity identifiers.
//!
//! IDs are never reused after free; deleted entities leave holes.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! entity_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $name {
            /// Null / unset sentinel. Never points at a live entity.
            pub const NONE: Self = Self(0);

            /// Create from raw value.
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            /// Raw numeric value.
            pub const fn raw(self) -> u32 {
                self.0
            }

            /// Whether this is the null sentinel.
            pub const fn is_none(self) -> bool {
                self.0 == 0
            }

            /// Whether this may reference a live entity.
            pub const fn is_some(self) -> bool {
                self.0 != 0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

entity_id!(
    /// Stable player seat / entity id.
    PlayerId
);
entity_id!(
    /// Stable card instance id.
    CardId
);
entity_id!(
    /// Stable skill instance id.
    SkillId
);
entity_id!(
    /// Stable zone id (deck, hand, discard, equip, …).
    ZoneId
);
entity_id!(
    /// Pending choice request id.
    ChoiceId
);
entity_id!(
    /// Content package / ruleset reference id.
    RulesetId
);
entity_id!(
    /// Card definition id (type, not instance).
    CardDefId
);
entity_id!(
    /// Character / general definition id.
    CharacterDefId
);
entity_id!(
    /// Skill definition id.
    SkillDefId
);
entity_id!(
    /// Mark / counter definition id.
    MarkDefId
);

/// Seat order index (0-based around the table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SeatIndex(pub u8);

impl SeatIndex {
    /// Create from raw seat index.
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    /// Raw value.
    pub const fn raw(self) -> u8 {
        self.0
    }
}
