use engine_api::{PublicPresentationBatch, MAX_PRESENTATION_BYTES};
use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize_optional<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<PublicPresentationBatch>, D::Error> {
    let batch = PublicPresentationBatch::deserialize(d)?;
    let bytes = serde_json::to_vec(&batch).map_err(serde::de::Error::custom)?;
    if bytes.len() > MAX_PRESENTATION_BYTES {
        return Err(serde::de::Error::custom(
            "public presentation batch exceeds 64KiB",
        ));
    }
    Ok(Some(batch))
}

pub(crate) fn validate(message: &crate::ServerMessage) -> Result<(), crate::wire::WireError> {
    if let crate::ServerMessage::CommandOutcome {
        view,
        presentation: Some(batch),
        ..
    } = message
    {
        batch.validate().map_err(crate::wire::WireError::Json)?;
        batch
            .validate_players(view.seats.len() as u32)
            .map_err(crate::wire::WireError::Json)?;
        if batch.revision != view.revision
            || serde_json::to_vec(batch)
                .map_err(|e| crate::wire::WireError::Json(e.to_string()))?
                .len()
                > MAX_PRESENTATION_BYTES
        {
            return Err(crate::wire::WireError::Json(
                "invalid public presentation revision or size".into(),
            ));
        }
    }
    Ok(())
}
