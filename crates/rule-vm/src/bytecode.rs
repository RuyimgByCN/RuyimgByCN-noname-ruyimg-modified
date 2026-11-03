//! Deterministic bytecode module format and opcodes.

use serde::{Deserialize, Serialize};

/// Magic ASCII `NNVM`.
pub const MODULE_MAGIC: [u8; 4] = *b"NNVM";
/// Module format version.
pub const MODULE_VERSION: u16 = 1;

/// Opcode byte values (stable; never reuse numbers).
#[allow(non_snake_case)]
pub mod op {
    /// No-op.
    pub const NOP: u8 = 0x00;
    /// Halt successfully.
    pub const HALT: u8 = 0x01;
    /// Push i32 immediate (le bytes).
    pub const CONST_I32: u8 = 0x10;
    /// Push u32 immediate.
    pub const CONST_U32: u8 = 0x11;
    /// Push true (1) / false (0) via CONST_I32 preferred; BOOL kept for clarity.
    pub const CONST_BOOL: u8 = 0x12;
    /// Load local slot u8.
    pub const LOAD: u8 = 0x20;
    /// Store local slot u8.
    pub const STORE: u8 = 0x21;
    /// Duplicate top.
    pub const DUP: u8 = 0x22;
    /// Pop top.
    pub const POP: u8 = 0x23;
    /// Add two i32.
    pub const ADD: u8 = 0x30;
    /// Sub.
    pub const SUB: u8 = 0x31;
    /// Mul.
    pub const MUL: u8 = 0x32;
    /// Eq → bool as i32.
    pub const EQ: u8 = 0x33;
    /// Lt.
    pub const LT: u8 = 0x34;
    /// Logical not on i32 (0→1, else 0).
    pub const NOT: u8 = 0x35;
    /// Element-wise minimum of two i32.
    pub const MIN: u8 = 0x36;
    /// Greater-than (Phase 3 DSL expression extension).
    pub const GT: u8 = 0x37;
    /// Logical and (Phase 3 DSL expression extension).
    pub const AND: u8 = 0x38;
    /// Unconditional jump i16 relative from after operand.
    pub const JUMP: u8 = 0x40;
    /// Jump if top is 0; pops.
    pub const JUMP_IF_FALSE: u8 = 0x41;
    /// Host: emit effect. operands: effect_tag u8, then stack args per tag.
    pub const HOST_EFFECT: u8 = 0x50;
    /// Host: run query; pushes result i32 (bool as 0/1).
    pub const HOST_QUERY: u8 = 0x51;
    /// Host: request choice (pauses).
    pub const HOST_CHOICE: u8 = 0x52;
    /// Push skill owner player id from host context.
    pub const HOST_SELF: u8 = 0x53;
    /// Push event player id from host context.
    pub const HOST_EVENT_PLAYER: u8 = 0x54;
    /// Push event source player id from host context.
    pub const HOST_EVENT_SOURCE: u8 = 0x55;
    /// Push event card id from host context (0 if none).
    pub const HOST_EVENT_CARD: u8 = 0x56;
    /// Push current skill instance id from host context.
    pub const HOST_SKILL: u8 = 0x57;
    /// Push damage amount from host context (0 outside damage events).
    pub const HOST_EVENT_AMOUNT: u8 = 0x58;
    /// Host: request one choice from an inclusive integer range (pauses).
    pub const HOST_CHOICE_RANGE: u8 = 0x59;
    /// Host: emit an effect with a little-endian u16 tag.
    pub const HOST_EFFECT_EXT: u8 = 0x5A;
    /// Host: push the count of player targets captured from the triggering event (Phase 3).
    pub const HOST_EVENT_TARGET_COUNT: u8 = 0x5B;
    /// Host: push the count of physical cards captured from the triggering event (Phase 3).
    pub const HOST_EVENT_CARD_COUNT: u8 = 0x5C;
    /// Trap intentionally (content fault).
    pub const TRAP: u8 = 0x7F;
}

/// Effect tags for HOST_EFFECT (stable).
pub mod effect_tag;

