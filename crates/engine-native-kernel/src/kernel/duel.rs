use super::*;

/// 决斗：目标先出杀，双方交替出杀；轮到某方其无杀（不询问）或放弃时，
/// 该方受对方造成的 1 点伤害，决斗结束。响应杀不占出牌阶段限一次；
/// 无距离限制——轮转起手/跳过语义/来源归属见覆盖清单行为差异表。
#[derive(Clone)]
pub struct DuelMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    pub(super) target: PlayerId,
    /// true = 轮到目标出杀；false = 轮到来源出杀。
    pub(super) on_target: bool,
    pub(super) asked: bool,
    /// 无双：对手需再次出杀（本轮已出一张）。
    pub(super) second_pending: bool,
}

impl DuelMachine {
    pub fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
        on_target: bool,
        asked: bool,
        second_pending: bool,
    ) -> Self {
        Self {
            card,
            source,
            target,
            on_target,
            asked,
            second_pending,
        }
    }
}

impl crate::catalog::SkillMachine for DuelMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Duel {
            card: self.card,
            source: self.source,
            target: self.target,
            on_target: self.on_target,
            asked: self.asked,
            second_pending: self.second_pending,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            if has_u01_respond_park(ctx) {
                return Flow::Yield;
            }
            let responder = if self.on_target {
                self.target
            } else {
                self.source
            };
            let opponent = if self.on_target {
                self.source
            } else {
                self.target
            };
            if let Some(ok) = take_u01_respond_result(ctx, responder) {
                if ok {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(responder, "dddfusi_window_refused".to_string()));
                    let card_ovr = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(responder, "u01_respond_card_override".to_string()))
                        .and_then(|v| v.checked_sub(1));
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(responder, "u01_respond_fusi_owner".to_string()));
                    if let Some(c) = card_ovr {
                        super::emit_card_responded_as(ctx, responder, c, CardKind::Sha);
                    } else {
                        super::record_virtual_respond(ctx, responder, CardKind::Sha);
                    }
                    let wushuang_holder = if self.on_target {
                        self.source
                    } else {
                        self.target
                    };
                    let opponent_has_wushuang = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::WU_SHUANG)
                        .map(|d| ctx.core.has_skill(wushuang_holder, d))
                        .unwrap_or(false);
                    if opponent_has_wushuang && !self.second_pending {
                        self.second_pending = true;
                        continue;
                    }
                    self.second_pending = false;
                    self.on_target = !self.on_target;
                    continue;
                }
                self.asked = false;
            }
            let shas: Vec<crate::ids::CardId> =
                response_kind_cards(ctx, responder, CardKind::Sha);
            let resp_skills = host_response_skills(ctx.core, ctx.catalog, responder, 2);
            if ctx
                .core
                .vm_aux
                .flags
                .remove(&(responder, "l10_reguhuo_duel_resume".to_string()))
                .is_some()
            {
                self.asked = false;
                if let Some(c) = (ctx.catalog.hooks.reguhuo_consume_resp)(ctx.core, responder) {
                    discard_response_card(ctx, responder, c, CardKind::Sha);
                    let base_depth = ctx.res.stack.len();
                    ctx.emit(EventData::CardResponded {
                        card: c,
                        player: responder,
                    });
                    let wushuang_holder = if self.on_target {
                        self.source
                    } else {
                        self.target
                    };
                    let opponent_has_wushuang = !ctx.catalog.is_rechanyuan_sealed_name(
                        ctx.core,
                        wushuang_holder,
                        crate::vocabulary::names::WU_SHUANG,
                    ) && ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::WU_SHUANG)
                        .map(|d| ctx.core.has_skill(wushuang_holder, d))
                        .unwrap_or(false);
                    if opponent_has_wushuang && !self.second_pending {
                        self.second_pending = true;
                    } else {
                        self.second_pending = false;
                        self.on_target = !self.on_target;
                    }
                    let has_final =
                        (ctx.catalog.hooks.reguhuo_has_pending_final)(ctx.core, responder);
                    let has_anruo = ctx
                        .core
                        .vm_aux
                        .flags
                        .contains_key(&(responder, format!("l10_scsanruo_effect:{c}")));
                    if has_final || has_anruo {
                        if has_final {
                            (ctx.catalog.hooks.reguhuo_spawn_final)(ctx, responder, base_depth);
                        }
                        return Flow::Yield;
                    }
                    continue;
                }
            }
            let skill_opts = if !ctx.catalog.blocks_all_cards(ctx.core, responder) {
                (ctx.catalog.hooks.respond_skill_options)(
                    ctx.core,
                    ctx.catalog,
                    responder,
                    CardKind::Sha,
                    false,
                )
            } else {
                Vec::new()
            };
            if !self.asked {
                if shas.is_empty()
                    && resp_skills.is_empty()
                    && skill_opts.is_empty()
                    && ctx.core.player(responder).alive
                {
                    if let Some((helper, helper_shas)) =
                        jijiang_shu_helper(ctx.core, ctx.catalog, responder)
                    {
                        self.asked = true;
                        let mut options = vec![ChoiceOption::Pass];
                        options.extend(helper_shas.into_iter().map(ChoiceOption::Card));
                        return Flow::WaitChoice {
                            observer: helper,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::RespondCard,
                                prompt: "duel_sha",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                }
                if (shas.is_empty() && resp_skills.is_empty() && skill_opts.is_empty())
                    || !ctx.core.player(responder).alive
                {
                    // 轮到方无杀：不询问，直接受伤，决斗结束
                    self.emit_duel_damage(ctx, opponent, responder);
                    return self.finish(ctx);
                }
                self.asked = true;
                let mut options = vec![ChoiceOption::Pass];
                if !ctx.catalog.blocks_all_cards(ctx.core, responder) {
                    options.extend(shas.into_iter().map(ChoiceOption::Card));
                    // 神弓（swd 批 148）：借用他人武器作为杀
                    options.extend((ctx.catalog.hooks.borrow_options)(
                        ctx, responder, false,
                    ));
                    for rdef in resp_skills.iter().copied() {
                        options.push(ChoiceOption::Skill(rdef));
                    }
                    for opt in skill_opts {
                        if !options.contains(&opt) {
                            options.push(opt);
                        }
                    }
                }
                return Flow::WaitChoice {
                    observer: responder,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::RespondCard,
                        prompt: "duel_sha",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            // 应答消费
            self.asked = false;
            let answer = ctx.answer().clone();
            if let Some(ChoiceOption::Skill(rdef)) = answer.first().copied() {
                if resp_skills.contains(&rdef) {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((responder, "l10_reguhuo_duel_resume".to_string()), 1);
                    (ctx.catalog.hooks.reguhuo_spawn_response)(
                        ctx,
                        responder,
                        2,
                        0,
                        rdef,
                        opponent,
                    );
                    return Flow::Yield;
                }
                let fid = ctx.res.stack.last().map(|f| f.id).unwrap_or(0);
                if let Some(immediate) = (ctx.catalog.hooks.try_activate_respond_skill)(
                    ctx,
                    responder,
                    rdef,
                    CardKind::Sha,
                    fid,
                ) {
                    if immediate {
                        super::record_virtual_respond(ctx, responder, CardKind::Sha);
                        let wushuang_holder = if self.on_target {
                            self.source
                        } else {
                            self.target
                        };
                        let opponent_has_wushuang = !ctx.catalog.is_rechanyuan_sealed_name(
                            ctx.core,
                            wushuang_holder,
                            crate::vocabulary::names::WU_SHUANG,
                        ) && ctx
                            .catalog
                            .skill_by_name(crate::vocabulary::names::WU_SHUANG)
                            .map(|d| ctx.core.has_skill(wushuang_holder, d))
                            .unwrap_or(false);
                        if opponent_has_wushuang && !self.second_pending {
                            self.second_pending = true;
                            continue;
                        }
                        self.second_pending = false;
                        self.on_target = !self.on_target;
                        continue;
                    }
                    return Flow::Yield;
                }
            }
            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "dddfusi_window_refused".to_string()));
                let jijiang_helper = if shas.is_empty() {
                    jijiang_shu_helper(ctx.core, ctx.catalog, responder).map(|(h, _)| h)
                } else {
                    None
                };
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "duel_jijiang_skip".to_string()));
                let actual_holder = jijiang_helper.unwrap_or_else(|| {
                    if ctx.core.player(responder).hand.contains(&c)
                        || ctx.core.player(responder).equips.contains(&c)
                    {
                        responder
                    } else {
                        (0..ctx.core.players.len() as u8)
                            .find(|&p| {
                                ctx.core.player(p).hand.contains(&c)
                                    || ctx.core.player(p).equips.contains(&c)
                            })
                            .unwrap_or(responder)
                    }
                });
                discard_response_card(ctx, actual_holder, c, CardKind::Sha);
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "responding_to_sha_from".to_string()));
                super::emit_card_responded_as(ctx, responder, c, CardKind::Sha);
                // 无双 / L25 `ssr_luoyi`：与持有者决斗的另一方每次出杀需求 +1
                let wushuang_holder = if self.on_target {
                    self.source
                } else {
                    self.target
                };
                let opponent_has_wushuang = !ctx.catalog.is_rechanyuan_sealed_name(
                    ctx.core,
                    wushuang_holder,
                    crate::vocabulary::names::WU_SHUANG,
                ) && ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::WU_SHUANG)
                    .map(|d| ctx.core.has_skill(wushuang_holder, d))
                    .unwrap_or(false)
                    || ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(wushuang_holder, "u05_yunchuizhen_wushuang".to_string()))
                        .copied()
                        .unwrap_or(0)
                        != 0;
                let opponent_has_luoyi = ctx
                    .catalog
                    .skill_by_name("ssr_luoyi")
                    .map(|d| ctx.core.has_skill(wushuang_holder, d))
                    .unwrap_or(false);
                let needed_per_step =
                    1u32 + u32::from(opponent_has_wushuang) + u32::from(opponent_has_luoyi);
                let step_key = format!("duel_sha_step:{}", self.card);
                let played_so_far = if needed_per_step <= 2 {
                    if self.second_pending { 2 } else { 1 }
                } else {
                    ctx.core
                        .vm_aux
                        .flags
                        .get(&(responder, step_key.clone()))
                        .copied()
                        .unwrap_or(0)
                        + 1
                };
                if played_so_far < needed_per_step {
                    self.second_pending = true;
                    if needed_per_step > 2 {
                        ctx.core.vm_aux.flags.insert((responder, step_key), played_so_far);
                    }
                    continue; // 同一响应者再次询问第二张杀（含 L25 ssr_luoyi 叠加需求）
                }
                ctx.core.vm_aux.flags.remove(&(responder, step_key));
                self.second_pending = false;
                self.on_target = !self.on_target;
                continue;
            }
            // 激将代打逐人推进：当前蜀势力角色放弃时，继续询问下一位蜀势力角色
            if shas.is_empty() && ctx.core.player(responder).alive {
                if let Some((helper, _)) = jijiang_shu_helper(ctx.core, ctx.catalog, responder) {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((responder, "duel_jijiang_skip".to_string()))
                        .or_insert(0) |= 1u32 << helper;
                    continue;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "duel_jijiang_skip".to_string()));
            }
            // 放弃 → 受伤，决斗结束（重置无双/步进标记）
            self.second_pending = false;
            let step_key = format!("duel_sha_step:{}", self.card);
            ctx.core.vm_aux.flags.remove(&(self.source, step_key.clone()));
            ctx.core.vm_aux.flags.remove(&(self.target, step_key));
            self.emit_duel_damage(ctx, opponent, responder);
            return self.finish(ctx);
        }
    }
}

