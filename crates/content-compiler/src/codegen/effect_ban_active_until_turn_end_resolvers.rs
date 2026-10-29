//! 效果语句代码生成：具名技能结算器（EffectResolve*，台服之外）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_ban_active_until_turn_end_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveSpxizhan {
                player,
                phase_player,
                fangzong_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_spxizhan");
                self.emit_expr(player)?;
                self.emit_expr(phase_player)?;
                self.ops.push(Op::ConstI32(*fangzong_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPXIZHAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_403);
            }
            StmtIr::EffectResolveSpsongshu {
                player,
                phase_player,
                renku_skill_def,
                block_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_spsongshu");
                self.emit_expr(player)?;
                self.emit_expr(phase_player)?;
                self.ops.push(Op::ConstI32(*renku_skill_def as i32));
                self.ops.push(Op::ConstI32(*block_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSONGSHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_404);
            }
            StmtIr::EffectResolveSpcunsi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spcunsi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPCUNSI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_406);
            }
            StmtIr::EffectResolveYuli { player, mode, span } => {
                self.mark(*span, "resolve_yuli");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YULI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_409);
            }
            StmtIr::EffectResolveTingwei { player, mode, span } => {
                self.mark(*span, "resolve_tingwei");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TINGWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_410);
            }
            StmtIr::EffectResolveXinmingshi {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_xinmingshi");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINMINGSHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_411);
            }
            StmtIr::EffectResolveXunyi { player, mode, span } => {
                self.mark(*span, "resolve_xunyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUNYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_412);
            }
            StmtIr::EffectResolveBoming {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_boming");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BOMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_413);
            }
            StmtIr::EffectResolveEjian {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_ejian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_EJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_414);
            }
            StmtIr::EffectResolveYiyong { player, span } => {
                self.mark(*span, "resolve_yiyong");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YIYONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_415);
            }
            StmtIr::EffectResolveShanxie { player, span } => {
                self.mark(*span, "resolve_shanxie");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHANXIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_416);
            }
            StmtIr::EffectResolveSheyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_sheyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHEYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_417);
            }
            StmtIr::EffectResolveTianyin { player, span } => {
                self.mark(*span, "resolve_tianyin");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIANYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_418);
            }
            StmtIr::EffectResolveDbzhuifeng { player, span } => {
                self.mark(*span, "resolve_dbzhuifeng");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DBZHUIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_419);
            }
            StmtIr::EffectResolveDbquedi {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_dbquedi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DBQUEDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_420);
            }
            StmtIr::EffectResolveSpyinju {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spyinju");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYINJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_421);
            }
            StmtIr::EffectResolveSpchijie {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_spchijie");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPCHIJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_422);
            }
            StmtIr::EffectResolveSpshangyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spshangyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSHANGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_423);
            }
            StmtIr::EffectResolveSpjianyi { player, span } => {
                self.mark(*span, "resolve_spjianyi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPJIANYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_424);
            }
            StmtIr::EffectResolveSpyajun {
                player,
                less_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spyajun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_428);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*less_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYAJUN));
            }
            StmtIr::EffectResolveSpzundi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spzundi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPZUNDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_429);
            }
            StmtIr::EffectResolveXinyangjie {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_xinyangjie");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINYANGJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_430);
            }
            StmtIr::EffectResolveQingyu {
                player,
                action,
                xuancun_def,
                mbyongjue_def,
                span,
            } => {
                self.mark(*span, "resolve_qingyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_431);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*xuancun_def as i32));
                self.ops.push(Op::ConstI32(*mbyongjue_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINGYU));
            }
            StmtIr::EffectResolveZhengsu {
                player,
                mode,
                stage,
                card,
                span,
            } => {
                self.mark(*span, "resolve_zhengsu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_432);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGSU));
            }
            StmtIr::EffectResolveHoufeng {
                player,
                target,
                mode,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_houfeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_433);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HOUFENG));
            }
            StmtIr::EffectResolveMbdiancai {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_mbdiancai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_434);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBDIANCAI));
            }
            StmtIr::EffectResolveRechuhai {
                player,
                target,
                card,
                target_card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_rechuhai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_435);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.emit_expr(target_card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RECHUHAI));
            }
            StmtIr::EffectResolveXiongshu {
                player,
                target,
                card,
                action,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_xiongshu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_436);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIONGSHU));
            }
            StmtIr::EffectResolveJianhui {
                player,
                source,
                target,
                span,
            } => {
                self.mark(*span, "resolve_jianhui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_437);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANHUI));
            }
            StmtIr::EffectResolveSpqiai {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spqiai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_439);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPQIAI));
            }
            StmtIr::EffectResolveSpshanxi {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spshanxi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_440);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSHANXI));
            }
            StmtIr::EffectResolveSpshanxiSuoming {
                player,
                actor,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spshanxi_suoming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_441);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SPSHANXI_SUOMING));
            }
            StmtIr::EffectResolveTongduo {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_tongduo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_442);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TONGDUO));
            }
            StmtIr::EffectResolveYuanqing { player, span } => {
                self.mark(*span, "resolve_yuanqing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_443);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YUANQING));
            }
            StmtIr::EffectResolveShuchen {
                player,
                dying,
                renku_def,
                span,
            } => {
                self.mark(*span, "resolve_shuchen");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_444);
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.ops.push(Op::ConstI32(*renku_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHUCHEN));
            }
            StmtIr::EffectResolveDuanbi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_duanbi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_445);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUANBI));
            }
            StmtIr::EffectResolveZaoli { player, mark, span } => {
                self.mark(*span, "resolve_zaoli");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_446);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAOLI));
            }
            StmtIr::EffectResolveGuying {
                player,
                actor,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_guying");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_447);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUYING));
            }
            StmtIr::EffectResolveFyjianyu { player, a, b, span } => {
                self.mark(*span, "resolve_fyjianyu");
                self.emit_expr(player)?;
                self.emit_expr(a)?;
                self.emit_expr(b)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FYJIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_425);
            }
            StmtIr::EffectResolveLiubing { player, card, span } => {
                self.mark(*span, "resolve_liubing");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIUBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_426);
            }
            StmtIr::EffectResolveHeji {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_heji");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HEJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_427);
            }
            StmtIr::EffectResolveCuijin {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_cuijin");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CUIJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_395);
            }
            StmtIr::EffectResolveQiluan {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qiluan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QILUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_605);
            }
            StmtIr::EffectResolveOllinjie {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_ollinjie");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLLINJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_608);
            }
            StmtIr::EffectResolveTurnEndDebates {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_turn_end_debates");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TURN_END_DEBATES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_663);
            }
            StmtIr::EffectResolvePotheyun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_potheyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTHEYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_677);
            }
            StmtIr::EffectResolveJsrgchaozheng {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgchaozheng");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGCHAOZHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_653);
            }
            StmtIr::EffectResolveJsrgsheju {
                player,
                counterpart,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgsheju");
                self.emit_expr(player)?;
                self.emit_expr(counterpart)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JSRGSHEJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_654);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
