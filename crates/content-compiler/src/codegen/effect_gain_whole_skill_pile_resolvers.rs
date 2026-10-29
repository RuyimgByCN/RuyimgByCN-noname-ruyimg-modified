//! 效果语句代码生成：具名技能结算器（EffectResolve*）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_gain_whole_skill_pile_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveQianlong {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qianlong");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_697);
            }
            StmtIr::EffectResolveJsrgzuozhan {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgzuozhan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_698);
            }
            StmtIr::EffectResolveJsrgzuozhanGain {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_jsrgzuozhan_gain");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_699);
            }
            StmtIr::EffectResolveTwmouzhi {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_twmouzhi");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMOUZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_700);
            }
            StmtIr::EffectResolveTwdaigui {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twdaigui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWDAIGUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_701);
            }
            StmtIr::EffectResolveTwmitu {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twmitu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMITU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_702);
            }
            StmtIr::EffectResolveQiantun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qiantun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANTUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_703);
            }
            StmtIr::EffectResolveYangming {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_yangming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YANGMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_704);
            }
            StmtIr::EffectResolveCanyun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_canyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_705);
            }
            StmtIr::EffectResolveHengwu {
                player,
                skill,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_hengwu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HENGWU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_706);
            }
            StmtIr::EffectResolveBiaozhao3 {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_biaozhao3");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO3));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_707);
            }
            StmtIr::EffectResolveChuanyun {
                player,
                skill,
                event_player,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_chuanyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_708);
            }
            StmtIr::EffectResolveBiaozhao2 {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_biaozhao2");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO2));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_709);
            }
            StmtIr::EffectResolveDcniji {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_dcniji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNIJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_710);
            }
            StmtIr::EffectResolveDcmoukui {
                player,
                skill,
                stage,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcmoukui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMOUKUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_711);
            }
            StmtIr::EffectResolveZhongyan {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_zhongyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_712);
            }
            StmtIr::EffectResolveChumei {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_chumei");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUMEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_713);
            }
            StmtIr::EffectResolveShuaiyan {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_715);
            }
            StmtIr::EffectResolveMbbeiming {
                player,
                skill,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbbeiming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBBEIMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_718);
            }
            StmtIr::EffectResolveReliangyin {
                player,
                skill,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_reliangyin");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*mode));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_RELIANGYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_717);
            }
            StmtIr::EffectResolveZhengjian {
                player,
                skill,
                stage,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_zhengjian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_716);
            }
            StmtIr::EffectResolveQinmian {
                player,
                skill,
                mode,
                amount,
                span,
            } => {
                self.mark(*span, "resolve_qinmian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINMIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_714);
            }
            StmtIr::EffectResolveJsrgdaoren {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgdaoren");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGDAOREN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_696);
            }
            StmtIr::EffectResolveDuwang {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_duwang");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUWANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_695);
            }
            StmtIr::EffectResolveQuanjia {
                player,
                skill,
                source,
                target,
                span,
            } => {
                self.mark(*span, "resolve_quanjia");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QUANJIA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_694);
            }
            StmtIr::EffectResolvePxqixian {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_pxqixian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_PXQIXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_693);
            }
            StmtIr::EffectResolveDcbingji {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_dcbingji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCBINGJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_692);
            }
            StmtIr::EffectResolveTianxiang {
                player,
                draw_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_heart_redirect_current_damage");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*draw_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_456);
            }
            StmtIr::EffectResolveRetianxiang { player, span } => {
                self.mark(*span, "choose_discard_heart_cancel_damage_branch");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_RETIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_457);
            }
            StmtIr::EffectResolveLianzhou { player, span } => {
                self.mark(*span, "choose_link_other_same_hp");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIANZHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_458);
            }
            StmtIr::EffectResolveStddechong { player, span } => {
                self.mark(*span, "choose_give_hand_equip_to_event_player");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDDECHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_459);
            }
            StmtIr::EffectResolveJieliangGain { player, span } => {
                self.mark(*span, "resolve_jieliang_gain");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JIELIANG_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_461);
            }
            StmtIr::EffectResolveStdjujian { player, span } => {
                self.mark(*span, "choose_other_gain_event_cards");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_STDJUJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_462);
            }
            StmtIr::EffectResolveLulve { player, span } => {
                self.mark(*span, "resolve_lulve");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LULVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_463);
            }
            StmtIr::EffectResolveDctingxian { player, span } => {
                self.mark(*span, "resolve_dctingxian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCTINGXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_464);
            }
            StmtIr::EffectResolveHfjieying {
                player,
                grant_def,
                span,
            } => {
                self.mark(*span, "choose_other_add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*grant_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HFJIEYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_465);
            }
            StmtIr::EffectResolveDcnuchen { player, span } => {
                self.mark(*span, "resolve_dcnuchen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNUCHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDcdanji {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_dcdanji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDczhiheng { player, span } => {
                self.mark(*span, "resolve_dczhiheng");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCZHIHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_467);
            }
            StmtIr::EffectResolveStdgaobian { player, span } => {
                self.mark(*span, "resolve_stdgaobian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDGAOBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_468);
            }
            StmtIr::EffectResolveSpjiedao { player, span } => {
                self.mark(*span, "resolve_spjiedao");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPJIEDAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_469);
            }
            StmtIr::EffectResolveSibian { player, span } => {
                self.mark(*span, "resolve_sibian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SIBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_470);
            }
            StmtIr::EffectResolveRefuhai { player, span } => {
                self.mark(*span, "resolve_refuhai");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REFUHAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_493);
            }
            StmtIr::EffectResolveGzjinfa {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_gzjinfa");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GZJINFA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_495);
            }
            StmtIr::EffectResolveWnQifeng {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_wn_qifeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WN_QIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_496);
            }
            StmtIr::EffectResolveDcjue {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcjue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCJUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_583);
            }
            StmtIr::EffectResolveRuanji {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ruanji");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RUANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_584);
            }
            StmtIr::EffectResolveZhengbi {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhengbi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGBI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveFengying {
                player,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_fengying");
                self.emit_expr(player)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FENGYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveQiaojian {
                player,
                target,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_qiaojian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIAOJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveXicha {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_xicha");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XICHA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveHuzun {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_huzun");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HUZUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_587);
            }
            StmtIr::EffectResolveZhugemengxue {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugemengxue");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGEMENGXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_588);
            }
            StmtIr::EffectResolveBailingyun {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bailingyun");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BAILINGYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_593);
            }
            StmtIr::EffectResolveDcshaowei {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcshaowei");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCSHAOWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdichou {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdichou");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDICHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdufeng {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdufeng");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDUFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_595);
            }
            StmtIr::EffectResolveDcctjiuxian {
                player,
                event_player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcctjiuxian");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCCTJIUXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_597);
            }
            StmtIr::EffectResolveBianyue {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bianyue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIANYUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_598);
            }
            StmtIr::EffectResolveZhupeilan {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhupeilan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUPEILAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_600);
            }
            StmtIr::EffectResolveDcSbZhangxiu {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhangxiu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHANGXIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_601);
            }
            StmtIr::EffectResolveDcSbZhouyu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhouyu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHOUYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_602);
            }
            StmtIr::EffectResolveDcSbLusu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_lusu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_LUSU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_603);
            }
            StmtIr::EffectResolveZhangjian {
                player,
                card,
                grant_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhangjian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*grant_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHANGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
            }
            StmtIr::EffectResolveZhugeruoxue {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugeruoxue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGERUOXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_605);
            }
            StmtIr::EffectResolveCaoyi {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_caoyi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CAOYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectResolveMalingli {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_malingli");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MALINGLI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
            }
            StmtIr::EffectResolveNsshuaiyan {
                player,
                recovered_player,
                span,
            } => {
                self.mark(*span, "resolve_nsshuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(recovered_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NSSHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_608);
            }
            StmtIr::EffectResolveWdbHanlong {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_hanlong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_HANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_609);
            }
            StmtIr::EffectResolveWdbLiufu {
                player,
                target,
                card,
                zhucheng_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liufu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*zhucheng_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUFU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_610);
            }
            StmtIr::EffectResolveWdbLiuyan {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuyan");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_611);
            }
            StmtIr::EffectResolveWdbLiuzan {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuzan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUZAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_612);
            }
            StmtIr::EffectResolveWdbTianyu {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_tianyu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_TIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_613);
            }
            StmtIr::EffectResolveWdbXizhenxihong {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_xizhenxihong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_XIZHENXIHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_614);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
