use super::*;

impl Assembler {
    pub(super) fn emit_effect_resolve_wdb_yangyi(
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
            StmtIr::EffectGuozhanUseZhulianBihe { player, span } => {
                self.mark(*span, "guozhan_use_zhulian_bihe");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GUOZHAN_USE_ZHULIAN_BIHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_641);
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
            StmtIr::EffectCancelIncomingCardUse {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "cancel_incoming_card_use");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CANCEL_INCOMING_CARD_USE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_643);
            }
            StmtIr::EffectChooseOtherGainOneOfTwoSkills {
                player,
                target,
                first_def,
                second_def,
                span,
            } => {
                self.mark(*span, "choose_other_gain_one_of_two_skills");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*first_def));
                self.ops.push(Op::ConstU32(*second_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_644);
            }
            StmtIr::EffectGwyewuRandomDiscardChain {
                player,
                wuyao_def,
                lang_def,
                span,
            } => {
                self.mark(*span, "gwyewu_random_discard_chain");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*wuyao_def));
                self.ops.push(Op::ConstU32(*lang_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GWYEWU_RANDOM_DISCARD_CHAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_645);
            }
            StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_then_target_may_use_draw_owner");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_497);
            }
            StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_category_then_target_give_matching_or_damage_plus_one",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_498);
            }
            StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip",
                );
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_499);
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
            StmtIr::EffectSkipNextDiscardPhase { player, span } => {
                self.mark(*span, "skip_next_discard_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_DISCARD_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_471);
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
            StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "give_selected_then_target_choose_discard_same_suit_or_lose_hp",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_123);
            }
            StmtIr::EffectGainEquipmentFromDrawPile { player, span } => {
                self.mark(*span, "gain_equipment_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_124);
            }
            StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "target_choose_suit_gain_owner_hand_damage_if_mismatch",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_168);
            }
            StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage {
                player,
                source,
                span,
            } => {
                self.mark(*span, "judge_source_discard_two_hand_or_damage");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_171);
            }
            StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_recover_after_phase_total_two");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_172);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_replace_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_173);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_reduce_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_311);
            }
            StmtIr::EffectDistributeTopTwoCards { player, span } => {
                self.mark(*span, "distribute_top_two_cards");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_TOP_TWO_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_174);
            }
            StmtIr::EffectGiveAllHandRecoverToOne {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand_recover_to_one");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_125);
            }
            StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { player, span } => {
                self.mark(
                    *span,
                    "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_127);
            }
            StmtIr::EffectChooseGiveOneHandEquipOrRecover {
                owner,
                giver,
                target,
                span,
            } => {
                self.mark(*span, "choose_give_one_hand_equip_or_recover");
                self.emit_expr(owner)?;
                self.emit_expr(giver)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_129);
            }
            StmtIr::EffectRemoveSkillByDef {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "remove_skill_by_def");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::REMOVE_SKILL_BY_DEF));
            }
            StmtIr::EffectChooseDiscardHandAddSkillByColor {
                player,
                red_skill_def,
                black_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_add_skill_by_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*red_skill_def));
                self.ops.push(Op::ConstU32(*black_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR,
                ));
            }
            StmtIr::EffectRevealTopGainNonBasicOrSwapBasic { player, span } => {
                self.mark(*span, "reveal_top_gain_non_basic_or_swap_basic");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC,
                ));
            }
            StmtIr::EffectRevealTopGainIfColor {
                player,
                color_mask,
                span,
            } => {
                self.mark(*span, "reveal_top_gain_if_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*color_mask)));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_TOP_GAIN_IF_COLOR));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_352);
            }
            StmtIr::EffectPreventCurrentDamage { span } => {
                self.mark(*span, "prevent_current_damage");
                self.ops
                    .push(Op::HostEffect(effect_tag::PREVENT_CURRENT_DAMAGE));
            }
            StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "judge_cancel_current_sha_target_on_red");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED,
                ));
            }
            StmtIr::EffectChooseDrawOrRecover {
                player,
                draw_count,
                recover_count,
                allow_cancel,
                span,
            } => {
                self.mark(*span, "choose_draw_or_recover");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(recover_count)?;
                self.emit_expr(allow_cancel)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DRAW_OR_RECOVER));
            }
            StmtIr::EffectChooseUseVirtualSha {
                player,
                ignore_distance,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_sha");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*ignore_distance)));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_SHA));
            }
            StmtIr::EffectTakeHand { from, to, span } => {
                self.mark(*span, "take_hand");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::HostEffect(effect_tag::TAKE_HAND));
            }
            StmtIr::EffectLoseHp {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "lose_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::LOSE_HP));
            }
            StmtIr::EffectAddSkill {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::ADD_SKILL));
            }
            StmtIr::EffectAddBoundSkill {
                player,
                skill_def,
                target,
                span,
            } => {
                self.mark(*span, "add_bound_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::ADD_BOUND_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_130);
            }
            StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                player,
                skill,
                skill_def,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "replace_skill_with_bound_after_resolution_if_target_alive",
                );
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_132);
            }
            StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { player, span } => {
                self.mark(
                    *span,
                    "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_133);
            }
            StmtIr::EffectChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_recover");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_134);
            }
            StmtIr::EffectRemoveSkill { skill, span } => {
                self.mark(*span, "remove_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_SKILL));
            }
            StmtIr::EffectRefreshSkill { skill, span } => {
                self.mark(*span, "refresh_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REFRESH_SKILL));
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
