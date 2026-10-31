//! JSON encode/decode with size limits.

use crate::messages::{ClientMessage, ServerMessage};
use crate::MAX_MESSAGE_BYTES;
use thiserror::Error;

/// Wire codec error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Message too large.
    #[error("message too large: {0} bytes")]
    TooLarge(usize),
    /// JSON error.
    #[error("json: {0}")]
    Json(String),
}

/// Encode client message.
pub fn encode_c2s(msg: &ClientMessage) -> Result<String, WireError> {
    let s = serde_json::to_string(msg).map_err(|e| WireError::Json(e.to_string()))?;
    if s.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge(s.len()));
    }
    Ok(s)
}

/// Decode client message.
pub fn decode_c2s(text: &str) -> Result<ClientMessage, WireError> {
    if text.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge(text.len()));
    }
    serde_json::from_str(text).map_err(|e| WireError::Json(e.to_string()))
}

/// Encode server message.
pub fn encode_s2c(msg: &ServerMessage) -> Result<String, WireError> {
    crate::presentation::validate(msg)?;
    let s = serde_json::to_string(msg).map_err(|e| WireError::Json(e.to_string()))?;
    if s.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge(s.len()));
    }
    Ok(s)
}

/// Decode server message (clients / tests).
pub fn decode_s2c(text: &str) -> Result<ServerMessage, WireError> {
    if text.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge(text.len()));
    }
    let message = serde_json::from_str(text).map_err(|e| WireError::Json(e.to_string()))?;
    crate::presentation::validate(&message)?;
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::ClientMessage;
    use engine_api::VersionBundle;

    #[test]
    fn roundtrip_hello() {
        let m = ClientMessage::Hello {
            versions: VersionBundle::current(),
            display_name: "tester".into(),
        };
        let s = encode_c2s(&m).unwrap();
        let d = decode_c2s(&s).unwrap();
        assert_eq!(m, d);
    }
}
