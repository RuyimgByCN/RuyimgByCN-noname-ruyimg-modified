//! 表达式代码生成。

use super::*;

impl Assembler {
    pub(super) fn emit_expr(&mut self, e: &ExprIr) -> Result<(), CompileError> {
        match e {
            ExprIr::Int(n) => self.ops.push(Op::ConstI32(*n)),
            ExprIr::U32(n) => self.ops.push(Op::ConstU32(*n)),
            ExprIr::SelfPlayer => self.ops.push(Op::HostSelf),
            ExprIr::SelfSkill => self.ops.push(Op::HostSkill),
            ExprIr::EventPlayer => self.ops.push(Op::HostEventPlayer),
            ExprIr::EventSource => self.ops.push(Op::HostEventSource),
            ExprIr::EventCard => self.ops.push(Op::HostEventCard),
            ExprIr::EventAmount => self.ops.push(Op::HostEventAmount),
            // Card-target count: Phase 3 — dedicated opcode reads host.event_targets().len().
            ExprIr::EventTargetCount => self.ops.push(Op::HostEventTargetCount),
            // Event-card count: Phase 3 — reads host.event_cards().len().
            ExprIr::EventCardCount => self.ops.push(Op::HostEventCardCount),
            // Phase-end turn history is event-disjoint from damage amount.
            ExprIr::TurnDamagedPlayerCount => self.ops.push(Op::HostEventAmount),
            ExprIr::Local(name) => {
                let slot = self
                    .locals
                    .iter()
                    .position(|l| l == name)
                    .ok_or_else(|| CompileError::Codegen(format!("local {name}")))?;
                self.ops.push(Op::Load(slot as u8));
            }
            ExprIr::BinOp { op, left, right } => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                match op.as_str() {
                    "+" => self.ops.push(Op::Add),
                    "-" => self.ops.push(Op::Sub),
                    "*" => self.ops.push(Op::Mul),
                    "==" => self.ops.push(Op::Eq),
                    "<" => self.ops.push(Op::Lt),
                    ">" => self.ops.push(Op::Gt),
                    "&&" => self.ops.push(Op::And),
                    _ => {
                        return Err(CompileError::Codegen(format!("bad op {op}")));
                    }
                }
            }
            ExprIr::Query { name, args } => {
                for a in args {
                    self.emit_expr(a)?;
                }
                let tag = match name.as_str() {
                    "distance" => query_tag::DISTANCE,
                    "attack_range" => query_tag::ATTACK_RANGE,
                    "hand_limit" => query_tag::HAND_LIMIT,
                    "hp" => query_tag::HP,
                    "hand_count" => query_tag::HAND_COUNT,
                    "empty_equip_slots" => query_tag::EMPTY_EQUIP_SLOTS,
                    "max_hp" => query_tag::MAX_HP,
                    "relation" => query_tag::RELATION,
                    "card_usable" => query_tag::CARD_USABLE,
                    "target_legal" => query_tag::TARGET_LEGAL,
                    "card_attribute" => query_tag::CARD_ATTRIBUTE,
                    "skill_uses_remaining" => query_tag::SKILL_USES_REMAINING,
                    "zone" => query_tag::ZONE,
                    "equipped_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_681);
                        query_tag::EQUIPPED_COUNT
                    }
                    "card_zone" => query_tag::CARD_ZONE,
                    "mark_count" => query_tag::MARK_COUNT,
                    "skill_pile_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_565);
                        query_tag::SKILL_PILE_COUNT
                    }
                    "alive_count" => query_tag::ALIVE_COUNT,
                    "card_action_count" => query_tag::CARD_ACTION_COUNT,
                    "cards_drawn_this_turn" => query_tag::CARDS_DRAWN_THIS_TURN,
                    "damage_dealt_this_turn" => query_tag::DAMAGE_DEALT_THIS_TURN,
                    "has_skill" => query_tag::HAS_SKILL,
                    "card_name_length" => query_tag::CARD_NAME_LENGTH,
                    "in_play_phase" => query_tag::IN_PLAY_PHASE,
                    "card_has_tag" => query_tag::CARD_HAS_TAG,
                    "storage_get" => query_tag::STORAGE_GET,
                    "storage_has" => query_tag::STORAGE_HAS,
                    "cards_used_this_round" => query_tag::CARDS_USED_THIS_ROUND,
                    "alive_group_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_6);
                        query_tag::ALIVE_GROUP_COUNT
                    }
                    "unique_max_current_hp" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_7);
                        query_tag::UNIQUE_MAX_CURRENT_HP
                    }
                    "unique_max_current_hp_at" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_7);
                        query_tag::UNIQUE_MAX_CURRENT_HP_AT
                    }
                    "lifetime_damage_dealt" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_8);
                        query_tag::LIFETIME_DAMAGE_DEALT
                    }
                    "public_card_kind_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_11);
                        query_tag::PUBLIC_CARD_KIND_COUNT
                    }
                    "field_equipment_kind_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_102);
                        query_tag::FIELD_EQUIPMENT_KIND_COUNT
                    }
                    "alive_wounded_or_guiming_wu_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_190);
                        query_tag::ALIVE_WOUNDED_OR_GUIMING_WU_COUNT
                    }
                    "hand_equip_suit_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_229);
                        query_tag::HAND_EQUIP_SUIT_COUNT
                    }
                    "hand_suit_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::HAND_SUIT_COUNT
                    }
                    "hand_color_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::HAND_COLOR_COUNT
                    }
                    "skill_content_flag" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::SKILL_CONTENT_FLAG
                    }
                    "current_phase_player" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_453);
                        query_tag::CURRENT_PHASE_PLAYER
                    }
                    "round_number_capped" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_75);
                        query_tag::ROUND_NUMBER_CAPPED
                    }
                    _ => return Err(CompileError::Codegen(format!("query {name}"))),
                };
                self.ops.push(Op::HostQuery(tag));
            }
            ExprIr::Choose { options } => {
                for o in options {
                    self.emit_expr(o)?;
                }
                self.ops.push(Op::HostChoice(options.len() as u8));
            }
            ExprIr::ChooseRange { min, max } => {
                self.emit_expr(min)?;
                self.emit_expr(max)?;
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_92);
                self.ops.push(Op::HostChoiceRange);
            }
            ExprIr::Min { left, right } => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_451);
                self.ops.push(Op::Min);
            }
        }
        Ok(())
    }
}