/// Query tags for HOST_QUERY.
pub mod query_tag {
    /// distance(from, to) → i32
    pub const DISTANCE: u8 = 1;
    /// attack_range(player) → i32
    pub const ATTACK_RANGE: u8 = 2;
    /// hand_limit(player) → i32
    pub const HAND_LIMIT: u8 = 3;
    /// hp(player) → i32
    pub const HP: u8 = 4;
    /// hand_count(player) → i32
    pub const HAND_COUNT: u8 = 5;
    /// max_hp(player) → i32
    pub const MAX_HP: u8 = 6;
    /// relation(a, b) → i32 (0=self, 1=ally, 2=enemy, 3=unknown)
    pub const RELATION: u8 = 7;
    /// card_usable(player, card) → i32 (0/1)
    pub const CARD_USABLE: u8 = 8;
    /// target_legal(user, card, target) → i32 (0/1)
    pub const TARGET_LEGAL: u8 = 9;
    /// card_attribute(card, attr) → i32
    pub const CARD_ATTRIBUTE: u8 = 10;
    /// skill_uses_remaining(skill_instance) → i32
    pub const SKILL_USES_REMAINING: u8 = 11;
    /// zone(player, kind_tag) → zone id
    pub const ZONE: u8 = 12;
    /// card_zone(card) → zone id
    pub const CARD_ZONE: u8 = 13;
    /// mark_count(player, mark_def) → i32
    pub const MARK_COUNT: u8 = 14;
    /// skill_pile_count(player, skill_def) → i32 (ABI 565)
    pub const SKILL_PILE_COUNT: u8 = 31;
    /// cards_drawn_this_turn(player) → i32 — Phase 3 history-count family.
    pub const CARDS_DRAWN_THIS_TURN: u8 = 32;
    /// damage_dealt_this_turn(player) → i32 — Phase 3 history-count family.
    pub const DAMAGE_DEALT_THIS_TURN: u8 = 33;
    /// has_skill(player, skill_def) → bool — Phase 3 hasSkill-check family.
    pub const HAS_SKILL: u8 = 34;
    /// card_name_length(player, card) → i32 — Phase 3 card-name-length family.
    pub const CARD_NAME_LENGTH: u8 = 35;
    /// in_play_phase() → bool — Phase 3 phase-parent family.
    pub const IN_PLAY_PHASE: u8 = 36;
    /// card_has_tag(card, skill_instance) → bool — gaintag membership (skill def id is the tag).
    pub const CARD_HAS_TAG: u8 = 37;
    /// storage_get(skill_instance, key) → i32 — keyed content storage value (0 if absent).
    pub const STORAGE_GET: u8 = 38;
    /// storage_has(skill_instance, key) → bool — keyed content storage membership.
    pub const STORAGE_HAS: u8 = 39;
    /// cards_used_this_round(player) → i32 — proactive cards used this round.
    pub const CARDS_USED_THIS_ROUND: u8 = 40;
    /// Number of cards in the player's equipment zone — ABI 681.
    pub const EQUIPPED_COUNT: u8 = 41;

    /// empty_equip_slots(player) → i32
    pub const EMPTY_EQUIP_SLOTS: u8 = 15;
    /// alive_count() → i32
    pub const ALIVE_COUNT: u8 = 16;
    /// card_action_count(player) → i32
    pub const CARD_ACTION_COUNT: u8 = 17;
    /// alive_group_count() → i32
    pub const ALIVE_GROUP_COUNT: u8 = 18;
    /// unique_max_current_hp(player) → i32 (0/1)
    pub const UNIQUE_MAX_CURRENT_HP: u8 = 19;
    /// unique_max_current_hp_at(player, value) → i32 (0/1)
    pub const UNIQUE_MAX_CURRENT_HP_AT: u8 = 20;
    /// lifetime_damage_dealt(player) → i32
    pub const LIFETIME_DAMAGE_DEALT: u8 = 21;
    /// public_card_kind_count(kind) → i32
    pub const PUBLIC_CARD_KIND_COUNT: u8 = 22;
    /// round_number_capped(cap) → i32
    pub const ROUND_NUMBER_CAPPED: u8 = 23;
    /// field_equipment_kind_count(kind) → i32
    pub const FIELD_EQUIPMENT_KIND_COUNT: u8 = 24;
    /// alive_wounded_or_guiming_wu_count(player) → i32
    pub const ALIVE_WOUNDED_OR_GUIMING_WU_COUNT: u8 = 25;
    /// hand_equip_suit_count(player, suit) → i32
    pub const HAND_EQUIP_SUIT_COUNT: u8 = 26;
    /// current_phase_player() → i32 (current turn player raw id)
    pub const CURRENT_PHASE_PLAYER: u8 = 27;
    /// hand_suit_count(player, suit) → i32
    pub const HAND_SUIT_COUNT: u8 = 28;
    /// hand_color_count(player, color) → i32
    pub const HAND_COLOR_COUNT: u8 = 29;
    /// skill_content_flag(player, skill_def, bit) → i32
    pub const SKILL_CONTENT_FLAG: u8 = 30;
}

/// Relation tag → i32 encoding for `query_tag::RELATION` VM results (stable).
pub mod relation_code {
    /// Same player.
    pub const SELF: i32 = 0;
    /// Same camp.
    pub const ALLY: i32 = 1;
    /// Enemy camp.
    pub const ENEMY: i32 = 2;
    /// Hidden / unknown identity.
    pub const UNKNOWN: i32 = 3;
}

