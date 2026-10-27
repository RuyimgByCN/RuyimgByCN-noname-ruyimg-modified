//! Deliberately simple reference interpreter for differential conformance.

use crate::bytecode::{caps, op, BytecodeModule};
use crate::host::VmHost;
use crate::verify::{verify_module, verify_resume_points};
use crate::vm::{decode_frame, encode_frame, ExecResult, Frame, RuleVm, Value};
use crate::{Continuation, VmError};

/// A second execution backend that decodes each instruction before applying it.
///
/// It intentionally does not reuse `RuleVm`'s dispatch loop. Both backends only
/// share the stable host-call ABI and continuation frame encoding.
#[derive(Debug, Default)]
pub struct ReferenceVm {
    next_continuation_id: u64,
}

impl ReferenceVm {
    /// Create a reference interpreter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify and execute a bytecode module from its entry point.
    pub fn execute(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        fuel: u32,
    ) -> Result<ExecResult, VmError> {
        verify_module(module)?;
        check_caps(module, host)?;
        let frame = Frame {
            locals: vec![Value(0); module.local_count as usize],
            stack: Vec::with_capacity(module.max_stack as usize),
            pc: module.entry_pc,
        };
        self.run(module, host, frame, fuel)
    }

    /// Resume a previously paused reference execution.
    pub fn resume(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        continuation: Continuation,
        chosen_token: u32,
    ) -> Result<ExecResult, VmError> {
        let resume_points = verify_resume_points(module)?;
        check_caps(module, host)?;
        if continuation.module_hash != module.logic_hash {
            return Err(VmError::InvalidBytecode(
                "continuation module mismatch".into(),
            ));
        }
        if !resume_points.contains(&continuation.pc) {
            return Err(VmError::InvalidBytecode(
                "continuation pc is not a choice resume point".into(),
            ));
        }
        let mut frame = decode_frame(&continuation.frames)?;
        frame.pc = continuation.pc;
        frame.stack.push(Value(chosen_token as i32));
        self.run(module, host, frame, continuation.fuel_remaining)
    }

