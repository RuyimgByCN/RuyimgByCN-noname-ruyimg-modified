//! Fuzz target: Command JSON decode (architecture §37.7 "Command / Protobuf 解码").
//!
//! Feeds arbitrary bytes as a JSON-serialized Command. serde_json must never
//! panic on malformed JSON — it must return a decode error.

#![no_main]

use engine_api::Command;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // serde_json handles arbitrary bytes safely (returns Err on invalid).
    let _: Result<Command, _> = serde_json::from_slice(data);
});
