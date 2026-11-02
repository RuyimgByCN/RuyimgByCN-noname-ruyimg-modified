//! Archive-only source stripping; wire ChoiceRequest keeps its strict validator.
use engine_api::{ChoiceId, ChoiceKind, ChoiceOption, ChoiceRequest, PlayerId, TimeoutPolicy};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub(crate) fn without_source(choice: &Option<ChoiceRequest>) -> Option<ChoiceRequest> {
    choice.clone().map(|mut choice| {
        choice.source_skill = None;
        choice
    })
}
pub(super) fn serialize<S: Serializer>(
    choice: &Option<ChoiceRequest>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    struct Legacy(#[serde(with = "ArchiveChoice")] ChoiceRequest);
    without_source(choice).map(Legacy).serialize(serializer)
}
pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ChoiceRequest>, D::Error> {
    struct Optional;
    impl<'de> Visitor<'de> for Optional {
        type Value = Option<ChoiceRequest>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("option")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            ArchiveChoice::deserialize(deserializer).map(Some)
        }
    }
    deserializer.deserialize_option(Optional)
}

// Keep the old 13-field visitor and diagnostics; skipped source is an unknown JSON key.
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(remote = "ChoiceRequest", rename = "ChoiceRequest")]
struct ArchiveChoice {
    choice_id: ChoiceId,
    observer: PlayerId,
    options: Vec<ChoiceOption>,
    #[serde(serialize_with = "serialize_count", deserialize_with = "deserialize_count")]
    min_count: u32,
    #[serde(serialize_with = "serialize_count", deserialize_with = "deserialize_count")]
    max_count: u32,
    allow_cancel: bool,
    timeout_policy: TimeoutPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deadline_unix_ms: Option<u64>,
    revision: u64,
    continuation_id: u64,
    kind: ChoiceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_native_kind")]
    native_kind: Option<String>,
    #[serde(skip)]
    source_skill: Option<String>,
}

fn serialize_count<S: Serializer>(value: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    let count = u8::try_from(*value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_u8(count)
}
fn deserialize_count<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    u8::deserialize(deserializer).map(u32::from)
}

fn deserialize_native_kind<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(deserializer)?;
    if value
        .as_deref()
        .is_some_and(|kind| engine_api::choice_kind_from_native(kind).is_none())
    {
        return Err(serde::de::Error::custom("invalid native_kind"));
    }
    Ok(value)
}
