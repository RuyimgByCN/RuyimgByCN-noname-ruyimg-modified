//! Runtime skill state for `general_slot_none` and related helpers.

use super::*;

/// Default `general_slot`: unattributed (identity mode never sets it).
pub(crate) fn general_slot_none() -> u8 {
    GENERAL_SLOT_NONE
}

pub(crate) fn no_player() -> PlayerId {
    PlayerId::NONE
}

pub(crate) fn no_card() -> CardId {
    CardId::NONE
}