    fn run(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        mut frame: Frame,
        mut fuel: u32,
    ) -> Result<ExecResult, VmError> {
        while fuel > 0 {
            fuel -= 1;
            let instruction = decode(&module.code, frame.pc)?;
            frame.pc = instruction.next_pc;

            match instruction.kind {
                InstructionKind::Nop => {}
                InstructionKind::Halt => {
                    return Ok(ExecResult::Halted {
                        fuel_remaining: fuel,
                    });
                }
                InstructionKind::Trap => {
                    return Err(VmError::Host("trap instruction".into()));
                }
                InstructionKind::Const(value) => {
                    push(&mut frame, Value(value), module.max_stack as usize)?;
                }
                InstructionKind::Load(slot) => {
                    let value = frame
                        .locals
                        .get(slot as usize)
                        .copied()
                        .ok_or_else(|| VmError::Stack("bad local".into()))?;
                    push(&mut frame, value, module.max_stack as usize)?;
                }
                InstructionKind::Store(slot) => {
                    let value = pop(&mut frame)?;
                    let local = frame
                        .locals
                        .get_mut(slot as usize)
                        .ok_or_else(|| VmError::Stack("bad local store".into()))?;
                    *local = value;
                }
                InstructionKind::Dup => {
                    let value = *frame
                        .stack
                        .last()
                        .ok_or_else(|| VmError::Stack("dup empty".into()))?;
                    push(&mut frame, value, module.max_stack as usize)?;
                }
                InstructionKind::Pop => {
                    pop(&mut frame)?;
                }
                InstructionKind::Add => binary(&mut frame, i32::wrapping_add)?,
                InstructionKind::Sub => binary(&mut frame, i32::wrapping_sub)?,
                InstructionKind::Mul => binary(&mut frame, i32::wrapping_mul)?,
                InstructionKind::Eq => binary(&mut frame, |a, b| i32::from(a == b))?,
                InstructionKind::Lt => binary(&mut frame, |a, b| i32::from(a < b))?,
                InstructionKind::Min => binary(&mut frame, i32::min)?,
                InstructionKind::Not => {
                    let value = pop(&mut frame)?;
                    push(
                        &mut frame,
                        Value(i32::from(value.0 == 0)),
                        module.max_stack as usize,
                    )?;
                }
                InstructionKind::Jump(offset) => {
                    frame.pc = jump_target(instruction.next_pc, offset, module.code.len())?;
                }
                InstructionKind::JumpIfFalse(offset) => {
                    if pop(&mut frame)?.0 == 0 {
                        frame.pc = jump_target(instruction.next_pc, offset, module.code.len())?;
                    }
                }
                InstructionKind::HostEffect(tag) => {
                    RuleVm::host_effect(host, &mut frame, tag)?;
                }
                InstructionKind::HostQuery(tag) => {
                    let value = RuleVm::host_query(host, &mut frame, tag)?;
                    push(&mut frame, Value(value), module.max_stack as usize)?;
                }
                InstructionKind::HostChoice(count) => {
                    if !host.capabilities().request_choice {
                        return Err(VmError::CapabilityDenied("request_choice".into()));
                    }
                    let mut options = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        options.push(pop(&mut frame)?.0 as u32);
                    }
                    options.reverse();
                    self.next_continuation_id = self.next_continuation_id.saturating_add(1);
                    return Ok(ExecResult::Paused {
                        continuation: Continuation {
                            id: self.next_continuation_id,
                            frames: encode_frame(&frame),
                            pc: frame.pc,
                            fuel_remaining: fuel,
                            module_hash: module.logic_hash,
                        },
                        options,
                    });
                }
                InstructionKind::HostChoiceRange => {
                    if !host.capabilities().request_choice {
                        return Err(VmError::CapabilityDenied("request_choice".into()));
                    }
                    let max = pop(&mut frame)?.0;
                    let min = pop(&mut frame)?.0;
                    if min < 0 || max < min {
                        return Err(VmError::Arithmetic("invalid choice range".into()));
                    }
                    let len = i64::from(max) - i64::from(min) + 1;
                    if len > 255 {
                        return Err(VmError::MemoryLimit);
                    }
                    let options = (min..=max).map(|value| value as u32).collect();
                    self.next_continuation_id = self.next_continuation_id.saturating_add(1);
                    return Ok(ExecResult::Paused {
                        continuation: Continuation {
                            id: self.next_continuation_id,
                            frames: encode_frame(&frame),
                            pc: frame.pc,
                            fuel_remaining: fuel,
                            module_hash: module.logic_hash,
                        },
                        options,
                    });
                }
                InstructionKind::HostSelf => push(
                    &mut frame,
                    Value(host.context().self_id as i32),
                    module.max_stack as usize,
                )?,
                InstructionKind::HostEventPlayer => push(
                    &mut frame,
                    Value(host.context().event_player as i32),
                    module.max_stack as usize,
                )?,
                InstructionKind::HostEventSource => push(
                    &mut frame,
                    Value(host.context().event_source as i32),
                    module.max_stack as usize,
                )?,
                InstructionKind::HostEventCard => push(
                    &mut frame,
                    Value(host.context().event_card as i32),
                    module.max_stack as usize,
                )?,
                InstructionKind::HostSkill => push(
                    &mut frame,
                    Value(host.context().skill_id as i32),
                    module.max_stack as usize,
                )?,
                InstructionKind::HostEventAmount => push(
                    &mut frame,
                    Value(host.context().event_amount as i32),
                    module.max_stack as usize,
                )?,
            }
        }
        Err(VmError::OutOfFuel)
    }
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    kind: InstructionKind,
    next_pc: u32,
}

#[derive(Debug, Clone, Copy)]
enum InstructionKind {
    Nop,
    Halt,
    Trap,
    Const(i32),
    Load(u8),
    Store(u8),
    Dup,
    Pop,
    Add,
    Sub,
    Mul,
    Eq,
    Lt,
    Not,
    Min,
    Jump(i16),
    JumpIfFalse(i16),
    HostEffect(u16),
    HostQuery(u8),
    HostChoice(u8),
    HostChoiceRange,
    HostSelf,
    HostEventPlayer,
    HostEventSource,
    HostEventCard,
    HostSkill,
    HostEventAmount,
}

