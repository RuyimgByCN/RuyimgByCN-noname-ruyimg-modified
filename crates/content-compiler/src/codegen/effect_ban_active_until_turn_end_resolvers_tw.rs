//! 效果语句代码生成：台服（Tw）具名技能结算器。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_ban_active_until_turn_end_resolvers_tw(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveTwjiaohua {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_twjiaohua");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIAOHUA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_363);
            }
            StmtIr::EffectResolveTwkujian {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twkujian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWKUJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_364);
            }
            StmtIr::EffectResolveTwruilian {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twruilian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWRUILIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_365);
            }
            StmtIr::EffectResolveTwjianwei {
                player,
                actor,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_twjianwei");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIANWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_367);
            }
            StmtIr::EffectResolveTwtanfeng { player, span } => {
                self.mark(*span, "resolve_twtanfeng");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWTANFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_368);
            }
            StmtIr::EffectResolveTwzhibian { player, span } => {
                self.mark(*span, "resolve_twzhibian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWZHIBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_369);
            }
            StmtIr::EffectResolveTwyuyan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twyuyan");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWYUYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_370);
            }
            StmtIr::EffectResolveTwguoyi {
                player,
                target,
                card,
                hand_lock_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_twguoyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*hand_lock_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWGUOYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_371);
            }
            StmtIr::EffectResolveTwchuhai {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchuhai");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHUHAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_372);
            }
            StmtIr::EffectResolveTwyizhu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twyizhu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWYIZHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_373);
            }
            StmtIr::EffectResolveTwluanchou {
                player,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twluanchou");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWLUANCHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_374);
            }
            StmtIr::EffectResolveTwxiawei {
                player,
                wangxing_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twxiawei");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*wangxing_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWXIAWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_375);
            }
            StmtIr::EffectResolveTwbianfuren {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twbianfuren");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWBIANFUREN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_376);
            }
            StmtIr::EffectResolveTwfeiyi {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfeiyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFEIYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_377);
            }
            StmtIr::EffectResolveTwchenzhen {
                player,
                target,
                helper_skill_def,
                record_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchenzhen");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(*record_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWCHENZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_378);
            }
            StmtIr::EffectResolveTwqianju {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_twqianju");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQIANJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_379);
            }
            StmtIr::EffectResolveTwqingxi {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twqingxi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQINGXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_379);
            }
            StmtIr::EffectResolveTwzaoli { player, span } => {
                self.mark(*span, "resolve_twzaoli");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWZAOLI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_380);
            }
            StmtIr::EffectResolveTwzhengjian {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twzhengjian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWZHENGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_381);
            }
            StmtIr::EffectResolveTwtianyu {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twtianyu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWTIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_382);
            }
            StmtIr::EffectResolveTwwujing {
                player,
                target_count,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twwujing");
                self.emit_expr(player)?;
                self.emit_expr(target_count)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWWUJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_384);
            }
            StmtIr::EffectResolveTwshanghe { player, span } => {
                self.mark(*span, "resolve_twshanghe");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWSHANGHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_385);
            }
            StmtIr::EffectResolveTwwangchang {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twwangchang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWWANGCHANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_386);
            }
            StmtIr::EffectResolveTwcaozhao {
                player,
                target,
                source,
                card,
                twfuzuan_skill_def,
                twfeifu_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twcaozhao");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*twfuzuan_skill_def as i32));
                self.ops.push(Op::ConstI32(*twfeifu_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCAOZHAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_387);
            }
            StmtIr::EffectResolveTwguohuai {
                player,
                target,
                source,
                card,
                mark_def,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twguohuai");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWGUOHUAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_388);
            }
            StmtIr::EffectResolveTwchengpu {
                player,
                target,
                source,
                card,
                chunlao_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchengpu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*chunlao_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHENGPU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_389);
            }
            StmtIr::EffectResolveTwcaoang {
                player,
                target,
                source,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twcaoang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCAOANG));
                self.required_abi = self.required_abi.max(if *action == 2 {
                    ADDITIVE_RULE_VM_ABI_VERSION_392
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_390
                });
            }
            StmtIr::EffectResolveTwniujin {
                player,
                limited_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twniujin");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*limited_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWNIUJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_391);
            }
            StmtIr::EffectResolveTwmoukui {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twmoukui");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMOUKUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_393);
            }
            StmtIr::EffectResolveTwfuman {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfuman");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFUMAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_394);
            }
            StmtIr::EffectResolveTwqingkou {
                player,
                winner,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twqingkou");
                self.emit_expr(player)?;
                self.emit_expr(winner)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQINGKOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_396);
            }
            StmtIr::EffectResolveTwchenxun {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchenxun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHENXUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_397);
            }
            StmtIr::EffectResolveTwfenxian {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfenxian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFENXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_399);
            }
            StmtIr::EffectResolveTwjieqiu {
                player,
                target,
                subaction,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_twjieqiu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*subaction));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIEQIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_673);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
