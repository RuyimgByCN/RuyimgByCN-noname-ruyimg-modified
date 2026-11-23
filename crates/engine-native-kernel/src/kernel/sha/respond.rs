use super::*;

impl ShaResolveMachine {
    pub(super) fn ask_respond_shan_or_hit(
        &mut self,
        ctx: &mut Ctx<'_>,
        direct_hit_in: bool,
    ) -> Flow {
        let tieji_blocks = self.tieji_red == Some(true)
            || ctx
                .core
                .vm_aux
                .flags
                .get(&(self.target, "mbcmfangzhu_ban".to_string()))
                .copied()
                .unwrap_or(0)
                > 0;
        let target_sentinel = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
            .map(|def| {
                ctx.core.has_skill(self.target, def)
                    && ctx.core.player(self.target).baolei_mode == 1
            })
            .unwrap_or(false);
        let target_can_play = !tieji_blocks
            && !target_sentinel
            && !ctx.catalog.blocks_card_use_and_save(ctx.core, self.target);
        if !direct_hit_in {
            crate::kernel::maybe_trigger_longxi_guarded(
                ctx,
                self.target,
                1,
                "xj_longxi_sha_guard",
            );
        }
        let mut shans: Vec<ChoiceOption> = if target_can_play {
            collect_shan_response_cards(ctx.core, ctx.catalog, self.target)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect()
        } else {
            Vec::new()
        };
        let chihe_block_mask = ctx
            .core
            .vm_aux
            .flags
            .get(&(self.target, "l10_scschihe_block_suits".to_string()))
            .copied()
            .unwrap_or(0);
        if chihe_block_mask != 0 {
            shans.retain(|o| match o {
                ChoiceOption::Card(c) => {
                    let bit = match ctx.catalog.effective_card_suit(ctx.core, self.target, *c) {
                        crate::catalog::Suit::Spade => 1 << 0,
                        crate::catalog::Suit::Heart => 1 << 1,
                        crate::catalog::Suit::Club => 1 << 2,
                        crate::catalog::Suit::Diamond => 1 << 3,
                    };
                    (chihe_block_mask & bit) == 0
                }
                _ => true,
            });
        }
        let req_suit_tag = if let Some(cid) = self.card {
            if ctx
                .core
                .vm_aux
                .flags
                .get(&(self.source, format!("xj_xiaoyue_card:{cid}")))
                .copied()
                .unwrap_or(0)
                > 0
            {
                2 // Suit::Heart only (`xiaoyue2`)
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .get(&(self.source, format!("xj_yanzhan_card:{cid}")))
                    .copied()
                    .unwrap_or(0)
            }
        } else {
            0
        };
        if req_suit_tag != 0 {
            ctx.core
                .vm_aux
                .flags
                .insert((self.target, "xj_sha_req_suit".to_string()), req_suit_tag);
            shans.retain(|o| match o {
                ChoiceOption::Card(c) => {
                    let stag = match ctx.catalog.effective_card_suit(ctx.core, self.target, *c) {
                        crate::catalog::Suit::Spade => 1,
                        crate::catalog::Suit::Heart => 2,
                        crate::catalog::Suit::Club => 3,
                        crate::catalog::Suit::Diamond => 4,
                    };
                    stag == req_suit_tag
                }
                _ => true,
            });
        }
        let guilong_direct_hit =
            crate::rvm_bridge::modifiers::d34_l28_mods::is_guilong_red_sha(
                ctx.core,
                ctx.catalog,
                self.source,
                self.card,
            );
        let jianwu_direct_hit = ctx
            .catalog
            .skill_by_name("jianwu")
            .is_some_and(|d| {
                ctx.core.has_skill(self.source, d)
                    && !ctx.catalog.is_tieji_sealed(ctx.core, self.source)
            })
            && ctx.core.computed_distance(ctx.catalog, self.target, self.source)
                > ctx.core.attack_range(ctx.catalog, self.target);
        let direct_hit = direct_hit_in || guilong_direct_hit || jianwu_direct_hit;
        let shengong_borrow = (ctx.catalog.hooks.borrow_options)(ctx, self.target, true);
        let sipu_ban_respond = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
            .map(|def| {
                ctx.core.has_skill(self.source, def)
                    && ctx.core.turn.current == self.source
                    && ctx.core.turn.cards_used_play_phase < 3
            })
            .unwrap_or(false);
        let skill_shans =
            self.collect_u01_respond_skill_options(ctx, target_can_play && !sipu_ban_respond);
        let resp_skills = if target_can_play && !sipu_ban_respond && !direct_hit {
            super::super::host_response_skills(ctx.core, ctx.catalog, self.target, 1)
        } else {
            Vec::new()
        };
        if direct_hit
            || (shans.is_empty()
                && shengong_borrow.is_empty()
                && resp_skills.is_empty()
                && skill_shans.is_empty())
            || !ctx.core.player(self.target).alive
        {
            return self.sha_hit_pre_damage(ctx);
        }
        let mut options = vec![ChoiceOption::Pass];
        if !sipu_ban_respond && target_can_play {
            options.extend(shans);
            options.extend(shengong_borrow);
            for rdef in resp_skills {
                options.push(ChoiceOption::Skill(rdef));
            }
            for opt in skill_shans {
                if !options.contains(&opt) {
                    options.push(opt);
                }
            }
        }
        self.asked = true;
        Flow::WaitChoice {
            observer: self.target,
            spec: ChoiceSpec {
                kind: ChoiceKind::RespondCard,
                prompt: "respond_shan",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }

    pub(super) fn sha_hit_pre_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let fid = self
            .current_sha_frame_id(ctx)
            .unwrap_or(crate::resolution::NO_FRAME);
        if (ctx.catalog.hooks.sha_hit_step)(
            ctx,
            fid,
            self.source,
            self.target,
            self.card,
        ) {
            self.asked = false;
            self.hujia_holder = None;
            self.liuli_stage = 0;
            ctx.core
                .vm_aux
                .flags
                .insert((self.source, format!("u05_sha_hit_resume:{fid}")), 1);
            return Flow::Yield;
        }
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(self.source, format!("u05_guiyanfadao_unhurt:{fid}:{}", self.target)))
            .is_some()
        {
            return self.finish_or_defer_sha_end(ctx, false);
        }
        if !self.asking_fengxueren
            && ctx.core.player(self.target).alive
            && !ctx.core.player(self.target).face_down
            && ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.equip_card_has_kind(ctx.core, *c, CardKind::Fengxueren))
        {
            self.asking_fengxueren = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "fengxueren_flip_confirm",
                    options: vec![ChoiceOption::Confirm],
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        if self.hanbing_can_fire(ctx) {
            self.asking_hanbing = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "hanbing_prevent_confirm",
                    options: vec![ChoiceOption::Confirm],
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        let horses = self.qilin_horses(ctx);
        if !horses.is_empty() && ctx.core.player(self.target).alive {
            self.asking_qilin = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qilin_discard_horse",
                    options: horses,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        self.emit_sha_damage(ctx)
    }

    pub(super) fn resolve_shan_card(&mut self, ctx: &mut Ctx<'_>, shan: crate::ids::CardId) -> Flow {
        if ctx.core.player(self.target).hand.contains(&shan)
            || ctx.core.player(self.target).equips.contains(&shan)
        {
            discard_response_card(ctx, self.target, shan, CardKind::Shan);
        } else if let Some(owner) = (ctx.catalog.hooks.borrowed_owner)(ctx, self.target, shan) {
            ctx.core.player_mut(owner).equips.retain(|x| *x != shan);
            revoke_equip_skills(ctx.core, ctx.catalog, owner, shan);
            ctx.to_discard(shan);
            ctx.log(LogEvent::SkillTriggered {
                skill: "神弓",
                owner: self.target,
                presentation: None,
            });
        } else {
            ctx.core.table.retain(|x| *x != shan);
            ctx.to_discard(shan);
        }
        note_longdan_sha_source(ctx, self.target, self.source);
        if ctx.catalog.vm_card_history {
            if let Some(fid) = self.current_sha_frame_id(ctx) {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.target, format!("l11_sha_dodge_ok:{fid}:{shan}")), 1);
            }
        }
        let base_depth = ctx.res.stack.len();
        crate::kernel::emit_card_responded_as(ctx, self.target, shan, CardKind::Shan);
        let has_final = (ctx.catalog.hooks.reguhuo_has_pending_final)(ctx.core, self.target);
        if has_final {
            (ctx.catalog.hooks.reguhuo_spawn_final)(ctx, self.target, base_depth);
        }
        self.asked = false;
        if let Some(fid) = self.current_sha_frame_id(ctx) {
            ctx.core
                .vm_aux
                .flags
                .insert((self.target, format!("sha_u01_shan_pending:{fid}")), 1);
            return Flow::Yield;
        }
        releiji_on_shan(ctx, self.target);
        self.dodges_used += 1;
        let has_anruo = ctx
            .core
            .vm_aux
            .flags
            .contains_key(&(self.target, format!("l10_scsanruo_effect:{shan}")));
        if has_final || has_anruo {
            ctx.core
                .vm_aux
                .flags
                .insert((self.target, "l10_reguhuo_sha_post_dodge".to_string()), 1);
            return Flow::Yield;
        }
        self.post_dodge_step(ctx)
    }

