//! Fuzz target: Snapshot::from_bytes (architecture §37.7).
//!
//! Feeds arbitrary bytes as a serialized snapshot. Decoding must never panic
//! on malformed input.

#![no_main]

use engine_core::Snapshot;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Snapshot::from_bytes(data);
});
