//! 效果语句代码生成：具名技能结算器（EffectResolve*，手杀/谋攻之外）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_resolve_wdb_yangyi_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveWdbYangyi {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_yangyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_YANGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_615);
            }
            StmtIr::EffectResolveWdbZaozhiRenjun {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_zaozhirenjun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_ZAOZHIRENJUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_616);
            }
            StmtIr::EffectResolveDddXianxi {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dddxianxi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIANXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_617);
            }
            StmtIr::EffectResolveDddXiaheng {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dddxiaheng");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIAHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_618);
            }
            StmtIr::EffectResolveDddLiuchong {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuchong");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUCHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_619);
            }
            StmtIr::EffectResolveDddDuanbing {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_duanbing");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_DUANBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_621);
            }
            StmtIr::EffectResolveDddWuzhi {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_wuzhi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_WUZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_622);
            }
            StmtIr::EffectResolveDddXujing {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xujing");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XUJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_623);
            }
            StmtIr::EffectResolveDddCaomao {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_caomao");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_CAOMAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_624);
            }
            StmtIr::EffectResolveDddXinxianying {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xinxianying");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XINXIANYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_625);
            }
            StmtIr::EffectResolveDddXianglang {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xianglang");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIANGLANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_626);
            }
            StmtIr::EffectResolveDddYujin {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_yujin");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_YUJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_627);
            }
            StmtIr::EffectResolveDddLiuye {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuye");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUYE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_628);
            }
            StmtIr::EffectResolveDddBaosanniang {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_baosanniang");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_BAOSANNIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_629);
            }
            StmtIr::EffectResolveDddZhenji {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhenji");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHENJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_630);
            }
            StmtIr::EffectResolveDddZhaoang {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhaoang");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHAOANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_631);
            }
            StmtIr::EffectResolveDddLiuba {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuba");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUBA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_632);
            }
            StmtIr::EffectResolveDddJianshuo {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_jianshuo");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_JIANSHUO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_633);
            }
            StmtIr::EffectResolveDddGuanning {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_guanning");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_GUANNING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_634);
            }
            StmtIr::EffectResolveDddCaoshuang {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_caoshuang");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_CAOSHUANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_635);
            }
            StmtIr::EffectResolveDddXuelingyun {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xuelingyun");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XUELINGYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_636);
            }
            StmtIr::EffectResolveDddLiuhong {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuhong");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_637);
            }
            StmtIr::EffectResolveDddXiahouxuan {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xiahouxuan");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIAHOUXUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_638);
            }
            StmtIr::EffectResolveDddZhangkai {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhangkai");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHANGKAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_639);
            }
            StmtIr::EffectResolveDddLiangxi {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liangxi");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIANGXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_640);
            }
            StmtIr::EffectResolveShoulijian {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_shoulijian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHOULIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_642);
            }
            StmtIr::EffectResolveMobiledanshou {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mobiledanshou");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MOBILEDANSHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_500);
            }
            StmtIr::EffectResolveJsrgchiying {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_jsrgchiying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGCHIYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_501);
            }
            StmtIr::EffectResolveSpfengyin {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spfengyin");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPFENGYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_471);
            }
            StmtIr::EffectResolveXinfuLianpian { player, span } => {
                self.mark(*span, "resolve_xinfu_lianpian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_LIANPIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_472);
            }
            StmtIr::EffectResolveGongjian { player, span } => {
                self.mark(*span, "resolve_gongjian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GONGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_473);
            }
            StmtIr::EffectResolveXinfuQinguo { player, span } => {
                self.mark(*span, "resolve_xinfu_qinguo");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_QINGUO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_474);
            }
            StmtIr::EffectResolveDcmanzhi { player, span } => {
                self.mark(*span, "resolve_dcmanzhi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMANZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_475);
            }
            StmtIr::EffectResolveXinxingluan { player, span } => {
                self.mark(*span, "resolve_xinxingluan");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINXINGLUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_476);
            }
            StmtIr::EffectResolveDckuizhen { player, span } => {
                self.mark(*span, "resolve_dckuizhen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCKUIZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_477);
            }
            StmtIr::EffectResolveDckuizhenResolve {
                player,
                winner,
                loser,
                span,
            } => {
                self.mark(*span, "resolve_dckuizhen_resolve");
                self.emit_expr(player)?;
                self.emit_expr(winner)?;
                self.emit_expr(loser)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCKUIZHEN_RESOLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_477);
            }
            StmtIr::EffectResolveNsdingzhou { player, span } => {
                self.mark(*span, "resolve_nsdingzhou");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NSDINGZHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_478);
            }
            StmtIr::EffectResolveSxrmlanjiao { player, span } => {
                self.mark(*span, "resolve_sxrmlanjiao");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMLANJIAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_479);
            }
            StmtIr::EffectResolveRekuanshi { player, span } => {
                self.mark(*span, "resolve_rekuanshi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REKUANSHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_480);
            }
            StmtIr::EffectResolveRekuanshiResolve {
                player,
                damaged,
                span,
            } => {
                self.mark(*span, "resolve_rekuanshi_resolve");
                self.emit_expr(player)?;
                self.emit_expr(damaged)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_REKUANSHI_RESOLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_480);
            }
            StmtIr::EffectResolveDcmoyu { player, span } => {
                self.mark(*span, "resolve_dcmoyu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMOYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_481);
            }
            StmtIr::EffectResolveXizhen { player, span } => {
                self.mark(*span, "resolve_xizhen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_482);
            }
            StmtIr::EffectResolveXuxie { player, span } => {
                self.mark(*span, "resolve_xuxie");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUXIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_483);
            }
            StmtIr::EffectResolveXuxieAdd { player, span } => {
                self.mark(*span, "resolve_xuxie_add");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUXIE_ADD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_483);
            }
            StmtIr::EffectResolveDcchiying { player, span } => {
                self.mark(*span, "resolve_dcchiying");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCCHIYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_484);
            }
            StmtIr::EffectResolveXinkuangfu { player, span } => {
                self.mark(*span, "resolve_xinkuangfu");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINKUANGFU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_485);
            }
            StmtIr::EffectResolveDcxiangshu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcxiangshu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCXIANG_SHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_486);
            }
            StmtIr::EffectResolveDcxiangshuLottery {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcxiangshu_lottery");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCXIANG_SHU_LOTTERY));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_486);
            }
            StmtIr::EffectResolveShuangxiong { player, span } => {
                self.mark(*span, "resolve_shuangxiong");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHUANGXIONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_511);
            }
            StmtIr::EffectResolveZhijian { player, span } => {
                self.mark(*span, "resolve_zhijian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_512);
            }
            StmtIr::EffectResolveGuzheng {
                player,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_guzheng");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUZHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_513);
            }
            StmtIr::EffectResolveZhiba {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_zhiba");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIBA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_523);
            }
            StmtIr::EffectResolveDrltCongjian { player, span } => {
                self.mark(*span, "resolve_drlt_congjian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_CONGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_524);
            }
            StmtIr::EffectResolveNzryFeijun { player, span } => {
                self.mark(*span, "resolve_nzry_feijun");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NZRY_FEIJUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_526);
            }
            StmtIr::EffectResolveDrltYongsi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_drlt_yongsi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_YONGSI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_527);
            }
            StmtIr::EffectResolveDrltWeidi { player, span } => {
                self.mark(*span, "resolve_drlt_weidi");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_WEIDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_528);
            }
            StmtIr::EffectResolveDcyingtu { player, span } => {
                self.mark(*span, "resolve_dcyingtu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCYINGTU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_487);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
