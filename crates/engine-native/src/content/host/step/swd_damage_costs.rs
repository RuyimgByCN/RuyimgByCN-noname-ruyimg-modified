use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_costs(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 挟雷：Act 弃一张牌（he 可取消=放弃）→ TargetChoice 选合法
            // 目标（use 排除原目标；respond 路径登记差异：事件载荷无来源
            // 不排除）→ 1 点雷伤
            HostBehavior::SwdXielei => match self.stage {
                HostStage::Act => {
                    // 记录排除面：use=原目标列表；respond=载荷无来源不排除
                    match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                        Some(crate::event::EventData::CardUsed(u)) => {
                            self.targets = u.targets.clone();
                        }
                        _ => self.targets.clear(),
                    }
                    let mut pool: Vec<CardId> =
                        ctx.core.player(self.owner).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(self.owner).equips.iter().copied());
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xielei_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let in_hand = ctx.core.player(self.owner).hand.contains(&picked);
                    if in_hand {
                        ctx.take_hand_card(self.owner, picked);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != picked);
                        if ctx.catalog.card_kind(picked).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                picked,
                            );
                        }
                    }
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !self.targets.contains(p))
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "xielei_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Thunder,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 断义：Act 弃两张杀（代价，selectCard 2 非默认弃置面——
            // 机器显式弃）→ SelectCard 消费 → TargetChoice 选目标 →
            // 1 伤 + 目标随机弃 X 张 he（X=其已损失体力）
            HostBehavior::SwdDuanyi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duanyi_discard",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.len() < 2 {
                        return Flow::Done;
                    }
                    for c in picks {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        }
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "duanyi_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                    let x =
                        (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as usize;
                    let mut pool: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(t).equips.iter().copied());
                    for _ in 0..x {
                        if pool.is_empty() {
                            break;
                        }
                        let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let c = pool.swap_remove(j);
                        let in_hand = ctx.core.player(t).hand.contains(&c);
                        if in_hand {
                            ctx.take_hand_card(t, c);
                        } else {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            if ctx.catalog.card_kind(c).is_equip() {
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            }
                        }
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 冰箭：SelectCard 弃黑色杀 → TargetChoice 有手牌目标 → 闪全
            // 弃或 1 雷伤
            HostBehavior::SwdBingjian => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c).is_sha_like()
                                && matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                )
                        })
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
                            prompt: "bingjian_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&picked) {
                        ctx.discard_from_hand(self.owner, picked);
                    } else {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "bingjian_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 亮手牌（登记展示）：闪全弃，否则 1 雷伤
                    let shans: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .collect();
                    if shans.is_empty() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Thunder,
                            None,
                        )));
                    } else {
                        for c in shans {
                            ctx.discard_from_hand(t, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 袭西·凤钰面：SelectCard 弃一张 he → TargetChoice 攻击距离内
            // 且体力不小于自己的角色 → 1 伤
            HostBehavior::SwdShangxi => match self.stage {
                HostStage::Act => {
                    let mut pool: Vec<CardId> =
                        ctx.core.player(self.owner).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(self.owner).equips.iter().copied());
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shangxi_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let in_hand = ctx.core.player(self.owner).hand.contains(&picked);
                    if in_hand {
                        ctx.take_hand_card(self.owner, picked);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != picked);
                        if ctx.catalog.card_kind(picked).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                picked,
                            );
                        }
                    }
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                                && ctx.core.player(self.owner).hp <= ctx.core.player(*p).hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "shangxi_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 奥剑：SelectCard 弃 1..X 张手牌 → TargetChoice 攻击距离内
            // 至多 X 名 → 各 1 伤
            HostBehavior::SwdAojian => match self.stage {
                HostStage::Act => {
                    let x = (ctx.core.player(self.owner).max_hp as i32
                        - ctx.core.player(self.owner).hp)
                        .max(0) as usize;
                    if x == 0 || ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    let n = x.min(ctx.core.player(self.owner).hand.len());
                    self.card = Some(n as u32);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "aojian_discard",
                            options: ctx
                                .core
                                .player(self.owner)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect(),
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.is_empty() {
                        return Flow::Done;
                    }
                    for c in &picks {
                        if ctx.core.player(self.owner).hand.contains(c) {
                            ctx.discard_from_hand(self.owner, *c);
                        }
                    }
                    let n = picks.len();
                    self.cards = picks;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "aojian_targets",
                            options,
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    let n = self.cards.len();
                    if targets.is_empty() || targets.len() > n {
                        return Flow::Done;
                    }
                    for t in targets {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    self.cards.clear();
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 凝弦：确认 → SelectCards 弃任意张黑色 he 牌 → SelectPlayers
            // 等量目标 → 各受 1 伤
            HostBehavior::SwdNingxian => match self.stage {
                HostStage::Act => {
                    let blacks: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if blacks.is_empty() {
                        return Flow::Done;
                    }
                    self.targets.clear();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "ningxian_discard",
                            options: blacks,
                            min: 1,
                            max: 4,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.is_empty() {
                        return Flow::Done;
                    }
                    for c in &picks {
                        let in_hand = ctx.core.player(self.owner).hand.contains(c);
                        if in_hand {
                            ctx.take_hand_card(self.owner, *c);
                        } else {
                            ctx.core.player_mut(self.owner).equips.retain(|x| x != c);
                            if ctx.catalog.card_kind(*c).is_equip() {
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    self.owner,
                                    *c,
                                );
                            }
                        }
                        ctx.to_discard(*c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![*c],
                        });
                    }
                    self.cards = picks.clone();
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|o| match o {
                            ChoiceOption::Player(p) => p,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<PlayerId>>();
                    if others.len() < picks.len() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "ningxian_targets",
                            options: others.into_iter().map(ChoiceOption::Player).collect(),
                            min: picks.len(),
                            max: picks.len(),
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picks = self.cards.clone();
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.len() != picks.len() {
                        return Flow::Done;
                    }
                    for t in targets {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    self.cards.clear();
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 头弹·麦星面：SelectCard 黑桃 he → TargetChoice 其他存活 →
            // 目标 1 火伤 + 自受害者起（含自己）座位序各弃一张 he
            //（确定性首张登记差异）
            HostBehavior::SwdToudan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(self.owner).equips.iter().copied())
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                            )
                        })
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
                            prompt: "toudan_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        if ctx.catalog.card_kind(c).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                        }
                        ctx.to_discard(c);
                    } else {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "toudan_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                    // 自受害者起座位序（含自己）各弃一张 he（首张）
                    let order = ctx.core.seat_order_from(t);
                    for p in order {
                        if p != self.owner && !ctx.core.player(p).alive {
                            continue;
                        }
                        let pick = ctx
                            .core
                            .player(p)
                            .hand
                            .first()
                            .copied()
                            .map(|c| (c, true))
                            .or_else(|| {
                                ctx.core
                                    .player(p)
                                    .equips
                                    .first()
                                    .copied()
                                    .map(|c| (c, false))
                            });
                        if let Some((c, in_hand)) = pick {
                            if in_hand {
                                ctx.discard_from_hand(p, c);
                            } else {
                                ctx.core.player_mut(p).equips.retain(|x| *x != c);
                                if ctx.catalog.card_kind(c).is_equip() {
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
                                }
                                ctx.to_discard(c);
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 荡平（武将面批 86）：SelectCard 弃一张 he → TargetChoice 受害
            // 者距离 ≤1 的另一名角色 → 1 伤
            HostBehavior::SwdDangping => match self.stage {
                HostStage::Act => {
                    // 受害者=触发帧伤害目标（上游 trigger.player 同口径）
                    self.target = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some(d.target),
                            _ => None,
                        });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(self.owner).equips.iter().copied())
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
                            prompt: "dangping_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        if ctx.catalog.card_kind(c).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                        }
                        ctx.to_discard(c);
                    }
                    // 受害者（本次伤害目标）
                    let victim = self.target;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && Some(*p) != victim
                                && victim
                                    .map(|v| ctx.core.computed_distance(ctx.catalog, v, *p) <= 1)
                                    .unwrap_or(false)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "dangping_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 活弹·舒莞儿面：SelectCard 红色手牌 → TargetChoice 1-2 名任一
            // 存活（1 名=2 火伤/2 名=各 1 火伤）→ 自失 1 体力（contentBefore
            // 同口径：先弃牌再失体力）
            HostBehavior::SwdHuodan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
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
                            prompt: "huodan_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !matches!(
                            ctx.catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
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
                            prompt: "huodan_target",
                            options,
                            min: 1,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() || targets.len() > 2 {
                        return Flow::Done;
                    }
                    // 先失去 1 体力（上游 contentBefore loseHp 同口径）
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                    if targets.len() == 1 {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            targets[0],
                            2,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    } else {
                        for t in targets {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 占星·肯迪面：SelectCards 弃任意张手牌 → 亮等量牌堆顶牌：
            // 方块摸 2/红桃回 1 或 +1 护甲/梅花各敌人随机弃 1/黑桃选一名
            // 角色受 1 无来源雷伤
            HostBehavior::SwdZhanxing => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "zhanxing_discard",
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.is_empty()
                        || picks
                            .iter()
                            .any(|c| !ctx.core.player(self.owner).hand.contains(c))
                    {
                        return Flow::Done;
                    }
                    let n = picks.len();
                    for c in picks {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    // 亮出等量牌堆顶牌（不足补余量不合口径——按实有）；
                    // 全部入弃堆（上游 discard 同口径）
                    let mut suits: Vec<crate::catalog::Suit> = Vec::new();
                    for _ in 0..n {
                        if ctx.core.deck.is_empty() {
                            if ctx.core.discard.is_empty() {
                                break;
                            }
                            let mut cards = std::mem::take(&mut ctx.core.discard);
                            ctx.core.rng.shuffle(&mut cards);
                            ctx.core.deck = cards;
                            ctx.log(LogEvent::DeckReshuffled);
                        }
                        let c = ctx.core.deck.pop().expect("zhanxing top");
                        suits.push(ctx.catalog.cards[c as usize].suit);
                        ctx.to_discard(c);
                    }
                    use crate::catalog::Suit;
                    // 方块：摸 2
                    if suits.contains(&Suit::Diamond) {
                        ctx.draw_cards(self.owner, 2);
                    }
                    // 红桃：已受伤回 1，否则 +1 护甲
                    if suits.contains(&Suit::Heart) {
                        let pl = ctx.core.player(self.owner);
                        if pl.hp < pl.max_hp as i32 {
                            ctx.recover(self.owner, 1, None);
                        } else {
                            ctx.change_hujia(self.owner, 1);
                        }
                    }
                    // 梅花：各敌人随机弃 1（敌对=非友；上游 getEnemies）
                    if suits.contains(&Suit::Club) {
                        let enemies: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != self.owner
                                    && !ctx.core.is_friend_of(ctx.catalog, self.owner, *p)
                            })
                            .collect();
                        for e in enemies {
                            let has_hand = !ctx.core.player(e).hand.is_empty();
                            let has_equip = !ctx.core.player(e).equips.is_empty();
                            if !has_hand && !has_equip {
                                continue;
                            }
                            let n = if has_hand {
                                ctx.core.player(e).hand.len()
                            } else {
                                ctx.core.player(e).equips.len()
                            };
                            let j = ctx.core.rng.gen_range(n as u32) as usize;
                            let c = if has_hand {
                                ctx.core.player_mut(e).hand.remove(j)
                            } else {
                                let c = ctx.core.player_mut(e).equips.remove(j);
                                if ctx.catalog.card_kind(c).is_equip() {
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, e, c);
                                }
                                c
                            };
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: e,
                                cards: vec![c],
                            });
                        }
                    }
                    // 黑桃：选一名角色受 1 无来源雷伤
                    if suits.contains(&Suit::Spade) {
                        let options: Vec<ChoiceOption> =
                            ctx.core.alive_players().map(ChoiceOption::Player).collect();
                        if !options.is_empty() {
                            self.stage = HostStage::TargetChoice;
                            return Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: "zhanxing_target",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            };
                        }
                    }
                    Flow::Done
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                None,
                                t,
                                1,
                                crate::event::DamageNature::Thunder,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 断月：SelectCard 选装备牌（手/装备区）→ TargetChoice 选其
            // 他角色 → Act 弃牌+造成 1 点伤害（牌留区到末段）
            HostBehavior::SwdDuanyue => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| ctx.catalog.card_slot_of(ctx.core, **c).is_some())
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duanyue_card",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收牌应答：存 cards 后前移 Act 问目标（每臂一答铁律）
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c) = picked else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "duanyue_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 收目标应答后结算（牌在 SelectCard 已存 cards）
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let (Some(discarded), Some(t)) = (self.cards.first().copied(), self.target)
                    else {
                        return Flow::Done;
                    };
                    // 取消任一段=无效果（牌留区）
                    if !ctx.core.player(self.owner).hand.contains(&discarded)
                        && !ctx.core.player(self.owner).equips.contains(&discarded)
                    {
                        return Flow::Done;
                    }
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
