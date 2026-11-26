//! Fuzz target: native NGSA archive decode + canonical hash (architecture §37.7).
//!
//! Feeds arbitrary bytes as a serialized replay/archive frame and exercises
//! structured error rejection without panicking on malformed input.

#![no_main]

use engine_native::archive::decode_archive;
use engine_native::Engine;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(snapshot) = decode_archive(data) {
        if let Ok(engine) = Engine::restore(&snapshot) {
            let _ = engine.state_hash();
        }
    }
});
