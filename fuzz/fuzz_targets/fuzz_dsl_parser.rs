//! Fuzz target: DSL parser (architecture §37.7).
//!
//! Feeds arbitrary bytes as a DSL source string into the content compiler.
//! The parser must never panic, abort, or hang on malformed input — it must
//! return a structured `CompileError`.
//!
//! Run: `cargo +nightly fuzz run fuzz_dsl_parser`

#![no_main]

use content_compiler::parse_source;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The parser accepts &str; interpret bytes as UTF-8 lossily. Invalid UTF-8
    // is itself an interesting input (must be rejected cleanly).
    let source = String::from_utf8_lossy(data);
    let _ = parse_source(&source);
    // If parsing succeeds, the AST must be safe to drop.
});
