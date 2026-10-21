use super::*;

impl Assembler {
    pub(super) fn emit_effect_ban_active_until_turn_end(
        &mut self,
        s: &StmtIr,
    ) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectBanActiveUntilTurnEnd {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "ban_active_until_turn_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::BAN_ACTIVE_UNTIL_TURN_END));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_400);
            }
            StmtIr::EffectSpwanweiActive {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_active");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_ACTIVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
            StmtIr::EffectSpwanweiDying {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_dying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_DYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
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
            StmtIr::EffectEquipWeaponFromDrawPile { player, span } => {
                self.mark(*span, "equip_weapon_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EQUIP_WEAPON_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_366);
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
            StmtIr::EffectSetSkillContentFlag {
                player,
                bit,
                value,
                span,
            } => {
                self.mark(*span, "set_skill_content_flag");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*bit)));
                self.ops.push(Op::ConstI32(i32::from(*value)));
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_SKILL_CONTENT_FLAG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
            }
            StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_reclaim_equipment_draw_if_range_reduced",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_97);
            }
            StmtIr::EffectChooseUseVirtualCard {
                player,
                kind,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_card");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_98);
            }
            StmtIr::EffectChooseUseVirtualNanman {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_nanman");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_NANMAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
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
            StmtIr::EffectDiscardAllOwnJudge {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_all_own_judge");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_OWN_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectStoreEventCardInSkillPile {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "store_event_card_in_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
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
            StmtIr::EffectChooseTargetGainMaxHp {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_target_gain_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_GAIN_MAX_HP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_646);
            }
            StmtIr::EffectDiscardBasicDrawEqual {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_basic_draw_equal");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_BASIC_DRAW_EQUAL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_657);
            }
            StmtIr::EffectStartDebateAllOthers {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_all_others");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_ALL_OTHERS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_655);
            }
            StmtIr::EffectStartDebateTargets {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_targets");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_656);
            }
            StmtIr::EffectPinDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "pin_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PIN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_658);
            }
            StmtIr::EffectAlignDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "align_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::ALIGN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_659);
            }
            StmtIr::EffectGainShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "gain_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_660);
            }
            StmtIr::EffectChangeGroup {
                player,
                group,
                skill,
                span,
            } => {
                self.mark(*span, "change_group");
                self.emit_expr(player)?;
                let code = match group.as_str() {
                    "wei" => 0,
                    "shu" => 1,
                    "wu" => 2,
                    "qun" => 3,
                    "jin" => 4,
                    _ => 0,
                };
                self.ops.push(Op::ConstI32(code));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CHANGE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_661);
            }
            StmtIr::EffectDistributeShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "distribute_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_662);
            }
            StmtIr::EffectRegisterTurnEndDebate {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "register_turn_end_debate");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REGISTER_TURN_END_DEBATE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_663);
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
            StmtIr::EffectChooseGroup {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_group");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CHOOSE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_664);
            }
            StmtIr::EffectChooseGainOtherSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_gain_other_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_OTHER_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_675);
            }
            StmtIr::EffectLoseGainedSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "lose_gained_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::LOSE_GAINED_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_676);
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
            StmtIr::EffectPlaceXumou {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "place_xumou");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::PLACE_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_665);
            }
            StmtIr::EffectPlaceDeckTopXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "place_deck_top_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectChoosePlaceHandXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_place_hand_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_PLACE_HAND_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectDiscardOneOwnXumou {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_one_own_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ONE_OWN_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectPlaceDeckTopExpansion {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "place_deck_top_expansion");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_667);
            }
            StmtIr::EffectCityConsume {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "city_consume");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CITY_CONSUME));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_668);
            }
            StmtIr::EffectQueueExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QUEUE_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_669);
            }
            StmtIr::EffectChooseOthersInsertExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_insert_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_INSERT_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_671);
            }
            StmtIr::EffectQueueExtraPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::QUEUE_EXTRA_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_672);
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
            StmtIr::EffectSkipRemainingPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "skip_remaining_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_REMAINING_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_674);
            }
            StmtIr::EffectChooseOthersAddMark {
                player,
                mark,
                delta,
                max_count,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops.push(Op::ConstU32(*max_count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_670);
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
            StmtIr::EffectChooseOtherAddMark {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "choose_other_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_652);
            }
            _ => unreachable!("statement routed to a different emitter"),
        }
        Ok(())
    }
}
