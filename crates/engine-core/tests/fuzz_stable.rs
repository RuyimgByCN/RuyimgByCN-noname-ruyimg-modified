//! Stable-testable pseudo-fuzz (architecture §37.7).
//!
//! libFuzzer harnesses live in `fuzz/` (nightly-only). This file provides a
//! CI-portable equivalent that runs under plain `cargo test` and asserts the
//! same contract: **no panic / abort on arbitrary bytes at every decode
//! boundary**. It uses `proptest` to generate random byte slices.
//!
//! These are NOT coverage-guided (they can't find deep bugs like libFuzzer),
//! but they guard against panics on malformed input in every CI run without
//! needing nightly Rust.

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// §37.7 DSL Parser: arbitrary bytes as DSL source must not panic.
    #[test]
    fn fuzz_stable_dsl_parser(data: Vec<u8>) {
        let source = String::from_utf8_lossy(&data);
        let _ = content_compiler::parse_source(&source);
    }

    /// §37.7 IR / bytecode decode: arbitrary bytes must not panic.
    #[test]
    fn fuzz_stable_bytecode_module(data: Vec<u8>) {
        let _ = rule_vm::bytecode::BytecodeModule::from_bytes(&data);
    }

    /// §37.7 bytecode verifier: decode then verify must not panic.
    #[test]
    fn fuzz_stable_bytecode_verify(data: Vec<u8>) {
        if let Ok(module) = rule_vm::bytecode::BytecodeModule::from_bytes(&data) {
            let _ = rule_vm::verify_module(&module);
        }
    }

    /// §37.7 Snapshot decode: arbitrary bytes must not panic.
    #[test]
    fn fuzz_stable_snapshot_decode(data: Vec<u8>) {
        let _ = engine_core::Snapshot::from_bytes(&data);
    }

    /// §37.7 Replay decode: arbitrary bytes must not panic.
    #[test]
    fn fuzz_stable_replay_decode(data: Vec<u8>) {
        let _ = engine_core::ReplayLog::from_bytes(&data);
    }

    /// §37.7 Command decode: arbitrary JSON bytes must not panic.
    #[test]
    fn fuzz_stable_command_json(data: Vec<u8>) {
        let _: Result<engine_api::Command, _> = serde_json::from_slice(&data);
    }

    /// §37.7 Content package decode: arbitrary JSON bytes must not panic.
    #[test]
    fn fuzz_stable_content_package(data: Vec<u8>) {
        let _ = content_schema::ContentPackage::from_json_bytes(&data);
    }
}