    pub(super) fn post_dodge_step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let needed = self.needed_shan_count(ctx);
        if self.dodges_used < needed {
            self.hujia_queue = None;
            self.hujia_holder = None;
            self.asked = false;
            return Flow::Yield;
        }
        let has_qinglong = !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.source)
            && !crate::kernel::mbcmfangzhu_banned(ctx.core, self.source)
            && ctx
                .core
                .has_active_equip_kind(ctx.catalog, self.source, CardKind::Qinglong);
        let hand_shas: Vec<ChoiceOption> = ctx
            .core
            .player(self.source)
            .hand
            .iter()
            .filter(|c| {
                matches!(
                    ctx.catalog.resolved_card_kind(ctx.core, **c),
                    CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                )
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        if has_qinglong && !hand_shas.is_empty() && !self.asking_qinglong {
            self.asking_qinglong = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qinglong_replay_sha",
                    options: hand_shas,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        let has_guanshi_override = ctx
            .catalog
            .skill_by_name("guanshi_skill")
            .is_some_and(|d| ctx.core.has_skill(self.source, d));
        let has_guanshi = !has_guanshi_override
            && !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.source)
            && ctx
                .core
                .has_active_equip_kind(ctx.catalog, self.source, CardKind::Guanshi);
        if has_guanshi && ctx.core.player(self.source).alive {
            let mut pool: Vec<ChoiceOption> = ctx
                .core
                .player(self.source)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            pool.extend(
                ctx.core
                    .player(self.source)
                    .equips
                    .iter()
                    .filter(|c| {
                        ctx.core.is_gw_dieyi_masked(ctx.catalog, self.source, **c)
                            || !ctx.catalog.equip_card_has_kind(ctx.core, **c, CardKind::Guanshi)
                    })
                    .copied()
                    .map(ChoiceOption::Card),
            );
            let need = crate::kernel::sha::resume::guanshi_need(ctx, self.source);
            if pool.len() >= need {
                self.asking_guanshi = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "guanshi_force_damage",
                        options: pool,
                        min: need,
                        max: need,
                        allow_cancel: true,
                    },
                };
            }
        }
        if self.dodges_used < ctx.core.turn.liaoyuan_count {
            return self.sha_hit_pre_damage(ctx);
        }
        if let Some(flow) = self.try_open_sadeng_on_miss(ctx) {
            return flow;
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "dddfusi_window_refused".to_string()));
        ctx.core.turn.liaoyuan_count = 0;
        self.after_sha_dodged(ctx)
    }

    pub(super) fn try_open_sadeng_on_miss(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        let has_equip = !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.source)
            && ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sadengjinhuan);
        let has_skill = ctx
            .catalog
            .skill_by_name("sadengjinhuan")
            .is_some_and(|d| ctx.core.has_skill(self.source, d));
        if !(has_equip || has_skill)
            || !ctx.core.player(self.source).alive
            || !ctx.core.player(self.target).alive
            || self.asking_sadeng
        {
            return None;
        }
        let fid = self
            .current_sha_frame_id(ctx)
            .unwrap_or(crate::resolution::NO_FRAME);
        let tried_key = format!("u05_sadeng_tried:{fid}:{}", self.target);
        if ctx
            .core
            .vm_aux
            .flags
            .contains_key(&(self.source, tried_key.clone()))
        {
            return None;
        }
        ctx.core.vm_aux.flags.insert((self.source, tried_key), 1);
        ctx.core.vm_aux.flags.insert(
            (self.source, format!("u05_sadeng_handled:{}", self.target)),
            1,
        );
        self.asking_sadeng = true;
        Some(Flow::WaitChoice {
            observer: self.source,
            spec: ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "sadengjinhuan_judge_confirm",
                options: vec![ChoiceOption::Confirm],
                min: 0,
                max: 1,
                allow_cancel: true,
            },
        })
    }
}

