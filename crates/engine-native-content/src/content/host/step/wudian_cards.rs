//! 武典（wudian）U07 宿主技：屯田·DIY、率言、筑城守卫、抗音。

use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 屯田·DIY（玩点论杀/character/skill.js:776-811）：自选一张装备牌（手/场）
            // 弃置并摸一张牌（上游 player.recast(cards) 同口径）。
            HostBehavior::WudianDiyjuntun => match self.stage {
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> = p
                        .equips
                        .iter()
                        .chain(p.hand.iter())
                        .copied()
                        .filter(|c| ctx.catalog.card_slot_of(ctx.core, *c).is_some())
                        .map(ChoiceOption::Card)
                        .collect();
                    options.dedup();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "wudian_diyjuntun_recast",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c) = picked else { return Flow::Done };
                    if !discard_owned_card_once(ctx, self.owner, c) {
                        return Flow::Done;
                    }
                    // 上游 `player.recast`（player.js:1641-1657）：先结算 CardsLost，再从牌堆顶摸 1 张牌
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    if ctx.core.player(self.owner).alive {
                        ctx.start_draw(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianShuaiyan => match self.stage {
                // 率言（U07 批二，规则 v112）：Act 问回复者二选一 →
                // SelectCard：Tag0 费诗摸 1 / Tag1 问回复者弃一张 →
                // TargetChoice 弃置。
                HostStage::Act => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: r,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_SHUAIYAN_CHOICE,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => {
                            ctx.start_draw(self.owner, 1);
                            Flow::Done
                        }
                        Some(ChoiceOption::Tag(1)) => {
                            let mut options: Vec<ChoiceOption> = ctx
                                .core
                                .player(r)
                                .hand
                                .iter()
                                .chain(ctx.core.player(r).equips.iter())
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            options.dedup();
                            if options.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::TargetChoice;
                            Flow::WaitChoice {
                                observer: r,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_SHUAIYAN_DISCARD,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        _ => Flow::Done,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(r).hand.contains(&c) {
                            ctx.discard_from_hand(r, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: r,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(r).equips.contains(&c) {
                            ctx.core.player_mut(r).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, r, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: r,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 筑城守卫（玩点论杀/character/skill.js:218-251）：trigger { target: "shaBefore" }
            // 目标有筑牌时，来源可自选弃置 num=zhucheng.len() 张 "he" 牌（手牌+装备）；
            // 若不足 num 张或取消则仅取消对当前筑城持有者目标的杀（若目标清空则杀入弃牌堆，不回手）。
            HostBehavior::WudianZhuchengGuard => match self.stage {
                HostStage::Act => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let num = ctx.core.player(self.owner).zhucheng.len();
                    let options = wudian_zhucheng_he_options(ctx.core, attacker);
                    if num == 0 || options.len() < num {
                        cancel_active_sha_target(ctx, attacker, self.owner);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: attacker,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_ZHUCHENG_GUARD_ASK,
                            options,
                            min: num,
                            max: num,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let num = ctx.core.player(self.owner).zhucheng.len();
                    let mut picks: Vec<CardId> = Vec::new();
                    for o in ctx.answer() {
                        if let ChoiceOption::Card(c) = o {
                            if !picks.contains(c)
                                && (ctx.core.player(attacker).hand.contains(c)
                                    || ctx.core.player(attacker).equips.contains(c))
                            {
                                picks.push(*c);
                            }
                        }
                    }
                    if num == 0 || picks.len() != num {
                        cancel_active_sha_target(ctx, attacker, self.owner);
                        return Flow::Done;
                    }
                    let mut discarded = Vec::with_capacity(picks.len());
                    for c in picks {
                        if discard_owned_card_once(ctx, attacker, c) {
                            discarded.push(c);
                        }
                    }
                    if !discarded.is_empty() {
                        ctx.emit(EventData::CardsDiscarded {
                            player: attacker,
                            cards: discarded,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianKangyin => match self.stage {
                // 抗yin（U07 批四，规则 v114）：Act 问目标 → SelectCard
                // 失去 1 体力 + 被弃者选弃一张 → TargetChoice 分支——已
                // 受伤：基本=选至多 X 名各摸 1（CardsAsk），非基本=授
                // kangyin2（回合内）；未受伤结束。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_KANGYIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    // 失去 1 点体力（上游 loseHp 同口径）
                    let hp0 = ctx.core.player(self.owner).hp;
                    ctx.core.player_mut(self.owner).hp = hp0 - 1;
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
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_KANGYIN_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    } else if ctx.core.player(t).equips.contains(&c) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    let p = ctx.core.player(self.owner);
                    if p.hp >= p.max_hp as i32 {
                        return Flow::Done;
                    }
                    let x = (p.max_hp - p.hp as u32).max(1);
                    let is_basic = yangpijuan_class(ctx.catalog.card_kind(c)) == Some(0);
                    if is_basic {
                        let options: Vec<ChoiceOption> =
                            ctx.core.alive_players().map(ChoiceOption::Player).collect();
                        self.cards = vec![x as CardId];
                        self.stage = HostStage::CardsAsk;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: WUDIAN_KANGYIN_DRAW,
                                options,
                                min: 1,
                                max: x as usize,
                                allow_cancel: true,
                            },
                        }
                    } else {
                        ctx.core.turn.kangyin2 = Some((self.owner, x));
                        Flow::Done
                    }
                }
                HostStage::CardsAsk => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    for p in picked {
                        ctx.start_draw(p, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianDuoqi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .zhucheng
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_DUOQI_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(self.owner).zhucheng.contains(&c) {
                            ctx.core.player_mut(self.owner).zhucheng.retain(|x| *x != c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                            ctx.core.turn.skip_play_phase = true;
                            if ctx.core.turn.phase == crate::event::Phase::Play {
                                ctx.emit(EventData::Phase {
                                    player: ctx.core.turn.current,
                                    phase: crate::event::Phase::Play,
                                    begin: false,
                                });
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 诱战（玩点论杀/character/skill.js:422-447）：逐目标 shaBefore 独立检查距离<=1，
            // 取消仅该目标；真实取消原杀不造成额外伤害；反击杀循环直到 shaDamage 方结束（被闪且停止则受1伤）；去重日志。
            HostBehavior::WudianYouzhan => loop {
                match self.stage {
                    HostStage::Act => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        if !ctx.core.player(self.owner).alive
                            || !ctx.core.player(attacker).alive
                            || self.targets.is_empty()
                        {
                            return Flow::Done;
                        }
                        let defender = self.targets[0];
                        let in_dist = defender == self.owner
                            || ctx.core.computed_distance(ctx.catalog, self.owner, defender) <= 1;
                        if !ctx.core.player(defender).alive
                            || !in_dist
                            || find_active_sha_frame_id(ctx, attacker, defender).is_none()
                        {
                            self.targets.remove(0);
                            continue;
                        }
                        let p = ctx.core.player(self.owner);
                        let mut options: Vec<ChoiceOption> = p
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| ctx.catalog.card_kind(*c).is_equip())
                            .chain(p.equips.iter().copied())
                            .map(ChoiceOption::Card)
                            .collect();
                        options.dedup();
                        if options.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::SelectCard;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: WUDIAN_YOUZHAN_DISCARD,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: true,
                            },
                        };
                    }
                    HostStage::SelectCard => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        let Some(defender) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                            self.targets.remove(0);
                            self.stage = HostStage::Act;
                            continue;
                        };
                        if find_active_sha_frame_id(ctx, attacker, defender).is_none() {
                            self.targets.remove(0);
                            self.stage = HostStage::Act;
                            continue;
                        }
                        if !discard_owned_card_once(ctx, self.owner, c) {
                            self.targets.remove(0);
                            self.stage = HostStage::Act;
                            continue;
                        }
                        ctx.emit(EventData::CardsDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        // 上游 `character/skill.js:428-445`: cost 弃置及 CardsDiscarded/CardsLost 结算完成后，
                        // 再于 TargetChoice 发出虚拟【诱敌深入】（经无懈窗口，未无懈时在 HuanrenPick 取消原杀）。
                        self.stage = HostStage::TargetChoice;
                        return Flow::Yield;
                    }
                    HostStage::TargetChoice => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        if self.targets.is_empty() {
                            self.stage = HostStage::Act;
                            continue;
                        }
                        let defender = self.targets.remove(0);
                        self.stage = HostStage::Act;
                        if !ctx.core.player(attacker).alive || !ctx.core.player(defender).alive {
                            continue;
                        }
                        let Some(sha_fid) = find_active_sha_frame_id(ctx, attacker, defender) else {
                            continue;
                        };
                        ctx.core.vm_aux.flags.insert(
                            (defender, "yd_pending_source".to_string()),
                            u32::from(attacker) + 1,
                        );
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((defender, "yd_pending_sha_frame".to_string()), sha_fid);
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: defender,
                            targets: Vec::new(),
                            nature: crate::event::DamageNature::Normal,
                            use_as: Some(CardKind::Youdishenru),
                            cancelled: false,
                        }));
                        return Flow::Yield;
                    }
                    HostStage::HuanrenPick => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        let Some(defender) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        if let Some(sha_fid) = ctx
                            .core
                            .vm_aux
                            .flags
                            .remove(&(defender, "yd_pending_sha_frame".to_string()))
                        {
                            cancel_active_sha_target(ctx, attacker, defender);
                            if self.card.is_some() {
                                ctx.core
                                    .vm_aux
                                    .flags
                                    .insert((defender, format!("yd_cancelled:{sha_fid}")), 1);
                            }
                        }
                        if let Some(yd) = self.card {
                            if ctx.core.table.contains(&yd) {
                                ctx.core.table.retain(|&x| x != yd);
                                ctx.to_discard(yd);
                            }
                        }
                        if ctx.core.player(attacker).alive && ctx.core.player(defender).alive {
                            return self.prompt_youdishenru_sha_or_backlash(
                                ctx, attacker, defender, self.card,
                            );
                        }
                        self.targets.remove(0);
                        self.stage = HostStage::Act;
                    }
                    HostStage::CardsAsk => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        let Some(defender) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        if let Some(ChoiceOption::Card(new_sha)) = ctx.answer().first().copied() {
                            if ctx.core.player(attacker).alive && ctx.core.player(defender).alive {
                                if let Some((use_as, is_mutou)) = crate::kernel::sha_card_use_as(
                                    ctx.core,
                                    ctx.catalog,
                                    attacker,
                                    new_sha,
                                ) {
                                    if is_mutou {
                                        if let Some(def) = ctx.catalog.skill_by_name(
                                            crate::vocabulary::host::GJ_MUTOU_SKILL_KEY,
                                        ) {
                                            if let Some(inst) =
                                                ctx.core.skills.get_mut(&(attacker, def))
                                            {
                                                inst.once_used = true;
                                            }
                                        }
                                    }
                                    ctx.core
                                        .vm_aux
                                        .flags
                                        .insert((attacker, "vm_va_sha_no_count".to_string()), 1);
                                    ctx.core.vm_aux.flags.insert(
                                        (
                                            attacker,
                                            format!("youdishenru_sha:{defender}:{new_sha}"),
                                        ),
                                        1,
                                    );
                                    self.cards = vec![new_sha];
                                    self.stage = HostStage::DrawSettle;
                                    ctx.emit(EventData::CardUsed(super::sha_card_use_data(
                                        ctx,
                                        attacker,
                                        new_sha,
                                        vec![defender],
                                        use_as,
                                    )));
                                    return Flow::Yield;
                                }
                            }
                        }
                        self.targets.remove(0);
                        self.stage = HostStage::Act;
                        if ctx.core.player(attacker).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(defender),
                                attacker,
                                1,
                                crate::event::DamageNature::Normal,
                                self.card,
                            )));
                            return Flow::Yield;
                        }
                    }
                    HostStage::DrawSettle => {
                        let Some(attacker) = self.target else {
                            return Flow::Done;
                        };
                        let Some(defender) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        let last_sha = self.cards.pop();
                        let loop_continues = last_sha.is_some_and(|s| {
                            ctx.core
                                .vm_aux
                                .flags
                                .remove(&(attacker, format!("youdishenru_sha:{defender}:{s}")))
                                .is_some()
                        });
                        if loop_continues
                            && ctx.core.player(attacker).alive
                            && ctx.core.player(defender).alive
                        {
                            return self.prompt_youdishenru_sha_or_backlash(
                                ctx, attacker, defender, self.card,
                            );
                        }
                        self.targets.remove(0);
                        self.stage = HostStage::Act;
                    }
                    _ => return Flow::Done,
                }
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }

    fn prompt_youdishenru_sha_or_backlash(
        &mut self,
        ctx: &mut Ctx<'_>,
        attacker: PlayerId,
        defender: PlayerId,
        damage_card: Option<CardId>,
    ) -> Flow {
        let shas = crate::kernel::usable_sha_options(ctx.core, ctx.catalog, attacker, defender);
        if !shas.is_empty() {
            self.stage = HostStage::CardsAsk;
            return Flow::WaitChoice {
                observer: attacker,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: WUDIAN_YOUDISHENRU_SHA,
                    options: shas,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        self.targets.remove(0);
        self.stage = HostStage::Act;
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(defender),
            attacker,
            1,
            crate::event::DamageNature::Normal,
            damage_card,
        )));
        Flow::Yield
    }
}

