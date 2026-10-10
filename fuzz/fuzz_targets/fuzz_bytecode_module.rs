//! Fuzz target: BytecodeModule::from_bytes (architecture §37.7).
//!
//! Feeds arbitrary bytes as a serialized VM bytecode module. Decoding must
//! never panic on truncated/corrupt data — it must return a structured error.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rule_vm::bytecode::BytecodeModule;

fuzz_target!(|data: &[u8]| {
    let _ = BytecodeModule::from_bytes(data);
});
