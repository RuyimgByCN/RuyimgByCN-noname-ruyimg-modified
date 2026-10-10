//! Runtime errors.

use thiserror::Error;

/// Content runtime error.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// I/O.
    #[error("io: {0}")]
    Io(String),
    /// Invalid argument / corrupt state.
    #[error("invalid: {0}")]
    Invalid(String),
    /// Package parse failure.
    #[error("package: {0}")]
    Package(String),
    /// Signature verification failed.
    #[error("signature: {0}")]
    Signature(String),
    /// Engine / ABI incompatible.
    #[error("incompatible: {0}")]
    Incompatible(String),
    /// Package is on the revoke list.
    #[error("revoked: {0}")]
    Revoked(String),
    /// Unsigned packages not allowed.
    #[error("unsigned package not allowed")]
    UnsignedNotAllowed,
    /// Active set / install not found.
    #[error("not found: {0}")]
    NotFound(String),
}

impl From<std::io::Error> for RuntimeError {
    fn from(e: std::io::Error) -> Self {
        RuntimeError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for RuntimeError {
    fn from(e: serde_json::Error) -> Self {
        RuntimeError::Invalid(e.to_string())
    }
}
