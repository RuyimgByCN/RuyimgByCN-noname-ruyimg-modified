//! Stack machine interpreter.

use crate::bytecode::{effect_tag, op, query_tag, relation_code, BytecodeModule};
use crate::host::VmHost;
use crate::verify::verify_module;
use crate::{Continuation, VmError};
use rule_api::{DrawTargetScope, EffectRequest, PlayerId, QueryRequest, QueryResponse};
use serde::{Deserialize, Serialize};

/// Default fuel per invocation.
pub const DEFAULT_FUEL: u32 = 10_000;
/// Default max operand stack depth.
pub const DEFAULT_MAX_STACK: usize = 64;

/// Runtime value (integers only — no floats).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Value(pub i32);

/// One call frame (Phase 4: single frame; multi-frame later).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Locals.
    pub locals: Vec<Value>,
    /// Operand stack.
    pub stack: Vec<Value>,
    /// PC.
    pub pc: u32,
}

/// Result of running the VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecResult {
    /// Finished HALT.
    Halted {
        /// Remaining fuel.
        fuel_remaining: u32,
    },
    /// Paused for choice (HOST_CHOICE).
    Paused {
        /// Continuation to resume (frames/pc/fuel/module already captured).
        continuation: Continuation,
        /// Option tokens offered to the chooser (in emission order).
        options: Vec<u32>,
    },
}

/// Rule VM instance (stateless w.r.t. game; holds continuation allocator).
#[derive(Debug, Default)]
pub struct RuleVm {
    next_continuation_id: u64,
}

impl RuleVm {
    /// Create VM.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate continuation id shell.
    pub fn alloc_continuation(&mut self, fuel: u32) -> Continuation {
        self.next_continuation_id = self.next_continuation_id.saturating_add(1);
        Continuation {
            id: self.next_continuation_id,
            frames: Vec::new(),
            pc: 0,
            fuel_remaining: fuel,
            module_hash: [0; 32],
        }
    }

    /// Verify then run module from entry with host.
    pub fn execute(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        fuel: u32,
    ) -> Result<ExecResult, VmError> {
        verify_module(module)?;
        self.check_caps(module, host)?;
        let mut frame = Frame {
            locals: vec![Value(0); module.local_count as usize],
            stack: Vec::with_capacity(module.max_stack as usize),
            pc: module.entry_pc,
        };
        self.run_loop(module, host, &mut frame, fuel)
    }

    /// Resume from continuation.
    /// Resume a paused continuation. `chosen_token` is the option token the
    /// chooser selected at the `HOST_CHOICE` site; it is pushed onto the operand
    /// stack so the resumed body can consume it (e.g. store into a local).
    pub fn resume(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        cont: Continuation,
        chosen_token: u32,
    ) -> Result<ExecResult, VmError> {
        verify_module(module)?;
        if cont.module_hash != module.logic_hash {
            return Err(VmError::InvalidBytecode(
                "continuation module mismatch".into(),
            ));
        }
        let mut frame = decode_frame(&cont.frames)?;
        frame.pc = cont.pc;
        frame.stack.push(Value(chosen_token as i32));
        self.run_loop(module, host, &mut frame, cont.fuel_remaining)
    }

    fn check_caps(&self, module: &BytecodeModule, host: &dyn VmHost) -> Result<(), VmError> {
        use crate::bytecode::caps;
        let c = host.capabilities();
        let m = module.caps_mask;
        if m & caps::EMIT_EFFECTS != 0 && !c.emit_effects {
            return Err(VmError::CapabilityDenied("emit_effects".into()));
        }
        if m & caps::RUN_QUERIES != 0 && !c.run_queries {
            return Err(VmError::CapabilityDenied("run_queries".into()));
        }
        if m & caps::REQUEST_CHOICE != 0 && !c.request_choice {
            return Err(VmError::CapabilityDenied("request_choice".into()));
        }
        Ok(())
    }

    fn run_loop(
        &mut self,
        module: &BytecodeModule,
        host: &mut dyn VmHost,
        frame: &mut Frame,
        mut fuel: u32,
    ) -> Result<ExecResult, VmError> {
        let code = &module.code;
        let max_stack = module.max_stack as usize;
        loop {
            if fuel == 0 {
                return Err(VmError::OutOfFuel);
            }
            fuel -= 1;

            let pc = frame.pc as usize;
            if pc >= code.len() {
                return Err(VmError::BadControlFlow("pc past end".into()));
            }
            let opc = code[pc];
            frame.pc = (pc + 1) as u32;

            match opc {
                op::NOP => {}
                op::HALT => {
                    return Ok(ExecResult::Halted {
                        fuel_remaining: fuel,
                    });
                }
                op::TRAP => {
                    return Err(VmError::Host("trap instruction".into()));
                }
                op::CONST_I32 => {
                    let v = read_i32(code, &mut frame.pc)?;
                    push(frame, Value(v), max_stack)?;
                }
                op::CONST_U32 | op::CONST_BOOL => {
                    let v = read_u32(code, &mut frame.pc)? as i32;
                    push(frame, Value(v), max_stack)?;
                }
                op::LOAD => {
                    let slot = read_u8(code, &mut frame.pc)? as usize;
                    let v = frame
                        .locals
                        .get(slot)
                        .copied()
                        .ok_or_else(|| VmError::Stack("bad local".into()))?;
                    push(frame, v, max_stack)?;
                }
                op::STORE => {
                    let slot = read_u8(code, &mut frame.pc)? as usize;
                    let v = pop(frame)?;
                    if slot >= frame.locals.len() {
                        return Err(VmError::Stack("bad local store".into()));
                    }
                    frame.locals[slot] = v;
                }
                op::DUP => {
                    let v = *frame
                        .stack
                        .last()
                        .ok_or_else(|| VmError::Stack("dup empty".into()))?;
                    push(frame, v, max_stack)?;
                }
                op::POP => {
                    pop(frame)?;
                }
                op::ADD => binop(frame, |a, b| a.wrapping_add(b))?,
                op::SUB => binop(frame, |a, b| a.wrapping_sub(b))?,
                op::MUL => binop(frame, |a, b| a.wrapping_mul(b))?,
                op::EQ => binop(frame, |a, b| i32::from(a == b))?,
                op::LT => binop(frame, |a, b| i32::from(a < b))?,
                op::GT => binop(frame, |a, b| i32::from(a > b))?,
                op::AND => binop(frame, |a, b| i32::from(a != 0 && b != 0))?,
                op::MIN => binop(frame, |a, b| a.min(b))?,
                op::NOT => {
                    let v = pop(frame)?;
                    push(frame, Value(i32::from(v.0 == 0)), max_stack)?;
                }
                op::JUMP => {
                    let off = read_i16(code, &mut frame.pc)?;
                    let new_pc = frame.pc as i32 + off as i32;
                    if new_pc < 0 || new_pc as usize > code.len() {
                        return Err(VmError::BadControlFlow("jump".into()));
                    }
                    frame.pc = new_pc as u32;
                }
                op::JUMP_IF_FALSE => {
                    let off = read_i16(code, &mut frame.pc)?;
                    let cond = pop(frame)?;
                    if cond.0 == 0 {
                        let new_pc = frame.pc as i32 + off as i32;
                        if new_pc < 0 || new_pc as usize > code.len() {
                            return Err(VmError::BadControlFlow("jump_if_false".into()));
                        }
                        frame.pc = new_pc as u32;
                    }
                }
                op::HOST_EFFECT => {
                    let tag = u16::from(read_u8(code, &mut frame.pc)?);
                    Self::host_effect(host, frame, tag)?;
                }
                op::HOST_EFFECT_EXT => {
                    let tag = read_u16(code, &mut frame.pc)?;
                    Self::host_effect(host, frame, tag)?;
                }
                op::HOST_QUERY => {
                    let tag = read_u8(code, &mut frame.pc)?;
                    let v = Self::host_query(host, frame, tag)?;
                    push(frame, Value(v), max_stack)?;
                }
                op::HOST_CHOICE => {
                    if !host.capabilities().request_choice {
                        return Err(VmError::CapabilityDenied("request_choice".into()));
                    }
                    let count = read_u8(code, &mut frame.pc)? as usize;
                    // Pop option tokens (emitted in order, so reverse on pop).
                    let mut options = Vec::with_capacity(count);
                    for _ in 0..count {
                        options.push(pop(frame)?.0 as u32);
                    }
                    options.reverse();
                    // Continuation resumes just after the choice; the caller
                    // pushes the chosen token onto the stack before resuming.
                    let cont = Continuation {
                        id: {
                            self.next_continuation_id = self.next_continuation_id.saturating_add(1);
                            self.next_continuation_id
                        },
                        frames: encode_frame(frame),
                        pc: frame.pc,
                        fuel_remaining: fuel,
                        module_hash: module.logic_hash,
                    };
                    return Ok(ExecResult::Paused {
                        continuation: cont,
                        options,
                    });
                }
                op::HOST_CHOICE_RANGE => {
                    if !host.capabilities().request_choice {
                        return Err(VmError::CapabilityDenied("request_choice".into()));
                    }
                    let max = pop(frame)?.0;
                    let min = pop(frame)?.0;
                    let options = inclusive_choice_range(min, max)?;
                    let cont = Continuation {
                        id: {
                            self.next_continuation_id = self.next_continuation_id.saturating_add(1);
                            self.next_continuation_id
                        },
                        frames: encode_frame(frame),
                        pc: frame.pc,
                        fuel_remaining: fuel,
                        module_hash: module.logic_hash,
                    };
                    return Ok(ExecResult::Paused {
                        continuation: cont,
                        options,
                    });
                }
                op::HOST_SELF => {
                    push(frame, Value(host.context().self_id as i32), max_stack)?;
                }
                op::HOST_EVENT_PLAYER => {
                    push(frame, Value(host.context().event_player as i32), max_stack)?;
                }
                op::HOST_EVENT_SOURCE => {
                    push(frame, Value(host.context().event_source as i32), max_stack)?;
                }
                op::HOST_EVENT_CARD => {
                    push(frame, Value(host.context().event_card as i32), max_stack)?;
                }
                op::HOST_SKILL => {
                    push(frame, Value(host.context().skill_id as i32), max_stack)?;
                }
                op::HOST_EVENT_AMOUNT => {
                    push(frame, Value(host.context().event_amount as i32), max_stack)?;
                }
                op::HOST_EVENT_TARGET_COUNT => {
                    push(frame, Value(host.event_targets().len() as i32), max_stack)?;
                }
                op::HOST_EVENT_CARD_COUNT => {
                    push(frame, Value(host.event_cards().len() as i32), max_stack)?;
                }
                _ => {
                    return Err(VmError::InvalidBytecode(format!("opcode {opc:#x}")));
                }
            }
        }
    }

