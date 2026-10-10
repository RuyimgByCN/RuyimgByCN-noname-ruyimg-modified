//! Versioned multiplayer protocol for noname-next (Phase 7).
//!
//! Wire format today: length-prefixed or WebSocket text JSON.
//! Field numbers / names are frozen for future Protobuf mapping.
//!
//! Architecture §12: versioned messages, idempotency, revision gaps → snapshot.

#![deny(missing_docs)]

mod messages;
mod wire;

pub use messages::*;
pub use wire::{decode_c2s, decode_s2c, encode_c2s, encode_s2c};

/// Network protocol version axis (bump on breaking wire changes).
pub const NETWORK_PROTOCOL_VERSION: u32 = 5;

/// Maximum JSON message size (bytes) accepted by server.
pub const MAX_MESSAGE_BYTES: usize = 256 * 1024;

/// Maximum command payload nesting / collection sizes (defensive).
pub const MAX_TARGETS: usize = 16;
