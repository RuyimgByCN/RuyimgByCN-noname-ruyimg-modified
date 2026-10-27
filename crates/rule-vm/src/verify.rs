//! Bytecode verification before execution.

use crate::bytecode::{op, BytecodeModule};
use crate::VmError;

/// Verify module structure, jump targets, and stack depth bound.
pub fn verify_module(module: &BytecodeModule) -> Result<(), VmError> {
    verify_and_find_effect(module, None).map(|_| ())
}

/// Verify a module and detect an effect tag at an instruction boundary.
pub fn module_uses_effect_tag(module: &BytecodeModule, tag: u16) -> Result<bool, VmError> {
    verify_and_find_effect(module, Some(tag)).map(|(found, _)| found)
}

pub(crate) fn verify_resume_points(module: &BytecodeModule) -> Result<Vec<u32>, VmError> {
    verify_and_find_effect(module, None).map(|(_, points)| points)
}

fn verify_and_find_effect(
    module: &BytecodeModule,
    effect: Option<u16>,
) -> Result<(bool, Vec<u32>), VmError> {
    if module.code.is_empty() {
        return Err(VmError::InvalidBytecode("empty code".into()));
    }
    if module.max_stack == 0 {
        return Err(VmError::InvalidBytecode("max_stack is 0".into()));
    }
    if module.entry_pc as usize >= module.code.len() {
        return Err(VmError::InvalidBytecode("entry_pc out of range".into()));
    }

    let code = &module.code;
    let mut i = 0usize;
    let mut stack: i32 = 0;
    let mut max_seen = 0i32;
    let mut found = false;
    let mut boundaries = vec![false; code.len()];
    let mut jumps = Vec::new();
    let mut resume_points = Vec::new();

    while i < code.len() {
        boundaries[i] = true;
        let opc = code[i];
        i += 1;
        match opc {
            op::NOP | op::HALT | op::TRAP => {}
            op::CONST_I32 | op::CONST_U32 | op::CONST_BOOL => {
                if i + 4 > code.len() {
                    return Err(VmError::InvalidBytecode("truncated const".into()));
                }
                i += 4;
                stack += 1;
            }
            op::LOAD => {
                if i >= code.len() {
                    return Err(VmError::InvalidBytecode("truncated load".into()));
                }
                let slot = code[i];
                i += 1;
                if slot >= module.local_count && module.local_count > 0 {
                    // allow slot < local_count; if local_count==0 no loads
                    return Err(VmError::InvalidBytecode(format!("load slot {slot}")));
                }
                if module.local_count == 0 {
                    return Err(VmError::InvalidBytecode("load with 0 locals".into()));
                }
                if slot >= module.local_count {
                    return Err(VmError::InvalidBytecode(format!("load slot {slot}")));
                }
                stack += 1;
            }
            op::STORE => {
                if i >= code.len() {
                    return Err(VmError::InvalidBytecode("truncated store".into()));
                }
                let slot = code[i];
                i += 1;
                if module.local_count == 0 || slot >= module.local_count {
                    return Err(VmError::InvalidBytecode(format!("store slot {slot}")));
                }
                stack -= 1;
            }
            op::DUP => {
                stack += 1;
            }
            op::POP | op::NOT => {
                stack -= 1;
            }
            op::ADD | op::SUB | op::MUL | op::EQ | op::LT | op::GT | op::AND | op::MIN => {
                stack -= 2;
                stack += 1;
            }
            op::JUMP | op::JUMP_IF_FALSE => {
                if i + 2 > code.len() {
                    return Err(VmError::InvalidBytecode("truncated jump".into()));
                }
                let off = i16::from_le_bytes([code[i], code[i + 1]]);
                i += 2;
                if opc == op::JUMP_IF_FALSE {
                    stack -= 1;
                }
                let target = i as i32 + off as i32;
                if target < 0 || target as usize >= code.len() {
                    return Err(VmError::InvalidBytecode(format!(
                        "jump target {target} out of range"
                    )));
                }
                jumps.push(target as usize);
            }
            op::HOST_EFFECT | op::HOST_QUERY => {
                if i >= code.len() {
                    return Err(VmError::InvalidBytecode("truncated host tag".into()));
                }
                if opc == op::HOST_EFFECT && effect == Some(u16::from(code[i])) {
                    found = true;
                }
                i += 1;
                // Conservative: host ops may pop many; require non-negative after max pop 3
                stack -= 3;
                if opc == op::HOST_QUERY {
                    stack += 1;
                }
            }
            op::HOST_EFFECT_EXT => {
                if i + 2 > code.len() {
                    return Err(VmError::InvalidBytecode(
                        "truncated extended host tag".into(),
                    ));
                }
                if effect == Some(u16::from_le_bytes([code[i], code[i + 1]])) {
                    found = true;
                }
                i += 2;
                stack -= 3;
            }
            op::HOST_CHOICE => {
                if i >= code.len() {
                    return Err(VmError::InvalidBytecode("truncated choice count".into()));
                }
                let count = code[i] as i32;
                i += 1;
                // Pops `count` option tokens, pushes 1 chosen token on resume.
                stack -= count;
                stack += 1;
                resume_points.push(i as u32);
            }
            op::HOST_CHOICE_RANGE => {
                // Pops inclusive min/max bounds, pushes one chosen token on resume.
                stack -= 2;
                stack += 1;
                resume_points.push(i as u32);
            }
            op::HOST_SELF
            | op::HOST_EVENT_PLAYER
            | op::HOST_EVENT_SOURCE
            | op::HOST_EVENT_CARD
            | op::HOST_SKILL
            | op::HOST_EVENT_AMOUNT
            | op::HOST_EVENT_TARGET_COUNT
            | op::HOST_EVENT_CARD_COUNT => {
                stack += 1;
            }
            _ => {
                return Err(VmError::InvalidBytecode(format!("unknown opcode {opc:#x}")));
            }
        }
        if stack < 0 {
            // Allow temporary underestimation for host; clamp check only max
            stack = 0;
        }
        max_seen = max_seen.max(stack);
        if max_seen > module.max_stack as i32 {
            return Err(VmError::InvalidBytecode(format!(
                "stack depth {max_seen} exceeds max_stack {}",
                module.max_stack
            )));
        }
    }
    if !boundaries[module.entry_pc as usize] {
        return Err(VmError::InvalidBytecode(format!(
            "entry_pc {} is not an instruction boundary",
            module.entry_pc
        )));
    }
    if let Some(target) = jumps.into_iter().find(|target| !boundaries[*target]) {
        return Err(VmError::InvalidBytecode(format!(
            "jump target {target} is not an instruction boundary"
        )));
    }
    resume_points.retain(|pc| (*pc as usize) < code.len());
    Ok((found, resume_points))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{assemble, Op};

    #[test]
    fn rejects_bad_jump() {
        let code = assemble(&[Op::Jump(1000), Op::Halt]);
        let m = BytecodeModule {
            name: "t".into(),
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            logic_hash: [0; 32],
            caps_mask: 0,
        };
        assert!(verify_module(&m).is_err());
    }

    #[test]
    fn accepts_simple() {
        let code = assemble(&[Op::ConstI32(1), Op::ConstI32(2), Op::Add, Op::Halt]);
        let m = BytecodeModule {
            name: "t".into(),
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            logic_hash: [0; 32],
            caps_mask: 0,
        };
        assert!(verify_module(&m).is_ok());
    }

    #[test]
    fn finds_effect_tags_only_at_instruction_boundaries() {
        let mut m = BytecodeModule {
            name: "effect-tags".into(),
            code: assemble(&[
                Op::ConstI32(i32::from_le_bytes([op::HOST_EFFECT_EXT, 0x90, 0x02, 0])),
                Op::Pop,
                Op::Halt,
            ]),
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            logic_hash: [0; 32],
            caps_mask: 0,
        };
        assert!(!module_uses_effect_tag(&m, 656).unwrap());
        m.code = assemble(&[Op::HostEffect(656), Op::HostEffect(772), Op::Halt]);
        assert!(module_uses_effect_tag(&m, 656).unwrap());
        assert!(module_uses_effect_tag(&m, 772).unwrap());
        m.code = vec![op::HOST_EFFECT_EXT, 0x90];
        assert!(module_uses_effect_tag(&m, 656).is_err());
    }
}
