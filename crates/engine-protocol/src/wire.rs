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
    fn choice_counts_are_u32_and_ordered() {
        let base = serde_json::json!({"choice_id":1,"observer":1,"options":[],"min_count":0,"max_count":1,"allow_cancel":true,"timeout_policy":"None","revision":0,"continuation_id":1,"kind":"SelectCards"});
        for count in [0,255,256,257,u32::MAX] {
            let mut raw=base.clone(); raw["max_count"]=count.into();
            let choice: engine_api::ChoiceRequest=serde_json::from_value(raw).unwrap();
            assert_eq!(choice.max_count,count);
        }
        for invalid in [serde_json::json!(-1),serde_json::json!(4294967296u64),serde_json::json!(1.5),serde_json::json!("256"),serde_json::Value::Null] {
            let mut raw=base.clone();raw["max_count"]=invalid;
            assert!(serde_json::from_value::<engine_api::ChoiceRequest>(raw).is_err());
        }
        let mut raw=base;raw["min_count"]=2.into();
        assert!(serde_json::from_value::<engine_api::ChoiceRequest>(raw).is_err());
        assert_eq!(VersionBundle::current().network_protocol,crate::NETWORK_PROTOCOL_VERSION);
    }

    #[test]
    fn existing_wire_size_guard_rejects_before_decoding() {
        let oversized=" ".repeat(MAX_MESSAGE_BYTES+1);
        assert!(matches!(decode_c2s(&oversized),Err(WireError::TooLarge(_))));
        assert!(matches!(decode_s2c(&oversized),Err(WireError::TooLarge(_))));
        let m=ClientMessage::Hello{versions:VersionBundle::current(),display_name:oversized};
        assert!(matches!(encode_c2s(&m),Err(WireError::TooLarge(_))));
    }

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
