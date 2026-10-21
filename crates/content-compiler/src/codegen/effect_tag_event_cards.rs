use super::*;

impl Assembler {
    pub(super) fn emit_effect_tag_event_cards(&mut self, s: &StmtIr) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectTagEventCards { skill, span } => {
                self.mark(*span, "tag_event_cards");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::TAG_CARD));
            }
            StmtIr::EffectUntagEventCards { skill, span } => {
                self.mark(*span, "untag_event_cards");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::UNTAG_CARD));
            }
            StmtIr::EffectStorageSet {
                skill,
                key,
                value,
                span,
            } => {
                self.mark(*span, "storage_set");
                self.emit_expr(skill)?;
                self.emit_expr(key)?;
                self.emit_expr(value)?;
                self.ops.push(Op::HostEffect(effect_tag::STORAGE_SET));
            }
            StmtIr::EffectDrawAllEventTargets { count, span } => {
                self.mark(*span, "draw_all_event_targets");
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_ALL_EVENT_TARGETS));
            }
            StmtIr::EffectClanshenjunReveal {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "clanshenjun_reveal");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CLANSHENJUN_REVEAL));
            }
            StmtIr::EffectClanshenjunUse {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "clanshenjun_use");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANSHENJUN_USE));
            }
            StmtIr::EffectClanyuzhiReveal {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "clanyuzhi_reveal");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANYUZHI_REVEAL));
            }
            StmtIr::EffectClanyuzhiRoundEnd {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "clanyuzhi_round_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CLANYUZHI_ROUND_END));
            }
            StmtIr::EffectQieyiPhaseStart {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "qieyi_phase_start");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QIEYI_PHASE_START));
            }
            StmtIr::EffectQieyiUseAfter {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "qieyi_use_after");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QIEYI_USE_AFTER));
            }
            StmtIr::EffectJianzhiPhaseEnd {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "jianzhi_phase_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::JIANZHI_PHASE_END));
            }
            StmtIr::EffectMoveCard {
                card,
                to,
                from,
                span,
            } => {
                self.mark(*span, "move_card");
                self.emit_expr(card)?;
                self.emit_expr(to)?;
                self.emit_expr(from)?;
                self.ops.push(Op::HostEffect(effect_tag::MOVE_CARD));
            }
            StmtIr::EffectSetTurnState {
                current,
                phase,
                span,
            } => {
                self.mark(*span, "set_turn_state");
                self.emit_expr(current)?;
                self.ops.push(Op::ConstI32(*phase));
                self.ops.push(Op::HostEffect(effect_tag::SET_TURN_STATE));
            }
            StmtIr::EffectUseCardOn {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "use_card_on");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::USE_CARD_ON));
            }
            StmtIr::EffectUseVirtualCardOn {
                player,
                kind,
                target,
                span,
            } => {
                self.mark(*span, "use_virtual_card_on");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_13);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::USE_VIRTUAL_CARD_ON));
            }
            StmtIr::EffectEnterDying { player, span } => {
                self.mark(*span, "enter_dying");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::ENTER_DYING));
            }
            StmtIr::EffectRevealCard { card, span } => {
                self.mark(*span, "reveal_card");
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::REVEAL_CARD));
            }
            StmtIr::EffectUseCard { player, card, span } => {
                self.mark(*span, "use_card");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::USE_CARD));
            }
            StmtIr::EffectRemoveCardsByKind { kind, span } => {
                self.mark(*span, "remove_cards_by_kind");
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::REMOVE_CARDS_BY_KIND));
            }
            StmtIr::EffectGainCardByKind { player, kind, span } => {
                self.mark(*span, "gain_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_10);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::HostEffect(effect_tag::GAIN_CARD_BY_KIND));
            }
            StmtIr::EffectKillPlayer {
                player,
                killer,
                span,
            } => {
                self.mark(*span, "kill_player");
                self.emit_expr(player)?;
                self.emit_expr(killer)?;
                self.ops.push(Op::HostEffect(effect_tag::KILL_PLAYER));
            }
            StmtIr::Let { name, value, span } => {
                self.mark(*span, "let");
                self.emit_expr(value)?;
                let slot = self.local_slot(name)?;
                self.ops.push(Op::Store(slot));
            }
            StmtIr::If {
                cond,
                then_body,
                else_body,
                span,
            } => {
                self.mark(*span, "if");
                self.emit_expr(cond)?;
                // JumpIfFalse to else; then jump over else.
                // Use placeholder offsets that are fixed post-assemble from actual bytes.
                let jif_idx = self.ops.len();
                self.ops.push(Op::JumpIfFalse(0)); // placeholder
                self.emit_stmts(then_body)?;
                let jmp_idx = self.ops.len();
                self.ops.push(Op::Jump(0)); // placeholder
                let else_op_idx = self.ops.len();
                self.emit_stmts(else_body)?;
                let end_op_idx = self.ops.len();
                self.backpatches.push(BackpatchEntry::If {
                    jif_op_idx: jif_idx,
                    jmp_op_idx: jmp_idx,
                    else_op_idx,
                    end_op_idx,
                });
            }
            StmtIr::Loop { times, body, span } => {
                self.mark(*span, "loop");
                // for i in 0..times { body } using local counter
                let slot_name = format!("__loop_{}", self.locals.len());
                let slot = self.local_slot(&slot_name)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::Store(slot));
                let loop_start = self.ops.len();
                // if counter >= times jump end
                self.ops.push(Op::Load(slot));
                self.ops.push(Op::ConstI32(*times as i32));
                self.ops.push(Op::Lt); // counter < times
                let jif_idx = self.ops.len();
                self.ops.push(Op::JumpIfFalse(0));
                self.emit_stmts(body)?;
                // counter++
                self.ops.push(Op::Load(slot));
                self.ops.push(Op::ConstI32(1));
                self.ops.push(Op::Add);
                self.ops.push(Op::Store(slot));
                // jump loop_start
                let jmp_back = self.ops.len();
                self.ops.push(Op::Jump(0));
                let end_op_idx = self.ops.len();
                self.backpatches.push(BackpatchEntry::Loop {
                    jif_op_idx: jif_idx,
                    jmp_back_op_idx: jmp_back,
                    loop_start_op_idx: loop_start,
                    end_op_idx,
                });
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