/// 筑城2 `he` 候选（Step 与 Asks 同源）：手牌 + 装备区去重。
pub(in crate::content::host) fn wudian_zhucheng_he_options(
    core: &crate::state::GameStateCore,
    player: PlayerId,
) -> Vec<ChoiceOption> {
    let p = core.player(player);
    let mut options: Vec<ChoiceOption> = p
        .hand
        .iter()
        .chain(p.equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect();
    options.dedup();
    options
}

pub(super) fn discard_owned_card_once(ctx: &mut Ctx<'_>, owner: PlayerId, c: CardId) -> bool {
    if ctx.core.player(owner).equips.contains(&c) {
        ctx.core.player_mut(owner).equips.retain(|x| *x != c);
        ctx.core.declared_slots.remove(&c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
        ctx.to_discard(c);
        ctx.note_cards_lost(owner, &[c]);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![c],
            origins: vec![crate::event::LostOrigin::Equip],
        });
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: vec![c],
        });
        true
    } else if ctx.core.player(owner).hand.contains(&c) {
        // discard_from_hand 已发射 CardsLost 与 LogEvent::CardDiscarded
        ctx.discard_from_hand(owner, c);
        true
    } else {
        false
    }
}

fn cancel_active_sha_target(ctx: &mut Ctx<'_>, attacker: PlayerId, defender: PlayerId) {
    if let Some(fid) = find_active_sha_frame_id(ctx, attacker, defender) {
        let mut discard_sha = None;
        if let Some(f) = ctx.res.frame_mut(fid) {
            if let EventData::CardUsed(u) = &mut f.data {
                u.targets.retain(|&x| x != defender);
                if u.targets.is_empty() {
                    u.cancelled = true;
                    discard_sha = u.card;
                }
            }
        }
        if let Some(sha_c) = discard_sha {
            ctx.core.table.retain(|x| *x != sha_c);
            ctx.to_discard(sha_c);
        }
    }
}

fn find_active_sha_frame_id(
    ctx: &Ctx<'_>,
    attacker: PlayerId,
    defender: PlayerId,
) -> Option<crate::ids::EventId> {
    ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(u)
            if !u.cancelled
                && u.player == attacker
                && u.targets.contains(&defender)
                && u.effective_kind(ctx.catalog).is_sha_like() =>
        {
            Some(f.id)
        }
        _ => None,
    })
}
