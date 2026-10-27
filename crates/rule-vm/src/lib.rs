//! Stack-based, pausable, deterministic Rule VM.
//!
//! Architecture §10.3: verified bytecode only; host capability whitelist;
//! fuel / stack / heap limits; serializable continuation. Never mutates
//! `GameState` directly — only via typed host requests.

#![deny(missing_docs)]

pub mod bytecode;
mod host;
mod reference;
mod verify;
mod vm;

pub use bytecode::{assemble, encode_caps, op, BytecodeModule, Op, MODULE_MAGIC, MODULE_VERSION};
pub use host::{CollectingHost, HostCall, HostContext, VmHost};
pub use reference::ReferenceVm;
pub use verify::{module_uses_effect_tag, verify_module};
pub use vm::{ExecResult, Frame, RuleVm, Value, VerifiedModule, DEFAULT_FUEL, DEFAULT_MAX_STACK};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Rule VM ABI version (independent version axis).
///
/// ABI 307 adds same-color repeat judgments while retaining additive compatibility with older
/// packages.
pub use engine_api::RULE_VM_ABI_VERSION;

/// Opaque continuation that can be snapshotted and resumed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Continuation {
    /// Continuation identity within a match.
    pub id: u64,
    /// Serialized operand stack + locals (bincode-free: custom pack).
    pub frames: Vec<u8>,
    /// Program counter into module code.
    pub pc: u32,
    /// Remaining fuel for this segment.
    pub fuel_remaining: u32,
    /// Module logic hash this continuation belongs to.
    pub module_hash: [u8; 32],
}

/// VM execution / verification error (deterministic trap).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VmError {
    /// Instruction fuel exhausted.
    #[error("out of fuel")]
    OutOfFuel,
    /// Operand or call stack depth exceeded.
    #[error("stack overflow")]
    StackOverflow,
    /// Heap / collection limit.
    #[error("memory limit")]
    MemoryLimit,
    /// Bytecode failed verification.
    #[error("invalid bytecode: {0}")]
    InvalidBytecode(String),
    /// Host denied capability.
    #[error("capability denied: {0}")]
    CapabilityDenied(String),
    /// Invalid host request.
    #[error("host error: {0}")]
    Host(String),
    /// Division by zero or arithmetic trap.
    #[error("arithmetic trap: {0}")]
    Arithmetic(String),
    /// Invalid PC / jump target.
    #[error("bad control flow: {0}")]
    BadControlFlow(String),
    /// Type / stack under/overflow at runtime.
    #[error("stack error: {0}")]
    Stack(String),
}

impl From<rule_api::HostTrap> for VmError {
    fn from(t: rule_api::HostTrap) -> Self {
        match t {
            rule_api::HostTrap::CapabilityDenied { name } => VmError::CapabilityDenied(name),
            rule_api::HostTrap::OutOfFuel => VmError::OutOfFuel,
            rule_api::HostTrap::StackOverflow => VmError::StackOverflow,
            rule_api::HostTrap::MemoryLimit => VmError::MemoryLimit,
            rule_api::HostTrap::InvalidRequest { message } => VmError::Host(message),
        }
    }
}
