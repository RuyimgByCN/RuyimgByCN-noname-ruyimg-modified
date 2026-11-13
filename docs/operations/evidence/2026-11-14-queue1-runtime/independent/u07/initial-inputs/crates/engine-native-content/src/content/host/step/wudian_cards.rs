//! 武典（wudian）U07 宿主技：屯田·DIY、率言、筑城守卫、抗音。

use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 屯田·DIY：自选一张装备牌（手/场）弃之，牌堆随机获得一张
            // 装备牌（实体发现模式；deck+discard 无装备则仅弃置）
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
                    // 弃置（装备栏走 revoke）
                    if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.to_discard(c);
                    } else {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 实体发现：牌堆随机一张装备牌入手（弃堆兜底）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| ctx.catalog.card_slot_of(ctx.core, *x).is_some())
                        .collect();
                    if !pool.is_empty() {
                        let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let got = pool.swap_remove(idx);
                        if let Some(pos) = ctx.core.deck.iter().position(|x| *x == got) {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx.core.discard.iter().position(|x| *x == got) {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(got);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![got],
                        });
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
            HostBehavior::WudianZhuchengGuard => match self.stage {
                // 筑城守卫（U07 批三 v113）：Act 问使用者——选一张=弃
                // 筑城张数张（登记差异：上游一次性选 n 张，原生首张自选
                // +其余随机代抽）；Pass=杀无效（帧 cancelled + 杀归还手
                // 牌，Intrinsic 阶段按 cancelled 跳过结算）。
                HostStage::Act => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(
                        ctx.core
                            .player(attacker)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: attacker,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_ZHUCHENG_GUARD_ASK,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let answered = ctx.answer().first().copied();
                    match answered {
                        Some(ChoiceOption::Card(c)) => {
                            let n = ctx.core.player(self.owner).zhucheng.len();
                            if ctx.core.player(attacker).hand.contains(&c) {
                                ctx.discard_from_hand(attacker, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: attacker,
                                    cards: vec![c],
                                });
                            }
                            let mut extra = n.saturating_sub(1);
                            while extra > 0 && !ctx.core.player(attacker).hand.is_empty() {
                                let idx = ctx
                                    .core
                                    .rng
                                    .gen_range(ctx.core.player(attacker).hand.len() as u32)
                                    as usize;
                                let c = ctx.core.player(attacker).hand[idx];
                                ctx.discard_from_hand(attacker, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: attacker,
                                    cards: vec![c],
                                });
                                extra -= 1;
                            }
                            Flow::Done
                        }
                        _ => {
                            // 杀无效：帧 cancelled + 杀归还手牌（Intrinsic
                            // 阶段按 cancelled 跳过建 ShaResolve）
                            if let Some(c) = self.card {
                                ctx.core.table.retain(|x| *x != c);
                                ctx.core.player_mut(attacker).hand.push(c);
                            }
                            let fid = ctx.current_event_frame_id();
                            if let Some(f) = ctx.res.frame_mut(fid) {
                                if let EventData::CardUsed(u) = &mut f.data {
                                    u.cancelled = true;
                                }
                            }
                            Flow::Done
                        }
                    }
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
            HostBehavior::WudianYouzhan => match self.stage {
                HostStage::Act => {
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
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_YOUZHAN_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.to_discard(c);
                    } else if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.emit(EventData::CardsDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    if let Some(sha_c) = self.card.take() {
                        ctx.core.table.retain(|x| *x != sha_c);
                        ctx.to_discard(sha_c);
                    }
                    let fid = ctx.current_event_frame_id();
                    if let Some(f) = ctx.res.frame_mut(fid) {
                        if let EventData::CardUsed(u) = &mut f.data {
                            u.cancelled = true;
                        }
                    }
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let defender = self.targets.first().copied().unwrap_or(self.owner);
                    if ctx.core.player(attacker).alive {
                        let hand_shas: Vec<ChoiceOption> = ctx
                            .core
                            .player(attacker)
                            .hand
                            .iter()
                            .filter(|x| ctx.catalog.card_kind(**x).is_sha_like())
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        if !hand_shas.is_empty() {
                            self.stage = HostStage::CardsAsk;
                            return Flow::WaitChoice {
                                observer: attacker,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_YOUDISHENRU_SHA,
                                    options: hand_shas,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            };
                        }
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(defender),
                            attacker,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                HostStage::CardsAsk => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let defender = self.targets.first().copied().unwrap_or(self.owner);
                    if let Some(ChoiceOption::Card(new_sha)) = ctx.answer().first().copied() {
                        if ctx.core.player(attacker).hand.contains(&new_sha) {
                            ctx.core.player_mut(attacker).hand.retain(|x| *x != new_sha);
                            ctx.core.table.push(new_sha);
                            ctx.log(LogEvent::CardPlayed {
                                player: attacker,
                                card: new_sha,
                                targets: vec![defender],
                                presentation: None,
                            });
                            ctx.emit(EventData::CardUsed(crate::machine::card_use(
                                new_sha,
                                attacker,
                                vec![defender],
                            )));
                            return Flow::Done;
                        }
                    }
                    if ctx.core.player(attacker).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(defender),
                            attacker,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianChoudu => match self.stage {
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .chain(p.equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.dedup();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_CHOUDU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_CHOUDU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(c) = self.card.take() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.to_discard(c);
                    } else if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.emit(EventData::CardsDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    self.owner = t;
                    self.targets = vec![t];
                    self.def_key = crate::content::host::WUDIAN_DIAOBING_KEY.to_string();
                    self.behavior = HostBehavior::WudianDiaobingCard;
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianDiaobingCard => match self.stage {
                HostStage::Act => {
                    ctx.start_draw(self.owner, 1);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let reveal_n = ctx.core.alive_players().count().div_ceil(2).max(1);
                    self.cards.clear();
                    for _ in 0..reveal_n {
                        if let Some(cid) = ctx.core.deck.pop() {
                            self.cards.push(cid);
                        }
                    }
                    ctx.core.vm_aux.flags.insert(
                        (self.owner, "u07_diaobing_resolved".to_string()),
                        reveal_n as u32,
                    );
                    ctx.core.vm_aux.flags.insert(
                        (self.owner, "diaobingqianjiang_resolved".to_string()),
                        1,
                    );
                    self.targets.retain(|p| ctx.core.player(*p).alive);
                    if self.targets.is_empty() && ctx.core.player(self.owner).alive {
                        self.targets.push(self.owner);
                    }
                    while !self.targets.is_empty() && !self.cards.is_empty() {
                        let t = self.targets.remove(0);
                        if ctx.core.player(t).alive && !ctx.core.player(t).hand.is_empty() {
                            self.target = Some(t);
                            self.stage = HostStage::CardsAsk;
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .player(t)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            return Flow::WaitChoice {
                                observer: t,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_DIAOBING_SWAP,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            };
                        }
                    }
                    for rem in self.cards.drain(..) {
                        ctx.to_discard(rem);
                    }
                    if let Some(c) = self.card.take() {
                        if ctx.core.table.contains(&c) {
                            ctx.core.table.retain(|&x| x != c);
                            ctx.to_discard(c);
                        }
                    }
                    Flow::Done
                }
                HostStage::CardsAsk => {
                    if let Some(t) = self.target.take() {
                        if let Some(ChoiceOption::Card(hand_c)) = ctx.answer().first().copied() {
                            if ctx.core.player(t).hand.contains(&hand_c) && !self.cards.is_empty() {
                                let market_c = self.cards.remove(0);
                                ctx.core.player_mut(t).hand.retain(|&x| x != hand_c);
                                ctx.core.player_mut(t).hand.push(market_c);
                                self.cards.push(hand_c);
                            }
                        }
                    }
                    while !self.targets.is_empty() && !self.cards.is_empty() {
                        let next_t = self.targets.remove(0);
                        if ctx.core.player(next_t).alive
                            && !ctx.core.player(next_t).hand.is_empty()
                        {
                            self.target = Some(next_t);
                            self.stage = HostStage::CardsAsk;
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .player(next_t)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            return Flow::WaitChoice {
                                observer: next_t,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_DIAOBING_SWAP,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            };
                        }
                    }
                    for rem in self.cards.drain(..) {
                        ctx.to_discard(rem);
                    }
                    if let Some(c) = self.card.take() {
                        if ctx.core.table.contains(&c) {
                            ctx.core.table.retain(|&x| x != c);
                            ctx.to_discard(c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