fn decode(code: &[u8], pc: u32) -> Result<Instruction, VmError> {
    let mut cursor = pc as usize;
    let opcode = take_u8(code, &mut cursor)?;
    let kind = match opcode {
        op::NOP => InstructionKind::Nop,
        op::HALT => InstructionKind::Halt,
        op::TRAP => InstructionKind::Trap,
        op::CONST_I32 | op::CONST_U32 | op::CONST_BOOL => {
            InstructionKind::Const(take_i32(code, &mut cursor)?)
        }
        op::LOAD => InstructionKind::Load(take_u8(code, &mut cursor)?),
        op::STORE => InstructionKind::Store(take_u8(code, &mut cursor)?),
        op::DUP => InstructionKind::Dup,
        op::POP => InstructionKind::Pop,
        op::ADD => InstructionKind::Add,
        op::SUB => InstructionKind::Sub,
        op::MUL => InstructionKind::Mul,
        op::EQ => InstructionKind::Eq,
        op::LT => InstructionKind::Lt,
        op::NOT => InstructionKind::Not,
        op::MIN => InstructionKind::Min,
        op::JUMP => InstructionKind::Jump(take_i16(code, &mut cursor)?),
        op::JUMP_IF_FALSE => InstructionKind::JumpIfFalse(take_i16(code, &mut cursor)?),
        op::HOST_EFFECT => InstructionKind::HostEffect(u16::from(take_u8(code, &mut cursor)?)),
        op::HOST_EFFECT_EXT => InstructionKind::HostEffect(take_u16(code, &mut cursor)?),
        op::HOST_QUERY => InstructionKind::HostQuery(take_u8(code, &mut cursor)?),
        op::HOST_CHOICE => InstructionKind::HostChoice(take_u8(code, &mut cursor)?),
        op::HOST_CHOICE_RANGE => InstructionKind::HostChoiceRange,
        op::HOST_SELF => InstructionKind::HostSelf,
        op::HOST_EVENT_PLAYER => InstructionKind::HostEventPlayer,
        op::HOST_EVENT_SOURCE => InstructionKind::HostEventSource,
        op::HOST_EVENT_CARD => InstructionKind::HostEventCard,
        op::HOST_SKILL => InstructionKind::HostSkill,
        op::HOST_EVENT_AMOUNT => InstructionKind::HostEventAmount,
        other => return Err(VmError::InvalidBytecode(format!("opcode {other:#x}"))),
    };
    Ok(Instruction {
        kind,
        next_pc: cursor as u32,
    })
}

fn check_caps(module: &BytecodeModule, host: &dyn VmHost) -> Result<(), VmError> {
    let required = module.caps_mask;
    let granted = host.capabilities();
    if required & caps::EMIT_EFFECTS != 0 && !granted.emit_effects {
        return Err(VmError::CapabilityDenied("emit_effects".into()));
    }
    if required & caps::RUN_QUERIES != 0 && !granted.run_queries {
        return Err(VmError::CapabilityDenied("run_queries".into()));
    }
    if required & caps::REQUEST_CHOICE != 0 && !granted.request_choice {
        return Err(VmError::CapabilityDenied("request_choice".into()));
    }
    Ok(())
}

fn jump_target(next_pc: u32, offset: i16, code_len: usize) -> Result<u32, VmError> {
    let target = next_pc as i32 + offset as i32;
    if target < 0 || target as usize > code_len {
        return Err(VmError::BadControlFlow("jump".into()));
    }
    Ok(target as u32)
}

fn push(frame: &mut Frame, value: Value, max: usize) -> Result<(), VmError> {
    if frame.stack.len() >= max {
        return Err(VmError::StackOverflow);
    }
    frame.stack.push(value);
    Ok(())
}

fn pop(frame: &mut Frame) -> Result<Value, VmError> {
    frame
        .stack
        .pop()
        .ok_or_else(|| VmError::Stack("stack underflow".into()))
}

fn binary(frame: &mut Frame, operation: fn(i32, i32) -> i32) -> Result<(), VmError> {
    let right = pop(frame)?;
    let left = pop(frame)?;
    frame.stack.push(Value(operation(left.0, right.0)));
    Ok(())
}

fn take_u8(code: &[u8], cursor: &mut usize) -> Result<u8, VmError> {
    let value = code
        .get(*cursor)
        .copied()
        .ok_or_else(|| VmError::BadControlFlow("read_u8".into()))?;
    *cursor += 1;
    Ok(value)
}

fn take_u16(code: &[u8], cursor: &mut usize) -> Result<u16, VmError> {
    if *cursor + 2 > code.len() {
        return Err(VmError::InvalidBytecode("truncated u16".into()));
    }
    let value = u16::from_le_bytes([code[*cursor], code[*cursor + 1]]);
    *cursor += 2;
    Ok(value)
}

fn take_i16(code: &[u8], cursor: &mut usize) -> Result<i16, VmError> {
    let bytes = code
        .get(*cursor..*cursor + 2)
        .ok_or_else(|| VmError::BadControlFlow("read_i16".into()))?;
    *cursor += 2;
    Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
}