/// Collects all cards `target` can use to respond with `Shan` (hand + viewAs + Yuchanqian + equips).
pub fn collect_shan_response_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    target: PlayerId,
) -> Vec<crate::ids::CardId> {
    let mbcm_banned = core
        .vm_aux
        .flags
        .get(&(target, "mbcmfangzhu_ban".to_string()))
        .copied()
        .unwrap_or(0)
        > 0;
    let target_sentinel = catalog
        .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
        .is_some_and(|def| core.has_skill(target, def) && core.player(target).baolei_mode == 1);
    if mbcm_banned || target_sentinel || catalog.blocks_card_use_and_save(core, target) {
        return Vec::new();
    }
    let hand_locked =
        crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, target);
    let target_hand_blocked = hand_locked || catalog.blocks_hand_cards(core, target);
    let target_sealed = catalog.is_tieji_sealed(core, target);
    let longdan = !target_sealed
        && !catalog.is_rechanyuan_sealed_name(core, target, crate::vocabulary::names::LONG_DAN)
        && catalog
            .skill_by_name(crate::vocabulary::names::LONG_DAN)
            .is_some_and(|d| core.has_skill(target, d));
    let qingguo = !target_sealed
        && !catalog.is_rechanyuan_sealed_name(core, target, crate::vocabulary::names::QING_GUO)
        && catalog
            .skill_by_name(crate::vocabulary::names::QING_GUO)
            .is_some_and(|d| core.has_skill(target, d));
    let rw_lanyin = crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_lanyinjia(
        core, catalog, target,
    );
    let hufu_shan = catalog
        .skill_by_name("g_hufu_shan")
        .is_some_and(|d| core.has_skill(target, d));
    let mut shans: Vec<crate::ids::CardId> = if target_hand_blocked {
        Vec::new()
    } else {
        core.player(target)
            .hand
            .iter()
            .copied()
            .filter(|&c| {
                let kind = catalog.resolved_card_kind(core, c);
                let is_shan_like = kind == CardKind::Shan
                    || kind.yuchan_auto_view_as() == Some(CardKind::Shan);
                let is_sha_like = kind.is_sha_like()
                    || kind
                        .yuchan_auto_view_as()
                        .is_some_and(|k| k.is_sha_like());
                rw_lanyin
                    || is_shan_like
                    || (longdan && is_sha_like)
                    || (hufu_shan && kind == CardKind::Hufu)
            })
            .collect()
    };
    if qingguo && !target_hand_blocked {
        for &c in &core.player(target).hand {
            let black = matches!(
                catalog.effective_card_suit(core, target, c),
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            );
            if black && !shans.contains(&c) {
                shans.push(c);
            }
        }
    }
    if !target_hand_blocked {
        for &c in &core.player(target).hand {
            if catalog
                .find_player_vm_view_as(core, target, c, false, Some(CardKind::Shan))
                .is_some()
                && !shans.contains(&c)
            {
                shans.push(c);
            }
        }
    }
    for c in catalog.player_vm_view_as_extra_cards(core, target, false, Some(CardKind::Shan)) {
        if (!hand_locked || !core.player(target).hand.contains(&c)) && !shans.contains(&c) {
            shans.push(c);
        }
    }
    shans.retain(|c| {
        !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
            core, catalog, target, *c,
        )
    });
    shans
}

pub(super) fn note_longdan_sha_source(ctx: &mut Ctx<'_>, responder: PlayerId, source: PlayerId) {
    let has_longdan = ["fz_new_longdan", "zylongdan"].iter().any(|name| {
        ctx.catalog
            .skill_by_name(name)
            .is_some_and(|d| ctx.core.has_skill(responder, d))
    });
    if has_longdan {
        ctx.core
            .vm_aux
            .flags
            .insert((responder, "responding_to_sha_from".to_string()), u32::from(source) + 1);
    }
}
