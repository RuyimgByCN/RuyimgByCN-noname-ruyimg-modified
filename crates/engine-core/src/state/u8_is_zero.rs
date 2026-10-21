//! Runtime skill state for `u8_is_zero` and related helpers.

use super::*;

pub(crate) fn u8_is_zero(value: &u8) -> bool {
    *value == 0
}

pub(crate) fn u32_is_zero(value: &u32) -> bool {
    *value == 0
}

/// Canonical authoritative state.
///
/// Only fields that affect future rule results belong here.
/// UI, network, caches, and presentation state are forbidden.
pub(crate) fn serialize_arc_content<S: serde::Serializer>(
    c: &std::sync::Arc<ContentRegistry>,
    s: S,
) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(&**c, s)
}

pub(crate) fn deserialize_arc_content<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<std::sync::Arc<ContentRegistry>, D::Error> {
    let registry: ContentRegistry = serde::Deserialize::deserialize(d)?;
    Ok(std::sync::Arc::new(registry))
}
