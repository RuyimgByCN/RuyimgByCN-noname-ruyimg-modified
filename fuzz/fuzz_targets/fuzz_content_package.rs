//! Fuzz target: ContentPackage::from_json_bytes (architecture §37.7 "内容包和依赖解析").
//!
//! Feeds arbitrary bytes as a JSON-serialized content package. Decoding must
//! never panic on malformed input or hostile field sizes.

#![no_main]

use content_schema::ContentPackage;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ContentPackage::from_json_bytes(data);
});
