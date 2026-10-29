//! 效果语句代码生成：具名技能结算器（EffectResolve*）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_convert_current_sha_to_fire_lose_hp_if_damage_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveYoudi { player, span } => {
                self.mark(*span, "resolve_youdi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_280);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YOUDI));
            }
            StmtIr::EffectResolveKuangfu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_kuangfu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_282);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KUANGFU));
            }
            StmtIr::EffectResolveShanjia {
                player,
                count,
                span,
            } => {
                self.mark(*span, "resolve_shanjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_283);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHANJIA));
            }
            StmtIr::EffectResolveNewYajiao { player, card, span } => {
                self.mark(*span, "resolve_new_yajiao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_284);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NEW_YAJIAO));
            }
            StmtIr::EffectResolveDingpin {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dingpin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_285);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGPIN));
            }
            StmtIr::EffectResolveXinfuGuolun {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_xinfu_guolun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_294);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_GUOLUN));
            }
            StmtIr::EffectResolveLieren {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_lieren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_531);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIEREN));
            }
            StmtIr::EffectResolveTuntian { player, span } => {
                self.mark(*span, "resolve_tuntian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_533);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TUNTIAN));
            }
            StmtIr::EffectResolvePotdimeng { player, span } => {
                self.mark(*span, "resolve_potdimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_535);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTDIMENG));
            }
            StmtIr::EffectResolvePothaoshi {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_pothaoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_536);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTHAOSHI));
            }
            StmtIr::EffectResolveSpmiaolve {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_spmiaolve");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_539);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPMIAOLVE));
            }
            StmtIr::EffectResolveSpyingjia {
                player,
                action,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_spyingjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_539);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*helper_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYINGJIA));
            }
            StmtIr::EffectResolveZaoxian {
                player,
                jixi_def,
                tuntian_def,
                span,
            } => {
                self.mark(*span, "resolve_zaoxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_534);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*jixi_def as i32));
                self.ops.push(Op::ConstI32(*tuntian_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAOXIAN));
            }
            StmtIr::EffectResolveTiaoxin { player, span } => {
                self.mark(*span, "resolve_tiaoxin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_537);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIAOXIN));
            }
            StmtIr::EffectResolveZhiji {
                player,
                guanxing_def,
                span,
            } => {
                self.mark(*span, "resolve_zhiji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_538);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*guanxing_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIJI));
            }
            StmtIr::EffectResolveZaiqi {
                player,
                extra,
                span,
            } => {
                self.mark(*span, "resolve_zaiqi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_540);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*extra as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAIQI));
            }
            StmtIr::EffectResolveTwqiushou { player, span } => {
                self.mark(*span, "resolve_twqiushou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_542);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQIUSHOU));
            }
            StmtIr::EffectResolveZhengrong { player, span } => {
                self.mark(*span, "resolve_zhengrong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_545);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGRONG));
            }
            StmtIr::EffectResolveHongju {
                player,
                qingjian_def,
                rong_def,
                span,
            } => {
                self.mark(*span, "resolve_hongju");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_546);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qingjian_def as i32));
                self.ops.push(Op::ConstI32(*rong_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HONGJU));
            }
            StmtIr::EffectResolveQingjian {
                player,
                rong_def,
                span,
            } => {
                self.mark(*span, "resolve_qingjian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_547);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*rong_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINGJIAN));
            }
            StmtIr::EffectResolveReleiji { player, span } => {
                self.mark(*span, "resolve_releiji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_549);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RELEIJI));
            }
            StmtIr::EffectResolveHuangtian {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_huangtian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_551);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HUANGTIAN));
            }
            StmtIr::EffectResolveHaoshi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_haoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_555);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HAOSHI));
            }
            StmtIr::EffectResolveDimeng { player, span } => {
                self.mark(*span, "resolve_dimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_556);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DIMENG));
            }
            StmtIr::EffectResolveQiaobian {
                player,
                phase,
                span,
            } => {
                self.mark(*span, "resolve_qiaobian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_559);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*phase as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIAOBIAN));
            }
            StmtIr::EffectResolveXiangle {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_xiangle");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIANGLE));
            }
            StmtIr::EffectResolveFangquanSkip { player, span } => {
                self.mark(*span, "resolve_fangquan_skip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FANGQUAN_SKIP));
            }
            StmtIr::EffectResolveFangquanGrant { player, span } => {
                self.mark(*span, "resolve_fangquan_grant");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FANGQUAN_GRANT));
            }
            StmtIr::EffectResolveRuoyu {
                player,
                jijiang_def,
                span,
            } => {
                self.mark(*span, "resolve_ruoyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*jijiang_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RUOYU));
            }
            StmtIr::EffectResolveZuilun { player, span } => {
                self.mark(*span, "resolve_zuilun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_563);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZUILUN));
            }
            StmtIr::EffectResolveFuyin {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_fuyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_563);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FUYIN));
            }
            StmtIr::EffectResolveJuzhan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_juzhan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_571);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JUZHAN));
            }
            StmtIr::EffectResolveQizhi { player, span } => {
                self.mark(*span, "resolve_qizhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_573);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIZHI));
            }
            StmtIr::EffectResolveJinqu {
                player,
                qizhi_def,
                span,
            } => {
                self.mark(*span, "resolve_jinqu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_573);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qizhi_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JINQU));
            }
            StmtIr::EffectResolveJianxiang { player, span } => {
                self.mark(*span, "resolve_jianxiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_574);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANXIANG));
            }
            StmtIr::EffectResolveShenshi {
                player,
                source,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shenshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_574);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHENSHI));
            }
            StmtIr::EffectResolveMingren {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mingren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_576);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MINGREN));
            }
            StmtIr::EffectResolveZhenliang {
                player,
                source,
                card,
                mingren_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_zhenliang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_576);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*mingren_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENLIANG));
            }
            StmtIr::EffectResolveKuizhu { player, span } => {
                self.mark(*span, "resolve_kuizhu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KUIZHU));
            }
            StmtIr::EffectResolveZhizheng { player, span } => {
                self.mark(*span, "resolve_zhizheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIZHENG));
            }
            StmtIr::EffectResolveLijun {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_lijun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIJUN));
            }
            StmtIr::EffectResolveLiangyin {
                player,
                kongsheng_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_liangyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_579);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kongsheng_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIANGYIN));
            }
            StmtIr::EffectResolveKongsheng {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_kongsheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_579);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KONGSHENG));
            }
            StmtIr::EffectResolvePotzhouyu {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potzhouyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_561);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTZHOUYU));
            }
            StmtIr::EffectResolvePotyuji {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potyuji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_570);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTYUJI));
            }
            StmtIr::EffectResolveRehengjiang {
                player,
                actor,
                action,
                hand_limit_def,
                draw_def,
                span,
            } => {
                self.mark(*span, "resolve_rehengjiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_572);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*hand_limit_def as i32));
                self.ops.push(Op::ConstI32(*draw_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_REHENGJIANG));
            }
            StmtIr::EffectResolveDccunsi {
                player,
                target,
                guixiu_def,
                yongjue_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_dccunsi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_578);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*guixiu_def as i32));
                self.ops.push(Op::ConstI32(*yongjue_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCCUNSI));
            }
            StmtIr::EffectResolvePotlvyi {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potlvyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_564);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLVYI));
            }
            StmtIr::EffectResolvePotcaoshuang {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potcaoshuang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_568);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCAOSHUANG));
            }
            StmtIr::EffectResolvePottuntian {
                player,
                action,
                zaoxian_def,
                span,
            } => {
                self.mark(*span, "resolve_pottuntian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_543);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*zaoxian_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTTUNTIAN));
            }
            StmtIr::EffectResolvePotjixi {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potjixi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_543);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIXI));
            }
            StmtIr::EffectResolvePotceduan {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potceduan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_544);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTCEDUAN));
            }
            StmtIr::EffectResolvePottaishici {
                player,
                target,
                action,
                hanzhan_def,
                zhanlie_def,
                span,
            } => {
                self.mark(*span, "resolve_pottaishici");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_548);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*hanzhan_def as i32));
                self.ops.push(Op::ConstI32(*zhanlie_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTTAISHICI));
            }
            StmtIr::EffectResolveMbzhonghui {
                player,
                target,
                action,
                amount,
                mbsizi_def,
                mbkechang_def,
                span,
            } => {
                self.mark(*span, "resolve_mbzhonghui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_553);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.emit_expr(amount)?;
                self.ops.push(Op::ConstI32(*mbsizi_def as i32));
                self.ops.push(Op::ConstI32(*mbkechang_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBZHONGHUI));
            }
            StmtIr::EffectResolvePotjiejie {
                player,
                actor,
                used_def,
                suit_defs,
                qingshi_def,
                span,
            } => {
                self.mark(*span, "resolve_potjiejie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_554);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(*used_def as i32));
                for def in suit_defs {
                    self.ops.push(Op::ConstI32(*def as i32));
                }
                self.ops.push(Op::ConstI32(*qingshi_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIEJIE));
            }
            StmtIr::EffectResolvePotqingshi { player, span } => {
                self.mark(*span, "resolve_potqingshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_554);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTQINGSHI));
            }
            StmtIr::EffectResolvePotweiyan {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potweiyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_558);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTWEIYAN));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