fn take_i32(code: &[u8], cursor: &mut usize) -> Result<i32, VmError> {
    let bytes = code
        .get(*cursor..*cursor + 4)
        .ok_or_else(|| VmError::BadControlFlow("read_i32".into()))?;
    *cursor += 4;
    Ok(i32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

#[cfg(test)]
mod tests {

    /// PA06：测试助手——优化后端只接受装载期已校验模块（参照实现保留自有校验）。
    fn vmod(m: &BytecodeModule) -> crate::vm::VerifiedModule {
        crate::vm::VerifiedModule::verify(m.clone()).unwrap()
    }
    use super::*;
    use crate::bytecode::{assemble, effect_tag, BytecodeModule, Op};
    use crate::{CollectingHost, HostContext};

    fn module(ops: &[Op]) -> BytecodeModule {
        let code = assemble(ops);
        BytecodeModule {
            name: "reference".into(),
            logic_hash: BytecodeModule::compute_logic_hash("reference", &code, 1, 0, 0),
            code,
            local_count: 1,
            max_stack: 8,
            entry_pc: 0,
            caps_mask: caps::EMIT_EFFECTS | caps::REQUEST_CHOICE,
        }
    }

    #[test]
    fn matches_rule_vm_across_choice_resume() {
        let module = module(&[
            Op::ConstU32(10),
            Op::ConstU32(20),
            Op::HostChoice(2),
            Op::Store(0),
            Op::HostSelf,
            Op::Load(0),
            Op::HostEffect(effect_tag::DRAW_CARDS),
            Op::Halt,
        ]);
        let context = HostContext {
            self_id: 7,
            ..Default::default()
        };
        let mut optimized_host = CollectingHost {
            ctx: context,
            ..Default::default()
        };
        let mut reference_host = optimized_host.clone();
        let mut optimized = RuleVm::new();
        let mut reference = ReferenceVm::new();

        let optimized_pause = optimized
            .execute(&vmod(&module), &mut optimized_host, 100)
            .unwrap();
        let reference_pause = reference
            .execute(&module, &mut reference_host, 100)
            .unwrap();
        assert_eq!(optimized_pause, reference_pause);

        let ExecResult::Paused { continuation, .. } = optimized_pause else {
            panic!("optimized backend did not pause");
        };
        let ExecResult::Paused {
            continuation: reference_continuation,
            ..
        } = reference_pause
        else {
            panic!("reference backend did not pause");
        };
        let optimized_result = optimized
            .resume(&vmod(&module), &mut optimized_host, continuation, 20)
            .unwrap();
        let reference_result = reference
            .resume(&module, &mut reference_host, reference_continuation, 20)
            .unwrap();
        assert_eq!(optimized_result, reference_result);
        assert_eq!(optimized_host.effects, reference_host.effects);
    }

    #[test]
    fn matches_rule_vm_across_range_choice_resume() {
        let module = module(&[
            Op::ConstI32(1),
            Op::ConstI32(4),
            Op::HostChoiceRange,
            Op::Store(0),
            Op::HostSelf,
            Op::Load(0),
            Op::HostEffect(effect_tag::DRAW_CARDS),
            Op::Halt,
        ]);
        let context = HostContext {
            self_id: 7,
            ..Default::default()
        };
        let mut optimized_host = CollectingHost {
            ctx: context,
            ..Default::default()
        };
        let mut reference_host = optimized_host.clone();
        let mut optimized = RuleVm::new();
        let mut reference = ReferenceVm::new();

        let optimized_pause = optimized
            .execute(&vmod(&module), &mut optimized_host, 100)
            .unwrap();
        let reference_pause = reference
            .execute(&module, &mut reference_host, 100)
            .unwrap();
        assert_eq!(optimized_pause, reference_pause);
        let ExecResult::Paused {
            continuation,
            options,
        } = optimized_pause
        else {
            panic!("optimized backend did not pause");
        };
        assert_eq!(options, [1, 2, 3, 4]);
        let ExecResult::Paused {
            continuation: reference_continuation,
            ..
        } = reference_pause
        else {
            panic!("reference backend did not pause");
        };
        let optimized_result = optimized
            .resume(&vmod(&module), &mut optimized_host, continuation, 3)
            .unwrap();
        let reference_result = reference
            .resume(&module, &mut reference_host, reference_continuation, 3)
            .unwrap();
        assert_eq!(optimized_result, reference_result);
        assert_eq!(optimized_host.effects, reference_host.effects);
    }
}
