use super::*;

/// 濒死救援：按座位顺序询问持桃者；无人救则进入死亡子事件。
#[derive(Clone)]
pub struct DyingSaveMachine {
    pub(super) dying: PlayerId,
    pub(super) source: Option<PlayerId>,
    pub(super) responders: Vec<PlayerId>,
    pub(super) asked: bool,
    /// 本轮是否已有人用桃（整轮无人用桃 → 濒死结算死亡，不再循环询问）。
    pub(super) round_used: bool,
    /// 归汉（玩点论杀批 17，规则 v280）：limited 确认询问挂起中。
    pub(super) guihan_asked: bool,
}

impl DyingSaveMachine {
    pub(crate) fn new(
        dying: PlayerId,
        source: Option<PlayerId>,
        responders: Vec<PlayerId>,
        asked: bool,
        round_used: bool,
    ) -> Self {
        Self {
            dying,
            source,
            responders,
            asked,
            round_used,
            guihan_asked: false,
        }
    }

    pub(crate) fn with_guihan_asked(mut self, guihan_asked: bool) -> Self {
        self.guihan_asked = guihan_asked;
        self
    }
}

impl crate::catalog::SkillMachine for DyingSaveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DyingSave {
            dying: self.dying,
            source: self.source,
            responders: self.responders.clone(),
            asked: self.asked,
            round_used: self.round_used,
            guihan_asked: self.guihan_asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 刺猬（U07 玩点论杀韩龙，规则 v111）：濒死即死标记在位——跳
        // 过救援询问直接死亡（上游 ciqiu_dying dyingBegin firstDo die
        // 同口径）
        let shiyuan_dying = ctx
            .catalog
            .skill_by_name("txhj_shiyuan")
            .is_some_and(|d| ctx.core.has_skill(self.dying, d))
            && !ctx.catalog.is_chanyuan_sealed(ctx.core, self.dying)
            && !ctx
                .catalog
                .skill_by_name("txhj_feilong_skill_blank")
                .is_some_and(|b| ctx.core.has_skill(self.dying, b));
        if shiyuan_dying || ctx.core.turn.ciqiu_dying.contains(&self.dying) {
            ctx.core.turn.ciqiu_dying.retain(|p| *p != self.dying);
            ctx.emit(EventData::Death {
                player: self.dying,
                killer: self.source,
            });
            return Flow::Done;
        }
        // 归汉（玩点论杀批 17，规则 v280）：limited 濒死自救——求救循环
        // 前一次性确认（登记差异：上游 chooseToUse 玩家主动+自选男性
        // 目标，原生锁定确认+座位序首男性其他存活；limited 每局一次同
        // 口径）。确认后自回 1（救回由循环顶 hp>0 判定自然发出）摸 2，
        // 男性目标回 1 摸 2。
        if !self.guihan_asked {
            let gui = ["guihan", crate::vocabulary::host::WUDIAN_GUIHAN_KEY]
                .iter()
                .any(|k| {
                    ctx.catalog
                        .skill_by_name(k)
                        .is_some_and(|d| ctx.core.has_skill(self.dying, d))
                });
            let unused = !ctx.core.player(self.dying).guihan_used;
            if gui && unused {
                let options: Vec<crate::choice::ChoiceOption> = (0..ctx.core.players.len() as u8)
                    .filter(|&i| {
                        i != self.dying
                            && ctx.core.player(i).alive
                            && ctx.core.matches_gender(
                                ctx.catalog,
                                i,
                                crate::vocabulary::generals::Gender::Male,
                            )
                    })
                    .map(crate::choice::ChoiceOption::Player)
                    .collect();
                if !options.is_empty() {
                    self.guihan_asked = true;
                    return Flow::WaitChoice {
                        observer: self.dying,
                        spec: crate::choice::ChoiceSpec {
                            kind: crate::choice::ChoiceKind::SelectPlayers,
                            prompt: "wudian_guihan_ask",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
        } else {
            self.guihan_asked = false;
            if let Some(m) = ctx.answer().first().and_then(|o| match o {
                crate::choice::ChoiceOption::Player(p) => Some(*p),
                _ => None,
            }) {
                let t = self.dying;
                if m != t
                    && ctx.core.player(m).alive
                    && ctx.core.matches_gender(
                        ctx.catalog,
                        m,
                        crate::vocabulary::generals::Gender::Male,
                    )
                {
                    ctx.core.player_mut(t).guihan_used = true;
                    ctx.recover(t, 1, None);
                    ctx.start_draw(t, 2);
                    ctx.recover(m, 1, None);
                    ctx.start_draw(m, 2);
                }
            }
        }
        match ctx
            .core
            .vm_aux
            .flags
            .get(&(self.dying, "gw_fusheng_stage".to_string()))
            .copied()
            .unwrap_or(0)
        {
            1 => {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.dying, "gw_fusheng_stage".to_string()), 2);
                if ctx.core.player(self.dying).alive {
                    ctx.recover(self.dying, 1, None);
                    return Flow::Yield;
                }
            }
            2 => {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, "gw_fusheng_stage".to_string()));
                let resp = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, "gw_fusheng_resp".to_string()))
                    .and_then(|v| v.checked_sub(1))
                    .map(|v| v as PlayerId);
                if let Some(responder) = resp {
                    if responder == self.dying {
                        if ctx.core.player(responder).alive {
                            ctx.start_draw(responder, 2);
                            return Flow::Yield;
                        }
                    } else {
                        let first_alive = ctx.core.player(responder).alive;
                        let second_alive = ctx.core.player(self.dying).alive;
                        if first_alive && second_alive {
                            ctx.core
                                .vm_aux
                                .flags
                                .insert((self.dying, "gw_fusheng_stage".to_string()), 3);
                            ctx.start_draw(responder, 1);
                            return Flow::Yield;
                        } else if first_alive {
                            ctx.start_draw(responder, 1);
                            return Flow::Yield;
                        } else if second_alive {
                            ctx.start_draw(self.dying, 1);
                            return Flow::Yield;
                        }
                    }
                }
            }
            3 => {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, "gw_fusheng_stage".to_string()));
                if ctx.core.player(self.dying).alive {
                    ctx.start_draw(self.dying, 1);
                    return Flow::Yield;
                }
            }
            _ => {}
        }
        loop {
            if super::has_u01_respond_park(ctx)
                || ctx
                    .res
                    .stack
                    .iter()
                    .any(|f| f.kind == crate::event::EventKind::Recover)
            {
                return Flow::Yield;
            }
            if let Some(tao_card) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.dying, "dying_pending_saved_card".to_string()))
            {
                let cancelled = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, "dying_recover_was_cancelled".to_string()))
                    .is_some();
                if !cancelled {
                    ctx.log(LogEvent::DyingSaved {
                        player: self.dying,
                        by_card: tao_card,
                    });
                }
            }
            let p = ctx.core.player(self.dying);
            let mbmowang_rest = ctx
                .core
                .vm_aux
                .flags
                .get(&(self.dying, "mbmowang_resting".to_string()))
                .copied()
                .unwrap_or(0)
                > 0;
            if (p.hp > 0 && !p.dying) || mbmowang_rest {
                ctx.core.player_mut(self.dying).dying = false;
                for pid in 0..ctx.core.players.len() as PlayerId {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(pid, "dddfusi_window_refused".to_string()));
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, format!("jsrgzonghai_forbid:{}", self.dying)));
                if !mbmowang_rest {
                    // 救援成功（dyingAfter 等价，规则 v157）：发射救回事件，
                    // 觉醒技（姜城·焚逝）等触发挂在该事件的 AfterTriggers 窗
                    // 口——时序在固有救援机之后，与上游 dyingAfter 一致。
                    ctx.emit(EventData::DyingSaved {
                        player: self.dying,
                        source: self.source,
                    });
                }
                return Flow::Done; // 已被救回或进入殁亡修整
            }
            if !self.asked {
                if let Some(responder) = self.responders.first().copied() {
                    if ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(responder, "hs_yuelu_saved".to_string()))
                        .is_some()
                    {
                        self.responders.remove(0);
                        self.round_used = true;
                        continue;
                    }
                    if ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(responder, "l10_reguhuo_dying_resume".to_string()))
                        .is_some()
                    {
                        if let Some(tao) =
                            (ctx.catalog.hooks.reguhuo_consume_resp)(ctx.core, responder)
                        {
                            ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "xj_longxi_dying_guard".to_string()));
                self.responders.remove(0);
                            let eff_kind = match ctx
                                .core
                                .vm_aux
                                .flags
                                .remove(&(responder, "l10_reguhuo_resp_kind".to_string()))
                            {
                                Some(5) => CardKind::Jiu,
                                _ => CardKind::Tao,
                            };
                            self.round_used = true;
                            let base_depth = ctx.res.stack.len();
                            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                tao,
                                responder,
                                vec![self.dying],
                                Some(eff_kind),
                            )));
                            let fid = ctx.top_frame_id();
                            ctx.core.vm_aux.flags.insert(
                                (responder, format!("dying_save_frame:{fid}")),
                                (u32::from(self.dying) + 1) | ((tao as u32) << 8),
                            );
                            if (ctx.catalog.hooks.reguhuo_has_pending_final)(ctx.core, responder) {
                                (ctx.catalog.hooks.reguhuo_spawn_final)(
                                    ctx,
                                    responder,
                                    base_depth,
                                );
                            }
                            return Flow::Yield;
                        }
                    }
                    if let Some(ok) = super::take_u01_respond_result(ctx, responder) {
                        if ok {
                            let card_ovr = ctx
                                .core
                                .vm_aux
                                .flags
                                .remove(&(responder, "u01_respond_card_override".to_string()))
                                .and_then(|v| v.checked_sub(1));
                            let fusi_owner = ctx
                                .core
                                .vm_aux
                                .flags
                                .remove(&(responder, "u01_respond_fusi_owner".to_string()))
                                .unwrap_or(0);
                            ctx.core
                                .vm_aux
                                .flags
                                .remove(&(responder, "dddfusi_window_refused".to_string()));
                            self.responders.remove(0);
                            self.round_used = true;
                            if ctx.core.player(self.dying).alive {
                                let new_fid = ctx.res.next_event_id;
                                if let Some(card) = card_ovr {
                                    let eff_kind = if crate::kernel::combat_rules::effective_card_kind_for(
                                        ctx.core,
                                        ctx.catalog,
                                        responder,
                                        card,
                                    ) == crate::catalog::CardKind::Jiu
                                        && responder == self.dying
                                    {
                                        crate::catalog::CardKind::Jiu
                                    } else {
                                        crate::catalog::CardKind::Tao
                                    };
                                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                        card,
                                        responder,
                                        vec![self.dying],
                                        Some(eff_kind),
                                    )));
                                    ctx.core.vm_aux.flags.insert(
                                        (responder, format!("dying_save_frame:{new_fid}")),
                                        (u32::from(self.dying) + 1) | ((card as u32) << 8),
                                    );
                                } else {
                                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                                        card: None,
                                        player: self.dying,
                                        targets: vec![self.dying],
                                        nature: crate::event::DamageNature::Normal,
                                        use_as: Some(crate::catalog::CardKind::Tao),
                                        cancelled: false,
                                    }));
                                }
                                if fusi_owner > 0 {
                                    ctx.core.vm_aux.flags.insert(
                                        (responder, format!("dddfusi_used_owner:{new_fid}")),
                                        fusi_owner,
                                    );
                                }
                            }
                            return Flow::Yield;
                        }
                    }
                }
            }
            // 跳过已死亡或无桃/无救援技能的应答者；队列耗尽即一轮问完——
            // 本轮有人用桃且场上仍有桃/救援技能 → 开新一轮（负体力可由同一
            // 持桃者跨轮连续多张救回）；否则无人愿救，落至死亡。
            loop {
                while let Some(&r) = self.responders.first() {
                    crate::kernel::maybe_trigger_longxi_guarded(ctx, r, 4, "xj_longxi_dying_guard");
                    let alive = ctx.core.player(r).alive;
                    let has_tao = alive
                        && (!dying_tao_cards(ctx, r, self.dying).is_empty()
                            || crate::kernel::mbcmjiushi_dying_skill(
                                ctx.core,
                                ctx.catalog,
                                r,
                                self.dying,
                            )
                            .is_some()
                            || crate::kernel::gwfusheng_dying_skill(
                                ctx.core,
                                ctx.catalog,
                                r,
                                self.dying,
                            )
                            .is_some()
                            || !dying_response_skills(ctx.core, ctx.catalog, r, self.dying)
                                .is_empty()
                            || !dying_save_skill_options(ctx.core, ctx.catalog, r, self.dying)
                                .is_empty()
                            || !(ctx.catalog.hooks.hearth_dying_save_skills)(
                                ctx.core,
                                ctx.catalog,
                                r,
                                self.dying,
                            )
                            .is_empty());
                    if !has_tao {
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(r, "xj_longxi_dying_guard".to_string()));
                        self.responders.remove(0);
                    } else {
                        break;
                    }
                }
                if !self.responders.is_empty() || !self.round_used {
                    break;
                }
                let any_tao = ctx.core.players.iter().enumerate().any(|(i, p)| {
                    let pid = i as PlayerId;
                    p.alive
                        && (!dying_tao_cards(ctx, pid, self.dying).is_empty()
                            || crate::kernel::mbcmjiushi_dying_skill(
                                ctx.core,
                                ctx.catalog,
                                pid,
                                self.dying,
                            )
                            .is_some()
                            || crate::kernel::gwfusheng_dying_skill(
                                ctx.core,
                                ctx.catalog,
                                pid,
                                self.dying,
                            )
                            .is_some()
                            || !dying_response_skills(ctx.core, ctx.catalog, pid, self.dying)
                                .is_empty()
                            || !dying_save_skill_options(ctx.core, ctx.catalog, pid, self.dying)
                                .is_empty()
                            || !(ctx.catalog.hooks.hearth_dying_save_skills)(
                                ctx.core,
                                ctx.catalog,
                                pid,
                                self.dying,
                            )
                            .is_empty())
                });
                if !any_tao {
                    break;
                }
                self.responders = ctx.core.seat_order_from(self.dying);
                self.round_used = false;
            }
            let Some(responder) = self.responders.first().copied() else {
                for pid in 0..ctx.core.players.len() as PlayerId {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(pid, "dddfusi_window_refused".to_string()));
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.dying, format!("jsrgzonghai_forbid:{}", self.dying)));
                ctx.emit(EventData::Death {
                    player: self.dying,
                    killer: self.source,
                });
                return Flow::Done;
            };
            if self.asked {
                self.asked = false;
                let answer = ctx.answer().clone();
                if let Some(ChoiceOption::Skill(rdef)) = answer.first().copied() {
                    let is_host_dying_skill = ctx.catalog.skill_by_name("shahun") == Some(rdef)
                        || ctx.catalog.skill_by_name("duhun") == Some(rdef);
                    if !is_host_dying_skill
                        && dying_response_skills(ctx.core, ctx.catalog, responder, self.dying)
                            .contains(&rdef)
                    {
                        let need_ctx = if responder == self.dying { 5 } else { 4 };
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((responder, "l10_reguhuo_dying_resume".to_string()), 1);
                        (ctx.catalog.hooks.reguhuo_spawn_response)(
                            ctx,
                            responder,
                            need_ctx,
                            self.dying,
                            rdef,
                            responder,
                        );
                        return Flow::Yield;
                    }
                    let fid = ctx.top_frame_id();
                    if let Some(immediate) = (ctx.catalog.hooks.try_activate_respond_skill)(
                        ctx,
                        responder,
                        rdef,
                        crate::catalog::CardKind::Tao,
                        fid,
                    ) {
                        if immediate {
                            let card_ovr = ctx
                                .core
                                .vm_aux
                                .flags
                                .remove(&(responder, "u01_respond_card_override".to_string()))
                                .and_then(|v| v.checked_sub(1));
                            let fusi_owner = ctx
                                .core
                                .vm_aux
                                .flags
                                .remove(&(responder, "u01_respond_fusi_owner".to_string()))
                                .unwrap_or(0);
                            ctx.core
                                .vm_aux
                                .flags
                                .remove(&(responder, "dddfusi_window_refused".to_string()));
                            self.responders.remove(0);
                            self.round_used = true;
                            if ctx.core.player(self.dying).alive {
                                let new_fid = ctx.res.next_event_id;
                                if let Some(card) = card_ovr {
                                    let eff_kind = if crate::kernel::combat_rules::effective_card_kind_for(
                                        ctx.core,
                                        ctx.catalog,
                                        responder,
                                        card,
                                    ) == crate::catalog::CardKind::Jiu
                                        && responder == self.dying
                                    {
                                        crate::catalog::CardKind::Jiu
                                    } else {
                                        crate::catalog::CardKind::Tao
                                    };
                                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                        card,
                                        responder,
                                        vec![self.dying],
                                        Some(eff_kind),
                                    )));
                                    ctx.core.vm_aux.flags.insert(
                                        (responder, format!("dying_save_frame:{new_fid}")),
                                        (u32::from(self.dying) + 1) | ((card as u32) << 8),
                                    );
                                } else {
                                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                                        card: None,
                                        player: self.dying,
                                        targets: vec![self.dying],
                                        nature: crate::event::DamageNature::Normal,
                                        use_as: Some(crate::catalog::CardKind::Tao),
                                        cancelled: false,
                                    }));
                                }
                                if fusi_owner > 0 {
                                    ctx.core.vm_aux.flags.insert(
                                        (responder, format!("dddfusi_used_owner:{new_fid}")),
                                        fusi_owner,
                                    );
                                }
                            }
                        }
                        return Flow::Yield;
                    }
                }
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "dddfusi_window_refused".to_string()));
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(responder, "xj_longxi_dying_guard".to_string()));
                self.responders.remove(0);
                if let Some(ChoiceOption::Card(tao)) = answer.first().copied() {
                    let raw_kind = crate::kernel::combat_rules::effective_card_kind_for(
                        ctx.core,
                        ctx.catalog,
                        responder,
                        tao,
                    );
                    let raw_cname = ctx.catalog.cards[tao as usize].name;
                    let in_hand = ctx.core.player(responder).hand.contains(&tao);
                    let eff_kind = if raw_cname == "gw_fuyuan" {
                        CardKind::Tao
                    } else if matches!(raw_cname, "gw_xianzumaijiu" | "gw_lang")
                        && responder == self.dying
                    {
                        CardKind::Jiu
                    } else if matches!(
                        raw_kind,
                        CardKind::Tao
                            | CardKind::Jiu
                            | CardKind::Hufu
                            | CardKind::Tianxianjiu
                            | CardKind::Dinvxuanshuang
                    ) && (!matches!(raw_kind, CardKind::Jiu | CardKind::Tianxianjiu)
                        || responder == self.dying)
                    {
                        raw_kind
                    } else if raw_kind == CardKind::Yuchanxun {
                        CardKind::Tao
                    } else if responder == self.dying && raw_kind == CardKind::Yuchanzhen {
                        CardKind::Jiu
                    } else if responder == self.dying && raw_kind == CardKind::Yuchanli {
                        CardKind::Tianxianjiu
                    } else if responder == self.dying
                        && ctx
                            .catalog
                            .find_player_vm_view_as(
                                ctx.core,
                                responder,
                                tao,
                                false,
                                Some(CardKind::Jiu),
                            )
                            .is_some()
                    {
                        CardKind::Jiu
                    } else {
                        CardKind::Tao
                    };
                    let use_as = (eff_kind != ctx.catalog.card_kind(tao) || !in_hand).then_some(eff_kind);
                    self.round_used = true;
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        tao,
                        responder,
                        vec![self.dying],
                        use_as,
                    )));
                    let fid = ctx.top_frame_id();
                    ctx.core.vm_aux.flags.insert(
                        (responder, format!("dying_save_frame:{fid}")),
                        (u32::from(self.dying) + 1) | ((tao as u32) << 8),
                    );
                    return Flow::Yield;
                } else if let Some(ChoiceOption::Skill(def)) = answer.first().copied() {
                    if crate::kernel::mbcmjiushi_dying_skill(
                        ctx.core,
                        ctx.catalog,
                        responder,
                        self.dying,
                    ) == Some(def)
                    {
                        self.round_used = true;
                        ctx.log(LogEvent::SkillTriggered {
                            skill: crate::vocabulary::host::HLSS_MBCMJIUSHI_KEY,
                            owner: responder,
                            presentation: None,
                        });
                        // LIFO: push virtual Jiu first so TurnOverBefore resolves before recovery.
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: responder,
                            targets: vec![self.dying],
                            nature: crate::event::DamageNature::Normal,
                            use_as: Some(CardKind::Jiu),
                            cancelled: false,
                        }));
                        let fid = ctx.top_frame_id();
                        ctx.core.vm_aux.flags.insert(
                            (responder, format!("dying_save_frame:{fid}")),
                            u32::from(self.dying) + 1,
                        );
                        ctx.emit(EventData::TurnOverBefore {
                            player: responder,
                            cancelled: false,
                        });
                        return Flow::Yield;
                    } else if let Some(sdef) = ctx.catalog.skill_by_name("shahun")
                        .filter(|&sd| sd == def && dying_response_skills(ctx.core, ctx.catalog, responder, self.dying).contains(&sd))
                    {
                        self.round_used = true;
                        if let Some(inst) = ctx.core.skills.get_mut(&(responder, sdef)) {
                            inst.once_used = true;
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "shahun",
                            owner: responder,
                            presentation: None,
                        });
                        let snap = crate::snap::MachineSnapshot::Host {
                            def_key: "shahun".to_string(),
                            behavior: crate::vocabulary::host::HostBehavior::U05GujianCharBatchA.tag(),
                            stage: 0,
                            owner: responder,
                            target: None,
                            card: None,
                            cards: Vec::new(),
                            targets: Vec::new(),
                        };
                        if let Some(Ok(machine)) = (ctx.catalog.hooks.restore_machine)(&snap, ctx.catalog) {
                            ctx.res.parks.push(crate::resolution::Park {
                                resume_depth: ctx.res.stack.len(),
                                slot: crate::resolution::Slot::Trigger { frame: crate::resolution::NO_FRAME },
                                machine,
                                awaiting_answer: false,
                            });
                            return Flow::Yield;
                        }
                    } else if let Some(_ddef) = ctx.catalog.skill_by_name("duhun")
                        .filter(|&dd| dd == def && dying_response_skills(ctx.core, ctx.catalog, responder, self.dying).contains(&dd))
                    {
                        self.round_used = true;
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "duhun",
                            owner: responder,
                            presentation: None,
                        });
                        let snap = crate::snap::MachineSnapshot::Host {
                            def_key: "duhun".to_string(),
                            behavior: crate::vocabulary::host::HostBehavior::U05GujianCharBatchC.tag(),
                            stage: 0,
                            owner: responder,
                            target: None,
                            card: None,
                            cards: Vec::new(),
                            targets: Vec::new(),
                        };
                        if let Some(Ok(machine)) = (ctx.catalog.hooks.restore_machine)(&snap, ctx.catalog) {
                            ctx.res.parks.push(crate::resolution::Park {
                                resume_depth: ctx.res.stack.len(),
                                slot: crate::resolution::Slot::Trigger { frame: crate::resolution::NO_FRAME },
                                machine,
                                awaiting_answer: false,
                            });
                            return Flow::Yield;
                        }
                    } else if crate::kernel::gwfusheng_dying_skill(
                        ctx.core,
                        ctx.catalog,
                        responder,
                        self.dying,
                    ) == Some(def)
                    {
                        self.round_used = true;
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "gwfusheng",
                            owner: responder,
                            presentation: None,
                        });
                        // character/gwent/skill.js:1391-1399: 令目标翻面并回复 1 体力，然后与目标各摸 1（自救摸 2）；分步 Yield 保证 LIFO 顺序。
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((self.dying, "gw_fusheng_stage".to_string()), 1);
                        ctx.core.vm_aux.flags.insert(
                            (self.dying, "gw_fusheng_resp".to_string()),
                            u32::from(responder) + 1,
                        );
                        ctx.turn_over(self.dying);
                        return Flow::Yield;
                    }
                    if (ctx.catalog.hooks.hearth_dying_save_skills)(
                        ctx.core,
                        ctx.catalog,
                        responder,
                        self.dying,
                    )
                    .contains(&def)
                    {
                        if (ctx.catalog.hooks.resolve_hearth_dying_save)(
                            ctx,
                            responder,
                            self.dying,
                            def,
                        ) {
                            self.round_used = true;
                            continue;
                        }
                        return Flow::Yield;
                    }
                }
                continue; // 无论用桃或放弃，继续处理队列（轮转见队首过滤）
            }
            self.asked = true;
            let mut options = vec![ChoiceOption::Pass];
            // 司圃（Boss 应昭，规则 v223/v232）：应昭出牌阶段已用牌数≤2
            // 期间其他人不可救（上游 cardSavable 同口径）——非应昭救援者
            // 的桃不入选项（酒同理由 cardRespondable）
            let sipu_ban_save = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
                .map(|def| {
                    ctx.core.has_skill(self.dying, def)
                        && ctx.core.turn.current == self.dying
                        && ctx.core.turn.cards_used_play_phase < 3
                })
                .unwrap_or(false);
            let sipu_holder = if ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
                .is_some()
            {
                self.dying
            } else {
                0
            };
            let zonghai_ban_save = ctx
                .core
                .vm_aux
                .flags
                .get(&(self.dying, format!("jsrgzonghai_forbid:{}", self.dying)))
                .copied()
                .unwrap_or(0)
                != 0;
            options.extend(
                dying_tao_cards(ctx, responder, self.dying)
                    .into_iter()
                    .filter(|c| {
                        (!sipu_ban_save
                            || (responder == sipu_holder
                                && is_tao_like(ctx, responder, *c, self.dying)))
                            && (!zonghai_ban_save || responder == self.dying)
                    })
                    .map(ChoiceOption::Card),
            );
            if let Some(def) = crate::kernel::mbcmjiushi_dying_skill(
                ctx.core,
                ctx.catalog,
                responder,
                self.dying,
            ) {
                options.push(ChoiceOption::Skill(def));
            }
            if let Some(def) = crate::kernel::gwfusheng_dying_skill(
                ctx.core,
                ctx.catalog,
                responder,
                self.dying,
            ) {
                if !options.contains(&ChoiceOption::Skill(def)) {
                    options.push(ChoiceOption::Skill(def));
                }
            }
            for rdef in dying_response_skills(ctx.core, ctx.catalog, responder, self.dying) {
                let opt = ChoiceOption::Skill(rdef);
                if !options.contains(&opt) {
                    options.push(opt);
                }
            }
            for opt in dying_save_skill_options(ctx.core, ctx.catalog, responder, self.dying) {
                if !options.contains(&opt) {
                    options.push(opt);
                }
            }
            for rdef in
                (ctx.catalog.hooks.hearth_dying_save_skills)(ctx.core, ctx.catalog, responder, self.dying)
            {
                if !options.contains(&ChoiceOption::Skill(rdef)) {
                    options.push(ChoiceOption::Skill(rdef));
                }
            }
            // 冰封·封（swd 批 134）与移魂2：封锁者不能求桃/自救
            let options = if ctx.catalog.blocks_card_use_and_save(ctx.core, responder) {
                vec![ChoiceOption::Pass]
            } else {
                options
            };
            return Flow::WaitChoice {
                observer: responder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "dying_save",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
    }
}

