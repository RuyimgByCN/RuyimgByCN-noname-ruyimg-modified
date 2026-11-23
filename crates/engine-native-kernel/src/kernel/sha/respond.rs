use super::*;

impl ShaResolveMachine {
    pub(super) fn ask_respond_shan_or_hit(
        &mut self,
        ctx: &mut Ctx<'_>,
        direct_hit_in: bool,
    ) -> Flow {
        let target_sealed = ctx.catalog.is_tieji_sealed(ctx.core, self.target);
        let longdan = !target_sealed
            && !ctx.catalog.is_rechanyuan_sealed_name(
                ctx.core,
                self.target,
                crate::vocabulary::names::LONG_DAN,
            )
            && ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::LONG_DAN)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
        let tieji_blocks = self.tieji_red == Some(true)
            || ctx
                .core
                .vm_aux
                .flags
                .get(&(self.target, "mbcmfangzhu_ban".to_string()))
                .copied()
                .unwrap_or(0)
                > 0;
        let qingguo = !target_sealed
            && !ctx.catalog.is_rechanyuan_sealed_name(
                ctx.core,
                self.target,
                crate::vocabulary::names::QING_GUO,
            )
            && ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::QING_GUO)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
        let target_sentinel = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
            .map(|def| {
                ctx.core.has_skill(self.target, def)
                    && ctx.core.player(self.target).baolei_mode == 1
            })
            .unwrap_or(false);
        let hand_locked = crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(
            ctx.core,
            ctx.catalog,
            self.target,
        );
        let target_can_play = !tieji_blocks
            && !target_sentinel
            && !ctx.catalog.blocks_all_cards(ctx.core, self.target);
        let target_can_play_hand = target_can_play && !hand_locked;
        let rw_lanyin = target_can_play_hand
            && crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_lanyinjia(
                ctx.core,
                ctx.catalog,
                self.target,
            );
        let mut shans: Vec<ChoiceOption> = ctx
            .core
            .player(self.target)
            .hand
            .iter()
            .filter(|c| {
                target_can_play_hand && {
                    let kind = ctx.catalog.resolved_card_kind(ctx.core, **c);
                    rw_lanyin || kind == CardKind::Shan || (longdan && kind == CardKind::Sha)
                }
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        if qingguo && target_can_play_hand {
            for c in ctx.core.player(self.target).hand.iter().copied() {
                let kind = ctx.catalog.resolved_card_kind(ctx.core, c);
                let already =
                    rw_lanyin || kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                let black = matches!(
                    ctx.catalog.effective_card_suit(ctx.core, self.target, c),
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if black && !already {
                    shans.push(ChoiceOption::Card(c));
                }
            }
        }
        if target_can_play {
            for &c in &ctx.core.player(self.target).hand {
                if !hand_locked
                    && ctx
                        .catalog
                        .find_player_vm_view_as(
                            ctx.core,
                            self.target,
                            c,
                            false,
                            Some(CardKind::Shan),
                        )
                        .is_some()
                    && !shans.contains(&ChoiceOption::Card(c))
                {
                    shans.push(ChoiceOption::Card(c));
                }
            }
            for c in ctx.catalog.player_vm_view_as_extra_cards(
                ctx.core,
                self.target,
                false,
                Some(CardKind::Shan),
            ) {
                if (!hand_locked || !ctx.core.player(self.target).hand.contains(&c))
                    && !shans.contains(&ChoiceOption::Card(c))
                {
                    shans.push(ChoiceOption::Card(c));
                }
            }
        }
        shans.retain(|o| {
            !matches!(o, ChoiceOption::Card(c)
                if crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
                    ctx.core,
                    ctx.catalog,
                    self.target,
                    *c,
                ))
        });
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
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.equip_card_has_kind(ctx.core, *c, CardKind::Qinglong));
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
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.equip_card_has_kind(ctx.core, *c, CardKind::Guanshi));
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
                    .filter(|c| !ctx.catalog.equip_card_has_kind(ctx.core, **c, CardKind::Guanshi))
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
        let has_sadeng = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.equip_card_has_kind(ctx.core, *c, CardKind::Sadengjinhuan));
        if has_sadeng && ctx.core.player(self.source).alive && !self.asking_sadeng {
            self.asking_sadeng = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "sadengjinhuan_judge_confirm",
                    options: vec![ChoiceOption::Confirm],
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        if self.dodges_used < ctx.core.turn.liaoyuan_count {
            return self.emit_sha_damage(ctx);
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "dddfusi_window_refused".to_string()));
        ctx.core.turn.liaoyuan_count = 0;
        self.after_sha_dodged(ctx)
    }
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
