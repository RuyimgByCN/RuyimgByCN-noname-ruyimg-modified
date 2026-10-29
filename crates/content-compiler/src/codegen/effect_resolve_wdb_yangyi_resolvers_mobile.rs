//! 效果语句代码生成：手杀（Mb）与谋攻（Pot）具名技能结算器。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_resolve_wdb_yangyi_resolvers_mobile(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveMbqingdao {
                player,
                source,
                card,
                damaged,
                span,
            } => {
                self.mark(*span, "resolve_mbqingdao");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(damaged)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBQINGDAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbxiugeng {
                player,
                helper_def,
                hand_limit_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiugeng");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*helper_def));
                self.ops.push(Op::ConstU32(*hand_limit_def));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIUGENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbchenshe {
                player,
                dying,
                source,
                span,
            } => {
                self.mark(*span, "resolve_mbchenshe");
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBCHENSHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbquanchong {
                player,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_mbquanchong");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*helper_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBQUNCHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_504);
            }
            StmtIr::EffectResolveMbrenxing { player, span } => {
                self.mark(*span, "resolve_mbrenxing");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBRENXING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_504);
            }
            StmtIr::EffectResolveMbchizhang { player, card, span } => {
                self.mark(*span, "resolve_mbchizhang");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBCHIZHANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_505);
            }
            StmtIr::EffectResolveMbduanyang {
                player,
                target,
                card,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_mbduanyang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBDUANYANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_505);
            }
            StmtIr::EffectResolveMbrunwei {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbrunwei");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBRUNWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_506);
            }
            StmtIr::EffectResolveMbshuanghuai {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbshuanghuai");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBSHUANGHUAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_506);
            }
            StmtIr::EffectResolveMbxiongtu {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiongtu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIONGTU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_507);
            }
            StmtIr::EffectResolveMbganggeng {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbganggeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBGANGGENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_508);
            }
            StmtIr::EffectResolveMbsijian {
                player,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_mbsijian");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBSIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_508);
            }
            StmtIr::EffectResolveMbjianji { player, span } => {
                self.mark(*span, "resolve_mbjianji");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJIANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_509);
            }
            StmtIr::EffectResolveMbyuanmo { player, span } => {
                self.mark(*span, "resolve_mbyuanmo");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBYUANMO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_509);
            }
            StmtIr::EffectResolveMbfeijing {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_mbfeijing");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBFEIJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_510);
            }
            StmtIr::EffectResolveMbxiaoge {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiaoge");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIAOGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_510);
            }
            StmtIr::EffectResolveMbxuye {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbxuye");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXUYE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_514);
            }
            StmtIr::EffectResolveMbkuangxiang {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbkuangxiang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBKUANGXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_514);
            }
            StmtIr::EffectResolvePotchengpu {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potchengpu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCHENGPU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_515);
            }
            StmtIr::EffectResolvePotjugu { player, span } => {
                self.mark(*span, "resolve_potjugu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJUGU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_516);
            }
            StmtIr::EffectResolvePotsifeng {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potsifeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTSIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_517);
            }
            StmtIr::EffectResolvePotchenqun {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potchenqun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCHENQUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_518);
            }
            StmtIr::EffectResolvePotwanglie {
                player,
                card,
                helper_skill_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potwanglie");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTWANGLIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_519);
            }
            StmtIr::EffectResolvePothongyi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_pothongyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTHONGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_519);
            }
            StmtIr::EffectResolvePotguansha {
                player,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_potguansha");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTGUANSHA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_520);
            }
            StmtIr::EffectResolvePotjiyu {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potjiyu");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_520);
            }
            StmtIr::EffectResolveMbzhuji {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbzhuji");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHUJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_521);
            }
            StmtIr::EffectResolvePotlibing {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potlibing");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLIBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            StmtIr::EffectResolvePotpoxi { player, span } => {
                self.mark(*span, "resolve_potpoxi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTPOXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            StmtIr::EffectResolvePotfuan {
                player,
                card,
                stage,
                range_helper_def,
                yinxian_def,
                span,
            } => {
                self.mark(*span, "resolve_potfuan");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::ConstI32(*range_helper_def as i32));
                self.ops.push(Op::ConstI32(*yinxian_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTFUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            StmtIr::EffectResolvePotyinxian {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potyinxian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTYINXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            StmtIr::EffectResolvePothuanjie {
                player,
                target,
                stage,
                qice_def,
                kanpo_def,
                span,
            } => {
                self.mark(*span, "resolve_pothuanjie");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::ConstI32(*qice_def as i32));
                self.ops.push(Op::ConstI32(*kanpo_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTHUANJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_529);
            }
            StmtIr::EffectResolvePotnigu {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potnigu");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTNIGU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_532);
            }
            StmtIr::EffectResolvePotlulian {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potlulian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLULIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_532);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
