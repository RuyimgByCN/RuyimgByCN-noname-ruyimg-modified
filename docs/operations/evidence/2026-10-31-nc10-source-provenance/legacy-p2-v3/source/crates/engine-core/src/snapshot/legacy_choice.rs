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
    without_source(choice).serialize(serializer)
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
#[derive(Deserialize)]
#[serde(remote = "ChoiceRequest", rename = "ChoiceRequest")]
struct ArchiveChoice {
    choice_id: ChoiceId,
    observer: PlayerId,
    options: Vec<ChoiceOption>,
    min_count: u8,
    max_count: u8,
    allow_cancel: bool,
    timeout_policy: TimeoutPolicy,
    #[serde(default)]
    deadline_unix_ms: Option<u64>,
    revision: u64,
    continuation_id: u64,
    kind: ChoiceKind,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default, deserialize_with = "deserialize_native_kind")]
    native_kind: Option<String>,
    #[serde(skip)]
    source_skill: Option<String>,
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