impl DuelMachine {
    fn emit_duel_damage(&self, ctx: &mut Ctx<'_>, opponent: PlayerId, responder: PlayerId) {
        let vm_card_bonus = ctx
            .core
            .vm_aux
            .flags
            .get(&(self.source, format!("card_damage_bonus:{}", self.card)))
            .copied()
            .unwrap_or(0);
        let dcard = (self.card != 0).then_some(self.card);
        ctx.emit(EventData::Damage(damage_data_with(
            Some(opponent),
            responder,
            (1 + luoyi_bonus(ctx, Some(opponent)) + vm_card_bonus) as i32,
            crate::event::DamageNature::Normal,
            dcard,
        )));
    }

    fn finish(&self, ctx: &mut Ctx<'_>) -> Flow {
        let card = self.card;
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, format!("card_damage_bonus:{card}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, "duel_jijiang_skip".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "duel_jijiang_skip".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, "dddfusi_window_refused".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "dddfusi_window_refused".to_string()));
        if card != 0 {
            if let Some(pos) = ctx.core.table.iter().position(|c| *c == card) {
                ctx.core.table.remove(pos);
                ctx.to_discard(card);
            }
        }
        Flow::Done
    }
}

pub(crate) fn jijiang_shu_helper(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    responder: PlayerId,
) -> Option<(PlayerId, Vec<crate::ids::CardId>)> {
    let has_jijiang = catalog
        .skill_by_name(crate::vocabulary::names::JI_JIANG)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false)
        && core.is_lord(responder);
    if !has_jijiang {
        return None;
    }
    let skip = core
        .vm_aux
        .flags
        .get(&(responder, "duel_jijiang_skip".to_string()))
        .copied()
        .unwrap_or(0);
    let n = core.players.len() as u8;
    for offset in 1..n {
        let p = (responder + offset) % n;
        if (skip & (1u32 << p)) != 0
            || !core.player(p).alive
            || core.effective_faction(catalog, p) != Some(crate::catalog::Faction::Shu)
            || catalog.blocks_all_cards(core, p)
        {
            continue;
        }
        let mut shas: Vec<crate::ids::CardId> = core
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| {
                effective_card_kind_in_core(core, catalog, *c).is_sha_like()
                    || catalog
                        .find_player_vm_view_as(core, p, *c, false, Some(CardKind::Sha))
                        .is_some()
            })
            .collect();
        for c in catalog.player_vm_view_as_extra_cards(core, p, false, Some(CardKind::Sha)) {
            if !shas.contains(&c) {
                shas.push(c);
            }
        }
        shas.retain(|&c| {
            !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(core, catalog, p, c)
        });
        if !shas.is_empty() {
            return Some((p, shas));
        }
    }
    None
}