impl DyingSaveMachine {
    pub(crate) fn apply_tao_save(
        &self,
        ctx: &mut Ctx<'_>,
        responder: PlayerId,
        tao: crate::ids::CardId,
    ) {
        let tao_name = ctx.catalog.cards[tao as usize].name;
        if tao_name == "gw_fuyuan" {
            ctx.recover(self.dying, 1, Some(tao));
            ctx.start_draw(self.dying, 1);
            ctx.log(LogEvent::DyingSaved {
                player: self.dying,
                by_card: tao,
            });
            return;
        }
        if matches!(tao_name, "gw_xianzumaijiu" | "gw_lang") {
            ctx.recover(self.dying, 1, Some(tao));
            ctx.log(LogEvent::DyingSaved {
                player: self.dying,
                by_card: tao,
            });
            return;
        }
        // 救援（孙权主公技，锁定；v202 对齐上游 filter）：主公
        // 资格 + 用桃者为吴势力 + 非濒死者本人——桃回复 +1
        let jiuyuan = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::JIU_YUAN)
            .map(|d| {
                ctx.core.has_skill(self.dying, d)
                    && responder != self.dying
                    && ctx.core.is_lord(self.dying)
                    && ctx.core.effective_faction(ctx.catalog, responder)
                        == Some(crate::catalog::Faction::Wu)
            })
            .unwrap_or(false);
        let amount = if jiuyuan { 2 } else { 1 }
            + ctx
                .core
                .vm_aux
                .flags
                .remove(&(responder, "vm_va_recover_bonus".to_string()))
                .unwrap_or(0);
        let gained = ctx.recover(self.dying, amount, Some(tao));
        if gained > 0 {
            ctx.core
                .vm_aux
                .flags
                .insert((self.dying, "dying_pending_saved_card".to_string()), tao);
        }
    }
}

