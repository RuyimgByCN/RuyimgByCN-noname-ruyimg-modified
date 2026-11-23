//! 杀结算的挂起应答恢复（复仇、霰弹、掠影与各武器/防具询问）：有待恢复的询问时返回本步结果。

use super::*;

pub(super) fn cixiong_need(ctx: &Ctx<'_>, source: PlayerId) -> usize {
    if ctx.catalog.vm_card_history
        && crate::rvm_bridge::modifiers::has_fuchendangmo(ctx.core, ctx.catalog, source)
    {
        2
    } else {
        1
    }
}

pub(super) fn guanshi_need(ctx: &Ctx<'_>, source: PlayerId) -> usize {
    if ctx.catalog.vm_card_history
        && crate::rvm_bridge::modifiers::has_shangjiangdelvli(ctx.core, ctx.catalog, source)
    {
        1
    } else {
        2
    }
}

fn gain_if_shenhanxueyin(ctx: &mut Ctx<'_>, source: PlayerId, c: crate::ids::CardId) {
    if ctx.catalog.vm_card_history
        && crate::rvm_bridge::modifiers::has_shenhanxueyin(ctx.core, ctx.catalog, source)
        && ctx.core.player(source).alive
        && ctx.core.discard.contains(&c)
    {
        ctx.core.discard.retain(|x| *x != c);
        ctx.core.player_mut(source).hand.push(c);
        ctx.out.push(LogEvent::CardsDrawn {
            player: source,
            cards: vec![c],
        });
    }
}

