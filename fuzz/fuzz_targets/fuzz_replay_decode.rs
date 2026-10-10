//! Fuzz target: ReplayLog::from_bytes (architecture §37.7).
//!
//! Feeds arbitrary bytes as a serialized replay file. Decoding must never
//! panic on malformed input.

#![no_main]

use engine_core::ReplayLog;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ReplayLog::from_bytes(data);
});
