//! 效果语句代码生成：具名技能结算器（EffectResolve*）。

use super::*;
use crate::abi_versions::ADDITIVE_RULE_VM_ABI_VERSION_757;

impl Assembler {
    pub(super) fn emit_effect_set_draw_phase_from_bottom_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveChenglve {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_chenglve");
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHENGLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectResolveShicai {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shicai");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHICAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectResolveSbbiyue { player, span } => {
                self.mark(*span, "resolve_sbbiyue");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SBBIYUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_599);
            }
            StmtIr::EffectResolveSxrmqiewang {
                player,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_sxrmqiewang");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMQIEWANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            StmtIr::EffectResolveSxrmqianliu {
                player,
                source,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_sxrmqianliu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(stage)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMQIANLIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            StmtIr::EffectResolveGzbuqu {
                player,
                delta,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_gzbuqu");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GZBUQU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_591);
            }
            StmtIr::EffectResolveGzbuquRecover {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_gzbuqu_recover");
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GZBUQU_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_591);
            }
            StmtIr::EffectResolveXiongluan {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_xiongluan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIONGLUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_590);
            }
            StmtIr::EffectResolveTwjuxiang {
                player,
                event_player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_twjuxiang");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJUXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_590);
            }
            StmtIr::EffectResolveOlJieDanshou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_ol_jie_danshou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_757);
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OL_JIE_DANSHOU));
            }
            StmtIr::EffectResolveRetieji {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "resolve_retieji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_316);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RETIEJI));
            }
            StmtIr::EffectResolveNewReluoyi {
                player,
                buff_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_new_reluoyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_317);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*buff_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NEW_RELUOYI));
            }
            StmtIr::EffectResolveRemoshi {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_remoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_318);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REMOSHI));
            }
            StmtIr::EffectResolveXinyaoming { player, span } => {
                self.mark(*span, "resolve_xinyaoming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_319);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINYAOMING));
            }
            StmtIr::EffectResolveRehuaiyi { player, span } => {
                self.mark(*span, "resolve_rehuaiyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_320);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REHUAIYI));
            }
            StmtIr::EffectResolveReanguo { player, span } => {
                self.mark(*span, "resolve_reanguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_321);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REANGUO));
            }
            StmtIr::EffectResolveBeizhu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_beizhu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_324);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BEIZHU));
            }
            StmtIr::EffectResolveMbshanjia {
                player,
                lost_equipment_count,
                distance_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_mbshanjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_326);
                self.emit_expr(player)?;
                self.emit_expr(lost_equipment_count)?;
                self.ops.push(Op::ConstU32(*distance_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBSHANJIA));
            }
            StmtIr::EffectResolveDaoji {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_daoji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_327);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DAOJI));
            }
            StmtIr::EffectResolveTwDaoji {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_tw_daoji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_355);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TW_DAOJI));
            }
            StmtIr::EffectResolveTwfupan {
                player,
                damage_amount,
                span,
            } => {
                self.mark(*span, "resolve_twfupan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_362);
                self.emit_expr(player)?;
                self.emit_expr(damage_amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFUPAN));
            }
            StmtIr::EffectResolveJiaohua { player, span } => {
                self.mark(*span, "resolve_jiaohua");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_328);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIAOHUA));
            }
            StmtIr::EffectResolveLuanqun {
                player,
                effect_skill_def,
                direct_hit_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_luanqun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_329);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*effect_skill_def));
                self.ops.push(Op::ConstU32(*direct_hit_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LUANQUN));
            }
            StmtIr::EffectResolveMbzhuguo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbzhuguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_331);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHUGUO));
            }
            StmtIr::EffectResolveMbanxian { player, span } => {
                self.mark(*span, "resolve_mbanxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_332);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBANXIAN));
            }
            StmtIr::EffectResolveMbjinzu {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_mbjinzu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_333);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJINZU));
            }
            StmtIr::EffectResolveDuoduan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_duoduan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_721);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUODUAN));
            }
            StmtIr::EffectResolveQiangyong {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_qiangyong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_727);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANGYONG));
            }
            StmtIr::EffectResolveDcqiaolve {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcqiaolve");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_732);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCQIAOLVE));
            }
            StmtIr::EffectResolveTwxiongjun { player, span } => {
                self.mark(*span, "resolve_twxiongjun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_733);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TW_XIONGJUN));
            }
            StmtIr::EffectResolveYixiang { player, span } => {
                self.mark(*span, "resolve_yixiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_734);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YIXIANG));
            }
            StmtIr::EffectResolveXinfuQiai { player, span } => {
                self.mark(*span, "resolve_xinfu_qiai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_735);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_QIAI));
            }
            StmtIr::EffectResolveOlwenren { player, span } => {
                self.mark(*span, "resolve_olwenren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_736);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLWENREN));
            }
            StmtIr::EffectResolveSpolaige { player, span } => {
                self.mark(*span, "resolve_spolaige");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_737);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPOLAIGE));
            }
            StmtIr::EffectResolveOlzhijie { player, span } => {
                self.mark(*span, "resolve_olzhijie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_738);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLZHIJIE));
            }
            StmtIr::EffectResolveOlsuyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_olsuyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_739);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLSUYI));
            }
            StmtIr::EffectResolveOlqiaozhi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_olqiaozhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_740);
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLQIAOZHI));
            }
            StmtIr::EffectResolveDczhee { player, mode, span } => {
                self.mark(*span, "resolve_dczhee");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_746);
                self.emit_expr(player)?;
                self.emit_expr(mode)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCZHEE));
            }
            StmtIr::EffectResolveDchaoxian { player, span } => {
                self.mark(*span, "resolve_dchaoxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_747);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCHAOXIAN));
            }
            StmtIr::EffectResolveZhongjie { player, span } => {
                self.mark(*span, "resolve_zhongjie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_748);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGJIE));
            }
            StmtIr::EffectResolveStarcuxiaWeiming {
                player,
                source,
                variant,
                span,
            } => {
                self.mark(*span, "resolve_starcuxia_weiming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_750);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(variant)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STARCUXIA_WEIMING));
            }
            StmtIr::EffectResolveSanshou { player, span } => {
                self.mark(*span, "resolve_sanshou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_751);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SANSHOU));
            }
            StmtIr::EffectResolveNhlundao {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_nhlundao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_752);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_NHLUNDAO));
            }
            StmtIr::EffectResolveXiangshu { player, span } => {
                self.mark(*span, "resolve_xiangshu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_753);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIANGSHU));
            }
            StmtIr::EffectResolveDingce {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_dingce");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_730);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGCE));
            }
            StmtIr::EffectResolveMeiyong { player, span } => {
                self.mark(*span, "resolve_meiyong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_726);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MEIYONG));
            }
            StmtIr::EffectResolveMbzhenann { player, span } => {
                self.mark(*span, "resolve_mbzhenann");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_725);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHENANN));
            }
            StmtIr::EffectResolveFriendbiwei {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_friendbiwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_724);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FRENDBIWEI));
            }
            StmtIr::EffectResolveChulao { player, span } => {
                self.mark(*span, "resolve_chulao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_723);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHULAO));
            }
            StmtIr::EffectResolveDcanxu { player, span } => {
                self.mark(*span, "resolve_dcanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_722);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCANXU));
            }
            StmtIr::EffectResolveGaoyuan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_gaoyuan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_720);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GAOYUAN));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