    pub(crate) fn host_effect(
        host: &mut dyn VmHost,
        frame: &mut Frame,
        tag: u16,
    ) -> Result<(), VmError> {
        if !host.capabilities().emit_effects {
            return Err(VmError::CapabilityDenied("emit_effects".into()));
        }
        let req = match tag {
            effect_tag::DRAW_CARDS => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawCards {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::MODIFY_CURRENT_DAMAGE => {
                let delta = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ModifyCurrentDamage {
                    player: PlayerId::new(player),
                    delta,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DEAL_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    amount,
                    is_lose_hp: false,
                    nature: 0,
                }
            }
            effect_tag::DEAL_NATURE_DAMAGE => {
                let nature = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    amount,
                    is_lose_hp: false,
                    nature,
                }
            }
            effect_tag::RECOVER_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverHp {
                    source: PlayerId::new(host.context().self_id),
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::RECOVER_HP_AS => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverHp {
                    source: PlayerId::new(source),
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::CHOOSE_DISCARD_REDIRECT_CURRENT_SHA => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardRedirectCurrentSha {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::JUDGE_SET_SKILL_MARK_TO_RANK => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSetSkillMarkToRank {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_FROM_EACH_LINKED => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneFromEachLinked {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::DISABLE_SKILL_BY_DEF_UNTIL_TURN_END => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DisableSkillByDefUntilTurnEnd {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                    stage: 0,
                    discarded_basic: false,
                }
            }
            effect_tag::EMIT_NOTE => {
                let code = pop(frame)?.0.max(0) as u32;
                EffectRequest::EmitRuleEvent {
                    code,
                    payload: vec![],
                }
            }
            effect_tag::CHANGE_MAX_HP => {
                let delta = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChangeMaxHp {
                    player: PlayerId::new(player),
                    delta,
                }
            }
            effect_tag::ADD_MARK => {
                let delta = pop(frame)?.0;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddMark {
                    player: PlayerId::new(player),
                    mark: rule_api::MarkDefId::new(mark),
                    delta,
                }
            }
            effect_tag::REMOVE_MARK => {
                let amount = pop(frame)?.0.max(0) as u32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveMark {
                    player: PlayerId::new(player),
                    mark: rule_api::MarkDefId::new(mark),
                    amount,
                }
            }
            effect_tag::DISCARD_HAND => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardFromHand {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::DISCARD_HAND_EQUIP => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardFromHandEquip {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::TAKE_HAND => {
                let to = pop(frame)?.0.max(0) as u32;
                let from = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeFromHand {
                    from: PlayerId::new(from),
                    to: PlayerId::new(to),
                }
            }
            effect_tag::LOSE_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: PlayerId::new(player),
                    amount,
                    is_lose_hp: true,
                    nature: 0,
                }
            }
            effect_tag::ADD_SKILL => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddSkill {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::ENTER_DYING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::EnterDying {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::REVEAL_CARD => {
                let card = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealCards {
                    cards: vec![rule_api::CardId::new(card)],
                    to: vec![],
                }
            }
            effect_tag::USE_CARD => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    targets: vec![],
                }
            }
            effect_tag::REMOVE_SKILL => {
                let skill = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveSkill {
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::MOVE_CARD => {
                let from = pop(frame)?.0.max(0) as u32;
                let to = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                EffectRequest::MoveCards {
                    cards: vec![rule_api::CardId::new(card)],
                    to: rule_api::ZoneId::new(to),
                    from: (from != 0).then(|| rule_api::ZoneId::new(from)),
                }
            }
            effect_tag::SET_TURN_STATE => {
                let phase = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let current = pop(frame)?.0.max(0) as u32;
                EffectRequest::SetTurnState {
                    current: PlayerId::new(current),
                    phase,
                }
            }
            effect_tag::USE_CARD_ON => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    targets: vec![PlayerId::new(target)],
                }
            }
            effect_tag::DISCARD_SELECTED => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardCards {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(rule_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::RECAST_SELECTED => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecastCards {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(rule_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::DISABLE_ALL_EQUIP_SLOTS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DisableEquipSlots {
                    player: PlayerId::new(player),
                    slots: vec![1, 2, 3, 4, 5],
                }
            }
            effect_tag::MOVE_EQUIPMENT_THEN_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::MoveEquipmentThenDraw {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_USE_VIRTUAL_SHA_OR_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseUseVirtualShaOrDraw {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_EQUIPMENT_PAIR_WITH_ARMOR => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardEquipmentPairWithArmor {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardTargetEquipmentGainMount {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::DISCARD_TARGET_EQUIPMENT_DRAW_WEAPON_RANGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_REVEAL_HAND_DAMAGE_IF_SHA => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetRevealHandDamageIfSha {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_GIVE_ONE_HAND_EQUIP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherGiveOneHandEquip {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_REVEAL_HAND_CARD_BIND_CATEGORY_TARGET => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECOVER_TO_MAX_THEN_OPTIONAL_EXCHANGE_SKILL => {
                let removable_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                    removable_skill_def: engine_api::SkillDefId::new(removable_skill_def),
                    restore_skill_def: engine_api::SkillDefId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::REVEAL_HAND_DISCARD_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandDiscardKind {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_AND_OR_DRAW => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_ATTACK_RANGE_CARDS_OR_INCREASE_DAMAGE_WEAPON_FORCES_BOTH => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DRAW_OR_RECAST_ADD_SKILL => {
                let recast_skill_def = pop(frame)?.0 as u32;
                let draw_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrRecastAddSkill {
                    player: PlayerId::new(player),
                    draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                    recast_skill_def: engine_api::SkillDefId::new(recast_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_SHA_FOR_OPENING_EXTRA_PLAY => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainShaForOpeningExtraPlay {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::FINISH_OPENING_EXTRA_PLAY_SHA_GAIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::FinishOpeningExtraPlayShaGain {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_TARGET_HAND_GAIN_SHAN_ARMOR_OR_DRAW_TWO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_ALL_HAND => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveAllHand {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::GAIN_CARD_IF_DISCARD => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainCardIfDiscard {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                }
            }
            effect_tag::CHOOSE_DRAW_OR_DISCARD_TARGET_CARD_RECORD_SHA => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_HAND_TO_PLAYER => {
                let observer = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandToPlayer {
                    player: PlayerId::new(player),
                    observer: PlayerId::new(observer),
                }
            }
            effect_tag::CHOOSE_OPTIONAL_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseOptionalDiscardTargetCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::REVEAL_HAND_CHOOSE_DISCARD_COLOR => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandChooseDiscardColor {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY => {
                let amount = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_SCSKUIJI_DISCARD => {
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseScskuijiDiscard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_PSKKUIJI_DISCARD => {
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChoosePskuijiDiscard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE => {
                let damage = pop(frame)?.0.max(0) as u32;
                let lose_hp = pop(frame)?.0.max(0) as u32;
                let failure_count = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    draw_count,
                    failure_count,
                    lose_hp,
                    damage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_TARGET_CARD_GAIN_MOUNT => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::DiscardTargetCardGainMount {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    retieji: false,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    judgment_suit: 0,
                }
            }
            effect_tag::RESOLVE_RETIEJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    retieji: true,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    judgment_suit: 0,
                }
            }
            effect_tag::RESOLVE_NEW_RELUOYI => {
                let buff_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNewReluoyi {
                    player: PlayerId::new(player),
                    buff_skill_def: engine_api::SkillDefId::new(buff_skill_def),
                    stage: 0,
                    cards: Vec::new(),
                    accepted: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REMOSHI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRemoshi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINYAOMING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinyaoming {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    action: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REHUAIYI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRehuaiyi {
                    player: PlayerId::new(player),
                    stage: 0,
                    color: 0,
                    discarded_count: 0,
                    targets: Vec::new(),
                    index: 0,
                    gained_count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REANGUO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveReanguo {
                    player: PlayerId::new(player),
                    target: host
                        .selected_targets()
                        .first()
                        .copied()
                        .map(PlayerId::new)
                        .unwrap_or(PlayerId::NONE),
                    stage: 0,
                    index: 0,
                    remaining: 0b111,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_DISTANCE_HAND_OR_INCREASE_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DRAW_VIEW_TWO_GAIN_OR_MOVE_EQUIPMENT => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BEIZHU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBeizhu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBSHANJIA => {
                let distance_skill_def = pop(frame)?.0 as u32;
                let lost_equipment_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbshanjia {
                    player: PlayerId::new(player),
                    lost_equipment_count,
                    distance_skill_def: engine_api::SkillDefId::new(distance_skill_def),
                    stage: 0,
                    reward_mask: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DAOJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDaoji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::NONE,
                    stage: 0,
                    tw: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TW_DAOJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDaoji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::NONE,
                    stage: 0,
                    tw: true,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIAOHUA => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJiaohua {
                    player: PlayerId::new(player),
                    category: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LUANQUN => {
                let direct_hit_skill_def = pop(frame)?.0 as u32;
                let effect_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLuanqun {
                    player: PlayerId::new(player),
                    effect_skill_def: engine_api::SkillDefId::new(effect_skill_def),
                    direct_hit_skill_def: engine_api::SkillDefId::new(direct_hit_skill_def),
                    stage: 0,
                    participants: Vec::new(),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let owner = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
                    owner: PlayerId::new(owner),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                    ready_to_redirect: false,
                }
            }
            effect_tag::CHOOSE_TARGET_GAIN_BOTTOM_CARD => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetGainBottomCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::JUDGE_BLACK_GAIN_REPEAT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeBlackGainRepeat {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSameColorGainRepeat {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    color: 0,
                }
            }
            effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCurrentTrickTargetsToNullify {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST => {
                EffectRequest::UseVirtualJuedouSecondOnFirst {
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    card: rule_api::CardId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseDiscardTargetCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::JUDGE_PREVENT_DAMAGE_LOSE_MAX_HP_ON_NONHEART => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                }
            }
            effect_tag::CHOOSE_GAIN_TOP_CARDS_BY_SUIT => {
                let suit = match pop(frame)?.0 {
                    1 => engine_api::Suit::Spade,
                    2 => engine_api::Suit::Heart,
                    3 => engine_api::Suit::Club,
                    4 => engine_api::Suit::Diamond,
                    _ => return Err(VmError::Host("invalid top-card selection suit".into())),
                };
                let count = u8::try_from(pop(frame)?.0)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| VmError::Host("invalid top-card view count".into()))?;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainTopCardsBySuit {
                    player: PlayerId::new(player),
                    count,
                    suit,
                }
            }
            effect_tag::CHOOSE_OTHER_DRAW_BY_HAND_COUNT => {
                let times = pop(frame)?.0.max(0) as u32;
                let empty_count = pop(frame)?.0.max(0) as u32;
                let normal_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDrawByHandCount {
                    player: PlayerId::new(player),
                    normal_count,
                    empty_count,
                    times,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_THEN_GAIN_TARGET_EQUIPMENT => {
                let slot = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    slot,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDraw {
                    player: PlayerId::new(player),
                    count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawByPlayPhaseCardCategoryCount {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED => {
                let card = pop(frame)?.0.max(0) as u32;
                let sha_user = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
                    player: PlayerId::new(player),
                    sha_user: PlayerId::new(sha_user),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::QUEUE_OPENING_EXTRA_PLAY_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QueueOpeningExtraPlayPhase {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddCurrentShaTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET_WITHIN_DISTANCE => {
                let max_distance = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    max_distance,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REFUND_CURRENT_SHA_USE => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RefundCurrentShaUse {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DAMAGE_THEN_DRAW_SELECTED_TARGETS => {
                let draw = pop(frame)?.0.max(0) as u32;
                let damage = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DamageThenDrawSelectedTargets {
                    source: PlayerId::new(source),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    damage,
                    draw,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YOUDI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYoudi {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_KUANGFU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveKuangfu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHANJIA => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShanjia {
                    player: PlayerId::new(player),
                    count,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NEW_YAJIAO => {
                let source_card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNewYajiao {
                    player: PlayerId::new(player),
                    source_card: engine_api::CardId::new(source_card),
                    revealed: engine_api::CardId::NONE,
                    target: PlayerId::NONE,
                    source_category: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DINGPIN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDingpin {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseStoreHandShaInSkillPile {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU => {
                let dying = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseSpendSkillPileForDyingJiu {
                    player: PlayerId::new(player),
                    dying: PlayerId::new(dying),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ChooseCompareSelectedHandCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RANDOMIZE_CURRENT_HAND_COMPARE_OPPONENT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RandomizeCurrentHandCompareOpponent {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_HIGHEST_RANK_SHA_FROM_EVENT_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainHighestRankShaFromEventCards {
                    player: PlayerId::new(player),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_EVENT_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainEventCards {
                    player: PlayerId::new(player),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_TO_MAX_HP_CAP => {
                let cap = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawToMaxHpCap {
                    player: PlayerId::new(player),
                    cap,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DRAW_OWNER_LOST_HP_TURN_OVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_HAND_COMPARE_RESULT_CARD => {
                let player = pop(frame)?.0.max(0) as u32;
                let context = host.context();
                EffectRequest::GainHandCompareResultCard {
                    player: PlayerId::new(player),
                    initiator: PlayerId::new(context.event_player),
                    target: PlayerId::new(context.event_source),
                    winner: PlayerId::new(context.event_amount.saturating_sub(1)),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_OR_REMOVE_CURRENT_CARD_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompare {
                    player: PlayerId::new(player),
                    win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE_GRANT_TARGET_ON_WIN => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_grant_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompareGrantTargetOnWin {
                    player: PlayerId::new(player),
                    win_grant_def: engine_api::SkillDefId::new(win_grant_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_SELECTED_THEN_RECOVER_SELF => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveSelectedThenRecoverSelf {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE_BETWEEN => {
                let first = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompareBetween {
                    player: PlayerId::new(player),
                    first: PlayerId::new(first),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GUANXING_TOP => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GuanxingTop {
                    player: PlayerId::new(player),
                    count,
                    round: 0,
                    flipped: Vec::new(),
                    top: Vec::new(),
                    bottom: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_GIVE_THEN_RECOVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherGiveThenRecover {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GUANXING_TOP_BY_FALU_MARKS => {
                let diamond = pop(frame)?.0 as u32;
                let club = pop(frame)?.0 as u32;
                let heart = pop(frame)?.0 as u32;
                let spade = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GuanxingTopByFaluMarks {
                    player: PlayerId::new(player),
                    marks: [
                        engine_api::SkillDefId::new(spade),
                        engine_api::SkillDefId::new(heart),
                        engine_api::SkillDefId::new(club),
                        engine_api::SkillDefId::new(diamond),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDamage {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseOtherDiscardTargetCard {
                    player: PlayerId::new(player),
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_DISCARD_CARD_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainDiscardCardByKind {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            effect_tag::REVEAL_UNIFORM_HAND_CHOOSE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealUniformHandChooseDamage {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_DISCARD_CARD_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainDiscardCardByKind {
                    player: PlayerId::new(player),
                    kind,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_FOR_LIFETIME_CARD_ACTION_DIVISORS => {
                let third = pop(frame)?.0.max(0) as u32;
                let second = pop(frame)?.0.max(0) as u32;
                let first = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawForLifetimeCardActionDivisors {
                    player: PlayerId::new(player),
                    divisors: [first, second, third],
                }
            }
            effect_tag::CHOOSE_ADD_MISSING_SKILL => {
                let third = pop(frame)?.0 as u32;
                let second = pop(frame)?.0 as u32;
                let first = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddMissingSkill {
                    player: PlayerId::new(player),
                    skill_defs: [
                        engine_api::SkillDefId::new(first),
                        engine_api::SkillDefId::new(second),
                        engine_api::SkillDefId::new(third),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::ADD_FIRST_MISSING_SKILL => {
                let third = pop(frame)?.0 as u32;
                let second = pop(frame)?.0 as u32;
                let first = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddFirstMissingSkill {
                    player: PlayerId::new(player),
                    skill_defs: [
                        engine_api::SkillDefId::new(first),
                        engine_api::SkillDefId::new(second),
                        engine_api::SkillDefId::new(third),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCCUNSI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let yongjue_def = pop(frame)?.0 as u32;
                let guixiu_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDccunsi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    guixiu_def: engine_api::SkillDefId::new(guixiu_def),
                    yongjue_def: engine_api::SkillDefId::new(yongjue_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECORD_CURRENT_SHA_ALL_TARGETS => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecordCurrentShaAllTargets {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GIVE_OR_DISCARD_HAND => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid give recipient scope".into()))?;
                let payment_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGiveOrDiscardHand {
                    player: PlayerId::new(player),
                    payment_count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER_TARGET => {
                let amount = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let chooser = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                    chooser: PlayerId::new(chooser),
                    target: PlayerId::new(target),
                    count,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCJUE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcjue {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    remaining: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RUANJI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRuanji {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHENGBI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhengbi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FENGYING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveFengying {
                    player: PlayerId::new(player),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIAOJIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQiaojian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    action,
                    cards: Vec::new(),
                    split: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XICHA => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXicha {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HUZUN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHuzun {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHUGEMENGXUE => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhugemengxue {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BAILINGYUN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBailingyun {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCSHAOWEI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcshaowei {
                    player: PlayerId::new(player),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCDICHOU => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcdichou {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action,
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCDUFENG => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcdufeng {
                    player: PlayerId::new(player),
                    action,
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_RANGE => {
                let max = pop(frame)?.0.max(0) as u32;
                let min = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandRange {
                    player: PlayerId::new(player),
                    min,
                    max,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCCTJIUXIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcctjiuxian {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BIANYUE => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBianyue {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHUPEILAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhupeilan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DC_SB_ZHANGXIU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcSbZhangxiu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DC_SB_ZHOUYU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcSbZhouyu {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    event_targets: Vec::new(),
                    damaged_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DC_SB_LUSU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcSbLusu {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    amount: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHANGJIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let grant_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhangjian {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    grant_skill_def: engine_api::SkillDefId::new(grant_skill_def),
                    action,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHUGERUOXUE => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhugeruoxue {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_CAOYI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveCaoyi {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    event_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MALINGLI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMalingli {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    nature: 0,
                    damage_id: 0,
                    event_cards: Vec::new(),
                    event_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NSSHUAIYAN => {
                let recovered_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNsshuaiyan {
                    player: PlayerId::new(player),
                    recovered_player: PlayerId::new(recovered_player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_HANLONG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbHanlong {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_LIUFU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let zhucheng_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbLiufu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    zhucheng_skill_def: engine_api::SkillDefId::new(zhucheng_skill_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_LIUYAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbLiuyan {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_LIUZAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbLiuzan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_TIANYU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbTianyu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_XIZHENXIHONG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbXizhenxihong {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_YANGYI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbYangyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WDB_ZAOZHIRENJUN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWdbZaozhiRenjun {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XIANXI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXianxi {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    action,
                    event_targets: Vec::new(),
                    damaged_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XIAHENG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXiaheng {
                    player: PlayerId::new(player),
                    discard_target: PlayerId::NONE,
                    damage_target: PlayerId::NONE,
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_LIUCHONG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddLiuchong {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_DUANBING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddDuanbing {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_WUZHI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddWuzhi {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XUJING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXujing {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_CAOMAO => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddCaomao {
                    player: PlayerId::new(player),
                    action,
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XINXIANYING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXinxianying {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XIANGLANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXianglang {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_YUJIN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddYujin {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    target: PlayerId::NONE,
                    move_mask: 0,
                    action,
                    candidates: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_LIUYE => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddLiuye {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: rule_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_BAOSANNIANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddBaosanniang {
                    player: PlayerId::new(player),
                    target: host
                        .selected_targets()
                        .first()
                        .copied()
                        .map(PlayerId::new)
                        .unwrap_or(PlayerId::NONE),
                    card: host
                        .selected_cards()
                        .first()
                        .copied()
                        .map(rule_api::CardId::new)
                        .unwrap_or(rule_api::CardId::NONE),
                    action,
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(rule_api::CardId::new)
                        .collect(),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    used_kind: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_ZHENJI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddZhenji {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    action,
                    stage: 0,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_ZHAOANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddZhaoang {
                    player: PlayerId::new(player),
                    action,
                    stage: 0,
                    card: rule_api::CardId::NONE,
                    kind: 0,
                    count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_LIUBA => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddLiuba {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_JIANSHUO => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddJianshuo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_GUANNING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddGuanning {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_CAOSHUANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddCaoshuang {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XUELINGYUN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXuelingyun {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    cards: Vec::new(),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_LIUHONG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddLiuhong {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_XIAHOUXUAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddXiahouxuan {
                    player: PlayerId::new(player),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_ZHANGKAI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddZhangkai {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    cards: Vec::new(),
                    amount: 0,
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DDD_LIANGXI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDddLiangxi {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    cards: Vec::new(),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GUOZHAN_USE_ZHULIAN_BIHE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GuozhanUseZhulianBihe {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_SHOULIJIAN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShoulijian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CANCEL_INCOMING_CARD_USE => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::CancelIncomingCardUse {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS => {
                let second_def = pop(frame)?.0 as u32;
                let first_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherGainOneOfTwoSkills {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    first_def: engine_api::SkillDefId::new(first_def),
                    second_def: engine_api::SkillDefId::new(second_def),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::GWYEWU_RANDOM_DISCARD_CHAIN => {
                let lang_def = pop(frame)?.0 as u32;
                let wuyao_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGwyewuChain {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                    cost_cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    wuyao_def: engine_api::SkillDefId::new(wuyao_def),
                    lang_def: engine_api::SkillDefId::new(lang_def),
                }
            }
            effect_tag::RESOLVE_XIONGLUAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXiongluan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWJUXIANG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwjuxiang {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SBBIYUE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSbbiyue {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SXRMQIEWANG => {
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSxrmqiewang {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SXRMQIANLIU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSxrmqianliu {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GZBUQU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let delta = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGzbuqu {
                    player: PlayerId::new(player),
                    delta,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GZBUQU_RECOVER => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGzbuquRecover {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SET_DRAW_PHASE_FROM_BOTTOM => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SetDrawPhaseFromBottom {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DRAW_CARDS_FROM_BOTTOM => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawCardsFromBottom {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::RESOLVE_CHENGLVE => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveChenglve {
                    player: PlayerId::new(player),
                    stage,
                    suits: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHICAI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShicai {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    placed: Vec::new(),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGETS_DRAW => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
                let target_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetsDraw {
                    player: PlayerId::new(player),
                    count,
                    target_count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGETS_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid damage target scope".into()))?;
                let max_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let min_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetsDamage {
                    player: PlayerId::new(player),
                    min_count,
                    max_count,
                    scope,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OPTIONAL_TARGETS_DRAW => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
                let max_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let min_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOptionalTargetsDraw {
                    player: PlayerId::new(player),
                    count,
                    min_count,
                    max_count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_TARGET_HAND_SHA_OR_DISCARD_BIND => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def: rule_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                    stage: 0,
                }
            }
            effect_tag::DRAW_REMEMBERED_HAND_CARDS => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawRememberedHandCards {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_REMEMBERED_HAND_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardRememberedHandCards {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_DRAW_CARDS_FOR_LIFETIME_CARD_ACTION_DIVISORS => {
                let third_group = pop(frame)?.0.max(0) as u8;
                let third_divisor = pop(frame)?.0.max(0) as u32;
                let second_group = pop(frame)?.0.max(0) as u8;
                let second_divisor = pop(frame)?.0.max(0) as u32;
                let first_group = pop(frame)?.0.max(0) as u8;
                let first_divisor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
                    player: PlayerId::new(player),
                    divisors: [first_divisor, second_divisor, third_divisor],
                    groups: [first_group, second_group, third_group],
                }
            }
            effect_tag::LOSE_HP_GAIN_DISCARD_CARD_FOR_OPENING_EXTRA_PLAY => {
                let mandatory_skill_def = pop(frame)?.0 as u32;
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                    player: PlayerId::new(player),
                    kind,
                    mandatory_skill_def: engine_api::SkillDefId::new(mandatory_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_OWN_LOST_DISCARD_CARD_THIS_TURN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_MOVE_FIELD_CARD_TO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseMoveFieldCardTo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS => {
                let give_cap = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenOptionallyDistributeHandCards {
                    player: PlayerId::new(player),
                    draw_count,
                    give_cap,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    finish: false,
                }
            }
            effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    used_targets: Vec::new(),
                    total_given: 0,
                    skill: engine_api::SkillId::NONE,
                    continue_only: false,
                }
            }
            effect_tag::CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN => {
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                    player: PlayerId::new(player),
                    mark: engine_api::MarkDefId::new(mark),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                    player: PlayerId::new(player),
                    priority: u8::MAX,
                    remaining_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                    player: PlayerId::new(player),
                    initialized: false,
                    remaining_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HIDDEN_SKILL_PILE => {
                let card = pop(frame)?.0.max(0) as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let pile_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHiddenSkillPile {
                    player: PlayerId::new(player),
                    pile_skill_def: engine_api::SkillDefId::new(pile_skill_def),
                    action,
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    hand_cards: Vec::new(),
                    pile_cards: Vec::new(),
                    event_cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SET_NON_LOCKED_SKILL_SUPPRESSION => {
                let enabled = pop(frame)?.0 != 0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SetNonLockedSkillSuppression {
                    player: PlayerId::new(player),
                    enabled,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DILI_YUHENG => {
                let card = pop(frame)?.0.max(0) as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDiliYuheng {
                    player: PlayerId::new(player),
                    action,
                    card: engine_api::CardId::new(card),
                    event_cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_SELECTED_DRAW_ALL_HAND_BONUS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardSelectedDrawAllHandBonus {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_YINGHUN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherYinghun {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    count: 0,
                    branch: 0,
                    ready_to_discard: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_BIZHENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherBizheng {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_XIASHU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherXiashu {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GUANCHAO_RANK => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGuanchaoRank {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GUANWEI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGuanwei {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LANJIANG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLanjiang {
                    player: PlayerId::new(player),
                    stage: 0,
                    index: 0,
                    target: PlayerId::NONE,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REBINGYI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRebingyi {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REBOTU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRebotu {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDIMENG => {
                let action = pop(frame)?.0.clamp(0, i32::from(u8::MAX)) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldimeng {
                    player: PlayerId::new(player),
                    stage: if action == 0 { 0 } else { 2 },
                    first: PlayerId::NONE,
                    second: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REJIEYIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRejieyin {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    card: host
                        .selected_cards()
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(engine_api::CardId::NONE),
                    equip: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REANXU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveReanxu {
                    player: PlayerId::new(player),
                    stage: 0,
                    first: PlayerId::NONE,
                    second: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    from_hand: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_JUEYAN => {
                let treasure_skill_def = pop(frame)?.0 as u32;
                let armor_skill_def = pop(frame)?.0 as u32;
                let mount_skill_def = pop(frame)?.0 as u32;
                let weapon_skill_def = pop(frame)?.0 as u32;
                let sha_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltJueyan {
                    player: PlayerId::new(player),
                    stage: 0,
                    slot: 0,
                    first: false,
                    sha_skill_def: engine_api::SkillDefId::new(sha_skill_def),
                    weapon_skill_def: engine_api::SkillDefId::new(weapon_skill_def),
                    mount_skill_def: engine_api::SkillDefId::new(mount_skill_def),
                    armor_skill_def: engine_api::SkillDefId::new(armor_skill_def),
                    treasure_skill_def: engine_api::SkillDefId::new(treasure_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REJIZHI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRejizhi {
                    player: PlayerId::new(player),
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIEXUN => {
                let funan_helper_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJiexun {
                    player: PlayerId::new(player),
                    stage: 0,
                    target: PlayerId::NONE,
                    draw_count: 0,
                    discard_count: 0,
                    funan_helper_skill_def: engine_api::SkillDefId::new(
                        funan_helper_skill_def,
                    ),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WENGUA_SETUP => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWenguaSetup {
                    player: PlayerId::new(player),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WENGUA => {
                let marker_skill_def = pop(frame)?.0 as u32;
                let main_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWengua {
                    player: PlayerId::new(player),
                    stage: 0,
                    target: PlayerId::NONE,
                    card: host
                        .selected_cards()
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(engine_api::CardId::NONE),
                    branch: 0,
                    main_skill_def: engine_api::SkillDefId::new(main_skill_def),
                    marker_skill_def: engine_api::SkillDefId::new(marker_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINANGUO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinanguo {
                    player: PlayerId::new(player),
                    target: host
                        .selected_targets()
                        .first()
                        .copied()
                        .map(PlayerId::new)
                        .unwrap_or(PlayerId::NONE),
                    stage: 0,
                    index: 0,
                    remaining: 0b111,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GANGLIE_THREE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGanglieThree {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHONGYI => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0 as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhongyi {
                    player: PlayerId::new(player),
                    action,
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    card: host
                        .selected_cards()
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(engine_api::CardId::NONE),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_EQUIPMENT_SLOT => {
                let slot = pop(frame)?.0 as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardEquipmentSlot {
                    player: PlayerId::new(player),
                    slot,
                }
            }
            effect_tag::ADD_SKILL_UNTIL_TURN_END => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddSkillUntilTurnEnd {
                    player: PlayerId::new(player),
                    skill_def: engine_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER => {
                let hand_count = pop(frame)?.0.max(0) as u32;
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let mark_count = pop(frame)?.0.max(0) as u32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMassDamageDiscardTurnOver {
                    player: PlayerId::new(player),
                    mark: engine_api::MarkDefId::new(mark),
                    mark_count,
                    damage_amount,
                    hand_count,
                    stage: 0,
                    index: 0,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP => {
                let discard_count = pop(frame)?.0.max(0) as u32;
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let mark_count = pop(frame)?.0.max(0) as u32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMassDamageDiscardHandEquip {
                    player: PlayerId::new(player),
                    mark: engine_api::MarkDefId::new(mark),
                    mark_count,
                    damage_amount,
                    discard_count,
                    stage: 0,
                    index: 0,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECORD_ACTIVE_TARGET => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecordActiveTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_WEAPON_OR_LOSE_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardWeaponOrLoseHp {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MARK_PARITY_CONTROL => {
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let mass_threshold = pop(frame)?.0.max(0) as u32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMarkParityControl {
                    player: PlayerId::new(player),
                    mark: engine_api::MarkDefId::new(mark),
                    mass_threshold,
                    damage_amount,
                    stage: 0,
                    index: 0,
                    target: PlayerId::NONE,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE => {
                let nature = pop(frame)?.0.max(0) as u8;
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                    player: PlayerId::new(player),
                    damage_amount,
                    nature,
                    stage: 0,
                    index: 0,
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION => {
                let count = pop(frame)?.0.max(0) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                    player: PlayerId::new(player),
                    count,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YEYAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYeyan {
                    player: PlayerId::new(player),
                    mode: u8::MAX,
                    stage: 0,
                    index: 0,
                    targets: Vec::new(),
                    data: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ALL_LOSE_HP_OR_RECOVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAllLoseHpOrRecover {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIXING => {
                let initialize = pop(frame)?.0 != 0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQixing {
                    player: PlayerId::new(player),
                    initialize,
                    stage: 0,
                    hand_cards: Vec::new(),
                    pile_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIXING_WEATHER => {
                let max_targets = pop(frame)?.0.max(0) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let qixing_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQixingWeather {
                    player: PlayerId::new(player),
                    qixing_skill_def: engine_api::SkillDefId::new(qixing_skill_def),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    max_targets,
                    stage: 0,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NZRY_LONGNU => {
                let thunder_skill_def = pop(frame)?.0 as u32;
                let fire_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNzryLongnu {
                    player: PlayerId::new(player),
                    fire_skill_def: engine_api::SkillDefId::new(fire_skill_def),
                    thunder_skill_def: engine_api::SkillDefId::new(thunder_skill_def),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_LINK_OTHER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseLinkOther {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_POXI => {
                let hand_limit_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltPoxi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    hand_limit_skill_def: engine_api::SkillDefId::new(hand_limit_skill_def),
                    stage: 0,
                    own_count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_JIEYING => {
                let mode = pop(frame)?.0.max(0) as u8;
                let mark = pop(frame)?.0 as u32;
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltJieying {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    mark: engine_api::MarkDefId::new(mark),
                    mode,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_DUORUI => {
                let stage = pop(frame)?.0.max(0) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltDuorui {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    stage,
                    selected_skill_def: engine_api::SkillDefId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_ZHITI => {
                let mode = pop(frame)?.0.max(0) as u8;
                let event_amount = pop(frame)?.0.max(0) as u32;
                let event_source = pop(frame)?.0.max(0) as u32;
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltZhiti {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    event_source: PlayerId::new(event_source),
                    event_amount,
                    mode,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHEN_SIMAYI => {
                let mode = pop(frame)?.0.max(0) as u8;
                let mark = pop(frame)?.0 as u32;
                let event_amount = pop(frame)?.0.max(0) as u32;
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShenSimayi {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    event_amount,
                    mark: engine_api::MarkDefId::new(mark),
                    mode,
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(engine_api::PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BEIGE => {
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBeige {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    stage: 0,
                    result_card: engine_api::CardId::NONE,
                    suit: engine_api::Suit::None,
                    rank: 0,
                    rewriters: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_SKILL_BOUND_CARD_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddSkillBoundCardTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_MATCHING_EQUIP_COLOR_ADD_SKILL => {
                let skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill_def: engine_api::SkillDefId::new(skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CURRENT_CARD_USE_ENHANCEMENTS => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCurrentCardUseEnhancements {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::MAKE_CURRENT_CARD_UNRESPONDABLE => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddCardUsePolicySkill {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    policy_mask: 1 << 1,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                }
            }
            effect_tag::CHOOSE_DRAW_OR_DISCARD_ADD_SKILL => {
                let discard_skill_def = pop(frame)?.0 as u32;
                let draw_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrDiscardAddSkill {
                    player: PlayerId::new(player),
                    draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                    discard_skill_def: engine_api::SkillDefId::new(discard_skill_def),
                    skill: engine_api::SkillId::NONE,
                    three_way: false,
                }
            }
            effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL => {
                let discard_skill_def = pop(frame)?.0 as u32;
                let draw_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrDiscardAddSkill {
                    player: PlayerId::new(player),
                    draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                    discard_skill_def: engine_api::SkillDefId::new(discard_skill_def),
                    skill: engine_api::SkillId::NONE,
                    three_way: true,
                }
            }
            effect_tag::CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
                    player: PlayerId::new(player),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    toggle_link: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    toggle_link: true,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseRevealOtherHandCardRememberCategory {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET => {
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    skill: engine_api::SkillId::NONE,
                    ready_to_give: false,
                }
            }
            effect_tag::CLEAR_PHASE_GIFT_TARGET => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ClearPhaseGiftTarget {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA => {
                let sha_card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
                    player: PlayerId::new(player),
                    sha_card: engine_api::CardId::new(sha_card),
                    shan_cards: Vec::new(),
                    excluded_recipients: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS => {
                let sha_card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
                    player: PlayerId::new(player),
                    sha_card: engine_api::CardId::new(sha_card),
                    shan_cards: Vec::new(),
                    excluded_recipients: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SKIP_CURRENT_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipCurrentPhase {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::FINISH_CURRENT_PHASE_ABORT_RESOLUTION => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::FinishCurrentPhaseAbortResolution {
                    player: PlayerId::new(player),
                    stage: 0,
                }
            }
            effect_tag::GAIN_TOP_CARD_TO_SKILL_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainTopCardToSkillPile {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_SKILL_PILE_CARD_ADD_MARKED_SKILL => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    selected: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_ONE_HAND_FROM_SELECTED_TARGETS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainOneHandFromSelectedTargets {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    index: 0,
                    selected: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_REPOLU_TARGETS_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseRepoluTargetsDraw {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINFU_GUOLUN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinfuGuolun {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    player_card: engine_api::CardId::NONE,
                    target_card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED_RESTORE_IF_EQUAL_RANK => {
                let restore_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let sha_user = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                    player: PlayerId::new(player),
                    sha_user: PlayerId::new(sha_user),
                    card: engine_api::CardId::new(card),
                    restore_skill_def: rule_api::SkillDefId::new(restore_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BEIGE_CONFIGURED => {
                let diamond_draw_count = pop(frame)?.0.max(0) as u32;
                let heart_recover_amount = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBeigeConfigured {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    stage: 0,
                    result_card: engine_api::CardId::NONE,
                    suit: engine_api::Suit::None,
                    rank: 0,
                    rewriters: Vec::new(),
                    heart_recover_amount,
                    diamond_draw_count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHUIYI => {
                let killer = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhuiyi {
                    player: PlayerId::new(player),
                    killer: PlayerId::new(killer),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DECADEZHENJUN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDecadezhenjun {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SP_XIAOGUO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpXiaoguo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                    player: PlayerId::new(player),
                    initialized: false,
                    remaining_targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveSelectedThenRecoverTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    amount,
                    skill: engine_api::SkillId::NONE,
                    ready_to_recover: false,
                }
            }
            effect_tag::RESOLVE_DINGPAN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDingpan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TANLU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTanlu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    amount: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealUniformHandDrawSelfChooseOthers {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CARD_USER_OR_ADD_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCardUserOrAddTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
                    player: PlayerId::new(player),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_EQUIPMENT_PREVENT_CURRENT_DAMAGE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
                    player: PlayerId::new(player),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_OR_DRAW => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseMoveTargetEquipmentOrDraw {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_ALL_EQUIPMENT_CHOOSE_RECOVER_OR_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_WEAPON_RANGE_HAND_OR_INCREASE_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COUNT_DIFFERENT_TARGET_DRAW_OR_RANDOM_DISCARD => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::INCREASE_CURRENT_DRAW_COUNT => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::IncreaseCurrentDrawCount {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                    player: PlayerId::new(player),
                    damage_id: 0,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_ONE_HAND_OR_EQUIP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneHandOrEquip {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DISCARD_ALL_HAND_AND_EQUIP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardAllHandAndEquip {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::REMOVE_ALL_SKILLS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveAllSkills {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::CHOOSE_OTHER_RECOVER => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherRecover {
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::ENABLE_EQUIP_SLOT => {
                let source = pop(frame)?.0.max(0) as u32;
                let slot = pop(frame)?.0.max(0) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::EnableEquipSlot {
                    player: PlayerId::new(player),
                    slot,
                    source: PlayerId::new(source),
                }
            }
            effect_tag::JUDGE_GAIN_EQUIPMENT_BY_COLOR => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeGainEquipmentByColor {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::REVEAL_DISCARD_SUIT_GAIN_OR_USE => {
                let max_mark = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let base_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealDiscardSuitGainOrUse {
                    player: PlayerId::new(player),
                    mark: rule_api::MarkDefId::new(mark),
                    base_count,
                    max_mark,
                    revealed: Vec::new(),
                    completed: 0,
                    step: 0,
                    data: 0,
                }
            }
            effect_tag::CHOOSE_DISCARD_OR_FIRE_DAMAGE_SKIP_PHASES => {
                let damage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let max_discard = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
                    player: PlayerId::new(player),
                    max_discard,
                    damage,
                    selected: 0,
                    step: 0,
                    target: PlayerId::NONE,
                }
            }
            effect_tag::DRAW_SELECTED => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawCards {
                    player: PlayerId::new(player),
                    count: host.selected_cards().len() as u32,
                }
            }
            effect_tag::DRAW_EACH_BY_SEAT => {
                let count = pop(frame)?.0.max(0) as u32;
                let second = pop(frame)?.0.max(0) as u32;
                let first = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawEachBySeat {
                    first: PlayerId::new(first),
                    second: PlayerId::new(second),
                    count,
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_THEN_GIVE_ONE => {
                let mode = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenGiveOne {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    drawn_only: mode & 1 != 0,
                    use_given_equipment: mode & 2 != 0,
                    skill: rule_api::SkillId::NONE,
                    drawn: Vec::new(),
                    ready_to_give: false,
                }
            }
            effect_tag::DRAW_THEN_TAKE_ONE => {
                let count = pop(frame)?.0.max(0) as u32;
                let taker = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenTakeOne {
                    player: PlayerId::new(player),
                    taker: PlayerId::new(taker),
                    count,
                    skill: rule_api::SkillId::NONE,
                    ready_to_take: false,
                }
            }
            effect_tag::TAKE_ONE_CARD => {
                let equip_only = pop(frame)?.0 != 0;
                let to = pop(frame)?.0.max(0) as u32;
                let from = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeOneCard {
                    from: PlayerId::new(from),
                    to: PlayerId::new(to),
                    equip_only,
                }
            }
            effect_tag::TAKE_ONE_EVENT_TARGET_HAND => {
                let to = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeOneHandCard {
                    from: PlayerId::NONE,
                    to: PlayerId::new(to),
                }
            }
            effect_tag::CHOOSE_OTHER_HAND_STORE_SKILL_PILE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherHandStoreSkillPile {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_ALL_SKILL_PILE_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainAllSkillPileCards {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_OTHER_SKILL_PILE_CARDS => {
                let skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainOtherSkillPileCards {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill_def,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_HE_OR_LOSE_HP => {
                let lose_amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneHeOrLoseHp {
                    player: PlayerId::new(player),
                    lose_amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::JUDGE_SPADE_TWO_TO_NINE_DRAW_TWO_OR_THUNDER_SHA_TWICE => {
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_SOURCE_CONFIRM_OWNER_LOSE_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let owner = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseSourceConfirmOwnerLoseHp {
                    owner: PlayerId::new(owner),
                    source: PlayerId::new(source),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_END_EXTRA_DRAW_OR_PLAY_BY_USED_SUITS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::LOSE_HP_DOUBLE_CURRENT_DAMAGE_THEN_REPLACE => {
                let replacement_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
                    player: PlayerId::new(player),
                    damage_id: 0,
                    amount: 0,
                    skill: engine_api::SkillId::NONE,
                    replacement_skill_def: engine_api::SkillDefId::new(replacement_skill_def),
                    ready_to_replace: false,
                }
            }
            effect_tag::TURN_OVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::TurnOver {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESTORE_CHARACTER_STATE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RestoreCharacterState {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DISCARD_ALL_HAND_EQUIP_JUDGE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardAllHandEquipJudge {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DAMAGE_THEN_DISCARD_ALL_JUDGE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DamageThenDiscardAllJudge {
                    player: PlayerId::new(player),
                    ready_to_discard: false,
                }
            }
            effect_tag::REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS => {
                let second_skill_def = pop(frame)?.0 as u32;
                let first_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player: PlayerId::new(player),
                    first_skill_def: rule_api::SkillDefId::new(first_skill_def),
                    second_skill_def: rule_api::SkillDefId::new(second_skill_def),
                    stage: 0,
                    cards: Vec::new(),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    selected: rule_api::CardId::NONE,
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS => {
                let bonus_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    bonus_skill_def: rule_api::SkillDefId::new(bonus_skill_def),
                }
            }
            effect_tag::RESOLVE_DCQIAOMENG => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcqiaomeng {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    targets: Vec::new(),
                    target: PlayerId::NONE,
                    selected: rule_api::CardId::NONE,
                    helper_skill_def: rule_api::SkillDefId::new(helper_skill_def),
                    stage: 0,
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::REMOVE_SKILL_BY_DEF => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveSkillByDef {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR => {
                let black_skill_def = pop(frame)?.0 as u32;
                let red_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandAddSkillByColor {
                    player: PlayerId::new(player),
                    red_skill_def: rule_api::SkillDefId::new(red_skill_def),
                    black_skill_def: rule_api::SkillDefId::new(black_skill_def),
                }
            }
            effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealTopGainNonBasicOrSwapBasic {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::PREVENT_CURRENT_DAMAGE => EffectRequest::PreventDamage { damage_id: 0 },
            effect_tag::JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeCancelCurrentShaTargetOnRed {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::CHOOSE_DRAW_OR_RECOVER => {
                let allow_cancel = pop(frame)?.0 != 0;
                let recover_count = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrRecover {
                    player: PlayerId::new(player),
                    draw_count,
                    recover_count,
                    allow_cancel,
                }
            }
            effect_tag::CHOOSE_USE_VIRTUAL_SHA => {
                let ignore_distance = pop(frame)?.0 != 0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseUseVirtualSha {
                    player: PlayerId::new(player),
                    ignore_distance,
                }
            }
            effect_tag::ADJUST_HAND_TO => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AdjustHandTo {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::REMOVE_CARDS_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                EffectRequest::RemoveCardsByKind { kind }
            }
            effect_tag::GAIN_CARD_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainCardByKind {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            effect_tag::JUDGE_RECOVER_ON_SUIT => {
                let amount = pop(frame)?.0.max(0) as u32;
                let suit = match pop(frame)?.0 {
                    1 => engine_api::Suit::Spade,
                    2 => engine_api::Suit::Heart,
                    3 => engine_api::Suit::Club,
                    4 => engine_api::Suit::Diamond,
                    tag => {
                        return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                    }
                };
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeRecoverOnSuit {
                    source: PlayerId::new(host.context().self_id),
                    player: PlayerId::new(player),
                    suit,
                    amount,
                }
            }
            effect_tag::JUDGE_RECOVER_OTHER_ON_SUIT => {
                let amount = pop(frame)?.0.max(0) as u32;
                let suit = match pop(frame)?.0 {
                    1 => engine_api::Suit::Spade,
                    2 => engine_api::Suit::Heart,
                    3 => engine_api::Suit::Club,
                    4 => engine_api::Suit::Diamond,
                    tag => {
                        return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                    }
                };
                let recover_player = pop(frame)?.0.max(0) as u32;
                let judge_player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeRecoverOtherOnSuit {
                    source: PlayerId::new(host.context().self_id),
                    judge_player: PlayerId::new(judge_player),
                    recover_player: PlayerId::new(recover_player),
                    suit,
                    amount,
                }
            }
            effect_tag::FORCE_OTHERS_USE_SHA_ON_NEAREST_OR_LOSE_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                    player: PlayerId::new(player),
                    remaining: Vec::new(),
                    started: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::USE_VIRTUAL_CARD_ON => {
                let target = pop(frame)?.0.max(0) as u32;
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseVirtualCardOn {
                    player: PlayerId::new(player),
                    kind,
                    target: PlayerId::new(target),
                }
            }
            effect_tag::JUDGE_DAMAGE_OTHER_ON_SUIT => {
                let nature = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let amount = pop(frame)?.0.max(0) as u32;
                let suit = match pop(frame)?.0 {
                    1 => engine_api::Suit::Spade,
                    2 => engine_api::Suit::Heart,
                    3 => engine_api::Suit::Club,
                    4 => engine_api::Suit::Diamond,
                    tag => {
                        return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                    }
                };
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeDamageOtherOnSuit {
                    player: PlayerId::new(player),
                    suit,
                    amount,
                    nature,
                }
            }
            effect_tag::KILL_PLAYER => {
                let killer = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::KillPlayer {
                    player: PlayerId::new(player),
                    killer: PlayerId::new(killer),
                }
            }
            effect_tag::USE_VIRTUAL_CARD_ON_SELECTED_TARGETS => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseVirtualCardOnSelectedTargets {
                    player: PlayerId::new(player),
                    kind,
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                }
            }
            effect_tag::DEAL_DAMAGE_BAN_ACTIVE_IF_DYING => {
                let skill = pop(frame)?.0.max(0) as u32;
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamageBanActiveIfDying {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    amount,
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED => {
                let skill = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_USE_VIRTUAL_CARD => {
                let skill = pop(frame)?.0.max(0) as u32;
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseUseVirtualCard {
                    player: PlayerId::new(player),
                    kind,
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_USE_VIRTUAL_NANMAN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseUseVirtualNanman {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_QILUAN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQiluan {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::DISCARD_ALL_OWN_JUDGE => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardAllOwnJudge {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE => {
                let skill = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::StoreEventCardInSkillPile {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_OLLINJIE => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOllinjie {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_GAIN_MAX_HP => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetGainMaxHp {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::GAIN_TARGET_HAND_CARD => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainTargetHandCard {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_CLAN_NOT_MAX_HAND_LIMIT_GAIN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseClanNotMaxHandLimitGain {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::GAIN_FROM_SKILL_PILE => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainFromSkillPile {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::DISCARD_SKILL_PILE => {
                // pile_def 是 u32 def id 字面量：经 i32 栈必须按位型还原（禁 max(0) 钳位，
                // 否则 > i32::MAX 的 def id 归零——批 193 定位的生产管线丢堆根因）。
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardSkillPile {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                }
            }
            effect_tag::PLACE_TOP_DECK_TO_EXPANSION => {
                let count = pop(frame)?.0.max(0) as u32;
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceTopDeckToExpansion {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                    count,
                }
            }
            effect_tag::CHOOSE_ONE_FROM_SKILL_PILE_DISCARD => {
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOneFromSkillPileDiscard {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                }
            }
            effect_tag::CHOOSE_TARGET_HAND_CARD_TO_PILE => {
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetHandCardToPile {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                }
            }
            effect_tag::REARRANGE_PILE_TO_DECK_TOP => {
                let count = pop(frame)?.0.max(0) as u32;
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RearrangePileToDeckTop {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                    count,
                }
            }
            effect_tag::RESOLVE_QIANLONG => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQianlong {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_JSRGZUOZHAN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgzuozhan {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_JSRGZUOZHAN_GAIN => {
                let event_player = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgzuozhanGain {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    event_player: PlayerId::new(event_player),
                }
            }
            effect_tag::RESOLVE_TWMOUZHI => {
                let event_card = pop(frame)?.0.max(0) as u32;
                let mode = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwmouzhi {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    mode,
                    event_card,
                }
            }
            effect_tag::RESOLVE_TWDAIGUI => {
                let count = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwdaigui {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    count,
                }
            }
            effect_tag::RESOLVE_TWMITU => {
                let count = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwmitu {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    count,
                }
            }
            effect_tag::RESOLVE_QIANTUN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQiantun {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_YANGMING => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYangming {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_CANYUN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveCanyun {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_HENGWU => {
                let event_card = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHengwu {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    event_card,
                }
            }
            effect_tag::RESOLVE_BIAOZHAO3 => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBiaozhao3 {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_BIAOZHAO2 => {
                let event_player = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBiaozhao2 {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    event_player: PlayerId::new(event_player),
                }
            }
            effect_tag::RESOLVE_DCNIJI => {
                let event_card = pop(frame)?.0.max(0) as u32;
                let mode = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcniji {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    mode,
                    event_card,
                }
            }
            effect_tag::RESOLVE_DCMOUKUI => {
                let target = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcmoukui {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_ZHONGYAN => {
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhongyan {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                }
            }
            effect_tag::RESOLVE_CHUMEI => {
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveChumei {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                }
            }
            effect_tag::RESOLVE_SHUAIYAN => {
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShuaiyan {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                }
            }            effect_tag::RESOLVE_TW_XIONGJUN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwxiongjun {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_YIXIANG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYixiang {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_XINFU_QIAI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinfuQiai {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_XIANGSHU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXiangshu {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                }
            }
            effect_tag::RESOLVE_NHLUNDAO => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNhlundao {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                }
            }
            effect_tag::RESOLVE_SANSHOU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSanshou {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_STARCUXIA_WEIMING => {
                let variant = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveStarcuxiaWeiming {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    variant,
                }
            }
            effect_tag::RESOLVE_ZHONGJIE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhongjie {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_DCHAOXIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDchaoxian {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_DCZHEE => {
                let mode = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDczhee {
                    player: PlayerId::new(player),
                    mode,
                }
            }
            effect_tag::RESOLVE_OLQIAOZHI => {
                let stage = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOlqiaozhi {
                    player: PlayerId::new(player),
                    stage,
                }
            }
            effect_tag::RESOLVE_OLSUYI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOlsuyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_OLZHIJIE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOlzhijie {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_SPOLAIGE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpolaige {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_OLWENREN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOlwenren {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCQIAOLVE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcqiaolve {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_DINGCE => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDingce {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                }
            }
            effect_tag::RESOLVE_QIANGYONG => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQiangyong {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::RESOLVE_MEIYONG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMeiyong {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_MBZHENANN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbzhenann {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_FRENDBIWEI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveFriendbiwei {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::new(
                        host.selected_cards().first().copied().unwrap_or(0),
                    ),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::RESOLVE_CHULAO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveChulao {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                }
            }
            effect_tag::RESOLVE_DCANXU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcanxu {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::RESOLVE_DUODUAN => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDuoduan {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    skill: engine_api::SkillId::new(host.context().skill_id),
                }
            }
            effect_tag::RESOLVE_GAOYUAN => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGaoyuan {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                }
            }
            effect_tag::RESET_LINKED_SELECTED_TARGETS => {
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResetLinkedSelectedTargets {
                    source: PlayerId::new(source),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                }
            }
            effect_tag::RESOLVE_MBBEIMING => {
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbbeiming {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                }
            }
            effect_tag::RESOLVE_RELIANGYIN => {
                let mode = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveReliangyin {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    mode,
                }
            }
            effect_tag::RESOLVE_ZHENGJIAN => {
                let event_player = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhengjian {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    event_player: PlayerId::new(event_player),
                }
            }
            effect_tag::RESOLVE_QINMIAN => {
                let amount = pop(frame)?.0.max(0) as u32;
                let mode = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQinmian {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    mode,
                    amount,
                }
            }
            effect_tag::RESOLVE_CHUANYUN => {
                let event_card = pop(frame)?.0.max(0) as u32;
                let event_player = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveChuanyun {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    event_player: PlayerId::new(event_player),
                    event_card,
                }
            }
            effect_tag::RESOLVE_JSRGDAOREN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgdaoren {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_DUWANG => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDuwang {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_QUANJIA => {
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQuanjia {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_PXQIXIAN => {
                let kind = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePxqixian {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    kind,
                }
            }
            effect_tag::RESOLVE_DCBINGJI => {
                let kind = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcbingji {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    stage,
                    kind,
                }
            }
            effect_tag::CHOOSE_HAND_CARDS_TO_SKILL_PILE => {
                let max_count = pop(frame)?.0 as u32;
                let min_count = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCardsToSkillPile {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    min_count,
                    max_count,
                }
            }
            effect_tag::PLACE_HAND_CARDS_TO_DECK_TOP => {
                let count = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceHandCardsToDeckTop {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    count,
                }
            }
            effect_tag::GAIN_WHOLE_SKILL_PILE => {
                let pile_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainWholeSkillPile {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                    pile_def,
                }
            }
            effect_tag::DISCARD_BASIC_DRAW_EQUAL => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardBasicDrawEqual {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::START_DEBATE_ALL_OTHERS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::StartDebateAllOthers {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::START_DEBATE_TARGETS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::StartDebateTargets {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::PIN_DEBATE_OPINIONS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PinDebateOpinions {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::ALIGN_DEBATE_OPINIONS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AlignDebateOpinions {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::GAIN_SHADOW_CARDS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainShadowCards {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHANGE_GROUP => {
                let skill = pop(frame)?.0.max(0) as u32;
                let group_code = pop(frame)?.0.max(0) as usize;
                let player = pop(frame)?.0.max(0) as u32;
                let group = match group_code {
                    0 => "wei".to_string(),
                    1 => "shu".to_string(),
                    2 => "wu".to_string(),
                    3 => "qun".to_string(),
                    4 => "jin".to_string(),
                    _ => String::new(),
                };
                EffectRequest::ChangeGroup {
                    player: PlayerId::new(player),
                    group,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::DISTRIBUTE_SHADOW_CARDS => {
                let skill = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DistributeShadowCards {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::REGISTER_TURN_END_DEBATE => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RegisterTurnEndDebate {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_TURN_END_DEBATES => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTurnEndDebates {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_GROUP => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGroup {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_GAIN_OTHER_SKILL => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainOtherSkill {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::LOSE_GAINED_SKILL => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::LoseGainedSkill {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_POTHEYUN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotheyun {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::PLACE_XUMOU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceXumou {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::PLACE_DECK_TOP_XUMOU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceDeckTopXumou {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_PLACE_HAND_XUMOU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChoosePlaceHandXumou {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::DISCARD_ONE_OWN_XUMOU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardOneOwnXumou {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::PLACE_DECK_TOP_EXPANSION => {
                let skill = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceDeckTopExpansion {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CITY_CONSUME => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::CityConsume {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::QUEUE_EXTRA_TURN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let check = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QueueExtraTurn {
                    player: PlayerId::new(player),
                    check,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_OTHERS_ADD_MARK => {
                let skill = pop(frame)?.0.max(0) as u32;
                let max_count = pop(frame)?.0.max(0) as u32;
                let delta = pop(frame)?.0 as i32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOthersAddMark {
                    player: PlayerId::new(player),
                    mark,
                    delta,
                    max_count,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_OTHERS_INSERT_EXTRA_TURN => {
                let skill = pop(frame)?.0.max(0) as u32;
                let check = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOthersInsertExtraTurn {
                    player: PlayerId::new(player),
                    check,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::QUEUE_EXTRA_PHASES => {
                let skill = pop(frame)?.0.max(0) as u32;
                let mask = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QueueExtraPhases {
                    player: PlayerId::new(player),
                    phases_mask: mask,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_TWJIEQIU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let sub = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwjieqiu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    subaction: sub,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::SKIP_REMAINING_PHASES => {
                let skill = pop(frame)?.0.max(0) as u32;
                let mask = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipRemainingPhases {
                    player: PlayerId::new(player),
                    phases_mask: mask,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_JSRGCHAOZHENG => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgchaozheng {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_JSRGSHEJU => {
                let skill = pop(frame)?.0.max(0) as u32;
                let counterpart = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgsheju {
                    player: PlayerId::new(player),
                    counterpart: PlayerId::new(counterpart),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_OTHER_ADD_MARK => {
                let delta = pop(frame)?.0 as i32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherAddMark {
                    player: PlayerId::new(player),
                    mark,
                    delta,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::PLACE_HAND_CARD_TO_EXPANSION => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::PlaceHandCardToExpansion {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::EXTRA_RESOLVE_CURRENT_TRICK => {
                let skill = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ExtraResolveCurrentTrick {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::INCREASE_CURRENT_SHA_BASE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::IncreaseCurrentShaBaseDamage {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SKIP_NEXT_PLAY_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipNextPlayPhase {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::SKIP_NEXT_JUDGE_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipNextJudgePhase {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT => {
                let amount = pop(frame)?.0.max(0) as u32;
                let suit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    suit,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DRAW_OR_RECOVER => {
                let times = pop(frame)?.0.max(0) as u32;
                let recover_count = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDrawOrRecover {
                    player: PlayerId::new(player),
                    draw_count,
                    recover_count,
                    times,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_SELECTED_TARGETS => {
                let count = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawSelectedTargets {
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECOVER_IF_MIN_CURRENT_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverIfMinCurrentHp {
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::INCREASE_CURRENT_CARD_BASE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::IncreaseCurrentCardBaseDamage {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
                        effect_tag::LOSE_HP_THEN_DAMAGE_OTHER_LOSE_MAX_HP_IF_DYING => {
                let skill = pop(frame)?.0.max(0) as u32;
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    amount,
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RECOVER_SELECTED_TARGETS => {
                let amount = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverSelectedTargets {
                    source: PlayerId::new(host.context().self_id),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECORD_SELECTED_PLAY_PHASE_CATEGORY => {
                let skill = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::RecordSelectedPlayPhaseCategory {
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX => {
                let skill = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeTargetHandThenRepeatUntilMax {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK => {
                let mark_count = pop(frame)?.0.max(0) as u32;
                // mark_def 是 u32 def id 字面量：位型保留（禁 max(0) 钳位）。
                let mark_def = pop(frame)?.0 as u32;
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    draw_count,
                    damage_amount,
                    mark_def: engine_api::MarkDefId::new(mark_def),
                    mark_count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND => {
                let payment_count = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawThenChooseGiveOrDiscardHand {
                    player: PlayerId::new(player),
                    draw_count,
                    payment_count,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::GAIN_TARGET_EQUIPMENT => {
                let slot = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainTargetEquipment {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    slot,
                }
            }
            effect_tag::DISCARD_ALL_TARGET_EQUIPMENT => {
                let target = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardAllTargetEquipment {
                    actor: PlayerId::new(actor),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::INCREASE_CURRENT_SHA_TARGET_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::IncreaseCurrentShaTargetDamage {
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    target: PlayerId::new(target),
                    amount,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    count,
                    skill: engine_api::SkillId::NONE,
                    ready_to_check: false,
                }
            }
            effect_tag::CHOOSE_BOUND_SOURCE_ATTACK_RANGE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseBoundSourceAttackRangeDamage {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS => {
                let cap = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    cap,
                    skill: engine_api::SkillId::NONE,
                    ready_to_discard: false,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL => {
                let add_def = pop(frame)?.0 as u32;
                let cap = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    cap,
                    skill: engine_api::SkillId::NONE,
                    add_def: engine_api::SkillDefId::new(add_def),
                }
            }
            effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    suit: engine_api::Suit::None,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainEquipmentFromDrawPile {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSourceDiscardTwoHandOrDamage {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                }
            }
            effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                    ready_to_recover: false,
                }
            }
            effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    index: 0,
                    selected: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    index: 0,
                    selected: engine_api::CardId::NONE,
                    draw_count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISTRIBUTE_TOP_TWO_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DistributeTopTwoCards {
                    player: PlayerId::new(player),
                    cards: Vec::new(),
                    selected: Vec::new(),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveAllHandRecoverToOne {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    ready_to_recover: false,
                }
            }
            effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_check: false,
                }
            }
            effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER => {
                let target = pop(frame)?.0.max(0) as u32;
                let giver = pop(frame)?.0.max(0) as u32;
                let owner = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGiveOneHandEquipOrRecover {
                    owner: PlayerId::new(owner),
                    giver: PlayerId::new(giver),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GIVE_TWO_DIFFERENT_COLORS_OR_RECOVER => {
                let target = pop(frame)?.0.max(0) as u32;
                let giver = pop(frame)?.0.max(0) as u32;
                let owner = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
                    owner: PlayerId::new(owner),
                    giver: PlayerId::new(giver),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBZHUGUO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbzhuguo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBANXIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbanxian {
                    player: PlayerId::new(player),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBJINZU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbjinzu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::TAKE_ONE_SCOPED_CARD => {
                let scope = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let to = pop(frame)?.0.max(0) as u32;
                let from = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeOneScopedCard {
                    from: PlayerId::new(from),
                    to: PlayerId::new(to),
                    scope: rule_api::TargetCardScope::from_tag(scope)
                        .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?,
                }
            }
            effect_tag::REVEAL_HAND => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHand {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DRAW_LINKED_PLAYERS => {
                let count = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawLinkedPlayers { count }
            }
            effect_tag::REVEAL_TOP_GAIN_IF_COLOR => {
                let color_mask = pop(frame)?.0.max(0) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealTopGainIfColor {
                    player: PlayerId::new(player),
                    color_mask,
                }
            }
            effect_tag::SPJUNGONG_DISCARD_OR_LOSE_THEN_SHA => {
                let skill = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SpjungongDiscardOrLoseThenSha {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::BAN_ACTIVE_UNTIL_TURN_END => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::BanActiveUntilTurnEnd {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::SPWANWEI_ACTIVE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SpwanweiActive {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::SPWANWEI_DYING => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SpwanweiDying {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_SPXIZHAN => {
                let fangzong = pop(frame)?.0.max(0) as u32;
                let phase_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpxizhan {
                    player: PlayerId::new(player),
                    phase_player: PlayerId::new(phase_player),
                    fangzong_skill_def: rule_api::SkillDefId::new(fangzong),
                }
            }
            effect_tag::RESOLVE_SPSONGSHU => {
                let block = pop(frame)?.0.max(0) as u32;
                let renku = pop(frame)?.0.max(0) as u32;
                let phase_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpsongshu {
                    player: PlayerId::new(player),
                    phase_player: PlayerId::new(phase_player),
                    renku_skill_def: rule_api::SkillDefId::new(renku),
                    block_skill_def: rule_api::SkillDefId::new(block),
                }
            }
            effect_tag::RESOLVE_SPCUNSI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpcunsi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_TWFUPAN => {
                let damage_amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwfupan {
                    player: PlayerId::new(player),
                    damage_amount,
                    card: engine_api::CardId::NONE,
                    target: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWJIAOHUA => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwjiaohua {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    event_cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWKUJIAN => {
                let action = pop(frame)?.0.max(0) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let context = host.context();
                let mut cards = if action == 0 {
                    host.selected_cards().to_vec()
                } else {
                    host.event_cards().to_vec()
                };
                if cards.is_empty() && context.event_card != 0 {
                    cards.push(context.event_card);
                }
                EffectRequest::ResolveTwkujian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: cards.into_iter().map(engine_api::CardId::new).collect(),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWRUILIAN => {
                let action = pop(frame)?.0.max(0) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwruilian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    event_cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    action,
                    category: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SKIP_JUDGE_AND_DRAW_PHASES => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipJudgeAndDrawPhases {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::EQUIP_WEAPON_FROM_DRAW_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::EquipWeaponFromDrawPile {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_TWJIANWEI => {
                let stage = pop(frame)?.0.max(0) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwjianwei {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    winner: PlayerId::NONE,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWTANFENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwtanfeng {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    phase: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWZHIBIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwzhibian {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    winner: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWYUYAN => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwyuyan {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWGUOYI => {
                let hand_lock_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwguoyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    hand_lock_skill_def,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCHUHAI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwchuhai {
                    player: PlayerId::new(player),
                    action,
                    stage: 0,
                    index: 0,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWYIZHU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwyizhu {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    stage: 0,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWLUANCHOU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwluanchou {
                    player: PlayerId::new(player),
                    helper_skill_def,
                    action,
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    amount: 0,
                    nature: 0,
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    card_flags: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWXIAWEI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let wangxing_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwxiawei {
                    player: PlayerId::new(player),
                    wangxing_skill_def,
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWBIANFUREN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwbianfuren {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def,
                    action,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWFEIYI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwfeiyi {
                    player: PlayerId::new(player),
                    action,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCHENZHEN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let record_skill_def = pop(frame)?.0 as u32;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwchenzhen {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def,
                    record_skill_def,
                    action,
                    stage: 0,
                    card_kind: 0,
                    card: engine_api::CardId::NONE,
                    branch: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWQIANJU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwqianju {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWQINGXI => {
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwqingxi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    count: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWZAOLI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwzaoli {
                    player: PlayerId::new(player),
                    stage: 0,
                    cards: Vec::new(),
                    slots: Vec::new(),
                    count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWZHENGJIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwzhengjian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def,
                    action,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    branch: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWTIANYU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwtianyu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_skill_def,
                    action,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    branch: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWWUJING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwwujing {
                    player: PlayerId::new(player),
                    target_count,
                    action,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWSHANGHE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwshanghe {
                    player: PlayerId::new(player),
                    stage: 0,
                    remaining: Vec::new(),
                    got_jiu: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWWANGCHANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwwangchang {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    selected_card: engine_api::CardId::NONE,
                    action,
                    stage: 0,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCAOZHAO => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let twfeifu_skill_def = pop(frame)?.0 as u32;
                let twfuzuan_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwcaozhao {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    selected_skill_def: 0,
                    twfuzuan_skill_def,
                    twfeifu_skill_def,
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWGUOHUAI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let mark_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwguohuai {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    mark_def,
                    helper_skill_def,
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCHENGPU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let chunlao_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwchengpu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    chunlao_skill_def,action,
                    stage: 0,
                    selected_card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCAOANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwcaoang {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    helper_skill_def,
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWNIUJIN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let limited_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwniujin {
                    player: PlayerId::new(player),
                    limited_skill_def: engine_api::SkillDefId::new(limited_skill_def),
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RETIANXIANG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRetianxiang {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    discard_card: engine_api::CardId::NONE,
                    branch: 0,
                    source: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LIANZHOU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLianzhou {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_STDDECHONG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveStddechong {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_HAND_EQUIP_COUNT => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardHandEquipCount {
                    player: PlayerId::new(player),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIELIANG_GAIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJieliangGain {
                    player: PlayerId::new(player),
                    mover: PlayerId::NONE,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_STDJUJIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveStdjujian {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LULVE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLulve {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    branch: 2,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCTINGXIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDctingxian {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::NONE,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HFJIEYING => {
                let grant_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHfjieying {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    grant_def: engine_api::SkillDefId::new(grant_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCNUCHEN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcnuchen {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCDANJI => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcdanji {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::RESOLVE_REFUHAI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRefuhai {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_TARGET_HAND_CEIL_HALF_DIFFERENCE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::RESOLVE_GZJINFA => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let cost_card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ResolveGzjinfa {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cost_card: engine_api::CardId::new(cost_card),
                    given_card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_WN_QIFENG => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveWnQifeng {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCZHIHENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDczhiheng {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_STDGAOBIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveStdgaobian {
                    player: PlayerId::new(player),
                    chooser: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPJIEDAO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpjiedao {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SIBIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSibian {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SKIP_NEXT_DISCARD_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SkipNextDiscardPhase {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::RESOLVE_SPFENGYIN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpfengyin {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINFU_LIANPIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinfuLianpian {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GONGJIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGongjian {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_CHOOSE_CATEGORY_DISTRIBUTE_SELF_THRESHOLD_HP_LOSS => {
                let recipient_cap = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let reveal_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player: PlayerId::new(player),
                    reveal_count,
                    recipient_cap,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_RANDOM_GAIN_REGION_THEN_DRAW_COMPARE_COLOR_LOSE_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINFU_QINGUO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinfuQinguo {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCMANZHI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcmanzhi {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINXINGLUAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinxingluan {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCKUIZHEN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDckuizhen {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCKUIZHEN_RESOLVE => {
                let loser = pop(frame)?.0.max(0) as u32;
                let winner = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDckuizhenResolve {
                    player: PlayerId::new(player),
                    winner: PlayerId::new(winner),
                    loser: PlayerId::new(loser),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NSDINGZHOU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNsdingzhou {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SXRMLANJIAO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSxrmlanjiao {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    view: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REKUANSHI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRekuanshi {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REKUANSHI_RESOLVE => {
                let damaged = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRekuanshiResolve {
                    player: PlayerId::new(player),
                    damaged: PlayerId::new(damaged),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCMOYU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcmoyu {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XIZHEN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXizhen {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XUXIE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXuxie {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XUXIE_ADD => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXuxieAdd {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCCHIYING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcchiying {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 1,
                    accum: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINKUANGFU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinkuangfu {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    sha_target: PlayerId::NONE,
                    stage: 1,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCXIANG_SHU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcxiangshu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCXIANG_SHU_LOTTERY => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcxiangshuLottery {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    targets: Vec::new(),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCYINGTU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcyingtu {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MOBILEDANSHOU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMobiledanshou {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JSRGCHIYING => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJsrgchiying {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    started: false,
                    remaining: Vec::new(),
                    accum: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBQINGDAO => {
                let damaged = pop(frame)?.0 != 0;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbqingdao {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    damaged,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBXIUGENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let hand_limit_def = pop(frame)?.0 as u32;
                let helper_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbxiugeng {
                    player: PlayerId::new(player),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    hand_limit_def: engine_api::SkillDefId::new(hand_limit_def),
                    stage,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBCHENSHE => {
                let source = pop(frame)?.0.max(0) as u32;
                let dying = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbchenshe {
                    player: PlayerId::new(player),
                    dying: PlayerId::new(dying),
                    source: PlayerId::new(source),
                    stage: 0,
                    roles: Vec::new(),
                    suits: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBQUNCHONG => {
                let helper_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbquanchong {
                    player: PlayerId::new(player),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBRENXING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbrenxing {
                    player: PlayerId::new(player),
                    stage: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBCHIZHANG => {
                let card = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbchizhang {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBDUANYANG => {
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbduanyang {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    mode,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBRUNWEI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbrunwei {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    value: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBSHUANGHUAI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbshuanghuai {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    choice: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBXIONGTU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbxiongtu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    card: engine_api::CardId::NONE,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBGANGGENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbganggeng {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBSIJIAN => {
                let helper_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbsijian {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    choice: 0,
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBJIANJI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbjianji {
                    player: PlayerId::new(player),
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBYUANMO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbyuanmo {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    value: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBFEIJING => {
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbfeijing {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    targets: Vec::new(),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBXIAOGE => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbxiaoge {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBXUYE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbxuye {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    was_max: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBKUANGXIANG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbkuangxiang {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTCHENGPU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotchengpu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTJUGU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotjugu {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTSIFENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotsifeng {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTCHENQUN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotchenqun {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTWANGLIE => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotwanglie {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTHONGYI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePothongyi {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTGUANSHA => {
                let helper_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotguansha {
                    player: PlayerId::new(player),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTJIYU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotjiyu {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(
                        host.selected_cards().first().copied().unwrap_or(0),
                    ),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBZHUJI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbzhuji {
                    player: PlayerId::new(player),
                    choice: 0,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTLIBING => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotlibing {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTPOXI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotpoxi {
                    player: PlayerId::new(player),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTFUAN => {
                let yinxian_def = pop(frame)?.0 as u32;
                let range_helper_def = pop(frame)?.0 as u32;
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotfuan {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    stage,
                    range_helper_def: engine_api::SkillDefId::new(range_helper_def),
                    yinxian_def: engine_api::SkillDefId::new(yinxian_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTYINXIAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotyinxian {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTHUANJIE => {
                let kanpo_def = pop(frame)?.0 as u32;
                let qice_def = pop(frame)?.0 as u32;
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePothuanjie {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    qice_def: engine_api::SkillDefId::new(qice_def),
                    kanpo_def: engine_api::SkillDefId::new(kanpo_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTNIGU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotnigu {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    targets: Vec::new(),
                    index: 0,
                    declined: 0,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTLULIAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotlulian {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    target: PlayerId::NONE,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTDIMENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotdimeng {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTHAOSHI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePothaoshi {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TIAOXIN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTiaoxin {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHIJI => {
                let guanxing_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhiji {
                    player: PlayerId::new(player),
                    guanxing_def: engine_api::SkillDefId::new(guanxing_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPMIAOLVE => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpmiaolve {
                    player: PlayerId::new(player),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPYINGJIA => {
                let helper_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpyingjia {
                    player: PlayerId::new(player),
                    action,
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YULI => {
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYuli {
                    player: PlayerId::new(player),
                    mode,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    amount: 0,
                    nature: 0,
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    card_flags: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TINGWEI => {
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTingwei {
                    player: PlayerId::new(player),
                    mode,
                    card: engine_api::CardId::NONE,
                    targets: Vec::new(),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINMINGSHI => {
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinmingshi {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XUNYI => {
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXunyi {
                    player: PlayerId::new(player),
                    mode,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BOMING => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBoming {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_EJIAN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveEjian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YIYONG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYiyong {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::NONE,
                    source: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHANXIE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShanxie {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHEYI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSheyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TIANYIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTianyin {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DBZHUIFENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDbzhuifeng {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DBQUEDI => {
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDbquedi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPYINJU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpyinju {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPCHIJIE => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpchijie {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPSHANGYI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpshangyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPJIANYI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpjianyi {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FYJIANYU => {
                let b = pop(frame)?.0.max(0) as u32;
                let a = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveFyjianyu {
                    player: PlayerId::new(player),
                    source: PlayerId::new(a),
                    target: PlayerId::new(b),
                    targets: host.selected_targets().iter().copied().map(PlayerId::new).collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LIUBING => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLiubing {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HEJI => {
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHeji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPYAJUN => {
                let stage = pop(frame)?.0.max(0) as u32;
                let less_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpyajun {
                    player: PlayerId::new(player),
                    less_def: engine_api::SkillDefId::new(less_def),
                    card: engine_api::CardId::NONE,
                    target: PlayerId::NONE,
                    target_card: engine_api::CardId::NONE,
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPZUNDI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpzundi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    destination: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINYANGJIE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinyangjie {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    target_card: engine_api::CardId::NONE,
                    source: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QINGYU => {
                let mbyongjue_def = pop(frame)?.0 as u32;
                let xuancun_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQingyu {
                    player: PlayerId::new(player),
                    action,
                    xuancun_def: engine_api::SkillDefId::new(xuancun_def),
                    mbyongjue_def: engine_api::SkillDefId::new(mbyongjue_def),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHENGSU => {
                let card = pop(frame)?.0.max(0) as u32;
                let stage = pop(frame)?.0.max(0) as u32;
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhengsu {
                    player: PlayerId::new(player),
                    mode,
                    stage: stage as u8,
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HOUFENG => {
                let stage = pop(frame)?.0.max(0) as u32;
                let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHoufeng {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    mode,
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBDIANCAI => {
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbdiancai {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RECHUHAI => {
                let stage = pop(frame)?.0.max(0) as u32;
                let target_card = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRechuhai {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    target_card: engine_api::CardId::new(target_card),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XIONGSHU => {
                let stage = pop(frame)?.0.max(0) as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXiongshu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIANHUI => {
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJianhui {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPQIAI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpqiai {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPSHANXI => {
                let stage = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpshanxi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SPSHANXI_SUOMING => {
                let stage = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveSpshanxiSuoming {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TONGDUO => {
                let stage = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTongduo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YUANQING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYuanqing {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHUCHEN => {
                let renku_def = pop(frame)?.0 as u32;
                let dying = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShuchen {
                    player: PlayerId::new(player),
                    dying: PlayerId::new(dying),
                    renku_def: engine_api::SkillDefId::new(renku_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DUANBI => {
                let stage = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDuanbi {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZAOLI => {
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZaoli {
                    player: PlayerId::new(player),
                    mark,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GUYING => {
                let stage = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGuying {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    card: engine_api::CardId::new(card),
                    stage: stage as u8,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZAIQI => {
                let extra = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZaiqi {
                    player: PlayerId::new(player),
                    extra,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWQIUSHOU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwqiushou {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTTUNTIAN => {
                let zaoxian_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePottuntian {
                    player: PlayerId::new(player),
                    action,
                    amount: 0,
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    zaoxian_def: engine_api::SkillDefId::new(zaoxian_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTJIXI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotjixi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTCEDUAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotceduan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTTAISHICI => {
                let zhanlie_def = pop(frame)?.0 as u32;
                let hanzhan_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePottaishici {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    hanzhan_def: engine_api::SkillDefId::new(hanzhan_def),
                    zhanlie_def: engine_api::SkillDefId::new(zhanlie_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RELEIJI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveReleiji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HUANGTIAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHuangtian {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBZHONGHUI => {
                let mbkechang_def = pop(frame)?.0 as u32;
                let mbsizi_def = pop(frame)?.0 as u32;
                let amount = pop(frame)?.0.max(0) as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbzhonghui {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    action,
                    amount,
                    mbsizi_def: engine_api::SkillDefId::new(mbsizi_def),
                    mbkechang_def: engine_api::SkillDefId::new(mbkechang_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTJIEJIE => {
                let qingshi_def = pop(frame)?.0 as u32;
                let diamond_def = pop(frame)?.0 as u32;
                let club_def = pop(frame)?.0 as u32;
                let heart_def = pop(frame)?.0 as u32;
                let spade_def = pop(frame)?.0 as u32;
                let used_def = pop(frame)?.0 as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotjiejie {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    used_def: engine_api::SkillDefId::new(used_def),
                    suit_defs: [spade_def, heart_def, club_def, diamond_def]
                        .map(engine_api::SkillDefId::new),
                    qingshi_def: engine_api::SkillDefId::new(qingshi_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTQINGSHI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotqingshi {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTWEIYAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotweiyan {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTZHOUYU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotzhouyu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    card_flags: 0,
                    targets: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTLVYI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotlvyi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    event_cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    targets: host
                        .event_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTCAOSHUANG => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotcaoshuang {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_POTYUJI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolvePotyuji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    targets: host
                        .event_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REHENGJIANG => {
                let draw_def = pop(frame)?.0 as u32;
                let hand_limit_def = pop(frame)?.0 as u32;
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRehengjiang {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    action,
                    hand_limit_def: engine_api::SkillDefId::new(hand_limit_def),
                    draw_def: engine_api::SkillDefId::new(draw_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HAOSHI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHaoshi {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DIMENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDimeng {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIAOBIAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let phase = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveQiaobian {
                    player: PlayerId::new(player),
                    phase,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::EXCLUDE_SAVED_SHA_TARGET => {
                let target = pop(frame)?.0 as u32;
                let card = pop(frame)?.0 as u32;
                EffectRequest::ExcludeSavedShaTarget {
                    card: engine_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_XIANGLE => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveXiangle {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FANGQUAN_SKIP => {
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveFangquanSkip {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FANGQUAN_GRANT => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveFangquanGrant {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RUOYU => {
                let jijiang_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveRuoyu {
                    player: PlayerId::new(player),
                    jijiang_def: engine_api::SkillDefId::new(jijiang_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZUILUN => {
                    let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveZuilun {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FUYIN => {
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveFuyin {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::BAN_USE_AGAINST => {
                let target = pop(frame)?.0 as u32;
                let user = pop(frame)?.0 as u32;
                EffectRequest::BanUseAgainst {
                    user: PlayerId::new(user),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::RESOLVE_JUZHAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveJuzhan {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    targets: host
                        .event_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIZHI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveQizhi {
                    player: PlayerId::new(player),
                    targets: host
                        .event_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JINQU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let qizhi_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveJinqu {
                    player: PlayerId::new(player),
                    qizhi_def: engine_api::SkillDefId::new(qizhi_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIANXIANG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveJianxiang {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHENSHI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveShenshi {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MINGREN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveMingren {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHENLIANG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let mingren_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveZhenliang {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    mingren_def: engine_api::SkillDefId::new(mingren_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_KUIZHU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveKuizhu {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHIZHENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveZhizheng {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LIJUN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveLijun {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LIANGYIN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let kongsheng_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveLiangyin {
                    player: PlayerId::new(player),
                    kongsheng_def: engine_api::SkillDefId::new(kongsheng_def),
                    stage,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_KONGSHENG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0 as u32;
                EffectRequest::ResolveKongsheng {
                    player: PlayerId::new(player),
                    stage,
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHENGRONG => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhengrong {
                    player: PlayerId::new(player),
                    candidates: host
                        .event_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    target: PlayerId::new(target),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_HONGJU => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let rong_def = pop(frame)?.0 as u32;
                let qingjian_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveHongju {
                    player: PlayerId::new(player),
                    qingjian_def: engine_api::SkillDefId::new(qingjian_def),
                    rong_def: engine_api::SkillDefId::new(rong_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QINGJIAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let rong_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQingjian {
                    player: PlayerId::new(player),
                    rong_def: engine_api::SkillDefId::new(rong_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TUNTIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTuntian {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZAOXIAN => {
                let tuntian_def = pop(frame)?.0 as u32;
                let jixi_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZaoxian {
                    player: PlayerId::new(player),
                    jixi_def: engine_api::SkillDefId::new(jixi_def),
                    tuntian_def: engine_api::SkillDefId::new(tuntian_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHUANGXIONG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShuangxiong {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHIJIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ResolveZhijian {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_GUZHENG => {
                let event_player = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveGuzheng {
                    player: PlayerId::new(player),
                    event_player: PlayerId::new(event_player),
                    stage: 0,
                    cards: Vec::new(),
                    return_card: engine_api::CardId::NONE,
                    take_rest: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHIBA => {
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhiba {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHIBA_SETTLE => {
                let target_card = pop(frame)?.0.max(0) as u32;
                let player_card = pop(frame)?.0.max(0) as u32;
                let winner = pop(frame)?.0.max(0) as u32;
                let actor = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhibaSettle {
                    player: PlayerId::new(player),
                    actor: PlayerId::new(actor),
                    winner: PlayerId::new(winner),
                    player_card: engine_api::CardId::new(player_card),
                    target_card: engine_api::CardId::new(target_card),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_CONGJIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltCongjian {
                    player: PlayerId::new(player),
                    targets: Vec::new(),
                    card: engine_api::CardId::NONE,
                    target: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_YONGSI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltYongsi {
                    player: PlayerId::new(player),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DRLT_WEIDI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDrltWeidi {
                    player: PlayerId::new(player),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LIEREN => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLieren {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NZRY_FEIJUN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNzryFeijun {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    branch: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SET_SKILL_CONTENT_FLAG => {
                let value = pop(frame)?.0 != 0;
                let bit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SetSkillContentFlag {
                    player: PlayerId::new(player),
                    bit,
                    value,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWMOUKUI => {
                let card = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwmoukui {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWFUMAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let context = host.context();
                let cards = match action {
                    0 => host.selected_cards().to_vec(),
                    1 => (context.event_card != 0)
                        .then_some(context.event_card)
                        .into_iter()
                        .collect(),
                    _ => host.event_cards().to_vec(),
                };
                EffectRequest::ResolveTwfuman {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    action,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_CUIJIN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveCuijin {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWQINGKOU => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let winner = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwqingkou {
                    player: PlayerId::new(player),
                    winner: PlayerId::new(winner),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWCHENXUN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwchenxun {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TWFENXIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTwfenxian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TIANXIANG => {
                let draw_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveTianxiang {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    discard_card: engine_api::CardId::NONE,
                    source: PlayerId::NONE,
                    amount: 0,
                    nature: 0,
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    card_flags: 0,
                    draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::SUPPRESS_NON_LOCKED_SKILLS_UNTIL_TURN_END => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::SuppressNonLockedSkillsUntilTurnEnd {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::CHOOSE_RECOVER_TARGET => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let chooser = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseRecoverTarget {
                    chooser: PlayerId::new(chooser),
                    target: PlayerId::new(target),
                    amount,
                }
            }
            effect_tag::GIVE_ALL_HAND_THEN_CHOOSE_USE_ONE_OR_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: 0,
                    card: rule_api::CardId::NONE,
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_SHA_UNRESPONDABLE => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    targets: Vec::new(),
                    stage: 0,
                    damage_bonus: 0,
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::ADD_BOUND_SKILL => {
                let target = pop(frame)?.0.max(0) as u32;
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddBoundSkill {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                    bound_target: PlayerId::new(target),
                }
            }
            effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE => {
                let target = pop(frame)?.0.max(0) as u32;
                let skill_def = pop(frame)?.0 as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                    bound_target: PlayerId::new(target),
                    ready_to_replace: false,
                }
            }
            effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                    zone: 0,
                    discarded_count: 0,
                    ready_to_draw: false,
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER => {
                let amount = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandEquipThenRecover {
                    player: PlayerId::new(player),
                    count,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDANXU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldAnxu {
                    player: PlayerId::new(player),
                    stage: 0,
                    giver: PlayerId::NONE,
                    receiver: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::STORE_SELECTED_IN_SKILL_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::StoreSelectedInSkillPile {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::END_MATCH_FOR_CAMP_IF_SKILL_PILE_AT_LEAST => {
                let count = pop(frame)?.0.max(0) as u32;
                let pile_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::EndMatchForCampIfSkillPileAtLeast {
                    player: PlayerId::new(player),
                    pile_skill_def: engine_api::SkillDefId::new(pile_skill_def),
                    count,
                }
            }
            effect_tag::RESOLVE_OLDQINQING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldQinqing {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    lord: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDMINGJIAN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldMingjian {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDXUANFENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldXuanfeng {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    branch: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIANROU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJianrou {
                    player: PlayerId::new(player),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDHUISHENG => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldHuisheng {
                    player: PlayerId::new(player),
                    source: PlayerId::NONE,
                    damage_id: 0,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_OLDJIEFAN => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveOldJiefan {
                    player: PlayerId::new(player),
                    dying: PlayerId::NONE,
                    current: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    damage_id: 0,
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_CHEGU => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveChegu {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::NONE,
                    targets: Vec::new(),
                    cards: Vec::new(),
                    branch: 0,
                    index: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JUNKGUIXIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJunkguixin {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    stage: 0,
                    group: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YAO_CHENWEI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYaoChenwei {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    mode: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REFRESH_SKILL => {
                let skill = pop(frame)?.0.max(0) as u32;
                EffectRequest::RefreshSkill {
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::TAG_CARD => {
                let skill = pop(frame)?.0.max(0) as u32;
                let cards: Vec<engine_api::CardId> = host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect();
                EffectRequest::TagCard { cards, tag: skill }
            }
            effect_tag::UNTAG_CARD => {
                let skill = pop(frame)?.0.max(0) as u32;
                let cards: Vec<engine_api::CardId> = host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect();
                EffectRequest::UntagCard { cards, tag: skill }
            }
            effect_tag::CLANSHENJUN_REVEAL => {
                let skill = pop(frame)?.0.max(0) as u32;
                let event_card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ClanshenjunReveal {
                    player: engine_api::PlayerId::new(player),
                    event_card: engine_api::CardId::new(event_card),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CLANSHENJUN_USE => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ClanshenjunUse {
                    player: engine_api::PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CLANYUZHI_REVEAL => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ClanyuzhiReveal {
                    player: engine_api::PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CLANYUZHI_ROUND_END => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ClanyuzhiRoundEnd {
                    player: engine_api::PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::QIEYI_PHASE_START => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QieyiPhaseStart {
                    player: engine_api::PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::QIEYI_USE_AFTER => {
                let skill = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QieyiUseAfter {
                    player: engine_api::PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::JIANZHI_PHASE_END => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JianzhiPhaseEnd {
                    player: engine_api::PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::STORAGE_SET => {
                let value = pop(frame)?.0;
                let key = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                EffectRequest::StorageSet {
                    skill: engine_api::SkillId::new(skill),
                    key,
                    value: value.max(0) as u32,
                }
            }
            effect_tag::DRAW_ALL_EVENT_TARGETS => {
                let count = pop(frame)?.0.max(0) as u32;
                // Event player + declared event targets, de-duplicated, in declaration order.
                let mut players = vec![PlayerId::new(host.context().event_player)];
                for target in host.event_targets() {
                    let player = PlayerId::new(*target);
                    if player != PlayerId::NONE && !players.contains(&player) {
                        players.push(player);
                    }
                }
                EffectRequest::DrawAllEventTargets { players, count }
            }
            _ => {
                return Err(VmError::Host(format!("unknown effect tag {tag}")));
            }
        };
        host.emit_effect(req).map_err(VmError::from)
    }

    pub(crate) fn host_query(
        host: &mut dyn VmHost,
        frame: &mut Frame,
        tag: u8,
    ) -> Result<i32, VmError> {
        if !host.capabilities().run_queries {
            return Err(VmError::CapabilityDenied("run_queries".into()));
        }
        let req = match tag {
            query_tag::DISTANCE => {
                let to = pop(frame)?.0.max(0) as u32;
                let from = pop(frame)?.0.max(0) as u32;
                QueryRequest::Distance {
                    from: PlayerId::new(from),
                    to: PlayerId::new(to),
                }
            }
            query_tag::ATTACK_RANGE => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::AttackRange {
                    player: PlayerId::new(player),
                }
            }
            query_tag::HAND_LIMIT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HandLimit {
                    player: PlayerId::new(player),
                }
            }
            query_tag::HP => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::Hp {
                    player: PlayerId::new(player),
                }
            }
            query_tag::HAND_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HandCount {
                    player: PlayerId::new(player),
                }
            }
            query_tag::EQUIPPED_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::EquippedCount {
                    player: PlayerId::new(player),
                }
            }
            query_tag::EMPTY_EQUIP_SLOTS => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::EmptyEquipSlots {
                    player: PlayerId::new(player),
                }
            }
            query_tag::MAX_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::MaxHp {
                    player: PlayerId::new(player),
                }
            }
            query_tag::RELATION => {
                let b = pop(frame)?.0.max(0) as u32;
                let a = pop(frame)?.0.max(0) as u32;
                QueryRequest::Relation {
                    a: PlayerId::new(a),
                    b: PlayerId::new(b),
                }
            }
            query_tag::CARD_USABLE => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardUsable {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                }
            }
            query_tag::TARGET_LEGAL => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let user = pop(frame)?.0.max(0) as u32;
                QueryRequest::TargetLegal {
                    user: PlayerId::new(user),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            query_tag::CARD_ATTRIBUTE => {
                let attr = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardAttribute {
                    card: rule_api::CardId::new(card),
                    attr,
                }
            }
            query_tag::SKILL_USES_REMAINING => {
                let skill = pop(frame)?.0.max(0) as u32;
                QueryRequest::SkillUsesRemaining {
                    skill: rule_api::SkillId::new(skill),
                }
            }
            query_tag::ZONE => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::Zone {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            query_tag::CARD_ZONE => {
                let card = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardZone {
                    card: rule_api::CardId::new(card),
                }
            }
            query_tag::MARK_COUNT => {
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::MarkCount {
                    player: PlayerId::new(player),
                    mark,
                }
            }
            query_tag::SKILL_PILE_COUNT => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::SkillPileCount {
                    player: PlayerId::new(player),
                    skill_def,
                }
            }
            query_tag::CARDS_DRAWN_THIS_TURN => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardsDrawnThisTurn {
                    player: PlayerId::new(player),
                }
            }
            query_tag::DAMAGE_DEALT_THIS_TURN => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::DamageDealtThisTurn {
                    player: PlayerId::new(player),
                }
            }
            query_tag::HAS_SKILL => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HasSkill {
                    player: PlayerId::new(player),
                    skill_def: engine_api::SkillDefId::new(skill_def),
                }
            }
            query_tag::CARD_NAME_LENGTH => {
                let card = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardNameLength {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                }
            }
            query_tag::IN_PLAY_PHASE => QueryRequest::InPlayPhase,
            query_tag::CARD_HAS_TAG => {
                let tag = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0 as u32;
                QueryRequest::CardHasTag {
                    card: engine_api::CardId::new(card),
                    tag,
                }
            }
            query_tag::STORAGE_GET => {
                let key = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                QueryRequest::StorageGet {
                    skill: engine_api::SkillId::new(skill),
                    key,
                }
            }
            query_tag::STORAGE_HAS => {
                let key = pop(frame)?.0.max(0) as u32;
                let skill = pop(frame)?.0.max(0) as u32;
                QueryRequest::StorageHas {
                    skill: engine_api::SkillId::new(skill),
                    key,
                }
            }
            query_tag::CARDS_USED_THIS_ROUND => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardsUsedThisRound {
                    player: PlayerId::new(player),
                }
            }
            query_tag::ALIVE_COUNT => QueryRequest::AliveCount,
            query_tag::CARD_ACTION_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::CardActionCount {
                    player: PlayerId::new(player),
                }
            }
            query_tag::ALIVE_GROUP_COUNT => QueryRequest::AliveGroupCount,
            query_tag::UNIQUE_MAX_CURRENT_HP => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::UniqueMaxCurrentHp {
                    player: PlayerId::new(player),
                }
            }
            query_tag::UNIQUE_MAX_CURRENT_HP_AT => {
                let value = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::UniqueMaxCurrentHpAt {
                    player: PlayerId::new(player),
                    value,
                }
            }
            query_tag::LIFETIME_DAMAGE_DEALT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::LifetimeDamageDealt {
                    player: PlayerId::new(player),
                }
            }
            query_tag::PUBLIC_CARD_KIND_COUNT => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                QueryRequest::PublicCardKindCount { kind }
            }
            query_tag::ROUND_NUMBER_CAPPED => {
                let cap = pop(frame)?.0.max(0) as u32;
                QueryRequest::RoundNumberCapped { cap }
            }
            query_tag::FIELD_EQUIPMENT_KIND_COUNT => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                QueryRequest::FieldEquipmentKindCount { kind }
            }
            query_tag::ALIVE_WOUNDED_OR_GUIMING_WU_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::AliveWoundedOrGuimingWuCount {
                    player: PlayerId::new(player),
                }
            }
            query_tag::HAND_EQUIP_SUIT_COUNT => {
                let suit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HandEquipSuitCount {
                    player: PlayerId::new(player),
                    suit,
                }
            }
            query_tag::CURRENT_PHASE_PLAYER => QueryRequest::CurrentPhasePlayer,
            query_tag::HAND_SUIT_COUNT => {
                let suit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HandSuitCount {
                    player: PlayerId::new(player),
                    suit,
                }
            }
            query_tag::HAND_COLOR_COUNT => {
                let color = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::HandColorCount {
                    player: PlayerId::new(player),
                    color,
                }
            }
            query_tag::SKILL_CONTENT_FLAG => {
                let bit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                QueryRequest::SkillContentFlag {
                    player: PlayerId::new(player),
                    skill_def: engine_api::SkillDefId::new(skill_def),
                    bit,
                }
            }
            _ => return Err(VmError::Host(format!("unknown query tag {tag}"))),
        };
        match host.run_query(req).map_err(VmError::from)? {
            QueryResponse::Int(v) => Ok(v),
            QueryResponse::Bool(b) => Ok(i32::from(b)),
            QueryResponse::Relation(rel) => Ok(match rel {
                rule_api::RelationTag::Self_ => relation_code::SELF,
                rule_api::RelationTag::Ally => relation_code::ALLY,
                rule_api::RelationTag::Enemy => relation_code::ENEMY,
                rule_api::RelationTag::Unknown => relation_code::UNKNOWN,
            }),
            QueryResponse::SkillPileCount { count } => Ok(count as i32),
            QueryResponse::Error { code } => Err(VmError::Host(format!("query error {code:?}"))),
        }
    }
}

fn inclusive_choice_range(min: i32, max: i32) -> Result<Vec<u32>, VmError> {
    if min < 0 || max < min {
        return Err(VmError::Arithmetic("invalid choice range".into()));
    }
    let len = i64::from(max) - i64::from(min) + 1;
    if len > 255 {
        return Err(VmError::MemoryLimit);
    }
    Ok((min..=max).map(|value| value as u32).collect())
}

fn push(frame: &mut Frame, v: Value, max: usize) -> Result<(), VmError> {
    if frame.stack.len() >= max {
        return Err(VmError::StackOverflow);
    }
    frame.stack.push(v);
    Ok(())
}

fn pop(frame: &mut Frame) -> Result<Value, VmError> {
    frame
        .stack
        .pop()
        .ok_or_else(|| VmError::Stack("stack underflow".into()))
}

fn binop(frame: &mut Frame, f: impl FnOnce(i32, i32) -> i32) -> Result<(), VmError> {
    let b = pop(frame)?;
    let a = pop(frame)?;
    frame.stack.push(Value(f(a.0, b.0)));
    Ok(())
}

fn read_u8(code: &[u8], pc: &mut u32) -> Result<u8, VmError> {
    let i = *pc as usize;
    if i >= code.len() {
        return Err(VmError::BadControlFlow("read_u8".into()));
    }
    *pc += 1;
    Ok(code[i])
}

fn read_u16(code: &[u8], pc: &mut u32) -> Result<u16, VmError> {
    let p = *pc as usize;
    if p + 2 > code.len() {
        return Err(VmError::InvalidBytecode("truncated u16".into()));
    }
    *pc += 2;
    Ok(u16::from_le_bytes([code[p], code[p + 1]]))
}

fn read_i16(code: &[u8], pc: &mut u32) -> Result<i16, VmError> {
    let i = *pc as usize;
    if i + 2 > code.len() {
        return Err(VmError::BadControlFlow("read_i16".into()));
    }
    let v = i16::from_le_bytes([code[i], code[i + 1]]);
    *pc += 2;
    Ok(v)
}

fn read_i32(code: &[u8], pc: &mut u32) -> Result<i32, VmError> {
    let i = *pc as usize;
    if i + 4 > code.len() {
        return Err(VmError::BadControlFlow("read_i32".into()));
    }
    let v = i32::from_le_bytes([code[i], code[i + 1], code[i + 2], code[i + 3]]);
    *pc += 4;
    Ok(v)
}

fn read_u32(code: &[u8], pc: &mut u32) -> Result<u32, VmError> {
    Ok(read_i32(code, pc)? as u32)
}

pub(crate) fn encode_frame(frame: &Frame) -> Vec<u8> {
    // Simple deterministic pack: local_count u16 | locals i32* | stack_count u16 | stack i32* | pc already separate
    let mut out = Vec::new();
    out.extend_from_slice(&(frame.locals.len() as u16).to_le_bytes());
    for v in &frame.locals {
        out.extend_from_slice(&v.0.to_le_bytes());
    }
    out.extend_from_slice(&(frame.stack.len() as u16).to_le_bytes());
    for v in &frame.stack {
        out.extend_from_slice(&v.0.to_le_bytes());
    }
    out
}

pub(crate) fn decode_frame(data: &[u8]) -> Result<Frame, VmError> {
    if data.len() < 4 {
        return Ok(Frame {
            locals: vec![],
            stack: vec![],
            pc: 0,
        });
    }
    let mut i = 0usize;
    let nloc = u16::from_le_bytes([data[i], data[i + 1]]) as usize;
    i += 2;
    let mut locals = Vec::with_capacity(nloc);
    for _ in 0..nloc {
        if i + 4 > data.len() {
            return Err(VmError::InvalidBytecode("frame locals".into()));
        }
        locals.push(Value(i32::from_le_bytes(
            data[i..i + 4].try_into().unwrap(),
        )));
        i += 4;
    }
    if i + 2 > data.len() {
        return Err(VmError::InvalidBytecode("frame stack count".into()));
    }
    let nst = u16::from_le_bytes([data[i], data[i + 1]]) as usize;
    i += 2;
    let mut stack = Vec::with_capacity(nst);
    for _ in 0..nst {
        if i + 4 > data.len() {
            return Err(VmError::InvalidBytecode("frame stack".into()));
        }
        stack.push(Value(i32::from_le_bytes(
            data[i..i + 4].try_into().unwrap(),
        )));
        i += 4;
    }
    Ok(Frame {
        locals,
        stack,
        pc: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{assemble, encode_caps, Op};
    use crate::host::{CollectingHost, HostContext};
    use rule_api::CapabilitySet;

    fn mod_from_ops(name: &str, ops: &[Op], locals: u8, caps: u32) -> BytecodeModule {
        let code = assemble(ops);
        let logic_hash = BytecodeModule::compute_logic_hash(name, &code, locals, caps, 0);
        BytecodeModule {
            name: name.into(),
            code,
            local_count: locals,
            max_stack: 16,
            entry_pc: 0,
            logic_hash,
            caps_mask: caps,
        }
    }

    #[test]
    fn add_and_halt() {
        let m = mod_from_ops(
            "add",
            &[Op::ConstI32(2), Op::ConstI32(3), Op::Add, Op::Halt],
            0,
            0,
        );
        let mut vm = RuleVm::new();
        let mut host = CollectingHost::default();
        let r = vm.execute(&m, &mut host, 100).unwrap();
        assert!(matches!(r, ExecResult::Halted { .. }));
    }

    #[test]
    fn min_picks_lower() {
        // min(7, 3) → 3; also min(-2, 5) → -2.
        let m = mod_from_ops(
            "min1",
            &[Op::ConstI32(7), Op::ConstI32(3), Op::Min, Op::Halt],
            0,
            0,
        );
        let mut vm = RuleVm::new();
        let mut host = CollectingHost::default();
        assert!(matches!(
            vm.execute(&m, &mut host, 100).unwrap(),
            ExecResult::Halted { .. }
        ));
        let m = mod_from_ops(
            "min2",
            &[Op::ConstI32(-2), Op::ConstI32(5), Op::Min, Op::Halt],
            0,
            0,
        );
        let mut vm = RuleVm::new();
        let mut host = CollectingHost::default();
        assert!(matches!(
            vm.execute(&m, &mut host, 100).unwrap(),
            ExecResult::Halted { .. }
        ));
    }

    #[test]
    fn infinite_loop_hits_fuel() {
        // JUMP -3 after reading jump offset: pc after JUMP opcode+imm points back
        // Const; Jump(-5) looping — simpler: Jump(0) infinite at same region
        let m = mod_from_ops("loop", &[Op::Jump(-3)], 0, 0);
        // Jump(-3): after reading opcode at 0, pc=1, read i16, pc=3, then pc = 3 + (-3) = 0
        let mut vm = RuleVm::new();
        let mut host = CollectingHost::default();
        let err = vm.execute(&m, &mut host, 50).unwrap_err();
        assert_eq!(err, VmError::OutOfFuel);
    }

    #[test]
    fn draw_effect_via_host() {
        use crate::bytecode::{caps, effect_tag};
        let m = mod_from_ops(
            "draw",
            &[
                Op::HostSelf,
                Op::ConstI32(2),
                Op::HostEffect(effect_tag::DRAW_CARDS),
                Op::Halt,
            ],
            0,
            caps::EMIT_EFFECTS,
        );
        let mut vm = RuleVm::new();
        let mut host = CollectingHost {
            ctx: HostContext {
                self_id: 7,
                ..Default::default()
            },
            caps: CapabilitySet::standard_skill(),
            ..Default::default()
        };
        vm.execute(&m, &mut host, 1000).unwrap();
        assert_eq!(host.effects.len(), 1);
        assert!(matches!(
            &host.effects[0],
            EffectRequest::DrawCards {
                player,
                count: 2
            } if player.raw() == 7
        ));
    }

    #[test]
    fn capability_denied() {
        use crate::bytecode::{caps, effect_tag};
        let m = mod_from_ops(
            "nope",
            &[
                Op::HostSelf,
                Op::ConstI32(1),
                Op::HostEffect(effect_tag::DRAW_CARDS),
                Op::Halt,
            ],
            0,
            caps::EMIT_EFFECTS,
        );
        let mut vm = RuleVm::new();
        let mut host = CollectingHost {
            caps: CapabilitySet::none(),
            ..Default::default()
        };
        let err = vm.execute(&m, &mut host, 100).unwrap_err();
        assert!(matches!(err, VmError::CapabilityDenied(_)));
        let _ = encode_caps;
    }
}