impl ShaResolveMachine {
    /// 杀结算的挂起应答恢复（复仇、诱敌深入 shaBefore、霰弹、掠影与各武器/防具询问）：有待恢复的询问时返回本步结果。
    pub(super) fn resume_pending_answer(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        if self.has_sha_trigger_park(ctx) {
            return Some(Flow::Yield);
        }
        if self.take_sha_end_settled(ctx) {
            return Some(self.advance_target(ctx));
        }
        if let Some(is_damage) = self.take_sha_pending_end(ctx) {
            return Some(self.finish_target_sha_end(ctx, is_damage));
        }
        if let Some(fid) = self.current_sha_frame_id(ctx) {
            if ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.source, format!("l11_guanshi_dodged_pending:{fid}")))
                .is_some()
            {
                let forced = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.source, "l11_guanshi_force_hit".to_string()))
                    .is_some();
                if forced && ctx.core.player(self.target).alive {
                    return Some(self.emit_sha_damage(ctx));
                }
                return Some(self.finish_or_defer_sha_end(ctx, false));
            }
            if ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.source, format!("u05_sha_hit_resume:{fid}")))
                .is_some()
            {
                return Some(self.sha_hit_pre_damage(ctx));
            }
        }
        // 复仇（玩点论杀批 14，规则 v279）：成为杀目标时交出 1 张牌（"he"）取消
        // 该杀（上游 target shaBefore direct chooseCard("he") 同面；所选牌归来
        // 源+标记来源供 fuchou2 回合末结算；杀回来源手=上游 cancel 回手
        // 同口径）。取消（空应答）则照常响应流程。
        if !self.asked && !self.asking_fuchou && self.fuchou_card.is_none() {
            let t = self.target;
            let has = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_FUCHOU_KEY)
                .map(|d| ctx.core.has_skill(t, d))
                .unwrap_or(false);
            if has && (!ctx.core.player(t).hand.is_empty() || !ctx.core.player(t).equips.is_empty())
            {
                self.asking_fuchou = true;
                let mut options: Vec<ChoiceOption> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    ctx.core
                        .player(t)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                return Some(Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "wudian_fuchou_cancel",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                });
            }
        } else if self.asking_fuchou {
            self.asking_fuchou = false;
            let t = self.target;
            let picked = ctx.answer().iter().find_map(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            if let Some(c) = picked {
                // 复仇（玩点论杀/character/skill.js:534-544）：取消该目标的杀并交出所选牌，
                // 原【杀】不退回来源手牌（全部目标结算/取消后由 advance_target 弃置）；
                // fuchou2 标记集按 `.add()` 去重。
                self.fuchou_card = Some(c);
                if ctx.core.player(t).equips.contains(&c) {
                    ctx.core.player_mut(t).equips.retain(|x| *x != c);
                    ctx.core.declared_slots.remove(&c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                    ctx.note_cards_lost(t, &[c]);
                    ctx.emit(EventData::CardsLost {
                        player: t,
                        cards: vec![c],
                        origins: vec![crate::event::LostOrigin::Equip],
                    });
                    ctx.core.player_mut(self.source).hand.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from: t,
                        to: self.source,
                    });
                    ctx.emit(EventData::CardsGiven {
                        giver: t,
                        recipient: self.source,
                        cards: vec![c],
                    });
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.source,
                        cards: vec![c],
                    });
                } else {
                    ctx.give_hand_card(t, self.source, c);
                }
                if !ctx.core.player(t).fuchou_marked.contains(&self.source) {
                    ctx.core.player_mut(t).fuchou_marked.push(self.source);
                }
                return Some(self.advance_target(ctx));
            } else {
                self.fuchou_card = Some(u32::MAX);
            }
        }
        // g_youdishenru（玩点论杀 card/card.js:169-202, card/skill.js:16-36）：
        // `shaBefore` 时点（早于铁骑/仁王盾/八卦/雌雄/霰弹/狩猎）：询问是否使用【诱敌深入】，
        // 以正式 `CardUsed` 入栈并开启 `_wuxie`（NullifyWindow）；若被无懈则原杀继续结算。
        if !self.asked && !self.armor_judged {
            let sha_fid = self
                .current_sha_frame_id(ctx)
                .unwrap_or_else(|| ctx.current_frame_id());
            if self.liuli_stage == 3 {
                self.liuli_stage = 0;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.target, format!("yd_tried:{sha_fid}")), 1);
                if let Some(ChoiceOption::Card(yd)) = ctx.answer().first().copied() {
                    if ctx.core.player(self.target).hand.contains(&yd)
                        && ctx.catalog.card_kind(yd) == CardKind::Youdishenru
                        && !crate::kernel::jinyan_blocks_black_trick(
                            ctx.core,
                            ctx.catalog,
                            self.target,
                            yd,
                        )
                    {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((self.target, "yd_pending_source".to_string()), u32::from(self.source) + 1);
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((self.target, "yd_pending_sha_frame".to_string()), sha_fid);
                        ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                            yd,
                            self.target,
                            Vec::new(),
                            Some(CardKind::Youdishenru),
                        )));
                        return Some(Flow::Yield);
                    }
                }
            } else if ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.target, format!("yd_cancelled:{sha_fid}")))
                .is_some()
            {
                return Some(self.advance_target(ctx));
            } else if !ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(self.target, format!("yd_tried:{sha_fid}")))
                && ctx.core.player(self.target).alive
                && !self.is_sha_direct_hit(ctx)
                && !ctx.catalog.blocks_card_use_and_save(ctx.core, self.target)
                && !ctx.core.player(self.target).fengyin_active
            {
                let yd_opts = Self::youdishenru_use_options(ctx.core, ctx.catalog, self.target);
                if !yd_opts.is_empty() {
                    self.liuli_stage = 3;
                    return Some(Flow::WaitChoice {
                        observer: self.target,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: crate::vocabulary::host::WUDIAN_YOUDISHENRU_USE,
                            options: yd_opts,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    });
                }
            }
        }
        // 寒冰剑（逐技对照批一，规则 v75）：目标自选弃牌恢复——上游
        // discardPlayerCard("he", forced) 逐张强制弃，目标从手牌+装备
        // 自选；次数用尽或目标无牌可弃后跳过伤害。
        // 霰弹（U05 ow 死神，规则 v140）：弃牌应答——红=直击（哨卫同
        // 款直中路径）；黑=本杀伤害 +1；取消=照常进入防具/闪询问。
        if self.asking_xiandan {
            self.asking_xiandan = false;
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                let red = matches!(
                    ctx.catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let in_hand = ctx.core.player(self.source).hand.contains(&c);
                if in_hand {
                    ctx.discard_from_hand(self.source, c);
                } else if ctx.core.player(self.source).equips.contains(&c) {
                    ctx.core.player_mut(self.source).equips.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::CardDiscarded {
                    player: self.source,
                    cards: vec![c],
                });
                if red {
                    if ctx.core.player(self.target).alive {
                        return Some(self.sha_hit_pre_damage(ctx));
                    }
                    return Some(self.finish_target_sha_end(ctx, false));
                }
                self.xiandan_bonus = true;
            }
            // 黑弃/取消：复位 asked 落回 !asked 分支重开出闪询问（青龙
            // 再杀 reboot 同款——否则霰弹应答会被门控应答区误读）。
            self.asked = false;
        }
        // 掠影（规则 v154；v194 两相协议）：相以应答形状区分——确认相
        // 只会收到 Confirm（确认）或空（放弃），选人相只会收到 Player。
        // Confirm=消耗 usable、获得目标 he 随机一张，仍有其他有牌者则
        // 转入目标选人相；空=放弃（不消耗 usable，重开闪询问）；Player=
        // 受害者选定，来源随机弃其 he 一张后复位 asked。
        if self.asking_lueying {
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(v)) => {
                    self.asking_lueying = false;
                    self.lueying_victim = Some(v);
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(v);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(v, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![dc],
                        });
                    }
                    self.asked = false;
                }
                Some(ChoiceOption::Confirm) => {
                    ctx.core.turn.lueying_used = true;
                    // 获得目标 he 随机一张
                    let pool: Vec<crate::ids::CardId> = {
                        let p = ctx.core.player(self.target);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let card = pool[k];
                        let public = ctx.core.player(self.target).equips.contains(&card);
                        ctx.take_owned_card(self.target, card);
                        ctx.core.player_mut(self.source).hand.push(card);
                        ctx.log(LogEvent::CardsTaken {
                            from: self.target,
                            to: self.source,
                            cards: vec![card],
                            public,
                        });
                    }
                    // 仍有其他有牌者→目标选人相；否则本技结束
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.source
                                && *p != self.target
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .collect();
                    if others.is_empty() {
                        self.asking_lueying = false;
                        self.asked = false;
                    } else {
                        let options: Vec<ChoiceOption> =
                            others.into_iter().map(ChoiceOption::Player).collect();
                        return Some(Flow::WaitChoice {
                            observer: self.target,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "lueying_choose_victim",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        });
                    }
                }
                _ => {
                    // 放弃：不消耗 usable（同上游 usable 口径），标记本杀
                    // 已问过（阻止 step 重入再问），重开闪询问
                    self.asking_lueying = false;
                    self.lueying_declined = true;
                    self.asked = false;
                }
            }
        }
        if self.hanbing_pending > 0 {
            let pick = ctx.answer().first().and_then(|o| match o {
                ChoiceOption::Card(c) => Some(*c),
                _ => None,
            });
            self.hanbing_pending -= 1;
            if let Some(c) = pick {
                if ctx.core.player(self.target).hand.contains(&c) {
                    ctx.discard_from_hand(self.target, c);
                    gain_if_shenhanxueyin(ctx, self.source, c);
                } else if ctx.core.player(self.target).equips.contains(&c) {
                    ctx.core.player_mut(self.target).equips.retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    ctx.to_discard(c);
                    gain_if_shenhanxueyin(ctx, self.source, c);
                }
            }
            if self.hanbing_pending > 0 && self.target_has_he(ctx) {
                return Some(self.open_hanbing_target_ask(ctx));
            }
            self.hanbing_pending = 0;
            return Some(self.finish_or_defer_sha_end(ctx, true));
        }
        // 寒冰剑（批③-b 第三把）：来源防止伤害确认恢复——确认后转入
        // 目标自选弃牌（两次；逐技对照批一消除「自动依次」差异）。
        // 风雪刃恢复：确认→目标翻面+摸 1（无伤害）；放弃→照常伤害
        if self.asking_fengxueren {
            self.asking_fengxueren = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                let fid = self.current_sha_frame_id(ctx);
                ctx.emit(EventData::TurnOverBefore {
                    player: self.target,
                    cancelled: false,
                });
                ctx.start_draw(self.target, 1);
                if Self::should_defer_sha_end(ctx, self.source) {
                    return Some(self.defer_sha_end(ctx, fid, false));
                }
                return Some(self.finish_target_sha_end(ctx, false));
            }
            return Some(self.emit_sha_damage(ctx));
        }
        if self.asking_hanbing {
            self.asking_hanbing = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed {
                let extra = ctx.catalog.vm_card_history
                    && crate::rvm_bridge::modifiers::has_fuchendangmo(
                        ctx.core,
                        ctx.catalog,
                        self.source,
                    );
                self.hanbing_pending = if extra { 3 } else { 2 };
                if self.target_has_he(ctx) {
                    return Some(self.open_hanbing_target_ask(ctx));
                }
                self.hanbing_pending = 0;
                return Some(self.finish_target_sha_end(ctx, true));
            }
            // 放弃：照常进入麒麟弓/伤害结算（单武器互斥，实际直接伤害）。
            return Some(self.emit_sha_damage(ctx));
        }
        // 贯石斧（批③-b 第五把）：弃二（上将的膂力时弃一）询问恢复——按应答弃牌，强制伤害。
        if self.asking_guanshi {
            self.asking_guanshi = false;
            let need = guanshi_need(ctx, self.source);
            let picks: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|c| match c {
                    ChoiceOption::Card(card) => Some(*card),
                    _ => None,
                })
                .collect();
            let fired = picks.len() == need;
            for card in picks {
                let in_hand = ctx.core.player(self.source).hand.contains(&card);
                if in_hand {
                    ctx.discard_from_hand(self.source, card);
                } else if ctx.core.player(self.source).equips.contains(&card) {
                    ctx.core
                        .player_mut(self.source)
                        .equips
                        .retain(|x| *x != card);
                    ctx.to_discard(card);
                }
            }
            self.guanshi_cards.clear();
            // 恰弃两张 → 强制伤害（被闪仍命中）；放弃 → 无伤。
            if fired && ctx.core.player(self.target).alive {
                let fid = self.current_sha_frame_id(ctx);
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(self.source),
                    self.target,
                    1,
                    self.nature,
                    self.card,
                )));
                if Self::should_defer_sha_end(ctx, self.source) {
                    return Some(self.defer_sha_end(ctx, fid, true));
                }
                return Some(self.finish_target_sha_end(ctx, true));
            }
            return Some(self.finish_target_sha_end(ctx, false));
        }
        // 青龙偃月刀（批③-b 第六把）：再杀询问恢复——弃置选中的杀并
        // 以其重启对同一目标的杀结算（防/闪/贯石斧/麒麟弓流程全部重走）。
        if self.asking_qinglong {
            self.asking_qinglong = false;
            if let Some(ChoiceOption::Card(new_sha)) = ctx.answer().first().copied() {
                if ctx.core.player(self.source).hand.contains(&new_sha) {
                    ctx.discard_from_hand(self.source, new_sha);
                    crate::kernel::emit_card_responded_as(ctx, self.source, new_sha, CardKind::Sha);
                    self.clear_sha_target_judge_flags(ctx);
                    if let Some(fid) = self.current_sha_frame_id(ctx) {
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.target, format!("yd_tried:{fid}")));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.target, format!("yd_cancelled:{fid}")));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.source, format!("u05_sha_hit_resume:{fid}")));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.source, format!("u05_sadeng_tried:{fid}:{}", self.target)));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.source, format!("u05_guiyanfadao_done:{fid}:{}", self.target)));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(self.source, format!("u05_guiyanfadao_unhurt:{fid}:{}", self.target)));
                    }
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.source, format!("u05_sadeng_handled:{}", self.target)));
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(self.source, format!("u05_guiyanfadao_active_fid:{}", self.target)));
                    self.shoulie_direct = false;
                    self.card = Some(new_sha);
                    self.asked = false;
                    self.armor_judged = false;
                    self.dodges_used = 0;
                    self.tieji_red = None;
                    self.liuli_done = false;
                    self.liuli_stage = 0;
                    self.liuli_card = None;
                    self.asking_fuchou = false;
                    self.fuchou_card = None;
                    self.asking_sadeng = false;
                    self.asking_sadeng_shan = false;
                    return Some(Flow::Yield);
                }
            }
            // 放弃：当前目标按被闪结算（无伤）→ shaEnd + 推进队列/结束。
            return Some(self.finish_target_sha_end(ctx, false));
        }
        // 萨登荆环（swd 卡牌面窗口机批）：被闪后判定确认恢复——确定性判
        // 定（内联口径同铁骑，登记差异：无鬼道改判窗）；红色→目标额外
        // 出闪询问，未打出则本杀改为命中（上游 shaMiss→judge→shaHit 改
        // 写同口径）；确认放弃或判定黑→被闪结算维持。
        if self.asking_sadeng {
            self.asking_sadeng = false;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if confirmed
                && ctx.core.player(self.target).alive
                && (!ctx.core.deck.is_empty() || !ctx.core.discard.is_empty())
            {
                let judged = ctx.draw_judge_card_with_elo(
                    self.source,
                    crate::kernel::EloJudgePref::RedPositive,
                );
                let (suit, _) = ctx.inline_judge_suit_rank(self.source, judged);
                let red = matches!(
                    suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                let _ = ctx.settle_inline_judge_card(self.source, suit, judged);
                if red {
                    let options: Vec<ChoiceOption> = super::respond::collect_shan_response_cards(
                        ctx.core,
                        ctx.catalog,
                        self.target,
                    )
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect();
                    if !options.is_empty() {
                        self.asking_sadeng_shan = true;
                        return Some(Flow::WaitChoice {
                            observer: self.target,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "sadengjinhuan_extra_shan",
                                options,
                                min: 0,
                                max: 1,
                                allow_cancel: false,
                            },
                        });
                    }
                    return Some(self.sadeng_continue_to_damage(ctx));
                }
            }
            ctx.emit(EventData::ShaDodged {
                card: self.card,
                source: self.source,
                target: self.target,
            });
            return Some(self.finish_or_defer_sha_end(ctx, false));
        }
        if self.asking_sadeng_shan {
            self.asking_sadeng_shan = false;
            if let Some(ChoiceOption::Card(shan)) = ctx.answer().first().copied() {
                if ctx.core.player(self.target).hand.contains(&shan)
                    || ctx.core.player(self.target).equips.contains(&shan)
                {
                    discard_response_card(ctx, self.target, shan, CardKind::Shan);
                    if ctx.catalog.vm_card_history {
                        if let Some(fid) = self.current_sha_frame_id(ctx) {
                            ctx.core
                                .vm_aux
                                .flags
                                .insert((self.target, format!("l11_sha_dodge_ok:{fid}:{shan}")), 1);
                        }
                    }
                    super::respond::note_longdan_sha_source(ctx, self.target, self.source);
                    crate::kernel::emit_card_responded_as(ctx, self.target, shan, CardKind::Shan);
                    releiji_on_shan(ctx, self.target);
                    return Some(self.after_sha_dodged(ctx));
                }
            }
            if ctx.core.player(self.target).alive {
                return Some(self.sadeng_continue_to_damage(ctx));
            }
            ctx.emit(EventData::ShaDodged {
                card: self.card,
                source: self.source,
                target: self.target,
            });
            return Some(self.finish_target_sha_end(ctx, false));
        }
        // 雌雄双股剑（批③-b 第八把；逐技对照批一 v75 消除首张差异）：
        // 弃牌/令摸恢复——上游 chooseToDiscard 一张手牌、可取消（取消=
        // 令来源摸 1），随后照常进入防具/出闪流程。
        if self.asking_cixiong {
            self.asking_cixiong = false;
            self.asked = false;
            let need = cixiong_need(ctx, self.source);
            let picks: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) if ctx.core.player(self.target).hand.contains(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picks.len() == need {
                for c in picks {
                    ctx.discard_from_hand(self.target, c);
                    gain_if_shenhanxueyin(ctx, self.source, c);
                }
            } else if ctx.core.player(self.source).alive {
                // R05：取消=摸 1 走真实 R 帧——先 Yield 让调度器完成 R 子树
                //（开放询问会暂停调度），再由 !asked 分支重开出闪询问。
                ctx.start_draw(self.source, 1);
                return Some(Flow::Yield);
            }
            // 落入下方 !asked 分支重开出闪询问（防具判定已由
            // armor_judged 标记幂等）。
        }
        // 麒麟弓（批③-b 第二把）：弃马询问恢复——按应答弃马，随后结算伤害（拂尘荡魔可再弃一匹）。
        if self.asking_qilin {
            self.asking_qilin = false;
            let cid = self.card.unwrap_or(0);
            // 空应答 = 来源放弃弃马。
            let mut discarded_one = false;
            if let Some(ChoiceOption::Card(horse)) = ctx.answer().first().copied() {
                if ctx.core.player(self.target).equips.contains(&horse) {
                    ctx.core
                        .player_mut(self.target)
                        .equips
                        .retain(|x| *x != horse);
                    ctx.to_discard(horse);
                    gain_if_shenhanxueyin(ctx, self.source, horse);
                    discarded_one = true;
                }
            }
            if discarded_one
                && ctx.catalog.vm_card_history
                && crate::rvm_bridge::modifiers::has_fuchendangmo(
                    ctx.core,
                    ctx.catalog,
                    self.source,
                )
                && ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(self.source, format!("qilin_fuchen_used:{cid}")))
                    .copied()
                    .unwrap_or(0)
                    == 0
            {
                let rem_horses = self.qilin_horses(ctx);
                if !rem_horses.is_empty() && ctx.core.player(self.target).alive {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((self.source, format!("qilin_fuchen_used:{cid}")), 1);
                    self.asking_qilin = true;
                    return Some(Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "qilin_discard_horse",
                            options: rem_horses,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    });
                }
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(self.source, format!("qilin_fuchen_used:{cid}")));
            self.qilin_horse = None;
            return Some(self.emit_sha_damage(ctx));
        }
        None
    }

    pub(super) fn clear_sha_target_judge_flags(&self, ctx: &mut Ctx<'_>) {
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "l10_scschihe_block_suits".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "xj_sha_req_suit".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.target, "xj_longxi_sha_guard".to_string()));
        if let Some(fid) = self.current_sha_frame_id(ctx) {
            for key in [
                "sha_cike_done",
                "sha_end_settled",
                "sha_pending_end",
                "sha_frame_damage_bonus",
                "l11_guanshi_dodged_pending",
                "sha_tianxian_done",
                "sha_longxiang_done",
                "sha_yueren_done",
                "u05_sha_hit_resume",
            ] {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.source, format!("{key}:{fid}")));
            }
            ctx.core.vm_aux.flags.remove(&(
                self.source,
                format!("u05_sadeng_tried:{fid}:{}", self.target),
            ));
            ctx.core.vm_aux.flags.remove(&(
                self.source,
                format!("u05_guiyanfadao_done:{fid}:{}", self.target),
            ));
            ctx.core.vm_aux.flags.remove(&(
                self.source,
                format!("u05_guiyanfadao_unhurt:{fid}:{}", self.target),
            ));
            ctx.core
                .vm_aux
                .flags
                .remove(&(self.target, format!("sha_cancelled_target:{fid}")));
            ctx.core.vm_aux.flags.remove(&(
                self.source,
                format!("sha_ow_begin_done:{fid}:{}", self.target),
            ));
            ctx.core.vm_aux.flags.remove(&(
                self.source,
                format!("sha_ow_bfengshi_direct:{fid}:{}", self.target),
            ));
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(self.source, format!("u05_sadeng_handled:{}", self.target)));
        ctx.core.vm_aux.flags.remove(&(
            self.source,
            format!("u05_guiyanfadao_active_fid:{}", self.target),
        ));
    }

    pub(super) fn resolve_tianxian(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
        let done_key = (self.source, format!("sha_tianxian_done:{fid}"));
        if ctx.core.vm_aux.flags.contains_key(&done_key) {
            return None;
        }
        let tianxian = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_XJ_TIANXIAN_KEY)
            .is_some_and(|d| ctx.core.has_skill(self.source, d));
        if !tianxian || (ctx.core.deck.is_empty() && ctx.core.discard.is_empty()) {
            return None;
        }
        ctx.core.vm_aux.flags.insert(done_key, 1);
        let judged =
            ctx.draw_judge_card_with_elo(self.source, crate::kernel::EloJudgePref::RedPositive);
        let (suit, _) = ctx.inline_judge_suit_rank(self.target, judged);
        let black = matches!(suit, crate::catalog::Suit::Spade | crate::catalog::Suit::Club);
        let settled_spawned = ctx.settle_inline_judge_card(self.target, suit, judged);
        if black {
            return Some(self.advance_target(ctx));
        }
        if settled_spawned {
            self.asked = false;
            return Some(Flow::Yield);
        }
        None
    }

    pub(super) fn resolve_tieji(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        let tieji = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::TIE_JI)
            .is_some_and(|d| ctx.core.has_skill(self.source, d));
        if tieji
            && self.tieji_red.is_none()
            && (!ctx.core.deck.is_empty() || !ctx.core.discard.is_empty())
        {
            let judged = ctx.draw_judge_card_with_elo(
                self.source,
                crate::kernel::EloJudgePref::RedPositive,
            );
            let (suit, _) = ctx.inline_judge_suit_rank(self.source, judged);
            let red = matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
            self.tieji_red = Some(red);
            if ctx.settle_inline_judge_card(self.source, suit, judged) {
                self.asked = false;
                return Some(Flow::Yield);
            }
        }
        None
    }

    pub(super) fn resolve_yueren(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
        let done_key = (self.source, format!("sha_yueren_done:{fid}"));
        if ctx.core.vm_aux.flags.contains_key(&done_key) {
            return None;
        }
        let has_yueren = ctx
            .catalog
            .skill_by_name("yueren")
            .is_some_and(|d| ctx.core.has_skill(self.source, d));
        if !has_yueren
            || !ctx.core.player(self.target).alive
            || (ctx.core.deck.is_empty() && ctx.core.discard.is_empty())
        {
            return None;
        }
        ctx.core.vm_aux.flags.insert(done_key, 1);
        let judged = ctx.draw_judge_card_with_elo(self.source, crate::kernel::EloJudgePref::Yueren);
        let (suit, _) = ctx.inline_judge_suit_rank(self.source, judged);
        let black = matches!(suit, crate::catalog::Suit::Spade | crate::catalog::Suit::Club);
        if black {
            let mut pool: Vec<crate::ids::CardId> =
                ctx.core.player(self.target).hand.iter().copied().collect();
            pool.extend(ctx.core.player(self.target).equips.iter().copied());
            if !pool.is_empty() {
                let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                let c = pool[j];
                if ctx.core.player(self.target).hand.contains(&c) {
                    ctx.take_hand_card(self.target, c);
                } else {
                    ctx.core.player_mut(self.target).equips.retain(|x| *x != c);
                    if ctx.catalog.resolved_card_kind(ctx.core, c).is_equip() {
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    }
                }
                ctx.to_discard(c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.target,
                    cards: vec![c],
                });
            }
        } else if ctx.core.player(self.source).alive {
            ctx.start_draw(self.source, 1);
        }
        if ctx.settle_inline_judge_card(self.source, suit, judged) {
            self.asked = false;
            return Some(Flow::Yield);
        }
        None
    }
}