/// High-level opcode for assembler / disassembler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Nop.
    Nop,
    /// Halt.
    Halt,
    /// Const i32.
    ConstI32(i32),
    /// Const u32 (stored as i32 bit pattern on stack).
    ConstU32(u32),
    /// Load local.
    Load(u8),
    /// Store local.
    Store(u8),
    /// Dup.
    Dup,
    /// Pop.
    Pop,
    /// Add.
    Add,
    /// Sub.
    Sub,
    /// Mul.
    Mul,
    /// Eq.
    Eq,
    /// Lt.
    Lt,
    /// Gt (greater-than; Phase 3 DSL expression extension).
    Gt,
    /// Logical and (both operands truthy; Phase 3 DSL expression extension).
    And,
    /// Not.
    Not,
    /// Element-wise minimum of two i32.
    Min,
    /// Jump relative.
    Jump(i16),
    /// Jump if false.
    JumpIfFalse(i16),
    /// Host effect.
    HostEffect(u16),
    /// Host query.
    HostQuery(u8),
    /// Host choice: pops `count` option tokens, pauses; on resume pushes the
    /// chosen token. Operand: option count u8.
    HostChoice(u8),
    /// Host choice over an inclusive non-negative integer range: pops max then
    /// min, pauses, and pushes the chosen token on resume.
    HostChoiceRange,
    /// Self id.
    HostSelf,
    /// Event player.
    HostEventPlayer,
    /// Event source.
    HostEventSource,
    /// Event card id.
    HostEventCard,
    /// Current skill instance id.
    HostSkill,
    /// Damage amount bound to the current event.
    HostEventAmount,
    /// Count of player targets captured from the triggering event (Phase 3).
    HostEventTargetCount,
    /// Count of physical cards captured from the triggering event (Phase 3).
    HostEventCardCount,
    /// Trap.
    Trap,
}

/// Verified (or to-verify) bytecode module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BytecodeModule {
    /// Logical module name.
    pub name: String,
    /// Raw code section (opcodes + immediates).
    pub code: Vec<u8>,
    /// Number of local slots required.
    pub local_count: u8,
    /// Max stack depth claimed (verifier checks).
    pub max_stack: u8,
    /// Entry PC (usually 0).
    pub entry_pc: u32,
    /// Content logic hash of this module (semantic).
    pub logic_hash: [u8; 32],
    /// Required capabilities bitmask (see encode_caps).
    pub caps_mask: u32,
}

impl BytecodeModule {
    /// Encode to portable bytes: magic | version | header | code.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&MODULE_MAGIC);
        out.extend_from_slice(&MODULE_VERSION.to_le_bytes());
        let name_bytes = self.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.push(self.local_count);
        out.push(self.max_stack);
        out.extend_from_slice(&self.entry_pc.to_le_bytes());
        out.extend_from_slice(&self.caps_mask.to_le_bytes());
        out.extend_from_slice(&self.logic_hash);
        out.extend_from_slice(&(self.code.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.code);
        out
    }

    /// Decode module bytes (does not verify control flow).
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut i = 0usize;
        let read = |i: &mut usize, n: usize| -> Result<&[u8], String> {
            if *i + n > data.len() {
                return Err("truncated module".into());
            }
            let s = &data[*i..*i + n];
            *i += n;
            Ok(s)
        };
        if read(&mut i, 4)? != MODULE_MAGIC {
            return Err("bad magic".into());
        }
        let ver = u16::from_le_bytes(read(&mut i, 2)?.try_into().unwrap());
        if ver != MODULE_VERSION {
            return Err(format!("unsupported module version {ver}"));
        }
        let name_len = u32::from_le_bytes(read(&mut i, 4)?.try_into().unwrap()) as usize;
        let name = String::from_utf8(read(&mut i, name_len)?.to_vec())
            .map_err(|_| "name utf8".to_string())?;
        let local_count = read(&mut i, 1)?[0];
        let max_stack = read(&mut i, 1)?[0];
        let entry_pc = u32::from_le_bytes(read(&mut i, 4)?.try_into().unwrap());
        let caps_mask = u32::from_le_bytes(read(&mut i, 4)?.try_into().unwrap());
        let mut logic_hash = [0u8; 32];
        logic_hash.copy_from_slice(read(&mut i, 32)?);
        let code_len = u32::from_le_bytes(read(&mut i, 4)?.try_into().unwrap()) as usize;
        let code = read(&mut i, code_len)?.to_vec();
        Ok(Self {
            name,
            code,
            local_count,
            max_stack,
            entry_pc,
            logic_hash,
            caps_mask,
        })
    }

    /// Recompute logic hash from semantic fields (name, code, locals, caps, entry).
    pub fn compute_logic_hash(
        name: &str,
        code: &[u8],
        local_count: u8,
        caps_mask: u32,
        entry_pc: u32,
    ) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/bytecode-module/v1");
        h.update(&(name.len() as u32).to_le_bytes());
        h.update(name.as_bytes());
        h.update(&[local_count]);
        h.update(&caps_mask.to_le_bytes());
        h.update(&entry_pc.to_le_bytes());
        h.update(&(code.len() as u32).to_le_bytes());
        h.update(code);
        *h.finalize().as_bytes()
    }
}

