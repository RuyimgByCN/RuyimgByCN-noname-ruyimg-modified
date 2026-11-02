use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianLonghuo => loop {
                // 龙火（U05 xianjian 姜城，规则 v157）：座位序对所有存活
                // 角色各 1 点火伤（含自己；上游 redo 逐目标结算——逐目标
                // Yield 后重查存活，传导致死者不再向后结算）。target 兼
                // 作装填标记（None=未装填）。
                if self.target.is_none() {
                    self.target = Some(self.owner);
                    self.targets = (0..ctx.core.players.len() as u8).collect();
                }
                let Some(t) = self.targets.first().copied() else {
                    return Flow::Done;
                };
                self.targets.remove(0);
                if !ctx.core.player(t).alive {
                    continue;
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    1,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                return Flow::Yield;
            },
            HostBehavior::XianjianMinsha => match self.stage {
                // 敏杀（U05 xianjian 龙幽，规则 v159）：弃两张 he 牌（必
                // 选）→ 指定体力>1 的其他角色受 1 雷伤 → 距其 1 内其他
                // 角色（除自己）按座位序各随机弃一张（手牌优先退随机装
                // 备，蓝格怪衣同款登记差异；逐目标 Yield 重查存活）。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_MINSHA_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    // 此刻离区弃置（SelectCards 应答已校验归属）
                    for c in &picked {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked,
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| *t != self.owner && ctx.core.player(*t).hp > 1)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_MINSHA_TARGET,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入：收目标应答
                    if self.target.is_none() {
                        self.target = Some(self.owner);
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        self.targets = picked;
                    } else if let Some(t) = self.card.map(|x| x as PlayerId) {
                        // 阶段 B：上一主目标已结算——其距 1 内随机弃队列
                        self.card = None;
                        let neighbors: Vec<PlayerId> = ctx
                            .core
                            .seat_order_from(t)
                            .into_iter()
                            .filter(|p| {
                                *p != t
                                    && *p != self.owner
                                    && ctx.core.player(*p).alive
                                    && ctx.core.computed_distance(ctx.catalog, t, *p) <= 1
                            })
                            .collect();
                        for x in neighbors {
                            if !ctx.core.player(x).alive {
                                continue;
                            }
                            if !ctx.core.player(x).hand.is_empty() {
                                let n2 = ctx.core.player(x).hand.len();
                                let i2 = ctx.core.rng.gen_range(n2 as u32) as usize;
                                let c = ctx.core.player(x).hand[i2];
                                ctx.discard_from_hand(x, c);
                            } else if !ctx.core.player(x).equips.is_empty() {
                                let n2 = ctx.core.player(x).equips.len();
                                let j2 = ctx.core.rng.gen_range(n2 as u32) as usize;
                                let c = ctx.core.player_mut(x).equips.remove(j2);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, x, c);
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: x,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                    // 阶段 A：下一主目标雷伤
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    self.card = Some(t as CardId);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLinghuo => {
                // 灵火（U05 xianjian 明绣，规则 v160）：确认后置 2 轮冷
                // 却并对阶段归属者 1 火伤（target=机器分流带入）。
                ctx.core.player_mut(self.owner).linghuo_ready_round = ctx.core.turn.round + 2;
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
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
            HostBehavior::XianjianGuanri => match self.stage {
                // 贯日（U05 xianjian 云天河，规则 v161）：弃两张红手牌
                //（必选）→ 失去 1 点体力 → 指定体力≥自己的其他角色受
                // 2 火伤，存活则弃其装备区所有牌。limited 一次性
                //（guanri_used）。每臂只收一个应答。
                HostStage::SelectCard => {
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
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUANRI_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    for c in &picked {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked,
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            *t != self.owner
                                && ctx.core.player(*t).hp >= ctx.core.player(self.owner).hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_GUANRI_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（target 为空）：置一次性标记+失 1 体力+对目标
                    // 2 火伤；Yield 后结「存活则弃其装备区所有牌」
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        ctx.core.player_mut(self.owner).guanri_used = true;
                        let t = match ctx.answer().first() {
                            Some(ChoiceOption::Player(p)) => Some(*p),
                            _ => None,
                        };
                        self.cards.clear();
                        if let Some(t) = t {
                            self.cards = vec![t as CardId];
                        }
                        ctx.emit(EventData::LoseHp {
                            player: self.owner,
                            amount: 1,
                        });
                        if let Some(t) = self.cards.first().map(|x| *x as PlayerId) {
                            if ctx.core.player(t).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(self.owner),
                                    t,
                                    2,
                                    crate::event::DamageNature::Fire,
                                    None,
                                )));
                            }
                        }
                        return Flow::Yield;
                    }
                    // 结算臂：目标存活则弃其装备区所有牌
                    if let Some(t) = self.cards.first().map(|x| *x as PlayerId) {
                        self.cards.clear();
                        if ctx.core.player(t).alive && !ctx.core.player(t).equips.is_empty() {
                            let equips: Vec<CardId> = ctx.core.player(t).equips.clone();
                            for c in equips {
                                ctx.core.player_mut(t).equips.retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianDuci => match self.stage {
                // 毒刺（U05 xianjian 雪见，规则 v165）：弃装备牌入弃堆后
                // 令距离 1 内一名其他角色受 1 伤害（chooseTarget 取消=放
                // 弃；登记差异：仅弃置路径触发）。首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).alive
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
                            prompt: WUDIAN_XJ_DUCI_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
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
            HostBehavior::XianjianHushaCount => {
                // 虎煞·集（U05 xianjian 王小虎，规则 v169）：+伤害量标记
                //（≤3；上游 husha_count forced 同口径）。
                let amount = match ctx.res.frame(ctx.current_event_frame_id()) {
                    Some(f) => match &f.data {
                        EventData::Damage(d) => d.amount,
                        _ => 0,
                    },
                    None => 0,
                } as u8;
                let cur = ctx.core.player(self.owner).husha_mark;
                ctx.core.player_mut(self.owner).husha_mark = (cur + amount).min(3);
                Flow::Done
            }
            HostBehavior::XianjianZhangmu => {
                // 张目（U05 xianjian 南宫煌，规则 v181）：需要打出闪响应
                // 时弃一张手牌闪直接视为已响应（杀不命中=前置直防；弃哪
                // 张闪确定性取首张闪=登记差异；每回合 1 次）。
                ctx.core.turn.zhangmu_used = true;
                let Some(shan) = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan)
                else {
                    return Flow::Done;
                };
                ctx.discard_from_hand(self.owner, shan);
                // 当前帧=杀伤害前置窗口：直写 prevented（诚信同款）
                {
                    let frame = ctx
                        .res
                        .frame_mut(ctx.current_event_frame_id())
                        .expect("frame");
                    if let EventData::Damage(d) = &mut frame.data {
                        d.prevented = true;
                    }
                }
                ctx.log(LogEvent::DamagePrevented { target: self.owner });
                Flow::Done
            }
            HostBehavior::XianjianLuanjian2 => {
                // 乱剑·增伤：50% 伤害+1（上游 Math.random()<0.5 同 RNG
                // 流；DamageBefore 帧直写 amount=张目 prevented 同款机
                // 制）；每目标独立 roll（filter 已过=载体锚定成立）。
                if ctx.core.rng.gen_range(100) < 50 {
                    let frame = ctx
                        .res
                        .frame_mut(ctx.current_event_frame_id())
                        .expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        if let Some(amount) = dm.amount.checked_add(1) {
                            dm.amount = amount;
                        } else {
                            ctx.fail_damage_amount();
                            return Flow::Done;
                        }
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianCtianfu2 => {
                // 天符·反噬（ctianfu2，规则 v188）：挂附者造成伤害后——
                // 受来自挂附来源的 1 点雷伤（来源亡=无来源）+随机弃 he
                // 一张+挂附闪入弃堆+清挂附（触发即清=链天然终止）。
                let src = ctx.core.player(self.owner).ctianfu_source;
                let Some(src) = src else { return Flow::Done };
                let src_alive = src < ctx.core.players.len() as u8 && ctx.core.player(src).alive;
                let nature = crate::event::DamageNature::Thunder;
                let (sid, dmg_src) = if src_alive {
                    (Some(src), Some(src))
                } else {
                    (None, None)
                };
                let _ = sid;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    dmg_src, self.owner, 1, nature, None,
                )));
                // 随机弃 he 一张（上游 he.randomGet 同 RNG）
                let pool: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .chain(ctx.core.player(self.owner).equips.iter())
                    .copied()
                    .collect();
                if let Some(x) = pool.get(ctx.core.rng.gen_range(pool.len() as u32) as usize) {
                    let x = *x;
                    if ctx.core.player(self.owner).hand.contains(&x) {
                        ctx.discard_from_hand(self.owner, x);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|y| *y != x);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, x);
                        ctx.to_discard(x);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![x],
                    });
                }
                // 挂附闪入弃堆+清挂附+清实例
                ctianfu_clear(ctx, self.owner);
                if let Some(def) = ctx.catalog.skill_by_name("ctianfu2") {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
