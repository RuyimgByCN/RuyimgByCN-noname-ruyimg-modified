//! Fuzz target: native NGSA archive decode + Engine::restore (architecture §37.7).
//!
//! Feeds arbitrary bytes as a serialized NGSA archive. Decoding and restoration
//! must never panic on malformed input.

#![no_main]

use engine_native::archive::decode_archive;
use engine_native::Engine;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(snapshot) = decode_archive(data) {
        let _ = Engine::restore(&snapshot);
    }
});