/// Assemble ops into code bytes.
pub fn assemble(ops: &[Op]) -> Vec<u8> {
    let mut code = Vec::new();
    for op in ops {
        match op {
            Op::Nop => code.push(op::NOP),
            Op::Halt => code.push(op::HALT),
            Op::ConstI32(v) => {
                code.push(op::CONST_I32);
                code.extend_from_slice(&v.to_le_bytes());
            }
            Op::ConstU32(v) => {
                code.push(op::CONST_U32);
                code.extend_from_slice(&v.to_le_bytes());
            }
            Op::Load(s) => {
                code.push(op::LOAD);
                code.push(*s);
            }
            Op::Store(s) => {
                code.push(op::STORE);
                code.push(*s);
            }
            Op::Dup => code.push(op::DUP),
            Op::Pop => code.push(op::POP),
            Op::Add => code.push(op::ADD),
            Op::Sub => code.push(op::SUB),
            Op::Mul => code.push(op::MUL),
            Op::Eq => code.push(op::EQ),
            Op::Lt => code.push(op::LT),
            Op::Gt => code.push(op::GT),
            Op::And => code.push(op::AND),
            Op::Not => code.push(op::NOT),
            Op::Min => code.push(op::MIN),
            Op::Jump(off) => {
                code.push(op::JUMP);
                code.extend_from_slice(&off.to_le_bytes());
            }
            Op::JumpIfFalse(off) => {
                code.push(op::JUMP_IF_FALSE);
                code.extend_from_slice(&off.to_le_bytes());
            }
            Op::HostEffect(tag) => {
                if let Ok(tag) = u8::try_from(*tag) {
                    code.push(op::HOST_EFFECT);
                    code.push(tag);
                } else {
                    code.push(op::HOST_EFFECT_EXT);
                    code.extend_from_slice(&tag.to_le_bytes());
                }
            }
            Op::HostQuery(tag) => {
                code.push(op::HOST_QUERY);
                code.push(*tag);
            }
            Op::HostChoice(n) => {
                code.push(op::HOST_CHOICE);
                code.push(*n);
            }
            Op::HostChoiceRange => code.push(op::HOST_CHOICE_RANGE),
            Op::HostSelf => code.push(op::HOST_SELF),
            Op::HostEventPlayer => code.push(op::HOST_EVENT_PLAYER),
            Op::HostEventSource => code.push(op::HOST_EVENT_SOURCE),
            Op::HostEventCard => code.push(op::HOST_EVENT_CARD),
            Op::HostSkill => code.push(op::HOST_SKILL),
            Op::HostEventAmount => code.push(op::HOST_EVENT_AMOUNT),
            Op::HostEventTargetCount => code.push(op::HOST_EVENT_TARGET_COUNT),
            Op::HostEventCardCount => code.push(op::HOST_EVENT_CARD_COUNT),
            Op::Trap => code.push(op::TRAP),
        }
    }
    code
}

/// Capability bitmask bits.
pub mod caps {
    /// emit_effects
    pub const EMIT_EFFECTS: u32 = 1 << 0;
    /// run_queries
    pub const RUN_QUERIES: u32 = 1 << 1;
    /// request_choice
    pub const REQUEST_CHOICE: u32 = 1 << 2;
    /// 已知能力位的全集——`check_caps` 拒绝任何未知位（t8 拒收③：能力
    /// 越权不能靠「编译器不会发出」侥幸放行；此前该检查对未知位静默
    /// 通过，R05 准备阶段令该测试的触发真实到达后暴露）。
    pub const KNOWN: u32 = EMIT_EFFECTS | RUN_QUERIES | REQUEST_CHOICE;
}

/// Encode CapabilitySet into mask.
pub fn encode_caps(c: &rule_api::CapabilitySet) -> u32 {
    let mut m = 0u32;
    if c.emit_effects {
        m |= caps::EMIT_EFFECTS;
    }
    if c.run_queries {
        m |= caps::RUN_QUERIES;
    }
    if c.request_choice {
        m |= caps::REQUEST_CHOICE;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::{assemble, op, Op};

    #[test]
    fn effect_tags_keep_legacy_bytes_and_extend_past_u8() {
        assert_eq!(
            assemble(&[Op::HostEffect(255), Op::HostEffect(256)]),
            [op::HOST_EFFECT, 255, op::HOST_EFFECT_EXT, 0, 1]
        );
    }
}
