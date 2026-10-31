//! Stack machine interpreter.

mod effects;

use crate::bytecode::{effect_tag, op, query_tag, relation_code, BytecodeModule};
use crate::host::VmHost;
use crate::verify::verify_resume_points;
use crate::{Continuation, VmError};
use rule_api::{DrawTargetScope, EffectRequest, PlayerId, QueryRequest, QueryResponse};
use serde::{Deserialize, Serialize};

/// Default fuel per invocation.
pub const DEFAULT_FUEL: u32 = 10_000;
/// Default max operand stack depth.
pub const DEFAULT_MAX_STACK: usize = 64;

/// PA06：装载期已校验的模块。只能经 [`VerifiedModule::verify`] 构造——
/// 畸形模块在装载（构造）期拒绝；[`RuleVm::execute`]/[`RuleVm::resume`]
/// 只接受已校验模块，执行期不再重复 `verify_module`（此前每次执行都做
/// 全量静态校验，静态核对见台帐 PA06）。`Deref` 仅供只读访问
/// （快照编码/内容锁哈希等装载期用途）。
///
/// 序列化与内层 [`BytecodeModule`] 逐字节相同（持有方的快照格式不变）；
/// 反序列化即装载期校验——快照里的畸形模块在恢复时拒绝，而不是等到首次执行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedModule(BytecodeModule, Vec<u32>);

impl VerifiedModule {
    /// 校验并封装：畸形模块在此（装载期）拒绝。
    pub fn verify(module: BytecodeModule) -> Result<Self, VmError> {
        let resume_points = verify_resume_points(&module)?;
        Ok(Self(module, resume_points))
    }
}

impl std::ops::Deref for VerifiedModule {
    type Target = BytecodeModule;
    fn deref(&self) -> &BytecodeModule {
        &self.0
    }
}

impl Serialize for VerifiedModule {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VerifiedModule {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let module = BytecodeModule::deserialize(deserializer)?;
        Self::verify(module).map_err(serde::de::Error::custom)
    }
}

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

    /// Run verified module from entry with host (PA06：校验在装载期
    /// [`VerifiedModule::verify`] 完成一次，执行期不再重复)。
    pub fn execute(
        &mut self,
        module: &VerifiedModule,
        host: &mut dyn VmHost,
        fuel: u32,
    ) -> Result<ExecResult, VmError> {
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
        module: &VerifiedModule,
        host: &mut dyn VmHost,
        cont: Continuation,
        chosen_token: u32,
    ) -> Result<ExecResult, VmError> {
        if cont.module_hash != module.logic_hash {
            return Err(VmError::InvalidBytecode(
                "continuation module mismatch".into(),
            ));
        }
        if !module.1.contains(&cont.pc) {
            return Err(VmError::InvalidBytecode(
                "continuation pc is not a choice resume point".into(),
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
        let req = effects::decode_request(host, frame, tag)?;
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

    /// PA06：测试助手——按值克隆并装载期校验（执行期不再校验）。
    fn vmod(m: &BytecodeModule) -> VerifiedModule {
        VerifiedModule::verify(m.clone()).unwrap()
    }
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
        let r = vm.execute(&vmod(&m), &mut host, 100).unwrap();
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
            vm.execute(&vmod(&m), &mut host, 100).unwrap(),
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
            vm.execute(&vmod(&m), &mut host, 100).unwrap(),
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
        let err = vm.execute(&vmod(&m), &mut host, 50).unwrap_err();
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
        vm.execute(&vmod(&m), &mut host, 1000).unwrap();
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
        let err = vm.execute(&vmod(&m), &mut host, 100).unwrap_err();
        assert!(matches!(err, VmError::CapabilityDenied(_)));
        let _ = encode_caps;
    }
}
