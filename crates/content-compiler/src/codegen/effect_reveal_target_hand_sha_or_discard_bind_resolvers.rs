//! 效果语句代码生成：具名技能结算器（EffectResolve*）。

use super::*;

impl Assembler {
    pub(super) fn emit_effect_reveal_target_hand_sha_or_discard_bind_resolvers(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectResolveHiddenSkillPile {
                player,
                pile_skill_def,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_hidden_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_238);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*pile_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_HIDDEN_SKILL_PILE));
            }
            StmtIr::EffectResolveDiliYuheng {
                player,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_dili_yuheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_240);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DILI_YUHENG));
            }
            StmtIr::EffectResolveGuanchaoRank { player, card, span } => {
                self.mark(*span, "resolve_guanchao_rank");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_245);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GUANCHAO_RANK));
            }
            StmtIr::EffectResolveGuanwei {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_guanwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_246);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUANWEI));
            }
            StmtIr::EffectResolveLanjiang { player, span } => {
                self.mark(*span, "resolve_lanjiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_247);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LANJIANG));
            }
            StmtIr::EffectResolveRebingyi { player, span } => {
                self.mark(*span, "resolve_rebingyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_248);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBINGYI));
            }
            StmtIr::EffectResolveRebotu { player, span } => {
                self.mark(*span, "resolve_rebotu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_249);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBOTU));
            }
            StmtIr::EffectResolveOldimeng {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_oldimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_250);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDIMENG));
            }
            StmtIr::EffectResolveRejieyin { player, span } => {
                self.mark(*span, "resolve_rejieyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_251);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIEYIN));
            }
            StmtIr::EffectResolveReanxu { player, span } => {
                self.mark(*span, "resolve_reanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_252);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REANXU));
            }
            StmtIr::EffectResolveDrltJueyan {
                player,
                sha_skill_def,
                weapon_skill_def,
                mount_skill_def,
                armor_skill_def,
                treasure_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jueyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_253);
                self.emit_expr(player)?;
                for skill_def in [
                    sha_skill_def,
                    weapon_skill_def,
                    mount_skill_def,
                    armor_skill_def,
                    treasure_skill_def,
                ] {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JUEYAN));
            }
            StmtIr::EffectResolveRejizhi { player, span } => {
                self.mark(*span, "resolve_rejizhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_254);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIZHI));
            }
            StmtIr::EffectResolveJiexun {
                player,
                funan_helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_jiexun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_255);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*funan_helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIEXUN));
            }
            StmtIr::EffectResolveWenguaSetup {
                player,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua_setup");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_256);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WENGUA_SETUP));
            }
            StmtIr::EffectResolveWengua {
                player,
                main_skill_def,
                marker_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_257);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*main_skill_def as i32));
                self.ops.push(Op::ConstI32(*marker_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WENGUA));
            }
            StmtIr::EffectResolveXinanguo { player, span } => {
                self.mark(*span, "resolve_xinanguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_258);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XINANGUO));
            }
            StmtIr::EffectResolveGanglieThree { player, span } => {
                self.mark(*span, "resolve_ganglie_three");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_259);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GANGLIE_THREE));
            }
            StmtIr::EffectResolveZhongyi {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_zhongyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_260);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYI));
            }
            StmtIr::EffectResolveOldAnxu { player, span } => {
                self.mark(*span, "resolve_oldanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_262);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDANXU));
            }
            StmtIr::EffectResolveOldQinqing { player, span } => {
                self.mark(*span, "resolve_oldqinqing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_265);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDQINQING));
            }
            StmtIr::EffectResolveOldMingjian { player, span } => {
                self.mark(*span, "resolve_oldmingjian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_266);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDMINGJIAN));
            }
            StmtIr::EffectResolveOldXuanfeng { player, span } => {
                self.mark(*span, "resolve_oldxuanfeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_267);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDXUANFENG));
            }
            StmtIr::EffectResolveJianrou { player, span } => {
                self.mark(*span, "resolve_jianrou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_268);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANROU));
            }
            StmtIr::EffectResolveOldHuisheng { player, span } => {
                self.mark(*span, "resolve_oldhuisheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_269);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDHUISHENG));
            }
            StmtIr::EffectResolveOldJiefan {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_oldjiefan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_270);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDJIEFAN));
            }
            StmtIr::EffectResolveChegu { player, span } => {
                self.mark(*span, "resolve_chegu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_271);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHEGU));
            }
            StmtIr::EffectResolveJunkguixin { player, span } => {
                self.mark(*span, "resolve_junkguixin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_272);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JUNKGUIXIN));
            }
            StmtIr::EffectResolveYaoChenwei { player, span } => {
                self.mark(*span, "resolve_yao_chenwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_275);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_YAO_CHENWEI));
            }
            StmtIr::EffectResolveMassDamageDiscardTurnOver {
                player,
                mark_def,
                mark_count,
                damage_amount,
                hand_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*hand_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER,
                ));
            }
            StmtIr::EffectResolveMassDamageDiscardHandEquip {
                player,
                mark_def,
                mark_count,
                damage_amount,
                discard_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_235);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*discard_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP,
                ));
            }
            StmtIr::EffectResolveMarkParityControl {
                player,
                mark_def,
                mass_threshold,
                damage_amount,
                span,
            } => {
                self.mark(*span, "resolve_mark_parity_control");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mass_threshold as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MARK_PARITY_CONTROL));
            }
            StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                span,
            } => {
                self.mark(
                    *span,
                    "resolve_selected_equipment_discard_then_choose_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(i32::from(*nature)));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE,
                ));
            }
            StmtIr::EffectResolveYeyan { player, span } => {
                self.mark(*span, "resolve_yeyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_210);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YEYAN));
            }
            StmtIr::EffectResolveQixing {
                player,
                initialize,
                span,
            } => {
                self.mark(*span, "resolve_qixing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*initialize)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIXING));
            }
            StmtIr::EffectResolveQixingWeather {
                player,
                qixing_skill_def,
                helper_skill_def,
                max_targets,
                span,
            } => {
                self.mark(*span, "resolve_qixing_weather");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qixing_skill_def as i32));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*max_targets)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_QIXING_WEATHER));
            }
            StmtIr::EffectResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_nzry_longnu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*fire_skill_def as i32));
                self.ops.push(Op::ConstI32(*thunder_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NZRY_LONGNU));
            }
            StmtIr::EffectResolveDrltPoxi {
                player,
                target,
                hand_limit_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_poxi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_214);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*hand_limit_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DRLT_POXI));
            }
            StmtIr::EffectResolveDrltJieying {
                player,
                event_player,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jieying");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_215);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(*mark as i32));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JIEYING));
            }
            StmtIr::EffectResolveDrltDuorui {
                player,
                target,
                helper_skill_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_drlt_duorui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_216);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_DUORUI));
            }
            StmtIr::EffectResolveDrltZhiti {
                player,
                event_player,
                event_source,
                event_amount,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_zhiti");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_217);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_source)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_ZHITI));
            }
            StmtIr::EffectResolveShenSimayi {
                player,
                event_player,
                event_amount,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_shen_simayi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_218);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHEN_SIMAYI));
            }
            StmtIr::EffectResolveBeige {
                player,
                target,
                source,
                span,
            } => {
                self.mark(*span, "resolve_beige");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_219);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BEIGE));
            }
            StmtIr::EffectResolveBeigeConfigured {
                player,
                target,
                source,
                heart_recover_amount,
                diamond_draw_count,
                span,
            } => {
                self.mark(*span, "resolve_beige_configured");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_296);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(heart_recover_amount)?;
                self.emit_expr(diamond_draw_count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BEIGE_CONFIGURED));
            }
            StmtIr::EffectResolveZhuiyi {
                player,
                killer,
                span,
            } => {
                self.mark(*span, "resolve_zhuiyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_297);
                self.emit_expr(player)?;
                self.emit_expr(killer)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUIYI));
            }
            StmtIr::EffectResolveDecadezhenjun { player, span } => {
                self.mark(*span, "resolve_decadezhenjun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_298);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DECADEZHENJUN));
            }
            StmtIr::EffectResolveSpXiaoguo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_sp_xiaoguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_299);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SP_XIAOGUO));
            }
            StmtIr::EffectResolveDingpan {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dingpan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_302);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGPAN));
            }
            StmtIr::EffectResolveTanlu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_tanlu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_303);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TANLU));
            }
            StmtIr::EffectResolveDcqiaomeng {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcqiaomeng");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCQIAOMENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_313);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