fn other_dying_save_banned(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> bool {
    if catalog.blocks_card_use_and_save(core, responder)
        || crate::kernel::is_guijiang_savable_blocked(core, catalog, dying)
    {
        return true;
    }
    if responder == dying {
        return false;
    }
    let sipu_ban = catalog
        .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
        .is_some_and(|def| {
            core.has_skill(dying, def)
                && core.turn.current == dying
                && core.turn.cards_used_play_phase < 3
        });
    let zonghai_ban = core
        .vm_aux
        .flags
        .get(&(dying, format!("jsrgzonghai_forbid:{dying}")))
        .copied()
        .unwrap_or(0)
        != 0;
    sipu_ban || zonghai_ban
}

pub(crate) fn dying_response_skills(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<crate::ids::SkillDefId> {
    if other_dying_save_banned(core, catalog, responder, dying) {
        return Vec::new();
    }
    if catalog.wansha_blocks_rescue(core, responder, dying) {
        return Vec::new();
    }
    let need_ctx = if responder == dying { 5 } else { 4 };
    crate::kernel::host_response_skills(core, catalog, responder, need_ctx)
}

pub(crate) fn dying_save_skill_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<ChoiceOption> {
    if other_dying_save_banned(core, catalog, responder, dying) {
        return Vec::new();
    }
    (catalog.hooks.respond_skill_options)(core, catalog, responder, crate::catalog::CardKind::Tao, true)
}
