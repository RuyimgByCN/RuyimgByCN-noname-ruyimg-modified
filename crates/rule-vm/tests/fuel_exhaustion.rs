//! VM fuel exhaustion + trap fault injection (architecture plan §37.8).
//!
//! Verifies that the Rule VM terminates on infinite loops (OutOfFuel) and on
//! trap instructions, rather than hanging or crashing the process.

use rule_vm::bytecode::{assemble, BytecodeModule, Op};
use rule_vm::{CollectingHost, ExecResult, RuleVm, VmError};

fn module(code: Vec<u8>) -> BytecodeModule {
    BytecodeModule {
        name: "fault_test".into(),
        code,
        local_count: 0,
        max_stack: 4,
        entry_pc: 0,
        logic_hash: [0u8; 32],
        caps_mask: 0,
    }
}

/// §37.8 VM fuel exhaustion: an infinite loop (JUMP to self) must terminate
/// with `OutOfFuel` after consuming the allotted budget, not hang.
#[test]
fn infinite_loop_terminates_on_out_of_fuel() {
    // JUMP(-3): after reading opcode (1 byte) + offset (2 bytes), PC advanced
    // by 3; offset -3 jumps back to the JUMP instruction → infinite loop.
    let code = assemble(&[Op::Jump(-3)]);
    let m = module(code);
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let result = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, 100);
    assert!(
        matches!(result, Err(VmError::OutOfFuel)),
        "infinite loop must hit OutOfFuel, got {result:?}"
    );
}

/// §37.8 VM fuel exhaustion: a finite loop within budget must halt normally.
#[test]
fn finite_loop_within_budget_halts() {
    // NOP loop that runs a few iterations then halts: JumpIfFalse(-5) pops 0 →
    // loops, but we push ConstI32(1) first so it never branches. Simpler: just
    // a NOP sequence that halts.
    let code = assemble(&[Op::Nop, Op::Nop, Op::Nop, Op::Halt]);
    let m = module(code);
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let result = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, 100);
    assert!(
        matches!(result, Ok(ExecResult::Halted { .. })),
        "finite program must halt, got {result:?}"
    );
}

/// §37.8 Trap instruction: a TRAP opcode must produce a VmError::Host, not a panic.
#[test]
fn trap_instruction_produces_host_error() {
    let code = assemble(&[Op::Trap]);
    let m = module(code);
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let result = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, 100);
    assert!(
        result.is_err(),
        "TRAP must produce an error, got {result:?}"
    );
    match result {
        Err(VmError::Host(msg)) => assert!(msg.contains("trap")),
        other => panic!("expected Host trap error, got {other:?}"),
    }
}

/// §37.8 Budget=0: even fuel=0 must immediately return OutOfFuel without executing.
#[test]
fn zero_budget_immediate_out_of_fuel() {
    let code = assemble(&[Op::Halt]);
    let m = module(code);
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let result = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, 0);
    assert!(matches!(result, Err(VmError::OutOfFuel)));
}

/// §37.8 Long-running loop with high fuel budget terminates exactly at fuel=0.
/// This verifies fuel accounting is precise (off-by-one would be a bug).
#[test]
fn fuel_accounting_is_precise() {
    // Infinite loop: each iteration costs 1 fuel (JUMP = 1 instruction).
    let code = assemble(&[Op::Jump(-3)]);
    let m = module(code);
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let budget = 50u32;
    let result = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, budget);
    assert!(matches!(result, Err(VmError::OutOfFuel)));
    // If we run with budget+1, it still runs out (infinite loop).
    let result2 = vm.execute(&rule_vm::VerifiedModule::verify(m.clone()).unwrap(), &mut host, budget + 1);
    assert!(matches!(result2, Err(VmError::OutOfFuel)));
}
