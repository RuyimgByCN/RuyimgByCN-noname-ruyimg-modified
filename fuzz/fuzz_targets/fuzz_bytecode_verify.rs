//! Fuzz target: bytecode verifier (architecture §37.7).
//!
//! Decodes arbitrary bytes as a bytecode module, then runs the verifier on
//! whatever decodes. The verifier must reject invalid modules with a
//! structured error, never panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rule_vm::bytecode::BytecodeModule;
use rule_vm::verify_module;

fuzz_target!(|data: &[u8]| {
    if let Ok(module) = BytecodeModule::from_bytes(data) {
        let _ = verify_module(&module);
    }
});
